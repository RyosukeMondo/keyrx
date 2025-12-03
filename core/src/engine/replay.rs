//! Session replay module for deterministic event replay.
//!
//! This module provides the [`ReplaySession`] type which implements the [`InputSource`]
//! trait, allowing recorded `.krx` session files to be replayed through the engine
//! with the same timing as the original recording.

use crate::engine::{EventRecord, InputEvent, OutputAction, SessionFile};
use crate::errors::KeyrxError;
use crate::keyrx_err;
use crate::traits::InputSource;
use async_trait::async_trait;
use std::collections::VecDeque;
use std::path::Path;
use std::time::Instant;

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

/// Session replay source that implements [`InputSource`].
///
/// Loads a `.krx` session file and replays events with the same inter-event
/// timing as the original recording, enabling deterministic replay for
/// debugging and verification.
///
/// # Example
///
/// ```ignore
/// use keyrx_core::engine::replay::ReplaySession;
///
/// let mut replay = ReplaySession::from_file("session.krx")?;
/// replay.start().await?;
///
/// loop {
///     let events = replay.poll_events().await?;
///     if events.is_empty() && replay.is_completed() {
///         break;
///     }
///     // Process events through engine...
/// }
/// ```
pub struct ReplaySession {
    /// Events remaining to be replayed.
    events: VecDeque<EventRecord>,
    /// Original session metadata.
    session: SessionFile,
    /// Timestamp when replay started.
    start_time: Option<Instant>,
    /// Current replay state.
    state: ReplayState,
    /// Speed multiplier (1.0 = realtime, 2.0 = 2x speed, 0 = instant).
    speed_multiplier: f64,
}

impl std::fmt::Debug for ReplaySession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReplaySession")
            .field("events_remaining", &self.events.len())
            .field("state", &self.state)
            .field("speed_multiplier", &self.speed_multiplier)
            .finish()
    }
}

impl ReplaySession {
    /// Create a new replay session from a session file.
    pub fn new(session: SessionFile) -> Self {
        let events: VecDeque<EventRecord> = session.events.clone().into_iter().collect();

        Self {
            events,
            session,
            start_time: None,
            state: ReplayState::Idle,
            speed_multiplier: 1.0,
        }
    }

    /// Load a replay session from a `.krx` file.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, KeyrxError> {
        use crate::errors::config::CONFIG_READ_ERROR;
        use crate::errors::runtime::{SESSION_FILE_CORRUPT, SESSION_VERSION_MISMATCH};

        let path = path.as_ref();
        let content = std::fs::read_to_string(path).map_err(|e| {
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
                error = e.to_string()
            )
        })?;

        // Verify version compatibility
        if session.version > crate::engine::SESSION_FILE_VERSION {
            return Err(keyrx_err!(
                SESSION_VERSION_MISMATCH,
                found = session.version.to_string(),
                expected = crate::engine::SESSION_FILE_VERSION.to_string()
            ));
        }

        Ok(Self::new(session))
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

    /// Check if the replay has completed.
    pub fn is_completed(&self) -> bool {
        self.state == ReplayState::Completed
    }

    /// Get the current replay state.
    pub fn state(&self) -> ReplayState {
        self.state
    }

    /// Get the number of events remaining to replay.
    pub fn events_remaining(&self) -> usize {
        self.events.len()
    }

    /// Get the total number of events in the session.
    pub fn total_events(&self) -> usize {
        self.session.event_count()
    }

    /// Get a reference to the session file.
    pub fn session(&self) -> &SessionFile {
        &self.session
    }

    /// Get the recorded outputs for an event by sequence number.
    pub fn get_recorded_output(&self, seq: u64) -> Option<&Vec<OutputAction>> {
        self.session
            .events
            .iter()
            .find(|e| e.seq == seq)
            .map(|e| &e.output)
    }

    /// Peek at the next event without consuming it.
    pub fn peek_next(&self) -> Option<&EventRecord> {
        self.events.front()
    }

    /// Calculate elapsed microseconds since replay start.
    fn elapsed_us(&self) -> u64 {
        self.start_time
            .map(|t| t.elapsed().as_micros() as u64)
            .unwrap_or(0)
    }

    /// Calculate the adjusted timestamp considering speed multiplier.
    fn adjusted_timestamp(&self, original_timestamp_us: u64) -> u64 {
        if self.speed_multiplier == 0.0 {
            // Instant mode: all events are ready immediately
            return 0;
        }
        // Scale the timestamp inversely with speed
        // At 2x speed, a 1000µs event should fire at 500µs
        (original_timestamp_us as f64 / self.speed_multiplier) as u64
    }
}

