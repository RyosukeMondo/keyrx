//! Integration tests for data validation (VAL-001 through VAL-005)
//!
//! This test suite verifies all validation requirements from WS7:
//! - VAL-001: Profile name validation
//! - VAL-002: Path construction safety
//! - VAL-003: File size limits
//! - VAL-004: Content validation
//! - VAL-005: Input sanitization

use keyrx_daemon::validation::{
    content::{
        scan_for_malicious_patterns, validate_content_size, validate_file_size,
        validate_krx_format, validate_rhai_content, validate_rhai_file, validate_rhai_syntax,
    },
    path::{safe_join, validate_existing_file, validate_path_within_base},
    profile_name::validate_profile_name,
    sanitization::{
        escape_html_entities, is_safe_ascii, remove_control_characters, remove_null_bytes,
        sanitize_config_value, sanitize_profile_name_for_display, validate_json_structure,
    },
    ValidationError, MAX_PROFILE_SIZE,
};
use std::fs;
use tempfile::TempDir;

// ============================================================================
// VAL-001: Profile Name Validation Tests
// ============================================================================

#[test]
fn val_001_valid_profile_names() {
    let max_length_name = "a".repeat(64);
    let valid_names = vec![
        "default",
        "my-profile",
        "my_profile",
        "Profile123",
        "test-config_v2",
        "a",             // minimum length
    ];

    for name in valid_names {
        assert!(
            validate_profile_name(name).is_ok(),
            "Should accept valid name: {}",
            name
        );
    }

    // Test maximum length separately
    assert!(
        validate_profile_name(&max_length_name).is_ok(),
        "Should accept max length name"
    );
}

#[test]
fn val_001_invalid_characters() {
    let invalid_names = vec![
        "my profile", // space
        "profile!",   // special characters
        "profile@home",
        "profile#1",
        "profile$",
        "../etc/passwd", // path traversal
        "test\\file",    // backslash
        "test/file",     // forward slash
    ];

    for name in invalid_names {
        assert!(
            validate_profile_name(name).is_err(),
            "Should reject invalid name: {}",
            name
        );
    }
}

#[test]
fn val_001_windows_reserved_names() {
    let reserved = vec!["con", "CON", "prn", "PRN", "aux", "nul", "com1", "lpt1"];

    for name in reserved {
        assert!(
            validate_profile_name(name).is_err(),
            "Should reject Windows reserved name: {}",
            name
        );
    }
}

#[test]
fn val_001_path_traversal_patterns() {
    let traversal = vec![".", "..", "../profile", "..\\profile"];

    for name in traversal {
        assert!(
            validate_profile_name(name).is_err(),
            "Should reject path traversal: {}",
            name
        );
    }
}

#[test]
fn val_001_null_bytes() {
    assert!(validate_profile_name("test\0").is_err());
    assert!(validate_profile_name("\0test").is_err());
    assert!(validate_profile_name("te\0st").is_err());
}

#[test]
fn val_001_unicode_and_emoji() {
    let unicode_names = vec![
        "プロファイル", // Japanese
        "配置文件",     // Chinese
        "профиль",      // Cyrillic
        "profile😀",    // Emoji
        "café",         // Accented
    ];

    for name in unicode_names {
        assert!(
            validate_profile_name(name).is_err(),
            "Should reject Unicode/emoji: {}",
            name
        );
    }
}

#[test]
fn val_001_length_violations() {
    // Too long (65 characters)
    let too_long = "a".repeat(65);
    assert!(validate_profile_name(&too_long).is_err());

    // Empty
    assert!(validate_profile_name("").is_err());
    assert!(validate_profile_name("   ").is_err());
}

// ============================================================================
// VAL-002: Path Construction Safety Tests
// ============================================================================

#[test]
fn val_002_safe_path_construction() {
    let temp_dir = TempDir::new().unwrap();
    let base = temp_dir.path();
    let profiles_dir = base.join("profiles");
    fs::create_dir(&profiles_dir).unwrap();

    // Create test file
    let test_file = profiles_dir.join("test.rhai");
    fs::write(&test_file, "// test").unwrap();

    // Validate safe path
    let result = validate_path_within_base(&profiles_dir, "test.rhai");
    assert!(result.is_ok());
}

