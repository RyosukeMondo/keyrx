//! Streaming marshalers for session data (recording/replay).
//!
//! This module provides efficient streaming support for large session data,
//! including:
//!
//! - **SessionFile**: Complete recording sessions with many events
//! - **DiscoverySummary**: Discovery session results with large keymaps
//!
//! These types can contain large amounts of data (>1MB for long sessions),
//! so we use chunked streaming to avoid memory pressure and blocking FFI calls.
//!
//! # Architecture
//!
//! For session data, we use JSON serialization with streaming:
//!
//! 1. **Serialize** the session data to JSON
//! 2. **Chunk** the JSON bytes using [`ChunkIterator`]
//! 3. **Transfer** chunks across FFI boundary
//! 4. **Reassemble** using [`ChunkCollector`]
//! 5. **Deserialize** JSON back to session data
//!
//! This approach balances:
//! - **Simplicity**: Reuse existing serde infrastructure
//! - **Efficiency**: Stream large JSON payloads in chunks
//! - **Compatibility**: Standard JSON format for debugging
//!
//! # Example
//!
//! ```no_run
//! use keyrx_core::ffi::marshal::impls::session::SessionFileStreamer;
//! use keyrx_core::ffi::marshal::stream::{ChunkIterator, ChunkCollector};
//! use keyrx_core::engine::event_recording::SessionFile;
//! use keyrx_core::engine::{EngineState, TimingConfig};
//!
//! // Create a session with many events
//! let session = SessionFile::new(
//!     "2024-01-15T10:30:00Z".to_string(),
//!     None,
//!     TimingConfig::default(),
//!     EngineState::default(),
//! );
//!
//! // Stream it
//! let streamer = SessionFileStreamer::new(session);
//! let json_bytes = streamer.to_json_bytes().unwrap();
//!
//! // Chunk for FFI transfer
//! let mut iter = ChunkIterator::new(&json_bytes, 64 * 1024);
//! let mut collector = ChunkCollector::new();
//!
//! while let Some(chunk) = iter.next() {
//!     // Transfer chunk across FFI...
//!     collector.add_chunk(chunk);
//! }
//!
//! // Reconstruct
//! let reconstructed = SessionFileStreamer::from_json_bytes(&collector.into_vec()).unwrap();
//! ```

use crate::discovery::session::DiscoverySummary;
use crate::engine::SessionFile;
use crate::ffi::error::{FfiError, FfiResult};
use crate::ffi::marshal::stream::{ChunkCollector, ChunkIterator, DEFAULT_CHUNK_SIZE};

/// Streaming wrapper for [`SessionFile`].
///
/// Provides efficient serialization and chunked streaming for large recording sessions.
///
/// # Memory Efficiency
///
/// A session with 10,000 events can easily exceed 10MB when serialized.
/// By streaming in 64KB chunks, we:
///
/// - Avoid blocking FFI calls
/// - Reduce peak memory usage
/// - Enable progress tracking
///
/// # Example
///
/// ```no_run
/// use keyrx_core::ffi::marshal::impls::session::SessionFileStreamer;
/// use keyrx_core::engine::event_recording::SessionFile;
/// use keyrx_core::engine::{EngineState, TimingConfig};
///
/// let session = SessionFile::new(
///     "2024-01-15T10:30:00Z".to_string(),
///     None,
///     TimingConfig::default(),
///     EngineState::default(),
/// );
///
/// let streamer = SessionFileStreamer::new(session);
/// let json = streamer.to_json_bytes().unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct SessionFileStreamer {
    session: SessionFile,
}

impl SessionFileStreamer {
    /// Create a new streamer for a session file.
    ///
    /// # Parameters
    ///
    /// * `session` - The session file to stream
    ///
    /// # Example
    ///
    /// ```no_run
    /// use keyrx_core::ffi::marshal::impls::session::SessionFileStreamer;
    /// use keyrx_core::engine::event_recording::SessionFile;
    /// use keyrx_core::engine::{EngineState, TimingConfig};
    ///
    /// let session = SessionFile::new(
    ///     "2024-01-15T10:30:00Z".to_string(),
    ///     None,
    ///     TimingConfig::default(),
    ///     EngineState::default(),
    /// );
    /// let streamer = SessionFileStreamer::new(session);
    /// ```
    pub fn new(session: SessionFile) -> Self {
        Self { session }
    }

    /// Get a reference to the underlying session.
    pub fn session(&self) -> &SessionFile {
        &self.session
    }

