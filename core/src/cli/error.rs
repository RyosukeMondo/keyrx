//! CLI command error types with context and exit code information.
//!
//! This module provides structured error types for CLI commands, each with
//! associated exit codes and contextual information for debugging.

use super::{ExitCode, HasExitCode};
use std::path::PathBuf;
use thiserror::Error;

/// Structured command errors with context.
///
/// This enum provides specific error variants for common CLI failure modes,
/// each implementing `HasExitCode` to return the appropriate exit code.
///
/// # Example
///
/// ```
/// use keyrx_core::cli::{CommandError, ExitCode, HasExitCode};
///
/// let error = CommandError::ValidationFailed {
///     message: "Invalid configuration syntax".to_string(),
///     location: Some("config.toml:15:3".to_string()),
/// };
///
/// assert_eq!(error.exit_code(), ExitCode::ValidationFailed);
/// ```
#[derive(Debug, Error, Clone)]
pub enum CommandError {
    /// Configuration or script validation failed (exit code 4).
    ///
    /// Used when:
    /// - Configuration file has syntax errors
    /// - Script validation fails
    /// - Schema validation fails
    ///
    /// The `location` field should contain file:line:col information when available.
    #[error("Validation failed: {message}{}", location.as_ref().map(|l| format!(" at {}", l)).unwrap_or_default())]
    ValidationFailed {
        /// Error message describing what validation failed
        message: String,
        /// Optional location information (file:line:col)
        location: Option<String>,
    },

    /// Test assertion or verification failure (exit code 2).
    ///
    /// Used when:
    /// - Test assertion fails
    /// - Replay output verification fails
    /// - Quality gate criteria not met
    ///
    /// Includes pass/fail counts for test reporting.
    #[error("Test failure: {message} ({passed} passed, {failed} failed)")]
    TestFailure {
        /// Error message describing the test failure
        message: String,
        /// Number of tests that passed
        passed: usize,
        /// Number of tests that failed
        failed: usize,
    },

    /// Required device not found (exit code 6).
    ///
    /// Used when:
    /// - Input device doesn't exist
    /// - Device discovery fails
    /// - Device disconnected during operation
    #[error("Device not found: {message}{}", path.as_ref().map(|p| format!(" ({})", p.display())).unwrap_or_default())]
    DeviceNotFound {
        /// Error message describing which device was not found
        message: String,
        /// Optional path to the device that was not found
        path: Option<PathBuf>,
    },

    /// Insufficient permissions (exit code 5).
    ///
    /// Used when:
    /// - Cannot access device file
    /// - Cannot write to output directory
    /// - Insufficient privileges for operation
    #[error("Permission denied: {message}{}", path.as_ref().map(|p| format!(" ({})", p.display())).unwrap_or_default())]
    PermissionDenied {
        /// Error message describing the permission issue
        message: String,
        /// Optional path where permission was denied
        path: Option<PathBuf>,
    },

    /// Operation timed out (exit code 3).
    ///
    /// Used when:
    /// - Test execution exceeds timeout
    /// - Fuzz testing detects timeout
    /// - Device operation times out
    #[error("Operation timed out: {message} (timeout: {timeout_ms}ms)")]
    Timeout {
        /// Error message describing what timed out
        message: String,
        /// Timeout duration in milliseconds
        timeout_ms: u64,
    },

    /// General error with context (exit code 1).
    ///
    /// Used for errors that don't fit other categories:
    /// - File not found
    /// - Parse/compilation error
    /// - Runtime error
    #[error("{message}{}", context.as_ref().map(|s| format!(": {}", s)).unwrap_or_default())]
    Other {
        /// Error message
        message: String,
        /// Optional context or source error message
        context: Option<String>,
    },
}

impl HasExitCode for CommandError {
    fn exit_code(&self) -> ExitCode {
        match self {
            CommandError::ValidationFailed { .. } => ExitCode::ValidationFailed,
            CommandError::TestFailure { .. } => ExitCode::AssertionFailed,
            CommandError::DeviceNotFound { .. } => ExitCode::DeviceNotFound,
            CommandError::PermissionDenied { .. } => ExitCode::PermissionDenied,
            CommandError::Timeout { .. } => ExitCode::Timeout,
            CommandError::Other { .. } => ExitCode::GeneralError,
        }
    }
}

