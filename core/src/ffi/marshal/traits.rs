//! Core traits for FFI marshaling.
//!
//! This module defines the fundamental traits for converting Rust types to and from
//! C-compatible representations that can cross the FFI boundary safely.
//!
//! # Architecture
//!
//! The marshaling system uses a two-tier strategy:
//!
//! 1. **Direct C Marshaling**: For small, simple types that can be represented as
//!    C-compatible structs (primitives, simple structs, arrays with known bounds).
//!    Uses [`FfiMarshaler`] trait with zero-copy or minimal allocation.
//!
//! 2. **Streaming Marshaling**: For large data (>1MB) that should be transferred
//!    in chunks to avoid memory pressure. Uses [`FfiStreamMarshaler`] trait with
//!    fixed chunk sizes.
//!
//! # Design Patterns
//!
//! Following established patterns from [`FfiExportable`](crate::ffi::traits::FfiExportable):
//!
//! - **Associated Types**: Use `type CRepr` to define C representation
//! - **Result Types**: All conversions return [`FfiResult`](crate::ffi::error::FfiResult)
//! - **Size Awareness**: Types can opt into streaming based on size
//! - **Thread Safety**: All marshalers must be `Send + Sync`
//!
//! # Example
//!
//! ```
//! # use keyrx_core::ffi::marshal::traits::{FfiMarshaler, CRepr};
//! # use keyrx_core::ffi::error::FfiResult;
//! #
//! // C-compatible representation
//! #[repr(C)]
//! #[derive(Copy, Clone)]
//! struct DeviceInfoC {
//!     vendor_id: u16,
//!     product_id: u16,
//!     name: [u8; 256], // Fixed-size buffer
//! }
//!
//! // Implement CRepr marker
//! impl CRepr for DeviceInfoC {}
//!
//! // Rust type
//! struct DeviceInfo {
//!     vendor_id: u16,
//!     product_id: u16,
//!     name: String,
//! }
//!
//! impl FfiMarshaler for DeviceInfo {
//!     type CRepr = DeviceInfoC;
//!
//!     fn to_c(&self) -> FfiResult<Self::CRepr> {
//!         let mut name_buf = [0u8; 256];
//!         let bytes = self.name.as_bytes();
//!         let len = bytes.len().min(255); // Reserve 1 byte for null terminator
//!         name_buf[..len].copy_from_slice(&bytes[..len]);
//!
//!         Ok(DeviceInfoC {
//!             vendor_id: self.vendor_id,
//!             product_id: self.product_id,
//!             name: name_buf,
//!         })
//!     }
//!
//!     fn from_c(c: Self::CRepr) -> FfiResult<Self> {
//!         // Find null terminator
//!         let len = c.name.iter().position(|&b| b == 0).unwrap_or(256);
//!         let name = String::from_utf8_lossy(&c.name[..len]).into_owned();
//!
//!         Ok(DeviceInfo {
//!             vendor_id: c.vendor_id,
//!             product_id: c.product_id,
//!             name,
//!         })
//!     }
//!
//!     fn estimated_size(&self) -> usize {
//!         std::mem::size_of::<DeviceInfoC>()
//!     }
//! }
//! ```

use crate::ffi::error::FfiResult;

/// Streaming threshold: data larger than 1MB should use streaming.
pub const STREAMING_THRESHOLD: usize = 1024 * 1024;

/// Marker trait for C-compatible types.
///
/// Types implementing this trait can be safely passed across the FFI boundary.
/// They must be:
/// - `Copy`: No heap allocations or complex destructors
/// - `Send`: Safe to transfer across thread boundaries
/// - `'static`: No borrowed references
///
/// # Safety
///
/// Types implementing `CRepr` should typically be `#[repr(C)]` structs containing
/// only:
/// - Primitives (u8, u16, u32, u64, i8, i16, i32, i64, f32, f64, bool)
/// - Fixed-size arrays of primitives
/// - Other `#[repr(C)]` structs
///
/// # Example
///
/// ```
/// # use keyrx_core::ffi::marshal::traits::CRepr;
/// #[repr(C)]
/// #[derive(Copy, Clone)]
/// struct Point {
///     x: f32,
///     y: f32,
/// }
///
/// impl CRepr for Point {}
/// ```
pub trait CRepr: Copy + Send + 'static {}

