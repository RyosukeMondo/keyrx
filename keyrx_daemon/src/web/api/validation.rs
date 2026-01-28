//! Request validation middleware and utilities.
//!
//! This module provides comprehensive request validation to prevent invalid
//! or malicious API requests from reaching business logic.

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};

use super::error::ApiError;

/// Maximum request body size (1MB) to prevent memory exhaustion
pub const MAX_BODY_SIZE: usize = 1024 * 1024;

/// Maximum profile name length to prevent path traversal attacks
pub const MAX_NAME_LENGTH: usize = 64;

/// Maximum config source length (512KB) to prevent memory exhaustion
pub const MAX_CONFIG_LENGTH: usize = 512 * 1024;

/// Validates profile name for security and correctness.
///
/// # Security Checks
/// - No path traversal (../)
/// - No path separators (/ or \)
/// - No null bytes
/// - Length <= MAX_NAME_LENGTH
/// - Only alphanumeric, dash, underscore
///
/// # Arguments
///
/// * `name` - Profile name to validate
///
/// # Returns
///
/// * `Ok(())` - Name is valid
/// * `Err(ApiError::BadRequest)` - Name is invalid with reason
///
/// # Examples
///
/// ```
/// use keyrx_daemon::web::api::validation::validate_profile_name;
///
/// assert!(validate_profile_name("gaming").is_ok());
/// assert!(validate_profile_name("my-profile_v2").is_ok());
/// assert!(validate_profile_name("../etc/passwd").is_err());
/// assert!(validate_profile_name("con").is_err()); // Windows reserved
/// ```
pub fn validate_profile_name(name: &str) -> Result<(), ApiError> {
    // Check length
    if name.is_empty() {
        return Err(ApiError::BadRequest(
            "Profile name cannot be empty".to_string(),
        ));
    }

    if name.len() > MAX_NAME_LENGTH {
        return Err(ApiError::BadRequest(format!(
            "Profile name too long (max {} characters, got {})",
            MAX_NAME_LENGTH,
            name.len()
        )));
    }

    // Check for path traversal
    if name.contains("..") {
        return Err(ApiError::BadRequest(
            "Profile name cannot contain path traversal (..)".to_string(),
        ));
    }

    // Check for path separators
    if name.contains('/') || name.contains('\\') {
        return Err(ApiError::BadRequest(
            "Profile name cannot contain path separators (/ or \\)".to_string(),
        ));
    }

    // Check for null bytes
    if name.contains('\0') {
        return Err(ApiError::BadRequest(
            "Profile name cannot contain null bytes".to_string(),
        ));
    }

    // Check for Windows reserved names (case-insensitive)
    let name_lower = name.to_lowercase();
    const WINDOWS_RESERVED: &[&str] = &[
        "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
        "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
    ];

    if WINDOWS_RESERVED.contains(&name_lower.as_str()) {
        return Err(ApiError::BadRequest(format!(
            "Profile name '{}' is reserved on Windows",
            name
        )));
    }

    // Check for valid characters (alphanumeric, dash, underscore, space)
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == ' ')
    {
        return Err(ApiError::BadRequest(
            "Profile name can only contain letters, numbers, dashes, underscores, and spaces"
                .to_string(),
        ));
    }

    // Check for leading/trailing whitespace
    if name.trim() != name {
        return Err(ApiError::BadRequest(
            "Profile name cannot have leading or trailing whitespace".to_string(),
        ));
    }

    Ok(())
}

/// Validates device ID for security and correctness.
///
/// Device IDs should be simple identifiers without path components.
pub fn validate_device_id(id: &str) -> Result<(), ApiError> {
    if id.is_empty() {
        return Err(ApiError::BadRequest(
            "Device ID cannot be empty".to_string(),
        ));
    }

    if id.len() > 256 {
        return Err(ApiError::BadRequest(format!(
            "Device ID too long (max 256 characters, got {})",
            id.len()
        )));
    }

    // No path traversal or separators
    if id.contains("..") || id.contains('/') || id.contains('\\') {
        return Err(ApiError::BadRequest(
            "Device ID cannot contain path components".to_string(),
        ));
    }

    Ok(())
}

/// Validates pagination parameters.
pub fn validate_pagination(limit: Option<usize>, offset: Option<usize>) -> Result<(), ApiError> {
    if let Some(limit) = limit {
        if limit == 0 {
            return Err(ApiError::BadRequest(
                "Limit must be greater than 0".to_string(),
            ));
        }
        if limit > 1000 {
            return Err(ApiError::BadRequest("Limit cannot exceed 1000".to_string()));
        }
    }

    if let Some(offset) = offset {
        if offset > 1_000_000 {
            return Err(ApiError::BadRequest(
                "Offset too large (max 1,000,000)".to_string(),
            ));
        }
    }

    Ok(())
}

/// Validates configuration source code length.
pub fn validate_config_source(source: &str) -> Result<(), ApiError> {
    if source.len() > MAX_CONFIG_LENGTH {
        return Err(ApiError::BadRequest(format!(
            "Configuration too large (max {} bytes, got {})",
            MAX_CONFIG_LENGTH,
            source.len()
        )));
    }
    Ok(())
}

