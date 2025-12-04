//! FFI error types for marshaling layer.
//!
//! This module provides comprehensive error handling for the FFI marshaling system.
//! Unlike the basic [`crate::ffi::error::FfiError`], these types are specifically
//! designed for marshaling operations and include additional context fields.
//!
//! # Architecture
//!
//! The error system consists of:
//!
//! - [`MarshalError`]: Rich Rust error type with code, message, hint, and context
//! - [`MarshalErrorC`]: C-compatible representation for crossing FFI boundaries
//! - Conversion functions for safe marshaling
//!
//! # Design Rationale
//!
//! While `crate::ffi::error::FfiError` provides basic error handling, the marshal
//! layer needs:
//!
//! - **Hint Field**: User-facing suggestions for resolution
//! - **Context Field**: Machine-readable error context (file paths, indices, etc.)
//! - **Code Preservation**: Stable error codes for client-side handling
//! - **Safe Memory**: Proper C string allocation and deallocation
//!
//! # Example
//!
//! ```
//! use keyrx_core::ffi::marshal::error::MarshalError;
//!
//! // Create error with hint and context
//! let error = MarshalError::new("BUFFER_OVERFLOW", "String too long for buffer")
//!     .with_hint("Maximum length is 255 bytes")
//!     .with_context("field", "device_name");
//!
//! assert_eq!(error.code(), "BUFFER_OVERFLOW");
//! assert_eq!(error.hint(), Some("Maximum length is 255 bytes"));
//! ```

use std::collections::HashMap;
use std::ffi::{c_char, CString};
use std::fmt;
use std::ptr;

/// Comprehensive error type for FFI marshaling operations.
///
/// This error type extends basic FFI errors with:
/// - **hint**: User-facing resolution suggestions
/// - **context**: Machine-readable error context (key-value pairs)
///
/// # Design
///
/// Unlike [`crate::ffi::error::FfiError`], this type is specifically designed for
/// marshaling operations where detailed error information helps debugging complex
/// FFI boundary issues.
///
/// # Example
///
/// ```
/// use keyrx_core::ffi::marshal::error::MarshalError;
///
/// let error = MarshalError::new("UTF8_ERROR", "Invalid UTF-8 sequence")
///     .with_hint("Ensure the string contains valid UTF-8")
///     .with_context("byte_index", "42");
///
/// assert_eq!(error.message(), "Invalid UTF-8 sequence");
/// ```
#[derive(Debug, Clone)]
pub struct MarshalError {
    /// Error code for programmatic handling (e.g., "BUFFER_OVERFLOW", "UTF8_ERROR").
    code: String,

    /// Human-readable error message describing what went wrong.
    message: String,

    /// Optional hint for how to resolve the error (user-facing).
    hint: Option<String>,

    /// Optional context for debugging (key-value pairs).
    /// Examples: file paths, field names, array indices, etc.
    context: HashMap<String, String>,
}

