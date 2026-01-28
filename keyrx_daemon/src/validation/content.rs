//! Configuration content validation (VAL-003, VAL-004)
//!
//! Validates:
//! - File sizes (max 100KB per profile)
//! - Rhai syntax (before saving)
//! - Binary .krx format (on load)
//! - Malicious code patterns (system calls, file I/O)

use super::{ValidationError, ValidationResult, MAX_PROFILE_SIZE};
use std::path::Path;

/// Malicious Rhai patterns that are not allowed in user configurations
const MALICIOUS_PATTERNS: &[&str] = &[
    "eval(",        // Dynamic code execution
    "system(",      // System command execution
    "exec(",        // Process execution
    "spawn(",       // Process spawning
    "open(",        // File operations
    "write(",       // File write operations
    "delete(",      // File deletion
    "read_file(",   // File reading
    "write_file(",  // File writing
    "delete_file(", // File deletion
    "import ",      // Module imports (could load malicious code)
    "include(",     // File inclusion
    "require(",     // Module loading
];

/// Validates file size constraints.
///
/// # Arguments
///
/// * `path` - Path to the file to check
/// * `max_size` - Maximum allowed size in bytes
///
/// # Returns
///
/// * `Ok(u64)` - The file size if valid
/// * `Err(ValidationError)` - If file is too large or inaccessible
pub fn validate_file_size<P: AsRef<Path>>(path: P, max_size: u64) -> ValidationResult<u64> {
    let path_ref = path.as_ref();

    let metadata = std::fs::metadata(path_ref)?;
    let size = metadata.len();

    if size > max_size {
        return Err(ValidationError::FileSizeTooLarge {
            actual: size,
            limit: max_size,
        });
    }

    Ok(size)
}

/// Validates the content size of a string.
///
/// # Arguments
///
/// * `content` - The content string to validate
/// * `max_size` - Maximum allowed size in bytes
///
/// # Returns
///
/// * `Ok(())` if within limits
/// * `Err(ValidationError)` if too large
pub fn validate_content_size(content: &str, max_size: u64) -> ValidationResult<()> {
    let size = content.len() as u64;

    if size > max_size {
        return Err(ValidationError::FileSizeTooLarge {
            actual: size,
            limit: max_size,
        });
    }

    Ok(())
}

/// Validates Rhai syntax without executing the code.
///
/// This performs a basic syntax check by attempting to parse the Rhai script.
///
/// # Arguments
///
/// * `content` - The Rhai script content to validate
///
/// # Returns
///
/// * `Ok(())` if syntax is valid
/// * `Err(ValidationError)` with parsing error details
pub fn validate_rhai_syntax(content: &str) -> ValidationResult<()> {
    // Create a Rhai engine for syntax checking
    let engine = rhai::Engine::new();

    // Attempt to compile (parse) the script
    match engine.compile(content) {
        Ok(_) => Ok(()),
        Err(e) => Err(ValidationError::InvalidContent(format!(
            "Rhai syntax error: {}",
            e
        ))),
    }
}

/// Scans for malicious code patterns in Rhai scripts.
///
/// This is a defense-in-depth measure to catch potentially dangerous
/// function calls that shouldn't be in user configuration scripts.
///
/// # Arguments
///
/// * `content` - The Rhai script content to scan
///
/// # Returns
///
/// * `Ok(())` if no malicious patterns found
/// * `Err(ValidationError)` if dangerous patterns detected
pub fn scan_for_malicious_patterns(content: &str) -> ValidationResult<()> {
    let content_lower = content.to_lowercase();

    for pattern in MALICIOUS_PATTERNS {
        if content_lower.contains(pattern) {
            return Err(ValidationError::MaliciousPattern(format!(
                "Detected potentially malicious pattern: {}",
                pattern
            )));
        }
    }

    Ok(())
}