    /// Consume the streamer and return the underlying session.
    pub fn into_session(self) -> SessionFile {
        self.session
    }

    /// Serialize the session to JSON bytes.
    ///
    /// # Returns
    ///
    /// A `Vec<u8>` containing the JSON representation.
    ///
    /// # Errors
    ///
    /// Returns [`FfiError`] if serialization fails.
    pub fn to_json_bytes(&self) -> FfiResult<Vec<u8>> {
        serde_json::to_vec(&self.session).map_err(|e| {
            FfiError::serialization_failed(format!("Failed to serialize SessionFile: {}", e))
        })
    }

    /// Deserialize a session from JSON bytes.
    ///
    /// # Parameters
    ///
    /// * `bytes` - JSON bytes to deserialize
    ///
    /// # Returns
    ///
    /// A new `SessionFileStreamer` containing the deserialized session.
    ///
    /// # Errors
    ///
    /// Returns [`FfiError`] if deserialization fails.
    pub fn from_json_bytes(bytes: &[u8]) -> FfiResult<Self> {
        let session = serde_json::from_slice(bytes).map_err(|e| {
            FfiError::deserialization_failed(format!("Failed to deserialize SessionFile: {}", e))
        })?;
        Ok(Self { session })
    }

    /// Create a chunk iterator for streaming the session data.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `ChunkIterator` or serialization error.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use keyrx_core::ffi::marshal::impls::session::SessionFileStreamer;
    /// use keyrx_core::engine::event_recording::SessionFile;
    /// use keyrx_core::engine::{EngineState, TimingConfig};
    ///
    /// let session = SessionFile::new(
    ///     "2024-01-15T10:30:00Z".to_string(),
    ///     None,
    ///     TimingConfig::default(),
    ///     EngineState::default(),
    /// );
    ///
    /// let streamer = SessionFileStreamer::new(session);
    /// let (iter, json_bytes) = streamer.into_chunks(64 * 1024).unwrap();
    ///
    /// for chunk in iter {
    ///     // Transfer chunk across FFI...
    ///     println!("Chunk size: {}", chunk.len());
    /// }
    /// ```
    pub fn into_chunks(self, chunk_size: usize) -> FfiResult<(ChunkIterator<'static>, Vec<u8>)> {
        let json_bytes = self.to_json_bytes()?;
        // SAFETY: We're moving json_bytes ownership, so we need to leak it
        // to get a 'static lifetime. The caller is responsible for cleanup.
        let leaked_bytes = Box::leak(json_bytes.into_boxed_slice());
        let iter = ChunkIterator::new(leaked_bytes, chunk_size);
        Ok((iter, leaked_bytes.to_vec()))
    }

    /// Create a chunk iterator with default chunk size.
    ///
    /// Uses [`DEFAULT_CHUNK_SIZE`] (64KB) for chunking.
    pub fn into_default_chunks(self) -> FfiResult<(ChunkIterator<'static>, Vec<u8>)> {
        self.into_chunks(DEFAULT_CHUNK_SIZE)
    }
}

/// Streaming wrapper for [`DiscoverySummary`].
///
/// Provides efficient serialization and chunked streaming for discovery session results.
///
/// # When to Use Streaming
///
/// Discovery summaries are typically small (<10KB), but in pathological cases
/// (e.g., 1000-key keyboards with extensive aliases), they can exceed 1MB.
///
/// Use streaming when:
/// - The device has >100 keys
/// - Large alias maps are present
/// - Memory is constrained
///
/// # Example
///
/// ```no_run
/// use keyrx_core::ffi::marshal::impls::session::DiscoverySummaryStreamer;
/// use keyrx_core::discovery::session::{DiscoverySummary, SessionStatus};
/// use keyrx_core::discovery::types::DeviceId;
/// use std::collections::HashMap;
///
/// let summary = DiscoverySummary {
///     device_id: DeviceId::new(0x1234, 0x5678),
///     status: SessionStatus::Completed,
///     message: None,
///     rows: 6,
///     cols_per_row: vec![15, 15, 15, 15, 15, 15],
///     captured: 90,
///     total: 90,
///     next: None,
///     unmapped: vec![],
///     duplicates: vec![],
///     keymap: HashMap::new(),
///     aliases: HashMap::new(),
/// };
///
/// let streamer = DiscoverySummaryStreamer::new(summary);
/// let json = streamer.to_json_bytes().unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct DiscoverySummaryStreamer {
    summary: DiscoverySummary,
}

