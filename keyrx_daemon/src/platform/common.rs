//! Common types and errors for platform abstraction.
//!
//! This module defines shared types used across all platform implementations,
//! including device information and platform-specific errors.

use thiserror::Error;

/// Information about an input device.
///
/// This structure contains metadata about a keyboard or other input device
/// that can be used for key remapping.
///
/// # Examples
///
/// ```
/// use keyrx_daemon::platform::common::DeviceInfo;
///
/// let device = DeviceInfo {
///     id: "keyboard-0".to_string(),
///     name: "AT Translated Set 2 keyboard".to_string(),
///     path: "/dev/input/event3".to_string(),
///     vendor_id: 0x0001,
///     product_id: 0x0001,
/// };
///
/// println!("Device: {} ({})", device.name, device.path);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    /// Unique identifier for the device.
    ///
    /// This ID is used to reference the device in configuration and API calls.
    pub id: String,

    /// Human-readable name of the device.
    ///
    /// Examples: "AT Translated Set 2 keyboard", "Logitech USB Keyboard"
    pub name: String,

    /// System path to the device.
    ///
    /// - Linux: `/dev/input/eventX` path
    /// - Windows: Device instance path
    pub path: String,

    /// USB vendor ID.
    ///
    /// Standard USB vendor identifier (e.g., 0x046d for Logitech).
    pub vendor_id: u16,

    /// USB product ID.
    ///
    /// Standard USB product identifier.
    pub product_id: u16,
}

/// Errors that can occur during platform operations.
///
/// These errors cover common failure modes across all platform implementations,
/// including device access, initialization, and I/O errors.
#[derive(Error, Debug)]
pub enum PlatformError {
    /// The requested platform operation is not supported on this system.
    ///
    /// This error occurs when trying to use platform-specific functionality
    /// on an unsupported operating system.
    #[error("Operation not supported on this platform: {operation}")]
    Unsupported {
        /// Description of the unsupported operation.
        operation: String,
    },

    /// The specified device was not found.
    ///
    /// This typically occurs when a device is unplugged or the device path
    /// is invalid.
    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    /// No input arrived while the platform waited for it. The platform has
    /// already blocked for its wait interval, so callers need not sleep.
    #[error("No input within the wait interval")]
    NoInput,

    /// Failed to access a device.
    ///
    /// This error occurs when device access is denied or fails.
    /// On Linux, this usually means the user is not in the `input` group.
    /// On Windows, this means the process lacks administrator privileges.
    #[error("Failed to access device '{device}': {reason}. {suggestion}")]
    DeviceAccess {
        /// Name or path of the device that failed to be accessed.
        device: String,
        /// Reason for the access failure.
        reason: String,
        /// Suggestion for how to resolve the issue.
        suggestion: String,
    },

    /// Permission denied when accessing a device or resource (legacy variant).
    ///
    /// On Linux, this usually means the user is not in the `input` group.
    /// On Windows, this means the process lacks administrator privileges.
    ///
    /// # Resolution
    ///
    /// - Linux: Add user to `input` group or run with `sudo`
    /// - Windows: Run as administrator
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Platform initialization failed.
    ///
    /// This error occurs when the platform cannot be initialized due to
    /// missing resources, failed system calls, or other setup issues.
    #[error("Platform initialization failed: {reason}")]
    InitializationFailed {
        /// Reason for initialization failure.
        reason: String,
    },

    /// Mutex was poisoned (another thread panicked while holding the lock).
    ///
    /// This error indicates that a mutex protecting shared state has been
    /// poisoned. The recovery utilities in [`crate::platform::recovery`]
    /// can handle this gracefully by recovering the inner guard.
    #[error("Mutex poisoned: {0}")]
    Poisoned(String),

    /// Input/output operation failed.
    ///
    /// This error wraps underlying I/O errors from device operations.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Keyboard event injection failed.
    ///
    /// This occurs when attempting to inject a keyboard event but the
    /// operation fails (e.g., output device not initialized, uinput unavailable).
    #[error("Failed to inject keyboard event: {reason}. {suggestion}")]
    InjectionFailed {
        /// Reason for injection failure.
        reason: String,
        /// Suggestion for recovery or resolution.
        suggestion: String,
    },
}

/// Convenience type alias for Results using PlatformError.
pub type Result<T> = std::result::Result<T, PlatformError>;
