//! Authentication module with JWT and password hashing
//!
//! This module provides comprehensive authentication with:
//! - JWT token generation and validation
//! - Argon2 password hashing
//! - Rate limiting for login attempts
//! - HTTP handlers for auth endpoints
//!
//! # Security Model
//!
//! - JWT access tokens with 15-minute expiry
//! - Refresh tokens with 7-day expiry
//! - Argon2id password hashing
//! - Rate limiting: 5 attempts per minute per IP
//! - Secure cookie flags (httpOnly, secure, sameSite)
//!
//! # Environment Variables
//!
//! - `KEYRX_JWT_SECRET`: JWT signing secret (required for production)
//! - `KEYRX_ADMIN_PASSWORD`: Admin password (for backward compatibility)

pub mod handlers;
pub mod jwt;
pub mod password;
pub mod rate_limit;

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex};

pub use handlers::{login, logout, refresh, validate};
pub use jwt::{Claims, JwtManager, TokenType};
pub use password::PasswordHasher;
pub use rate_limit::LoginRateLimiter;

/// Authentication mode based on environment configuration
#[derive(Debug, Clone)]
pub enum AuthMode {
    /// Development mode - no authentication required
    DevMode,
    /// JWT authentication with password hashing
    Jwt,
    /// Legacy password authentication (backward compatibility)
    Password(String),
}

impl AuthMode {
    /// Load authentication mode from environment
    pub fn from_env() -> Self {
        // Check for JWT secret first (new auth system)
        if env::var("KEYRX_JWT_SECRET").is_ok() {
            log::info!("JWT authentication enabled");
            return AuthMode::Jwt;
        }

        // Fall back to legacy password auth
        match env::var("KEYRX_ADMIN_PASSWORD") {
            Ok(password) if !password.is_empty() => {
                log::info!("Legacy password authentication enabled");
                AuthMode::Password(password)
            }
            _ => {
                log::warn!(
                    "No authentication configured - running in dev mode (all endpoints accessible)"
                );
                log::warn!(
                    "Set KEYRX_JWT_SECRET environment variable to enable JWT authentication"
                );
                AuthMode::DevMode
            }
        }
    }

    /// Check if authentication is required
    pub fn is_auth_required(&self) -> bool {
        !matches!(self, AuthMode::DevMode)
    }

    /// Validate password against stored password (legacy mode)
    pub fn validate_password(&self, provided: &str) -> bool {
        match self {
            AuthMode::DevMode => true,
            AuthMode::Password(password) => {
                constant_time_eq(password.as_bytes(), provided.as_bytes())
            }
            AuthMode::Jwt => false, // JWT mode uses token validation
        }
    }
}

/// Authentication service
pub struct AuthService {
    pub jwt_manager: JwtManager,
    pub rate_limiter: LoginRateLimiter,
    user_store: Arc<Mutex<HashMap<String, String>>>, // username -> password_hash
}

impl AuthService {
    /// Create a new authentication service
    pub fn new() -> Self {
        let jwt_secret = env::var("KEYRX_JWT_SECRET")
            .unwrap_or_else(|_| "default_dev_secret_change_in_production".to_string());

        let mut user_store = HashMap::new();

        // Create default admin user if configured
        if let Ok(admin_pass) = env::var("KEYRX_ADMIN_PASSWORD") {
            let hasher = PasswordHasher::new();
            if let Ok(hash) = hasher.hash_password(&admin_pass) {
                user_store.insert("admin".to_string(), hash);
                log::info!("Default admin user created from KEYRX_ADMIN_PASSWORD");
            }
        }

        Self {
            jwt_manager: JwtManager::new(&jwt_secret),
            rate_limiter: LoginRateLimiter::new(),
            user_store: Arc::new(Mutex::new(user_store)),
        }
    }

    /// Get password hash for a user
    pub fn get_user_password_hash(&self, username: &str) -> Option<String> {
        let store = self.user_store.lock().unwrap();
        store.get(username).cloned()
    }

    /// Add or update a user
    pub fn upsert_user(&self, username: &str, password_hash: &str) {
        let mut store = self.user_store.lock().unwrap();
        store.insert(username.to_string(), password_hash.to_string());
    }
}

impl Default for AuthService {
    fn default() -> Self {
        Self::new()
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
mod unit_tests {
    use super::*;

    #[test]
    fn test_auth_mode_dev() {
        let mode = AuthMode::DevMode;
        assert!(!mode.is_auth_required());
        assert!(mode.validate_password("anything"));
        assert!(mode.validate_password(""));
    }

    #[test]
    fn test_auth_service_default_user() {
        std::env::set_var("KEYRX_ADMIN_PASSWORD", "TestP@ssw0rd123");
        let service = AuthService::new();
        assert!(service.get_user_password_hash("admin").is_some());
        std::env::remove_var("KEYRX_ADMIN_PASSWORD");
    }
}
