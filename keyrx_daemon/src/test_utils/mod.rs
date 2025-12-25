//! Test utilities for virtual E2E testing.
//!
//! This module provides infrastructure for end-to-end testing of the keyrx daemon
//! without requiring physical keyboard hardware. It leverages Linux's uinput subsystem
//! to create virtual input devices that can inject key events into the kernel.
//!
//! # Overview
//!
//! The virtual E2E testing framework enables automated testing of keyboard remapping
//! by creating software-based input/output devices. This allows tests to run in CI
//! environments without physical hardware access.
//!
//! # Components
//!
//! - [`VirtualKeyboard`]: Creates a virtual input device for injecting test key events
//! - [`OutputCapture`]: Captures events from the daemon's virtual output device
//! - [`VirtualDeviceError`]: Error type with actionable fix instructions
//! - [`assert_events`], [`compare_events`]: Assertion helpers for comparing captured vs expected events
//!
//! # Requirements
//!
//! - Linux with uinput support (`/dev/uinput`)
//! - Read/write access to uinput (typically requires `input` group membership or root)
//!
//! # Running Tests
//!
//! Tests that require uinput access use the [`skip_if_no_uinput!`] macro for runtime
//! permission checking. Tests will automatically run when uinput is accessible, or
//! gracefully skip with a message when permissions are not available.
//!
//! ```bash
//! # Run all tests (E2E tests auto-skip if uinput not accessible)
//! cargo test -p keyrx_daemon --features linux
//!
//! # To run E2E tests, ensure uinput access is configured:
//! # Option 1: Add user to uinput group (recommended, persistent)
//! sudo usermod -aG uinput $USER
//! # Log out and back in, then run tests normally
//!
//! # Option 2: Temporarily grant access (session only)
//! sudo chmod 666 /dev/uinput
//! cargo test -p keyrx_daemon --features linux
//! ```
//!
//! # Example: Basic Usage
//!
//! ```ignore
//! use keyrx_daemon::test_utils::{VirtualKeyboard, OutputCapture, VirtualDeviceError};
//! use keyrx_core::config::KeyCode;
//! use keyrx_core::runtime::event::KeyEvent;
//! use std::time::Duration;
//!
//! // Create a virtual keyboard for input injection
//! let mut keyboard = VirtualKeyboard::create("test-keyboard")?;
//!
//! // Inject key events
//! keyboard.inject(KeyEvent::Press(KeyCode::A))?;
//! keyboard.inject(KeyEvent::Release(KeyCode::A))?;
//!
//! // Device is automatically cleaned up when dropped
//! ```
//!
//! # Example: Full E2E Test Flow
//!
//! ```ignore
//! use keyrx_daemon::test_utils::{VirtualKeyboard, OutputCapture, assert_events};
//! use keyrx_core::config::KeyCode;
//! use keyrx_core::runtime::event::KeyEvent;
//! use std::time::Duration;
//!
//! // 1. Create virtual keyboard
//! let mut keyboard = VirtualKeyboard::create("e2e-test")?;
//!
//! // 2. Wait for the device to be registered
//! std::thread::sleep(Duration::from_millis(100));
//!
//! // 3. Find and open the device for output capture
//! let mut capture = OutputCapture::find_by_name(
//!     keyboard.name(),
//!     Duration::from_secs(5)
//! )?;
//!
//! // 4. Drain any pending events
//! capture.drain()?;
//!
//! // 5. Inject test input
//! keyboard.inject(KeyEvent::Press(KeyCode::A))?;
//! keyboard.inject(KeyEvent::Release(KeyCode::A))?;
//!
//! // 6. Capture output events
//! let captured = capture.collect_events(Duration::from_millis(100))?;
//!
//! // 7. Verify captured events match expected
//! let expected = vec![
//!     KeyEvent::Press(KeyCode::A),
//!     KeyEvent::Release(KeyCode::A),
//! ];
//! assert_events(&captured, &expected);
//! ```
//!
//! # Example: Using Assertion Helpers
//!
//! ```ignore
//! use keyrx_daemon::test_utils::{compare_events, assert_events_msg};
//! use keyrx_core::config::KeyCode;
//! use keyrx_core::runtime::event::KeyEvent;
//!
//! let captured = vec![KeyEvent::Press(KeyCode::B)];
//! let expected = vec![KeyEvent::Press(KeyCode::B)];
//!
//! // Get detailed comparison result
//! let result = compare_events(&captured, &expected);
//! if result.passed {
//!     println!("Test passed: {} matches", result.matches);
//! } else {
//!     println!("{}", result.format_diff());
//! }
//!
//! // Or use the assertion helper with custom message
//! assert_events_msg(&captured, &expected, "Testing A→B remapping");
//! ```

