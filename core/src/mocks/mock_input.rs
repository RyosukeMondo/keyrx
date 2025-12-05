//! Mock input source for testing.

use crate::engine::{InputEvent, KeyCode, OutputAction};
use crate::errors::KeyrxError;
use crate::keyrx_err;
use crate::traits::InputSource;
use async_trait::async_trait;
use std::collections::VecDeque;
use std::time::Instant;

/// Represents a recorded method call on MockInput.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MockCall {
    /// start() was called.
    Start,
    /// stop() was called.
    Stop,
    /// poll_events() was called.
    PollEvents,
    /// send_output() was called with the given action.
    SendOutput(OutputAction),
}

/// Mock input source for testing without real keyboard.
pub struct MockInput {
    /// Queued input events to return.
    input_queue: VecDeque<InputEvent>,
    /// Captured output actions.
    output_log: Vec<OutputAction>,
    /// Whether the mock is "running".
    running: bool,
    /// History of all method calls for verification.
    call_history: Vec<MockCall>,
    /// Optional error to return from start().
    start_error: Option<String>,
    /// Start time for computing monotonic timestamps.
    start_time: Instant,
    /// Monotonic counter for generating unique timestamps when Instant is not suitable.
    event_counter: u64,
}

impl MockInput {
    /// Create a new mock input source.
    pub fn new() -> Self {
        Self {
            input_queue: VecDeque::new(),
            output_log: Vec::new(),
            running: false,
            call_history: Vec::new(),
            start_error: None,
            start_time: Instant::now(),
            event_counter: 0,
        }
    }

    /// Configure start() to return an error.
    ///
    /// # Example
    /// ```ignore
    /// let mock = MockInput::new()
    ///     .with_error_on_start("Device busy");
    /// ```
    pub fn with_error_on_start(mut self, error: impl Into<String>) -> Self {
        self.start_error = Some(error.into());
        self
    }

    /// Queue an input event for the next poll.
    ///
    /// The event's metadata will be preserved as-is. Use `queue_key_event()`
    /// for automatic metadata population.
    pub fn queue_event(&mut self, event: InputEvent) {
        self.input_queue.push_back(event);
    }

    /// Queue a key event with automatically populated metadata.
    ///
    /// This is the preferred method for queueing events in tests, as it
    /// automatically sets mock-appropriate metadata:
    /// - `timestamp_us`: Monotonic counter (microseconds since MockInput creation)
    /// - `device_id`: Some("mock")
    /// - `is_repeat`: false
    /// - `is_synthetic`: false
    /// - `scan_code`: 0
    pub fn queue_key_event(&mut self, key: KeyCode, pressed: bool) {
        self.event_counter += 1;
        let timestamp_us = self.start_time.elapsed().as_micros() as u64;

        let event = InputEvent::with_metadata(
            key,
            pressed,
            timestamp_us,
            Some("mock".to_string()),
            false, // is_repeat
            false, // is_synthetic
            0,     // scan_code
            None,  // serial_number
        );
        self.input_queue.push_back(event);
    }

    /// Queue a key down event with automatically populated metadata.
    pub fn queue_key_down(&mut self, key: KeyCode) {
        self.queue_key_event(key, true);
    }

    /// Queue a key up event with automatically populated metadata.
    pub fn queue_key_up(&mut self, key: KeyCode) {
        self.queue_key_event(key, false);
    }

    /// Queue a repeat event (key held down causing auto-repeat).
    pub fn queue_key_repeat(&mut self, key: KeyCode) {
        self.event_counter += 1;
        let timestamp_us = self.start_time.elapsed().as_micros() as u64;

        let event = InputEvent::with_metadata(
            key,
            true, // pressed
            timestamp_us,
            Some("mock".to_string()),
            true,  // is_repeat
            false, // is_synthetic
            0,     // scan_code
            None,  // serial_number
        );
        self.input_queue.push_back(event);
    }

    /// Queue a synthetic event (simulating software-injected input).
    ///
    /// Useful for testing synthetic event filtering.
    pub fn queue_synthetic_event(&mut self, key: KeyCode, pressed: bool) {
        self.event_counter += 1;
        let timestamp_us = self.start_time.elapsed().as_micros() as u64;

        let event = InputEvent::with_metadata(
            key,
            pressed,
            timestamp_us,
            Some("mock".to_string()),
            false, // is_repeat
            true,  // is_synthetic
            0,     // scan_code
            None,  // serial_number
        );
        self.input_queue.push_back(event);
    }

    /// Get all captured output actions.
    pub fn output_log(&self) -> &[OutputAction] {
        &self.output_log
    }

    /// Clear the output log.
    pub fn clear_output_log(&mut self) {
        self.output_log.clear();
    }

    /// Get the history of all method calls.
    ///
    /// Useful for verifying the order and types of operations performed.
    pub fn call_history(&self) -> &[MockCall] {
        &self.call_history
    }

    /// Clear the call history.
    pub fn clear_call_history(&mut self) {
        self.call_history.clear();
    }
}

impl Default for MockInput {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl InputSource for MockInput {
    async fn poll_events(&mut self) -> Result<Vec<InputEvent>, KeyrxError> {
        self.call_history.push(MockCall::PollEvents);
        Ok(self.input_queue.drain(..).collect())
    }

    async fn send_output(&mut self, action: OutputAction) -> Result<(), KeyrxError> {
        self.call_history.push(MockCall::SendOutput(action.clone()));
        self.output_log.push(action);
        Ok(())
    }

    async fn start(&mut self) -> Result<(), KeyrxError> {
        use crate::errors::runtime::ENGINE_START_FAILED;

        self.call_history.push(MockCall::Start);
        if let Some(ref error) = self.start_error {
            return Err(keyrx_err!(ENGINE_START_FAILED, reason = error.clone()));
        }
        self.running = true;
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), KeyrxError> {
        self.call_history.push(MockCall::Stop);
        self.running = false;
        Ok(())
    }
}
