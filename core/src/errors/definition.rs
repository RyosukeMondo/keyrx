//! Error definitions with message templates and metadata.
//!
//! ErrorDef stores the static definition of an error including its code,
//! message template, hint, severity, and optional documentation link.
//! These definitions are used by the registry and error creation macros.

use super::code::ErrorCode;
use std::collections::HashMap;

/// Severity level for errors.
///
/// Severity helps categorize the impact of an error:
/// - Fatal: Application cannot continue
/// - Error: Operation failed but app can continue
/// - Warning: Something wrong but operation succeeded
/// - Info: Informational message (rare for errors)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// Fatal error - application cannot continue
    Fatal,
    /// Error - operation failed but application can continue
    Error,
    /// Warning - something concerning but operation succeeded
    Warning,
    /// Info - informational message
    Info,
}

/// Static error definition with template and metadata.
///
/// ErrorDef represents the compile-time definition of an error.
/// It contains the error code, message template with placeholders,
/// optional hint for users, severity level, and documentation link.
///
/// Message templates use {key} syntax for placeholder substitution.
///
/// # Example
///
/// ```ignore
/// use keyrx_core::errors::{ErrorCode, ErrorCategory, ErrorDef, ErrorSeverity};
///
/// const CONFIG_NOT_FOUND: ErrorDef = ErrorDef {
///     code: ErrorCode::new(ErrorCategory::Config, 1001),
///     message_template: "Configuration file not found: {path}",
///     hint: Some("Check that the file exists and has read permissions"),
///     severity: ErrorSeverity::Error,
///     doc_link: Some("https://docs.keyrx.com/errors#KRX-C1001"),
/// };
///
/// let msg = CONFIG_NOT_FOUND.format(&[("path", "/etc/keyrx.toml")]);
/// assert_eq!(msg, "Configuration file not found: /etc/keyrx.toml");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorDef {
    /// Unique error code
    pub code: ErrorCode,
    /// Message template with {key} placeholders
    pub message_template: &'static str,
    /// Optional hint for resolving the error
    pub hint: Option<&'static str>,
    /// Severity level
    pub severity: ErrorSeverity,
    /// Optional documentation link
    pub doc_link: Option<&'static str>,
}

impl ErrorDef {
    /// Create a new error definition.
    pub const fn new(
        code: ErrorCode,
        message_template: &'static str,
        hint: Option<&'static str>,
        severity: ErrorSeverity,
        doc_link: Option<&'static str>,
    ) -> Self {
        Self {
            code,
            message_template,
            hint,
            severity,
            doc_link,
        }
    }

    /// Format the message template with provided arguments.
    ///
    /// Replaces {key} placeholders in the template with values from args.
    /// If a placeholder is not found in args, it is left as-is.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let msg = def.format(&[("file", "config.toml"), ("line", "42")]);
    /// ```
    pub fn format(&self, args: &[(&str, &str)]) -> String {
        let mut message = self.message_template.to_string();
        for (key, value) in args {
            let placeholder = format!("{{{}}}", key);
            message = message.replace(&placeholder, value);
        }
        message
    }

    /// Format the message template with a HashMap of arguments.
    ///
    /// This is a convenience method for when args are already in a HashMap.
    pub fn format_map(&self, args: &HashMap<String, String>) -> String {
        let mut message = self.message_template.to_string();
        for (key, value) in args {
            let placeholder = format!("{{{}}}", key);
            message = message.replace(&placeholder, value);
        }
        message
    }

    /// Get the error code.
    pub const fn code(&self) -> ErrorCode {
        self.code
    }

    /// Get the message template.
    pub const fn message_template(&self) -> &'static str {
        self.message_template
    }

    /// Get the hint, if any.
    pub const fn hint(&self) -> Option<&'static str> {
        self.hint
    }

    /// Get the severity level.
    pub const fn severity(&self) -> ErrorSeverity {
        self.severity
    }

    /// Get the documentation link, if any.
    pub const fn doc_link(&self) -> Option<&'static str> {
        self.doc_link
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::code::ErrorCategory;

    #[test]
    fn error_def_format() {
        let def = ErrorDef {
            code: ErrorCode::new(ErrorCategory::Config, 1001),
            message_template: "File {file} not found at line {line}",
            hint: Some("Check the path"),
            severity: ErrorSeverity::Error,
            doc_link: None,
        };

        let msg = def.format(&[("file", "config.toml"), ("line", "42")]);
        assert_eq!(msg, "File config.toml not found at line 42");
    }

    #[test]
    fn error_def_format_missing_arg() {
        let def = ErrorDef {
            code: ErrorCode::new(ErrorCategory::Config, 1001),
            message_template: "File {file} not found",
            hint: None,
            severity: ErrorSeverity::Error,
            doc_link: None,
        };

        // Missing arg leaves placeholder as-is
        let msg = def.format(&[]);
        assert_eq!(msg, "File {file} not found");
    }

    #[test]
    fn error_def_format_map() {
        let def = ErrorDef {
            code: ErrorCode::new(ErrorCategory::Runtime, 2001),
            message_template: "Process {name} failed with code {code}",
            hint: Some("Check logs"),
            severity: ErrorSeverity::Error,
            doc_link: None,
        };

        let mut args = HashMap::new();
        args.insert("name".to_string(), "worker".to_string());
        args.insert("code".to_string(), "137".to_string());

        let msg = def.format_map(&args);
        assert_eq!(msg, "Process worker failed with code 137");
    }

    #[test]
    fn error_def_accessors() {
        let code = ErrorCode::new(ErrorCategory::Driver, 3001);
        let def = ErrorDef {
            code,
            message_template: "Driver error",
            hint: Some("Reinstall driver"),
            severity: ErrorSeverity::Fatal,
            doc_link: Some("https://example.com/docs"),
        };

        assert_eq!(def.code(), code);
        assert_eq!(def.message_template(), "Driver error");
        assert_eq!(def.hint(), Some("Reinstall driver"));
        assert_eq!(def.severity(), ErrorSeverity::Fatal);
        assert_eq!(def.doc_link(), Some("https://example.com/docs"));
    }

    #[test]
    fn error_severity_levels() {
        let fatal = ErrorSeverity::Fatal;
        let error = ErrorSeverity::Error;
        let warning = ErrorSeverity::Warning;
        let info = ErrorSeverity::Info;

        assert_eq!(fatal, ErrorSeverity::Fatal);
        assert_ne!(fatal, error);
        assert_ne!(error, warning);
        assert_ne!(warning, info);
    }
}
