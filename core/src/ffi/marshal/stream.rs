//! Streaming utilities for large data transfer across FFI boundaries.
//!
//! This module provides helper types and utilities for implementing chunked
//! streaming transfer of large data (>1MB) to avoid memory pressure and
//! blocking FFI calls.
//!
//! # Architecture
//!
//! Streaming works in three phases:
//!
//! 1. **Chunking**: Break large data into fixed-size chunks using [`ChunkIterator`]
//! 2. **Transfer**: Transfer chunks one at a time across FFI boundary
//! 3. **Reassembly**: Reconstruct data using [`ChunkCollector`]
//!
//! # Key Types
//!
//! - [`ChunkIterator`]: Iterator for breaking data into chunks
//! - [`ChunkCollector`]: Stateful collector for reassembling chunks
//! - [`StreamContext`]: Metadata for streaming operations
//!
//! # Example
//!
//! ```
//! use keyrx_core::ffi::marshal::stream::{ChunkIterator, ChunkCollector};
//!
//! // Break data into chunks
//! let data = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
//! let mut iter = ChunkIterator::new(&data, 3); // 3-byte chunks
//!
//! // Collect chunks
//! let mut collector = ChunkCollector::new();
//! while let Some(chunk) = iter.next() {
//!     collector.add_chunk(chunk);
//! }
//!
//! // Reconstruct
//! let reconstructed = collector.into_vec();
//! assert_eq!(reconstructed, data);
//! ```

use crate::ffi::error::FfiResult;
use crate::ffi::marshal::traits::CRepr;

/// Default chunk size for streaming: 64KB.
///
/// This is a good balance between:
/// - **Low overhead**: Fewer FFI calls
/// - **Responsive**: Small enough to allow progress tracking
/// - **Memory**: Reasonable buffer size for embedded systems
pub const DEFAULT_CHUNK_SIZE: usize = 64 * 1024;

/// Maximum chunk size: 1MB.
///
/// Larger chunks defeat the purpose of streaming (memory pressure).
pub const MAX_CHUNK_SIZE: usize = 1024 * 1024;

/// Iterator for breaking byte slices into fixed-size chunks.
///
/// This iterator is useful for implementing [`FfiStreamMarshaler::get_chunk`]
/// by providing a zero-copy view into the underlying data.
///
/// # Example
///
/// ```
/// use keyrx_core::ffi::marshal::stream::ChunkIterator;
///
/// let data = vec![1, 2, 3, 4, 5, 6, 7];
/// let mut iter = ChunkIterator::new(&data, 3);
///
/// assert_eq!(iter.next(), Some(&[1, 2, 3][..]));
/// assert_eq!(iter.next(), Some(&[4, 5, 6][..]));
/// assert_eq!(iter.next(), Some(&[7][..])); // Last chunk may be smaller
/// assert_eq!(iter.next(), None);
/// ```
#[derive(Debug, Clone)]
pub struct ChunkIterator<'a> {
    /// Remaining data to chunk
    data: &'a [u8],
    /// Size of each chunk in bytes
    chunk_size: usize,
    /// Current chunk index (for debugging)
    current_index: usize,
    /// Original data length (for total_chunks calculation)
    original_len: usize,
}

impl<'a> ChunkIterator<'a> {
    /// Create a new chunk iterator.
    ///
    /// # Parameters
    ///
    /// * `data` - The data to chunk
    /// * `chunk_size` - Size of each chunk in bytes
    ///
    /// # Panics
    ///
    /// Panics if `chunk_size` is 0 or exceeds [`MAX_CHUNK_SIZE`].
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::ffi::marshal::stream::ChunkIterator;
    ///
    /// let data = vec![1, 2, 3, 4];
    /// let iter = ChunkIterator::new(&data, 2);
    /// ```
    pub fn new(data: &'a [u8], chunk_size: usize) -> Self {
        assert!(chunk_size > 0, "Chunk size must be greater than 0");
        assert!(
            chunk_size <= MAX_CHUNK_SIZE,
            "Chunk size must not exceed {} bytes",
            MAX_CHUNK_SIZE
        );

        Self {
            data,
            chunk_size,
            current_index: 0,
            original_len: data.len(),
        }
    }

    /// Get the total number of chunks.
    ///
    /// # Returns
    ///
    /// The total number of chunks (ceiling division).
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::ffi::marshal::stream::ChunkIterator;
    ///
    /// let data = vec![1, 2, 3, 4, 5];
    /// let iter = ChunkIterator::new(&data, 2);
    /// assert_eq!(iter.total_chunks(), 3); // 5 bytes / 2 = 3 chunks
    /// ```
    pub fn total_chunks(&self) -> usize {
        if self.original_len == 0 {
            0
        } else {
            self.original_len.div_ceil(self.chunk_size)
        }
    }