impl DiscoverySummaryStreamer {
    /// Create a new streamer for a discovery summary.
    ///
    /// # Parameters
    ///
    /// * `summary` - The discovery summary to stream
    pub fn new(summary: DiscoverySummary) -> Self {
        Self { summary }
    }

    /// Get a reference to the underlying summary.
    pub fn summary(&self) -> &DiscoverySummary {
        &self.summary
    }

    /// Consume the streamer and return the underlying summary.
    pub fn into_summary(self) -> DiscoverySummary {
        self.summary
    }

    /// Serialize the summary to JSON bytes.
    ///
    /// # Returns
    ///
    /// A `Vec<u8>` containing the JSON representation.
    ///
    /// # Errors
    ///
    /// Returns [`FfiError`] if serialization fails.
    pub fn to_json_bytes(&self) -> FfiResult<Vec<u8>> {
        serde_json::to_vec(&self.summary).map_err(|e| {
            FfiError::serialization_failed(format!("Failed to serialize DiscoverySummary: {}", e))
        })
    }

    /// Deserialize a summary from JSON bytes.
    ///
    /// # Parameters
    ///
    /// * `bytes` - JSON bytes to deserialize
    ///
    /// # Returns
    ///
    /// A new `DiscoverySummaryStreamer` containing the deserialized summary.
    ///
    /// # Errors
    ///
    /// Returns [`FfiError`] if deserialization fails.
    pub fn from_json_bytes(bytes: &[u8]) -> FfiResult<Self> {
        let summary = serde_json::from_slice(bytes).map_err(|e| {
            FfiError::deserialization_failed(format!(
                "Failed to deserialize DiscoverySummary: {}",
                e
            ))
        })?;
        Ok(Self { summary })
    }

    /// Create a chunk iterator for streaming the summary data.
    ///
    /// # Parameters
    ///
    /// * `chunk_size` - Size of each chunk in bytes
    ///
    /// # Returns
    ///
    /// A `Result` containing a `ChunkIterator` and the owned JSON bytes.
    ///
    /// # Note
    ///
    /// The returned bytes must be kept alive for the iterator's lifetime.
    pub fn into_chunks(self, chunk_size: usize) -> FfiResult<(ChunkIterator<'static>, Vec<u8>)> {
        let json_bytes = self.to_json_bytes()?;
        let leaked_bytes = Box::leak(json_bytes.into_boxed_slice());
        let iter = ChunkIterator::new(leaked_bytes, chunk_size);
        Ok((iter, leaked_bytes.to_vec()))
    }

    /// Create a chunk iterator with default chunk size.
    ///
    /// Uses [`DEFAULT_CHUNK_SIZE`] (64KB) for chunking.
    pub fn into_default_chunks(self) -> FfiResult<(ChunkIterator<'static>, Vec<u8>)> {
        self.into_chunks(DEFAULT_CHUNK_SIZE)
    }
}

/// Helper to collect chunks and reconstruct a [`SessionFile`].
///
/// # Example
///
/// ```no_run
/// use keyrx_core::ffi::marshal::impls::session::collect_session_chunks;
/// use keyrx_core::ffi::marshal::stream::ChunkCollector;
///
/// let mut collector = ChunkCollector::new();
/// // ... add chunks via collector.add_chunk(&chunk) ...
///
/// let session = collect_session_chunks(collector).unwrap();
/// ```
pub fn collect_session_chunks(collector: ChunkCollector) -> FfiResult<SessionFile> {
    let bytes = collector.into_vec();
    let streamer = SessionFileStreamer::from_json_bytes(&bytes)?;
    Ok(streamer.into_session())
}

