//! CLI exit code definitions.
//!
//! This module provides a unified set of exit codes used by
//! all CLI commands for consistent error reporting.
//!
//! Exit code semantics follow Unix conventions where 0 indicates
//! success and non-zero values indicate various failure modes.

/// Unified exit codes for all KeyRx CLI commands.
///
/// These codes provide consistent exit status across all commands:
/// - `keyrx test`
/// - `keyrx replay`
/// - `keyrx uat`
/// - `keyrx golden`
/// - `keyrx regression`
/// - `keyrx ci-check`
///
/// # Exit Code Semantics
///
/// | Code | Constant | Meaning |
/// |------|----------|---------|
/// | 0 | `SUCCESS` | Operation completed successfully |
/// | 1 | `ERROR` | General error (file not found, parse error, test failure) |
/// | 2 | `VERIFICATION_FAILED` | Verification/assertion/gate/regression failure |
/// | 3 | `SPECIAL` | Context-dependent (timeout, crash, confirmation required) |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum ExitCode {
    /// Operation completed successfully (code 0).
    ///
    /// Used when:
    /// - All tests pass
    /// - Replay completes without verification failure
    /// - All golden sessions match
    /// - Quality gate passes
    Success = 0,

    /// General error (code 1).
    ///
    /// Used when:
    /// - File not found
    /// - Parse/compilation error
    /// - Test failure (unit tests, UAT tests)
    /// - Runtime error
    Error = 1,

    /// Verification or assertion failure (code 2).
    ///
    /// Used when:
    /// - Test assertion fails
    /// - Replay output verification fails
    /// - Quality gate criteria not met
    /// - Regression detected (outputs don't match golden)
    VerificationFailed = 2,

    /// Context-dependent special condition (code 3).
    ///
    /// Used when:
    /// - Test timeout occurred
    /// - Crash detected during fuzz testing
    /// - Confirmation required but not provided
    Special = 3,
}

impl ExitCode {
    /// Convert to i32 for process exit.
    #[inline]
    pub const fn as_i32(self) -> i32 {
        self as i32
    }

    /// Convert to u8 for compatibility with some APIs.
    #[inline]
    pub const fn as_u8(self) -> u8 {
        self as i32 as u8
    }

    /// Check if this exit code indicates success.
    #[inline]
    pub const fn is_success(self) -> bool {
        matches!(self, ExitCode::Success)
    }

    /// Check if this exit code indicates any kind of failure.
    #[inline]
    pub const fn is_failure(self) -> bool {
        !matches!(self, ExitCode::Success)
    }
}

impl From<ExitCode> for i32 {
    fn from(code: ExitCode) -> i32 {
        code.as_i32()
    }
}

impl From<ExitCode> for u8 {
    fn from(code: ExitCode) -> u8 {
        code.as_u8()
    }
}

// Backward-compatible constants for gradual migration.
// These match the existing `exit_codes` module pattern used in CLI commands.

/// Operation completed successfully.
/// Alias for `ExitCode::Success.as_i32()`.
pub const SUCCESS: i32 = ExitCode::Success as i32;

/// Pass (alias for SUCCESS, used in test contexts).
pub const PASS: i32 = SUCCESS;

/// General error or test failure.
/// Alias for `ExitCode::Error.as_i32()`.
pub const ERROR: i32 = ExitCode::Error as i32;

/// Test failure (alias for ERROR, used in UAT/CI contexts).
pub const TEST_FAIL: i32 = ERROR;

/// Verification, assertion, gate, or regression failure.
/// Alias for `ExitCode::VerificationFailed.as_i32()`.
pub const VERIFICATION_FAILED: i32 = ExitCode::VerificationFailed as i32;

/// Assertion failure in tests (alias for VERIFICATION_FAILED).
pub const ASSERTION_FAIL: i32 = VERIFICATION_FAILED;

/// Quality gate failure (alias for VERIFICATION_FAILED).
pub const GATE_FAIL: i32 = VERIFICATION_FAILED;