    /// Get the current chunk index.
    ///
    /// # Returns
    ///
    /// The zero-based index of the next chunk to be returned.
    pub fn current_index(&self) -> usize {
        self.current_index
    }

    /// Get the chunk size.
    pub fn chunk_size(&self) -> usize {
        self.chunk_size
    }
}

impl<'a> Iterator for ChunkIterator<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.data.is_empty() {
            return None;
        }

        let chunk_size = self.chunk_size.min(self.data.len());
        let (chunk, rest) = self.data.split_at(chunk_size);
        self.data = rest;
        self.current_index += 1;

        Some(chunk)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let chunks = self.total_chunks();
        (chunks, Some(chunks))
    }
}

impl ExactSizeIterator for ChunkIterator<'_> {
    fn len(&self) -> usize {
        self.total_chunks()
    }
}

/// Collector for reassembling data from chunks.
///
/// This type is useful for implementing [`FfiStreamMarshaler::from_chunks`]
/// by providing a stateful buffer that collects chunks.
///
/// # Example
///
/// ```
/// use keyrx_core::ffi::marshal::stream::ChunkCollector;
///
/// let mut collector = ChunkCollector::new();
/// collector.add_chunk(&[1, 2, 3]);
/// collector.add_chunk(&[4, 5]);
///
/// let data = collector.into_vec();
/// assert_eq!(data, vec![1, 2, 3, 4, 5]);
/// ```
#[derive(Debug, Clone)]
pub struct ChunkCollector {
    /// Collected data buffer
    buffer: Vec<u8>,
    /// Number of chunks collected
    chunk_count: usize,
}

impl ChunkCollector {
    /// Create a new chunk collector.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::ffi::marshal::stream::ChunkCollector;
    ///
    /// let collector = ChunkCollector::new();
    /// ```
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            chunk_count: 0,
        }
    }

    /// Create a new chunk collector with a capacity hint.
    ///
    /// # Parameters
    ///
    /// * `capacity` - Expected total size in bytes
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::ffi::marshal::stream::ChunkCollector;
    ///
    /// // Pre-allocate for 1MB of data
    /// let collector = ChunkCollector::with_capacity(1024 * 1024);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            chunk_count: 0,
        }
    }

    /// Add a chunk to the collector.
    ///
    /// # Parameters
    ///
    /// * `chunk` - The chunk data to add
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::ffi::marshal::stream::ChunkCollector;
    ///
    /// let mut collector = ChunkCollector::new();
    /// collector.add_chunk(&[1, 2, 3]);
    /// ```
    pub fn add_chunk(&mut self, chunk: &[u8]) {
        self.buffer.extend_from_slice(chunk);
        self.chunk_count += 1;
    }

    /// Get the number of chunks collected so far.
    ///
    /// # Returns
    ///
    /// The number of chunks added via [`add_chunk`](Self::add_chunk).
    pub fn chunk_count(&self) -> usize {
        self.chunk_count
    }

    /// Get the total size of collected data in bytes.
    ///
    /// # Returns
    ///
    /// The total number of bytes collected.
    pub fn total_size(&self) -> usize {
        self.buffer.len()
    }

    /// Get a reference to the collected data.
    ///
    /// # Returns
    ///
    /// A slice of all collected data so far.
    pub fn as_slice(&self) -> &[u8] {
        &self.buffer
    }

    /// Consume the collector and return the collected data.
    ///
    /// # Returns
    ///
    /// A `Vec<u8>` containing all collected data.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::ffi::marshal::stream::ChunkCollector;
    ///
    /// let mut collector = ChunkCollector::new();
    /// collector.add_chunk(&[1, 2, 3]);
    /// let data = collector.into_vec();
    /// assert_eq!(data, vec![1, 2, 3]);
    /// ```
    pub fn into_vec(self) -> Vec<u8> {
        self.buffer
    }

    /// Clear the collector, removing all collected data.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::ffi::marshal::stream::ChunkCollector;
    ///
    /// let mut collector = ChunkCollector::new();
    /// collector.add_chunk(&[1, 2, 3]);
    /// collector.clear();
    /// assert_eq!(collector.total_size(), 0);
    /// ```
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.chunk_count = 0;
    }
}

impl Default for ChunkCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Metadata for streaming operations.
///
/// This type provides information about a streaming transfer, useful for
/// progress tracking, validation, and resumption.
///
/// # Example
///
/// ```
/// use keyrx_core::ffi::marshal::stream::StreamContext;
///
/// let ctx = StreamContext::new(100, 10);
/// assert_eq!(ctx.total_chunks(), 10);
/// assert_eq!(ctx.chunk_size(), 10);
/// ```
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct StreamContext {
    /// Total size of data in bytes
    total_size: u64,
    /// Number of chunks
    chunk_count: u32,
    /// Size of each chunk (except possibly the last)
    chunk_size: u32,
}