impl MarshalError {
    /// Create a new marshaling error.
    ///
    /// # Parameters
    ///
    /// * `code` - Error code for programmatic handling
    /// * `message` - Human-readable error message
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::ffi::marshal::error::MarshalError;
    ///
    /// let error = MarshalError::new("INVALID_INPUT", "Null pointer received");
    /// assert_eq!(error.code(), "INVALID_INPUT");
    /// ```
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            hint: None,
            context: HashMap::new(),
        }
    }

    /// Add a resolution hint to the error.
    ///
    /// # Parameters
    ///
    /// * `hint` - User-facing suggestion for resolving the error
    ///
    /// # Returns
    ///
    /// Self for method chaining
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::ffi::marshal::error::MarshalError;
    ///
    /// let error = MarshalError::new("SIZE_LIMIT", "Data too large")
    ///     .with_hint("Split data into chunks < 1MB");
    /// ```
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// Add a context key-value pair.
    ///
    /// # Parameters
    ///
    /// * `key` - Context key (e.g., "field", "index", "path")
    /// * `value` - Context value
    ///
    /// # Returns
    ///
    /// Self for method chaining
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::ffi::marshal::error::MarshalError;
    ///
    /// let error = MarshalError::new("PARSE_ERROR", "Failed to parse")
    ///     .with_context("field", "device_name")
    ///     .with_context("value", "invalid\0name");
    /// ```
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    /// Get the error code.
    pub fn code(&self) -> &str {
        &self.code
    }

    /// Get the error message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Get the hint, if present.
    pub fn hint(&self) -> Option<&str> {
        self.hint.as_deref()
    }

    /// Get the context map.
    pub fn context(&self) -> &HashMap<String, String> {
        &self.context
    }

    /// Convert to C-compatible representation.
    ///
    /// Allocates C strings for all fields. The caller must free these using
    /// [`MarshalErrorC::free`].
    ///
    /// # Example
    ///
    /// ```
    /// # use keyrx_core::ffi::marshal::error::MarshalError;
    /// let error = MarshalError::new("TEST", "test message");
    /// let c_error = error.to_c();
    /// // ... use c_error ...
    /// unsafe { c_error.free(); }
    /// ```
    #[allow(clippy::unwrap_used)] // Fallback strings are guaranteed valid
    pub fn to_c(&self) -> MarshalErrorC {
        // Convert code to numeric hash
        let code = hash_error_code(&self.code);

        // Convert message to C string
        let message = to_c_string(&self.message, "Unknown error");

        // Convert hint to C string (null if not present)
        let hint = self
            .hint
            .as_ref()
            .map(|h| to_c_string(h, ""))
            .unwrap_or(ptr::null_mut());

        // Serialize context to JSON string (null if empty)
        let context = if self.context.is_empty() {
            ptr::null_mut()
        } else {
            match serde_json::to_string(&self.context) {
                Ok(json) => to_c_string(&json, "{}"),
                Err(_) => to_c_string("{}", "{}"),
            }
        };

        MarshalErrorC {
            code,
            message,
            hint,
            context,
        }
    }

    /// Common error constructors for marshaling operations.
    ///
    /// Create a buffer overflow error.
    pub fn buffer_overflow(field: &str, max_size: usize) -> Self {
        Self::new("BUFFER_OVERFLOW", "Data too large for buffer")
            .with_hint(format!("Maximum size is {max_size} bytes"))
            .with_context("field", field)
    }

    /// Create a UTF-8 validation error.
    pub fn invalid_utf8(field: &str) -> Self {
        Self::new("INVALID_UTF8", format!("Invalid UTF-8 in {field}"))
            .with_hint("Ensure string contains valid UTF-8")
            .with_context("field", field)
    }

    /// Create a null pointer error.
    pub fn null_pointer(param: &str) -> Self {
        Self::new("NULL_POINTER", format!("Null pointer for {param}"))
            .with_hint("Ensure pointer is non-null")
            .with_context("parameter", param)
    }

    /// Create an out of bounds error.
    pub fn out_of_bounds(index: usize, max: usize) -> Self {
        Self::new("OUT_OF_BOUNDS", format!("Index {index} out of bounds"))
            .with_hint(format!("Valid range is 0..{max}"))
            .with_context("index", index.to_string())
            .with_context("max", max.to_string())
    }

    /// Create a marshaling error.
    pub fn marshal_failed(type_name: &str, reason: &str) -> Self {
        Self::new("MARSHAL_FAILED", format!("Failed to marshal {type_name}"))
            .with_hint(reason)
            .with_context("type", type_name)
    }
}

impl fmt::Display for MarshalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)?;
        if let Some(hint) = &self.hint {
            write!(f, " (hint: {})", hint)?;
        }
        if !self.context.is_empty() {
            write!(f, " context: {:?}", self.context)?;
        }
        Ok(())
    }
}

impl std::error::Error for MarshalError {}

/// C-compatible error representation.
///
/// This struct can be safely passed across FFI boundaries. All string fields
/// are null-terminated C strings allocated on the Rust heap.
///
/// # Memory Management
///
/// All non-null pointers must be freed by calling [`MarshalErrorC::free`].
/// The C/Dart side is responsible for calling the free function.
///
/// # Layout
///
/// ```c
/// typedef struct {
///     uint32_t code;       // Numeric error code
///     char* message;       // Required error message
///     char* hint;          // Optional hint (may be NULL)
///     char* context;       // Optional JSON context (may be NULL)
/// } MarshalErrorC;
/// ```
#[repr(C)]
pub struct MarshalErrorC {
    /// Numeric error code (hash of code string).
    pub code: u32,

    /// Error message as null-terminated C string (never null).
    pub message: *mut c_char,

    /// Optional hint as null-terminated C string (null if not present).
    pub hint: *mut c_char,

    /// Optional context as JSON string (null if not present).
    pub context: *mut c_char,
}