impl CommandError {
    /// Create a validation error with location information.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::cli::CommandError;
    ///
    /// let error = CommandError::validation("Invalid syntax", Some("config.toml:15:3"));
    /// ```
    pub fn validation(message: impl Into<String>, location: Option<impl Into<String>>) -> Self {
        Self::ValidationFailed {
            message: message.into(),
            location: location.map(|l| l.into()),
        }
    }

    /// Create a test failure error with pass/fail counts.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::cli::CommandError;
    ///
    /// let error = CommandError::test_failure("Some tests failed", 5, 2);
    /// ```
    pub fn test_failure(message: impl Into<String>, passed: usize, failed: usize) -> Self {
        Self::TestFailure {
            message: message.into(),
            passed,
            failed,
        }
    }

    /// Create a device not found error.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::cli::CommandError;
    /// use std::path::PathBuf;
    ///
    /// let error = CommandError::device_not_found(
    ///     "Input device not available",
    ///     Some(PathBuf::from("/dev/input/event0"))
    /// );
    /// ```
    pub fn device_not_found(message: impl Into<String>, path: Option<PathBuf>) -> Self {
        Self::DeviceNotFound {
            message: message.into(),
            path,
        }
    }

    /// Create a permission denied error.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::cli::CommandError;
    /// use std::path::PathBuf;
    ///
    /// let error = CommandError::permission_denied(
    ///     "Cannot access device",
    ///     Some(PathBuf::from("/dev/input/event0"))
    /// );
    /// ```
    pub fn permission_denied(message: impl Into<String>, path: Option<PathBuf>) -> Self {
        Self::PermissionDenied {
            message: message.into(),
            path,
        }
    }

    /// Create a timeout error.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::cli::CommandError;
    ///
    /// let error = CommandError::timeout("Test execution exceeded limit", 5000);
    /// ```
    pub fn timeout(message: impl Into<String>, timeout_ms: u64) -> Self {
        Self::Timeout {
            message: message.into(),
            timeout_ms,
        }
    }

