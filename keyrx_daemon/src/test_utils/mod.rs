//! Test utilities for virtual E2E testing.
//!
//! This module provides infrastructure for end-to-end testing of the keyrx daemon
//! without requiring physical keyboard hardware. It leverages Linux's uinput subsystem
//! to create virtual input devices that can inject key events into the kernel.
//!
//! # Components
//!
//! - [`VirtualKeyboard`]: Creates a virtual input device for injecting test key events
//! - [`OutputCapture`]: Captures events from the daemon's virtual output device
//!
//! # Requirements
//!
//! - Linux with uinput support (`/dev/uinput`)
//! - Read/write access to uinput (typically requires `input` group membership or root)
//!
//! # Example
//!
//! ```ignore
//! use keyrx_daemon::test_utils::{VirtualKeyboard, OutputCapture, VirtualDeviceError};
//!
//! // Create a virtual keyboard for input injection
//! let keyboard = VirtualKeyboard::create("test-keyboard")?;
//!
//! // Inject a key press
//! keyboard.inject(KeyEvent::Press(KeyCode::A))?;
//! ```

use std::io;
use thiserror::Error;

pub mod output_capture;
pub mod virtual_keyboard;

pub use output_capture::{
    assert_events, assert_events_msg, compare_events, CapturedEvent, EventAssertionResult,
    EventComparison, OutputCapture,
};
pub use virtual_keyboard::VirtualKeyboard;

/// Errors that can occur during virtual device operations.
///
/// These errors provide actionable messages to help diagnose and fix issues
/// when working with virtual input devices.
#[derive(Debug, Error)]
pub enum VirtualDeviceError {
    /// Permission denied when accessing uinput or evdev devices.
    ///
    /// This typically occurs when:
    /// - The user is not in the `input` group
    /// - `/dev/uinput` permissions are too restrictive
    ///
    /// # Fix
    ///
    /// Add your user to the `input` group:
    /// ```bash
    /// sudo usermod -aG input $USER
    /// # Log out and back in for changes to take effect
    /// ```
    ///
    /// Or temporarily grant access:
    /// ```bash
    /// sudo chmod 666 /dev/uinput
    /// ```
    #[error("permission denied: {message}\n\nFix: {fix_instruction}")]
    PermissionDenied {
        /// Description of what operation failed
        message: String,
        /// Instructions for fixing the permission issue
        fix_instruction: String,
    },

    /// Device was not found within the timeout period.
    ///
    /// This can occur when:
    /// - The target device name doesn't exist
    /// - The device hasn't been created yet (race condition)
    /// - The device was removed before it could be opened
    #[error("device not found: {name} (searched for {timeout_ms}ms)")]
    NotFound {
        /// Name of the device being searched for
        name: String,
        /// How long we waited before giving up
        timeout_ms: u64,
    },

    /// Operation timed out waiting for expected events.
    ///
    /// This can occur when:
    /// - The daemon is not processing events fast enough
    /// - The expected events were never generated
    /// - There's a deadlock or hang in the event pipeline
    #[error("timeout after {timeout_ms}ms waiting for {operation}")]
    Timeout {
        /// What operation timed out
        operation: String,
        /// How long we waited
        timeout_ms: u64,
    },

    /// I/O error during device operation.
    ///
    /// Wraps underlying system I/O errors that occur during device
    /// creation, reading, or writing.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Failed to create virtual device.
    ///
    /// This can occur when:
    /// - uinput device creation fails
    /// - Invalid device parameters were specified
    /// - System resource limits exceeded
    #[error("failed to create virtual device: {message}")]
    CreationFailed {
        /// Description of what went wrong
        message: String,
    },
}

impl VirtualDeviceError {
    /// Creates a PermissionDenied error for uinput access.
    pub fn uinput_permission_denied() -> Self {
        VirtualDeviceError::PermissionDenied {
            message: "cannot access /dev/uinput".to_string(),
            fix_instruction: "Add your user to the 'input' group: sudo usermod -aG input $USER\n\
                              Then log out and back in, or run: sudo chmod 666 /dev/uinput"
                .to_string(),
        }
    }

    /// Creates a PermissionDenied error for evdev access.
    pub fn evdev_permission_denied(device_path: &str) -> Self {
        VirtualDeviceError::PermissionDenied {
            message: format!("cannot access {device_path}"),
            fix_instruction: "Add your user to the 'input' group: sudo usermod -aG input $USER\n\
                              Then log out and back in."
                .to_string(),
        }
    }

    /// Creates a NotFound error with device name and timeout.
    pub fn device_not_found(name: &str, timeout_ms: u64) -> Self {
        VirtualDeviceError::NotFound {
            name: name.to_string(),
            timeout_ms,
        }
    }

    /// Creates a Timeout error for the specified operation.
    pub fn timeout(operation: &str, timeout_ms: u64) -> Self {
        VirtualDeviceError::Timeout {
            operation: operation.to_string(),
            timeout_ms,
        }
    }

    /// Creates a CreationFailed error with the given message.
    pub fn creation_failed(message: impl Into<String>) -> Self {
        VirtualDeviceError::CreationFailed {
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_permission_denied() {
        let err = VirtualDeviceError::uinput_permission_denied();
        let msg = err.to_string();
        assert!(msg.contains("permission denied"));
        assert!(msg.contains("/dev/uinput"));
        assert!(msg.contains("'input' group"));
    }

    #[test]
    fn test_error_display_not_found() {
        let err = VirtualDeviceError::device_not_found("test-device", 5000);
        let msg = err.to_string();
        assert!(msg.contains("test-device"));
        assert!(msg.contains("5000ms"));
    }

    #[test]
    fn test_error_display_timeout() {
        let err = VirtualDeviceError::timeout("reading events", 1000);
        let msg = err.to_string();
        assert!(msg.contains("timeout"));
        assert!(msg.contains("1000ms"));
        assert!(msg.contains("reading events"));
    }

    #[test]
    fn test_error_display_io() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err = VirtualDeviceError::from(io_err);
        let msg = err.to_string();
        assert!(msg.contains("I/O error"));
    }

    #[test]
    fn test_error_display_creation_failed() {
        let err = VirtualDeviceError::creation_failed("uinput not available");
        let msg = err.to_string();
        assert!(msg.contains("failed to create"));
        assert!(msg.contains("uinput not available"));
    }
}
