//! Mock input source for testing.

use crate::engine::{InputEvent, OutputAction};
use crate::traits::InputSource;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::VecDeque;

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
    pub fn queue_event(&mut self, event: InputEvent) {
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
    async fn poll_events(&mut self) -> Result<Vec<InputEvent>> {
        self.call_history.push(MockCall::PollEvents);
        Ok(self.input_queue.drain(..).collect())
    }

    async fn send_output(&mut self, action: OutputAction) -> Result<()> {
        self.call_history.push(MockCall::SendOutput(action.clone()));
        self.output_log.push(action);
        Ok(())
    }

    async fn start(&mut self) -> Result<()> {
        self.call_history.push(MockCall::Start);
        if let Some(ref error) = self.start_error {
            return Err(anyhow::anyhow!("{}", error));
        }
        self.running = true;
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        self.call_history.push(MockCall::Stop);
        self.running = false;
        Ok(())
    }
}
