//! Unified error types for platform-specific drivers.
//!
//! This module provides a comprehensive error type (`DriverError`) that covers
//! all possible failure scenarios across Windows and Linux drivers, with recovery
//! hints and actionable suggestions for users.

use std::io;
use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

/// Errors that can occur during driver operations.
///
/// Each error variant includes helpful context and hints for recovery.
/// Errors are classified as retryable or permanent to enable automatic recovery.
#[derive(Debug, Error)]
#[allow(dead_code)] // Infrastructure for future use
pub enum DriverError {
    /// Device not found at the specified path.
    #[error("Device not found: {path}")]
    DeviceNotFound {
        /// Path to the missing device.
        path: PathBuf,
    },

    /// Permission denied when accessing a resource.
    #[error("Permission denied: {resource}\nHint: {hint}")]
    PermissionDenied {
        /// Resource that couldn't be accessed.
        resource: String,
        /// Actionable hint for resolving the permission issue.
        hint: String,
    },

    /// Device was disconnected during operation.
    #[error("Device disconnected: {device}")]
    DeviceDisconnected {
        /// Name or path of the disconnected device.
        device: String,
    },

    /// Failed to install Windows keyboard hook.
    #[error("Hook installation failed with error code: {code:#x}")]
    HookFailed {
        /// Windows error code from GetLastError.
        code: u32,
    },

    /// Failed to unhook Windows keyboard hook.
    #[error("Failed to unhook keyboard hook: {reason}")]
    UnhookFailed {
        /// Reason for unhook failure.
        reason: String,
    },

    /// Failed to grab exclusive access to an input device.
    #[error("Failed to grab device: {reason}")]
    GrabFailed {
        /// Reason for grab failure.
        reason: String,
    },

    /// Failed to ungrab an input device.
    #[error("Failed to ungrab device: {reason}")]
    UngrabFailed {
        /// Reason for ungrab failure.
        reason: String,
    },

    /// Failed to create or access virtual input device (uinput).
    #[error("Virtual device error: {message}")]
    VirtualDeviceError {
        /// Error message.
        message: String,
    },

    /// Event injection failed.
    #[error("Event injection failed: {reason}")]
    InjectionFailed {
        /// Reason for injection failure.
        reason: String,
    },

    /// Invalid event data.
    #[error("Invalid event: {details}")]
    InvalidEvent {
        /// Details about why the event is invalid.
        details: String,
    },

    /// Temporary error that can be retried.
    #[error("Temporary error (retryable): {message}")]
    Temporary {
        /// Error message.
        message: String,
        /// Suggested duration before retry.
        retry_after: Duration,
    },

    /// Thread-local storage error.
    #[error("Thread-local storage error: {details}")]
    ThreadLocalError {
        /// Details about the thread-local error.
        details: String,
    },

    /// Callback panic was caught.
    #[error("Callback panicked: {panic_message}")]
    CallbackPanic {
        /// The panic message from the callback.
        panic_message: String,
    },

    /// Platform-specific I/O error.
    #[error("Platform I/O error: {0}")]
    Platform(#[from] io::Error),

    /// Generic driver initialization error.
    #[error("Driver initialization failed: {reason}")]
    InitializationFailed {
        /// Reason for initialization failure.
        reason: String,
    },

    /// Operation timed out.
    #[error("Operation timed out after {duration:?}")]
    Timeout {
        /// Duration after which the operation timed out.
        duration: Duration,
    },

    /// Channel send/receive error.
    #[error("Channel error: {details}")]
    ChannelError {
        /// Details about the channel error.
        details: String,
    },
}

#[allow(dead_code)] // Infrastructure for future use
impl DriverError {
    /// Returns whether this error is retryable.
    ///
    /// Retryable errors are temporary and may succeed if the operation is
    /// attempted again after a delay. Non-retryable errors are permanent
    /// and require user intervention or configuration changes.
    ///
    /// # Examples
    ///
    /// ```
    /// use keyrx_core::drivers::common::error::DriverError;
    /// use std::time::Duration;
    ///
    /// let temporary = DriverError::Temporary {
    ///     message: "Resource busy".to_string(),
    ///     retry_after: Duration::from_millis(100),
    /// };
    /// assert!(temporary.is_retryable());
    ///
    /// let permanent = DriverError::DeviceNotFound {
    ///     path: "/dev/input/event0".into(),
    /// };
    /// assert!(!permanent.is_retryable());
    /// ```
    pub fn is_retryable(&self) -> bool {
        match self {
            // Explicitly retryable errors
            DriverError::Temporary { .. } => true,
            DriverError::DeviceDisconnected { .. } => true,
            DriverError::Timeout { .. } => true,
            DriverError::GrabFailed { .. } => true,
            DriverError::InjectionFailed { .. } => true,

            // I/O errors that might be retryable
            DriverError::Platform(io_err) => matches!(
                io_err.kind(),
                io::ErrorKind::Interrupted
                    | io::ErrorKind::WouldBlock
                    | io::ErrorKind::TimedOut
                    | io::ErrorKind::BrokenPipe
                    | io::ErrorKind::ConnectionReset
            ),

            // Permanent errors requiring intervention
            DriverError::DeviceNotFound { .. }
            | DriverError::PermissionDenied { .. }
            | DriverError::HookFailed { .. }
            | DriverError::UnhookFailed { .. }
            | DriverError::UngrabFailed { .. }
            | DriverError::VirtualDeviceError { .. }
            | DriverError::InvalidEvent { .. }
            | DriverError::ThreadLocalError { .. }
            | DriverError::CallbackPanic { .. }
            | DriverError::InitializationFailed { .. }
            | DriverError::ChannelError { .. } => false,
        }
    }

