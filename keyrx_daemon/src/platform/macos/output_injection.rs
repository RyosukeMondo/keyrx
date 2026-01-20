//! macOS keyboard output injection using enigo.
//!
//! This module provides keyboard event injection using the enigo crate,
//! which uses CGEventPost for synthetic keyboard events.

use keyrx_core::runtime::KeyEvent;

use crate::platform::{DeviceError, OutputDevice};

/// macOS keyboard output injector.
///
/// Injects keyboard events using enigo::Enigo.
pub struct MacosOutputInjector {}

impl MacosOutputInjector {
    /// Creates a new output injector instance.
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for MacosOutputInjector {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputDevice for MacosOutputInjector {
    fn inject_event(&mut self, _event: KeyEvent) -> Result<(), DeviceError> {
        // Placeholder implementation - will be fully implemented in task 7
        // For now, just return Ok to make it compile
        Ok(())
    }
}
