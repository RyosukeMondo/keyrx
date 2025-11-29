//! Input source trait for OS-level keyboard hooks.

use crate::engine::{InputEvent, OutputAction};
use anyhow::Result;
use async_trait::async_trait;

/// Trait for input sources (keyboard hooks, virtual devices).
///
/// Implementations:
/// - `WindowsInput`: WH_KEYBOARD_LL hook on Windows
/// - `LinuxInput`: evdev/uinput on Linux
/// - `MockInput`: Test mock for simulation
#[async_trait]
pub trait InputSource: Send {
    /// Poll for pending input events.
    async fn poll_events(&mut self) -> Result<Vec<InputEvent>>;

    /// Send an output action to the OS.
    async fn send_output(&mut self, action: OutputAction) -> Result<()>;

    /// Start capturing input.
    async fn start(&mut self) -> Result<()>;

    /// Stop capturing input.
    async fn stop(&mut self) -> Result<()>;
}
