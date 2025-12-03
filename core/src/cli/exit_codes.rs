//! CLI exit code definitions.
//!
//! This module provides a type-safe exit code system used by
//! all CLI commands for consistent error reporting.
//!
//! Exit code semantics follow Unix conventions where 0 indicates
//! success and non-zero values indicate various failure modes.

use std::fmt;

/// Unified exit codes for all KeyRx CLI commands.
///
/// These codes provide consistent exit status across all commands:
/// - `keyrx test`
/// - `keyrx replay`
/// - `keyrx uat`
/// - `keyrx check`
/// - `keyrx simulate`
/// - And all other CLI commands
///
/// # Exit Code Semantics
///
/// | Code | Variant | Meaning |
/// |------|---------|---------|
/// | 0 | `Success` | Operation completed successfully |
/// | 1 | `GeneralError` | General error (file not found, parse error, runtime error) |
/// | 2 | `AssertionFailed` | Test assertion or verification failure |
/// | 3 | `Timeout` | Operation timed out |
/// | 4 | `ValidationFailed` | Configuration or script validation failed |
/// | 5 | `PermissionDenied` | Insufficient permissions for operation |
/// | 6 | `DeviceNotFound` | Required device not found |
/// | 7 | `ScriptError` | Script execution error |
/// | 101 | `Panic` | Unhandled panic (Rust convention) |
///
/// # Example
///
/// ```
/// use keyrx_core::cli::ExitCode;
///
/// let code = ExitCode::Success;
/// assert_eq!(code.as_u8(), 0);
/// assert!(code.is_success());
///
/// let error = ExitCode::ValidationFailed;
/// assert_eq!(error.as_u8(), 4);
/// assert!(error.is_failure());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ExitCode {
    /// Operation completed successfully (code 0).
    ///
    /// Used when:
    /// - All tests pass
    /// - Replay completes without verification failure
    /// - All golden sessions match
    /// - Quality gate passes
    /// - Command execution succeeds
    Success = 0,

    /// General error (code 1).
    ///
    /// Used when:
    /// - File not found
    /// - Parse/compilation error
    /// - Runtime error
    /// - Generic command failure
    GeneralError = 1,

    /// Test assertion or verification failure (code 2).
    ///
    /// Used when:
    /// - Test assertion fails
    /// - Replay output verification fails
    /// - Quality gate criteria not met
    /// - Regression detected (outputs don't match golden)
    AssertionFailed = 2,

    /// Operation timed out (code 3).
    ///
    /// Used when:
    /// - Test execution exceeds timeout
    /// - Fuzz testing detects timeout
    /// - Device operation times out
    Timeout = 3,

    /// Configuration or script validation failed (code 4).
    ///
    /// Used when:
    /// - Configuration file has syntax errors
    /// - Script validation fails
    /// - Schema validation fails
    ValidationFailed = 4,

    /// Insufficient permissions (code 5).
    ///
    /// Used when:
    /// - Cannot access device file
    /// - Cannot write to output directory
    /// - Insufficient privileges for operation
    PermissionDenied = 5,

    /// Required device not found (code 6).
    ///
    /// Used when:
    /// - Input device doesn't exist
    /// - Device discovery fails
    /// - Device disconnected during operation
    DeviceNotFound = 6,

    /// Script execution error (code 7).
    ///
    /// Used when:
    /// - Rhai script runtime error
    /// - Script compilation error
    /// - Script API misuse
    ScriptError = 7,

    /// Unhandled panic (code 101).
    ///
    /// Used when:
    /// - Rust panic occurs
    /// - Unrecoverable error detected
    ///
    /// Note: This follows Rust convention for panic exit codes.
    Panic = 101,
}

impl ExitCode {
    /// Convert to u8 for process exit.
    #[inline]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    /// Convert to i32 for compatibility with some APIs.
    #[inline]
    pub const fn as_i32(self) -> i32 {
        self as u8 as i32
    }

