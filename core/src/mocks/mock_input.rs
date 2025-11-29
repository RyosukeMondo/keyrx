//! Mock input source for testing.

use crate::engine::{InputEvent, OutputAction};
use crate::traits::InputSource;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::VecDeque;

/// Mock input source for testing without real keyboard.
pub struct MockInput {
    /// Queued input events to return.
    input_queue: VecDeque<InputEvent>,
    /// Captured output actions.
    output_log: Vec<OutputAction>,
    /// Whether the mock is "running".
    running: bool,
}

impl MockInput {
    /// Create a new mock input source.
    pub fn new() -> Self {
        Self {
            input_queue: VecDeque::new(),
            output_log: Vec::new(),
            running: false,
        }
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
}

impl Default for MockInput {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl InputSource for MockInput {
    async fn poll_events(&mut self) -> Result<Vec<InputEvent>> {
        Ok(self.input_queue.drain(..).collect())
    }

    async fn send_output(&mut self, action: OutputAction) -> Result<()> {
        self.output_log.push(action);
        Ok(())
    }

    async fn start(&mut self) -> Result<()> {
        self.running = true;
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        self.running = false;
        Ok(())
    }
}
