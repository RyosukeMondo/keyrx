//! Hash subcommand handler.
//!
//! Handles the `hash` subcommand which extracts and verifies SHA256 hashes
//! from .krx binary files.

use std::fmt;
use std::io;
use std::path::Path;

/// Errors that can occur during the hash subcommand.
#[derive(Debug)]
pub enum HashError {
    /// Hash verification failed (mismatch between embedded and computed hash).
    HashMismatch {
        embedded: [u8; 32],
        computed: [u8; 32],
    },

    /// File is too small to contain a valid .krx header.
    FileTooSmall { size: usize, min_size: usize },

    /// I/O error during file operations.
    IoError(io::Error),
}

impl fmt::Display for HashError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HashMismatch { embedded, computed } => {
                write!(
                    f,
                    "Hash mismatch: embedded={:02x?}, computed={:02x?}",
                    embedded, computed
                )
            }
            Self::FileTooSmall { size, min_size } => {
                write!(
                    f,
                    "File too small: {} bytes (minimum {} bytes required)",
                    size, min_size
                )
            }
            Self::IoError(err) => write!(f, "I/O error: {}", err),
        }
    }
}

impl std::error::Error for HashError {}

impl From<io::Error> for HashError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}

/// Handles the hash subcommand.
///
/// # Arguments
///
/// * `file` - Path to the .krx binary file.
/// * `verify` - If true, compute the hash and verify it matches the embedded hash.
///
/// # Returns
///
/// `Ok(())` on success, or `HashError` on failure.
pub fn handle_hash(file: &Path, verify: bool) -> Result<(), HashError> {
    // TODO: Implementation in task 15
    eprintln!("Extracting hash from {:?} (verify={})", file, verify);
    eprintln!("TODO: Implementation in task 15");
    Ok(())
}
