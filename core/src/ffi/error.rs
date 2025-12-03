//! FFI error types and result handling.
//!
//! Provides standardized error handling across all FFI exports with JSON serialization.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Standard error type for FFI operations.
///
/// Serializes to JSON format: `{code, message, details?}`
/// Used in FFI result serialization as `error:{...}` format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FfiError {
    /// Error code for programmatic handling (e.g., "INVALID_UTF8", "DEVICE_NOT_FOUND")
    pub code: String,

    /// Human-readable error message
    pub message: String,

    /// Optional additional context as JSON value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl FfiError {
    /// Create a new FFI error with code and message.
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
        }
    }

    /// Create an FFI error with additional details.
    pub fn with_details(
        code: impl Into<String>,
        message: impl Into<String>,
        details: serde_json::Value,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: Some(details),
        }
    }

    /// Create an invalid input error.
    ///
    /// # Example
    /// ```
    /// # use keyrx_core::ffi::error::FfiError;
    /// let err = FfiError::invalid_input("device_id must not be empty");
    /// assert_eq!(err.code, "INVALID_INPUT");
    /// ```
    pub fn invalid_input(msg: impl Into<String>) -> Self {
        Self::new("INVALID_INPUT", msg)
    }

    /// Create an internal error (typically for panics or unexpected failures).
    ///
    /// # Example
    /// ```
    /// # use keyrx_core::ffi::error::FfiError;
    /// let err = FfiError::internal("unexpected panic in discovery");
    /// assert_eq!(err.code, "INTERNAL_ERROR");
    /// ```
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::new("INTERNAL_ERROR", msg)
    }

    /// Create a not found error.
    ///
    /// # Example
    /// ```
    /// # use keyrx_core::ffi::error::FfiError;
    /// let err = FfiError::not_found("device");
    /// assert_eq!(err.code, "NOT_FOUND");
    /// assert_eq!(err.message, "device not found");
    /// ```
    pub fn not_found(resource: impl Into<String>) -> Self {
        let resource = resource.into();
        Self::new("NOT_FOUND", format!("{resource} not found"))
    }

    /// Create a null pointer error.
    pub fn null_pointer(param: impl Into<String>) -> Self {
        let param = param.into();
        Self::new(
            "NULL_POINTER",
            format!("null pointer for parameter: {param}"),
        )
    }

    /// Create an invalid UTF-8 error.
    pub fn invalid_utf8(param: impl Into<String>) -> Self {
        let param = param.into();
        Self::new(
            "INVALID_UTF8",
            format!("invalid UTF-8 in parameter: {param}"),
        )
    }
}

impl fmt::Display for FfiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for FfiError {}

/// Result type for FFI operations.
///
/// Serializes to one of two formats:
/// - Success: `"ok:{...serialized T...}"`
/// - Error: `"error:{code, message, details?}"`
pub type FfiResult<T> = Result<T, FfiError>;

/// Serialize an FfiResult to the standard FFI JSON format.
///
/// # Format
/// - Success: `"ok:{...}"` where `{...}` is the serialized success value
/// - Error: `"error:{code, message, details?}"` where the error fields are serialized
///
/// # Example
/// ```
/// # use keyrx_core::ffi::error::{FfiError, serialize_ffi_result};
/// # use serde::Serialize;
/// #[derive(Serialize)]
/// struct StartResult { total_keys: usize }
///
/// let result: Result<StartResult, FfiError> = Ok(StartResult { total_keys: 42 });
/// let json = serialize_ffi_result(&result).unwrap();
/// assert!(json.starts_with("ok:"));
/// ```
pub fn serialize_ffi_result<T: Serialize>(result: &FfiResult<T>) -> serde_json::Result<String> {
    match result {
        Ok(value) => {
            let json = serde_json::to_string(value)?;
            Ok(format!("ok:{json}"))
        }
        Err(error) => {
            let json = serde_json::to_string(error)?;
            Ok(format!("error:{json}"))
        }
    }
}

