//! Input sanitization utilities (VAL-005)
//!
//! Provides functions to sanitize user input by:
//! - Escaping HTML entities
//! - Removing control characters
//! - Validating JSON structure

use super::{ValidationError, ValidationResult};

/// HTML entities that need to be escaped
const HTML_ENTITIES: &[(&str, &str)] = &[
    ("&", "&amp;"),
    ("<", "&lt;"),
    (">", "&gt;"),
    ("\"", "&quot;"),
    ("'", "&#x27;"),
    ("/", "&#x2F;"),
];

/// Escapes HTML entities in a string to prevent XSS attacks.
///
/// # Arguments
///
/// * `input` - The string to escape
///
/// # Returns
///
/// A new String with HTML entities escaped
///
/// # Examples
///
/// ```
/// use keyrx_daemon::validation::sanitization::escape_html_entities;
///
/// let input = "<script>alert('xss')</script>";
/// let escaped = escape_html_entities(input);
/// assert_eq!(escaped, "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;&#x2F;script&gt;");
/// ```
pub fn escape_html_entities(input: &str) -> String {
    let mut result = input.to_string();

    for (entity, replacement) in HTML_ENTITIES {
        result = result.replace(entity, replacement);
    }

    result
}

/// Removes control characters (except newline, carriage return, tab) from a string.
///
/// Control characters (ASCII 0-31 except \n, \r, \t) can cause issues in
/// various contexts and may be used in injection attacks.
///
/// # Arguments
///
/// * `input` - The string to sanitize
///
/// # Returns
///
/// A new String with control characters removed
///
/// # Examples
///
/// ```
/// use keyrx_daemon::validation::sanitization::remove_control_characters;
///
/// let input = "Hello\x00World\x01Test";
/// let sanitized = remove_control_characters(input);
/// assert_eq!(sanitized, "HelloWorldTest");
///
/// // Allowed control characters preserved
/// let input = "Line1\nLine2\r\nTab\there";
/// let sanitized = remove_control_characters(input);
/// assert_eq!(sanitized, "Line1\nLine2\r\nTab\there");
/// ```
pub fn remove_control_characters(input: &str) -> String {
    input
        .chars()
        .filter(|&c| {
            // Keep printable characters and allowed control characters
            !c.is_control() || c == '\n' || c == '\r' || c == '\t'
        })
        .collect()
}

/// Removes null bytes from a string.
///
/// Null bytes can cause issues with C-style string functions and may be
/// used in path traversal attacks.
///
/// # Arguments
///
/// * `input` - The string to sanitize
///
/// # Returns
///
/// A new String with null bytes removed
pub fn remove_null_bytes(input: &str) -> String {
    input.replace('\0', "")
}

/// Sanitizes a profile name for safe use in UI display.
///
/// Combines multiple sanitization steps:
/// 1. Remove control characters
/// 2. Remove null bytes
/// 3. Escape HTML entities
///
/// # Arguments
///
/// * `name` - The profile name to sanitize
///
/// # Returns
///
/// A sanitized String safe for HTML display
pub fn sanitize_profile_name_for_display(name: &str) -> String {
    let no_control = remove_control_characters(name);
    let no_nulls = remove_null_bytes(&no_control);
    escape_html_entities(&no_nulls)
}

/// Validates JSON structure without parsing content.
///
/// Performs a basic structural validation to ensure the input
/// is valid JSON before attempting to parse it.
///
/// # Arguments
///
/// * `json_str` - The JSON string to validate
///
/// # Returns
///
/// * `Ok(())` if JSON is structurally valid
/// * `Err(ValidationError)` if JSON is invalid
///
/// # Examples
///
/// ```
/// use keyrx_daemon::validation::sanitization::validate_json_structure;
///
/// let valid_json = r#"{"key": "value"}"#;
/// assert!(validate_json_structure(valid_json).is_ok());
///
/// let invalid_json = r#"{"key": "value""#;
/// assert!(validate_json_structure(invalid_json).is_err());
/// ```
pub fn validate_json_structure(json_str: &str) -> ValidationResult<()> {
    serde_json::from_str::<serde_json::Value>(json_str)
        .map_err(|e| ValidationError::InvalidContent(format!("Invalid JSON structure: {}", e)))?;

    Ok(())
}

/// Sanitizes configuration values by removing dangerous characters.
///
/// This is a general-purpose sanitizer for configuration values that:
/// - Removes control characters (except whitespace)
/// - Removes null bytes
/// - Trims leading/trailing whitespace
///
/// # Arguments
///
/// * `value` - The configuration value to sanitize
///
/// # Returns
///
/// A sanitized String
pub fn sanitize_config_value(value: &str) -> String {
    let no_control = remove_control_characters(value);
    let no_nulls = remove_null_bytes(&no_control);
    no_nulls.trim().to_string()
}