/// Validates a complete Rhai configuration file.
///
/// Performs all validation checks:
/// 1. File size limits
/// 2. Syntax validation
/// 3. Malicious pattern detection
///
/// # Arguments
///
/// * `path` - Path to the .rhai file
///
/// # Returns
///
/// * `Ok(String)` - The validated content
/// * `Err(ValidationError)` if any check fails
pub fn validate_rhai_file<P: AsRef<Path>>(path: P) -> ValidationResult<String> {
    let path_ref = path.as_ref();

    // Check file size
    validate_file_size(path_ref, MAX_PROFILE_SIZE)?;

    // Read content
    let content = std::fs::read_to_string(path_ref)?;

    // Validate content size (redundant check, but explicit)
    validate_content_size(&content, MAX_PROFILE_SIZE)?;

    // Validate syntax
    validate_rhai_syntax(&content)?;

    // Scan for malicious patterns
    scan_for_malicious_patterns(&content)?;

    Ok(content)
}

/// Validates a Rhai configuration string before saving.
///
/// # Arguments
///
/// * `content` - The configuration content to validate
///
/// # Returns
///
/// * `Ok(())` if all checks pass
/// * `Err(ValidationError)` otherwise
pub fn validate_rhai_content(content: &str) -> ValidationResult<()> {
    // Check content size
    validate_content_size(content, MAX_PROFILE_SIZE)?;

    // Validate syntax
    validate_rhai_syntax(content)?;

    // Scan for malicious patterns
    scan_for_malicious_patterns(content)?;

    Ok(())
}

