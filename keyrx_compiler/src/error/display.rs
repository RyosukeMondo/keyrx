use crate::error::formatting::hex_encode;
use crate::error::types::{DeserializeError, ParseError, SerializeError};
use std::error::Error;
use std::fmt;

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::SyntaxError {
                file,
                line,
                column,
                message,
                import_chain: _,
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
                import_chain: _,
            } => write!(
                f,
                "Invalid prefix: expected {}, got '{}' (context: {})",
                expected, got, context
            ),

            ParseError::ModifierIdOutOfRange {
                got,
                max,
                import_chain: _,
            } => write!(
                f,
                "Modifier ID out of range: {} (valid range: 00-{:02X})",
                got, max
            ),

            ParseError::LockIdOutOfRange {
                got,
                max,
                import_chain: _,
            } => write!(
                f,
                "Lock ID out of range: {} (valid range: 00-{:02X})",
                got, max
            ),

            ParseError::PhysicalModifierInMD {
                name,
                import_chain: _,
            } => write!(
                f,
                "Physical modifier name '{}' cannot be used with MD_ prefix. \
                 Use MD_00 through MD_FE for custom modifiers.",
                name
            ),

            ParseError::MissingPrefix {
                key,
                context,
                import_chain: _,
            } => write!(
                f,
                "Missing prefix for key '{}' (context: {}). \
                 Use VK_ for virtual keys, MD_ for modifiers, LK_ for locks.",
                key, context
            ),

            ParseError::ImportNotFound {
                path,
                searched_paths,
                import_chain: _,
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

            ParseError::ResourceLimitExceeded {
                limit_type,
                import_chain: _,
            } => write!(f, "Resource limit exceeded: {}", limit_type),
        }
    }
}

impl Error for ParseError {}

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