impl MarshalErrorC {
    /// Convert from C representation back to Rust error.
    ///
    /// # Safety
    ///
    /// - All non-null pointers must point to valid null-terminated C strings
    /// - This consumes ownership of the C strings (they are freed)
    ///
    /// # Example
    ///
    /// ```
    /// # use keyrx_core::ffi::marshal::error::MarshalError;
    /// let error = MarshalError::new("TEST", "test");
    /// let c_error = error.to_c();
    /// let back = unsafe { c_error.into_rust() };
    /// assert_eq!(back.code(), "ERROR_HASH"); // Code is reconstructed
    /// ```
    #[allow(unsafe_code)]
    pub unsafe fn into_rust(self) -> MarshalError {
        // Convert message (required)
        let message = if !self.message.is_null() {
            from_c_string(self.message)
        } else {
            "Unknown error".to_string()
        };

        // Convert hint (optional)
        let hint = if !self.hint.is_null() {
            Some(from_c_string(self.hint))
        } else {
            None
        };

        // Convert context (optional JSON)
        let context = if !self.context.is_null() {
            let json = from_c_string(self.context);
            serde_json::from_str(&json).unwrap_or_default()
        } else {
            HashMap::new()
        };

        // Reconstruct code string from hash
        // In production, use a proper error code registry
        let code = format!("ERROR_{}", self.code);

        MarshalError {
            code,
            message,
            hint,
            context,
        }
    }

    /// Free all allocated strings.
    ///
    /// # Safety
    ///
    /// - All non-null pointers must point to valid C strings allocated by Rust
    /// - This function must be called exactly once per error
    /// - After calling, the error struct becomes invalid
    #[allow(unsafe_code)]
    pub unsafe fn free(self) {
        if !self.message.is_null() {
            let _ = CString::from_raw(self.message);
        }
        if !self.hint.is_null() {
            let _ = CString::from_raw(self.hint);
        }
        if !self.context.is_null() {
            let _ = CString::from_raw(self.context);
        }
    }
}

/// Convert a string to a C string, with fallback.
///
/// Replaces null bytes with "\0" to avoid CString errors.
#[allow(clippy::unwrap_used)] // Fallback is guaranteed valid
fn to_c_string(s: &str, fallback: &str) -> *mut c_char {
    let cleaned = s.replace('\0', "\\0");
    CString::new(cleaned)
        .unwrap_or_else(|_| CString::new(fallback).unwrap())
        .into_raw()
}

/// Convert a C string back to Rust String.
///
/// # Safety
///
/// - Pointer must be non-null and point to valid null-terminated C string
/// - This consumes ownership (frees the C string)
#[allow(unsafe_code)]
unsafe fn from_c_string(ptr: *mut c_char) -> String {
    CString::from_raw(ptr)
        .into_string()
        .unwrap_or_else(|_| "Invalid UTF-8".to_string())
}

/// Hash an error code string to a u32.
///
/// Uses DJB2 hash algorithm. In production, use a proper error code registry
/// for stable, documented codes.
fn hash_error_code(code: &str) -> u32 {
    let mut hash: u32 = 5381;
    for byte in code.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u32);
    }
    hash
}

/// Convert from basic FfiError to MarshalError.
impl From<crate::ffi::error::FfiError> for MarshalError {
    fn from(error: crate::ffi::error::FfiError) -> Self {
        let mut marshal_error = Self::new(error.code, error.message);

        // Convert details JSON to context if present
        if let Some(details) = error.details {
            if let Ok(map) = serde_json::from_value::<HashMap<String, String>>(details) {
                for (k, v) in map {
                    marshal_error = marshal_error.with_context(k, v);
                }
            }
        }

        marshal_error
    }
}