/// Validates a .krx binary configuration file format.
///
/// Checks:
/// - File exists and is readable
/// - File size is within limits
/// - Binary format magic bytes are correct
///
/// # Arguments
///
/// * `path` - Path to the .krx file
///
/// # Returns
///
/// * `Ok(())` if format is valid
/// * `Err(ValidationError)` otherwise
pub fn validate_krx_format<P: AsRef<Path>>(path: P) -> ValidationResult<()> {
    let path_ref = path.as_ref();

    // Check file size
    validate_file_size(path_ref, MAX_PROFILE_SIZE)?;

    // Read first 8 bytes for magic check
    let bytes = std::fs::read(path_ref)?;

    if bytes.len() < 8 {
        return Err(ValidationError::InvalidBinaryFormat(
            "File too short to be valid .krx format".to_string(),
        ));
    }

    // Check magic bytes (KRX format: "KRX\0" = [0x4B, 0x52, 0x58, 0x00])
    // Note: Adjust these bytes based on actual keyrx_compiler format
    const MAGIC: &[u8] = b"KRX\0";
    if !bytes.starts_with(MAGIC) {
        return Err(ValidationError::InvalidBinaryFormat(
            "Invalid magic bytes - not a valid .krx file".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_validate_file_size() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // Small file (valid)
        fs::write(&file_path, "small content").unwrap();
        let result = validate_file_size(&file_path, MAX_PROFILE_SIZE);
        assert!(result.is_ok());

        // Large file (invalid) - create a file larger than limit
        let large_content = "x".repeat((MAX_PROFILE_SIZE + 1) as usize);
        fs::write(&file_path, large_content).unwrap();
        let result = validate_file_size(&file_path, MAX_PROFILE_SIZE);
        assert!(result.is_err());
        match result {
            Err(ValidationError::FileSizeTooLarge { actual, limit }) => {
                assert!(actual > limit);
            }
            _ => panic!("Expected FileSizeTooLarge error"),
        }
    }

    #[test]
    fn test_validate_content_size() {
        let small_content = "small string";
        assert!(validate_content_size(small_content, MAX_PROFILE_SIZE).is_ok());

        let large_content = "x".repeat((MAX_PROFILE_SIZE + 1) as usize);
        assert!(validate_content_size(&large_content, MAX_PROFILE_SIZE).is_err());
    }

    #[test]
    fn test_validate_rhai_syntax_valid() {
        let valid_scripts = vec![
            r#"layer("base", #{ "KEY_A": simple("KEY_B") });"#,
            r#"let x = 42; x + 1;"#,
            r#"// Comment\nlayer("test", #{});"#,
        ];

        for script in valid_scripts {
            let result = validate_rhai_syntax(script);
            assert!(result.is_ok(), "Failed for: {}", script);
        }
    }

    #[test]
    fn test_validate_rhai_syntax_invalid() {
        let invalid_scripts = vec![
            r#"layer("base",  // missing closing brace"#,
            r#"let x = ;"#, // incomplete statement
            r#"}}}"#,       // unmatched braces
        ];

        for script in invalid_scripts {
            let result = validate_rhai_syntax(script);
            assert!(result.is_err(), "Should fail for: {}", script);
        }
    }

    #[test]
    fn test_scan_for_malicious_patterns() {
        // Safe content
        let safe_content = r#"layer("base", #{ "KEY_A": simple("KEY_B") });"#;
        assert!(scan_for_malicious_patterns(safe_content).is_ok());

        // Malicious patterns
        let malicious_examples = vec![
            r#"eval("malicious code");"#,
            r#"system("rm -rf /");"#,
            r#"open("/etc/passwd");"#,
            r#"write_file("/tmp/bad", data);"#,
            r#"import os; // malicious"#,
        ];

        for script in malicious_examples {
            let result = scan_for_malicious_patterns(script);
            assert!(
                result.is_err(),
                "Should detect malicious pattern in: {}",
                script
            );
            match result {
                Err(ValidationError::MaliciousPattern(_)) => (),
                _ => panic!("Expected MaliciousPattern error"),
            }
        }
    }

    #[test]
    fn test_validate_rhai_content() {
        // Valid content
        let valid = r#"layer("base", #{ "KEY_A": simple("KEY_B") });"#;
        assert!(validate_rhai_content(valid).is_ok());

        // Invalid syntax
        let invalid_syntax = r#"layer("base", {"#;
        assert!(validate_rhai_content(invalid_syntax).is_err());

        // Malicious pattern
        let malicious = r#"eval("bad code");"#;
        assert!(validate_rhai_content(malicious).is_err());

        // Too large
        let too_large = "x".repeat((MAX_PROFILE_SIZE + 1) as usize);
        assert!(validate_rhai_content(&too_large).is_err());
    }

    #[test]
    fn test_validate_rhai_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rhai");

        // Valid file
        let valid_content = r#"layer("base", #{ "KEY_A": simple("KEY_B") });"#;
        fs::write(&file_path, valid_content).unwrap();
        let result = validate_rhai_file(&file_path);
        assert!(result.is_ok());

        // Invalid syntax
        fs::write(&file_path, r#"layer("base","#).unwrap();
        let result = validate_rhai_file(&file_path);
        assert!(result.is_err());

        // Malicious content
        fs::write(&file_path, r#"system("rm -rf /");"#).unwrap();
        let result = validate_rhai_file(&file_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_krx_format() {
        let temp_dir = TempDir::new().unwrap();
        let krx_path = temp_dir.path().join("test.krx");

        // Valid magic bytes
        let mut valid_content = b"KRX\0".to_vec();
        valid_content.extend_from_slice(&[0; 100]); // Padding
        fs::write(&krx_path, valid_content).unwrap();
        assert!(validate_krx_format(&krx_path).is_ok());

        // Invalid magic bytes
        fs::write(&krx_path, b"INVALID").unwrap();
        assert!(validate_krx_format(&krx_path).is_err());

        // File too short
        fs::write(&krx_path, b"KR").unwrap();
        assert!(validate_krx_format(&krx_path).is_err());
    }

    #[test]
    fn test_case_insensitive_malicious_detection() {
        // Test case variations
        let variations = vec![
            r#"EVAL("code");"#,
            r#"Eval("code");"#,
            r#"EvAl("code");"#,
            r#"SYSTEM("cmd");"#,
            r#"System("cmd");"#,
        ];

        for script in variations {
            let result = scan_for_malicious_patterns(script);
            assert!(result.is_err(), "Should detect case variation: {}", script);
        }
    }
}