/// Checks if a string contains only safe ASCII printable characters.
///
/// # Arguments
///
/// * `input` - The string to check
///
/// # Returns
///
/// `true` if all characters are safe (ASCII printable or whitespace)
pub fn is_safe_ascii(input: &str) -> bool {
    input
        .chars()
        .all(|c| (c.is_ascii_graphic() || c.is_ascii_whitespace()) && c != '\0')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_html_entities() {
        assert_eq!(escape_html_entities("hello"), "hello");
        assert_eq!(
            escape_html_entities("<script>alert('xss')</script>"),
            "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;&#x2F;script&gt;"
        );
        assert_eq!(
            escape_html_entities("A & B < C > D"),
            "A &amp; B &lt; C &gt; D"
        );
        assert_eq!(
            escape_html_entities(r#"<a href="/test">link</a>"#),
            "&lt;a href=&quot;&#x2F;test&quot;&gt;link&lt;&#x2F;a&gt;"
        );
    }

    #[test]
    fn test_remove_control_characters() {
        // Remove null and other control characters
        assert_eq!(
            remove_control_characters("Hello\x00World\x01Test"),
            "HelloWorldTest"
        );

        // Keep newlines, carriage returns, and tabs
        assert_eq!(
            remove_control_characters("Line1\nLine2\r\nTab\there"),
            "Line1\nLine2\r\nTab\there"
        );

        // Remove all control characters except allowed ones
        let input = "Test\x00\x01\x02\x03\x04\x05\x06\x07\x08\t\n\x0B\x0C\r\x0E\x0F";
        let expected = "Test\t\n\r";
        assert_eq!(remove_control_characters(input), expected);
    }

    #[test]
    fn test_remove_null_bytes() {
        assert_eq!(remove_null_bytes("hello"), "hello");
        assert_eq!(remove_null_bytes("hello\0world"), "helloworld");
        assert_eq!(remove_null_bytes("\0test\0"), "test");
    }

    #[test]
    fn test_sanitize_profile_name_for_display() {
        // Normal name
        assert_eq!(
            sanitize_profile_name_for_display("my-profile"),
            "my-profile"
        );

        // Name with HTML entities
        assert_eq!(
            sanitize_profile_name_for_display("<script>"),
            "&lt;script&gt;"
        );

        // Name with control characters
        assert_eq!(
            sanitize_profile_name_for_display("test\x00\x01name"),
            "testname"
        );

        // Combined issues
        assert_eq!(
            sanitize_profile_name_for_display("<test\0>&"),
            "&lt;test&gt;&amp;"
        );
    }

    #[test]
    fn test_validate_json_structure() {
        // Valid JSON
        assert!(validate_json_structure(r#"{"key": "value"}"#).is_ok());
        assert!(validate_json_structure(r#"[1, 2, 3]"#).is_ok());
        assert!(validate_json_structure(r#"null"#).is_ok());
        assert!(validate_json_structure(r#"42"#).is_ok());
        assert!(validate_json_structure(r#""string""#).is_ok());

        // Invalid JSON
        assert!(validate_json_structure(r#"{"key": "value""#).is_err());
        assert!(validate_json_structure(r#"{key: value}"#).is_err()); // unquoted keys
        assert!(validate_json_structure(r#"{'key': 'value'}"#).is_err()); // single quotes
        assert!(validate_json_structure(r#"[1, 2, 3,]"#).is_err()); // trailing comma
    }

    #[test]
    fn test_sanitize_config_value() {
        // Normal value
        assert_eq!(sanitize_config_value("normal value"), "normal value");

        // Value with leading/trailing whitespace
        assert_eq!(sanitize_config_value("  trimmed  "), "trimmed");

        // Value with control characters
        assert_eq!(sanitize_config_value("test\x00\x01value"), "testvalue");

        // Value with newlines (preserved)
        assert_eq!(sanitize_config_value("line1\nline2"), "line1\nline2");
    }

    #[test]
    fn test_is_safe_ascii() {
        // Safe strings
        assert!(is_safe_ascii("hello world"));
        assert!(is_safe_ascii("123-456_789"));
        assert!(is_safe_ascii("Line1\nLine2\tTab"));

        // Unsafe strings
        assert!(!is_safe_ascii("hello\0world")); // null byte
        assert!(!is_safe_ascii("test\x01value")); // control character
        assert!(!is_safe_ascii("hello\x1Bworld")); // escape character
    }

    #[test]
    fn test_unicode_handling() {
        // Unicode should be preserved (but may not be "safe ASCII")
        let unicode_input = "プロファイル";
        let sanitized = sanitize_config_value(unicode_input);
        assert_eq!(sanitized, unicode_input);

        // But is_safe_ascii should return false
        assert!(!is_safe_ascii(unicode_input));
    }

    #[test]
    fn test_emoji_handling() {
        let emoji_input = "profile😀name";
        let sanitized = sanitize_config_value(emoji_input);
        assert_eq!(sanitized, emoji_input);

        // Emoji is not safe ASCII
        assert!(!is_safe_ascii(emoji_input));
    }

    #[test]
    fn test_xss_payloads() {
        let xss_payloads = vec![
            r#"<script>alert('XSS')</script>"#,
            r#"<img src=x onerror=alert('XSS')>"#,
            r#""><script>alert(String.fromCharCode(88,83,83))</script>"#,
            r#"<iframe src="javascript:alert('XSS')"></iframe>"#,
        ];

        for payload in xss_payloads {
            let escaped = escape_html_entities(payload);
            // Verify no angle brackets remain unescaped
            assert!(!escaped.contains("<script"));
            assert!(!escaped.contains("<img"));
            assert!(!escaped.contains("<iframe"));
        }
    }

    #[test]
    fn test_sql_injection_patterns() {
        // While our system doesn't use SQL, we should handle these safely
        let sql_patterns = vec![
            "'; DROP TABLE users; --",
            "1' OR '1'='1",
            "admin'--",
            "' OR 1=1--",
        ];

        for pattern in sql_patterns {
            let sanitized = sanitize_config_value(pattern);
            // Should preserve the content but remove control characters
            assert!(!sanitized.contains('\0'));
        }
    }

    #[test]
    fn test_path_traversal_in_values() {
        let traversal_attempts = vec![
            "../../../etc/passwd",
            "..\\..\\..\\windows\\system32",
            "/etc/shadow",
            "C:\\Windows\\System32\\config\\SAM",
        ];

        for attempt in traversal_attempts {
            let sanitized = sanitize_config_value(attempt);
            // Should preserve path characters (sanitization doesn't block paths)
            // Path validation happens in path.rs module
            assert_eq!(sanitized, attempt);
        }
    }
}