/// Convert from MarshalError to basic FfiError.
impl From<MarshalError> for crate::ffi::error::FfiError {
    fn from(error: MarshalError) -> Self {
        let mut ffi_error = Self::new(error.code, error.message);

        // Combine hint and context into details
        let mut details = serde_json::Map::new();

        if let Some(hint) = error.hint {
            details.insert("hint".to_string(), serde_json::Value::String(hint));
        }

        if !error.context.is_empty() {
            for (k, v) in error.context {
                details.insert(k, serde_json::Value::String(v));
            }
        }

        if !details.is_empty() {
            ffi_error.details = Some(serde_json::Value::Object(details));
        }

        ffi_error
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marshal_error_creation() {
        let error = MarshalError::new("TEST_CODE", "test message");
        assert_eq!(error.code(), "TEST_CODE");
        assert_eq!(error.message(), "test message");
        assert!(error.hint().is_none());
        assert!(error.context().is_empty());
    }

    #[test]
    fn test_marshal_error_with_hint() {
        let error = MarshalError::new("TEST", "message").with_hint("Try this solution");
        assert_eq!(error.hint(), Some("Try this solution"));
    }

    #[test]
    fn test_marshal_error_with_context() {
        let error = MarshalError::new("TEST", "message")
            .with_context("field", "name")
            .with_context("value", "test");

        assert_eq!(error.context().get("field"), Some(&"name".to_string()));
        assert_eq!(error.context().get("value"), Some(&"test".to_string()));
    }

    #[test]
    fn test_buffer_overflow_error() {
        let error = MarshalError::buffer_overflow("device_name", 255);
        assert_eq!(error.code(), "BUFFER_OVERFLOW");
        assert!(error.message().contains("too large"));
        assert!(error.hint().unwrap().contains("255"));
        assert_eq!(
            error.context().get("field"),
            Some(&"device_name".to_string())
        );
    }

    #[test]
    fn test_invalid_utf8_error() {
        let error = MarshalError::invalid_utf8("name");
        assert_eq!(error.code(), "INVALID_UTF8");
        assert!(error.message().contains("name"));
    }

    #[test]
    fn test_null_pointer_error() {
        let error = MarshalError::null_pointer("device_id");
        assert_eq!(error.code(), "NULL_POINTER");
        assert_eq!(
            error.context().get("parameter"),
            Some(&"device_id".to_string())
        );
    }

    #[test]
    fn test_out_of_bounds_error() {
        let error = MarshalError::out_of_bounds(10, 5);
        assert_eq!(error.code(), "OUT_OF_BOUNDS");
        assert_eq!(error.context().get("index"), Some(&"10".to_string()));
        assert_eq!(error.context().get("max"), Some(&"5".to_string()));
    }

    #[test]
    fn test_marshal_failed_error() {
        let error = MarshalError::marshal_failed("DeviceInfo", "buffer overflow");
        assert_eq!(error.code(), "MARSHAL_FAILED");
        assert!(error.hint().unwrap().contains("overflow"));
        assert_eq!(error.context().get("type"), Some(&"DeviceInfo".to_string()));
    }

    #[test]
    fn test_error_display() {
        let error = MarshalError::new("TEST", "message").with_hint("hint text");
        let display = format!("{}", error);
        assert!(display.contains("[TEST]"));
        assert!(display.contains("message"));
        assert!(display.contains("hint: hint text"));
    }

    #[test]
    #[allow(unsafe_code)]
    fn test_c_error_roundtrip() {
        let error = MarshalError::new("TEST_CODE", "test message")
            .with_hint("test hint")
            .with_context("key", "value");

        let c_error = error.to_c();
        assert!(!c_error.message.is_null());
        assert!(!c_error.hint.is_null());
        assert!(!c_error.context.is_null());

        let back = unsafe { c_error.into_rust() };
        assert_eq!(back.message(), "test message");
        assert_eq!(back.hint(), Some("test hint"));
        assert_eq!(back.context().get("key"), Some(&"value".to_string()));
    }

    #[test]
    #[allow(unsafe_code)]
    fn test_c_error_minimal() {
        let error = MarshalError::new("SIMPLE", "simple error");
        let c_error = error.to_c();

        assert!(!c_error.message.is_null());
        assert!(c_error.hint.is_null());
        assert!(c_error.context.is_null());

        unsafe {
            c_error.free();
        }
    }

    #[test]
    fn test_hash_error_code() {
        let hash1 = hash_error_code("TEST_CODE");
        let hash2 = hash_error_code("TEST_CODE");
        let hash3 = hash_error_code("OTHER_CODE");

        assert_eq!(hash1, hash2); // Same input = same hash
        assert_ne!(hash1, hash3); // Different input = different hash
    }

    #[test]
    fn test_from_ffi_error() {
        let ffi_error = crate::ffi::error::FfiError::new("FFI_CODE", "ffi message");
        let marshal_error: MarshalError = ffi_error.into();

        assert_eq!(marshal_error.code(), "FFI_CODE");
        assert_eq!(marshal_error.message(), "ffi message");
    }

    #[test]
    fn test_into_ffi_error() {
        let marshal_error =
            MarshalError::new("MARSHAL_CODE", "marshal message").with_hint("some hint");
        let ffi_error: crate::ffi::error::FfiError = marshal_error.into();

        assert_eq!(ffi_error.code, "MARSHAL_CODE");
        assert_eq!(ffi_error.message, "marshal message");
        assert!(ffi_error.details.is_some());
    }

    #[test]
    fn test_to_c_string_with_null_bytes() {
        let s = "test\0string";
        let c_str = to_c_string(s, "fallback");
        assert!(!c_str.is_null());
        #[allow(unsafe_code)]
        unsafe {
            let back = CString::from_raw(c_str).into_string().unwrap();
            assert_eq!(back, "test\\0string");
        }
    }
}
