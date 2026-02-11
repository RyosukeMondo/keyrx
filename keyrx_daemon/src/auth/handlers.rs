//! Authentication HTTP handlers
//!
//! This module provides HTTP handlers for:
//! - Login (username/password)
//! - Logout (invalidate token)
//! - Token refresh
//! - Token validation

use axum::{
    extract::{ConnectInfo, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;

use super::{password::PasswordHasher, AuthService};

/// Login request
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Login response
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

/// Token refresh request
#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

/// Token validation response
#[derive(Debug, Serialize)]
pub struct ValidationResponse {
    pub valid: bool,
    pub user_id: Option<String>,
    pub expires_at: Option<u64>,
}

/// Authentication error response
#[derive(Debug, Serialize)]
pub struct AuthError {
    pub error: String,
    pub retry_after: Option<u64>,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let status = if self.retry_after.is_some() {
            StatusCode::TOO_MANY_REQUESTS
        } else {
            StatusCode::UNAUTHORIZED
        };

        (status, Json(self)).into_response()
    }
}

/// Login handler
pub async fn login(
    State(auth_service): State<Arc<AuthService>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AuthError> {
    // Check rate limit
    if let Err(e) = auth_service.rate_limiter.check_rate_limit(addr.ip()) {
        return Err(AuthError {
            error: e.to_string(),
            retry_after: Some(60),
        });
    }

    // Validate credentials
    let hasher = PasswordHasher::new();
    let stored_hash = auth_service
        .get_user_password_hash(&req.username)
        .ok_or_else(|| AuthError {
            error: "Invalid credentials".to_string(),
            retry_after: None,
        })?;

    let valid = hasher
        .verify_password(&req.password, &stored_hash)
        .map_err(|e| AuthError {
            error: format!("Authentication failed: {}", e),
            retry_after: None,
        })?;

    if !valid {
        return Err(AuthError {
            error: "Invalid credentials".to_string(),
            retry_after: None,
        });
    }

    // Reset rate limit on successful login
    auth_service.rate_limiter.reset(addr.ip());

    // Generate tokens
    let access_token = auth_service
        .jwt_manager
        .generate_access_token(&req.username)
        .map_err(|e| AuthError {
            error: format!("Failed to generate token: {}", e),
            retry_after: None,
        })?;

    let refresh_token = auth_service
        .jwt_manager
        .generate_refresh_token(&req.username)
        .map_err(|e| AuthError {
            error: format!("Failed to generate refresh token: {}", e),
            retry_after: None,
        })?;

    Ok(Json(LoginResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: 15 * 60, // 15 minutes
    }))
}

/// Logout handler (placeholder - token invalidation would require token storage)
pub async fn logout() -> Result<StatusCode, AuthError> {
    // In a stateless JWT system, logout is typically handled client-side by discarding the token
    // For a more secure implementation, maintain a token blacklist
    Ok(StatusCode::NO_CONTENT)
}

/// Token refresh handler
pub async fn refresh(
    State(auth_service): State<Arc<AuthService>>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<LoginResponse>, AuthError> {
    // Validate refresh token
    let claims = auth_service
        .jwt_manager
        .validate_refresh_token(&req.refresh_token)
        .map_err(|e| AuthError {
            error: format!("Invalid refresh token: {}", e),
            retry_after: None,
        })?;

    // Generate new access token
    let access_token = auth_service
        .jwt_manager
        .generate_access_token(&claims.sub)
        .map_err(|e| AuthError {
            error: format!("Failed to generate token: {}", e),
            retry_after: None,
        })?;

    // Optionally rotate refresh token
    let refresh_token = auth_service
        .jwt_manager
        .generate_refresh_token(&claims.sub)
        .map_err(|e| AuthError {
            error: format!("Failed to generate refresh token: {}", e),
            retry_after: None,
        })?;

    Ok(Json(LoginResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: 15 * 60,
    }))
}

/// Token validation handler
pub async fn validate(
    State(auth_service): State<Arc<AuthService>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<ValidationResponse>, AuthError> {
    // Extract token from Authorization header
    let auth_header = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AuthError {
            error: "Missing authorization header".to_string(),
            retry_after: None,
        })?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AuthError {
            error: "Invalid authorization header format".to_string(),
            retry_after: None,
        })?;

    // Validate token
    match auth_service.jwt_manager.validate_access_token(token) {
        Ok(claims) => Ok(Json(ValidationResponse {
            valid: true,
            user_id: Some(claims.sub),
            expires_at: Some(claims.exp),
        })),
        Err(_) => Ok(Json(ValidationResponse {
            valid: false,
            user_id: None,
            expires_at: None,
        })),
    }
}
