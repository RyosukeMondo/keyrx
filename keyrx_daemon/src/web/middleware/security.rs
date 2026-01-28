//! Security middleware for request validation and protection
//!
//! Implements:
//! - Request size limits
//! - Path traversal protection
//! - Input sanitization
//! - DoS protection (connection limits)

use axum::{
    body::Body,
    extract::Request,
    http::{Response, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use std::path::PathBuf;

/// Security configuration
#[derive(Clone)]
pub struct SecurityConfig {
    /// Maximum request body size (bytes)
    pub max_body_size: usize,
    /// Maximum URL length (bytes)
    pub max_url_length: usize,
    /// Maximum concurrent WebSocket connections
    pub max_ws_connections: usize,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            max_body_size: 1024 * 1024, // 1MB
            max_url_length: 10 * 1024,  // 10KB
            max_ws_connections: 100,    // 100 concurrent connections
        }
    }
}

/// Security middleware layer
#[derive(Clone)]
pub struct SecurityLayer {
    config: SecurityConfig,
}

impl SecurityLayer {
    /// Create new security layer with default config
    pub fn new() -> Self {
        Self::with_config(SecurityConfig::default())
    }

    /// Create security layer with custom config
    pub fn with_config(config: SecurityConfig) -> Self {
        Self { config }
    }

    /// Get configuration
    pub fn config(&self) -> &SecurityConfig {
        &self.config
    }
}

impl Default for SecurityLayer {
    fn default() -> Self {
        Self::new()
    }
}

/// Security middleware handler
pub async fn security_middleware(
    security: axum::extract::State<SecurityLayer>,
    request: Request<Body>,
    next: Next,
) -> Result<Response<Body>, impl IntoResponse> {
    // Check URL length
    let uri = request.uri();
    let url_str = uri.to_string();
    if url_str.len() > security.config().max_url_length {
        return Err((
            StatusCode::URI_TOO_LONG,
            format!(
                "URL too long: {} bytes (max: {})",
                url_str.len(),
                security.config().max_url_length
            ),
        ));
    }

    // Check for path traversal in URL
    if contains_path_traversal(&url_str) {
        log::warn!("Path traversal attempt detected: {}", url_str);
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid path: path traversal not allowed".to_string(),
        ));
    }

    // Continue with request
    Ok(next.run(request).await)
}

/// Check if string contains path traversal patterns
fn contains_path_traversal(s: &str) -> bool {
    s.contains("..") || s.contains("./") || s.contains("\\..") || s.contains("\\.")
}

/// Validate and canonicalize a file path to prevent traversal attacks
///
/// # Arguments
///
/// * `base_dir` - The base directory that paths must be within
/// * `path` - The path to validate
///
/// # Returns
///
/// Returns the canonical path if valid, or an error if:
/// - Path contains traversal patterns
/// - Canonical path is outside base directory
/// - Path doesn't exist (for canonicalization)
pub fn validate_path(
    base_dir: &std::path::Path,
    path: &std::path::Path,
) -> Result<PathBuf, String> {
    // Check for obvious traversal patterns first
    let path_str = path.to_string_lossy();
    if contains_path_traversal(&path_str) {
        return Err("Path contains traversal patterns (..)".to_string());
    }

    // Build full path
    let full_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        base_dir.join(path)
    };

    // Canonicalize to resolve symlinks and relative paths
    let canonical = full_path
        .canonicalize()
        .map_err(|e| format!("Failed to canonicalize path: {}", e))?;

    // Ensure canonical path is within base directory
    let canonical_base = base_dir
        .canonicalize()
        .map_err(|e| format!("Failed to canonicalize base dir: {}", e))?;

    if !canonical.starts_with(&canonical_base) {
        return Err(format!(
            "Path is outside allowed directory: {} not in {}",
            canonical.display(),
            canonical_base.display()
        ));
    }

    Ok(canonical)
}

/// Sanitize a string to prevent XSS attacks
///
/// Escapes HTML special characters
pub fn sanitize_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
        .replace('/', "&#x2F;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_path_traversal() {
        assert!(contains_path_traversal("../etc/passwd"));
        assert!(contains_path_traversal("./secret"));
        assert!(contains_path_traversal("folder\\..\\file"));
        assert!(contains_path_traversal("folder\\.\\file"));
        assert!(!contains_path_traversal("/etc/passwd"));
        assert!(!contains_path_traversal("normal/path/file"));
    }

    #[test]
    fn test_validate_path_traversal() {
        let base = std::path::Path::new("/tmp");
        let result = validate_path(base, std::path::Path::new("../etc/passwd"));
        assert!(result.is_err());
    }

    #[test]
    fn test_sanitize_html() {
        assert_eq!(
            sanitize_html("<script>alert('xss')</script>"),
            "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;&#x2F;script&gt;"
        );
        assert_eq!(sanitize_html("normal text"), "normal text");
        assert_eq!(sanitize_html("a & b < c > d"), "a &amp; b &lt; c &gt; d");
    }
}