/// Helper trait for serializing FfiResult into the standard format.
pub trait SerializeFfiResult {
    /// Serialize to standard FFI format: `"ok:{...}"` or `"error:{...}"`.
    fn to_ffi_json(&self) -> serde_json::Result<String>;
}

impl<T: Serialize> SerializeFfiResult for FfiResult<T> {
    fn to_ffi_json(&self) -> serde_json::Result<String> {
        serialize_ffi_result(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serde_json::json;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestData {
        value: i32,
    }

    #[test]
    fn test_ffi_error_constructors() {
        let err = FfiError::invalid_input("test message");
        assert_eq!(err.code, "INVALID_INPUT");
        assert_eq!(err.message, "test message");
        assert!(err.details.is_none());

        let err = FfiError::internal("panic caught");
        assert_eq!(err.code, "INTERNAL_ERROR");
        assert_eq!(err.message, "panic caught");

        let err = FfiError::not_found("device");
        assert_eq!(err.code, "NOT_FOUND");
        assert_eq!(err.message, "device not found");

        let err = FfiError::null_pointer("device_id");
        assert_eq!(err.code, "NULL_POINTER");
        assert_eq!(err.message, "null pointer for parameter: device_id");

        let err = FfiError::invalid_utf8("name");
        assert_eq!(err.code, "INVALID_UTF8");
        assert_eq!(err.message, "invalid UTF-8 in parameter: name");
    }

    #[test]
    fn test_ffi_error_with_details() {
        let details = json!({"attempted_value": "invalid"});
        let err = FfiError::with_details("PARSE_ERROR", "failed to parse", details.clone());
        assert_eq!(err.code, "PARSE_ERROR");
        assert_eq!(err.message, "failed to parse");
        assert_eq!(err.details, Some(details));
    }

    #[test]
    fn test_ffi_error_serialization() {
        let err = FfiError::invalid_input("test");
        let json = serde_json::to_string(&err).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["code"], "INVALID_INPUT");
        assert_eq!(parsed["message"], "test");
        assert!(parsed.get("details").is_none());
    }

    #[test]
    fn test_serialize_ffi_result_success() {
        let result: FfiResult<TestData> = Ok(TestData { value: 42 });
        let json = serialize_ffi_result(&result).unwrap();

        assert!(json.starts_with("ok:"));
        let payload = &json[3..];
        let parsed: TestData = serde_json::from_str(payload).unwrap();
        assert_eq!(parsed.value, 42);
    }

    #[test]
    fn test_serialize_ffi_result_error() {
        let result: FfiResult<TestData> = Err(FfiError::not_found("resource"));
        let json = serialize_ffi_result(&result).unwrap();

        assert!(json.starts_with("error:"));
        let payload = &json[6..];
        let parsed: FfiError = serde_json::from_str(payload).unwrap();
        assert_eq!(parsed.code, "NOT_FOUND");
        assert_eq!(parsed.message, "resource not found");
    }

    #[test]
    fn test_serialize_trait() {
        let result: FfiResult<TestData> = Ok(TestData { value: 99 });
        let json = result.to_ffi_json().unwrap();
        assert!(json.starts_with("ok:"));
    }

    #[test]
    fn test_error_display() {
        let err = FfiError::invalid_input("test message");
        let display = format!("{}", err);
        assert_eq!(display, "[INVALID_INPUT] test message");
    }

    #[test]
    fn test_error_with_details_serialization() {
        let details = json!({"key": "value", "count": 5});
        let err = FfiError::with_details("CUSTOM", "custom error", details);
        let json = serde_json::to_string(&err).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["code"], "CUSTOM");
        assert_eq!(parsed["message"], "custom error");
        assert_eq!(parsed["details"]["key"], "value");
        assert_eq!(parsed["details"]["count"], 5);
    }
}
