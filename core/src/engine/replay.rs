//! Session replay module for deterministic event replay.
//!
//! Provides streaming playback for indexed recording files written by
//! `SessionRecorder` while remaining backward-compatible with legacy JSON
//! session files.

use crate::engine::recording::{
    decompress_block, BlockIndex, CompressionError, CompressionKind, RecordingHeader,
    RecordingMetadata, BLOCK_INDEX_SIZE, RECORDING_HEADER_SIZE, RECORDING_MAGIC,
};
use crate::engine::{EventRecord, InputEvent, OutputAction, SessionFile, TimingConfig};
use crate::errors::config::CONFIG_READ_ERROR;
use crate::errors::runtime::{
    SESSION_FILE_CORRUPT, SESSION_REPLAY_FAILED, SESSION_VERSION_MISMATCH,
};
use crate::errors::KeyrxError;
use crate::keyrx_err;
use crate::traits::InputSource;
use async_trait::async_trait;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

/// Error types for replay operations.
#[derive(Debug)]
pub enum ReplayError {
    /// Failed to load session file.
    LoadError(String),
    /// Session file format version mismatch.
    VersionMismatch { expected: u32, got: u32 },
    /// Replay completed.
    Completed,
}

impl std::fmt::Display for ReplayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReplayError::LoadError(msg) => write!(f, "Failed to load session: {}", msg),
            ReplayError::VersionMismatch { expected, got } => {
                write!(
                    f,
                    "Session version mismatch: expected {}, got {}",
                    expected, got
                )
            }
            ReplayError::Completed => write!(f, "Replay completed"),
        }
    }
}

impl std::error::Error for ReplayError {}

/// Replay state tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayState {
    /// Not started yet.
    Idle,
    /// Actively replaying events.
    Playing,
    /// Replay paused (for step-by-step debugging).
    Paused,
    /// Replay completed (all events emitted).
    Completed,
}

/// Lightweight manifest describing a recording to replay.
#[derive(Debug, Clone)]
pub struct ReplayManifest {
    /// Total events described by the recording.
    pub total_events: usize,
    /// Duration of the trace in microseconds.
    pub duration_us: u64,
    /// Optional script path captured by the legacy format.
    pub script_path: Option<String>,
    /// Timing configuration stored by the legacy format (defaulted for streaming).
    pub timing_config: TimingConfig,
    /// Compression codec used for the indexed blocks.
    pub compression: Option<CompressionKind>,
    /// Creation time in milliseconds since epoch.
    pub created_at_ms: Option<u64>,
    /// Engine version string captured in metadata.
    pub engine_version: Option<String>,
}

enum ReplaySource {
    Legacy {
        session: SessionFile,
        queue: VecDeque<EventRecord>,
    },
    Streaming {
        path: PathBuf,
        file: File,
        header: RecordingHeader,
        index: Vec<BlockIndex>,
        current_block: usize,
        buffer: VecDeque<EventRecord>,
    },
}

/// Session replay source that implements [`InputSource`].
///
/// Supports both legacy JSON `.krx` files and new indexed binary recordings.
pub struct ReplaySession {
    source: ReplaySource,
    manifest: ReplayManifest,
    session_state: crate::engine::SessionState,
    speed_multiplier: f64,
    emitted: Vec<EventRecord>,
    consumed: usize,
}

impl std::fmt::Debug for ReplaySession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReplaySession")
            .field("events_remaining", &self.events_remaining())
            .field("session_state", &self.session_state)
            .field("speed_multiplier", &self.speed_multiplier)
            .finish()
    }
}

impl ReplaySession {
    /// Create a replay session from an in-memory legacy session file.
    pub fn new(session: SessionFile) -> Self {
        Self::from_session(session)
    }

    /// Load a replay session from disk. Automatically detects indexed recordings
    /// via the magic header and falls back to legacy JSON parsing otherwise.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, KeyrxError> {
        let path = path.as_ref();
        let mut file = File::open(path).map_err(|e| {
            keyrx_err!(
                CONFIG_READ_ERROR,
                path = path.display().to_string(),
                error = e.to_string()
            )
        })?;

