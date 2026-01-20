//! macOS keyboard input capture using rdev.
//!
//! This module provides keyboard event capture using the rdev crate,
//! which wraps the macOS Accessibility API.

use crossbeam_channel::Receiver;
use keyrx_core::runtime::KeyEvent;

/// macOS keyboard input capture.
///
/// Captures keyboard events using rdev::listen on a background thread.
pub struct MacosInputCapture {
    receiver: Receiver<KeyEvent>,
}

impl MacosInputCapture {
    /// Creates a new input capture instance.
    ///
    /// # Arguments
    ///
    /// * `receiver` - Channel receiver for keyboard events
    pub fn new(receiver: Receiver<KeyEvent>) -> Self {
        Self { receiver }
    }
}
