//! Device registry operation error types.

use std::io;
use thiserror::Error;

/// Device registry operation errors.
///
/// This error type covers failures when loading or saving the device registry.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum RegistryError {
    /// IO error occurred during registry operation.
    #[error("IO error: {0:?}")]
    IOError(io::ErrorKind),

    /// Registry file is corrupted.
    #[error("Corrupted registry: {0}")]
    CorruptedRegistry(String),

    /// Failed to load registry file.
    #[error("Failed to load registry: {0:?}")]
    FailedToLoad(io::ErrorKind),
}
