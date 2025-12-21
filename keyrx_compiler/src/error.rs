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

/// Formats a ParseError in a user-friendly format with code snippets and suggestions.
#[allow(dead_code)] // Will be used by main.rs CLI implementation
pub fn format_error_user_friendly(error: &ParseError) -> String {
    match error {
        ParseError::SyntaxError {
            file,
            line,
            column,
            message,
        } => {
            format!(
                "{}:{}:{}: Syntax error: {}\n\n\
                 Help: Check your Rhai script syntax at the indicated location.",
                file.display(),
                line,
                column,
                message
            )
        }

        ParseError::InvalidPrefix {
            expected,
            got,
            context,
        } => {
            let suggestion = if got.starts_with("MD_") {
                // Physical modifier name in MD_ prefix
                format!(
                    "Unknown key prefix: {} (use MD_00 through MD_FE for custom modifiers)\n\n\
                     Example: Instead of 'MD_LShift', use 'MD_00' for a custom modifier.",
                    got
                )
            } else if got.starts_with("VK_") && context.contains("hold") {
                // Wrong prefix in tap_hold
                format!(
                    "tap_hold hold parameter must have MD_ prefix, got: {}\n\n\
                     Example: tap_hold(\"Space\", \"VK_Space\", \"MD_00\", 200)",
                    got
                )
            } else {
                format!(
                    "Invalid prefix: expected {}, got '{}' (context: {})\n\n\
                     Valid prefixes:\n\
                     - VK_ for virtual keys (e.g., VK_A, VK_Enter)\n\
                     - MD_ for custom modifiers (e.g., MD_00, MD_01)\n\
                     - LK_ for custom locks (e.g., LK_00, LK_01)",
                    expected, got, context
                )
            };
            suggestion
        }

        ParseError::ModifierIdOutOfRange { got, max } => {
            format!(
                "Modifier ID out of range: {} (valid range: MD_00 to MD_{:02X})\n\n\
                 Help: Custom modifier IDs must be in the range 00-{:02X} (0-{}).",
                got, max, max, max
            )
        }

        ParseError::LockIdOutOfRange { got, max } => {
            format!(
                "Lock ID out of range: {} (valid range: LK_00 to LK_{:02X})\n\n\
                 Help: Custom lock IDs must be in the range 00-{:02X} (0-{}).",
                got, max, max, max
            )
        }

        ParseError::PhysicalModifierInMD { name } => {
            format!(
                "Physical modifier name '{}' cannot be used with MD_ prefix.\n\n\
                 Physical modifiers (LShift, RShift, LCtrl, RCtrl, LAlt, RAlt, LMeta, RMeta)\n\
                 should be used directly without prefixes in input contexts, or with VK_ in output contexts.\n\n\
                 For custom modifiers, use MD_00 through MD_FE.\n\n\
                 Example: map(\"CapsLock\", \"MD_00\")  // CapsLock becomes custom modifier 00",
                name
            )
        }

        ParseError::MissingPrefix { key, context } => {
            let suggestion = if context.contains("output") || context.contains("to") {
                format!(
                    "Output must have VK_, MD_, or LK_ prefix: {} → use VK_{} for virtual key\n\n\
                     Examples:\n\
                     - map(\"A\", \"VK_B\")        // Remap A to B (virtual key)\n\
                     - map(\"CapsLock\", \"MD_00\") // CapsLock acts as custom modifier 00\n\
                     - map(\"ScrollLock\", \"LK_00\") // ScrollLock toggles custom lock 00",
                    key, key
                )
            } else {
                format!(
                    "Missing prefix for key '{}' (context: {})\n\n\
                     Use VK_ for virtual keys, MD_ for modifiers, LK_ for locks.",
                    key, context
                )
            };
            suggestion
        }

        ParseError::ImportNotFound {
            path,
            searched_paths,
        } => {
            let mut msg = format!("Import file not found: {}\n", path.display());
            if !searched_paths.is_empty() {
                msg.push_str("\nSearched paths:\n");
                for p in searched_paths {
                    msg.push_str(&format!("  - {}\n", p.display()));
                }
            }
            msg.push_str("\nHelp: Make sure the file exists and the path is correct.");
            msg
        }

        ParseError::CircularImport { chain } => {
            let mut msg = String::from("Circular import detected:\n");
            for (i, path) in chain.iter().enumerate() {
                msg.push_str(&format!("  {}. {}", i + 1, path.display()));
                if i < chain.len() - 1 {
                    msg.push_str(" →\n");
                }
            }
            msg.push_str("\n\nHelp: Remove the circular dependency by restructuring your imports.");
            msg
        }

        ParseError::ResourceLimitExceeded { limit_type } => {
            format!(
                "Resource limit exceeded: {}\n\n\
                 Help: Your script is too complex. Consider simplifying or breaking it into smaller parts.",
                limit_type
            )
        }
    }
}

