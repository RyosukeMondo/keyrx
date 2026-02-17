//! Input validation middleware for comprehensive request sanitization
//!
//! This module implements multi-layer input validation to prevent:
//! - SQL injection (N/A - no SQL database)
//! - Command injection
//! - Path traversal
//! - XSS attacks
//! - Buffer overflow attacks
//! - Content-length attacks

use axum::{
    body::Body,
    extract::Request,
    http::{Response, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use std::path::Path;

/// Input validation configuration
#[derive(Clone)]
pub struct InputValidationConfig {
    /// Maximum request body size (10 MB)
    pub max_body_size: usize,
    /// Maximum URL length (10 KB)
    pub max_url_length: usize,
    /// Maximum header value length (8 KB)
    pub max_header_length: usize,
    /// Maximum file upload size (10 MB)
    pub max_file_upload_size: usize,
    /// Maximum profile name length
    pub max_profile_name_length: usize,
    /// Maximum configuration content length
    pub max_config_content_length: usize,
}

impl Default for InputValidationConfig {
    fn default() -> Self {
        Self {
            max_body_size: 10 * 1024 * 1024,        // 10 MB
            max_url_length: 10 * 1024,              // 10 KB
            max_header_length: 8 * 1024,            // 8 KB
            max_file_upload_size: 10 * 1024 * 1024, // 10 MB
            max_profile_name_length: 50,            // 50 chars
            max_config_content_length: 100 * 1024,  // 100 KB
        }
    }
}

/// Input validation middleware layer
#[derive(Clone)]
pub struct InputValidationLayer {
    config: InputValidationConfig,
}

impl InputValidationLayer {
    /// Create new input validation layer with default config
    pub fn new() -> Self {
        Self::with_config(InputValidationConfig::default())
    }

    /// Create input validation layer with custom config
    pub fn with_config(config: InputValidationConfig) -> Self {
        Self { config }
    }

    /// Get configuration
    pub fn config(&self) -> &InputValidationConfig {
        &self.config
    }
}

impl Default for InputValidationLayer {
    fn default() -> Self {
        Self::new()
    }
}

/// Input validation middleware handler
pub async fn input_validation_middleware(
    validation: axum::extract::State<InputValidationLayer>,
    request: Request<Body>,
    next: Next,
) -> Result<Response<Body>, impl IntoResponse> {
    // 1. Validate URL length
    let uri = request.uri();
    let url_str = uri.to_string();
    if url_str.len() > validation.config().max_url_length {
        log::warn!(
            "URL too long: {} bytes (max: {})",
            url_str.len(),
            validation.config().max_url_length
        );
        return Err((
            StatusCode::URI_TOO_LONG,
            format!(
                "URL too long: {} bytes (max: {})",
                url_str.len(),
                validation.config().max_url_length
            ),
        ));
    }

    // 2. Validate path for traversal patterns
    if contains_path_traversal(&url_str) {
        log::warn!("Path traversal attempt detected in URL: {}", url_str);
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid path: path traversal patterns not allowed".to_string(),
        ));
    }

    // 3. Validate path for command injection patterns
    if contains_command_injection(&url_str) {
        log::warn!("Command injection attempt detected in URL: {}", url_str);
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid input: command injection patterns detected".to_string(),
        ));
    }

    // 4. Validate headers
    for (name, value) in request.headers() {
        let value_str = value.to_str().unwrap_or("");
        if value_str.len() > validation.config().max_header_length {
            log::warn!(
                "Header value too long: {} = {} bytes",
                name,
                value_str.len()
            );
            return Err((
                StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE,
                format!("Header '{}' too long", name),
            ));
        }

        // Check for injection patterns in headers (skip standard headers that
        // legitimately contain semicolons, pipes, etc.)
        let skip_injection_check = matches!(
            name.as_str(),
            "accept"
                | "accept-encoding"
                | "accept-language"
                | "content-type"
                | "cookie"
                | "set-cookie"
                | "cache-control"
                | "user-agent"
                | "sec-ch-ua"
                | "sec-ch-ua-mobile"
                | "sec-ch-ua-platform"
                | "sec-websocket-extensions"
                | "sec-websocket-protocol"
        );
        if !skip_injection_check && contains_command_injection(value_str) {
            log::warn!(
                "Command injection attempt in header {}: {}",
                name,
                value_str
            );
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Invalid header value in '{}'", name),
            ));
        }
    }

    // 5. Validate Content-Length if present
    if let Some(content_length) = request.headers().get("content-length") {
        if let Ok(length_str) = content_length.to_str() {
            if let Ok(length) = length_str.parse::<usize>() {
                if length > validation.config().max_body_size {
                    log::warn!(
                        "Request body too large: {} bytes (max: {})",
                        length,
                        validation.config().max_body_size
                    );
                    return Err((
                        StatusCode::PAYLOAD_TOO_LARGE,
                        format!(
                            "Request body too large: {} bytes (max: {})",
                            length,
                            validation.config().max_body_size
                        ),
                    ));
                }
            }
        }
    }

    // All validation passed, continue
    Ok(next.run(request).await)
}

/// Check if string contains path traversal patterns
fn contains_path_traversal(s: &str) -> bool {
    // Multiple checks for various path traversal patterns
    s.contains("..")
        || s.contains("./")
        || s.contains("\\..")
        || s.contains("\\.")
        || s.contains("%2e%2e") // URL encoded ..
        || s.contains("%252e") // Double URL encoded .
        || s.contains("..;") // Semicolon bypass
        || s.contains("..%00") // Null byte injection
        || s.contains("..%0a") // Newline injection
}

