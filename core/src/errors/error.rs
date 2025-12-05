//! Runtime error type for KeyRx with context chaining and JSON serialization.
//!
//! KeyrxError is the main error type used throughout KeyRx. It wraps an
//! ErrorDef with runtime context and supports error chaining via source.
//! Errors can be serialized to JSON for logging and FFI boundaries.

use super::definition::ErrorDef;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error as StdError;
use std::fmt;

/// Runtime error with code, message, context, and source chain.
///
/// KeyrxError combines a static ErrorDef with runtime context and
/// optional source error. It implements std::error::Error for compatibility
/// with the Rust error ecosystem and can be serialized to JSON.
///
/// # Example
///
/// ```ignore
/// use keyrx_core::errors::{KeyrxError, ErrorCode, ErrorCategory};
///
/// let err = KeyrxError::new(
///     CONFIG_NOT_FOUND,
///     vec![("path".to_string(), "/etc/keyrx.toml".to_string())],
///     None,
/// );
///
/// println!("{}", err); // Prints formatted error with code
/// let json = serde_json::to_string(&err)?; // Serialize for logging
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyrxError {
    /// The error definition (code, template, hint, severity)
    #[serde(skip)]
    definition: Option<&'static ErrorDef>,

    /// Error code (stored separately for serialization)
    code: String,

    /// Formatted error message
    message: String,

    /// Runtime context as key-value pairs
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    context: HashMap<String, String>,

    /// Optional hint for resolving the error
    #[serde(skip_serializing_if = "Option::is_none")]
    hint: Option<String>,

    /// Severity level
    severity: String,

    /// Optional source error (as string for serialization)
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,
}

impl KeyrxError {
    /// Create a new KeyrxError from an ErrorDef with context.
    ///
    /// # Arguments
    ///
    /// * `def` - The error definition
    /// * `context` - Key-value pairs for message formatting and context
    /// * `source` - Optional source error
    ///
    /// # Example
    ///
    /// ```ignore
    /// let err = KeyrxError::new(
    ///     &CONFIG_PARSE_ERROR,
    ///     vec![("file".to_string(), "config.toml".to_string())],
    ///     Some(Box::new(parse_err)),
    /// );
    /// ```
    pub fn new(
        def: &'static ErrorDef,
        context: Vec<(String, String)>,
        source: Option<Box<dyn StdError + Send + Sync>>,
    ) -> Self {
        let context_map: HashMap<String, String> = context.into_iter().collect();

        // Format message with context
        let message = def.format_map(&context_map);

        // Convert source to string for serialization
        let source_str = source.map(|e| e.to_string());

        Self {
            definition: Some(def),
            code: def.code().to_string(),
            message,
            context: context_map,
            hint: def.hint().map(|h| h.to_string()),
            severity: format!("{:?}", def.severity()),
            source: source_str,
        }
    }

    /// Create a KeyrxError with a simple message and no context.
    ///
    /// This is useful for quick error creation when you don't need
    /// template substitution or additional context.
    pub fn simple(def: &'static ErrorDef) -> Self {
        Self::new(def, Vec::new(), None)
    }

    /// Add additional context to an existing error.
    ///
    /// This allows adding context as the error propagates up the call stack.
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    /// Chain a source error onto this error.
    pub fn with_source(mut self, source: Box<dyn StdError + Send + Sync>) -> Self {
        self.source = Some(source.to_string());
        self
    }

    /// Get the error code.
    pub fn code(&self) -> &str {
        &self.code
    }

    /// Get the formatted message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Get the context as a map.
    pub fn context(&self) -> &HashMap<String, String> {
        &self.context
    }

    /// Get the hint, if any.
    pub fn hint(&self) -> Option<&str> {
        self.hint.as_deref()
    }

    /// Get the severity level.
    pub fn severity(&self) -> &str {
        &self.severity
    }

    /// Get the source error string, if any.
    pub fn source_str(&self) -> Option<&str> {
        self.source.as_deref()
    }

    /// Get the error definition, if available.
    pub fn definition(&self) -> Option<&'static ErrorDef> {
        self.definition
    }
}

impl fmt::Display for KeyrxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)?;

        if let Some(hint) = &self.hint {
            write!(f, "\nHint: {}", hint)?;
        }

        if let Some(source) = &self.source {
            write!(f, "\nCaused by: {}", source)?;
        }

        Ok(())
    }
}

impl StdError for KeyrxError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        // Since we store source as String for serialization,
        // we can't return a reference to the original error.
        // This is a trade-off for JSON serialization support.
        None
    }
}

/// Convert from std::io::Error to KeyrxError.
///
/// This is a convenience for common I/O errors. Since we don't have
/// a specific ErrorDef, we create a generic internal error.
impl From<std::io::Error> for KeyrxError {
    fn from(err: std::io::Error) -> Self {
        // We'll need to define a generic I/O error in the registry later
        let context = vec![("error".to_string(), err.to_string())];

        // For now, create a minimal error without an ErrorDef
        // This will be improved once we have the full registry
        Self {
            definition: None,
            code: "KRX-I9001".to_string(),
            message: format!("I/O error: {}", err),
            context: context.into_iter().collect(),
            hint: Some("Check file permissions and paths".to_string()),
            severity: "Error".to_string(),
            source: None,
        }
    }
}

