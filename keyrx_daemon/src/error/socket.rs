//! IPC socket operation error types.

use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// IPC socket operation errors.
///
/// This error type covers failures in Unix socket or named pipe operations
/// used for inter-process communication.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SocketError {
    /// Failed to bind socket at the specified path.
    #[error("Failed to bind socket at {path:?}: {error}")]
    BindFailed {
        /// Path where socket binding was attempted.
        path: PathBuf,
        /// Underlying IO error.
        error: io::Error,
    },

    /// Failed to listen on the socket.
    #[error("Failed to listen on socket: {error}")]
    ListenFailed {
        /// Underlying IO error.
        error: io::Error,
    },

    /// Attempted operation on disconnected socket.
    #[error("Socket not connected")]
    NotConnected,

    /// Attempted to connect an already connected socket.
    #[error("Socket already connected")]
    AlreadyConnected,

    /// IO error occurred during socket operation.
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}
