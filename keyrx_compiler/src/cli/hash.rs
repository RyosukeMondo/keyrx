//! Hash subcommand handler.
//!
//! Handles the `hash` subcommand which extracts and verifies SHA256 hashes
//! from .krx binary files.

use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

use sha2::{Digest, Sha256};

/// Errors that can occur during the hash subcommand.
#[derive(Debug)]
#[allow(dead_code)] // Will be used when integrated into main.rs in task 17
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
#[allow(dead_code)] // Will be used when integrated into main.rs in task 17
pub fn handle_hash(file: &Path, verify: bool) -> Result<(), HashError> {
    // Read the .krx file
    let bytes = fs::read(file)?;

    // Minimum size is 48 bytes (header)
    const MIN_SIZE: usize = 48;
    if bytes.len() < MIN_SIZE {
        return Err(HashError::FileTooSmall {
            size: bytes.len(),
            min_size: MIN_SIZE,
        });
    }

    // Extract embedded hash from bytes 8-40 (32 bytes)
    // SAFETY: Slice bounds checked at line 77: file size >= 48 bytes guarantees [8..40] is valid
    #[allow(clippy::expect_used)]
    let embedded_hash: [u8; 32] = bytes[8..40]
        .try_into()
        .expect("Hash slice is exactly 32 bytes");

    // Print the embedded hash in hexadecimal format
    let hash_hex = hex::encode(embedded_hash);
    println!("{}", hash_hex);

    // If verify flag is set, compute hash of data section and compare
    if verify {
        // Data section starts at byte 48
        let data = &bytes[48..];

        // Compute SHA256 hash of data section
        let mut hasher = Sha256::new();
        hasher.update(data);
        let computed_hash: [u8; 32] = hasher.finalize().into();

        // Compare hashes
        if computed_hash == embedded_hash {
            eprintln!("✓ Hash matches");
        } else {
            eprintln!("✗ Hash mismatch");
            eprintln!("  Embedded:  {}", hex::encode(embedded_hash));
            eprintln!("  Computed:  {}", hex::encode(computed_hash));
            return Err(HashError::HashMismatch {
                embedded: embedded_hash,
                computed: computed_hash,
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    use crate::serialize::serialize;
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// Helper to create a valid .krx file for testing
    fn create_test_krx_file() -> NamedTempFile {
        // Create a simple Rhai script
        let script = r#"
device_start("Test Device");
map("A", "VK_B");
device_end();
"#;

        // Write script to temp file
        let mut script_file = NamedTempFile::new().expect("Failed to create temp script file");
        script_file
            .write_all(script.as_bytes())
            .expect("Failed to write script");

        // Parse and serialize
        let mut parser = Parser::new();
        let config = parser
            .parse_script(script_file.path())
            .expect("Parse failed");
        let krx_bytes = serialize(&config).expect("Serialize failed");

        // Write to temp file
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        temp_file
            .write_all(&krx_bytes)
            .expect("Failed to write to temp file");
        temp_file
    }

    #[test]
    fn test_handle_hash_extract_only() {
        let temp_file = create_test_krx_file();
        let result = handle_hash(temp_file.path(), false);
        assert!(result.is_ok(), "Hash extraction should succeed");
    }

    #[test]
    fn test_handle_hash_verify_valid() {
        let temp_file = create_test_krx_file();
        let result = handle_hash(temp_file.path(), true);
        assert!(result.is_ok(), "Hash verification should succeed");
    }

    #[test]
    fn test_handle_hash_verify_corrupted() {
        let temp_file = create_test_krx_file();

        // Read and corrupt the file
        let path = temp_file.path().to_path_buf();
        let mut bytes = fs::read(&path).expect("Failed to read file");

        // Corrupt data section (after byte 48)
        if bytes.len() > 50 {
            bytes[50] = !bytes[50];
        }

        // Write corrupted data back
        fs::write(&path, bytes).expect("Failed to write corrupted file");

        // Verify should fail
        let result = handle_hash(&path, true);
        assert!(result.is_err(), "Hash verification should fail");
        assert!(
            matches!(result.unwrap_err(), HashError::HashMismatch { .. }),
            "Error should be HashMismatch"
        );
    }

    #[test]
    fn test_handle_hash_file_too_small() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        temp_file
            .write_all(&[0u8; 10])
            .expect("Failed to write to temp file");

        let result = handle_hash(temp_file.path(), false);
        assert!(result.is_err(), "Should fail on too small file");
        assert!(
            matches!(result.unwrap_err(), HashError::FileTooSmall { .. }),
            "Error should be FileTooSmall"
        );
    }

    #[test]
    fn test_handle_hash_missing_file() {
        let result = handle_hash(Path::new("/nonexistent/file.krx"), false);
        assert!(result.is_err(), "Should fail on missing file");
        assert!(
            matches!(result.unwrap_err(), HashError::IoError(_)),
            "Error should be IoError"
        );
    }

    #[test]
    fn test_hash_error_display() {
        let error = HashError::FileTooSmall {
            size: 10,
            min_size: 48,
        };
        let display = format!("{}", error);
        assert!(display.contains("10 bytes"));
        assert!(display.contains("48 bytes"));
    }

    #[test]
    fn test_hash_mismatch_error_display() {
        let embedded = [1u8; 32];
        let computed = [2u8; 32];
        let error = HashError::HashMismatch { embedded, computed };
        let display = format!("{}", error);
        assert!(display.contains("Hash mismatch"));
        assert!(display.contains("embedded"));
        assert!(display.contains("computed"));
    }
}