    /// Create a general error.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::cli::CommandError;
    ///
    /// let error = CommandError::other("Something went wrong", None);
    /// ```
    pub fn other(message: impl Into<String>, context: Option<impl Into<String>>) -> Self {
        Self::Other {
            message: message.into(),
            context: context.map(|s| s.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_failed_exit_code() {
        let error = CommandError::ValidationFailed {
            message: "Invalid config".to_string(),
            location: Some("config.toml:10:5".to_string()),
        };
        assert_eq!(error.exit_code(), ExitCode::ValidationFailed);
    }

    #[test]
    fn validation_failed_display_with_location() {
        let error = CommandError::ValidationFailed {
            message: "Invalid syntax".to_string(),
            location: Some("config.toml:10:5".to_string()),
        };
        assert_eq!(
            error.to_string(),
            "Validation failed: Invalid syntax at config.toml:10:5"
        );
    }

    #[test]
    fn validation_failed_display_without_location() {
        let error = CommandError::ValidationFailed {
            message: "Invalid syntax".to_string(),
            location: None,
        };
        assert_eq!(error.to_string(), "Validation failed: Invalid syntax");
    }

    #[test]
    fn test_failure_exit_code() {
        let error = CommandError::TestFailure {
            message: "Assertions failed".to_string(),
            passed: 5,
            failed: 2,
        };
        assert_eq!(error.exit_code(), ExitCode::AssertionFailed);
    }

    #[test]
    fn test_failure_display() {
        let error = CommandError::TestFailure {
            message: "Some tests failed".to_string(),
            passed: 5,
            failed: 2,
        };
        assert_eq!(
            error.to_string(),
            "Test failure: Some tests failed (5 passed, 2 failed)"
        );
    }

    #[test]
    fn device_not_found_exit_code() {
        let error = CommandError::DeviceNotFound {
            message: "Device unavailable".to_string(),
            path: Some(PathBuf::from("/dev/input/event0")),
        };
        assert_eq!(error.exit_code(), ExitCode::DeviceNotFound);
    }

    #[test]
    fn device_not_found_display_with_path() {
        let error = CommandError::DeviceNotFound {
            message: "Device unavailable".to_string(),
            path: Some(PathBuf::from("/dev/input/event0")),
        };
        assert!(error
            .to_string()
            .contains("Device not found: Device unavailable"));
        assert!(error.to_string().contains("/dev/input/event0"));
    }

    #[test]
    fn device_not_found_display_without_path() {
        let error = CommandError::DeviceNotFound {
            message: "Device unavailable".to_string(),
            path: None,
        };
        assert_eq!(error.to_string(), "Device not found: Device unavailable");
    }

    #[test]
    fn permission_denied_exit_code() {
        let error = CommandError::PermissionDenied {
            message: "Cannot access".to_string(),
            path: Some(PathBuf::from("/dev/input/event0")),
        };
        assert_eq!(error.exit_code(), ExitCode::PermissionDenied);
    }

    #[test]
    fn permission_denied_display() {
        let error = CommandError::PermissionDenied {
            message: "Cannot access device".to_string(),
            path: Some(PathBuf::from("/dev/input/event0")),
        };
        assert!(error
            .to_string()
            .contains("Permission denied: Cannot access device"));
    }

    #[test]
    fn timeout_exit_code() {
        let error = CommandError::Timeout {
            message: "Test timed out".to_string(),
            timeout_ms: 5000,
        };
        assert_eq!(error.exit_code(), ExitCode::Timeout);
    }

    #[test]
    fn timeout_display() {
        let error = CommandError::Timeout {
            message: "Test timed out".to_string(),
            timeout_ms: 5000,
        };
        assert_eq!(
            error.to_string(),
            "Operation timed out: Test timed out (timeout: 5000ms)"
        );
    }

    #[test]
    fn other_exit_code() {
        let error = CommandError::Other {
            message: "Something went wrong".to_string(),
            context: None,
        };
        assert_eq!(error.exit_code(), ExitCode::GeneralError);
    }

    #[test]
    fn other_display_with_context() {
        let error = CommandError::Other {
            message: "Failed to process".to_string(),
            context: Some("file not found".to_string()),
        };
        assert_eq!(error.to_string(), "Failed to process: file not found");
    }

    #[test]
    fn other_display_without_context() {
        let error = CommandError::Other {
            message: "Failed to process".to_string(),
            context: None,
        };
        assert_eq!(error.to_string(), "Failed to process");
    }

    #[test]
    fn validation_constructor() {
        let error = CommandError::validation("Invalid syntax", Some("config.toml:10:5"));
        assert_eq!(error.exit_code(), ExitCode::ValidationFailed);
        assert!(error.to_string().contains("Invalid syntax"));
        assert!(error.to_string().contains("config.toml:10:5"));
    }

    #[test]
    fn test_failure_constructor() {
        let error = CommandError::test_failure("Some tests failed", 5, 2);
        assert_eq!(error.exit_code(), ExitCode::AssertionFailed);
        assert!(error.to_string().contains("5 passed"));
        assert!(error.to_string().contains("2 failed"));
    }

    #[test]
    fn device_not_found_constructor() {
        let error = CommandError::device_not_found(
            "Input device not available",
            Some(PathBuf::from("/dev/input/event0")),
        );
        assert_eq!(error.exit_code(), ExitCode::DeviceNotFound);
    }

    #[test]
    fn permission_denied_constructor() {
        let error =
            CommandError::permission_denied("Cannot access", Some(PathBuf::from("/dev/input")));
        assert_eq!(error.exit_code(), ExitCode::PermissionDenied);
    }

    #[test]
    fn timeout_constructor() {
        let error = CommandError::timeout("Test exceeded limit", 5000);
        assert_eq!(error.exit_code(), ExitCode::Timeout);
        assert!(error.to_string().contains("5000ms"));
    }

    #[test]
    fn other_constructor() {
        let error = CommandError::other("Failed", Some("IO error"));
        assert_eq!(error.exit_code(), ExitCode::GeneralError);
        assert!(error.to_string().contains("Failed"));
        assert!(error.to_string().contains("IO error"));
    }
}
