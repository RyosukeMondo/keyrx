//! Authentication REST API endpoints
//!
//! This module provides HTTP endpoints for JWT authentication:
//! - POST /api/auth/login - Login with username/password
//! - POST /api/auth/logout - Logout (invalidate session)
//! - POST /api/auth/refresh - Refresh access token
//! - GET /api/auth/validate - Validate current token

use axum::{
    extract::{ConnectInfo, State},
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;

use crate::auth::{self, AuthService};

/// Create authentication router
pub fn routes() -> Router<Arc<crate::web::AppState>> {
    let auth_service = Arc::new(AuthService::new());

    Router::new()
        .route("/auth/login", post(login_handler))
        .route("/auth/logout", post(logout_handler))
        .route("/auth/refresh", post(refresh_handler))
        .route("/auth/validate", get(validate_handler))
        .with_state(auth_service)
}

/// Login endpoint handler
async fn login_handler(
    State(auth_service): State<Arc<AuthService>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    body: axum::Json<auth::handlers::LoginRequest>,
) -> Result<
    axum::Json<auth::handlers::LoginResponse>,
    auth::handlers::AuthError,
> {
    auth::handlers::login(
        State(auth_service),
        ConnectInfo(addr),
        axum::Json(body.0),
    )
    .await
}

/// Logout endpoint handler
async fn logout_handler() -> Result<axum::http::StatusCode, auth::handlers::AuthError> {
    auth::handlers::logout().await
}

/// Token refresh endpoint handler
async fn refresh_handler(
    State(auth_service): State<Arc<AuthService>>,
    body: axum::Json<auth::handlers::RefreshRequest>,
) -> Result<
    axum::Json<auth::handlers::LoginResponse>,
    auth::handlers::AuthError,
> {
    auth::handlers::refresh(State(auth_service), axum::Json(body.0)).await
}

/// Token validation endpoint handler
async fn validate_handler(
    State(auth_service): State<Arc<AuthService>>,
    headers: axum::http::HeaderMap,
) -> Result<
    axum::Json<auth::handlers::ValidationResponse>,
    auth::handlers::AuthError,
> {
    auth::handlers::validate(State(auth_service), headers).await
}

#[cfg(test)]
mod tests {
    
    
    

    // TODO: Fix Router oneshot tests - requires proper Service trait setup
    // #[tokio::test]
    // async fn test_login_with_valid_credentials() {
    //     std::env::set_var("KEYRX_ADMIN_PASSWORD", "ValidP@ssw0rd123");
    //     std::env::set_var("KEYRX_JWT_SECRET", "test_secret_key_12345");
    //
    //     let router = routes();
    //
    //     let request = Request::builder()
    //         .uri("/auth/login")
    //         .method("POST")
    //         .header("content-type", "application/json")
    //         .body(Body::from(r#"{"username":"admin","password":"ValidP@ssw0rd123"}"#))
    //         .unwrap();
    //
    //     let response = router.oneshot(request).await.unwrap();
    //     assert_eq!(response.status(), StatusCode::OK);
    //
    //     std::env::remove_var("KEYRX_ADMIN_PASSWORD");
    //     std::env::remove_var("KEYRX_JWT_SECRET");
    // }
    //
    // #[tokio::test]
    // async fn test_login_with_invalid_credentials() {
    //     std::env::set_var("KEYRX_ADMIN_PASSWORD", "ValidP@ssw0rd123");
    //     std::env::set_var("KEYRX_JWT_SECRET", "test_secret_key_12345");
    //
    //     let router = routes();
    //
    //     let request = Request::builder()
    //         .uri("/auth/login")
    //         .method("POST")
    //         .header("content-type", "application/json")
    //         .body(Body::from(r#"{"username":"admin","password":"wrongpassword"}"#))
    //         .unwrap();
    //
    //     let response = router.oneshot(request).await.unwrap();
    //     assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    //
    //     std::env::remove_var("KEYRX_ADMIN_PASSWORD");
    //     std::env::remove_var("KEYRX_JWT_SECRET");
    // }
}