/// Middleware to enforce request timeout (5 seconds default).
///
/// Prevents slow loris attacks and ensures responsive API.
pub async fn timeout_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    let timeout = tokio::time::Duration::from_secs(5);

    match tokio::time::timeout(timeout, next.run(req)).await {
        Ok(response) => Ok(response),
        Err(_) => Err(StatusCode::REQUEST_TIMEOUT),
    }
}

/// Middleware to limit request body size.
///
/// Prevents memory exhaustion from large uploads.
pub async fn size_limit_middleware(req: Request, next: Next) -> Response {
    // Extract content-length header
    let content_length = req
        .headers()
        .get(axum::http::header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<usize>().ok());

    // Reject if exceeds limit
    if let Some(length) = content_length {
        if length > MAX_BODY_SIZE {
            let error_response = ApiError::BadRequest(format!(
                "Request body too large (max {} bytes, got {})",
                MAX_BODY_SIZE, length
            ));
            return error_response.into_response();
        }
    }

    next.run(req).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_profile_name_valid() {
        assert!(validate_profile_name("gaming").is_ok());
        assert!(validate_profile_name("my-profile").is_ok());
        assert!(validate_profile_name("profile_v2").is_ok());
        assert!(validate_profile_name("Profile 1").is_ok());
        assert!(validate_profile_name("a").is_ok());
    }

    #[test]
    fn test_validate_profile_name_empty() {
        let err = validate_profile_name("").unwrap_err();
        assert!(matches!(err, ApiError::BadRequest(_)));
    }

    #[test]
    fn test_validate_profile_name_too_long() {
        let long_name = "a".repeat(MAX_NAME_LENGTH + 1);
        let err = validate_profile_name(&long_name).unwrap_err();
        assert!(matches!(err, ApiError::BadRequest(_)));
    }

    #[test]
    fn test_validate_profile_name_path_traversal() {
        assert!(validate_profile_name("../passwd").is_err());
        assert!(validate_profile_name("..\\windows").is_err());
        assert!(validate_profile_name("dir/../file").is_err());
    }

    #[test]
    fn test_validate_profile_name_path_separators() {
        assert!(validate_profile_name("dir/file").is_err());
        assert!(validate_profile_name("dir\\file").is_err());
    }

    #[test]
    fn test_validate_profile_name_null_byte() {
        assert!(validate_profile_name("test\0file").is_err());
    }

    #[test]
    fn test_validate_profile_name_windows_reserved() {
        assert!(validate_profile_name("con").is_err());
        assert!(validate_profile_name("CON").is_err());
        assert!(validate_profile_name("prn").is_err());
        assert!(validate_profile_name("aux").is_err());
        assert!(validate_profile_name("nul").is_err());
        assert!(validate_profile_name("com1").is_err());
        assert!(validate_profile_name("lpt1").is_err());
    }

    #[test]
    fn test_validate_profile_name_invalid_chars() {
        assert!(validate_profile_name("test@file").is_err());
        assert!(validate_profile_name("test#file").is_err());
        assert!(validate_profile_name("test$file").is_err());
    }

    #[test]
    fn test_validate_profile_name_whitespace() {
        assert!(validate_profile_name(" test").is_err());
        assert!(validate_profile_name("test ").is_err());
        assert!(validate_profile_name("  test  ").is_err());
    }

    #[test]
    fn test_validate_device_id_valid() {
        assert!(validate_device_id("device-123").is_ok());
        assert!(validate_device_id("keyboard_0").is_ok());
    }

    #[test]
    fn test_validate_device_id_empty() {
        assert!(validate_device_id("").is_err());
    }

    #[test]
    fn test_validate_device_id_too_long() {
        let long_id = "a".repeat(257);
        assert!(validate_device_id(&long_id).is_err());
    }

    #[test]
    fn test_validate_device_id_path_components() {
        assert!(validate_device_id("../device").is_err());
        assert!(validate_device_id("dir/device").is_err());
        assert!(validate_device_id("dir\\device").is_err());
    }

    #[test]
    fn test_validate_pagination_valid() {
        assert!(validate_pagination(Some(10), Some(0)).is_ok());
        assert!(validate_pagination(Some(1000), Some(999999)).is_ok());
        assert!(validate_pagination(None, None).is_ok());
    }

    #[test]
    fn test_validate_pagination_zero_limit() {
        assert!(validate_pagination(Some(0), None).is_err());
    }

    #[test]
    fn test_validate_pagination_limit_too_large() {
        assert!(validate_pagination(Some(1001), None).is_err());
    }

    #[test]
    fn test_validate_pagination_offset_too_large() {
        assert!(validate_pagination(None, Some(1_000_001)).is_err());
    }

    #[test]
    fn test_validate_config_source_valid() {
        assert!(validate_config_source("let config = {};").is_ok());
        let large_but_valid = "a".repeat(MAX_CONFIG_LENGTH);
        assert!(validate_config_source(&large_but_valid).is_ok());
    }

    #[test]
    fn test_validate_config_source_too_large() {
        let too_large = "a".repeat(MAX_CONFIG_LENGTH + 1);
        assert!(validate_config_source(&too_large).is_err());
    }
}