impl CRepr for StreamContext {}

impl StreamContext {
    /// Create a new stream context.
    ///
    /// # Parameters
    ///
    /// * `total_size` - Total size of data in bytes
    /// * `chunk_count` - Number of chunks
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::ffi::marshal::stream::StreamContext;
    ///
    /// let ctx = StreamContext::new(1024, 2);
    /// assert_eq!(ctx.total_size(), 1024);
    /// ```
    pub fn new(total_size: u64, chunk_count: u32) -> Self {
        let chunk_size = if chunk_count > 0 {
            total_size.div_ceil(chunk_count as u64) as u32
        } else {
            0
        };

        Self {
            total_size,
            chunk_count,
            chunk_size,
        }
    }

    /// Create from data size and chunk size.
    ///
    /// # Parameters
    ///
    /// * `total_size` - Total size of data in bytes
    /// * `chunk_size` - Size of each chunk in bytes
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::ffi::marshal::stream::StreamContext;
    ///
    /// let ctx = StreamContext::from_chunk_size(1024, 512);
    /// assert_eq!(ctx.chunk_count(), 2);
    /// ```
    pub fn from_chunk_size(total_size: u64, chunk_size: u32) -> Self {
        let chunk_count = if chunk_size > 0 {
            total_size.div_ceil(chunk_size as u64) as u32
        } else {
            0
        };

        Self {
            total_size,
            chunk_count,
            chunk_size,
        }
    }

    /// Get the total size in bytes.
    pub fn total_size(&self) -> u64 {
        self.total_size
    }

    /// Get the number of chunks.
    pub fn total_chunks(&self) -> u32 {
        self.chunk_count
    }

    /// Get the chunk size in bytes.
    pub fn chunk_size(&self) -> u32 {
        self.chunk_size
    }

    /// Calculate the size of a specific chunk.
    ///
    /// # Parameters
    ///
    /// * `index` - Zero-based chunk index
    ///
    /// # Returns
    ///
    /// The size of the chunk in bytes, or `None` if index is out of bounds.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::ffi::marshal::stream::StreamContext;
    ///
    /// let ctx = StreamContext::from_chunk_size(10, 3); // 4 chunks: 3,3,3,1
    /// assert_eq!(ctx.chunk_actual_size(0), Some(3));
    /// assert_eq!(ctx.chunk_actual_size(3), Some(1)); // Last chunk is smaller
    /// assert_eq!(ctx.chunk_actual_size(4), None);    // Out of bounds
    /// ```
    pub fn chunk_actual_size(&self, index: u32) -> Option<u32> {
        if index >= self.chunk_count {
            return None;
        }

        if index == self.chunk_count - 1 {
            // Last chunk may be smaller
            let previous_chunks_size = (self.chunk_count - 1) as u64 * self.chunk_size as u64;
            Some((self.total_size - previous_chunks_size) as u32)
        } else {
            Some(self.chunk_size)
        }
    }

