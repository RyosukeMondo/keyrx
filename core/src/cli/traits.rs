//! CLI error trait for exit code extraction.
//!
//! This module provides the `HasExitCode` trait which allows any error type
//! to specify its intended exit code. This enables uniform exit code handling
//! across different error types.

use super::ExitCode;

/// Trait for types that can provide an exit code.
///
/// This trait allows different error types to specify their intended exit code,
/// enabling uniform exit code handling throughout the CLI.
///
/// # Example
///
/// ```
/// use keyrx_core::cli::{ExitCode, HasExitCode};
///
/// struct MyError {
///     message: String,
/// }
///
/// impl HasExitCode for MyError {
///     fn exit_code(&self) -> ExitCode {
///         ExitCode::GeneralError
///     }
/// }
///
/// let error = MyError {
///     message: "Something went wrong".to_string(),
/// };
/// assert_eq!(error.exit_code(), ExitCode::GeneralError);
/// ```
pub trait HasExitCode {
    /// Get the exit code for this error.
    fn exit_code(&self) -> ExitCode;
}

/// Implementation for `anyhow::Error`.
///
/// Attempts to downcast to known error types to extract specific exit codes.
/// Falls back to `GeneralError` if the error type is not recognized.
impl HasExitCode for anyhow::Error {
    fn exit_code(&self) -> ExitCode {
        // Try to downcast to std::io::Error first
        if let Some(io_err) = self.downcast_ref::<std::io::Error>() {
            return io_err.exit_code();
        }

        // Default to general error if we can't determine specifics
        ExitCode::GeneralError
    }
}

/// Implementation for `std::io::Error`.
///
/// Maps I/O error kinds to appropriate exit codes.
impl HasExitCode for std::io::Error {
    fn exit_code(&self) -> ExitCode {
        use std::io::ErrorKind;
        match self.kind() {
            ErrorKind::NotFound => ExitCode::DeviceNotFound,
            ErrorKind::PermissionDenied => ExitCode::PermissionDenied,
            ErrorKind::TimedOut => ExitCode::Timeout,
            ErrorKind::InvalidData | ErrorKind::InvalidInput => ExitCode::ValidationFailed,
            _ => ExitCode::GeneralError,
        }
    }
}

/// Implementation for `Box<dyn std::error::Error>`.
///
/// Attempts to downcast to known error types to extract specific exit codes.
impl HasExitCode for Box<dyn std::error::Error> {
    fn exit_code(&self) -> ExitCode {
        // Try to downcast to std::io::Error
        if let Some(io_err) = self.downcast_ref::<std::io::Error>() {
            return io_err.exit_code();
        }

        // Default to general error
        ExitCode::GeneralError
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn io_error_not_found() {
        let error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        assert_eq!(error.exit_code(), ExitCode::DeviceNotFound);
    }

    #[test]
    fn io_error_permission_denied() {
        let error = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        assert_eq!(error.exit_code(), ExitCode::PermissionDenied);
    }

    #[test]
    fn io_error_timed_out() {
        let error = io::Error::new(io::ErrorKind::TimedOut, "operation timed out");
        assert_eq!(error.exit_code(), ExitCode::Timeout);
    }

    #[test]
    fn io_error_invalid_data() {
        let error = io::Error::new(io::ErrorKind::InvalidData, "invalid data");
        assert_eq!(error.exit_code(), ExitCode::ValidationFailed);
    }

    #[test]
    fn io_error_other() {
        let error = io::Error::new(io::ErrorKind::Other, "other error");
        assert_eq!(error.exit_code(), ExitCode::GeneralError);
    }

    #[test]
    fn anyhow_with_io_error() {
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let anyhow_error: anyhow::Error = io_error.into();
        assert_eq!(anyhow_error.exit_code(), ExitCode::PermissionDenied);
    }

    #[test]
    fn anyhow_with_string_error() {
        let anyhow_error: anyhow::Error = anyhow::anyhow!("some error");
        assert_eq!(anyhow_error.exit_code(), ExitCode::GeneralError);
    }

    #[test]
    fn boxed_io_error() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "not found");
        let boxed: Box<dyn std::error::Error> = Box::new(io_error);
        assert_eq!(boxed.exit_code(), ExitCode::DeviceNotFound);
    }
}
