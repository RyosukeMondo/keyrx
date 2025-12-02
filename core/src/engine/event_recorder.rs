//! Event recorder middleware for session recording.
//!
//! This module provides the `EventRecorder` middleware for recording keyboard
//! event sessions to .krx files.

use super::event_recording::{EventRecord, RecordingError, SessionFile, SESSION_FILE_VERSION};
use crate::engine::{EngineState, TimingConfig};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::time::Instant;

/// Middleware for recording keyboard events to a .krx session file.
///
/// Events are buffered in memory and written to disk when `finish()` is called.
/// This ensures atomic writes and avoids I/O during event processing.
#[derive(Debug)]
pub struct EventRecorder {
    /// Path to the output .krx file.
    path: std::path::PathBuf,
    /// Instant when recording started.
    session_start: Instant,
    /// Buffered events.
    events: Vec<EventRecord>,
    /// Script path used during recording.
    script_path: Option<String>,
    /// Timing configuration.
    timing_config: TimingConfig,
    /// Initial engine state.
    initial_state: EngineState,
}

impl EventRecorder {
    /// Create a new EventRecorder that will write to the given path.
    ///
    /// The file is not created until `finish()` is called.
    pub fn new<P: AsRef<Path>>(
        path: P,
        script_path: Option<String>,
        timing_config: TimingConfig,
        initial_state: EngineState,
    ) -> Result<Self, RecordingError> {
        let path = path.as_ref().to_path_buf();

        // Verify parent directory exists (fail fast)
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                return Err(RecordingError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Parent directory does not exist: {}", parent.display()),
                )));
            }
        }

        Ok(Self {
            path,
            session_start: Instant::now(),
            events: Vec::new(),
            script_path,
            timing_config,
            initial_state,
        })
    }

    /// Record an event.
    ///
    /// Events are buffered in memory until `finish()` is called.
    pub fn record_event(&mut self, event: EventRecord) {
        self.events.push(event);
    }

    /// Get the number of recorded events.
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Get elapsed time since recording started in microseconds.
    pub fn elapsed_us(&self) -> u64 {
        self.session_start.elapsed().as_micros() as u64
    }

    /// Finish recording and write the session file.
    ///
    /// Consumes the recorder and writes all buffered events to the .krx file.
    pub fn finish(self) -> Result<SessionFile, RecordingError> {
        let created_at = chrono::Utc::now().to_rfc3339();

        let session = SessionFile {
            version: SESSION_FILE_VERSION,
            created_at,
            script_path: self.script_path,
            timing_config: self.timing_config,
            initial_state: self.initial_state,
            events: self.events,
        };

        // Write to file
        let file = File::create(&self.path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &session)?;

        Ok(session)
    }

    /// Abort recording without writing.
    ///
    /// Useful for graceful cleanup on error.
    pub fn abort(self) {
        // Just drop without writing
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{
        DecisionType, EventRecordBuilder, KeyCode, LayerStack, ModifierState, OutputAction,
    };

    fn make_initial_state() -> EngineState {
        EngineState {
            pressed_keys: vec![],
            modifiers: ModifierState::default(),
            layers: LayerStack::new(),
            pending: vec![],
            timing: TimingConfig::default(),
            safe_mode: false,
        }
    }

    fn make_input_event(key: KeyCode, timestamp_us: u64) -> crate::engine::InputEvent {
        crate::engine::InputEvent::key_down(key, timestamp_us)
    }

    #[test]
    fn event_recorder_records_events() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let path = dir.path().join("test_session.krx");

        let mut recorder = EventRecorder::new(
            &path,
            Some("/scripts/test.rhai".to_string()),
            TimingConfig::default(),
            make_initial_state(),
        )
        .expect("create recorder");

        assert_eq!(recorder.event_count(), 0);

        // Record some events
        let input = make_input_event(KeyCode::A, 1000);
        recorder.record_event(
            EventRecordBuilder::new()
                .seq(0)
                .timestamp_us(1000)
                .input(input)
                .output(vec![OutputAction::KeyDown(KeyCode::B)])
                .decision_type(DecisionType::Remap)
                .active_layers(vec![0])
                .modifiers_state(ModifierState::default())
                .latency_us(50)
                .build(),
        );

        assert_eq!(recorder.event_count(), 1);

        let input2 = crate::engine::InputEvent::key_up(KeyCode::A, 2000);
        recorder.record_event(
            EventRecordBuilder::new()
                .seq(1)
                .timestamp_us(2000)
                .input(input2)
                .output(vec![OutputAction::KeyUp(KeyCode::B)])
                .decision_type(DecisionType::Remap)
                .active_layers(vec![0])
                .modifiers_state(ModifierState::default())
                .latency_us(45)
                .build(),
        );

        assert_eq!(recorder.event_count(), 2);

        // Finish recording
        let session = recorder.finish().expect("finish recording");

        // Verify session
        assert_eq!(session.version, SESSION_FILE_VERSION);
        assert_eq!(session.script_path, Some("/scripts/test.rhai".to_string()));
        assert_eq!(session.event_count(), 2);
        assert_eq!(session.events[0].seq, 0);
        assert_eq!(session.events[1].seq, 1);

        // Verify file was written
        assert!(path.exists());

        // Verify file content can be read back
        let content = std::fs::read_to_string(&path).expect("read file");
        let loaded = SessionFile::from_json(&content).expect("parse file");
        assert_eq!(loaded.event_count(), 2);
    }

    #[test]
    fn event_recorder_fails_on_nonexistent_parent() {
        let result = EventRecorder::new(
            "/nonexistent/directory/session.krx",
            None,
            TimingConfig::default(),
            make_initial_state(),
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, RecordingError::Io(_)));
    }

    #[test]
    fn event_recorder_abort_does_not_write() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let path = dir.path().join("aborted_session.krx");

        let mut recorder =
            EventRecorder::new(&path, None, TimingConfig::default(), make_initial_state())
                .expect("create recorder");

        // Record an event
        let input = make_input_event(KeyCode::A, 1000);
        recorder.record_event(
            EventRecordBuilder::new()
                .seq(0)
                .timestamp_us(1000)
                .input(input)
                .output(vec![OutputAction::PassThrough])
                .decision_type(DecisionType::PassThrough)
                .active_layers(vec![0])
                .modifiers_state(ModifierState::default())
                .latency_us(10)
                .build(),
        );

        // Abort instead of finish
        recorder.abort();

        // File should not exist
        assert!(!path.exists());
    }

    #[test]
    fn event_recorder_elapsed_time() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let path = dir.path().join("elapsed_session.krx");

        let recorder =
            EventRecorder::new(&path, None, TimingConfig::default(), make_initial_state())
                .expect("create recorder");

        // Sleep a tiny bit and check elapsed
        std::thread::sleep(std::time::Duration::from_millis(1));
        let elapsed = recorder.elapsed_us();

        // Should be at least 1000 microseconds (1ms)
        assert!(
            elapsed >= 1000,
            "elapsed should be at least 1ms, got {}",
            elapsed
        );

        recorder.abort();
    }
}
