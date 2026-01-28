//! Profile name validation (VAL-001)
//!
//! Validates profile names to prevent:
//! - Path traversal attacks
//! - Windows reserved names
//! - Invalid characters
//! - Length violations

use super::{ValidationError, ValidationResult};
use regex::Regex;
use std::sync::OnceLock;

/// Maximum profile name length (alphanumeric, dash, underscore only)
const MAX_NAME_LENGTH: usize = 64;

/// Minimum profile name length
const MIN_NAME_LENGTH: usize = 1;

/// Windows reserved names (case-insensitive)
const WINDOWS_RESERVED: &[&str] = &[
    ".", "..", "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7",
    "com8", "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
];

/// Get the compiled regex pattern for valid profile names
fn profile_name_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r"^[a-zA-Z0-9_-]{1,64}$").expect("Failed to compile profile name regex")
    })
}

/// Validates a profile name according to strict security rules.
///
/// Rules:
/// - Length: 1-64 characters
/// - Allowed characters: alphanumeric (a-zA-Z0-9), dash (-), underscore (_)
/// - Rejected: Windows reserved names (con, prn, aux, etc.)
/// - Rejected: Path traversal patterns (., ..)
/// - Rejected: Empty strings or whitespace-only strings
///
/// # Arguments
///
/// * `name` - The profile name to validate
///
/// # Returns
///
/// * `Ok(())` if validation passes
/// * `Err(ValidationError)` with details if validation fails
///
/// # Examples
///
/// ```
/// use keyrx_daemon::validation::profile_name::validate_profile_name;
///
/// // Valid names
/// assert!(validate_profile_name("default").is_ok());
/// assert!(validate_profile_name("my-profile_123").is_ok());
///
/// // Invalid names
/// assert!(validate_profile_name("").is_err());
/// assert!(validate_profile_name("..").is_err());
/// assert!(validate_profile_name("con").is_err());
/// assert!(validate_profile_name("my profile").is_err()); // spaces not allowed
/// ```
pub fn validate_profile_name(name: &str) -> ValidationResult<()> {
    // Check empty or whitespace-only
    if name.trim().is_empty() {
        return Err(ValidationError::InvalidProfileName(
            "Name cannot be empty or whitespace-only".to_string(),
        ));
    }

    // Check length constraints
    if name.len() < MIN_NAME_LENGTH {
        return Err(ValidationError::InvalidProfileName(format!(
            "Name too short: must be at least {} character(s)",
            MIN_NAME_LENGTH
        )));
    }

    if name.len() > MAX_NAME_LENGTH {
        return Err(ValidationError::InvalidProfileName(format!(
            "Name too long: {} characters (max {})",
            name.len(),
            MAX_NAME_LENGTH
        )));
    }

    // Check regex pattern (alphanumeric, dash, underscore only)
    if !profile_name_regex().is_match(name) {
        return Err(ValidationError::InvalidProfileName(
            "Name must contain only alphanumeric characters, dashes, and underscores".to_string(),
        ));
    }

    // Check for Windows reserved names (case-insensitive)
    let name_lower = name.to_lowercase();
    if WINDOWS_RESERVED.contains(&name_lower.as_str()) {
        return Err(ValidationError::InvalidProfileName(format!(
            "Reserved name '{}' is not allowed (Windows compatibility)",
            name
        )));
    }

    // Check for path traversal patterns explicitly
    if name.contains("..") || name == "." {
        return Err(ValidationError::InvalidProfileName(
            "Path traversal patterns (. or ..) are not allowed".to_string(),
        ));
    }

    // Check for null bytes
    if name.contains('\0') {
        return Err(ValidationError::InvalidProfileName(
            "Null bytes are not allowed".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_names() {
        assert!(validate_profile_name("default").is_ok());
        assert!(validate_profile_name("my-profile").is_ok());
        assert!(validate_profile_name("my_profile").is_ok());
        assert!(validate_profile_name("profile123").is_ok());
        assert!(validate_profile_name("Profile-123_ABC").is_ok());
        assert!(validate_profile_name("a").is_ok()); // Min length
        assert!(validate_profile_name("a".repeat(64).as_str()).is_ok()); // Max length
    }

    #[test]
    fn test_empty_and_whitespace() {
        assert!(validate_profile_name("").is_err());
        assert!(validate_profile_name(" ").is_err());
        assert!(validate_profile_name("   ").is_err());
        assert!(validate_profile_name("\t").is_err());
        assert!(validate_profile_name("\n").is_err());
    }

    #[test]
    fn test_length_violations() {
        // Too long (65 characters)
        assert!(validate_profile_name(&"a".repeat(65)).is_err());
    }

    #[test]
    fn test_invalid_characters() {
        assert!(validate_profile_name("my profile").is_err()); // space
        assert!(validate_profile_name("profile!").is_err()); // exclamation
        assert!(validate_profile_name("profile@home").is_err()); // at symbol
        assert!(validate_profile_name("profile#1").is_err()); // hash
        assert!(validate_profile_name("profile$").is_err()); // dollar
        assert!(validate_profile_name("profile%").is_err()); // percent
        assert!(validate_profile_name("profile&test").is_err()); // ampersand
        assert!(validate_profile_name("profile*").is_err()); // asterisk
        assert!(validate_profile_name("profile(").is_err()); // parenthesis
        assert!(validate_profile_name("profile)").is_err());
        assert!(validate_profile_name("profile+").is_err()); // plus
        assert!(validate_profile_name("profile=").is_err()); // equals
        assert!(validate_profile_name("profile[").is_err()); // brackets
        assert!(validate_profile_name("profile]").is_err());
        assert!(validate_profile_name("profile{").is_err()); // braces
        assert!(validate_profile_name("profile}").is_err());
        assert!(validate_profile_name("profile\\").is_err()); // backslash
        assert!(validate_profile_name("profile/").is_err()); // forward slash
        assert!(validate_profile_name("profile|").is_err()); // pipe
        assert!(validate_profile_name("profile:").is_err()); // colon
        assert!(validate_profile_name("profile;").is_err()); // semicolon
        assert!(validate_profile_name("profile'").is_err()); // quote
        assert!(validate_profile_name("profile\"").is_err()); // double quote
        assert!(validate_profile_name("profile<").is_err()); // angle brackets
        assert!(validate_profile_name("profile>").is_err());
        assert!(validate_profile_name("profile,").is_err()); // comma
        assert!(validate_profile_name("profile.").is_err()); // dot
        assert!(validate_profile_name("profile?").is_err()); // question mark
    }

    #[test]
    fn test_windows_reserved_names() {
        // Case-insensitive check
        assert!(validate_profile_name("con").is_err());
        assert!(validate_profile_name("CON").is_err());
        assert!(validate_profile_name("Con").is_err());
        assert!(validate_profile_name("prn").is_err());
        assert!(validate_profile_name("PRN").is_err());
        assert!(validate_profile_name("aux").is_err());
        assert!(validate_profile_name("AUX").is_err());
        assert!(validate_profile_name("nul").is_err());
        assert!(validate_profile_name("NUL").is_err());
        assert!(validate_profile_name("com1").is_err());
        assert!(validate_profile_name("COM1").is_err());
        assert!(validate_profile_name("lpt1").is_err());
        assert!(validate_profile_name("LPT1").is_err());
    }

    #[test]
    fn test_path_traversal_patterns() {
        assert!(validate_profile_name(".").is_err());
        assert!(validate_profile_name("..").is_err());
        // Note: "../foo" would fail the regex test before reaching this check
    }

    #[test]
    fn test_null_bytes() {
        assert!(validate_profile_name("test\0").is_err());
        assert!(validate_profile_name("\0test").is_err());
        assert!(validate_profile_name("te\0st").is_err());
    }

    #[test]
    fn test_unicode_characters() {
        // Unicode characters should fail (only ASCII alphanumeric allowed)
        assert!(validate_profile_name("プロファイル").is_err()); // Japanese
        assert!(validate_profile_name("配置文件").is_err()); // Chinese
        assert!(validate_profile_name("профиль").is_err()); // Cyrillic
        assert!(validate_profile_name("profile😀").is_err()); // Emoji
        assert!(validate_profile_name("café").is_err()); // Accented characters
    }
}
