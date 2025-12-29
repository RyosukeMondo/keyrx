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
    use crate::serialize::deserialize;

    // Read .krx file bytes
    let bytes = std::fs::read(file)?;

    // Attempt to deserialize (which performs all validation)
    match deserialize(&bytes) {
        Ok(config) => {
            // All validation passed
            eprintln!("✓ Magic bytes valid");
            eprintln!("✓ Version: {}", crate::serialize::KRX_VERSION);
            eprintln!("✓ SHA256 hash matches");
            eprintln!("✓ rkyv deserialization successful");
            eprintln!("✓ Configuration valid:");
            eprintln!("  - Devices: {}", config.devices.len());

            let total_mappings: usize = config.devices.iter().map(|d| d.mappings.len()).sum();
            eprintln!("  - Total mappings: {}", total_mappings);

            // Display metadata
            eprintln!("\n📋 Metadata:");
            eprintln!("  - Compiler version: {}", config.metadata.compiler_version);
            eprintln!("  - Source hash (SHA256): {}", config.metadata.source_hash);
            eprintln!(
                "  - Compilation timestamp: {}",
                config.metadata.compilation_timestamp
            );

            eprintln!("\n✓ Verification passed");
            Ok(())
        }
        Err(err) => {
            // Validation failed - print specific error details
            match &err {
                DeserializeError::InvalidMagic { expected, got } => {
                    eprintln!("✗ Magic bytes invalid");
                    eprintln!("  Expected: {:?}", expected);
                    eprintln!("  Got: {:?}", got);
                }
                DeserializeError::VersionMismatch { expected, got } => {
                    eprintln!("✗ Version mismatch");
                    eprintln!("  Expected: {}", expected);
                    eprintln!("  Got: {}", got);
                }
                DeserializeError::HashMismatch { expected, computed } => {
                    eprintln!("✗ SHA256 hash mismatch (data corruption)");
                    eprintln!("  Expected: {}", hex::encode(expected));
                    eprintln!("  Computed: {}", hex::encode(computed));
                }
                DeserializeError::RkyvError(msg) => {
                    eprintln!("✗ rkyv deserialization failed");
                    eprintln!("  Error: {}", msg);
                }
                DeserializeError::IoError(msg) => {
                    eprintln!("✗ I/O error");
                    eprintln!("  Error: {}", msg);
                }
            }

            eprintln!("\n✗ Verification failed: {:?}", err);
            Err(err.into())
        }
    }
}