/// Trait for types that can be marshaled across the FFI boundary.
///
/// This trait defines the contract for converting between Rust types and their
/// C-compatible representations. Implementations handle:
///
/// - **Serialization**: Converting Rust types to C structs via `to_c()`
/// - **Deserialization**: Converting C structs back to Rust types via `from_c()`
/// - **Size Estimation**: Predicting buffer sizes via `estimated_size()`
/// - **Streaming Decision**: Choosing between direct and streaming transfer
///
/// # Type Safety
///
/// The associated type `CRepr` must implement [`CRepr`], ensuring compile-time
/// verification that the C representation is safe to pass across FFI boundaries.
///
/// # Performance Considerations
///
/// - Small types (<1MB): Use direct C representation for minimal overhead
/// - Large types (>1MB): Use streaming to avoid memory pressure
/// - Override `use_streaming()` for custom size thresholds
///
/// # Error Handling
///
/// All conversion methods return [`FfiResult`] to handle:
/// - UTF-8 validation failures
/// - Buffer size mismatches
/// - Invalid data conversions
///
/// # Thread Safety
///
/// Implementors must be thread-safe. FFI calls may arrive on any thread, so
/// marshaling logic must not rely on thread-local state.
///
/// # Example
///
/// See module-level documentation for a complete example.
pub trait FfiMarshaler: Sized + Send + Sync {
    /// The C-compatible representation of this type.
    ///
    /// Must implement [`CRepr`] to ensure it's safe to pass across FFI boundaries.
    type CRepr: CRepr;

    /// Convert this Rust type to its C representation.
    ///
    /// This method performs serialization, which may involve:
    /// - Copying data into fixed-size buffers
    /// - Converting dynamic strings to null-terminated C strings
    /// - Flattening complex structures into flat C structs
    ///
    /// # Returns
    ///
    /// - `Ok(CRepr)`: Successfully converted to C representation
    /// - `Err(FfiError)`: Conversion failed (e.g., string too long, invalid data)
    ///
    /// # Errors
    ///
    /// Common error cases:
    /// - String data exceeds buffer size
    /// - Invalid UTF-8 in string fields
    /// - Numeric values out of C type range
    ///
    /// # Example
    ///
    /// ```
    /// # use keyrx_core::ffi::marshal::traits::{FfiMarshaler, CRepr};
    /// # use keyrx_core::ffi::error::{FfiResult, FfiError};
    /// # #[repr(C)]
    /// # #[derive(Copy, Clone)]
    /// # struct StatusC { code: u32 }
    /// # impl CRepr for StatusC {}
    /// # struct Status { code: u32 }
    /// # impl FfiMarshaler for Status {
    /// #     type CRepr = StatusC;
    /// #     fn from_c(_c: Self::CRepr) -> FfiResult<Self> { unimplemented!() }
    /// #     fn estimated_size(&self) -> usize { 4 }
    /// fn to_c(&self) -> FfiResult<StatusC> {
    ///     Ok(StatusC { code: self.code })
    /// }
    /// # }
    /// ```
    fn to_c(&self) -> FfiResult<Self::CRepr>;

    /// Convert from C representation to Rust type.
    ///
    /// This method performs deserialization, which may involve:
    /// - Validating buffer contents
    /// - Converting null-terminated strings to Rust Strings
    /// - Reconstructing complex structures from flat C structs
    ///
    /// # Parameters
    ///
    /// * `c` - The C representation to convert
    ///
    /// # Returns
    ///
    /// - `Ok(Self)`: Successfully converted to Rust type
    /// - `Err(FfiError)`: Conversion failed (e.g., invalid UTF-8, out of range)
    ///
    /// # Errors
    ///
    /// Common error cases:
    /// - Invalid UTF-8 in string buffers
    /// - Out-of-range numeric values
    /// - Malformed data structures
    ///
    /// # Example
    ///
    /// ```
    /// # use keyrx_core::ffi::marshal::traits::{FfiMarshaler, CRepr};
    /// # use keyrx_core::ffi::error::{FfiResult, FfiError};
    /// # #[repr(C)]
    /// # #[derive(Copy, Clone)]
    /// # struct StatusC { code: u32 }
    /// # impl CRepr for StatusC {}
    /// # struct Status { code: u32 }
    /// # impl FfiMarshaler for Status {
    /// #     type CRepr = StatusC;
    /// #     fn to_c(&self) -> FfiResult<Self::CRepr> { unimplemented!() }
    /// #     fn estimated_size(&self) -> usize { 4 }
    /// fn from_c(c: StatusC) -> FfiResult<Status> {
    ///     Ok(Status { code: c.code })
    /// }
    /// # }
    /// ```
    fn from_c(c: Self::CRepr) -> FfiResult<Self>;

