//! Contract type to Dart type mapping
//!
//! This module converts FFI contract types (from JSON contracts) to their
//! corresponding Dart FFI types and native Dart types.

use crate::types::DartFfiType;

/// Error type for mapping failures
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeMappingError {
    pub contract_type: String,
    pub message: String,
}

impl std::fmt::Display for TypeMappingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cannot map contract type '{}': {}",
            self.contract_type, self.message
        )
    }
}

impl std::error::Error for TypeMappingError {}

/// Maps a contract type string to its Dart FFI type representation.
///
/// # Arguments
/// * `contract_type` - The type name from the JSON contract (e.g., "string", "int32")
///
/// # Returns
/// * `Ok(DartFfiType)` - The corresponding FFI type
/// * `Err(TypeMappingError)` - If the type is unknown or cannot be mapped
pub fn map_to_dart_ffi_type(contract_type: &str) -> Result<DartFfiType, TypeMappingError> {
    let normalized = contract_type.trim().to_lowercase();

    match normalized.as_str() {
        // Void
        "void" => Ok(DartFfiType::Void),

        // Boolean
        "bool" | "boolean" => Ok(DartFfiType::Bool),

        // Signed integers
        "int8" | "i8" => Ok(DartFfiType::Int8),
        "int16" | "i16" => Ok(DartFfiType::Int16),
        "int32" | "i32" | "int" => Ok(DartFfiType::Int32),
        "int64" | "i64" => Ok(DartFfiType::Int64),

        // Unsigned integers
        "uint8" | "u8" | "byte" => Ok(DartFfiType::Uint8),
        "uint16" | "u16" => Ok(DartFfiType::Uint16),
        "uint32" | "u32" => Ok(DartFfiType::Uint32),
        "uint64" | "u64" => Ok(DartFfiType::Uint64),

        // Floating point
        "float" | "float32" | "f32" => Ok(DartFfiType::Float),
        "double" | "float64" | "f64" => Ok(DartFfiType::Double),

        // String - passed as UTF-8 pointer
        "string" | "str" => Ok(DartFfiType::PointerUtf8),

        // Complex types (object, array) are serialized as JSON strings
        "object" | "array" => Ok(DartFfiType::PointerUtf8),

        // Unknown type
        _ => Err(TypeMappingError {
            contract_type: contract_type.to_string(),
            message: "Unknown or unsupported contract type".to_string(),
        }),
    }
}

/// Maps a contract type string to its native Dart type string.
///
/// This returns the Dart type used in wrapper functions (high-level API),
/// not the FFI type used in native function signatures.
///
/// # Arguments
/// * `contract_type` - The type name from the JSON contract
///
/// # Returns
/// * `Ok(String)` - The native Dart type string (e.g., "int", "String", "bool")
/// * `Err(TypeMappingError)` - If the type is unknown
pub fn map_to_dart_native_type(contract_type: &str) -> Result<String, TypeMappingError> {
    let normalized = contract_type.trim().to_lowercase();

    let dart_type = match normalized.as_str() {
        // Void
        "void" => "void",

        // Boolean
        "bool" | "boolean" => "bool",

        // All integers map to Dart's int type
        "int8" | "i8" | "int16" | "i16" | "int32" | "i32" | "int" |
        "int64" | "i64" | "uint8" | "u8" | "byte" | "uint16" | "u16" |
        "uint32" | "u32" | "uint64" | "u64" => "int",

        // Floating point maps to double
        "float" | "float32" | "f32" | "double" | "float64" | "f64" => "double",

        // String stays as String
        "string" | "str" => "String",

        // Complex types map to dynamic (parsed from JSON)
        "object" => "Map<String, dynamic>",
        "array" => "List<dynamic>",

        // Unknown type
        _ => {
            return Err(TypeMappingError {
                contract_type: contract_type.to_string(),
                message: "Unknown or unsupported contract type".to_string(),
            });
        }
    };

    Ok(dart_type.to_string())
}

/// Check if a contract type represents a complex type (object or array)
/// that needs JSON serialization for FFI transport.
pub fn is_complex_type(contract_type: &str) -> bool {
    let normalized = contract_type.trim().to_lowercase();
    matches!(normalized.as_str(), "object" | "array")
}

/// Check if a contract type represents a nullable type.
/// Convention: types ending with "?" are nullable.
pub fn is_nullable(contract_type: &str) -> bool {
    contract_type.trim().ends_with('?')
}

/// Extract the base type from a potentially nullable type.
/// E.g., "string?" -> "string"
pub fn base_type(contract_type: &str) -> &str {
    contract_type.trim().trim_end_matches('?')
}

#[cfg(test)]
mod tests {
    use super::*;

    // FFI type mapping tests
    #[test]
    fn test_map_void_to_ffi() {
        assert_eq!(map_to_dart_ffi_type("void").unwrap(), DartFfiType::Void);
    }

    #[test]
    fn test_map_bool_to_ffi() {
        assert_eq!(map_to_dart_ffi_type("bool").unwrap(), DartFfiType::Bool);
        assert_eq!(map_to_dart_ffi_type("boolean").unwrap(), DartFfiType::Bool);
    }

