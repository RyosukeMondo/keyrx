//! Block-based session recorder that writes indexed, compressed recording files.
//!
//! The recorder writes a placeholder header, emits JSON metadata, streams
//! compressed blocks of serialized [`EventRecord`] rows, and finally appends
//! a fixed-width block index. On completion the header is rewritten with the
//! correct offsets so replay can seek directly to each block.

use std::fs::OpenOptions;
use std::io::{self, BufWriter, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::Serialize;
use thiserror::Error;

use crate::engine::event_recording::EventRecord;

use super::compression::{compress_block, CompressionError};
use super::format::{
    BlockIndex, CompressionKind, RecordingHeader, BLOCK_INDEX_SIZE, RECORDING_HEADER_SIZE,
};

/// Maximum uncompressed block size before a flush is forced (10 MiB).
pub const MAX_BLOCK_UNCOMPRESSED: usize = 10 * 1024 * 1024;

/// Metadata stored in the metadata segment ahead of the block payloads.
#[derive(Debug, Clone, Serialize)]
pub struct RecordingMetadata {
    /// Recording format version.
    pub format_version: u32,
    /// Milliseconds since epoch when the recording started.
    pub created_at_ms: u64,
    /// Compression codec identifier (matches [`CompressionKind::as_byte`]).
    pub compression: u8,
    /// Engine crate version string for traceability.
    pub engine_version: String,
    /// Optional free-form note supplied by the caller.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl RecordingMetadata {
    /// Construct metadata with the current timestamp and engine version.
    pub fn new(compression: CompressionKind) -> Self {
        Self {
            format_version: super::RECORDING_VERSION,
            created_at_ms: Utc::now().timestamp_millis() as u64,
            compression: compression.as_byte(),
            engine_version: env!("CARGO_PKG_VERSION").to_string(),
            note: None,
        }
    }

    /// Attach an optional note to the metadata.
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }
}

/// Summary of a completed recording.
#[derive(Debug, Clone)]
pub struct RecordingSummary {
    /// Number of compressed blocks written.
    pub block_count: u32,
    /// Total events recorded across all blocks.
    pub events: u64,
    /// Final byte length of the recording file.
    pub bytes_written: u64,
    /// Generated block index table.
    pub index: Vec<BlockIndex>,
}

/// Errors surfaced while recording a session.
#[derive(Debug, Error)]
pub enum RecorderError {
    /// I/O failure while writing the recording.
    #[error("recording I/O error: {0}")]
    Io(#[from] io::Error),
    /// Serialization failure when encoding events or metadata.
    #[error("failed to serialize recording data: {0}")]
    Serialize(#[from] serde_json::Error),
    /// Compression backend failure.
    #[error("failed to compress block: {0}")]
    Compression(#[from] CompressionError),
    /// Attempted to finalize a recording without any blocks.
    #[error("recording contains no blocks to index")]
    EmptyRecording,
    /// Uncompressed block exceeded the configured maximum size.
    #[error("uncompressed block too large: {size} bytes (max {max})")]
    BlockTooLarge { size: usize, max: usize },
    /// Integer conversion would overflow the target field.
    #[error("value for {field} exceeds supported range")]
    ValueOverflow { field: &'static str },
}

/// Writes session events into compressed blocks with an index for replay.
pub struct SessionRecorder {
    writer: BufWriter<std::fs::File>,
    path: PathBuf,
    metadata_offset: u64,
    metadata_len: u32,
    created_at_ms: u64,
    compression: CompressionKind,
    /// Accumulates uncompressed bytes for the current block.
    current_block: Vec<u8>,
    current_block_start_seq: Option<u64>,
    current_block_start_ts: Option<u64>,
    current_block_end_seq: u64,
    current_block_end_ts: u64,
    block_index: Vec<BlockIndex>,
    total_events: u64,
}

impl SessionRecorder {
    /// Create a new recorder at the given path and write header + metadata.
    pub fn new<P: AsRef<Path>>(
        path: P,
        compression: CompressionKind,
        metadata: RecordingMetadata,
    ) -> Result<Self, RecorderError> {
        let path_buf = path.as_ref().to_path_buf();
        if let Some(parent) = path_buf.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                return Err(RecorderError::Io(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("parent directory does not exist: {}", parent.display()),
                )));
            }
        }

        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path_buf)?;
        let mut writer = BufWriter::new(file);

        // Reserve space for header.
        writer.write_all(&[0u8; RECORDING_HEADER_SIZE])?;

        // Write metadata immediately after the header.
        let metadata_bytes = serde_json::to_vec(&metadata)?;
        let metadata_offset = RECORDING_HEADER_SIZE as u64;
        let metadata_len =
            u32::try_from(metadata_bytes.len()).map_err(|_| RecorderError::ValueOverflow {
                field: "metadata_len",
            })?;
        writer.write_all(&metadata_bytes)?;

        Ok(Self {
            path: path_buf,
            writer,
            metadata_offset,
            metadata_len,
            created_at_ms: metadata.created_at_ms,
            compression,
            current_block: Vec::new(),
            current_block_start_seq: None,
            current_block_start_ts: None,
            current_block_end_seq: 0,
            current_block_end_ts: 0,
            block_index: Vec::new(),
            total_events: 0,
        })
    }

