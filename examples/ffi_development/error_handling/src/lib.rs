//! FFI Error Handling Patterns
//!
//! This example demonstrates comprehensive error handling patterns for FFI,
//! including custom error types, error conversion, and error propagation.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Comprehensive FFI error type demonstrating different error patterns.
#[derive(Debug, Error, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum FfiError {
    /// Validation error with field-specific details.
    #[error("Validation error: {message}")]
    Validation {
        message: String,
        field: Option<String>,
        constraint: Option<String>,
    },

    /// Resource not found error.
    #[error("Resource not found: {resource_type} with id '{resource_id}'")]
    NotFound {
        resource_type: String,
        resource_id: String,
    },

    /// Permission denied error.
    #[error("Permission denied: {operation} on {resource}")]
    PermissionDenied { operation: String, resource: String },

    /// Resource conflict (e.g., duplicate key).
    #[error("Conflict: {message}")]
    Conflict { message: String, existing_id: Option<String> },

    /// Internal error (should not expose details to client).
    #[error("Internal error occurred")]
    Internal { debug_info: String },

    /// IO error (converted from std::io::Error).
    #[error("IO error: {message}")]
    Io { message: String },

    /// Parse/deserialization error.
    #[error("Parse error: {message}")]
    Parse { message: String, position: Option<usize> },

    /// Timeout error.
    #[error("Operation timed out after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    /// Rate limit exceeded.
    #[error("Rate limit exceeded: {limit} requests per {window_secs}s")]
    RateLimitExceeded { limit: u32, window_secs: u32, retry_after_secs: u32 },
}

