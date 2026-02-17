//! Security headers middleware for HTTP response hardening
//!
//! Adds critical security headers to all responses:
//! - Content-Security-Policy (XSS and injection protection)
//! - X-Frame-Options (Clickjacking protection)
//! - X-Content-Type-Options (MIME sniffing protection)
//! - Strict-Transport-Security (HTTPS enforcement)
//! - Referrer-Policy (Privacy protection)
//! - X-Permitted-Cross-Domain-Policies (Flash/PDF protection)
//! - Permissions-Policy (Feature control)

use axum::{
    extract::Request,
    http::{header, HeaderValue},
    middleware::Next,
    response::Response,
};

/// Security headers configuration
#[derive(Clone, Debug)]
pub struct SecurityHeadersConfig {
    /// Enable Strict-Transport-Security (HSTS)
    pub enable_hsts: bool,
    /// HSTS max-age in seconds
    pub hsts_max_age: u32,
    /// Enable HSTS subdomains
    pub hsts_include_subdomains: bool,
    /// Content-Security-Policy header value
    pub csp_policy: String,
}

impl Default for SecurityHeadersConfig {
    fn default() -> Self {
        Self {
            enable_hsts: true,
            hsts_max_age: 63072000, // 2 years
            hsts_include_subdomains: true,
            // Restrictive CSP: no inline scripts, same-origin only
            csp_policy: "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; img-src 'self' data:; font-src 'self'; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'".to_string(),
        }
    }
}

impl SecurityHeadersConfig {
    /// Create config for development (relaxed HSTS)
    pub fn dev() -> Self {
        Self {
            enable_hsts: false, // Don't enforce HSTS in dev
            hsts_max_age: 3600, // 1 hour for testing
            hsts_include_subdomains: false,
            csp_policy: "default-src 'self'; script-src 'self' 'unsafe-inline' 'wasm-unsafe-eval' https://cdn.jsdelivr.net; style-src 'self' 'unsafe-inline' https://cdn.jsdelivr.net; font-src 'self' https://cdn.jsdelivr.net; worker-src 'self' blob:; img-src 'self' data:; connect-src 'self' ws: wss:; frame-ancestors 'none'; base-uri 'self'; form-action 'self'".to_string(),
        }
    }

    /// Create config for production (strict enforcement)
    pub fn production() -> Self {
        Self::default()
    }

    /// Build HSTS header value
    fn build_hsts_header(&self) -> String {
        let mut hsts = format!("max-age={}", self.hsts_max_age);
        if self.hsts_include_subdomains {
            hsts.push_str("; includeSubDomains");
        }
        hsts.push_str("; preload");
        hsts
    }
}

/// Middleware layer for security headers
#[derive(Clone)]
pub struct SecurityHeadersLayer {
    config: SecurityHeadersConfig,
}

impl SecurityHeadersLayer {
    /// Create new security headers layer with default config
    pub fn new() -> Self {
        Self::with_config(SecurityHeadersConfig::default())
    }

    /// Create with development configuration
    pub fn dev() -> Self {
        Self::with_config(SecurityHeadersConfig::dev())
    }

    /// Create with production configuration
    pub fn production() -> Self {
        Self::with_config(SecurityHeadersConfig::production())
    }

    /// Create security headers layer with custom config
    pub fn with_config(config: SecurityHeadersConfig) -> Self {
        Self { config }
    }

    /// Get configuration
    pub fn config(&self) -> &SecurityHeadersConfig {
        &self.config
    }
}

impl Default for SecurityHeadersLayer {
    fn default() -> Self {
        Self::new()
    }
}

/// Security headers middleware handler
pub async fn security_headers_middleware(
    headers_layer: axum::extract::State<SecurityHeadersLayer>,
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    let config = headers_layer.config();

    // Content-Security-Policy: Prevent XSS and injection attacks
    if let Ok(csp_value) = HeaderValue::from_str(&config.csp_policy) {
        headers.insert(
            header::HeaderName::from_static("content-security-policy"),
            csp_value,
        );
    }

    // X-Content-Type-Options: Prevent MIME sniffing
    headers.insert(
        header::HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );

    // X-Frame-Options: Prevent clickjacking
    headers.insert(
        header::HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );

    // Referrer-Policy: Control referrer information
    headers.insert(
        header::HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // X-Permitted-Cross-Domain-Policies: Restrict Adobe Flash/PDF access
    headers.insert(
        header::HeaderName::from_static("x-permitted-cross-domain-policies"),
        HeaderValue::from_static("none"),
    );

    // Permissions-Policy: Control browser features
    // Block: geolocation, microphone, camera, magnetometer, gyroscope, accelerometer, usb
    headers.insert(
        header::HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static("geolocation=(), microphone=(), camera=(), magnetometer=(), gyroscope=(), accelerometer=(), usb=(), payment=()"),
    );

    // Strict-Transport-Security: Enforce HTTPS (when enabled)
    if config.enable_hsts {
        if let Ok(hsts_value) = HeaderValue::from_str(&config.build_hsts_header()) {
            headers.insert(
                header::HeaderName::from_static("strict-transport-security"),
                hsts_value,
            );
        }
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SecurityHeadersConfig::default();
        assert!(config.enable_hsts);
        assert_eq!(config.hsts_max_age, 63072000);
        assert!(config.hsts_include_subdomains);
        assert!(config.csp_policy.contains("default-src 'self'"));
    }

    #[test]
    fn test_dev_config() {
        let config = SecurityHeadersConfig::dev();
        assert!(!config.enable_hsts);
        assert!(config.csp_policy.contains("'unsafe-inline'"));
    }

    #[test]
    fn test_production_config() {
        let config = SecurityHeadersConfig::production();
        assert!(config.enable_hsts);
        assert!(!config.csp_policy.contains("'unsafe-inline'"));
    }

    #[test]
    fn test_build_hsts_header() {
        let config = SecurityHeadersConfig::default();
        let hsts = config.build_hsts_header();
        assert!(hsts.contains("max-age=63072000"));
        assert!(hsts.contains("includeSubDomains"));
        assert!(hsts.contains("preload"));
    }

    #[test]
    fn test_hsts_header_without_subdomains() {
        let config = SecurityHeadersConfig {
            enable_hsts: true,
            hsts_max_age: 31536000,
            hsts_include_subdomains: false,
            csp_policy: "default-src 'self'".to_string(),
        };
        let hsts = config.build_hsts_header();
        assert!(!hsts.contains("includeSubDomains"));
        assert!(hsts.contains("preload"));
    }

    #[test]
    fn test_security_headers_layer_creation() {
        let layer = SecurityHeadersLayer::new();
        assert!(layer.config().enable_hsts);

        let dev_layer = SecurityHeadersLayer::dev();
        assert!(!dev_layer.config().enable_hsts);

        let prod_layer = SecurityHeadersLayer::production();
        assert!(prod_layer.config().enable_hsts);
    }

    #[test]
    fn test_csp_contains_required_directives() {
        let config = SecurityHeadersConfig::default();
        assert!(config.csp_policy.contains("default-src 'self'"));
        assert!(config.csp_policy.contains("frame-ancestors 'none'"));
        assert!(config.csp_policy.contains("base-uri 'self'"));
        assert!(config.csp_policy.contains("form-action 'self'"));
    }
}