    /// Estimate the size of the C representation in bytes.
    ///
    /// Used for:
    /// - Buffer allocation on the C side
    /// - Deciding between direct and streaming transfer
    /// - Memory usage tracking
    ///
    /// # Returns
    ///
    /// The estimated size in bytes. Should be:
    /// - Exact for fixed-size types
    /// - Conservative upper bound for variable-size types
    ///
    /// # Example
    ///
    /// ```
    /// # use keyrx_core::ffi::marshal::traits::{FfiMarshaler, CRepr};
    /// # use keyrx_core::ffi::error::FfiResult;
    /// # #[repr(C)]
    /// # #[derive(Copy, Clone)]
    /// # struct Point { x: f32, y: f32 }
    /// # impl CRepr for Point {}
    /// # struct Vector { x: f32, y: f32 }
    /// # impl FfiMarshaler for Vector {
    /// #     type CRepr = Point;
    /// #     fn to_c(&self) -> FfiResult<Self::CRepr> { unimplemented!() }
    /// #     fn from_c(_c: Self::CRepr) -> FfiResult<Self> { unimplemented!() }
    /// fn estimated_size(&self) -> usize {
    ///     std::mem::size_of::<Point>() // 8 bytes
    /// }
    /// # }
    /// ```
    fn estimated_size(&self) -> usize;

    /// Determine if this instance should use streaming transfer.
    ///
    /// Default implementation uses the [`STREAMING_THRESHOLD`] constant (1MB).
    /// Override for custom logic based on type-specific knowledge.
    ///
    /// # Returns
    ///
    /// - `true`: Use [`FfiStreamMarshaler`] for chunked transfer
    /// - `false`: Use direct C representation transfer
    ///
    /// # Example
    ///
    /// ```
    /// # use keyrx_core::ffi::marshal::traits::{FfiMarshaler, CRepr, STREAMING_THRESHOLD};
    /// # use keyrx_core::ffi::error::FfiResult;
    /// # #[repr(C)]
    /// # #[derive(Copy, Clone)]
    /// # struct DataC { size: usize }
    /// # impl CRepr for DataC {}
    /// # struct LargeData { buffer: Vec<u8> }
    /// # impl FfiMarshaler for LargeData {
    /// #     type CRepr = DataC;
    /// #     fn to_c(&self) -> FfiResult<Self::CRepr> { unimplemented!() }
    /// #     fn from_c(_c: Self::CRepr) -> FfiResult<Self> { unimplemented!() }
    /// #     fn estimated_size(&self) -> usize { self.buffer.len() }
    /// // Override for custom threshold
    /// fn use_streaming(&self) -> bool {
    ///     self.buffer.len() > 512 * 1024 // Custom 512KB threshold
    /// }
    /// # }
    /// ```
    fn use_streaming(&self) -> bool {
        self.estimated_size() > STREAMING_THRESHOLD
    }
}