use std::fs::OpenOptions;
use std::io;
use thiserror::Error;

pub mod output_capture;
pub mod virtual_keyboard;

pub use output_capture::{
    assert_events, assert_events_msg, compare_events, CapturedEvent, EventAssertionResult,
    EventComparison, OutputCapture,
};
pub use virtual_keyboard::VirtualKeyboard;

/// Checks if uinput is accessible for virtual device operations.
///
/// Returns `true` if `/dev/uinput` exists and is readable/writable by the current process,
/// AND at least one `/dev/input/event*` device is accessible for reading (needed to capture
/// events from virtual devices).
///
/// Most E2E tests need both permissions:
/// - uinput access to create virtual keyboards
/// - input access to read events from those virtual keyboards
///
/// # Example
///
/// ```ignore
/// if can_access_uinput() {
///     // Run tests that require uinput + input access
/// } else {
///     eprintln!("Skipping: uinput or input devices not accessible");
/// }
/// ```
pub fn can_access_uinput() -> bool {
    #[cfg(target_os = "linux")]
    {
        // Check uinput access (for creating virtual devices)
        let uinput_ok = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/uinput")
            .is_ok();

        // Check input device access (for reading events from virtual devices)
        let input_ok = can_access_input_devices();

        uinput_ok && input_ok
    }

    #[cfg(target_os = "windows")]
    {
        // On Windows, we assume SendInput is always available.
        // In the future, we could check for UIPI or other restrictions.
        true
    }
}

/// Checks if input devices are accessible for reading.
///
/// Returns `true` if any `/dev/input/event*` device is readable by the current process.
/// This is used to determine whether tests requiring input device access can run.
pub fn can_access_input_devices() -> bool {
    #[cfg(target_os = "linux")]
    {
        for i in 0..20 {
            let path = format!("/dev/input/event{}", i);
            if OpenOptions::new().read(true).open(&path).is_ok() {
                return true;
            }
        }
        false
    }

    #[cfg(target_os = "windows")]
    {
        true
    }
}

/// Skips the current test if uinput is not accessible.
///
/// This macro checks for uinput access at runtime and returns early from the test
/// function with a skip message if access is not available. This allows E2E tests
/// to run automatically when permissions are configured, while gracefully skipping
/// on systems without uinput access.
///
/// # Usage
///
/// ```ignore
/// #[test]
/// fn test_virtual_keyboard() {
///     skip_if_no_uinput!();
///     // Test body - only runs if uinput is accessible
/// }
/// ```
///
/// # Note
///
/// Tests that skip via this macro will show as "passed" in test output, not "ignored".
/// The skip message is printed to stderr for visibility.
#[macro_export]
macro_rules! skip_if_no_uinput {
    () => {
        if !$crate::test_utils::can_access_uinput() {
            eprintln!(
                "SKIPPED: {} - uinput/input not accessible (add user to 'uinput' and 'input' groups or run with sudo)",
                module_path!()
            );
            return;
        }
    };
}

/// Skips the current test if input devices are not accessible.
///
/// This macro checks for `/dev/input/event*` access at runtime and returns early
/// from the test function with a skip message if access is not available.
///
/// # Usage
///
/// ```ignore
/// #[test]
/// fn test_evdev_device() {
///     skip_if_no_input_access!();
///     // Test body - only runs if input devices are accessible
/// }
/// ```
#[macro_export]
macro_rules! skip_if_no_input_access {
    () => {
        if !$crate::test_utils::can_access_input_devices() {
            eprintln!(
                "SKIPPED: {} - input devices not accessible (add user to 'input' group or run with sudo)",
                module_path!()
            );
            return;
        }
    };
}

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
            fix_instruction: "Add your user to the 'uinput' group: sudo usermod -aG uinput $USER\n\
                              Then log out and back in (or run: newgrp uinput)"
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
        assert!(msg.contains("'uinput' group"));
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