/// Regression detected (alias for VERIFICATION_FAILED).
pub const REGRESSION: i32 = VERIFICATION_FAILED;

/// Special condition: timeout, crash, or confirmation required.
/// Alias for `ExitCode::Special.as_i32()`.
pub const SPECIAL: i32 = ExitCode::Special as i32;

/// Test timeout (alias for SPECIAL).
pub const TIMEOUT: i32 = SPECIAL;

/// Crash detected during fuzz testing (alias for SPECIAL).
pub const CRASH: i32 = SPECIAL;

/// Confirmation required but not provided (alias for SPECIAL).
pub const CONFIRMATION_REQUIRED: i32 = SPECIAL;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_code_values() {
        assert_eq!(ExitCode::Success.as_i32(), 0);
        assert_eq!(ExitCode::Error.as_i32(), 1);
        assert_eq!(ExitCode::VerificationFailed.as_i32(), 2);
        assert_eq!(ExitCode::Special.as_i32(), 3);
    }

    #[test]
    fn exit_code_u8_conversion() {
        assert_eq!(ExitCode::Success.as_u8(), 0u8);
        assert_eq!(ExitCode::Error.as_u8(), 1u8);
        assert_eq!(ExitCode::VerificationFailed.as_u8(), 2u8);
        assert_eq!(ExitCode::Special.as_u8(), 3u8);
    }

    #[test]
    fn constant_values_match_enum() {
        assert_eq!(SUCCESS, ExitCode::Success.as_i32());
        assert_eq!(PASS, ExitCode::Success.as_i32());
        assert_eq!(ERROR, ExitCode::Error.as_i32());
        assert_eq!(TEST_FAIL, ExitCode::Error.as_i32());
        assert_eq!(VERIFICATION_FAILED, ExitCode::VerificationFailed.as_i32());
        assert_eq!(ASSERTION_FAIL, ExitCode::VerificationFailed.as_i32());
        assert_eq!(GATE_FAIL, ExitCode::VerificationFailed.as_i32());
        assert_eq!(REGRESSION, ExitCode::VerificationFailed.as_i32());
        assert_eq!(SPECIAL, ExitCode::Special.as_i32());
        assert_eq!(TIMEOUT, ExitCode::Special.as_i32());
        assert_eq!(CRASH, ExitCode::Special.as_i32());
        assert_eq!(CONFIRMATION_REQUIRED, ExitCode::Special.as_i32());
    }

    #[test]
    fn is_success_and_is_failure() {
        assert!(ExitCode::Success.is_success());
        assert!(!ExitCode::Success.is_failure());

        assert!(!ExitCode::Error.is_success());
        assert!(ExitCode::Error.is_failure());

        assert!(!ExitCode::VerificationFailed.is_success());
        assert!(ExitCode::VerificationFailed.is_failure());

        assert!(!ExitCode::Special.is_success());
        assert!(ExitCode::Special.is_failure());
    }

    #[test]
    fn from_trait_implementations() {
        let code = ExitCode::Error;
        let i: i32 = code.into();
        let u: u8 = code.into();
        assert_eq!(i, 1);
        assert_eq!(u, 1);
    }

    #[test]
    fn backward_compatible_constants() {
        // Test commands use these values
        assert_eq!(PASS, 0);
        assert_eq!(ERROR, 1);
        assert_eq!(ASSERTION_FAIL, 2);
        assert_eq!(TIMEOUT, 3);

        // Replay commands use these values
        assert_eq!(SUCCESS, 0);
        assert_eq!(VERIFICATION_FAILED, 2);

        // UAT commands use these values
        assert_eq!(TEST_FAIL, 1);
        assert_eq!(GATE_FAIL, 2);
        assert_eq!(CRASH, 3);

        // Golden commands use these values
        assert_eq!(CONFIRMATION_REQUIRED, 3);

        // Regression commands use these values
        assert_eq!(REGRESSION, 2);
    }
}
