//! CLI command result type that carries exit codes.
//!
//! This module provides the `CommandResult<T>` type which wraps a command's
//! return value along with an exit code and optional messages. This allows
//! commands to propagate exit codes through the type system rather than
//! relying on string parsing or side effects.

use super::ExitCode;
use std::fmt;

/// Result of a CLI command execution.
///
/// This type carries both the result value and the intended exit code,
/// allowing commands to communicate success/failure through the type system.
///
/// # Example
///
/// ```
/// use keyrx_core::cli::{CommandResult, ExitCode};
///
/// // Success with value
/// let result = CommandResult::success(42);
/// assert!(result.is_success());
/// assert_eq!(result.value(), Some(42));
///
/// // Failure with message
/// let result: CommandResult<()> = CommandResult::failure(
///     ExitCode::ValidationFailed,
///     "Invalid configuration"
/// );
/// assert!(!result.is_success());
/// assert_eq!(result.exit_code(), ExitCode::ValidationFailed);
/// ```
#[derive(Debug, Clone)]
pub struct CommandResult<T> {
    value: Option<T>,
    exit_code: ExitCode,
    messages: Vec<String>,
}

impl<T> CommandResult<T> {
    /// Create a successful result with a value.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::cli::CommandResult;
    ///
    /// let result = CommandResult::success(42);
    /// assert!(result.is_success());
    /// assert_eq!(result.value(), Some(42));
    /// ```
    pub fn success(value: T) -> Self {
        Self {
            value: Some(value),
            exit_code: ExitCode::Success,
            messages: Vec::new(),
        }
    }

    /// Create a successful result with a value and a message.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::cli::CommandResult;
    ///
    /// let result = CommandResult::success_with_message(42, "Operation completed");
    /// assert!(result.is_success());
    /// ```
    pub fn success_with_message(value: T, msg: impl Into<String>) -> Self {
        Self {
            value: Some(value),
            exit_code: ExitCode::Success,
            messages: vec![msg.into()],
        }
    }

    /// Create a failure result with an exit code and message.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::cli::{CommandResult, ExitCode};
    ///
    /// let result: CommandResult<()> = CommandResult::failure(
    ///     ExitCode::ValidationFailed,
    ///     "Invalid configuration"
    /// );
    /// assert!(!result.is_success());
    /// assert_eq!(result.exit_code(), ExitCode::ValidationFailed);
    /// ```
    pub fn failure(code: ExitCode, msg: impl Into<String>) -> Self {
        Self {
            value: None,
            exit_code: code,
            messages: vec![msg.into()],
        }
    }

    /// Add a message to this result.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::cli::CommandResult;
    ///
    /// let result = CommandResult::success(())
    ///     .with_message("Step 1 completed")
    ///     .with_message("Step 2 completed");
    /// ```
    pub fn with_message(mut self, msg: impl Into<String>) -> Self {
        self.messages.push(msg.into());
        self
    }

    /// Get the exit code for this result.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::cli::{CommandResult, ExitCode};
    ///
    /// let result = CommandResult::success(42);
    /// assert_eq!(result.exit_code(), ExitCode::Success);
    /// ```
    pub fn exit_code(&self) -> ExitCode {
        self.exit_code
    }

    /// Check if this result indicates success.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::cli::{CommandResult, ExitCode};
    ///
    /// let success = CommandResult::success(42);
    /// assert!(success.is_success());
    ///
    /// let failure: CommandResult<()> = CommandResult::failure(
    ///     ExitCode::GeneralError,
    ///     "Error"
    /// );
    /// assert!(!failure.is_success());
    /// ```
    pub fn is_success(&self) -> bool {
        self.exit_code.is_success()
    }

    /// Check if this result indicates failure.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::cli::{CommandResult, ExitCode};
    ///
    /// let success = CommandResult::success(42);
    /// assert!(!success.is_failure());
    ///
    /// let failure: CommandResult<()> = CommandResult::failure(
    ///     ExitCode::GeneralError,
    ///     "Error"
    /// );
    /// assert!(failure.is_failure());
    /// ```
    pub fn is_failure(&self) -> bool {
        self.exit_code.is_failure()
    }

    /// Consume this result and return the value if present.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::cli::{CommandResult, ExitCode};
    ///
    /// let result = CommandResult::success(42);
    /// assert_eq!(result.value(), Some(42));
    ///
    /// let failure: CommandResult<i32> = CommandResult::failure(
    ///     ExitCode::GeneralError,
    ///     "Error"
    /// );
    /// assert_eq!(failure.value(), None);
    /// ```
    pub fn value(self) -> Option<T> {
        self.value
    }

    /// Get a reference to the value if present.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::cli::CommandResult;
    ///
    /// let result = CommandResult::success(42);
    /// assert_eq!(result.value_ref(), Some(&42));
    /// ```
    pub fn value_ref(&self) -> Option<&T> {
        self.value.as_ref()
    }

