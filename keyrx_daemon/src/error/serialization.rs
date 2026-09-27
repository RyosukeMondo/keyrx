//! Binary format parsing and serialization error types.

use std::io;
use thiserror::Error;

/// Binary format parsing and serialization errors.
///
/// This error type covers failures when parsing or validating .krx binary files.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SerializationError {
    /// Magic number in binary file doesn't match expected value.
    #[error("Invalid magic number: expected {expected:#010x}, found {found:#010x}")]
    InvalidMagic {
        /// Expected magic number.
        expected: u32,
        /// Actual magic number found in file.
        found: u32,
    },

    /// Version number is not supported.
    #[error("Unsupported version: expected {expected}, found {found}")]
    InvalidVersion {
        /// Expected version number.
        expected: u32,
        /// Actual version number found in file.
        found: u32,
    },

    /// Buffer size doesn't match expected size.
    #[error("Invalid size: expected {expected} bytes, found {found} bytes")]
    InvalidSize {
        /// Expected buffer size in bytes.
        expected: usize,
        /// Actual buffer size in bytes.
        found: usize,
    },

    /// Data is corrupted or malformed.
    #[error("Corrupted data: {0}")]
    CorruptedData(String),

    /// IO error occurred during serialization.
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}