/// Trait for types that support streaming marshaling for large data.
///
/// Large data (>1MB) should be transferred in fixed-size chunks to avoid:
/// - Excessive memory allocation
/// - Long blocking FFI calls
/// - UI freezes during transfer
///
/// # Architecture
///
/// Streaming works in three phases:
///
/// 1. **Rust Side**: Break data into fixed-size chunks via `get_chunk()`
/// 2. **FFI Boundary**: Transfer chunks one at a time with progress tracking
/// 3. **Dart Side**: Reassemble chunks via `from_chunks()`
///
/// # Chunk Size
///
/// Default chunk size is 64KB (good balance between overhead and throughput).
/// Override `chunk_size()` for custom sizes based on data characteristics.
///
/// # Resumability
///
/// The chunk-based design enables:
/// - Progress tracking (chunk N of M)
/// - Partial retry on failure (resume from failed chunk)
/// - Cancellation (stop after current chunk)
///
/// # Example
///
/// ```
/// # use keyrx_core::ffi::marshal::traits::{FfiStreamMarshaler, CRepr};
/// # use keyrx_core::ffi::error::FfiResult;
/// #
/// # #[repr(C)]
/// # #[derive(Copy, Clone)]
/// # struct ChunkC {
/// #     data: [u8; 64 * 1024],
/// #     len: usize,
/// # }
/// # impl CRepr for ChunkC {}
/// #
/// struct RecordingData {
///     frames: Vec<Vec<u8>>,
/// }
///
/// impl FfiStreamMarshaler for RecordingData {
///     type Chunk = ChunkC;
///
///     fn chunk_count(&self) -> usize {
///         let total_size: usize = self.frames.iter().map(|f| f.len()).sum();
///         (total_size + 64 * 1024 - 1) / (64 * 1024) // Ceiling division
///     }
///
///     fn get_chunk(&self, index: usize) -> FfiResult<Self::Chunk> {
///         // Implementation: collect data into chunk
///         # unimplemented!()
///     }
///
///     fn from_chunks(chunks: &[Self::Chunk]) -> FfiResult<Self> {
///         // Implementation: reassemble chunks into frames
///         # unimplemented!()
///     }
/// }
/// ```
pub trait FfiStreamMarshaler: Sized + Send + Sync {
    /// The C-compatible chunk type.
    ///
    /// Typically a fixed-size buffer struct with a length field:
    /// ```
    /// # use keyrx_core::ffi::marshal::traits::CRepr;
    /// #[repr(C)]
    /// #[derive(Copy, Clone)]
    /// struct Chunk {
    ///     data: [u8; 64 * 1024], // Fixed 64KB buffer
    ///     len: usize,             // Actual data length in this chunk
    /// }
    /// impl CRepr for Chunk {}
    /// ```
    type Chunk: CRepr;

    /// Total number of chunks needed to transfer this data.
    ///
    /// Used for:
    /// - Progress tracking (chunk N of chunk_count())
    /// - Memory allocation on receiving side
    /// - Validation (ensure all chunks received)
    ///
    /// # Returns
    ///
    /// The total number of chunks. Must be consistent across calls.
    fn chunk_count(&self) -> usize;

    /// Get a specific chunk by index.
    ///
    /// # Parameters
    ///
    /// * `index` - Zero-based chunk index (0 to chunk_count() - 1)
    ///
    /// # Returns
    ///
    /// - `Ok(Chunk)`: Chunk data successfully retrieved
    /// - `Err(FfiError)`: Invalid index or internal error
    ///
    /// # Errors
    ///
    /// - Index out of bounds (>= chunk_count())
    /// - Data corruption detected
    /// - Internal buffer errors
    fn get_chunk(&self, index: usize) -> FfiResult<Self::Chunk>;

    /// Reconstruct the original data from chunks.
    ///
    /// # Parameters
    ///
    /// * `chunks` - Slice of all chunks in order (length == chunk_count())
    ///
    /// # Returns
    ///
    /// - `Ok(Self)`: Data successfully reconstructed
    /// - `Err(FfiError)`: Invalid chunks, wrong count, or corruption detected
    ///
    /// # Errors
    ///
    /// - Wrong number of chunks
    /// - Chunk validation failure
    /// - Reconstruction logic failure (e.g., invalid framing)
    fn from_chunks(chunks: &[Self::Chunk]) -> FfiResult<Self>;