/// Helper to collect chunks and reconstruct a [`DiscoverySummary`].
///
/// # Example
///
/// ```no_run
/// use keyrx_core::ffi::marshal::impls::session::collect_discovery_chunks;
/// use keyrx_core::ffi::marshal::stream::ChunkCollector;
///
/// let mut collector = ChunkCollector::new();
/// // ... add chunks via collector.add_chunk(&chunk) ...
///
/// let summary = collect_discovery_chunks(collector).unwrap();
/// ```
pub fn collect_discovery_chunks(collector: ChunkCollector) -> FfiResult<DiscoverySummary> {
    let bytes = collector.into_vec();
    let streamer = DiscoverySummaryStreamer::from_json_bytes(&bytes)?;
    Ok(streamer.into_summary())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::session::{
        DiscoverySummary, DuplicateWarning, ExpectedPosition, SessionStatus,
    };
    use crate::discovery::types::{DeviceId, PhysicalKey};
    use crate::engine::{
        DecisionType, EngineState, EventRecord, InputEvent, KeyCode, LayerStack, ModifierState,
        OutputAction, TimingConfig,
    };
    use std::collections::HashMap;

    fn make_test_session() -> SessionFile {
        let initial_state = EngineState {
            pressed_keys: vec![],
            modifiers: ModifierState::default(),
            layers: LayerStack::new(),
            pending: vec![],
            timing: TimingConfig::default(),
            safe_mode: false,
        };

        let mut session = SessionFile::new(
            "2024-01-15T10:30:00Z".to_string(),
            Some("/path/to/script.rhai".to_string()),
            TimingConfig::default(),
            initial_state,
        );

        // Add several events to make it more realistic
        for i in 0..100 {
            let input = InputEvent::key_down(KeyCode::A, i * 1000);
            let output = vec![OutputAction::KeyDown(KeyCode::A)];
            session.add_event(
                EventRecord::builder()
                    .seq(i)
                    .timestamp_us(i * 1000)
                    .input(input)
                    .output(output)
                    .decision_type(DecisionType::PassThrough)
                    .active_layers(vec![0])
                    .modifiers_state(ModifierState::default())
                    .latency_us(50)
                    .build(),
            );
        }

        session
    }

    fn make_test_discovery_summary() -> DiscoverySummary {
        let mut keymap = HashMap::new();
        let mut aliases = HashMap::new();

        // Simulate a 6-row keyboard with 90 keys
        for row in 0..6u8 {
            for col in 0..15u8 {
                let scan_code = (row as u16 * 15) + col as u16 + 1;
                let alias = format!("r{}_c{}", row, col);
                let mut key = PhysicalKey::new(scan_code, row, col);
                key.alias = Some(alias.clone());
                keymap.insert(scan_code, key);
                aliases.insert(alias, scan_code);
            }
        }

        DiscoverySummary {
            device_id: DeviceId::new(0x1234, 0x5678),
            status: SessionStatus::Completed,
            message: None,
            rows: 6,
            cols_per_row: vec![15, 15, 15, 15, 15, 15],
            captured: 90,
            total: 90,
            next: None,
            unmapped: vec![],
            duplicates: vec![],
            keymap,
            aliases,
        }
    }

    #[test]
    fn test_session_file_streamer_roundtrip() {
        let session = make_test_session();
        let original_event_count = session.event_count();

        let streamer = SessionFileStreamer::new(session);
        let json_bytes = streamer.to_json_bytes().expect("serialize");

        let reconstructed_streamer =
            SessionFileStreamer::from_json_bytes(&json_bytes).expect("deserialize");
        let reconstructed = reconstructed_streamer.into_session();

        assert_eq!(reconstructed.event_count(), original_event_count);
        assert_eq!(reconstructed.version, 1);
        assert_eq!(reconstructed.created_at, "2024-01-15T10:30:00Z");
    }

    #[test]
    fn test_session_file_streaming_with_chunks() {
        let session = make_test_session();
        let streamer = SessionFileStreamer::new(session.clone());

        let json_bytes = streamer.to_json_bytes().unwrap();
        let mut iter = ChunkIterator::new(&json_bytes, 1024); // 1KB chunks

        let mut collector = ChunkCollector::new();
        while let Some(chunk) = iter.next() {
            collector.add_chunk(chunk);
        }

        let reconstructed = collect_session_chunks(collector).unwrap();
        assert_eq!(reconstructed.event_count(), session.event_count());
    }

    #[test]
    fn test_discovery_summary_streamer_roundtrip() {
        let summary = make_test_discovery_summary();
        let original_keymap_size = summary.keymap.len();

        let streamer = DiscoverySummaryStreamer::new(summary);
        let json_bytes = streamer.to_json_bytes().expect("serialize");

        let reconstructed_streamer =
            DiscoverySummaryStreamer::from_json_bytes(&json_bytes).expect("deserialize");
        let reconstructed = reconstructed_streamer.into_summary();

        assert_eq!(reconstructed.keymap.len(), original_keymap_size);
        assert_eq!(reconstructed.status, SessionStatus::Completed);
        assert_eq!(reconstructed.rows, 6);
    }

    #[test]
    fn test_discovery_summary_streaming_with_chunks() {
        let summary = make_test_discovery_summary();
        let streamer = DiscoverySummaryStreamer::new(summary.clone());

        let json_bytes = streamer.to_json_bytes().unwrap();
        let mut iter = ChunkIterator::new(&json_bytes, 512); // 512-byte chunks

        let mut collector = ChunkCollector::new();
        while let Some(chunk) = iter.next() {
            collector.add_chunk(chunk);
        }

        let reconstructed = collect_discovery_chunks(collector).unwrap();
        assert_eq!(reconstructed.keymap.len(), summary.keymap.len());
        assert_eq!(reconstructed.aliases.len(), summary.aliases.len());
    }

    #[test]
    fn test_session_file_streamer_large_data() {
        // Create a session with many events to test chunking
        let initial_state = EngineState {
            pressed_keys: vec![],
            modifiers: ModifierState::default(),
            layers: LayerStack::new(),
            pending: vec![],
            timing: TimingConfig::default(),
            safe_mode: false,
        };

        let mut session = SessionFile::new(
            "2024-01-15T10:30:00Z".to_string(),
            None,
            TimingConfig::default(),
            initial_state,
        );

        // Add 1000 events (~500KB when serialized)
        for i in 0..1000 {
            let input = InputEvent::key_down(KeyCode::A, i * 1000);
            let output = vec![OutputAction::KeyDown(KeyCode::B)];
            session.add_event(
                EventRecord::builder()
                    .seq(i)
                    .timestamp_us(i * 1000)
                    .input(input)
                    .output(output)
                    .decision_type(DecisionType::Remap)
                    .active_layers(vec![0, 1])
                    .modifiers_state(ModifierState::default())
                    .latency_us(75)
                    .build(),
            );
        }

        let streamer = SessionFileStreamer::new(session);
        let json_bytes = streamer.to_json_bytes().unwrap();

        // Should be fairly large
        assert!(json_bytes.len() > 10_000, "Expected large JSON payload");

        // Chunk it
        let mut iter = ChunkIterator::new(&json_bytes, DEFAULT_CHUNK_SIZE);
        let mut collector = ChunkCollector::new();

        let mut chunk_count = 0;
        while let Some(chunk) = iter.next() {
            collector.add_chunk(chunk);
            chunk_count += 1;
        }

        assert!(chunk_count >= 1, "Should have at least one chunk");

        // Reconstruct and verify
        let reconstructed = collect_session_chunks(collector).unwrap();
        assert_eq!(reconstructed.event_count(), 1000);
    }

    #[test]
    fn test_discovery_summary_with_duplicates() {
        let mut summary = make_test_discovery_summary();
        summary.duplicates.push(DuplicateWarning {
            scan_code: 42,
            existing: ExpectedPosition { row: 0, col: 0 },
            attempted: ExpectedPosition { row: 1, col: 1 },
        });

        let streamer = DiscoverySummaryStreamer::new(summary.clone());
        let json_bytes = streamer.to_json_bytes().unwrap();

        let reconstructed_streamer =
            DiscoverySummaryStreamer::from_json_bytes(&json_bytes).unwrap();
        let reconstructed = reconstructed_streamer.into_summary();

        assert_eq!(reconstructed.duplicates.len(), 1);
        assert_eq!(reconstructed.duplicates[0].scan_code, 42);
    }

    #[test]
    fn test_error_handling_invalid_json() {
        let invalid_json = b"not valid json";

        let result = SessionFileStreamer::from_json_bytes(invalid_json);
        assert!(result.is_err());

        let result = DiscoverySummaryStreamer::from_json_bytes(invalid_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_session() {
        let initial_state = EngineState {
            pressed_keys: vec![],
            modifiers: ModifierState::default(),
            layers: LayerStack::new(),
            pending: vec![],
            timing: TimingConfig::default(),
            safe_mode: false,
        };

        let session = SessionFile::new(
            "2024-01-15T10:30:00Z".to_string(),
            None,
            TimingConfig::default(),
            initial_state,
        );

        let streamer = SessionFileStreamer::new(session);
        let json_bytes = streamer.to_json_bytes().unwrap();

        let reconstructed_streamer = SessionFileStreamer::from_json_bytes(&json_bytes).unwrap();
        let reconstructed = reconstructed_streamer.into_session();

        assert_eq!(reconstructed.event_count(), 0);
    }
}