    /// Append a recorded event to the current block, flushing when necessary.
    pub fn record_event(&mut self, event: &EventRecord) -> Result<(), RecorderError> {
        let mut encoded = serde_json::to_vec(event)?;
        encoded.push(b'\n');

        let new_len = self.current_block.len() + encoded.len();
        if new_len > MAX_BLOCK_UNCOMPRESSED {
            // Flush the current block before adding a large event to avoid oversizing.
            self.flush_block()?;
            if encoded.len() > MAX_BLOCK_UNCOMPRESSED {
                return Err(RecorderError::BlockTooLarge {
                    size: encoded.len(),
                    max: MAX_BLOCK_UNCOMPRESSED,
                });
            }
        }

        if self.current_block_start_seq.is_none() {
            self.current_block_start_seq = Some(event.seq);
            self.current_block_start_ts = Some(event.timestamp_us);
        }

        self.current_block_end_seq = event.seq;
        self.current_block_end_ts = event.timestamp_us;

        self.current_block.extend_from_slice(&encoded);
        self.total_events += 1;

        if self.current_block.len() >= MAX_BLOCK_UNCOMPRESSED {
            self.flush_block()?;
        }

        Ok(())
    }

    /// Force a flush of the current block if it contains data.
    pub fn flush_block(&mut self) -> Result<(), RecorderError> {
        if self.current_block.is_empty() {
            return Ok(());
        }

        let offset = self.writer.stream_position()?;
        let uncompressed_len: u32 =
            u32::try_from(self.current_block.len()).map_err(|_| RecorderError::ValueOverflow {
                field: "uncompressed_len",
            })?;
        let compressed = compress_block(self.compression, &self.current_block)?;
        let compressed_len: u32 =
            u32::try_from(compressed.len()).map_err(|_| RecorderError::ValueOverflow {
                field: "compressed_len",
            })?;

        self.writer.write_all(&compressed)?;

        let block_id: u32 = u32::try_from(self.block_index.len())
            .map_err(|_| RecorderError::ValueOverflow { field: "block_id" })?;

        let index_entry = BlockIndex {
            block_id,
            start_seq: self.current_block_start_seq.unwrap_or(0),
            end_seq: self.current_block_end_seq,
            start_timestamp_us: self.current_block_start_ts.unwrap_or(0),
            end_timestamp_us: self.current_block_end_ts,
            offset,
            compressed_len,
            uncompressed_len,
            checksum: 0,
        };

        self.block_index.push(index_entry);
        self.current_block.clear();
        self.current_block_start_seq = None;
        self.current_block_start_ts = None;
        self.current_block_end_seq = 0;
        self.current_block_end_ts = 0;

        Ok(())
    }

    /// Finalize the recording by writing the index and header.
    pub fn finish(mut self) -> Result<RecordingSummary, RecorderError> {
        self.flush_block()?;

        if self.block_index.is_empty() {
            return Err(RecorderError::EmptyRecording);
        }

        let index_offset = self.writer.stream_position()?;
        for entry in &self.block_index {
            self.writer.write_all(&entry.encode())?;
        }

        let block_count: u32 =
            u32::try_from(self.block_index.len()).map_err(|_| RecorderError::ValueOverflow {
                field: "block_count",
            })?;
        let index_len = BLOCK_INDEX_SIZE as u32 * block_count;

        let header = RecordingHeader::new(
            self.compression,
            self.created_at_ms,
            self.metadata_offset,
            self.metadata_len,
            index_offset,
            index_len,
            block_count,
        );

        // Seek to start and rewrite the header.
        self.writer.seek(SeekFrom::Start(0))?;
        self.writer.write_all(&header.encode())?;
        self.writer.flush()?;

        let bytes_written = self.writer.get_ref().metadata()?.len();

        Ok(RecordingSummary {
            block_count,
            events: self.total_events,
            bytes_written,
            index: self.block_index,
        })
    }

    /// Abort recording without finalizing the header.
    pub fn abort(self) {
        // Dropping without writing the header leaves the file incomplete.
        // The caller can delete the file if desired.
        let _ = self;
    }

    /// Path of the recording file being written.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{
        DecisionType, EventRecordBuilder, InputEvent, KeyCode, ModifierState, OutputAction,
    };
    use std::io::Read;

    fn make_event(seq: u64, ts: u64) -> EventRecord {
        EventRecordBuilder::new()
            .seq(seq)
            .timestamp_us(ts)
            .input(InputEvent::key_down(KeyCode::A, ts))
            .output(vec![OutputAction::KeyDown(KeyCode::A)])
            .decision_type(DecisionType::PassThrough)
            .active_layers(vec![0])
            .modifiers_state(ModifierState::default())
            .latency_us(10)
            .build()
    }

    #[test]
    fn writes_header_metadata_and_index() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("session.krr");

        let metadata = RecordingMetadata::new(CompressionKind::None);
        let mut recorder =
            SessionRecorder::new(&path, CompressionKind::None, metadata).expect("create recorder");

        recorder
            .record_event(&make_event(0, 100))
            .expect("record 0");
        recorder
            .record_event(&make_event(1, 200))
            .expect("record 1");

        let summary = recorder.finish().expect("finish");
        assert_eq!(summary.block_count, 1);
        assert_eq!(summary.events, 2);
        assert_eq!(summary.index.len(), 1);

        let mut file = std::fs::File::open(&path).expect("open");
        let mut header_buf = [0u8; RECORDING_HEADER_SIZE];
        file.read_exact(&mut header_buf).expect("read header");
        let header = RecordingHeader::decode(&header_buf).expect("decode header");

        assert_eq!(header.compression, CompressionKind::None);
        assert_eq!(header.block_count, 1);
        assert_eq!(header.index_len, BLOCK_INDEX_SIZE as u32);

        file.seek(SeekFrom::Start(header.index_offset))
            .expect("seek index");
        let mut index_buf = vec![0u8; header.index_len as usize];
        file.read_exact(&mut index_buf).expect("read index");
        let entry = BlockIndex::decode(&index_buf).expect("decode index");

        assert_eq!(entry.start_seq, 0);
        assert_eq!(entry.end_seq, 1);
        assert_eq!(entry.start_timestamp_us, 100);
        assert_eq!(entry.end_timestamp_us, 200);
    }
}