    /// Get the chunk size in bytes.
    ///
    /// Default is 64KB. Override for custom sizes.
    ///
    /// # Returns
    ///
    /// The size of each chunk in bytes (except possibly the last chunk).
    fn chunk_size(&self) -> usize {
        64 * 1024 // 64KB default
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::error::FfiError;

    // Test CRepr marker trait
    #[repr(C)]
    #[derive(Copy, Clone)]
    struct TestC {
        value: u32,
    }

    impl CRepr for TestC {}

    struct TestData {
        value: u32,
    }

    impl FfiMarshaler for TestData {
        type CRepr = TestC;

        fn to_c(&self) -> FfiResult<Self::CRepr> {
            Ok(TestC { value: self.value })
        }

        fn from_c(c: Self::CRepr) -> FfiResult<Self> {
            Ok(TestData { value: c.value })
        }

        fn estimated_size(&self) -> usize {
            std::mem::size_of::<TestC>()
        }
    }

    #[test]
    fn test_ffi_marshaler_roundtrip() {
        let data = TestData { value: 42 };
        let c_repr = data.to_c().unwrap();
        assert_eq!(c_repr.value, 42);

        let reconstructed = TestData::from_c(c_repr).unwrap();
        assert_eq!(reconstructed.value, 42);
    }

    #[test]
    fn test_estimated_size() {
        let data = TestData { value: 123 };
        assert_eq!(data.estimated_size(), std::mem::size_of::<TestC>());
    }

    #[test]
    fn test_use_streaming_default() {
        let small_data = TestData { value: 1 };
        assert!(!small_data.use_streaming()); // 4 bytes << 1MB

        // Test with mock large data
        struct LargeData;
        impl FfiMarshaler for LargeData {
            type CRepr = TestC;
            fn to_c(&self) -> FfiResult<Self::CRepr> {
                Ok(TestC { value: 0 })
            }
            fn from_c(_c: Self::CRepr) -> FfiResult<Self> {
                Ok(LargeData)
            }
            fn estimated_size(&self) -> usize {
                2 * 1024 * 1024 // 2MB
            }
        }

        let large = LargeData;
        assert!(large.use_streaming()); // 2MB > 1MB threshold
    }

    #[test]
    fn test_streaming_threshold_constant() {
        assert_eq!(STREAMING_THRESHOLD, 1024 * 1024);
    }

    // Test streaming marshaler
    #[repr(C)]
    #[derive(Copy, Clone)]
    struct ChunkC {
        data: [u8; 4],
        len: usize,
    }

    impl CRepr for ChunkC {}

    struct StreamingData {
        bytes: Vec<u8>,
    }

    impl FfiStreamMarshaler for StreamingData {
        type Chunk = ChunkC;

        fn chunk_count(&self) -> usize {
            (self.bytes.len() + 3) / 4 // Ceiling division for 4-byte chunks
        }

        fn get_chunk(&self, index: usize) -> FfiResult<Self::Chunk> {
            if index >= self.chunk_count() {
                return Err(FfiError::invalid_input("chunk index out of bounds"));
            }

            let start = index * 4;
            let end = (start + 4).min(self.bytes.len());
            let chunk_len = end - start;

            let mut data = [0u8; 4];
            data[..chunk_len].copy_from_slice(&self.bytes[start..end]);

            Ok(ChunkC {
                data,
                len: chunk_len,
            })
        }

        fn from_chunks(chunks: &[Self::Chunk]) -> FfiResult<Self> {
            let mut bytes = Vec::new();
            for chunk in chunks {
                bytes.extend_from_slice(&chunk.data[..chunk.len]);
            }
            Ok(StreamingData { bytes })
        }
    }

    #[test]
    fn test_streaming_marshaler() {
        let data = StreamingData {
            bytes: vec![1, 2, 3, 4, 5, 6, 7],
        };

        // Should need 2 chunks (7 bytes / 4 = 2 ceiling)
        assert_eq!(data.chunk_count(), 2);

        // Get first chunk
        let chunk0 = data.get_chunk(0).unwrap();
        assert_eq!(chunk0.len, 4);
        assert_eq!(&chunk0.data[..4], &[1, 2, 3, 4]);

        // Get second chunk
        let chunk1 = data.get_chunk(1).unwrap();
        assert_eq!(chunk1.len, 3);
        assert_eq!(&chunk1.data[..3], &[5, 6, 7]);

        // Out of bounds should error
        assert!(data.get_chunk(2).is_err());
    }

    #[test]
    fn test_streaming_roundtrip() {
        let original = StreamingData {
            bytes: vec![10, 20, 30, 40, 50],
        };

        // Collect all chunks
        let chunk_count = original.chunk_count();
        let mut chunks = Vec::new();
        for i in 0..chunk_count {
            chunks.push(original.get_chunk(i).unwrap());
        }

        // Reconstruct
        let reconstructed = StreamingData::from_chunks(&chunks).unwrap();
        assert_eq!(reconstructed.bytes, original.bytes);
    }

    #[test]
    fn test_chunk_size_default() {
        let data = StreamingData { bytes: vec![1, 2] };
        assert_eq!(data.chunk_size(), 64 * 1024);
    }
}
