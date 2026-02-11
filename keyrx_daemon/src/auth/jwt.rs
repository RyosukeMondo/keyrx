//! JWT token generation and validation
//!
//! This module provides secure JWT token management with:
//! - 15-minute access token expiry
//! - Refresh token support (7-day expiry)
//! - Claims-based authorization
//! - Secure key management

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject (user identifier)
    pub sub: String,
    /// Issued at timestamp
    pub iat: u64,
    /// Expiration timestamp
    pub exp: u64,
    /// Token type (access or refresh)
    pub token_type: TokenType,
}

/// Token type enumeration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Access,
    Refresh,
}

/// JWT errors
#[derive(Debug, Error)]
pub enum JwtError {
    #[error("Failed to encode token: {0}")]
    EncodingError(#[from] jsonwebtoken::errors::Error),
    #[error("Token has expired")]
    Expired,
    #[error("Invalid token type")]
    InvalidTokenType,
    #[error("System time error")]
    SystemTimeError,
}

/// JWT token manager
pub struct JwtManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtManager {
    /// Create a new JWT manager with the given secret
    pub fn new(secret: &str) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
        }
    }

    /// Generate a new access token (15-minute expiry)
    pub fn generate_access_token(&self, user_id: &str) -> Result<String, JwtError> {
        self.generate_token(user_id, TokenType::Access, 15 * 60)
    }

    /// Generate a new refresh token (7-day expiry)
    pub fn generate_refresh_token(&self, user_id: &str) -> Result<String, JwtError> {
        self.generate_token(user_id, TokenType::Refresh, 7 * 24 * 60 * 60)
    }

    /// Generate a token with specified type and duration
    fn generate_token(
        &self,
        user_id: &str,
        token_type: TokenType,
        duration_secs: u64,
    ) -> Result<String, JwtError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| JwtError::SystemTimeError)?
            .as_secs();

        let claims = Claims {
            sub: user_id.to_string(),
            iat: now,
            exp: now + duration_secs,
            token_type,
        };

        encode(&Header::default(), &claims, &self.encoding_key).map_err(JwtError::from)
    }

    /// Validate and decode a token
    pub fn validate_token(&self, token: &str) -> Result<Claims, JwtError> {
        let validation = Validation::default();
        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)?;

        // Check if token has expired
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| JwtError::SystemTimeError)?
            .as_secs();

        if token_data.claims.exp < now {
            return Err(JwtError::Expired);
        }

        Ok(token_data.claims)
    }

    /// Validate access token
    pub fn validate_access_token(&self, token: &str) -> Result<Claims, JwtError> {
        let claims = self.validate_token(token)?;
        if claims.token_type != TokenType::Access {
            return Err(JwtError::InvalidTokenType);
        }
        Ok(claims)
    }

    /// Validate refresh token
    pub fn validate_refresh_token(&self, token: &str) -> Result<Claims, JwtError> {
        let claims = self.validate_token(token)?;
        if claims.token_type != TokenType::Refresh {
            return Err(JwtError::InvalidTokenType);
        }
        Ok(claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_validate_access_token() {
        let manager = JwtManager::new("test_secret_key_12345");
        let token = manager.generate_access_token("user123").unwrap();
        let claims = manager.validate_access_token(&token).unwrap();

        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.token_type, TokenType::Access);
    }

    #[test]
    fn test_generate_and_validate_refresh_token() {
        let manager = JwtManager::new("test_secret_key_12345");
        let token = manager.generate_refresh_token("user123").unwrap();
        let claims = manager.validate_refresh_token(&token).unwrap();

        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.token_type, TokenType::Refresh);
    }

    #[test]
    fn test_invalid_token_type() {
        let manager = JwtManager::new("test_secret_key_12345");
        let access_token = manager.generate_access_token("user123").unwrap();

        // Try to validate access token as refresh token
        let result = manager.validate_refresh_token(&access_token);
        assert!(matches!(result, Err(JwtError::InvalidTokenType)));
    }

    #[test]
    fn test_invalid_token() {
        let manager = JwtManager::new("test_secret_key_12345");
        let result = manager.validate_token("invalid.token.here");
        assert!(result.is_err());
    }
}
