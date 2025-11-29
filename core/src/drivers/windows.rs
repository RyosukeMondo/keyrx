//! Windows input driver using WH_KEYBOARD_LL.

use crate::engine::{InputEvent, OutputAction};
use crate::traits::InputSource;
use anyhow::Result;
use async_trait::async_trait;

/// Windows input source using low-level keyboard hook.
pub struct WindowsInput {
    running: bool,
}

impl WindowsInput {
    /// Create a new Windows input source.
    pub fn new() -> Result<Self> {
        Ok(Self { running: false })
    }
}

impl Default for WindowsInput {
    fn default() -> Self {
        Self { running: false }
    }
}

#[async_trait]
impl InputSource for WindowsInput {
    async fn poll_events(&mut self) -> Result<Vec<InputEvent>> {
        // TODO: Implement WH_KEYBOARD_LL hook
        Ok(vec![])
    }

    async fn send_output(&mut self, _action: OutputAction) -> Result<()> {
        // TODO: Implement SendInput
        Ok(())
    }

    async fn start(&mut self) -> Result<()> {
        self.running = true;
        // TODO: Install keyboard hook
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        self.running = false;
        // TODO: Uninstall keyboard hook
        Ok(())
    }
}