    /// Returns a suggested action for resolving this error.
    ///
    /// This provides actionable guidance for users or automated recovery systems
    /// on how to handle the error.
    ///
    /// # Examples
    ///
    /// ```
    /// use keyrx_core::drivers::common::error::DriverError;
    /// use std::path::PathBuf;
    ///
    /// let error = DriverError::PermissionDenied {
    ///     resource: "/dev/input/event0".to_string(),
    ///     hint: "Add user to 'input' group".to_string(),
    /// };
    ///
    /// let action = error.suggested_action();
    /// assert!(action.contains("group") || action.contains("permission"));
    /// ```
    pub fn suggested_action(&self) -> &'static str {
        match self {
            DriverError::DeviceNotFound { .. } => {
                "Check that the device is connected and the path is correct"
            }
            DriverError::PermissionDenied { .. } => {
                "Check user permissions and group membership (see error hint)"
            }
            DriverError::DeviceDisconnected { .. } => {
                "Reconnect the device or wait for automatic reconnection"
            }
            DriverError::HookFailed { .. } => {
                "Restart the application or check for conflicting keyboard hooks"
            }
            DriverError::UnhookFailed { .. } => {
                "The hook may already be removed; this is usually safe to ignore"
            }
            DriverError::GrabFailed { .. } => {
                "Close other applications using the device or check device permissions"
            }
            DriverError::UngrabFailed { .. } => {
                "The device may already be ungrabbed; this is usually safe to ignore"
            }
            DriverError::VirtualDeviceError { .. } => {
                "Check that /dev/uinput exists and is accessible"
            }
            DriverError::InjectionFailed { .. } => "Retry the operation or check device status",
            DriverError::InvalidEvent { .. } => {
                "Check the event data and ensure it matches the expected format"
            }
            DriverError::Temporary { .. } => "Wait and retry; this error is temporary",
            DriverError::ThreadLocalError { .. } => {
                "This is an internal error; please report it as a bug"
            }
            DriverError::CallbackPanic { .. } => {
                "This is an internal error; please report it with the panic message"
            }
            DriverError::Platform(_) => {
                "Check system logs and ensure the driver has necessary permissions"
            }
            DriverError::InitializationFailed { .. } => {
                "Check driver configuration and system requirements"
            }
            DriverError::Timeout { .. } => "Retry the operation or increase the timeout duration",
            DriverError::ChannelError { .. } => {
                "This is an internal communication error; try restarting the application"
            }
        }
    }

    /// Returns the retry delay for retryable errors.
    ///
    /// For retryable errors, this provides a suggested duration to wait before
    /// retrying the operation. For non-retryable errors, returns `None`.
    pub fn retry_delay(&self) -> Option<Duration> {
        match self {
            DriverError::Temporary { retry_after, .. } => Some(*retry_after),
            DriverError::DeviceDisconnected { .. } => Some(Duration::from_millis(500)),
            DriverError::Timeout { .. } => Some(Duration::from_millis(100)),
            DriverError::GrabFailed { .. } => Some(Duration::from_millis(200)),
            DriverError::InjectionFailed { .. } => Some(Duration::from_millis(50)),
            _ => None,
        }
    }