    /// Get the messages associated with this result.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::cli::{CommandResult, ExitCode};
    ///
    /// let result: CommandResult<()> = CommandResult::failure(
    ///     ExitCode::ValidationFailed,
    ///     "Invalid config"
    /// );
    /// assert_eq!(result.messages(), &["Invalid config"]);
    /// ```
    pub fn messages(&self) -> &[String] {
        &self.messages
    }

    /// Map the value of a successful result.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::cli::CommandResult;
    ///
    /// let result = CommandResult::success(42);
    /// let doubled = result.map(|x| x * 2);
    /// assert_eq!(doubled.value(), Some(84));
    /// ```
    pub fn map<U, F>(self, f: F) -> CommandResult<U>
    where
        F: FnOnce(T) -> U,
    {
        CommandResult {
            value: self.value.map(f),
            exit_code: self.exit_code,
            messages: self.messages,
        }
    }

    /// Convert to a standard Result.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::cli::{CommandResult, ExitCode};
    ///
    /// let result = CommandResult::success(42);
    /// assert!(result.into_result().is_ok());
    ///
    /// let failure: CommandResult<()> = CommandResult::failure(
    ///     ExitCode::GeneralError,
    ///     "Error occurred"
    /// );
    /// assert!(failure.into_result().is_err());
    /// ```
    pub fn into_result(self) -> Result<T, CommandError> {
        match self.value {
            Some(value) => Ok(value),
            None => Err(CommandError {
                exit_code: self.exit_code,
                message: self.messages.join("; "),
            }),
        }
    }
}

impl<T> CommandResult<T>
where
    T: Default,
{
    /// Create a result with a default value and exit code.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::cli::{CommandResult, ExitCode};
    ///
    /// let result: CommandResult<i32> = CommandResult::with_code(ExitCode::Success);
    /// assert_eq!(result.value(), Some(0));
    /// ```
    pub fn with_code(code: ExitCode) -> Self {
        Self {
            value: Some(T::default()),
            exit_code: code,
            messages: Vec::new(),
        }
    }
}

/// Error type for converting CommandResult to Result.
#[derive(Debug, Clone)]
pub struct CommandError {
    pub exit_code: ExitCode,
    pub message: String,
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CommandError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_result() {
        let result = CommandResult::success(42);
        assert!(result.is_success());
        assert!(!result.is_failure());
        assert_eq!(result.exit_code(), ExitCode::Success);
        assert!(result.messages().is_empty());
        assert_eq!(result.value(), Some(42));
    }

    #[test]
    fn success_with_message() {
        let result = CommandResult::success_with_message(42, "All tests passed");
        assert!(result.is_success());
        assert_eq!(result.messages(), &["All tests passed"]);
    }

    #[test]
    fn failure_result() {
        let result: CommandResult<()> =
            CommandResult::failure(ExitCode::ValidationFailed, "Invalid configuration");
        assert!(!result.is_success());
        assert!(result.is_failure());
        assert_eq!(result.exit_code(), ExitCode::ValidationFailed);
        assert_eq!(result.messages(), &["Invalid configuration"]);
        assert_eq!(result.value(), None);
    }

    #[test]
    fn chained_messages() {
        let result = CommandResult::success(())
            .with_message("Step 1 completed")
            .with_message("Step 2 completed")
            .with_message("Step 3 completed");
        assert_eq!(
            result.messages(),
            &["Step 1 completed", "Step 2 completed", "Step 3 completed"]
        );
    }

    #[test]
    fn map_value() {
        let result = CommandResult::success(21);
        let doubled = result.map(|x| x * 2);
        assert!(doubled.is_success());
        assert_eq!(doubled.value(), Some(42));
    }

    #[test]
    fn map_preserves_messages() {
        let result = CommandResult::success(21).with_message("Original message");
        let doubled = result.map(|x| x * 2);
        assert_eq!(doubled.messages(), &["Original message"]);
    }

    #[test]
    fn value_ref() {
        let result = CommandResult::success(42);
        assert_eq!(result.value_ref(), Some(&42));
        // Can still use result after value_ref
        assert_eq!(result.value(), Some(42));
    }

    #[test]
    fn into_result_success() {
        let cmd_result = CommandResult::success(42);
        let result = cmd_result.into_result();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn into_result_failure() {
        let cmd_result: CommandResult<()> =
            CommandResult::failure(ExitCode::ValidationFailed, "Error message");
        let result = cmd_result.into_result();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.exit_code, ExitCode::ValidationFailed);
        assert_eq!(err.message, "Error message");
    }

    #[test]
    fn with_code_default() {
        let result: CommandResult<i32> = CommandResult::with_code(ExitCode::Success);
        assert_eq!(result.exit_code(), ExitCode::Success);
        assert_eq!(result.value(), Some(0));
    }

    #[test]
    fn command_error_display() {
        let error = CommandError {
            exit_code: ExitCode::GeneralError,
            message: "Test error".to_string(),
        };
        assert_eq!(error.to_string(), "Test error");
    }
}
