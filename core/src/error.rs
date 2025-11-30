//! Custom error types for KeyRx.
//!
//! This module provides structured error handling with actionable error messages.
//! All errors implement `std::error::Error` via `thiserror` and can be converted
//! from common error sources like `std::io::Error` and Rhai's `EvalAltResult`.

use thiserror::Error;

/// The main error type for KeyRx operations.
///
/// Each variant provides specific context about what went wrong and suggests
/// corrective action where possible.
#[derive(Debug, Error)]
pub enum KeyRxError {
    /// A key name was not recognized.
    ///
    /// This occurs when a script uses an invalid key name in `remap()`, `block()`,
    /// or `pass()`. Check `docs/KEYS.md` for valid key names.
    #[error("Unknown key '{key}'. See docs/KEYS.md for valid key names.")]
    UnknownKey { key: String },

    /// A script failed to compile.
    ///
    /// This indicates a syntax error or other compilation issue. The message
    /// contains details from the Rhai compiler, and line/column information
    /// when available.
    #[error("Script compilation failed: {message}{}", format_position(*.line, *.column))]
    ScriptCompileError {
        message: String,
        line: Option<usize>,
        column: Option<usize>,
    },

    /// A script failed during execution.
    ///
    /// This occurs when a script encounters a runtime error, such as calling
    /// an undefined function or a type mismatch.
    #[error("Script runtime error: {message}")]
    ScriptRuntimeError { message: String },

    /// A file path was invalid.
    ///
    /// This can occur with non-UTF8 paths or paths that don't meet other
    /// requirements (e.g., must be a file, not a directory).
    #[error("Invalid path '{path}': {reason}")]
    InvalidPath { path: String, reason: String },

    /// An I/O operation failed.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// A platform-specific operation failed.
    ///
    /// This covers OS-level errors from input drivers or other platform APIs.
    #[error("Platform error: {message}")]
    PlatformError { message: String },
}

impl KeyRxError {
    /// Create an UnknownKey error.
    pub fn unknown_key(key: impl Into<String>) -> Self {
        Self::UnknownKey { key: key.into() }
    }

    /// Create a ScriptCompileError with position information.
    pub fn script_compile(
        message: impl Into<String>,
        line: Option<usize>,
        column: Option<usize>,
    ) -> Self {
        Self::ScriptCompileError {
            message: message.into(),
            line,
            column,
        }
    }

    /// Create a ScriptCompileError without position information.
    pub fn script_compile_simple(message: impl Into<String>) -> Self {
        Self::ScriptCompileError {
            message: message.into(),
            line: None,
            column: None,
        }
    }

    /// Create a ScriptRuntimeError.
    pub fn script_runtime(message: impl Into<String>) -> Self {
        Self::ScriptRuntimeError {
            message: message.into(),
        }
    }

    /// Create an InvalidPath error.
    pub fn invalid_path(path: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidPath {
            path: path.into(),
            reason: reason.into(),
        }
    }

    /// Create a PlatformError.
    pub fn platform(message: impl Into<String>) -> Self {
        Self::PlatformError {
            message: message.into(),
        }
    }
}

impl From<Box<rhai::EvalAltResult>> for KeyRxError {
    fn from(err: Box<rhai::EvalAltResult>) -> Self {
        let position = err.position();
        if position.is_none() {
            Self::ScriptRuntimeError {
                message: err.to_string(),
            }
        } else {
            Self::ScriptRuntimeError {
                message: format!("{} at {}", err, position),
            }
        }
    }
}

/// Helper to format optional position information.
fn format_position(line: Option<usize>, column: Option<usize>) -> String {
    match (line, column) {
        (Some(l), Some(c)) => format!(" at line {}, column {}", l, c),
        (Some(l), None) => format!(" at line {}", l),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_key_error_message() {
        let err = KeyRxError::unknown_key("BadKey");
        assert!(err.to_string().contains("BadKey"));
        assert!(err.to_string().contains("KEYS.md"));
    }

    #[test]
    fn script_compile_error_with_position() {
        let err = KeyRxError::script_compile("syntax error", Some(10), Some(5));
        let msg = err.to_string();
        assert!(msg.contains("syntax error"));
        assert!(msg.contains("line 10"));
        assert!(msg.contains("column 5"));
    }

    #[test]
    fn script_compile_error_without_position() {
        let err = KeyRxError::script_compile_simple("unknown token");
        let msg = err.to_string();
        assert!(msg.contains("unknown token"));
        assert!(!msg.contains("line"));
    }

    #[test]
    fn io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: KeyRxError = io_err.into();
        assert!(err.to_string().contains("file not found"));
    }

    #[test]
    fn invalid_path_error() {
        let err = KeyRxError::invalid_path("/some/path", "not UTF-8");
        assert!(err.to_string().contains("/some/path"));
        assert!(err.to_string().contains("not UTF-8"));
    }

    #[test]
    fn platform_error() {
        let err = KeyRxError::platform("device not found");
        assert!(err.to_string().contains("device not found"));
    }
}
