//! Authentication middleware for protecting API endpoints
//!
//! This middleware supports two authentication modes:
//! 1. JWT authentication (KEYRX_JWT_SECRET set) - validates JWT tokens
//! 2. Legacy password auth (KEYRX_ADMIN_PASSWORD set) - validates passwords
//! 3. Dev mode (neither set) - no authentication required
//!
//! Endpoints that bypass authentication:
//! - /health, /api/health
//! - /api/auth/login, /api/auth/refresh

use axum::{
    body::Body,
    extract::Request,
    http::{Response, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use std::sync::Arc;

use crate::auth::{AuthMode, AuthService};

/// Authentication middleware state
#[derive(Clone)]
pub struct AuthMiddleware {
    auth_mode: Arc<AuthMode>,
    auth_service: Option<Arc<AuthService>>,
}

impl AuthMiddleware {
    /// Create new authentication middleware
    pub fn new(auth_mode: AuthMode) -> Self {
        let auth_service = if matches!(auth_mode, AuthMode::Jwt) {
            Some(Arc::new(AuthService::new()))
        } else {
            None
        };

        Self {
            auth_mode: Arc::new(auth_mode),
            auth_service,
        }
    }

    /// Get the auth mode for manual checking
    pub fn auth_mode(&self) -> &AuthMode {
        &self.auth_mode
    }
}

/// Authentication middleware handler
pub async fn auth_middleware(
    auth: axum::extract::State<AuthMiddleware>,
    request: Request,
    next: Next,
) -> Result<Response<Body>, impl IntoResponse> {
    let path = request.uri().path();

    // Skip authentication for public endpoints
    let public_paths = [
        "/health",
        "/api/health",
        "/api/auth/login",
        "/api/auth/refresh",
    ];

    if public_paths.contains(&path) {
        return Ok(next.run(request).await);
    }

    // If in dev mode, allow all requests
    if !auth.auth_mode().is_auth_required() {
        return Ok(next.run(request).await);
    }

    // Check Authorization header
    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok());

    let token = match auth_header {
        Some(header) => {
            // Extract Bearer token
            if let Some(token) = header.strip_prefix("Bearer ") {
                token
            } else {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    "Invalid Authorization header format. Use: Authorization: Bearer <token>",
                ));
            }
        }
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                "Missing Authorization header. Use: Authorization: Bearer <token>",
            ));
        }
    };

    // Validate based on auth mode
    match (auth.auth_mode(), &auth.auth_service) {
        (AuthMode::Jwt, Some(auth_service)) => {
            // Validate JWT token
            match auth_service.jwt_manager.validate_access_token(token) {
                Ok(_claims) => {
                    // Token is valid, continue
                    Ok(next.run(request).await)
                }
                Err(_) => Err((StatusCode::UNAUTHORIZED, "Invalid or expired token")),
            }
        }
        (AuthMode::Password(_), _) => {
            // Legacy password validation
            if !auth.auth_mode().validate_password(token) {
                return Err((StatusCode::UNAUTHORIZED, "Invalid password"));
            }
            Ok(next.run(request).await)
        }
        _ => {
            // Should not reach here, but default to unauthorized
            Err((StatusCode::UNAUTHORIZED, "Authentication failed"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        middleware,
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    async fn test_handler() -> &'static str {
        "OK"
    }

    #[tokio::test]
    async fn test_dev_mode_allows_all() {
        let auth = AuthMiddleware::new(AuthMode::DevMode);
        let app = Router::new()
            .route("/api/test", get(test_handler))
            .layer(middleware::from_fn_with_state(auth, auth_middleware));

        let request = Request::builder()
            .uri("/api/test")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_password_mode_requires_auth() {
        let auth = AuthMiddleware::new(AuthMode::Password("secret".to_string()));
        let app = Router::new()
            .route("/api/test", get(test_handler))
            .layer(middleware::from_fn_with_state(auth, auth_middleware));

        // No auth header - should fail
        let request = Request::builder()
            .uri("/api/test")
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Wrong password - should fail
        let request = Request::builder()
            .uri("/api/test")
            .header("authorization", "Bearer wrong")
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Correct password - should succeed
        let request = Request::builder()
            .uri("/api/test")
            .header("authorization", "Bearer secret")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_health_endpoint_always_allowed() {
        let auth = AuthMiddleware::new(AuthMode::Password("secret".to_string()));
        let app = Router::new()
            .route("/health", get(test_handler))
            .layer(middleware::from_fn_with_state(auth, auth_middleware));

        let request = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
