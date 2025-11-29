//! Linux input driver using evdev/uinput.

use crate::engine::{InputEvent, OutputAction};
use crate::traits::InputSource;
use anyhow::Result;
use async_trait::async_trait;

/// Linux input source using evdev for capture and uinput for injection.
pub struct LinuxInput {
    running: bool,
}

impl LinuxInput {
    /// Create a new Linux input source.
    pub fn new() -> Result<Self> {
        Ok(Self { running: false })
    }
}

impl Default for LinuxInput {
    fn default() -> Self {
        Self { running: false }
    }
}

#[async_trait]
impl InputSource for LinuxInput {
    async fn poll_events(&mut self) -> Result<Vec<InputEvent>> {
        // TODO: Implement evdev polling
        Ok(vec![])
    }

    async fn send_output(&mut self, _action: OutputAction) -> Result<()> {
        // TODO: Implement uinput injection
        Ok(())
    }

    async fn start(&mut self) -> Result<()> {
        self.running = true;
        // TODO: Open evdev device and create uinput device
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        self.running = false;
        // TODO: Close devices
        Ok(())
    }
}