    /// Try to create an ExitCode from a u8 value.
    ///
    /// Returns None if the value doesn't correspond to a valid exit code.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::cli::ExitCode;
    ///
    /// assert_eq!(ExitCode::from_u8(0), Some(ExitCode::Success));
    /// assert_eq!(ExitCode::from_u8(2), Some(ExitCode::AssertionFailed));
    /// assert_eq!(ExitCode::from_u8(99), None);
    /// ```
    pub const fn from_u8(code: u8) -> Option<ExitCode> {
        match code {
            0 => Some(ExitCode::Success),
            1 => Some(ExitCode::GeneralError),
            2 => Some(ExitCode::AssertionFailed),
            3 => Some(ExitCode::Timeout),
            4 => Some(ExitCode::ValidationFailed),
            5 => Some(ExitCode::PermissionDenied),
            6 => Some(ExitCode::DeviceNotFound),
            7 => Some(ExitCode::ScriptError),
            101 => Some(ExitCode::Panic),
            _ => None,
        }
    }

    /// Convert to std::process::ExitCode for main return.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use keyrx_core::cli::ExitCode;
    ///
    /// fn main() -> std::process::ExitCode {
    ///     let result = ExitCode::Success;
    ///     result.as_process_code()
    /// }
    /// ```
    #[inline]
    pub fn as_process_code(self) -> std::process::ExitCode {
        std::process::ExitCode::from(self.as_u8())
    }

    /// Get a human-readable description of this exit code.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::cli::ExitCode;
    ///
    /// assert_eq!(
    ///     ExitCode::ValidationFailed.description(),
    ///     "Configuration or script validation failed"
    /// );
    /// ```
    pub const fn description(self) -> &'static str {
        match self {
            ExitCode::Success => "Operation completed successfully",
            ExitCode::GeneralError => "General error occurred",
            ExitCode::AssertionFailed => "Test assertion or verification failed",
            ExitCode::Timeout => "Operation timed out",
            ExitCode::ValidationFailed => "Configuration or script validation failed",
            ExitCode::PermissionDenied => "Insufficient permissions",
            ExitCode::DeviceNotFound => "Required device not found",
            ExitCode::ScriptError => "Script execution error",
            ExitCode::Panic => "Unhandled panic",
        }
    }

    /// Check if this exit code indicates success.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::cli::ExitCode;
    ///
    /// assert!(ExitCode::Success.is_success());
    /// assert!(!ExitCode::GeneralError.is_success());
    /// ```
    #[inline]
    pub const fn is_success(self) -> bool {
        matches!(self, ExitCode::Success)
    }

    /// Check if this exit code indicates any kind of failure.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::cli::ExitCode;
    ///
    /// assert!(!ExitCode::Success.is_failure());
    /// assert!(ExitCode::GeneralError.is_failure());
    /// ```
    #[inline]
    pub const fn is_failure(self) -> bool {
        !self.is_success()
    }
}

impl fmt::Display for ExitCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.description(), self.as_u8())
    }
}

impl From<ExitCode> for u8 {
    fn from(code: ExitCode) -> u8 {
        code.as_u8()
    }
}

impl From<ExitCode> for i32 {
    fn from(code: ExitCode) -> i32 {
        code.as_i32()
    }
}

