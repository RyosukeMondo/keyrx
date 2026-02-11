//! Device ID validation (VAL-006)
//!
//! Validates device IDs to prevent:
//! - Path traversal attacks
//! - Injection attacks
//! - Control characters
//! - Length violations

use super::{ValidationError, ValidationResult};

/// Maximum device ID length
const MAX_DEVICE_ID_LENGTH: usize = 256;

/// Minimum device ID length
const MIN_DEVICE_ID_LENGTH: usize = 1;

/// Validates a device ID according to strict security rules.
///
/// Rules:
/// - Length: 1-256 characters
/// - No path traversal patterns (..)
/// - No path separators (/ or \)
/// - No control characters
/// - No null bytes
///
/// # Arguments
///
/// * `id` - The device ID to validate
///
/// # Returns
///
/// * `Ok(())` if validation passes
/// * `Err(ValidationError)` with details if validation fails
///
/// # Examples
///
/// ```
/// use keyrx_daemon::validation::device::validate_device_id;
///
/// // Valid IDs
/// assert!(validate_device_id("device-123").is_ok());
/// assert!(validate_device_id("keyboard_usb_001").is_ok());
///
/// // Invalid IDs
/// assert!(validate_device_id("").is_err());
/// assert!(validate_device_id("..").is_err());
/// assert!(validate_device_id("test/device").is_err());
/// ```
pub fn validate_device_id(id: &str) -> ValidationResult<()> {
    // Check empty
    if id.is_empty() {
        return Err(ValidationError::InvalidContent(
            "Device ID cannot be empty".to_string(),
        ));
    }

    // Check length constraints
    if id.len() < MIN_DEVICE_ID_LENGTH {
        return Err(ValidationError::InvalidContent(format!(
            "Device ID too short: must be at least {} character(s)",
            MIN_DEVICE_ID_LENGTH
        )));
    }

    if id.len() > MAX_DEVICE_ID_LENGTH {
        return Err(ValidationError::InvalidContent(format!(
            "Device ID too long: {} characters (max {})",
            id.len(),
            MAX_DEVICE_ID_LENGTH
        )));
    }

    // Check for path traversal patterns
    if id.contains("..") {
        return Err(ValidationError::InvalidContent(
            "Device ID cannot contain path traversal (..)".to_string(),
        ));
    }

    // Check for path separators
    if id.contains('/') || id.contains('\\') {
        return Err(ValidationError::InvalidContent(
            "Device ID cannot contain path separators (/ or \\)".to_string(),
        ));
    }

    // Check for null bytes
    if id.contains('\0') {
        return Err(ValidationError::InvalidContent(
            "Device ID cannot contain null bytes".to_string(),
        ));
    }

    // Check for control characters (excluding normal ASCII printable)
    if id.chars().any(|c| c.is_control()) {
        return Err(ValidationError::InvalidContent(
            "Device ID cannot contain control characters".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_device_ids() {
        assert!(validate_device_id("device-123").is_ok());
        assert!(validate_device_id("keyboard_usb_001").is_ok());
        assert!(validate_device_id("Test-Device_456").is_ok());
        assert!(validate_device_id("a").is_ok()); // Min length
        assert!(validate_device_id(&"a".repeat(256)).is_ok()); // Max length
        assert!(validate_device_id("device.usb").is_ok()); // dots are allowed
    }

    #[test]
    fn test_empty_device_id() {
        assert!(validate_device_id("").is_err());
    }

    #[test]
    fn test_device_id_too_long() {
        let long_id = "a".repeat(257);
        assert!(validate_device_id(&long_id).is_err());
    }

    #[test]
    fn test_device_id_path_traversal() {
        assert!(validate_device_id("..").is_err());
        assert!(validate_device_id("../etc/passwd").is_err());
        assert!(validate_device_id("test/../device").is_err());
    }

    #[test]
    fn test_device_id_path_separators() {
        assert!(validate_device_id("test/device").is_err());
        assert!(validate_device_id("test\\device").is_err());
        assert!(validate_device_id("/root").is_err());
        assert!(validate_device_id("C:\\Windows").is_err());
    }

    #[test]
    fn test_device_id_null_bytes() {
        assert!(validate_device_id("device\0id").is_err());
        assert!(validate_device_id("\0device").is_err());
        assert!(validate_device_id("dev\0ice").is_err());
    }

    #[test]
    fn test_device_id_control_characters() {
        assert!(validate_device_id("device\nid").is_err()); // newline
        assert!(validate_device_id("device\tid").is_err()); // tab
        assert!(validate_device_id("device\x01id").is_err()); // control char
        assert!(validate_device_id("device\x1Bid").is_err()); // escape
    }

    #[test]
    fn test_device_id_special_chars() {
        // These should be allowed (device IDs can have various formats)
        assert!(validate_device_id("device:123").is_ok()); // colon
        assert!(validate_device_id("device@host").is_ok()); // at
        assert!(validate_device_id("device#1").is_ok()); // hash
        assert!(validate_device_id("device-_").is_ok()); // dash and underscore
        assert!(validate_device_id("device.vendor.product").is_ok()); // dots
    }

    #[test]
    fn test_device_id_unicode() {
        // Unicode characters should be allowed (device IDs may use various formats)
        assert!(validate_device_id("デバイス").is_ok());
        assert!(validate_device_id("устройство").is_ok());
    }
}