/// Convert from DriverError to KeyrxError.
///
/// Maps driver-specific errors to appropriate KeyrxError categories.
impl From<crate::drivers::common::error::DriverError> for KeyrxError {
    fn from(err: crate::drivers::common::error::DriverError) -> Self {
        use crate::errors::driver::*;
        use crate::keyrx_err;

        // Map driver errors to appropriate KeyrxError codes
        match err {
            crate::drivers::common::error::DriverError::DeviceNotFound { path } => {
                keyrx_err!(DRIVER_DEVICE_NOT_FOUND, device = path.display().to_string())
            }
            crate::drivers::common::error::DriverError::PermissionDenied { resource, .. } => {
                keyrx_err!(DRIVER_PERMISSION_DENIED, device = resource)
            }
            crate::drivers::common::error::DriverError::DeviceDisconnected { device } => {
                keyrx_err!(DRIVER_DEVICE_DISCONNECTED, device = device)
            }
            crate::drivers::common::error::DriverError::HookFailed { code } => {
                keyrx_err!(
                    WINDOWS_HOOK_FAILED,
                    reason = format!("error code 0x{:x}", code)
                )
            }
            crate::drivers::common::error::DriverError::GrabFailed { reason } => {
                keyrx_err!(EVDEV_DEVICE_GRAB_FAILED, device = reason)
            }
            crate::drivers::common::error::DriverError::VirtualDeviceError { message } => {
                keyrx_err!(EVDEV_UINPUT_CREATE_FAILED, reason = message)
            }
            crate::drivers::common::error::DriverError::InjectionFailed { reason } => {
                use crate::errors::runtime::OUTPUT_INJECTION_FAILED;
                keyrx_err!(OUTPUT_INJECTION_FAILED, reason = reason)
            }
            crate::drivers::common::error::DriverError::InvalidEvent { details } => {
                use crate::errors::runtime::INVALID_EVENT_DATA;
                keyrx_err!(INVALID_EVENT_DATA, reason = details)
            }
            _ => {
                // For other errors, create a generic driver error
                Self {
                    definition: None,
                    code: "KRX-D3999".to_string(),
                    message: format!("Driver error: {}", err),
                    context: Default::default(),
                    hint: Some(err.suggested_action().to_string()),
                    severity: "Error".to_string(),
                    source: None,
                }
            }
        }
    }
}

/// Convert from anyhow::Error to KeyrxError.
///
/// This is a convenience for errors coming from dependencies or internal
/// helpers that use anyhow.
impl From<anyhow::Error> for KeyrxError {
    fn from(err: anyhow::Error) -> Self {
        let context = vec![("error".to_string(), err.to_string())];

        Self {
            definition: None,
            code: "KRX-I9002".to_string(),
            message: format!("Internal error: {}", err),
            context: context.into_iter().collect(),
            hint: Some("This indicates an internal logic error".to_string()),
            severity: "Error".to_string(),
            source: None,
        }
    }
}

#[cfg(windows)]
impl From<crate::error::WindowsDriverError> for KeyrxError {
    fn from(err: crate::error::WindowsDriverError) -> Self {
        Self {
            definition: None,
            code: "KRX-D3998".to_string(),
            message: format!("Windows driver error: {}", err),
            context: Default::default(),
            hint: None,
            severity: "Error".to_string(),
            source: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::code::{ErrorCategory, ErrorCode};
    use crate::errors::definition::ErrorSeverity;

    // Test error definition
    const TEST_ERROR: ErrorDef = ErrorDef {
        code: ErrorCode::new(ErrorCategory::Config, 1001),
        message_template: "File {file} not found",
        hint: Some("Check that the file exists"),
        severity: ErrorSeverity::Error,
        doc_link: None,
    };

    #[test]
    fn keyrx_error_new() {
        let err = KeyrxError::new(
            &TEST_ERROR,
            vec![("file".to_string(), "config.toml".to_string())],
            None,
        );

        assert_eq!(err.code(), "KRX-C1001");
        assert_eq!(err.message(), "File config.toml not found");
        assert_eq!(err.hint(), Some("Check that the file exists"));
        assert_eq!(err.severity(), "Error");
    }

    #[test]
    fn keyrx_error_simple() {
        let err = KeyrxError::simple(&TEST_ERROR);

        assert_eq!(err.code(), "KRX-C1001");
        assert_eq!(err.message(), "File {file} not found"); // No substitution
    }

    #[test]
    fn keyrx_error_with_context() {
        let err = KeyrxError::simple(&TEST_ERROR)
            .with_context("user", "alice")
            .with_context("timestamp", "2024-01-01");

        assert_eq!(err.context().get("user").map(|s| s.as_str()), Some("alice"));
        assert_eq!(
            err.context().get("timestamp").map(|s| s.as_str()),
            Some("2024-01-01")
        );
    }

    #[test]
    fn keyrx_error_display() {
        let err = KeyrxError::new(
            &TEST_ERROR,
            vec![("file".to_string(), "config.toml".to_string())],
            None,
        );

        let display = format!("{}", err);
        assert!(display.contains("KRX-C1001"));
        assert!(display.contains("File config.toml not found"));
        assert!(display.contains("Hint: Check that the file exists"));
    }

    #[test]
    fn keyrx_error_json_serialization() {
        let err = KeyrxError::new(
            &TEST_ERROR,
            vec![("file".to_string(), "config.toml".to_string())],
            None,
        );

        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("KRX-C1001"));
        assert!(json.contains("File config.toml not found"));

        // Deserialize back
        let deserialized: KeyrxError = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.code(), err.code());
        assert_eq!(deserialized.message(), err.message());
    }

    #[test]
    fn keyrx_error_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = KeyrxError::from(io_err);

        assert_eq!(err.code(), "KRX-I9001");
        assert!(err.message().contains("I/O error"));
    }

    #[test]
    fn keyrx_error_implements_std_error() {
        let err = KeyrxError::simple(&TEST_ERROR);

        // Should compile - KeyrxError implements Error
        fn takes_error(_e: &dyn StdError) {}
        takes_error(&err);
    }
}