    /// Validate that a chunk index is in bounds.
    ///
    /// # Parameters
    ///
    /// * `index` - Zero-based chunk index
    ///
    /// # Returns
    ///
    /// `Ok(())` if valid, `Err(FfiError)` if out of bounds.
    pub fn validate_chunk_index(&self, index: u32) -> FfiResult<()> {
        if index >= self.chunk_count {
            Err(crate::ffi::error::FfiError::invalid_input(format!(
                "Chunk index {} out of bounds (total: {})",
                index, self.chunk_count
            )))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_iterator_basic() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let mut iter = ChunkIterator::new(&data, 3);

        assert_eq!(iter.next(), Some(&[1, 2, 3][..]));
        assert_eq!(iter.next(), Some(&[4, 5, 6][..]));
        assert_eq!(iter.next(), Some(&[7, 8][..]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_chunk_iterator_exact_size() {
        let data = vec![1, 2, 3, 4, 5, 6];
        let iter = ChunkIterator::new(&data, 2);

        assert_eq!(iter.len(), 3);
        assert_eq!(iter.total_chunks(), 3);
    }

    #[test]
    fn test_chunk_iterator_empty() {
        let data: Vec<u8> = vec![];
        let mut iter = ChunkIterator::new(&data, 10);

        assert_eq!(iter.next(), None);
        assert_eq!(iter.total_chunks(), 0);
    }

    #[test]
    fn test_chunk_iterator_single_chunk() {
        let data = vec![1, 2, 3];
        let mut iter = ChunkIterator::new(&data, 10);

        assert_eq!(iter.next(), Some(&[1, 2, 3][..]));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.total_chunks(), 1);
    }

    #[test]
    fn test_chunk_iterator_index_tracking() {
        let data = vec![1, 2, 3, 4];
        let mut iter = ChunkIterator::new(&data, 2);

        assert_eq!(iter.current_index(), 0);
        iter.next();
        assert_eq!(iter.current_index(), 1);
        iter.next();
        assert_eq!(iter.current_index(), 2);
    }

    #[test]
    #[should_panic(expected = "Chunk size must be greater than 0")]
    fn test_chunk_iterator_zero_size() {
        let data = vec![1, 2, 3];
        let _iter = ChunkIterator::new(&data, 0);
    }

    #[test]
    #[should_panic(expected = "Chunk size must not exceed")]
    fn test_chunk_iterator_too_large() {
        let data = vec![1, 2, 3];
        let _iter = ChunkIterator::new(&data, MAX_CHUNK_SIZE + 1);
    }

    #[test]
    fn test_chunk_collector_basic() {
        let mut collector = ChunkCollector::new();

        collector.add_chunk(&[1, 2, 3]);
        collector.add_chunk(&[4, 5]);

        assert_eq!(collector.chunk_count(), 2);
        assert_eq!(collector.total_size(), 5);
        assert_eq!(collector.as_slice(), &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_chunk_collector_into_vec() {
        let mut collector = ChunkCollector::new();
        collector.add_chunk(&[1, 2, 3]);

        let vec = collector.into_vec();
        assert_eq!(vec, vec![1, 2, 3]);
    }

    #[test]
    fn test_chunk_collector_with_capacity() {
        let collector = ChunkCollector::with_capacity(1024);
        assert_eq!(collector.total_size(), 0);
        assert!(collector.buffer.capacity() >= 1024);
    }

    #[test]
    fn test_chunk_collector_clear() {
        let mut collector = ChunkCollector::new();
        collector.add_chunk(&[1, 2, 3]);

        collector.clear();
        assert_eq!(collector.chunk_count(), 0);
        assert_eq!(collector.total_size(), 0);
    }

    #[test]
    fn test_chunk_collector_default() {
        let collector = ChunkCollector::default();
        assert_eq!(collector.total_size(), 0);
    }

    #[test]
    fn test_stream_context_new() {
        let ctx = StreamContext::new(1024, 10);

        assert_eq!(ctx.total_size(), 1024);
        assert_eq!(ctx.total_chunks(), 10);
        // Ceiling division: (1024 + 10 - 1) / 10 = 103
        assert_eq!(ctx.chunk_size(), 103);
    }

    #[test]
    fn test_stream_context_from_chunk_size() {
        let ctx = StreamContext::from_chunk_size(1024, 512);

        assert_eq!(ctx.total_size(), 1024);
        assert_eq!(ctx.total_chunks(), 2);
        assert_eq!(ctx.chunk_size(), 512);
    }

    #[test]
    fn test_stream_context_chunk_actual_size() {
        let ctx = StreamContext::from_chunk_size(10, 3); // 4 chunks: 3,3,3,1

        assert_eq!(ctx.chunk_actual_size(0), Some(3));
        assert_eq!(ctx.chunk_actual_size(1), Some(3));
        assert_eq!(ctx.chunk_actual_size(2), Some(3));
        assert_eq!(ctx.chunk_actual_size(3), Some(1)); // Last chunk smaller
        assert_eq!(ctx.chunk_actual_size(4), None); // Out of bounds
    }

    #[test]
    fn test_stream_context_validate_chunk_index() {
        let ctx = StreamContext::new(100, 5);

        assert!(ctx.validate_chunk_index(0).is_ok());
        assert!(ctx.validate_chunk_index(4).is_ok());
        assert!(ctx.validate_chunk_index(5).is_err());
        assert!(ctx.validate_chunk_index(100).is_err());
    }

    #[test]
    fn test_stream_context_zero_chunks() {
        let ctx = StreamContext::new(0, 0);

        assert_eq!(ctx.total_size(), 0);
        assert_eq!(ctx.total_chunks(), 0);
        assert_eq!(ctx.chunk_size(), 0);
    }

    #[test]
    fn test_roundtrip_with_iterator_and_collector() {
        let original_data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let chunk_size = 3;

        // Break into chunks
        let iter = ChunkIterator::new(&original_data, chunk_size);

        // Collect chunks
        let mut collector = ChunkCollector::new();
        for chunk in iter {
            collector.add_chunk(chunk);
        }

        // Verify roundtrip
        let reconstructed = collector.into_vec();
        assert_eq!(reconstructed, original_data);
    }

    #[test]
    fn test_constants() {
        assert_eq!(DEFAULT_CHUNK_SIZE, 64 * 1024);
        assert_eq!(MAX_CHUNK_SIZE, 1024 * 1024);
        assert!(DEFAULT_CHUNK_SIZE < MAX_CHUNK_SIZE);
    }
}
