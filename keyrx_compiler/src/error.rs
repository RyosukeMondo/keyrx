//! Error types for the keyrx compiler.
//!
//! This module defines structured error types for all phases of compilation:
//! - Parsing errors (syntax, validation, imports)
//! - Serialization errors
//! - Deserialization errors

use std::error::Error;
use std::fmt;
use std::path::PathBuf;

/// Errors that can occur during Rhai script parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // Will be used by parser module
pub enum ParseError {
    /// Syntax error in Rhai script.
    SyntaxError {
        file: PathBuf,
        line: usize,
        column: usize,
        message: String,
    },

    /// Invalid prefix used (expected VK_/MD_/LK_, got something else).
    InvalidPrefix {
        expected: String,
        got: String,
        context: String,
    },

    /// Custom modifier ID out of valid range (00-FE).
    ModifierIdOutOfRange { got: u16, max: u8 },

    /// Custom lock ID out of valid range (00-FE).
    LockIdOutOfRange { got: u16, max: u8 },

    /// Physical modifier name used in MD_ prefix (e.g., MD_LShift).
    PhysicalModifierInMD { name: String },

    /// Required prefix missing from key name.
    MissingPrefix { key: String, context: String },

    /// Import file not found.
    ImportNotFound {
        path: PathBuf,
        searched_paths: Vec<PathBuf>,
    },

    /// Circular import detected.
    CircularImport { chain: Vec<PathBuf> },

    /// Resource limit exceeded (operations, depth, etc.).
    ResourceLimitExceeded { limit_type: String },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::SyntaxError {
                file,
                line,
                column,
                message,
            } => write!(
                f,
                "{}:{}:{}: Syntax error: {}",
                file.display(),
                line,
                column,
                message
            ),

            ParseError::InvalidPrefix {
                expected,
                got,
                context,
            } => write!(
                f,
                "Invalid prefix: expected {}, got '{}' (context: {})",
                expected, got, context
            ),

            ParseError::ModifierIdOutOfRange { got, max } => write!(
                f,
                "Modifier ID out of range: {} (valid range: 00-{:02X})",
                got, max
            ),

            ParseError::LockIdOutOfRange { got, max } => write!(
                f,
                "Lock ID out of range: {} (valid range: 00-{:02X})",
                got, max
            ),

            ParseError::PhysicalModifierInMD { name } => write!(
                f,
                "Physical modifier name '{}' cannot be used with MD_ prefix. \
                 Use MD_00 through MD_FE for custom modifiers.",
                name
            ),

            ParseError::MissingPrefix { key, context } => write!(
                f,
                "Missing prefix for key '{}' (context: {}). \
                 Use VK_ for virtual keys, MD_ for modifiers, LK_ for locks.",
                key, context
            ),

            ParseError::ImportNotFound {
                path,
                searched_paths,
            } => {
                write!(f, "Import file not found: {}", path.display())?;
                if !searched_paths.is_empty() {
                    write!(f, "\nSearched paths:")?;
                    for p in searched_paths {
                        write!(f, "\n  - {}", p.display())?;
                    }
                }
                Ok(())
            }

            ParseError::CircularImport { chain } => {
                writeln!(f, "Circular import detected:")?;
                for (i, path) in chain.iter().enumerate() {
                    write!(f, "  {}. {}", i + 1, path.display())?;
                    if i < chain.len() - 1 {
                        writeln!(f, " →")?;
                    }
                }
                Ok(())
            }

            ParseError::ResourceLimitExceeded { limit_type } => {
                write!(f, "Resource limit exceeded: {}", limit_type)
            }
        }
    }
}

impl Error for ParseError {}

/// Errors that can occur during serialization.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // Will be used by serialize module
pub enum SerializeError {
    /// rkyv serialization error.
    RkyvError(String),

    /// I/O error during file operations.
    IoError(String),
}

impl fmt::Display for SerializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SerializeError::RkyvError(msg) => write!(f, "Serialization error: {}", msg),
            SerializeError::IoError(msg) => write!(f, "I/O error: {}", msg),
        }
    }
}

impl Error for SerializeError {}

impl From<std::io::Error> for SerializeError {
    fn from(err: std::io::Error) -> Self {
        SerializeError::IoError(err.to_string())
    }
}

/// Errors that can occur during deserialization.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // Will be used by serialize module
pub enum DeserializeError {
    /// Invalid magic bytes (expected "KRX\n").
    InvalidMagic { expected: [u8; 4], got: [u8; 4] },

    /// Version mismatch.
    VersionMismatch { expected: u32, got: u32 },

    /// Hash mismatch (data corruption detected).
    HashMismatch {
        expected: [u8; 32],
        computed: [u8; 32],
    },

    /// rkyv deserialization error.
    RkyvError(String),

    /// I/O error during file operations.
    IoError(String),
}

impl fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeserializeError::InvalidMagic { expected, got } => write!(
                f,
                "Invalid magic bytes: expected {:?}, got {:?}",
                expected, got
            ),

            DeserializeError::VersionMismatch { expected, got } => {
                write!(f, "Version mismatch: expected {}, got {}", expected, got)
            }

            DeserializeError::HashMismatch { expected, computed } => {
                write!(
                    f,
                    "Hash mismatch (data corruption detected):\n  Expected: {}\n  Computed: {}",
                    hex_encode(expected),
                    hex_encode(computed)
                )
            }

            DeserializeError::RkyvError(msg) => write!(f, "Deserialization error: {}", msg),

            DeserializeError::IoError(msg) => write!(f, "I/O error: {}", msg),
        }
    }
}

impl Error for DeserializeError {}

impl From<std::io::Error> for DeserializeError {
    fn from(err: std::io::Error) -> Self {
        DeserializeError::IoError(err.to_string())
    }
}

