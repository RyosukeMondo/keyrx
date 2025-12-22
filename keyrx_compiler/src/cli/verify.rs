//! Verify subcommand handler.
//!
//! Handles the `verify` subcommand which validates .krx binary files.

use std::fmt;
use std::io;
use std::path::Path;

use crate::error::DeserializeError;

/// Errors that can occur during the verify subcommand.
#[derive(Debug)]
pub enum VerifyError {
    /// Failed to deserialize .krx file.
    DeserializeError(DeserializeError),

    /// I/O error during file operations.
    IoError(io::Error),
}

impl fmt::Display for VerifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DeserializeError(err) => write!(f, "Deserialization error: {:?}", err),
            Self::IoError(err) => write!(f, "I/O error: {}", err),
        }
    }
}

impl std::error::Error for VerifyError {}

impl From<io::Error> for VerifyError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}

impl From<DeserializeError> for VerifyError {
    fn from(err: DeserializeError) -> Self {
        Self::DeserializeError(err)
    }
}

/// Handles the verify subcommand.
///
/// # Arguments
///
/// * `file` - Path to the .krx binary file to verify.
///
/// # Returns
///
/// `Ok(())` on success, or `VerifyError` on failure.
pub fn handle_verify(file: &Path) -> Result<(), VerifyError> {
    // TODO: Implementation in task 14
    eprintln!("Verifying {:?}", file);
    eprintln!("TODO: Implementation in task 14");
    Ok(())
}