        let mut magic_buf = [0u8; 4];
        file.read_exact(&mut magic_buf).map_err(|e| {
            keyrx_err!(
                CONFIG_READ_ERROR,
                path = path.display().to_string(),
                error = e.to_string()
            )
        })?;
        file.seek(SeekFrom::Start(0)).map_err(|e| {
            keyrx_err!(
                CONFIG_READ_ERROR,
                path = path.display().to_string(),
                error = e.to_string()
            )
        })?;

        if magic_buf == RECORDING_MAGIC {
            Self::from_streaming_file(path.to_path_buf(), file)
        } else {
            let mut content = String::new();
            file.read_to_string(&mut content).map_err(|e| {
                keyrx_err!(
                    CONFIG_READ_ERROR,
                    path = path.display().to_string(),
                    error = e.to_string()
                )
            })?;

            let session = SessionFile::from_json(&content).map_err(|e| {
                keyrx_err!(
                    SESSION_FILE_CORRUPT,
                    path = path.display().to_string(),
                    error = format!("failed to parse session file: {e}")
                )
            })?;

            if session.version > crate::engine::SESSION_FILE_VERSION {
                return Err(keyrx_err!(
                    SESSION_VERSION_MISMATCH,
                    found = session.version.to_string(),
                    expected = crate::engine::SESSION_FILE_VERSION.to_string()
                ));
            }

            Ok(Self::from_session(session))
        }
    }

    /// Summary metadata for the loaded recording.
    pub fn manifest(&self) -> &ReplayManifest {
        &self.manifest
    }

    /// For legacy recordings, return the full session file.
    pub fn session(&self) -> Option<&SessionFile> {
        match &self.source {
            ReplaySource::Legacy { session, .. } => Some(session),
            _ => None,
        }
    }

    /// Get the number of events remaining to replay.
    pub fn events_remaining(&self) -> usize {
        self.manifest.total_events.saturating_sub(self.consumed)
    }

    /// Total number of events in the recording.
    pub fn total_events(&self) -> usize {
        self.manifest.total_events
    }

    /// Get the current replay state.
    pub fn state(&self) -> ReplayState {
        use crate::engine::SessionStatus;
        match self.session_state.status() {
            SessionStatus::Idle => ReplayState::Idle,
            SessionStatus::Active => ReplayState::Playing,
            SessionStatus::Paused => ReplayState::Paused,
            SessionStatus::Completed => ReplayState::Completed,
        }
    }

    /// Check if the replay has completed.
    pub fn is_completed(&self) -> bool {
        self.session_state.is_completed()
    }

    /// Peek at the next event without consuming it.
    pub fn peek_next(&mut self) -> Option<&EventRecord> {
        let _ = self.load_next_block_if_needed();
        match &self.source {
            ReplaySource::Legacy { queue, .. } => queue.front(),
            ReplaySource::Streaming { buffer, .. } => buffer.front(),
        }
    }

    /// Get the recorded outputs for an event by sequence number if available.
    pub fn get_recorded_output(&self, seq: u64) -> Option<&Vec<OutputAction>> {
        match &self.source {
            ReplaySource::Legacy { session, .. } => session
                .events
                .iter()
                .find(|e| e.seq == seq)
                .map(|e| &e.output),
            ReplaySource::Streaming { buffer, .. } => buffer
                .iter()
                .chain(self.emitted.iter())
                .find(|e| e.seq == seq)
                .map(|e| &e.output),
        }
    }

    /// Drain the set of events emitted by the most recent poll.
    pub fn take_emitted_records(&mut self) -> Vec<EventRecord> {
        std::mem::take(&mut self.emitted)
    }

    /// Set the replay speed multiplier.
    ///
    /// - `1.0` = realtime (same timing as original)
    /// - `2.0` = 2x speed (half the delays)
    /// - `0.5` = half speed (double the delays)
    /// - `0.0` = instant (no delays, for fast replay)
    pub fn set_speed(&mut self, multiplier: f64) {
        self.speed_multiplier = multiplier.max(0.0);
    }

    /// Get the current replay speed multiplier.
    pub fn speed(&self) -> f64 {
        self.speed_multiplier
    }

    fn from_session(session: SessionFile) -> Self {
        let manifest = ReplayManifest {
            total_events: session.event_count(),
            duration_us: session.duration_us(),
            script_path: session.script_path.clone(),
            timing_config: session.timing_config.clone(),
            compression: None,
            created_at_ms: None,
            engine_version: None,
        };

        Self {
            emitted: Vec::new(),
            consumed: 0,
            source: ReplaySource::Legacy {
                queue: session.events.clone().into_iter().collect(),
                session,
            },
            manifest,
            session_state: crate::engine::SessionState::new(),
            speed_multiplier: 1.0,
        }
    }

    fn from_streaming_file(path: PathBuf, mut file: File) -> Result<Self, KeyrxError> {
        let header = Self::read_streaming_header(&mut file, &path)?;
        let metadata = Self::read_streaming_metadata(&mut file, &header, &path)?;
        let index = Self::read_streaming_index(&mut file, &header, &path)?;

        let manifest = Self::build_streaming_manifest(&header, &metadata, &index);

        let mut session = Self {
            emitted: Vec::new(),
            consumed: 0,
            source: ReplaySource::Streaming {
                path,
                file,
                header,
                index,
                current_block: 0,
                buffer: VecDeque::new(),
            },
            manifest,
            session_state: crate::engine::SessionState::new(),
            speed_multiplier: 1.0,
        };

        // Eagerly load the first block so peek_next and counts behave correctly.
        let _ = session.load_next_block_if_needed();
        Ok(session)
    }

    fn read_streaming_header(file: &mut File, path: &Path) -> Result<RecordingHeader, KeyrxError> {
        let mut header_buf = [0u8; RECORDING_HEADER_SIZE];
        file.read_exact(&mut header_buf).map_err(|e| {
            keyrx_err!(
                CONFIG_READ_ERROR,
                path = path.display().to_string(),
                error = e.to_string()
            )
        })?;
        RecordingHeader::decode(&header_buf).map_err(|e| {
            keyrx_err!(
                SESSION_FILE_CORRUPT,
                path = path.display().to_string(),
                error = e.to_string()
            )
        })
    }

    fn read_streaming_metadata(
        file: &mut File,
        header: &RecordingHeader,
        path: &Path,
    ) -> Result<RecordingMetadata, KeyrxError> {
        file.seek(SeekFrom::Start(header.metadata_offset))
            .map_err(|e| {
                keyrx_err!(
                    CONFIG_READ_ERROR,
                    path = path.display().to_string(),
                    error = e.to_string()
                )
            })?;
        let mut metadata_buf = vec![0u8; header.metadata_len as usize];
        file.read_exact(&mut metadata_buf).map_err(|e| {
            keyrx_err!(
                CONFIG_READ_ERROR,
                path = path.display().to_string(),
                error = e.to_string()
            )
        })?;
        serde_json::from_slice(&metadata_buf).map_err(|e| {
            keyrx_err!(
                SESSION_FILE_CORRUPT,
                path = path.display().to_string(),
                error = e.to_string()
            )
        })
    }

    fn read_streaming_index(
        file: &mut File,
        header: &RecordingHeader,
        path: &Path,
    ) -> Result<Vec<BlockIndex>, KeyrxError> {
        file.seek(SeekFrom::Start(header.index_offset))
            .map_err(|e| {
                keyrx_err!(
                    CONFIG_READ_ERROR,
                    path = path.display().to_string(),
                    error = e.to_string()
                )
            })?;
        let mut index_buf = vec![0u8; header.index_len as usize];
        file.read_exact(&mut index_buf).map_err(|e| {
            keyrx_err!(
                CONFIG_READ_ERROR,
                path = path.display().to_string(),
                error = e.to_string()
            )
        })?;

        let mut index = Vec::with_capacity(header.block_count as usize);
        for chunk in index_buf.chunks(BLOCK_INDEX_SIZE) {
            index.push(BlockIndex::decode(chunk).map_err(|e| {
                keyrx_err!(
                    SESSION_FILE_CORRUPT,
                    path = path.display().to_string(),
                    error = e.to_string()
                )
            })?);
        }
        Ok(index)
    }

    fn build_streaming_manifest(
        header: &RecordingHeader,
        metadata: &RecordingMetadata,
        index: &[BlockIndex],
    ) -> ReplayManifest {
        let total_events: usize = index
            .iter()
            .map(|entry| (entry.end_seq - entry.start_seq + 1) as usize)
            .sum();
        let duration_us = index
            .last()
            .map(|entry| entry.end_timestamp_us)
            .unwrap_or_default();

        ReplayManifest {
            total_events,
            duration_us,
            script_path: None,
            timing_config: TimingConfig::default(),
            compression: Some(header.compression),
            created_at_ms: Some(metadata.created_at_ms),
            engine_version: Some(metadata.engine_version.clone()),
        }
    }

    fn elapsed_us(&self) -> u64 {
        self.session_state.elapsed_us()
    }

    fn adjusted_timestamp(&self, original_timestamp_us: u64) -> u64 {
        if self.speed_multiplier == 0.0 {
            return 0;
        }
        (original_timestamp_us as f64 / self.speed_multiplier) as u64
    }

    fn load_next_block_if_needed(&mut self) -> Result<bool, KeyrxError> {
        match &mut self.source {
            ReplaySource::Streaming {
                file,
                header,
                index,
                current_block,
                buffer,
                path,
                ..
            } => {
                if !buffer.is_empty() {
                    return Ok(false);
                }
                if *current_block >= index.len() {
                    return Ok(false);
                }

                let entry = index[*current_block].clone();
                *current_block += 1;
                *buffer = Self::load_block_events(file, header, &entry, path.as_path())?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn load_block_events(
        file: &mut File,
        header: &RecordingHeader,
        entry: &BlockIndex,
        path: &Path,
    ) -> Result<VecDeque<EventRecord>, KeyrxError> {
        file.seek(SeekFrom::Start(entry.offset)).map_err(|e| {
            keyrx_err!(
                CONFIG_READ_ERROR,
                path = path.display().to_string(),
                error = e.to_string()
            )
        })?;
        let mut compressed = vec![0u8; entry.compressed_len as usize];
        file.read_exact(&mut compressed).map_err(|e| {
            keyrx_err!(
                CONFIG_READ_ERROR,
                path = path.display().to_string(),
                error = e.to_string()
            )
        })?;

        let decompressed = decompress_block(header.compression, &compressed)
            .map_err(|e| Self::compression_err(e, path))?;

        let mut events = VecDeque::new();
        for line in decompressed.split(|b| *b == b'\n') {
            if line.is_empty() {
                continue;
            }

            let event: EventRecord = serde_json::from_slice(line).map_err(|e| {
                keyrx_err!(
                    SESSION_FILE_CORRUPT,
                    path = path.display().to_string(),
                    error = e.to_string()
                )
            })?;
            events.push_back(event);
        }

        Ok(events)
    }

    fn compression_err(err: CompressionError, path: &Path) -> KeyrxError {
        let reason = err.to_string();
        keyrx_err!(
            SESSION_REPLAY_FAILED,
            path = path.display().to_string(),
            reason = reason
        )
    }

    fn has_more_events(&self) -> bool {
        self.consumed < self.manifest.total_events
    }
}

#[async_trait]
impl InputSource for ReplaySession {
    async fn start(&mut self) -> Result<(), KeyrxError> {
        use crate::errors::runtime::SESSION_REPLAY_FAILED;

        if self.session_state.is_active() {
            return Err(keyrx_err!(
                SESSION_REPLAY_FAILED,
                reason = "Replay already started".to_string()
            ));
        }

        self.session_state.start();
        self.load_next_block_if_needed()?;

        tracing::info!("Starting replay of {} events", self.manifest.total_events);

        if !self.has_more_events() {
            self.session_state.complete();
        }

        Ok(())
    }

    async fn poll_events(&mut self) -> Result<Vec<InputEvent>, KeyrxError> {
        self.emitted.clear();

        if !self.session_state.is_active() || !self.has_more_events() {
            return Ok(vec![]);
        }

        self.load_next_block_if_needed()?;

        let elapsed = self.elapsed_us();
        let mut ready_events = Vec::new();

        loop {
            let next_record = match &self.source {
                ReplaySource::Legacy { queue, .. } => queue.front().cloned(),
                ReplaySource::Streaming { buffer, .. } => buffer.front().cloned(),
            };

            let Some(record) = next_record else {
                // No buffered events; try to load another block then break.
                self.load_next_block_if_needed()?;
                break;
            };

            let adjusted_ts = self.adjusted_timestamp(record.timestamp_us);
            if elapsed >= adjusted_ts {
                let record = match &mut self.source {
                    ReplaySource::Legacy { queue, .. } => queue.pop_front(),
                    ReplaySource::Streaming { buffer, .. } => buffer.pop_front(),
                }
                .ok_or_else(|| {
                    keyrx_err!(
                        SESSION_REPLAY_FAILED,
                        reason = "Replay buffer unexpectedly empty".to_string()
                    )
                })?;

                ready_events.push(record.input.clone());
                self.emitted.push(record);
                self.consumed += 1;

                // If the current block is depleted, load the next before continuing.
                self.load_next_block_if_needed()?;
            } else {
                break;
            }
        }

        if !self.has_more_events() {
            self.session_state.complete();
        }

        Ok(ready_events)
    }

    async fn send_output(&mut self, _action: OutputAction) -> Result<(), KeyrxError> {
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), KeyrxError> {
        self.session_state.stop();
        self.emitted.clear();
        self.consumed = self.manifest.total_events;

        if let ReplaySource::Legacy { queue, .. } = &mut self.source {
            queue.clear();
        } else if let ReplaySource::Streaming { buffer, .. } = &mut self.source {
            buffer.clear();
        }

        tracing::info!("Replay stopped");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::recording::{RecordingMetadata, SessionRecorder};
    use crate::engine::state::EngineState;
    use crate::engine::{
        DecisionType, EventRecordBuilder, KeyCode, ModifierState, OutputAction,
        SESSION_FILE_VERSION,
    };
    use tempfile::TempDir;

    fn make_test_session() -> SessionFile {
        let initial_state = EngineState::new(TimingConfig::default());
        let initial_snapshot = (&initial_state).into();

        let mut session = SessionFile::new(
            "2024-01-15T10:30:00Z".to_string(),
            Some("/test/script.rhai".to_string()),
            TimingConfig::default(),
            initial_snapshot,
        );

        for i in 0..5 {
            let input = InputEvent::key_down(KeyCode::A, (i * 10_000) as u64);
            session.add_event(
                EventRecordBuilder::new()
                    .seq(i)
                    .timestamp_us((i * 10_000) as u64)
                    .input(input)
                    .output(vec![OutputAction::KeyDown(KeyCode::A)])
                    .decision_type(DecisionType::PassThrough)
                    .active_layers(vec![0])
                    .modifiers_state(ModifierState::default())
                    .latency_us(50)
                    .build(),
            );
        }

        session
    }

    #[test]
    fn replay_session_creation_legacy() {
        let session = make_test_session();
        let replay = ReplaySession::new(session);

        assert_eq!(replay.events_remaining(), 5);
        assert_eq!(replay.total_events(), 5);
        assert_eq!(replay.state(), ReplayState::Idle);
        assert!(!replay.is_completed());
    }

    #[tokio::test]
    async fn replay_start_and_stop() {
        let session = make_test_session();
        let mut replay = ReplaySession::new(session);

        assert_eq!(replay.state(), ReplayState::Idle);

        replay.start().await.expect("start");
        assert_eq!(replay.state(), ReplayState::Playing);

        replay.stop().await.expect("stop");
        assert_eq!(replay.state(), ReplayState::Idle);
        assert_eq!(replay.events_remaining(), 0);
    }

    #[tokio::test]
    async fn replay_double_start_fails() {
        let session = make_test_session();
        let mut replay = ReplaySession::new(session);

        replay.start().await.expect("first start");
        let result = replay.start().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn replay_instant_mode() {
        let session = make_test_session();
        let mut replay = ReplaySession::new(session);
        replay.set_speed(0.0);

        replay.start().await.expect("start");

        let events = replay.poll_events().await.expect("poll");
        assert_eq!(events.len(), 5);

        let events = replay.poll_events().await.expect("poll");
        assert!(events.is_empty());
        assert!(replay.is_completed());
    }

    #[tokio::test]
    async fn replay_empty_before_start() {
        let session = make_test_session();
        let mut replay = ReplaySession::new(session);

        let events = replay.poll_events().await.expect("poll");
        assert!(events.is_empty());
    }

    #[test]
    fn replay_speed_settings() {
        let session = make_test_session();
        let mut replay = ReplaySession::new(session);

        assert_eq!(replay.speed(), 1.0);

        replay.set_speed(2.0);
        assert_eq!(replay.speed(), 2.0);

        replay.set_speed(0.5);
        assert_eq!(replay.speed(), 0.5);

        replay.set_speed(-1.0);
        assert_eq!(replay.speed(), 0.0);
    }

    #[test]
    fn replay_peek_next() {
        let session = make_test_session();
        let mut replay = ReplaySession::new(session);

        let next = replay.peek_next().expect("peek");
        assert_eq!(next.seq, 0);
        assert_eq!(next.timestamp_us, 0);
    }

    #[test]
    fn replay_get_recorded_output() {
        let session = make_test_session();
        let replay = ReplaySession::new(session);

        let output = replay.get_recorded_output(0).expect("get output");
        assert_eq!(output.len(), 1);
        assert!(matches!(output[0], OutputAction::KeyDown(KeyCode::A)));

        assert!(replay.get_recorded_output(999).is_none());
    }

    #[test]
    fn replay_debug_format() {
        let session = make_test_session();
        let replay = ReplaySession::new(session);

        let debug = format!("{:?}", replay);
        assert!(debug.contains("ReplaySession"));
        assert!(debug.contains("events_remaining"));
    }

    #[tokio::test]
    async fn replay_send_output_is_noop() {
        let session = make_test_session();
        let mut replay = ReplaySession::new(session);

        let result = replay.send_output(OutputAction::KeyDown(KeyCode::A)).await;
        assert!(result.is_ok());
    }

    #[test]
    fn replay_manifest_legacy_fields() {
        let session = make_test_session();
        let replay = ReplaySession::new(session);

        let manifest = replay.manifest();
        assert_eq!(manifest.total_events, 5);
        assert_eq!(manifest.script_path.as_deref(), Some("/test/script.rhai"));
        assert!(manifest.compression.is_none());
    }

    #[tokio::test]
    async fn replay_streaming_block_file() {
        let dir = TempDir::new().expect("temp dir");
        let path = dir.path().join("streaming.krx");

        let metadata = RecordingMetadata::new(CompressionKind::None);
        let mut recorder =
            SessionRecorder::new(&path, CompressionKind::None, metadata).expect("create recorder");

        for i in 0..3 {
            let input = InputEvent::key_down(KeyCode::A, (i * 5_000) as u64);
            let record = EventRecordBuilder::new()
                .seq(i as u64)
                .timestamp_us((i * 5_000) as u64)
                .input(input)
                .output(vec![OutputAction::KeyDown(KeyCode::A)])
                .decision_type(DecisionType::PassThrough)
                .active_layers(vec![0])
                .modifiers_state(ModifierState::default())
                .latency_us(10)
                .build();
            recorder.record_event(&record).expect("record event");
        }

        let summary = recorder.finish().expect("finish recording");
        assert_eq!(summary.block_count, 1);
        assert!(path.exists());

        let mut replay = ReplaySession::from_file(&path).expect("load streaming replay");
        assert_eq!(replay.manifest().compression, Some(CompressionKind::None));
        assert_eq!(replay.total_events(), 3);
        replay.set_speed(0.0);
        replay.start().await.expect("start replay");

        let events = replay.poll_events().await.expect("poll");
        assert_eq!(events.len(), 3);

        let recorded = replay.take_emitted_records();
        assert_eq!(recorded.len(), 3);
        assert_eq!(recorded[0].seq, 0);
        assert_eq!(recorded[1].timestamp_us, 5_000);
    }

    #[test]
    fn replay_rejects_future_version() {
        let mut session = make_test_session();
        session.version = SESSION_FILE_VERSION + 1;

        let dir = TempDir::new().expect("temp dir");
        let path = dir.path().join("future.krx");
        let json = session.to_json().expect("serialize");
        std::fs::write(&path, json).expect("write");

        let result = ReplaySession::from_file(&path);
        assert!(result.is_err());
    }
}