#[async_trait]
impl InputSource for ReplaySession {
    async fn start(&mut self) -> Result<(), KeyrxError> {
        use crate::errors::runtime::SESSION_REPLAY_FAILED;

        if self.state == ReplayState::Playing {
            return Err(keyrx_err!(
                SESSION_REPLAY_FAILED,
                reason = "Replay already started".to_string()
            ));
        }

        self.start_time = Some(Instant::now());
        self.state = ReplayState::Playing;

        tracing::info!(
            "Starting replay of {} events from session",
            self.events.len()
        );

        Ok(())
    }

    async fn poll_events(&mut self) -> Result<Vec<InputEvent>, KeyrxError> {
        if self.state != ReplayState::Playing {
            return Ok(vec![]);
        }

        if self.events.is_empty() {
            self.state = ReplayState::Completed;
            tracing::info!("Replay completed");
            return Ok(vec![]);
        }

        let elapsed = self.elapsed_us();
        let mut ready_events = Vec::new();

        // Collect all events whose adjusted timestamp has passed
        while let Some(event) = self.events.front() {
            let adjusted_ts = self.adjusted_timestamp(event.timestamp_us);

            if elapsed >= adjusted_ts {
                if let Some(event) = self.events.pop_front() {
                    ready_events.push(event.input.clone());
                }
            } else {
                // Next event not ready yet
                break;
            }
        }

        if self.events.is_empty() && ready_events.is_empty() {
            self.state = ReplayState::Completed;
        }

        Ok(ready_events)
    }

    async fn send_output(&mut self, _action: OutputAction) -> Result<(), KeyrxError> {
        // In replay mode, we don't actually send outputs to the OS.
        // The caller can optionally compare outputs against recorded ones.
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), KeyrxError> {
        self.state = ReplayState::Idle;
        self.start_time = None;
        // Clear remaining events
        self.events.clear();

        tracing::info!("Replay stopped");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{
        DecisionType, EngineState, EventRecordBuilder, KeyCode, LayerStack, ModifierState,
        TimingConfig,
    };

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
            Some("/test/script.rhai".to_string()),
            TimingConfig::default(),
            initial_state,
        );

        // Add events with increasing timestamps
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
    fn replay_session_creation() {
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
        replay.set_speed(0.0); // Instant mode

        replay.start().await.expect("start");

        // All events should be available immediately
        let events = replay.poll_events().await.expect("poll");
        assert_eq!(events.len(), 5);

        // Second poll should return empty and mark completed
        let events = replay.poll_events().await.expect("poll");
        assert!(events.is_empty());
        assert!(replay.is_completed());
    }

    #[tokio::test]
    async fn replay_empty_before_start() {
        let session = make_test_session();
        let mut replay = ReplaySession::new(session);

        // Polling before start should return empty
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

        // Negative values should be clamped to 0
        replay.set_speed(-1.0);
        assert_eq!(replay.speed(), 0.0);
    }

    #[test]
    fn replay_peek_next() {
        let session = make_test_session();
        let replay = ReplaySession::new(session);

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

        // Non-existent sequence
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

        // send_output should succeed but do nothing
        let result = replay.send_output(OutputAction::KeyDown(KeyCode::A)).await;
        assert!(result.is_ok());
    }

    #[test]
    fn replay_session_accessor() {
        let session = make_test_session();
        let replay = ReplaySession::new(session);

        let session_ref = replay.session();
        assert_eq!(session_ref.event_count(), 5);
        assert_eq!(
            session_ref.script_path,
            Some("/test/script.rhai".to_string())
        );
    }
}
