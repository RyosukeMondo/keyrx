//! Type Mapping Module for FFI Contract Validation
//!
//! Maps contract type strings to expected Rust FFI types and validates
//! that parsed Rust types match their contract definitions.

use super::parser::ParsedType;

/// Represents the expected Rust FFI type for a contract type.
#[derive(Debug, Clone, PartialEq)]
pub enum RustFfiType {
    /// Unit type `()`
    Unit,
    /// Boolean type
    Bool,
    /// Signed 32-bit integer
    I32,
    /// Unsigned 8-bit integer
    U8,
    /// Unsigned 32-bit integer
    U32,
    /// Unsigned 64-bit integer
    U64,
    /// 64-bit floating point
    F64,
    /// Const pointer to c_char (for strings and JSON)
    ConstCharPtr,
    /// Mutable pointer to c_char
    MutCharPtr,
    /// Double mutable pointer to c_char (error pointer)
    ErrorPtr,
    /// Unknown or unmapped type
    Unknown(String),
}

impl RustFfiType {
    /// Returns a human-readable string representation of the type.
    pub fn to_display_string(&self) -> String {
        match self {
            RustFfiType::Unit => "()".to_string(),
            RustFfiType::Bool => "bool".to_string(),
            RustFfiType::I32 => "i32".to_string(),
            RustFfiType::U8 => "u8".to_string(),
            RustFfiType::U32 => "u32".to_string(),
            RustFfiType::U64 => "u64".to_string(),
            RustFfiType::F64 => "f64".to_string(),
            RustFfiType::ConstCharPtr => "*const c_char".to_string(),
            RustFfiType::MutCharPtr => "*mut c_char".to_string(),
            RustFfiType::ErrorPtr => "*mut *mut c_char".to_string(),
            RustFfiType::Unknown(s) => format!("unknown({})", s),
        }
    }
}

/// Error type for type mapping failures.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeMismatch {
    /// The contract type that was being mapped
    pub contract_type: String,
    /// The expected Rust FFI type
    pub expected: RustFfiType,
    /// The actual Rust type found
    pub found: String,
    /// Human-readable description of the mismatch
    pub message: String,
}

impl std::fmt::Display for TypeMismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for TypeMismatch {}

/// Maps a contract type string to the expected Rust FFI type.
///
/// # Type Mapping Rules
/// | Contract Type | Rust FFI Type |
/// |--------------|---------------|
/// | `string` | `*const c_char` |
/// | `int`, `int32` | `i32` |
/// | `uint8` | `u8` |
/// | `uint32` | `u32` |
/// | `uint64` | `u64` |
/// | `float64` | `f64` |
/// | `bool` | `bool` |
/// | `void` | `()` |
/// | `object` | `*const c_char` (JSON) |
/// | `array` | `*const c_char` (JSON) |
pub fn map_contract_to_rust(contract_type: &str) -> RustFfiType {
    match contract_type.to_lowercase().as_str() {
        // Primitive types
        "void" | "()" => RustFfiType::Unit,
        "bool" | "boolean" => RustFfiType::Bool,
        "int" | "int32" | "i32" => RustFfiType::I32,
        "uint8" | "u8" => RustFfiType::U8,
        "uint32" | "u32" => RustFfiType::U32,
        "uint64" | "u64" => RustFfiType::U64,
        "float64" | "f64" | "double" => RustFfiType::F64,

        // String type - passed as const char pointer
        "string" => RustFfiType::ConstCharPtr,

        // Complex types are JSON-serialized as strings
        "object" | "array" => RustFfiType::ConstCharPtr,

        // Unknown type
        other => RustFfiType::Unknown(other.to_string()),
    }
}