/// Formats a ParseError as a JSON object for machine consumption.
#[allow(dead_code)] // Will be used by main.rs CLI implementation
pub fn format_error_json(error: &ParseError) -> String {
    match error {
        ParseError::SyntaxError {
            file,
            line,
            column,
            message,
        } => {
            serde_json::json!({
                "error_code": "E001",
                "error_type": "SyntaxError",
                "message": message,
                "file": file.to_string_lossy(),
                "line": line,
                "column": column,
                "suggestion": "Check your Rhai script syntax at the indicated location."
            })
            .to_string()
        }

        ParseError::InvalidPrefix {
            expected,
            got,
            context,
        } => {
            let suggestion = if got.starts_with("MD_") && !got.chars().nth(3).is_some_and(|c| c.is_ascii_hexdigit()) {
                format!("Use MD_00 through MD_FE for custom modifiers, not physical modifier names like '{}'", got)
            } else if got.starts_with("VK_") && context.contains("hold") {
                "tap_hold hold parameter must have MD_ prefix for custom modifiers".to_string()
            } else {
                "Use VK_ for virtual keys, MD_ for custom modifiers (00-FE), LK_ for custom locks (00-FE)".to_string()
            };

            serde_json::json!({
                "error_code": "E002",
                "error_type": "InvalidPrefix",
                "message": format!("Invalid prefix: expected {}, got '{}'", expected, got),
                "expected": expected,
                "got": got,
                "context": context,
                "suggestion": suggestion
            })
            .to_string()
        }

        ParseError::ModifierIdOutOfRange { got, max } => {
            serde_json::json!({
                "error_code": "E003",
                "error_type": "ModifierIdOutOfRange",
                "message": format!("Modifier ID {} is out of valid range", got),
                "got": got,
                "max": max,
                "valid_range": format!("MD_00 to MD_{:02X}", max),
                "suggestion": format!("Use a modifier ID between 00 and {:02X} ({} in decimal)", max, max)
            })
            .to_string()
        }

        ParseError::LockIdOutOfRange { got, max } => {
            serde_json::json!({
                "error_code": "E004",
                "error_type": "LockIdOutOfRange",
                "message": format!("Lock ID {} is out of valid range", got),
                "got": got,
                "max": max,
                "valid_range": format!("LK_00 to LK_{:02X}", max),
                "suggestion": format!("Use a lock ID between 00 and {:02X} ({} in decimal)", max, max)
            })
            .to_string()
        }

        ParseError::PhysicalModifierInMD { name } => {
            serde_json::json!({
                "error_code": "E005",
                "error_type": "PhysicalModifierInMD",
                "message": format!("Physical modifier name '{}' cannot be used with MD_ prefix", name),
                "physical_modifier": name,
                "suggestion": "Use MD_00 through MD_FE for custom modifiers. Physical modifiers (LShift, RShift, etc.) should not have MD_ prefix."
            })
            .to_string()
        }

        ParseError::MissingPrefix { key, context } => {
            let suggestion = if context.contains("output") || context.contains("to") {
                format!("Add prefix to '{}': use VK_{} for virtual key, MD_XX for custom modifier, or LK_XX for custom lock", key, key)
            } else {
                "Keys must have VK_, MD_, or LK_ prefix in this context".to_string()
            };

            serde_json::json!({
                "error_code": "E006",
                "error_type": "MissingPrefix",
                "message": format!("Missing prefix for key '{}'", key),
                "key": key,
                "context": context,
                "suggestion": suggestion
            })
            .to_string()
        }

        ParseError::ImportNotFound {
            path,
            searched_paths,
        } => {
            serde_json::json!({
                "error_code": "E007",
                "error_type": "ImportNotFound",
                "message": format!("Import file not found: {}", path.display()),
                "path": path.to_string_lossy(),
                "searched_paths": searched_paths.iter().map(|p| p.to_string_lossy().to_string()).collect::<Vec<_>>(),
                "suggestion": "Make sure the file exists and the path is correct"
            })
            .to_string()
        }

        ParseError::CircularImport { chain } => {
            serde_json::json!({
                "error_code": "E008",
                "error_type": "CircularImport",
                "message": "Circular import detected",
                "import_chain": chain.iter().map(|p| p.to_string_lossy().to_string()).collect::<Vec<_>>(),
                "suggestion": "Remove the circular dependency by restructuring your imports"
            })
            .to_string()
        }

        ParseError::ResourceLimitExceeded { limit_type } => {
            serde_json::json!({
                "error_code": "E009",
                "error_type": "ResourceLimitExceeded",
                "message": format!("Resource limit exceeded: {}", limit_type),
                "limit_type": limit_type,
                "suggestion": "Simplify your script or break it into smaller parts"
            })
            .to_string()
        }
    }
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

    #[test]
    fn test_format_error_user_friendly_syntax_error() {
        let err = ParseError::SyntaxError {
            file: PathBuf::from("test.rhai"),
            line: 42,
            column: 10,
            message: "unexpected token".to_string(),
        };
        let formatted = format_error_user_friendly(&err);
        assert!(formatted.contains("test.rhai:42:10"));
        assert!(formatted.contains("unexpected token"));
        assert!(formatted.contains("Help"));
    }

    #[test]
    fn test_format_error_user_friendly_missing_prefix() {
        let err = ParseError::MissingPrefix {
            key: "B".to_string(),
            context: "map output".to_string(),
        };
        let formatted = format_error_user_friendly(&err);
        assert!(formatted.contains("Output must have VK_, MD_, or LK_ prefix"));
        assert!(formatted.contains("VK_B"));
        assert!(formatted.contains("Examples"));
    }

    #[test]
    fn test_format_error_user_friendly_physical_modifier_in_md() {
        let err = ParseError::PhysicalModifierInMD {
            name: "LShift".to_string(),
        };
        let formatted = format_error_user_friendly(&err);
        assert!(formatted.contains("LShift"));
        assert!(formatted.contains("MD_00"));
        assert!(formatted.contains("MD_FE"));
        assert!(formatted.contains("Example"));
    }

    #[test]
    fn test_format_error_user_friendly_invalid_prefix_md() {
        let err = ParseError::InvalidPrefix {
            expected: "MD_00-MD_FE".to_string(),
            got: "MD_LShift".to_string(),
            context: "modifier".to_string(),
        };
        let formatted = format_error_user_friendly(&err);
        assert!(formatted.contains("MD_LShift"));
        assert!(formatted.contains("MD_00 through MD_FE"));
        assert!(formatted.contains("Example"));
    }

    #[test]
    fn test_format_error_user_friendly_invalid_prefix_tap_hold() {
        let err = ParseError::InvalidPrefix {
            expected: "MD_XX".to_string(),
            got: "VK_Space".to_string(),
            context: "tap_hold hold parameter".to_string(),
        };
        let formatted = format_error_user_friendly(&err);
        assert!(formatted.contains("tap_hold hold parameter must have MD_ prefix"));
        assert!(formatted.contains("VK_Space"));
        assert!(formatted.contains("Example"));
    }

    #[test]
    fn test_format_error_user_friendly_modifier_out_of_range() {
        let err = ParseError::ModifierIdOutOfRange { got: 255, max: 254 };
        let formatted = format_error_user_friendly(&err);
        assert!(formatted.contains("255"));
        assert!(formatted.contains("MD_FE"));
        assert!(formatted.contains("Help"));
    }

    #[test]
    fn test_format_error_user_friendly_lock_out_of_range() {
        let err = ParseError::LockIdOutOfRange { got: 300, max: 254 };
        let formatted = format_error_user_friendly(&err);
        assert!(formatted.contains("300"));
        assert!(formatted.contains("LK_FE"));
        assert!(formatted.contains("Help"));
    }

    #[test]
    fn test_format_error_user_friendly_import_not_found() {
        let err = ParseError::ImportNotFound {
            path: PathBuf::from("missing.rhai"),
            searched_paths: vec![
                PathBuf::from("/path1/missing.rhai"),
                PathBuf::from("/path2/missing.rhai"),
            ],
        };
        let formatted = format_error_user_friendly(&err);
        assert!(formatted.contains("missing.rhai"));
        assert!(formatted.contains("/path1/missing.rhai"));
        assert!(formatted.contains("/path2/missing.rhai"));
        assert!(formatted.contains("Help"));
    }

    #[test]
    fn test_format_error_user_friendly_circular_import() {
        let err = ParseError::CircularImport {
            chain: vec![
                PathBuf::from("a.rhai"),
                PathBuf::from("b.rhai"),
                PathBuf::from("a.rhai"),
            ],
        };
        let formatted = format_error_user_friendly(&err);
        assert!(formatted.contains("Circular import"));
        assert!(formatted.contains("a.rhai"));
        assert!(formatted.contains("b.rhai"));
        assert!(formatted.contains("Help"));
    }

    #[test]
    fn test_format_error_user_friendly_resource_limit() {
        let err = ParseError::ResourceLimitExceeded {
            limit_type: "max_operations".to_string(),
        };
        let formatted = format_error_user_friendly(&err);
        assert!(formatted.contains("max_operations"));
        assert!(formatted.contains("Help"));
    }

    #[test]
    fn test_format_error_json_syntax_error() {
        let err = ParseError::SyntaxError {
            file: PathBuf::from("test.rhai"),
            line: 42,
            column: 10,
            message: "unexpected token".to_string(),
        };
        let json_str = format_error_json(&err);
        let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        assert_eq!(json["error_code"], "E001");
        assert_eq!(json["error_type"], "SyntaxError");
        assert_eq!(json["message"], "unexpected token");
        assert_eq!(json["file"], "test.rhai");
        assert_eq!(json["line"], 42);
        assert_eq!(json["column"], 10);
        assert!(json["suggestion"].as_str().unwrap().contains("syntax"));
    }

    #[test]
    fn test_format_error_json_invalid_prefix() {
        let err = ParseError::InvalidPrefix {
            expected: "VK_/MD_/LK_".to_string(),
            got: "XY_A".to_string(),
            context: "map output".to_string(),
        };
        let json_str = format_error_json(&err);
        let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        assert_eq!(json["error_code"], "E002");
        assert_eq!(json["error_type"], "InvalidPrefix");
        assert_eq!(json["expected"], "VK_/MD_/LK_");
        assert_eq!(json["got"], "XY_A");
        assert_eq!(json["context"], "map output");
        assert!(json["suggestion"].is_string());
    }

    #[test]
    fn test_format_error_json_modifier_out_of_range() {
        let err = ParseError::ModifierIdOutOfRange { got: 255, max: 254 };
        let json_str = format_error_json(&err);
        let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        assert_eq!(json["error_code"], "E003");
        assert_eq!(json["error_type"], "ModifierIdOutOfRange");
        assert_eq!(json["got"], 255);
        assert_eq!(json["max"], 254);
        assert_eq!(json["valid_range"], "MD_00 to MD_FE");
        assert!(json["suggestion"].as_str().unwrap().contains("FE"));
    }

    #[test]
    fn test_format_error_json_lock_out_of_range() {
        let err = ParseError::LockIdOutOfRange { got: 300, max: 254 };
        let json_str = format_error_json(&err);
        let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        assert_eq!(json["error_code"], "E004");
        assert_eq!(json["error_type"], "LockIdOutOfRange");
        assert_eq!(json["got"], 300);
        assert_eq!(json["max"], 254);
        assert_eq!(json["valid_range"], "LK_00 to LK_FE");
    }

    #[test]
    fn test_format_error_json_physical_modifier_in_md() {
        let err = ParseError::PhysicalModifierInMD {
            name: "LShift".to_string(),
        };
        let json_str = format_error_json(&err);
        let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        assert_eq!(json["error_code"], "E005");
        assert_eq!(json["error_type"], "PhysicalModifierInMD");
        assert_eq!(json["physical_modifier"], "LShift");
        assert!(json["suggestion"].as_str().unwrap().contains("MD_00"));
    }

    #[test]
    fn test_format_error_json_missing_prefix() {
        let err = ParseError::MissingPrefix {
            key: "B".to_string(),
            context: "map output".to_string(),
        };
        let json_str = format_error_json(&err);
        let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        assert_eq!(json["error_code"], "E006");
        assert_eq!(json["error_type"], "MissingPrefix");
        assert_eq!(json["key"], "B");
        assert_eq!(json["context"], "map output");
        assert!(json["suggestion"].as_str().unwrap().contains("VK_B"));
    }

    #[test]
    fn test_format_error_json_import_not_found() {
        let err = ParseError::ImportNotFound {
            path: PathBuf::from("missing.rhai"),
            searched_paths: vec![PathBuf::from("/path1/missing.rhai")],
        };
        let json_str = format_error_json(&err);
        let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        assert_eq!(json["error_code"], "E007");
        assert_eq!(json["error_type"], "ImportNotFound");
        assert_eq!(json["path"], "missing.rhai");
        assert!(json["searched_paths"].is_array());
        assert_eq!(json["searched_paths"][0], "/path1/missing.rhai");
    }

    #[test]
    fn test_format_error_json_circular_import() {
        let err = ParseError::CircularImport {
            chain: vec![
                PathBuf::from("a.rhai"),
                PathBuf::from("b.rhai"),
                PathBuf::from("a.rhai"),
            ],
        };
        let json_str = format_error_json(&err);
        let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        assert_eq!(json["error_code"], "E008");
        assert_eq!(json["error_type"], "CircularImport");
        assert!(json["import_chain"].is_array());
        assert_eq!(json["import_chain"][0], "a.rhai");
        assert_eq!(json["import_chain"][1], "b.rhai");
    }

    #[test]
    fn test_format_error_json_resource_limit() {
        let err = ParseError::ResourceLimitExceeded {
            limit_type: "max_operations".to_string(),
        };
        let json_str = format_error_json(&err);
        let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        assert_eq!(json["error_code"], "E009");
        assert_eq!(json["error_type"], "ResourceLimitExceeded");
        assert_eq!(json["limit_type"], "max_operations");
        assert!(json["suggestion"].as_str().unwrap().contains("Simplify"));
    }

    #[test]
    fn test_json_output_is_valid() {
        // Test that all error types produce valid JSON
        let errors = vec![
            ParseError::SyntaxError {
                file: PathBuf::from("test.rhai"),
                line: 1,
                column: 1,
                message: "test".to_string(),
            },
            ParseError::InvalidPrefix {
                expected: "VK_".to_string(),
                got: "A".to_string(),
                context: "test".to_string(),
            },
            ParseError::ModifierIdOutOfRange { got: 255, max: 254 },
            ParseError::LockIdOutOfRange { got: 255, max: 254 },
            ParseError::PhysicalModifierInMD {
                name: "LShift".to_string(),
            },
            ParseError::MissingPrefix {
                key: "A".to_string(),
                context: "test".to_string(),
            },
            ParseError::ImportNotFound {
                path: PathBuf::from("test.rhai"),
                searched_paths: vec![],
            },
            ParseError::CircularImport {
                chain: vec![PathBuf::from("a.rhai")],
            },
            ParseError::ResourceLimitExceeded {
                limit_type: "test".to_string(),
            },
        ];

        for error in errors {
            let json_str = format_error_json(&error);
            let result = serde_json::from_str::<serde_json::Value>(&json_str);
            assert!(
                result.is_ok(),
                "Failed to parse JSON for {:?}: {}",
                error,
                json_str
            );
        }
    }
}
