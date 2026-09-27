//! Platform-specific operation error types.

use std::io;
use thiserror::Error;

/// Platform-specific operation errors.
///
/// This error type covers failures in platform-specific operations such as
/// device access, mutex operations, and platform initialization.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum PlatformError {
    /// Mutex was poisoned (another thread panicked while holding the lock).
    #[error("Mutex poisoned: {0}")]
    Poisoned(String),

    /// Platform initialization failed.
    #[error("Platform initialization failed: {reason}")]
    InitializationFailed {
        /// Reason for initialization failure.
        reason: String,
    },

    /// Device access failed (e.g., permission denied, device not found).
    #[error("Failed to access device '{device}': {reason}. {suggestion}")]
    DeviceAccess {
        /// Name or path of the device that failed to be accessed.
        device: String,
        /// Reason for the access failure.
        reason: String,
        /// Suggestion for how to resolve the issue.
        suggestion: String,
    },

    /// Keyboard event injection failed.
    #[error("Failed to inject keyboard event: {reason}. {suggestion}")]
    InjectionFailed {
        /// Reason for injection failure.
        reason: String,
        /// Suggestion for recovery or resolution.
        suggestion: String,
    },

    /// Requested platform operation is not supported.
    #[error("Operation not supported on this platform: {operation}")]
    Unsupported {
        /// Description of the unsupported operation.
        operation: String,
    },

    /// Device operation failed (legacy variant, prefer more specific variants).
    #[error("Device operation failed: {0}")]
    DeviceError(String),

    /// IO error occurred during platform operation.
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}
