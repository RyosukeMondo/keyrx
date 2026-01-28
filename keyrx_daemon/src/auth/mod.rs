//! Simple password-based authentication for admin API endpoints
//!
//! This module provides basic password authentication without JWT tokens.
//! Authentication is controlled by the KEYRX_ADMIN_PASSWORD environment variable.
//!
//! # Security Model
//!
//! - If KEYRX_ADMIN_PASSWORD is set: all API endpoints (except /health) require
//!   the password in the Authorization header: `Authorization: Bearer <password>`
//! - If KEYRX_ADMIN_PASSWORD is not set: dev mode, all endpoints are accessible
//! - Password is checked on every request (no session/token management)
//!
//! # Usage
//!
//! ```bash
//! # Set admin password
//! export KEYRX_ADMIN_PASSWORD=your_secure_password
//!
//! # Make authenticated request
//! curl -H "Authorization: Bearer your_secure_password" http://localhost:9867/api/profiles
//! ```

use std::env;

/// Authentication mode based on environment configuration
#[derive(Debug, Clone)]
pub enum AuthMode {
    /// Development mode - no authentication required
    DevMode,
    /// Password authentication required
    Password(String),
}

impl AuthMode {
    /// Load authentication mode from environment
    pub fn from_env() -> Self {
        match env::var("KEYRX_ADMIN_PASSWORD") {
            Ok(password) if !password.is_empty() => {
                log::info!("Admin password authentication enabled");
                AuthMode::Password(password)
            }
            _ => {
                log::warn!(
                    "No admin password set - running in dev mode (all endpoints accessible)"
                );
                log::warn!(
                    "Set KEYRX_ADMIN_PASSWORD environment variable to enable authentication"
                );
                AuthMode::DevMode
            }
        }
    }

    /// Check if authentication is required
    pub fn is_auth_required(&self) -> bool {
        matches!(self, AuthMode::Password(_))
    }

    /// Validate password against stored password
    pub fn validate_password(&self, provided: &str) -> bool {
        match self {
            AuthMode::DevMode => true, // No auth in dev mode
            AuthMode::Password(password) => {
                // Constant-time comparison to prevent timing attacks
                constant_time_eq(password.as_bytes(), provided.as_bytes())
            }
        }
    }
}

/// Constant-time string comparison to prevent timing attacks
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_time_eq() {
        assert!(constant_time_eq(b"password", b"password"));
        assert!(!constant_time_eq(b"password", b"Password"));
        assert!(!constant_time_eq(b"password", b"passwor"));
        assert!(!constant_time_eq(b"password", b"password1"));
    }

    #[test]
    fn test_auth_mode_dev() {
        let mode = AuthMode::DevMode;
        assert!(!mode.is_auth_required());
        assert!(mode.validate_password("anything"));
        assert!(mode.validate_password(""));
    }

    #[test]
    fn test_auth_mode_password() {
        let mode = AuthMode::Password("secret123".to_string());
        assert!(mode.is_auth_required());
        assert!(mode.validate_password("secret123"));
        assert!(!mode.validate_password("wrong"));
        assert!(!mode.validate_password(""));
    }
}