impl From<std::io::Error> for FfiError {
    fn from(err: std::io::Error) -> Self {
        FfiError::Io {
            message: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for FfiError {
    fn from(err: serde_json::Error) -> Self {
        FfiError::Parse {
            message: err.to_string(),
            position: err.column().map(|c| c as usize),
        }
    }
}

/// Validate input and return detailed validation errors.
///
/// Demonstrates:
/// - Field-specific validation errors
/// - Multiple validation constraints
/// - Detailed error messages for debugging
// #[ffi_export(domain = "validation", version = "1.0")]
pub fn validate_input(input: &str, min_length: usize, max_length: usize) -> Result<(), FfiError> {
    if input.is_empty() {
        return Err(FfiError::Validation {
            message: "Input cannot be empty".to_string(),
            field: Some("input".to_string()),
            constraint: Some("non_empty".to_string()),
        });
    }

    if input.len() < min_length {
        return Err(FfiError::Validation {
            message: format!("Input must be at least {} characters", min_length),
            field: Some("input".to_string()),
            constraint: Some(format!("min_length:{}", min_length)),
        });
    }

    if input.len() > max_length {
        return Err(FfiError::Validation {
            message: format!("Input must be at most {} characters", max_length),
            field: Some("input".to_string()),
            constraint: Some(format!("max_length:{}", max_length)),
        });
    }

    Ok(())
}

/// Get a resource by ID, demonstrating NotFound errors.
// #[ffi_export(domain = "resource", version = "1.0")]
pub fn get_resource(resource_type: &str, id: &str) -> Result<String, FfiError> {
    if id.is_empty() {
        return Err(FfiError::Validation {
            message: "Resource ID cannot be empty".to_string(),
            field: Some("id".to_string()),
            constraint: Some("non_empty".to_string()),
        });
    }

    // Simulate resource lookup
    if id == "nonexistent" {
        return Err(FfiError::NotFound {
            resource_type: resource_type.to_string(),
            resource_id: id.to_string(),
        });
    }

    Ok(format!("Resource data for {}", id))
}

/// Create a resource, demonstrating conflict errors.
// #[ffi_export(domain = "resource", version = "1.0")]
pub fn create_resource(id: &str, data: &str) -> Result<String, FfiError> {
    validate_input(id, 1, 100)?;
    validate_input(data, 1, 1000)?;

    // Simulate conflict check
    if id == "existing-resource" {
        return Err(FfiError::Conflict {
            message: "Resource with this ID already exists".to_string(),
            existing_id: Some(id.to_string()),
        });
    }

    Ok(format!("Created resource: {}", id))
}

/// Delete a resource, demonstrating permission errors.
// #[ffi_export(domain = "resource", version = "1.0")]
pub fn delete_resource(id: &str, user_role: &str) -> Result<(), FfiError> {
    // Check permissions
    if user_role != "admin" {
        return Err(FfiError::PermissionDenied {
            operation: "delete".to_string(),
            resource: format!("resource:{}", id),
        });
    }

    // Simulate deletion
    Ok(())
}

/// Parse JSON data, demonstrating parse errors.
// #[ffi_export(domain = "parser", version = "1.0")]
pub fn parse_json(json_str: &str) -> Result<serde_json::Value, FfiError> {
    // The ? operator will automatically convert serde_json::Error to FfiError
    let value: serde_json::Value = serde_json::from_str(json_str)?;
    Ok(value)
}

/// Perform an operation with timeout, demonstrating timeout errors.
// #[ffi_export(domain = "operation", version = "1.0")]
pub fn operation_with_timeout(timeout_ms: u64) -> Result<String, FfiError> {
    if timeout_ms == 0 {
        return Err(FfiError::Timeout { timeout_ms });
    }

    // Simulate operation
    Ok("Operation completed".to_string())
}

/// Rate-limited operation.
// #[ffi_export(domain = "operation", version = "1.0")]
pub fn rate_limited_operation(request_count: u32) -> Result<String, FfiError> {
    const RATE_LIMIT: u32 = 100;
    const WINDOW_SECS: u32 = 60;

    if request_count > RATE_LIMIT {
        return Err(FfiError::RateLimitExceeded {
            limit: RATE_LIMIT,
            window_secs: WINDOW_SECS,
            retry_after_secs: 30,
        });
    }

    Ok("Request processed".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_empty_input() {
        let result = validate_input("", 5, 100);
        assert!(result.is_err());
        match result.unwrap_err() {
            FfiError::Validation { field, .. } => {
                assert_eq!(field, Some("input".to_string()));
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_validation_too_short() {
        let result = validate_input("abc", 5, 100);
        assert!(result.is_err());
        match result.unwrap_err() {
            FfiError::Validation { constraint, .. } => {
                assert!(constraint.unwrap().contains("min_length"));
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_validation_success() {
        let result = validate_input("valid input", 5, 100);
        assert!(result.is_ok());
    }

    #[test]
    fn test_not_found_error() {
        let result = get_resource("profile", "nonexistent");
        assert!(result.is_err());
        match result.unwrap_err() {
            FfiError::NotFound { resource_id, .. } => {
                assert_eq!(resource_id, "nonexistent");
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_conflict_error() {
        let result = create_resource("existing-resource", "data");
        assert!(result.is_err());
        match result.unwrap_err() {
            FfiError::Conflict { existing_id, .. } => {
                assert!(existing_id.is_some());
            }
            _ => panic!("Expected Conflict error"),
        }
    }

    #[test]
    fn test_permission_denied() {
        let result = delete_resource("resource-1", "user");
        assert!(result.is_err());
        match result.unwrap_err() {
            FfiError::PermissionDenied { operation, .. } => {
                assert_eq!(operation, "delete");
            }
            _ => panic!("Expected PermissionDenied error"),
        }
    }

    #[test]
    fn test_permission_allowed() {
        let result = delete_resource("resource-1", "admin");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_error() {
        let result = parse_json("invalid json {");
        assert!(result.is_err());
        match result.unwrap_err() {
            FfiError::Parse { .. } => {}
            _ => panic!("Expected Parse error"),
        }
    }

    #[test]
    fn test_parse_success() {
        let result = parse_json(r#"{"key": "value"}"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_timeout_error() {
        let result = operation_with_timeout(0);
        assert!(result.is_err());
        match result.unwrap_err() {
            FfiError::Timeout { timeout_ms } => {
                assert_eq!(timeout_ms, 0);
            }
            _ => panic!("Expected Timeout error"),
        }
    }

    #[test]
    fn test_rate_limit_error() {
        let result = rate_limited_operation(150);
        assert!(result.is_err());
        match result.unwrap_err() {
            FfiError::RateLimitExceeded { limit, retry_after_secs, .. } => {
                assert_eq!(limit, 100);
                assert_eq!(retry_after_secs, 30);
            }
            _ => panic!("Expected RateLimitExceeded error"),
        }
    }

    #[test]
    fn test_error_serialization() {
        let error = FfiError::NotFound {
            resource_type: "profile".to_string(),
            resource_id: "test-123".to_string(),
        };

        let json = serde_json::to_string(&error).unwrap();
        let deserialized: FfiError = serde_json::from_str(&json).unwrap();

        match deserialized {
            FfiError::NotFound { resource_id, .. } => {
                assert_eq!(resource_id, "test-123");
            }
            _ => panic!("Deserialization failed"),
        }
    }
}