    /// Creates a permission denied error with Linux-specific hints.
    ///
    /// This is a convenience constructor for Linux permission errors that
    /// provides helpful hints about group membership and udev rules.
    #[cfg(target_os = "linux")]
    pub fn linux_permission_denied(resource: impl Into<String>) -> Self {
        DriverError::PermissionDenied {
            resource: resource.into(),
            hint: "Add your user to the 'input' group: sudo usermod -aG input $USER\n\
                   Or configure udev rules for device access"
                .to_string(),
        }
    }

    /// Creates a permission denied error with Windows-specific hints.
    ///
    /// This is a convenience constructor for Windows permission errors.
    #[cfg(windows)]
    pub fn windows_permission_denied(resource: impl Into<String>) -> Self {
        DriverError::PermissionDenied {
            resource: resource.into(),
            hint: "Run the application with administrator privileges if required".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temporary_errors_are_retryable() {
        let err = DriverError::Temporary {
            message: "test".to_string(),
            retry_after: Duration::from_millis(100),
        };
        assert!(err.is_retryable());
    }

    #[test]
    fn device_disconnected_is_retryable() {
        let err = DriverError::DeviceDisconnected {
            device: "keyboard".to_string(),
        };
        assert!(err.is_retryable());
    }

    #[test]
    fn permission_denied_not_retryable() {
        let err = DriverError::PermissionDenied {
            resource: "/dev/input/event0".to_string(),
            hint: "add to input group".to_string(),
        };
        assert!(!err.is_retryable());
    }

    #[test]
    fn device_not_found_not_retryable() {
        let err = DriverError::DeviceNotFound {
            path: PathBuf::from("/dev/input/event99"),
        };
        assert!(!err.is_retryable());
    }

    #[test]
    fn interrupted_io_is_retryable() {
        let io_err = io::Error::from(io::ErrorKind::Interrupted);
        let err = DriverError::Platform(io_err);
        assert!(err.is_retryable());
    }

    #[test]
    fn permission_io_not_retryable() {
        let io_err = io::Error::from(io::ErrorKind::PermissionDenied);
        let err = DriverError::Platform(io_err);
        assert!(!err.is_retryable());
    }

    #[test]
    fn suggested_actions_are_helpful() {
        let err = DriverError::PermissionDenied {
            resource: "/dev/input/event0".to_string(),
            hint: "hint".to_string(),
        };
        let action = err.suggested_action();
        assert!(action.contains("permission") || action.contains("group"));
    }

    #[test]
    fn retry_delay_for_temporary() {
        let err = DriverError::Temporary {
            message: "test".to_string(),
            retry_after: Duration::from_millis(250),
        };
        assert_eq!(err.retry_delay(), Some(Duration::from_millis(250)));
    }

    #[test]
    fn retry_delay_for_device_disconnected() {
        let err = DriverError::DeviceDisconnected {
            device: "keyboard".to_string(),
        };
        assert!(err.retry_delay().is_some());
    }

    #[test]
    fn no_retry_delay_for_permanent_errors() {
        let err = DriverError::DeviceNotFound {
            path: PathBuf::from("/dev/input/event0"),
        };
        assert_eq!(err.retry_delay(), None);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_permission_denied_has_hint() {
        let err = DriverError::linux_permission_denied("/dev/input/event0");
        match err {
            DriverError::PermissionDenied { hint, .. } => {
                assert!(hint.contains("input"));
                assert!(hint.contains("udev"));
            }
            _ => panic!("Expected PermissionDenied"),
        }
    }

    #[cfg(windows)]
    #[test]
    fn windows_permission_denied_has_hint() {
        let err = DriverError::windows_permission_denied("keyboard hook");
        match err {
            DriverError::PermissionDenied { hint, .. } => {
                assert!(hint.contains("administrator"));
            }
            _ => panic!("Expected PermissionDenied"),
        }
    }

    #[test]
    fn error_display_formatting() {
        let err = DriverError::DeviceNotFound {
            path: PathBuf::from("/dev/input/event0"),
        };
        let display = format!("{}", err);
        assert!(display.contains("/dev/input/event0"));
    }

    #[test]
    fn hook_failed_shows_hex_code() {
        let err = DriverError::HookFailed { code: 0x1234 };
        let display = format!("{}", err);
        assert!(display.contains("0x1234"));
    }

    #[test]
    fn callback_panic_preserves_message() {
        let err = DriverError::CallbackPanic {
            panic_message: "test panic".to_string(),
        };
        let display = format!("{}", err);
        assert!(display.contains("test panic"));
    }

    #[test]
    fn timeout_shows_duration() {
        let err = DriverError::Timeout {
            duration: Duration::from_secs(5),
        };
        let display = format!("{}", err);
        assert!(display.contains("5"));
    }
}