impl From<ExitCode> for std::process::ExitCode {
    fn from(code: ExitCode) -> std::process::ExitCode {
        code.as_process_code()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_code_values() {
        assert_eq!(ExitCode::Success.as_u8(), 0);
        assert_eq!(ExitCode::GeneralError.as_u8(), 1);
        assert_eq!(ExitCode::AssertionFailed.as_u8(), 2);
        assert_eq!(ExitCode::Timeout.as_u8(), 3);
        assert_eq!(ExitCode::ValidationFailed.as_u8(), 4);
        assert_eq!(ExitCode::PermissionDenied.as_u8(), 5);
        assert_eq!(ExitCode::DeviceNotFound.as_u8(), 6);
        assert_eq!(ExitCode::ScriptError.as_u8(), 7);
        assert_eq!(ExitCode::Panic.as_u8(), 101);
    }

    #[test]
    fn exit_code_i32_conversion() {
        assert_eq!(ExitCode::Success.as_i32(), 0);
        assert_eq!(ExitCode::GeneralError.as_i32(), 1);
        assert_eq!(ExitCode::Panic.as_i32(), 101);
    }

    #[test]
    fn is_success_and_is_failure() {
        assert!(ExitCode::Success.is_success());
        assert!(!ExitCode::Success.is_failure());

        assert!(!ExitCode::GeneralError.is_success());
        assert!(ExitCode::GeneralError.is_failure());

        assert!(!ExitCode::AssertionFailed.is_success());
        assert!(ExitCode::AssertionFailed.is_failure());

        assert!(!ExitCode::Timeout.is_success());
        assert!(ExitCode::Timeout.is_failure());

        assert!(!ExitCode::ValidationFailed.is_success());
        assert!(ExitCode::ValidationFailed.is_failure());

        assert!(!ExitCode::PermissionDenied.is_success());
        assert!(ExitCode::PermissionDenied.is_failure());

        assert!(!ExitCode::DeviceNotFound.is_success());
        assert!(ExitCode::DeviceNotFound.is_failure());

        assert!(!ExitCode::ScriptError.is_success());
        assert!(ExitCode::ScriptError.is_failure());

        assert!(!ExitCode::Panic.is_success());
        assert!(ExitCode::Panic.is_failure());
    }

    #[test]
    fn from_trait_implementations() {
        let code = ExitCode::ValidationFailed;
        let u: u8 = code.into();
        let i: i32 = code.into();
        assert_eq!(u, 4);
        assert_eq!(i, 4);
    }

    #[test]
    fn display_implementation() {
        assert_eq!(
            ExitCode::Success.to_string(),
            "Operation completed successfully (0)"
        );
        assert_eq!(
            ExitCode::ValidationFailed.to_string(),
            "Configuration or script validation failed (4)"
        );
        assert_eq!(ExitCode::Panic.to_string(), "Unhandled panic (101)");
    }

    #[test]
    fn descriptions() {
        assert_eq!(
            ExitCode::Success.description(),
            "Operation completed successfully"
        );
        assert_eq!(
            ExitCode::GeneralError.description(),
            "General error occurred"
        );
        assert_eq!(
            ExitCode::AssertionFailed.description(),
            "Test assertion or verification failed"
        );
        assert_eq!(ExitCode::Timeout.description(), "Operation timed out");
        assert_eq!(
            ExitCode::ValidationFailed.description(),
            "Configuration or script validation failed"
        );
        assert_eq!(
            ExitCode::PermissionDenied.description(),
            "Insufficient permissions"
        );
        assert_eq!(
            ExitCode::DeviceNotFound.description(),
            "Required device not found"
        );
        assert_eq!(
            ExitCode::ScriptError.description(),
            "Script execution error"
        );
        assert_eq!(ExitCode::Panic.description(), "Unhandled panic");
    }

    #[test]
    fn from_u8_roundtrip() {
        assert_eq!(ExitCode::from_u8(0), Some(ExitCode::Success));
        assert_eq!(ExitCode::from_u8(1), Some(ExitCode::GeneralError));
        assert_eq!(ExitCode::from_u8(2), Some(ExitCode::AssertionFailed));
        assert_eq!(ExitCode::from_u8(3), Some(ExitCode::Timeout));
        assert_eq!(ExitCode::from_u8(4), Some(ExitCode::ValidationFailed));
        assert_eq!(ExitCode::from_u8(5), Some(ExitCode::PermissionDenied));
        assert_eq!(ExitCode::from_u8(6), Some(ExitCode::DeviceNotFound));
        assert_eq!(ExitCode::from_u8(7), Some(ExitCode::ScriptError));
        assert_eq!(ExitCode::from_u8(101), Some(ExitCode::Panic));
        assert_eq!(ExitCode::from_u8(99), None);
    }
}