/// Validates that a parsed Rust type matches the expected contract type.
///
/// Returns `Ok(())` if the types match, or `Err(TypeMismatch)` with details.
pub fn validate_type_match(
    contract_type: &str,
    rust_type: &ParsedType,
) -> Result<(), TypeMismatch> {
    let expected = map_contract_to_rust(contract_type);

    match (&expected, rust_type) {
        // Unit type matches
        (RustFfiType::Unit, ParsedType::Unit) => Ok(()),

        // Bool matches
        (RustFfiType::Bool, ParsedType::Primitive(s)) if s == "bool" => Ok(()),

        // Integer types match
        (RustFfiType::I32, ParsedType::Primitive(s)) if s == "i32" => Ok(()),
        (RustFfiType::U8, ParsedType::Primitive(s)) if s == "u8" => Ok(()),
        (RustFfiType::U32, ParsedType::Primitive(s)) if s == "u32" => Ok(()),
        (RustFfiType::U64, ParsedType::Primitive(s)) if s == "u64" => Ok(()),

        // Float types match
        (RustFfiType::F64, ParsedType::Primitive(s)) if s == "f64" => Ok(()),

        // Const char pointer matches (for string, object, array)
        (RustFfiType::ConstCharPtr, ParsedType::Pointer { target, is_mut }) => {
            if target == "c_char" && !is_mut {
                Ok(())
            } else if *is_mut {
                Err(TypeMismatch {
                    contract_type: contract_type.to_string(),
                    expected: expected.clone(),
                    found: rust_type.to_type_string(),
                    message: format!(
                        "Expected const pointer for '{}', found mutable pointer",
                        contract_type
                    ),
                })
            } else {
                Err(TypeMismatch {
                    contract_type: contract_type.to_string(),
                    expected: expected.clone(),
                    found: rust_type.to_type_string(),
                    message: format!(
                        "Expected *const c_char for '{}', found *const {}",
                        contract_type, target
                    ),
                })
            }
        }

        // Mutable char pointer matches
        (RustFfiType::MutCharPtr, ParsedType::Pointer { target, is_mut }) => {
            if target == "c_char" && *is_mut {
                Ok(())
            } else {
                Err(TypeMismatch {
                    contract_type: contract_type.to_string(),
                    expected: expected.clone(),
                    found: rust_type.to_type_string(),
                    message: format!("Expected *mut c_char, found {}", rust_type.to_type_string()),
                })
            }
        }

        // Unknown type - warn but don't fail
        (RustFfiType::Unknown(unknown), _) => Err(TypeMismatch {
            contract_type: contract_type.to_string(),
            expected: expected.clone(),
            found: rust_type.to_type_string(),
            message: format!("Unknown contract type '{}', cannot validate", unknown),
        }),

        // Type mismatch
        _ => Err(TypeMismatch {
            contract_type: contract_type.to_string(),
            expected: expected.clone(),
            found: rust_type.to_type_string(),
            message: format!(
                "Type mismatch: expected {} for '{}', found {}",
                expected.to_display_string(),
                contract_type,
                rust_type.to_type_string()
            ),
        }),
    }
}

/// Checks if a parsed type represents an error pointer (*mut *mut c_char).
pub fn is_error_pointer(rust_type: &ParsedType) -> bool {
    matches!(
        rust_type,
        ParsedType::Pointer { target, is_mut: true } if target == "*mut c_char"
    )
}

/// Checks if a parameter type string represents an error pointer.
pub fn is_error_pointer_str(rust_type: &str) -> bool {
    rust_type.contains("*mut *mut c_char")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_void_type() {
        assert_eq!(map_contract_to_rust("void"), RustFfiType::Unit);
        assert_eq!(map_contract_to_rust("()"), RustFfiType::Unit);
    }

    #[test]
    fn test_map_bool_type() {
        assert_eq!(map_contract_to_rust("bool"), RustFfiType::Bool);
        assert_eq!(map_contract_to_rust("boolean"), RustFfiType::Bool);
    }

    #[test]
    fn test_map_integer_types() {
        assert_eq!(map_contract_to_rust("int"), RustFfiType::I32);
        assert_eq!(map_contract_to_rust("int32"), RustFfiType::I32);
        assert_eq!(map_contract_to_rust("i32"), RustFfiType::I32);
        assert_eq!(map_contract_to_rust("uint8"), RustFfiType::U8);
        assert_eq!(map_contract_to_rust("uint32"), RustFfiType::U32);
        assert_eq!(map_contract_to_rust("uint64"), RustFfiType::U64);
    }

    #[test]
    fn test_map_float_type() {
        assert_eq!(map_contract_to_rust("float64"), RustFfiType::F64);
        assert_eq!(map_contract_to_rust("f64"), RustFfiType::F64);
        assert_eq!(map_contract_to_rust("double"), RustFfiType::F64);
    }

    #[test]
    fn test_map_string_type() {
        assert_eq!(map_contract_to_rust("string"), RustFfiType::ConstCharPtr);
    }

    #[test]
    fn test_map_complex_types() {
        assert_eq!(map_contract_to_rust("object"), RustFfiType::ConstCharPtr);
        assert_eq!(map_contract_to_rust("array"), RustFfiType::ConstCharPtr);
    }

    #[test]
    fn test_map_unknown_type() {
        assert_eq!(
            map_contract_to_rust("custom_type"),
            RustFfiType::Unknown("custom_type".to_string())
        );
    }

    #[test]
    fn test_case_insensitivity() {
        assert_eq!(map_contract_to_rust("STRING"), RustFfiType::ConstCharPtr);
        assert_eq!(map_contract_to_rust("Bool"), RustFfiType::Bool);
        assert_eq!(map_contract_to_rust("INT32"), RustFfiType::I32);
    }

    #[test]
    fn test_rust_ffi_type_display() {
        assert_eq!(RustFfiType::Unit.to_display_string(), "()");
        assert_eq!(RustFfiType::Bool.to_display_string(), "bool");
        assert_eq!(RustFfiType::I32.to_display_string(), "i32");
        assert_eq!(
            RustFfiType::ConstCharPtr.to_display_string(),
            "*const c_char"
        );
        assert_eq!(
            RustFfiType::ErrorPtr.to_display_string(),
            "*mut *mut c_char"
        );
    }
}