    #[test]
    fn test_map_signed_integers_to_ffi() {
        assert_eq!(map_to_dart_ffi_type("int8").unwrap(), DartFfiType::Int8);
        assert_eq!(map_to_dart_ffi_type("i8").unwrap(), DartFfiType::Int8);
        assert_eq!(map_to_dart_ffi_type("int16").unwrap(), DartFfiType::Int16);
        assert_eq!(map_to_dart_ffi_type("int32").unwrap(), DartFfiType::Int32);
        assert_eq!(map_to_dart_ffi_type("int").unwrap(), DartFfiType::Int32);
        assert_eq!(map_to_dart_ffi_type("int64").unwrap(), DartFfiType::Int64);
    }

    #[test]
    fn test_map_unsigned_integers_to_ffi() {
        assert_eq!(map_to_dart_ffi_type("uint8").unwrap(), DartFfiType::Uint8);
        assert_eq!(map_to_dart_ffi_type("byte").unwrap(), DartFfiType::Uint8);
        assert_eq!(map_to_dart_ffi_type("uint16").unwrap(), DartFfiType::Uint16);
        assert_eq!(map_to_dart_ffi_type("uint32").unwrap(), DartFfiType::Uint32);
        assert_eq!(map_to_dart_ffi_type("uint64").unwrap(), DartFfiType::Uint64);
    }

    #[test]
    fn test_map_floats_to_ffi() {
        assert_eq!(map_to_dart_ffi_type("float").unwrap(), DartFfiType::Float);
        assert_eq!(map_to_dart_ffi_type("float32").unwrap(), DartFfiType::Float);
        assert_eq!(map_to_dart_ffi_type("double").unwrap(), DartFfiType::Double);
        assert_eq!(map_to_dart_ffi_type("float64").unwrap(), DartFfiType::Double);
    }

    #[test]
    fn test_map_string_to_ffi() {
        assert_eq!(
            map_to_dart_ffi_type("string").unwrap(),
            DartFfiType::PointerUtf8
        );
        assert_eq!(
            map_to_dart_ffi_type("str").unwrap(),
            DartFfiType::PointerUtf8
        );
    }

    #[test]
    fn test_map_complex_types_to_ffi() {
        // Complex types are serialized as JSON strings
        assert_eq!(
            map_to_dart_ffi_type("object").unwrap(),
            DartFfiType::PointerUtf8
        );
        assert_eq!(
            map_to_dart_ffi_type("array").unwrap(),
            DartFfiType::PointerUtf8
        );
    }

    #[test]
    fn test_map_unknown_type_to_ffi_fails() {
        let result = map_to_dart_ffi_type("unknown_type");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.contract_type, "unknown_type");
    }

    #[test]
    fn test_case_insensitive_mapping() {
        assert_eq!(map_to_dart_ffi_type("STRING").unwrap(), DartFfiType::PointerUtf8);
        assert_eq!(map_to_dart_ffi_type("Int32").unwrap(), DartFfiType::Int32);
        assert_eq!(map_to_dart_ffi_type("BOOL").unwrap(), DartFfiType::Bool);
    }

    #[test]
    fn test_whitespace_trimming() {
        assert_eq!(map_to_dart_ffi_type("  string  ").unwrap(), DartFfiType::PointerUtf8);
        assert_eq!(map_to_dart_ffi_type("\tint32\n").unwrap(), DartFfiType::Int32);
    }

    // Native type mapping tests
    #[test]
    fn test_map_void_to_native() {
        assert_eq!(map_to_dart_native_type("void").unwrap(), "void");
    }

    #[test]
    fn test_map_bool_to_native() {
        assert_eq!(map_to_dart_native_type("bool").unwrap(), "bool");
    }

    #[test]
    fn test_map_integers_to_native() {
        // All integers map to Dart's int type
        assert_eq!(map_to_dart_native_type("int8").unwrap(), "int");
        assert_eq!(map_to_dart_native_type("int32").unwrap(), "int");
        assert_eq!(map_to_dart_native_type("uint64").unwrap(), "int");
    }

    #[test]
    fn test_map_floats_to_native() {
        assert_eq!(map_to_dart_native_type("float").unwrap(), "double");
        assert_eq!(map_to_dart_native_type("double").unwrap(), "double");
    }

    #[test]
    fn test_map_string_to_native() {
        assert_eq!(map_to_dart_native_type("string").unwrap(), "String");
    }

    #[test]
    fn test_map_complex_to_native() {
        assert_eq!(
            map_to_dart_native_type("object").unwrap(),
            "Map<String, dynamic>"
        );
        assert_eq!(
            map_to_dart_native_type("array").unwrap(),
            "List<dynamic>"
        );
    }

    // Utility function tests
    #[test]
    fn test_is_complex_type() {
        assert!(is_complex_type("object"));
        assert!(is_complex_type("array"));
        assert!(is_complex_type("OBJECT"));
        assert!(!is_complex_type("string"));
        assert!(!is_complex_type("int32"));
    }

    #[test]
    fn test_is_nullable() {
        assert!(is_nullable("string?"));
        assert!(is_nullable("int32?"));
        assert!(!is_nullable("string"));
        assert!(!is_nullable("int32"));
    }

    #[test]
    fn test_base_type() {
        assert_eq!(base_type("string?"), "string");
        assert_eq!(base_type("int32?"), "int32");
        assert_eq!(base_type("string"), "string");
        assert_eq!(base_type("  string?  "), "string");
    }
}