#[test]
fn val_002_block_path_traversal() {
    let temp_dir = TempDir::new().unwrap();
    let base = temp_dir.path();
    let profiles_dir = base.join("profiles");
    fs::create_dir(&profiles_dir).unwrap();

    // Create file outside profiles directory
    let outside_file = base.join("outside.txt");
    fs::write(&outside_file, "outside").unwrap();

    // Attempt traversal
    let result = validate_path_within_base(&profiles_dir, "../outside.txt");
    assert!(result.is_err());
    match result {
        Err(ValidationError::PathTraversal(_)) => (),
        _ => panic!("Expected PathTraversal error"),
    }
}

#[test]
fn val_002_block_absolute_paths() {
    let temp_dir = TempDir::new().unwrap();
    let base = temp_dir.path();

    // Attempt absolute path
    #[cfg(unix)]
    let result = validate_path_within_base(base, "/etc/passwd");
    #[cfg(windows)]
    let result = validate_path_within_base(base, "C:\\Windows\\System32\\config\\SAM");

    assert!(result.is_err());
}

#[test]
fn val_002_safe_join_utility() {
    use std::path::PathBuf;

    let base = PathBuf::from("/home/user/.config/keyrx");
    let component = "profiles";
    let result = safe_join(&base, component);

    // Check that the result ends with the expected component
    assert!(result.ends_with("profiles"));

    // On Unix, verify exact path
    #[cfg(unix)]
    assert_eq!(
        result.to_str().unwrap(),
        "/home/user/.config/keyrx/profiles"
    );
}

#[test]
fn val_002_validate_existing_file() {
    let temp_dir = TempDir::new().unwrap();
    let base = temp_dir.path();

    // Create test file
    let test_file = base.join("test.txt");
    fs::write(&test_file, "test").unwrap();

    // Valid file
    assert!(validate_existing_file(base, "test.txt").is_ok());

    // Non-existent file
    assert!(validate_existing_file(base, "nonexistent.txt").is_err());

    // Directory instead of file
    let subdir = base.join("subdir");
    fs::create_dir(&subdir).unwrap();
    assert!(validate_existing_file(base, "subdir").is_err());
}

// ============================================================================
// VAL-003: File Size Limits Tests
// ============================================================================

#[test]
fn val_003_file_size_within_limit() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("small.txt");

    // Small file (valid)
    fs::write(&file_path, "small content").unwrap();
    assert!(validate_file_size(&file_path, MAX_PROFILE_SIZE).is_ok());
}

#[test]
fn val_003_file_size_exceeds_limit() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("large.txt");

    // Large file exceeding 100KB limit
    let large_content = "x".repeat((MAX_PROFILE_SIZE + 1) as usize);
    fs::write(&file_path, large_content).unwrap();

    let result = validate_file_size(&file_path, MAX_PROFILE_SIZE);
    assert!(result.is_err());
    match result {
        Err(ValidationError::FileSizeTooLarge { actual, limit }) => {
            assert!(actual > limit);
            assert_eq!(limit, MAX_PROFILE_SIZE);
        }
        _ => panic!("Expected FileSizeTooLarge error"),
    }
}

#[test]
fn val_003_content_size_validation() {
    // Small content
    assert!(validate_content_size("small", MAX_PROFILE_SIZE).is_ok());

    // Large content
    let large = "x".repeat((MAX_PROFILE_SIZE + 1) as usize);
    assert!(validate_content_size(&large, MAX_PROFILE_SIZE).is_err());
}

// ============================================================================
// VAL-004: Content Validation Tests
// ============================================================================

#[test]
fn val_004_valid_rhai_syntax() {
    let valid_scripts = vec![
        r#"layer("base", #{ "KEY_A": simple("KEY_B") });"#,
        r#"let x = 42;"#,
        r#"// Comment\nlayer("test", #{});"#,
    ];

    for script in valid_scripts {
        assert!(
            validate_rhai_syntax(script).is_ok(),
            "Valid syntax rejected: {}",
            script
        );
    }
}

#[test]
fn val_004_invalid_rhai_syntax() {
    let invalid_scripts = vec![
        r#"layer("base","#, // incomplete
        r#"let x = ;"#,     // invalid statement
        r#"}}}"#,           // unmatched braces
    ];

    for script in invalid_scripts {
        assert!(
            validate_rhai_syntax(script).is_err(),
            "Invalid syntax accepted: {}",
            script
        );
    }
}

