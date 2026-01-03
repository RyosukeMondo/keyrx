//! Error types for keyrx_core
//!
//! This module defines the error hierarchy for the core library.
//! All errors use thiserror for automatic trait implementations.

use alloc::string::String;

/// Core library error type
///
/// Represents errors that can occur in the platform-agnostic core library.
/// These errors are designed to be lightweight and no_std compatible.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    /// Invalid state transition or state violation
    ///
    /// # Examples
    ///
    /// ```
    /// use keyrx_core::error::CoreError;
    ///
    /// let err = CoreError::InvalidState {
    ///     message: "Cannot activate layer that doesn't exist".to_string(),
    /// };
    /// ```
    #[error("Invalid state: {message}")]
    InvalidState {
        /// Description of the invalid state
        message: String,
    },

    /// Validation error for configuration or input data
    ///
    /// # Examples
    ///
    /// ```
    /// use keyrx_core::error::CoreError;
    ///
    /// let err = CoreError::Validation {
    ///     field: "key_code".to_string(),
    ///     reason: "Key code must be between 0 and 255".to_string(),
    /// };
    /// ```
    #[error("Validation error in field '{field}': {reason}")]
    Validation {
        /// Field that failed validation
        field: String,
        /// Reason for validation failure
        reason: String,
    },

    /// Configuration error
    ///
    /// # Examples
    ///
    /// ```
    /// use keyrx_core::error::CoreError;
    ///
    /// let err = CoreError::Config {
    ///     message: "Missing required field 'layers'".to_string(),
    /// };
    /// ```
    #[error("Configuration error: {message}")]
    Config {
        /// Description of the configuration error
        message: String,
    },
}

/// Result type alias for core library operations
///
/// This is a convenience type alias for operations that can fail with a CoreError.
///
/// # Examples
///
/// ```
/// use keyrx_core::error::{CoreResult, CoreError};
///
/// fn validate_key_code(code: u32) -> CoreResult<u8> {
///     if code > 255 {
///         return Err(CoreError::Validation {
///             field: "key_code".to_string(),
///             reason: format!("Key code {} exceeds maximum value 255", code),
///         });
///     }
///     Ok(code as u8)
/// }
/// ```
pub type CoreResult<T> = Result<T, CoreError>;

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_invalid_state_error() {
        let err = CoreError::InvalidState {
            message: "test error".to_string(),
        };
        assert!(err.to_string().contains("Invalid state"));
        assert!(err.to_string().contains("test error"));
    }

    #[test]
    fn test_validation_error() {
        let err = CoreError::Validation {
            field: "key_code".to_string(),
            reason: "invalid value".to_string(),
        };
        let err_string = err.to_string();
        assert!(err_string.contains("Validation error"));
        assert!(err_string.contains("key_code"));
        assert!(err_string.contains("invalid value"));
    }

    #[test]
    fn test_config_error() {
        let err = CoreError::Config {
            message: "missing field".to_string(),
        };
        assert!(err.to_string().contains("Configuration error"));
        assert!(err.to_string().contains("missing field"));
    }

    #[test]
    fn test_core_result_ok() {
        fn returns_ok() -> CoreResult<i32> {
            Ok(42)
        }
        assert_eq!(returns_ok().unwrap(), 42);
    }

    #[test]
    fn test_core_result_err() {
        fn returns_err() -> CoreResult<i32> {
            Err(CoreError::InvalidState {
                message: "test".to_string(),
            })
        }
        assert!(returns_err().is_err());
    }
}
