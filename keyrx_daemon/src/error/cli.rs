//! CLI command error types.

use thiserror::Error;

/// CLI command errors.
///
/// This error type covers failures when executing CLI commands.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CliError {
    /// Invalid command-line arguments.
    #[error("Invalid arguments: {reason}")]
    InvalidArguments {
        /// Reason why arguments are invalid.
        reason: String,
    },

    /// Command execution failed.
    #[error("Command '{command}' failed: {reason}")]
    CommandFailed {
        /// Name of the command that failed.
        command: String,
        /// Reason for command failure.
        reason: String,
    },

    /// The command already told the user what went wrong (text on stderr or
    /// a `--json` error object); the caller only needs the non-zero exit
    /// code and must not print a second, generic message.
    #[error("command failed (already reported)")]
    Reported,

    /// Output formatting error.
    #[error("Failed to format output: {reason}")]
    OutputError {
        /// Reason for output formatting failure.
        reason: String,
    },
}

impl From<serde_json::Error> for CliError {
    fn from(err: serde_json::Error) -> Self {
        CliError::OutputError {
            reason: err.to_string(),
        }
    }
}