#[test]
fn val_004_detect_malicious_patterns() {
    let malicious = vec![
        r#"eval("code");"#,
        r#"system("rm -rf /");"#,
        r#"open("/etc/passwd");"#,
        r#"write_file("/tmp/bad", data);"#,
        r#"import os;"#,
        r#"EVAL("code");"#, // case insensitive
        r#"System("cmd");"#,
    ];

    for script in malicious {
        assert!(
            scan_for_malicious_patterns(script).is_err(),
            "Malicious pattern not detected: {}",
            script
        );
    }
}

#[test]
fn val_004_safe_rhai_patterns() {
    let safe = vec![
        r#"layer("base", #{ "KEY_A": simple("KEY_B") });"#,
        r#"let key = "KEY_ENTER";"#,
        r#"tap_hold("KEY_CAPSLOCK", 200, simple("KEY_ESC"), layer_toggle("nav"));"#,
    ];

    for script in safe {
        assert!(
            scan_for_malicious_patterns(script).is_ok(),
            "Safe pattern rejected: {}",
            script
        );
    }
}

#[test]
fn val_004_validate_complete_rhai_content() {
    // Valid content
    let valid = r#"layer("base", #{ "KEY_A": simple("KEY_B") });"#;
    assert!(validate_rhai_content(valid).is_ok());

    // Invalid syntax
    let invalid_syntax = r#"layer("base","#;
    assert!(validate_rhai_content(invalid_syntax).is_err());

    // Malicious
    let malicious = r#"system("bad");"#;
    assert!(validate_rhai_content(malicious).is_err());

    // Too large
    let too_large = "x".repeat((MAX_PROFILE_SIZE + 1) as usize);
    assert!(validate_rhai_content(&too_large).is_err());
}

