//! macOS keyboard input capture using rdev.
//!
//! This module provides keyboard event capture using the rdev crate,
//! which wraps the macOS Accessibility API.

use crossbeam_channel::Receiver;
use keyrx_core::runtime::KeyEvent;

use crate::platform::{DeviceError, InputDevice};

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

impl InputDevice for MacosInputCapture {
    fn next_event(&mut self) -> Result<KeyEvent, DeviceError> {
        // Non-blocking receive from channel
        self.receiver
            .try_recv()
            .map_err(|e| match e {
                crossbeam_channel::TryRecvError::Empty => DeviceError::EndOfStream,
                crossbeam_channel::TryRecvError::Disconnected => {
                    DeviceError::Io(std::io::Error::other("Input channel disconnected"))
                }
            })
    }

    fn grab(&mut self) -> Result<(), DeviceError> {
        // macOS doesn't have an explicit grab mechanism like Linux's EVIOCGRAB.
        // Event capture via Accessibility API is already "exclusive" in the sense
        // that we control whether to propagate events or suppress them.
        Ok(())
    }

    fn release(&mut self) -> Result<(), DeviceError> {
        // No explicit release needed for macOS
        Ok(())
    }
}
