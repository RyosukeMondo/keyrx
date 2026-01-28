//! API error types and conversions.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// API error type that maps to HTTP status codes and provides structured error responses.
///
/// All variants automatically convert to appropriate HTTP responses with JSON bodies
/// containing error codes and messages.
///
/// # Examples
///
/// ```
/// use keyrx_daemon::web::api::error::ApiError;
///
/// let err = ApiError::NotFound("Profile 'gaming' not found".to_string());
/// // This will return a 404 response with {"success": false, "error": {...}}
/// ```
#[derive(Debug)]
pub enum ApiError {
    /// Resource not found (404 NOT_FOUND)
    NotFound(String),

    /// Invalid request parameters (400 BAD_REQUEST)
    BadRequest(String),

    /// Resource conflict (409 CONFLICT)
    Conflict(String),

    /// Internal server error (500 INTERNAL_SERVER_ERROR)
    InternalError(String),

    /// Daemon service unavailable (503 SERVICE_UNAVAILABLE)
    DaemonNotRunning,
}

/// Converts `ApiError` into an Axum HTTP response with appropriate status code and JSON body.
///
/// The response body follows the format:
/// ```json
/// {
///   "success": false,
///   "error": {
///     "code": "ERROR_CODE",
///     "message": "Error description"
///   }
/// }
/// ```
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT", msg),
            ApiError::InternalError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", msg)
            }
            ApiError::DaemonNotRunning => (
                StatusCode::SERVICE_UNAVAILABLE,
                "DAEMON_NOT_RUNNING",
                "Daemon is not running".to_string(),
            ),
        };

        let body = Json(json!({
            "success": false,
            "error": {
                "code": code,
                "message": message,
            }
        }));

        (status, body).into_response()
    }
}

/// Converts `std::io::Error` to `ApiError::InternalError`.
impl From<std::io::Error> for ApiError {
    fn from(e: std::io::Error) -> Self {
        ApiError::InternalError(e.to_string())
    }
}

/// Converts boxed error trait objects to `ApiError::InternalError`.
impl From<Box<dyn std::error::Error>> for ApiError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        ApiError::InternalError(e.to_string())
    }
}

/// Implements Display for ApiError to enable to_string()
impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::NotFound(msg) => write!(f, "Not found: {}", msg),
            ApiError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            ApiError::Conflict(msg) => write!(f, "Conflict: {}", msg),
            ApiError::InternalError(msg) => write!(f, "Internal error: {}", msg),
            ApiError::DaemonNotRunning => write!(f, "Daemon is not running"),
        }
    }
}

/// Implements Error trait for ApiError
impl std::error::Error for ApiError {}