/// Helper function to encode bytes as hex string.
#[allow(dead_code)] // Used by DeserializeError Display impl
fn hex_encode(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_syntax_display() {
        let err = ParseError::SyntaxError {
            file: PathBuf::from("test.rhai"),
            line: 42,
            column: 10,
            message: "unexpected token".to_string(),
        };
        let display = err.to_string();
        assert!(display.contains("test.rhai"));
        assert!(display.contains("42"));
        assert!(display.contains("10"));
        assert!(display.contains("unexpected token"));
    }

    #[test]
    fn test_parse_error_invalid_prefix() {
        let err = ParseError::InvalidPrefix {
            expected: "VK_/MD_/LK_".to_string(),
            got: "XY_A".to_string(),
            context: "map output".to_string(),
        };
        let display = err.to_string();
        assert!(display.contains("VK_/MD_/LK_"));
        assert!(display.contains("XY_A"));
        assert!(display.contains("map output"));
    }

    #[test]
    fn test_parse_error_modifier_out_of_range() {
        let err = ParseError::ModifierIdOutOfRange { got: 255, max: 254 };
        let display = err.to_string();
        assert!(display.contains("255"));
        assert!(display.contains("FE"));
    }

    #[test]
    fn test_parse_error_lock_out_of_range() {
        let err = ParseError::LockIdOutOfRange { got: 255, max: 254 };
        let display = err.to_string();
        assert!(display.contains("255"));
        assert!(display.contains("FE"));
    }

    #[test]
    fn test_parse_error_physical_modifier() {
        let err = ParseError::PhysicalModifierInMD {
            name: "LShift".to_string(),
        };
        let display = err.to_string();
        assert!(display.contains("LShift"));
        assert!(display.contains("MD_00"));
        assert!(display.contains("MD_FE"));
    }

    #[test]
    fn test_parse_error_missing_prefix() {
        let err = ParseError::MissingPrefix {
            key: "A".to_string(),
            context: "map output".to_string(),
        };
        let display = err.to_string();
        assert!(display.contains("'A'"));
        assert!(display.contains("VK_"));
        assert!(display.contains("MD_"));
        assert!(display.contains("LK_"));
    }

    #[test]
    fn test_parse_error_import_not_found() {
        let err = ParseError::ImportNotFound {
            path: PathBuf::from("missing.rhai"),
            searched_paths: vec![
                PathBuf::from("/path1/missing.rhai"),
                PathBuf::from("/path2/missing.rhai"),
            ],
        };
        let display = err.to_string();
        assert!(display.contains("missing.rhai"));
        assert!(display.contains("/path1/missing.rhai"));
        assert!(display.contains("/path2/missing.rhai"));
    }

    #[test]
    fn test_parse_error_circular_import() {
        let err = ParseError::CircularImport {
            chain: vec![
                PathBuf::from("a.rhai"),
                PathBuf::from("b.rhai"),
                PathBuf::from("c.rhai"),
                PathBuf::from("a.rhai"),
            ],
        };
        let display = err.to_string();
        assert!(display.contains("Circular import"));
        assert!(display.contains("a.rhai"));
        assert!(display.contains("b.rhai"));
        assert!(display.contains("c.rhai"));
    }

    #[test]
    fn test_serialize_error_display() {
        let err = SerializeError::RkyvError("invalid data".to_string());
        let display = err.to_string();
        assert!(display.contains("Serialization error"));
        assert!(display.contains("invalid data"));

        let err = SerializeError::IoError("file not found".to_string());
        let display = err.to_string();
        assert!(display.contains("I/O error"));
        assert!(display.contains("file not found"));
    }

    #[test]
    fn test_deserialize_error_invalid_magic() {
        let err = DeserializeError::InvalidMagic {
            expected: [0x4B, 0x52, 0x58, 0x0A],
            got: [0x00, 0x00, 0x00, 0x00],
        };
        let display = err.to_string();
        assert!(display.contains("Invalid magic bytes"));
    }

    #[test]
    fn test_deserialize_error_version_mismatch() {
        let err = DeserializeError::VersionMismatch {
            expected: 1,
            got: 2,
        };
        let display = err.to_string();
        assert!(display.contains("Version mismatch"));
        assert!(display.contains("1"));
        assert!(display.contains("2"));
    }

    #[test]
    fn test_deserialize_error_hash_mismatch() {
        let expected = [0u8; 32];
        let mut computed = [0u8; 32];
        computed[0] = 0xFF;

        let err = DeserializeError::HashMismatch { expected, computed };
        let display = err.to_string();
        assert!(display.contains("Hash mismatch"));
        assert!(display.contains("corruption"));
    }

    #[test]
    fn test_deserialize_error_io_error() {
        let err = DeserializeError::IoError("file not found".to_string());
        let display = err.to_string();
        assert!(display.contains("I/O error"));
        assert!(display.contains("file not found"));
    }

    #[test]
    fn test_hex_encode() {
        let bytes = [0x4B, 0x52, 0x58, 0x0A];
        let hex = hex_encode(&bytes);
        assert_eq!(hex, "4b52580a");
    }

    #[test]
    fn test_error_trait_implemented() {
        // Verify all error types implement std::error::Error
        let parse_err: Box<dyn Error> = Box::new(ParseError::ResourceLimitExceeded {
            limit_type: "operations".to_string(),
        });
        assert!(parse_err.to_string().contains("operations"));

        let serialize_err: Box<dyn Error> = Box::new(SerializeError::RkyvError("test".to_string()));
        assert!(serialize_err.to_string().contains("test"));

        let deserialize_err: Box<dyn Error> =
            Box::new(DeserializeError::RkyvError("test".to_string()));
        assert!(deserialize_err.to_string().contains("test"));
    }
}
