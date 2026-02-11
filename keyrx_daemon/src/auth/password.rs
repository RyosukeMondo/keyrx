//! Password hashing and validation using Argon2
//!
//! This module provides secure password hashing with:
//! - Argon2id algorithm (recommended by OWASP)
//! - Salt generation using secure random
//! - Configurable memory and iteration parameters
//! - Password complexity validation

use argon2::{
    password_hash::{
        rand_core::OsRng, PasswordHash, PasswordHasher as Argon2PasswordHasher, PasswordVerifier,
        SaltString,
    },
    Argon2,
};
use thiserror::Error;

/// Password errors
#[derive(Debug, Error)]
pub enum PasswordError {
    #[error("Password hashing failed: {0}")]
    HashingError(String),
    #[error("Password verification failed")]
    VerificationFailed,
    #[error("Password too weak: {0}")]
    WeakPassword(String),
}

/// Password hasher using Argon2
pub struct PasswordHasher {
    argon2: Argon2<'static>,
}

impl Default for PasswordHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl PasswordHasher {
    /// Create a new password hasher with default parameters
    pub fn new() -> Self {
        Self {
            argon2: Argon2::default(),
        }
    }

    /// Hash a password with Argon2
    pub fn hash_password(&self, password: &str) -> Result<String, PasswordError> {
        // Validate password complexity first
        self.validate_password_complexity(password)?;

        let salt = SaltString::generate(&mut OsRng);
        let password_hash = self
            .argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| PasswordError::HashingError(e.to_string()))?;

        Ok(password_hash.to_string())
    }

    /// Verify a password against a hash
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, PasswordError> {
        let parsed_hash =
            PasswordHash::new(hash).map_err(|e| PasswordError::HashingError(e.to_string()))?;

        match self
            .argon2
            .verify_password(password.as_bytes(), &parsed_hash)
        {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Validate password complexity requirements
    fn validate_password_complexity(&self, password: &str) -> Result<(), PasswordError> {
        if password.len() < 12 {
            return Err(PasswordError::WeakPassword(
                "Password must be at least 12 characters".to_string(),
            ));
        }

        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_special = password.chars().any(|c| !c.is_alphanumeric());

        let complexity_score = [has_uppercase, has_lowercase, has_digit, has_special]
            .iter()
            .filter(|&&x| x)
            .count();

        if complexity_score < 3 {
            return Err(PasswordError::WeakPassword(
                "Password must contain at least 3 of: uppercase, lowercase, digit, special character".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify_password() {
        let hasher = PasswordHasher::new();
        let password = "SecureP@ssw0rd123";
        let hash = hasher.hash_password(password).unwrap();

        assert!(hasher.verify_password(password, &hash).unwrap());
        assert!(!hasher.verify_password("WrongPassword", &hash).unwrap());
    }

    #[test]
    fn test_password_too_short() {
        let hasher = PasswordHasher::new();
        let result = hasher.hash_password("Short1!");
        assert!(matches!(result, Err(PasswordError::WeakPassword(_))));
    }

    #[test]
    fn test_password_no_complexity() {
        let hasher = PasswordHasher::new();
        let result = hasher.hash_password("alllowercase");
        assert!(matches!(result, Err(PasswordError::WeakPassword(_))));
    }

    #[test]
    fn test_valid_complex_password() {
        let hasher = PasswordHasher::new();
        let result = hasher.hash_password("C0mpl3x!P@ssw0rd");
        assert!(result.is_ok());
    }
}
