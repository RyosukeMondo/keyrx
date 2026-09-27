//! Web server and API error types.

use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Web server and API errors.
///
/// This error type covers failures in the embedded web server and REST API.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum WebError {
    /// Failed to bind web server to the specified address.
    #[error("Failed to bind web server to {address}: {reason}")]
    BindFailed {
        /// Address where binding was attempted.
        address: String,
        /// Reason for bind failure.
        reason: String,
    },

    /// Invalid API request.
    #[error("Invalid API request: {reason}")]
    InvalidRequest {
        /// Reason why the request is invalid.
        reason: String,
    },

    /// WebSocket error occurred.
    #[error("WebSocket error: {reason}")]
    WebSocketError {
        /// Reason for WebSocket failure.
        reason: String,
    },

    /// Failed to serve static files.
    #[error("Failed to serve static file {path:?}: {reason}")]
    StaticFileError {
        /// Path to the static file.
        path: PathBuf,
        /// Reason for failure.
        reason: String,
    },

    /// IO error occurred during web operation.
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}
