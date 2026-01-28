//! Authentication middleware for protecting API endpoints
//!
//! This middleware checks the Authorization header on all requests (except /health).
//! If KEYRX_ADMIN_PASSWORD is set, requests must include:
//! `Authorization: Bearer <password>`

use axum::{
    body::Body,
    extract::Request,
    http::{Response, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use std::sync::Arc;

use crate::auth::AuthMode;

/// Authentication middleware state
#[derive(Clone)]
pub struct AuthMiddleware {
    auth_mode: Arc<AuthMode>,
}

impl AuthMiddleware {
    /// Create new authentication middleware
    pub fn new(auth_mode: AuthMode) -> Self {
        Self {
            auth_mode: Arc::new(auth_mode),
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

    // Skip authentication for health endpoint
    if path == "/health" || path == "/api/health" {
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

    let password = match auth_header {
        Some(header) => {
            // Extract Bearer token
            if let Some(token) = header.strip_prefix("Bearer ") {
                token
            } else {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    "Invalid Authorization header format. Use: Authorization: Bearer <password>",
                ));
            }
        }
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                "Missing Authorization header. Use: Authorization: Bearer <password>",
            ));
        }
    };

    // Validate password
    if !auth.auth_mode().validate_password(password) {
        return Err((StatusCode::UNAUTHORIZED, "Invalid password"));
    }

    // Password is valid, continue
    Ok(next.run(request).await)
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