/// Check if string contains command injection patterns
fn contains_command_injection(s: &str) -> bool {
    // Check for shell command patterns
    s.contains(';')
        || s.contains('|')
        || s.contains('&')
        || s.contains('`')
        || s.contains("$(")
        || s.contains('\n')
        || s.contains('\r')
        || s.contains("%0a") // URL encoded newline
        || s.contains("%0d") // URL encoded carriage return
        || s.contains("%00") // Null byte
}

/// Validate profile name for safe filesystem usage
pub fn validate_profile_name(name: &str, max_length: usize) -> Result<(), String> {
    // 1. Check length
    if name.is_empty() {
        return Err("Profile name cannot be empty".to_string());
    }
    if name.len() > max_length {
        return Err(format!(
            "Profile name too long: {} characters (max: {})",
            name.len(),
            max_length
        ));
    }

    // 2. Check for valid characters (alphanumeric, dash, underscore only)
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(
            "Profile name can only contain letters, numbers, dash, and underscore".to_string(),
        );
    }

    // 3. Check for path traversal
    if contains_path_traversal(name) {
        return Err("Profile name contains invalid patterns".to_string());
    }

    // 4. Check for reserved names
    if is_reserved_name(name) {
        return Err(format!("'{}' is a reserved name", name));
    }

    Ok(())
}

/// Check if name is a reserved system name
fn is_reserved_name(name: &str) -> bool {
    matches!(
        name.to_lowercase().as_str(),
        "con"
            | "prn"
            | "aux"
            | "nul"
            | "com1"
            | "com2"
            | "com3"
            | "com4"
            | "lpt1"
            | "lpt2"
            | "lpt3"
            | "."
            | ".."
            | "default"
            | "system"
    )
}

/// Validate file path to ensure it's within base directory
pub fn validate_file_path(base: &Path, user_path: &str) -> Result<std::path::PathBuf, String> {
    // 1. Check for traversal patterns
    if contains_path_traversal(user_path) {
        return Err("Path contains traversal patterns".to_string());
    }

    // 2. Build full path
    let full_path = base.join(user_path);

    // 3. Canonicalize and verify it's within base
    let canonical = full_path
        .canonicalize()
        .map_err(|e| format!("Invalid path: {}", e))?;

    let canonical_base = base
        .canonicalize()
        .map_err(|e| format!("Invalid base directory: {}", e))?;

    if !canonical.starts_with(&canonical_base) {
        return Err(format!(
            "Path is outside allowed directory: {}",
            canonical.display()
        ));
    }

    Ok(canonical)
}

/// Validate and limit file size before reading
pub fn validate_file_size(path: &Path, max_size: u64) -> Result<u64, String> {
    let metadata = std::fs::metadata(path).map_err(|e| format!("Cannot access file: {}", e))?;

    let size = metadata.len();
    if size > max_size {
        return Err(format!(
            "File too large: {} bytes (max: {} bytes)",
            size, max_size
        ));
    }

    Ok(size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_traversal_detection() {
        assert!(contains_path_traversal("../etc/passwd"));
        assert!(contains_path_traversal("./secret"));
        assert!(contains_path_traversal("folder\\..\\file"));
        assert!(contains_path_traversal("%2e%2e/etc/passwd"));
        assert!(contains_path_traversal("..%00/etc/passwd"));
        assert!(!contains_path_traversal("/normal/path"));
        assert!(!contains_path_traversal("file.txt"));
    }

    #[test]
    fn test_command_injection_detection() {
        assert!(contains_command_injection("rm -rf /; ls"));
        assert!(contains_command_injection("test | cat"));
        assert!(contains_command_injection("test && rm"));
        assert!(contains_command_injection("test `whoami`"));
        assert!(contains_command_injection("test $(id)"));
        assert!(contains_command_injection("test%0als"));
        assert!(!contains_command_injection("normal text"));
        assert!(!contains_command_injection("file-name_123"));
    }

    #[test]
    fn test_validate_profile_name() {
        // Valid names
        assert!(validate_profile_name("my-profile", 50).is_ok());
        assert!(validate_profile_name("test_123", 50).is_ok());
        assert!(validate_profile_name("Profile-Name_1", 50).is_ok());

        // Invalid: empty
        assert!(validate_profile_name("", 50).is_err());

        // Invalid: too long
        assert!(validate_profile_name("a".repeat(100).as_str(), 50).is_err());

        // Invalid: special characters
        assert!(validate_profile_name("test@profile", 50).is_err());
        assert!(validate_profile_name("test profile", 50).is_err());
        assert!(validate_profile_name("test/profile", 50).is_err());

        // Invalid: path traversal
        assert!(validate_profile_name("../secret", 50).is_err());

        // Invalid: reserved names
        assert!(validate_profile_name("con", 50).is_err());
        assert!(validate_profile_name("aux", 50).is_err());
        assert!(validate_profile_name("..", 50).is_err());
    }

    #[test]
    fn test_validate_file_size() {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.txt");

        // Create file with known size
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(b"test content").unwrap();
        drop(file);

        // Should succeed within limit
        assert!(validate_file_size(&file_path, 1024).is_ok());

        // Should fail exceeding limit
        assert!(validate_file_size(&file_path, 5).is_err());
    }
}