#[test]
fn val_004_validate_rhai_file_integration() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("config.rhai");

    // Valid file
    let valid = r#"layer("base", #{ "KEY_A": simple("KEY_B") });"#;
    fs::write(&file_path, valid).unwrap();
    assert!(validate_rhai_file(&file_path).is_ok());

    // Invalid syntax
    fs::write(&file_path, r#"layer("base","#).unwrap();
    assert!(validate_rhai_file(&file_path).is_err());

    // Malicious
    fs::write(&file_path, r#"eval("bad");"#).unwrap();
    assert!(validate_rhai_file(&file_path).is_err());
}

#[test]
fn val_004_validate_krx_format() {
    let temp_dir = TempDir::new().unwrap();
    let krx_path = temp_dir.path().join("config.krx");

    // Valid magic bytes
    let mut valid = b"KRX\0".to_vec();
    valid.extend_from_slice(&[0; 100]);
    fs::write(&krx_path, valid).unwrap();
    assert!(validate_krx_format(&krx_path).is_ok());

    // Invalid magic
    fs::write(&krx_path, b"INVALID").unwrap();
    assert!(validate_krx_format(&krx_path).is_err());

    // Too short
    fs::write(&krx_path, b"KR").unwrap();
    assert!(validate_krx_format(&krx_path).is_err());
}

// ============================================================================
// VAL-005: Input Sanitization Tests
// ============================================================================

#[test]
fn val_005_escape_html_entities() {
    assert_eq!(escape_html_entities("hello"), "hello");
    assert_eq!(
        escape_html_entities("<script>alert('xss')</script>"),
        "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;&#x2F;script&gt;"
    );
    assert_eq!(
        escape_html_entities("A & B < C > D"),
        "A &amp; B &lt; C &gt; D"
    );
}

#[test]
fn val_005_remove_control_characters() {
    // Remove null and control
    assert_eq!(
        remove_control_characters("Hello\x00World\x01Test"),
        "HelloWorldTest"
    );

    // Keep newlines, tabs, carriage returns
    assert_eq!(
        remove_control_characters("Line1\nLine2\r\nTab\there"),
        "Line1\nLine2\r\nTab\there"
    );
}

#[test]
fn val_005_remove_null_bytes() {
    assert_eq!(remove_null_bytes("hello"), "hello");
    assert_eq!(remove_null_bytes("hello\0world"), "helloworld");
    assert_eq!(remove_null_bytes("\0test\0"), "test");
}

#[test]
fn val_005_sanitize_profile_name() {
    // Normal
    assert_eq!(
        sanitize_profile_name_for_display("my-profile"),
        "my-profile"
    );

    // HTML entities
    assert_eq!(
        sanitize_profile_name_for_display("<script>"),
        "&lt;script&gt;"
    );

    // Control characters
    assert_eq!(
        sanitize_profile_name_for_display("test\x00\x01name"),
        "testname"
    );

    // Combined
    assert_eq!(
        sanitize_profile_name_for_display("<test\0>&"),
        "&lt;test&gt;&amp;"
    );
}

#[test]
fn val_005_validate_json_structure() {
    // Valid JSON
    assert!(validate_json_structure(r#"{"key": "value"}"#).is_ok());
    assert!(validate_json_structure(r#"[1, 2, 3]"#).is_ok());
    assert!(validate_json_structure(r#"null"#).is_ok());

    // Invalid JSON
    assert!(validate_json_structure(r#"{"key": "value""#).is_err());
    assert!(validate_json_structure(r#"{key: value}"#).is_err());
    assert!(validate_json_structure(r#"[1, 2, 3,]"#).is_err());
}

#[test]
fn val_005_sanitize_config_value() {
    assert_eq!(sanitize_config_value("normal value"), "normal value");
    assert_eq!(sanitize_config_value("  trimmed  "), "trimmed");
    assert_eq!(sanitize_config_value("test\x00\x01value"), "testvalue");
}

#[test]
fn val_005_is_safe_ascii() {
    // Safe
    assert!(is_safe_ascii("hello world"));
    assert!(is_safe_ascii("123-456_789"));
    assert!(is_safe_ascii("Line1\nLine2\tTab"));

    // Unsafe
    assert!(!is_safe_ascii("hello\0world"));
    assert!(!is_safe_ascii("test\x01value"));
    assert!(!is_safe_ascii("hello\x1Bworld"));
}

#[test]
fn val_005_xss_payloads() {
    let xss_payloads = vec![
        r#"<script>alert('XSS')</script>"#,
        r#"<img src=x onerror=alert('XSS')>"#,
        r#""><script>alert(String.fromCharCode(88,83,83))</script>"#,
        r#"<iframe src="javascript:alert('XSS')"></iframe>"#,
    ];

    for payload in xss_payloads {
        let escaped = escape_html_entities(payload);
        assert!(!escaped.contains("<script"));
        assert!(!escaped.contains("<img"));
        assert!(!escaped.contains("<iframe"));
    }
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn edge_case_empty_strings() {
    assert!(validate_profile_name("").is_err());
    assert_eq!(remove_control_characters(""), "");
    assert_eq!(escape_html_entities(""), "");
}

#[test]
fn edge_case_whitespace_only() {
    assert!(validate_profile_name("   ").is_err());
    assert!(validate_profile_name("\t\n").is_err());
}

#[test]
fn edge_case_max_lengths() {
    // Profile name at max length (64 chars)
    let max_name = "a".repeat(64);
    assert!(validate_profile_name(&max_name).is_ok());

    // One char over max
    let over_max = "a".repeat(65);
    assert!(validate_profile_name(&over_max).is_err());
}

#[test]
fn edge_case_unicode_normalization() {
    // Different Unicode representations of same character
    let nfc = "café"; // NFC normalized
    let nfd = "café"; // NFD normalized (e + combining accent)

    // Both should be rejected (non-ASCII)
    assert!(validate_profile_name(nfc).is_err());
    assert!(validate_profile_name(nfd).is_err());
}

#[test]
fn edge_case_mixed_line_endings() {
    let mixed = "line1\nline2\r\nline3\r";
    let sanitized = remove_control_characters(mixed);
    assert_eq!(sanitized, mixed); // Line endings preserved
}

#[test]
fn edge_case_nested_html() {
    let nested = r#"<div><span><script>alert('xss')</script></span></div>"#;
    let escaped = escape_html_entities(nested);
    assert!(!escaped.contains("<"));
    assert!(!escaped.contains(">"));
}
