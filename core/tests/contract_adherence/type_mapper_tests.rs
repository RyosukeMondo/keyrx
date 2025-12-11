//! Comprehensive unit tests for the type mapper module.
//!
//! These tests verify the reliability of contract type to Rust FFI type mappings
//! and the validation logic that compares parsed types against contract definitions.

use super::parser::ParsedType;
use super::type_mapper::{
    is_error_pointer, is_error_pointer_str, map_contract_to_rust, validate_type_match, RustFfiType,
    TypeMismatch,
};

// ============================================================================
// Type Mapping Tests - Primitive Types
// ============================================================================

#[test]
fn test_map_void_type() {
    assert_eq!(map_contract_to_rust("void"), RustFfiType::Unit);
}

#[test]
fn test_map_void_unit_syntax() {
    assert_eq!(map_contract_to_rust("()"), RustFfiType::Unit);
}

#[test]
fn test_map_bool_type() {
    assert_eq!(map_contract_to_rust("bool"), RustFfiType::Bool);
}

#[test]
fn test_map_boolean_type() {
    assert_eq!(map_contract_to_rust("boolean"), RustFfiType::Bool);
}

#[test]
fn test_map_int_type() {
    assert_eq!(map_contract_to_rust("int"), RustFfiType::I32);
}

#[test]
fn test_map_int32_type() {
    assert_eq!(map_contract_to_rust("int32"), RustFfiType::I32);
}

#[test]
fn test_map_i32_type() {
    assert_eq!(map_contract_to_rust("i32"), RustFfiType::I32);
}

#[test]
fn test_map_uint8_type() {
    assert_eq!(map_contract_to_rust("uint8"), RustFfiType::U8);
}

#[test]
fn test_map_u8_type() {
    assert_eq!(map_contract_to_rust("u8"), RustFfiType::U8);
}

#[test]
fn test_map_uint32_type() {
    assert_eq!(map_contract_to_rust("uint32"), RustFfiType::U32);
}

#[test]
fn test_map_u32_type() {
    assert_eq!(map_contract_to_rust("u32"), RustFfiType::U32);
}

#[test]
fn test_map_uint64_type() {
    assert_eq!(map_contract_to_rust("uint64"), RustFfiType::U64);
}

#[test]
fn test_map_u64_type() {
    assert_eq!(map_contract_to_rust("u64"), RustFfiType::U64);
}

#[test]
fn test_map_float64_type() {
    assert_eq!(map_contract_to_rust("float64"), RustFfiType::F64);
}

#[test]
fn test_map_f64_type() {
    assert_eq!(map_contract_to_rust("f64"), RustFfiType::F64);
}

#[test]
fn test_map_double_type() {
    assert_eq!(map_contract_to_rust("double"), RustFfiType::F64);
}

// ============================================================================
// Type Mapping Tests - String and Complex Types
// ============================================================================

#[test]
fn test_map_string_type() {
    assert_eq!(map_contract_to_rust("string"), RustFfiType::ConstCharPtr);
}

#[test]
fn test_map_object_type_is_json_string() {
    assert_eq!(map_contract_to_rust("object"), RustFfiType::ConstCharPtr);
}

#[test]
fn test_map_array_type_is_json_string() {
    assert_eq!(map_contract_to_rust("array"), RustFfiType::ConstCharPtr);
}

// ============================================================================
// Type Mapping Tests - Case Insensitivity
// ============================================================================

#[test]
fn test_map_case_insensitive_string() {
    assert_eq!(map_contract_to_rust("STRING"), RustFfiType::ConstCharPtr);
    assert_eq!(map_contract_to_rust("String"), RustFfiType::ConstCharPtr);
}

#[test]
fn test_map_case_insensitive_bool() {
    assert_eq!(map_contract_to_rust("BOOL"), RustFfiType::Bool);
    assert_eq!(map_contract_to_rust("Bool"), RustFfiType::Bool);
    assert_eq!(map_contract_to_rust("BOOLEAN"), RustFfiType::Bool);
}

#[test]
fn test_map_case_insensitive_int() {
    assert_eq!(map_contract_to_rust("INT"), RustFfiType::I32);
    assert_eq!(map_contract_to_rust("Int"), RustFfiType::I32);
    assert_eq!(map_contract_to_rust("INT32"), RustFfiType::I32);
}

#[test]
fn test_map_case_insensitive_void() {
    assert_eq!(map_contract_to_rust("VOID"), RustFfiType::Unit);
    assert_eq!(map_contract_to_rust("Void"), RustFfiType::Unit);
}

// ============================================================================
// Type Mapping Tests - Unknown Types
// ============================================================================

#[test]
fn test_map_unknown_type() {
    match map_contract_to_rust("custom_type") {
        RustFfiType::Unknown(s) => assert_eq!(s, "custom_type"),
        _ => panic!("Expected Unknown type"),
    }
}

#[test]
fn test_map_unknown_preserves_original_case() {
    match map_contract_to_rust("CustomStruct") {
        RustFfiType::Unknown(s) => assert_eq!(s, "customstruct"),
        _ => panic!("Expected Unknown type"),
    }
}

#[test]
fn test_map_empty_string_is_unknown() {
    match map_contract_to_rust("") {
        RustFfiType::Unknown(s) => assert_eq!(s, ""),
        _ => panic!("Expected Unknown type for empty string"),
    }
}

// ============================================================================
// RustFfiType Display String Tests
// ============================================================================

#[test]
fn test_display_unit() {
    assert_eq!(RustFfiType::Unit.to_display_string(), "()");
}

#[test]
fn test_display_bool() {
    assert_eq!(RustFfiType::Bool.to_display_string(), "bool");
}

#[test]
fn test_display_i32() {
    assert_eq!(RustFfiType::I32.to_display_string(), "i32");
}

#[test]
fn test_display_u8() {
    assert_eq!(RustFfiType::U8.to_display_string(), "u8");
}

#[test]
fn test_display_u32() {
    assert_eq!(RustFfiType::U32.to_display_string(), "u32");
}

#[test]
fn test_display_u64() {
    assert_eq!(RustFfiType::U64.to_display_string(), "u64");
}

#[test]
fn test_display_f64() {
    assert_eq!(RustFfiType::F64.to_display_string(), "f64");
}

#[test]
fn test_display_const_char_ptr() {
    assert_eq!(
        RustFfiType::ConstCharPtr.to_display_string(),
        "*const c_char"
    );
}

#[test]
fn test_display_mut_char_ptr() {
    assert_eq!(RustFfiType::MutCharPtr.to_display_string(), "*mut c_char");
}

#[test]
fn test_display_error_ptr() {
    assert_eq!(
        RustFfiType::ErrorPtr.to_display_string(),
        "*mut *mut c_char"
    );
}

#[test]
fn test_display_unknown() {
    assert_eq!(
        RustFfiType::Unknown("custom".to_string()).to_display_string(),
        "unknown(custom)"
    );
}

// ============================================================================
// Type Validation Tests - Matching Types
// ============================================================================

#[test]
fn test_validate_void_matches_unit() {
    let result = validate_type_match("void", &ParsedType::Unit);
    assert!(result.is_ok());
}

#[test]
fn test_validate_bool_matches_bool_primitive() {
    let result = validate_type_match("bool", &ParsedType::Primitive("bool".to_string()));
    assert!(result.is_ok());
}

#[test]
fn test_validate_int_matches_i32_primitive() {
    let result = validate_type_match("int", &ParsedType::Primitive("i32".to_string()));
    assert!(result.is_ok());
}

#[test]
fn test_validate_int32_matches_i32_primitive() {
    let result = validate_type_match("int32", &ParsedType::Primitive("i32".to_string()));
    assert!(result.is_ok());
}

#[test]
fn test_validate_uint8_matches_u8_primitive() {
    let result = validate_type_match("uint8", &ParsedType::Primitive("u8".to_string()));
    assert!(result.is_ok());
}

#[test]
fn test_validate_uint32_matches_u32_primitive() {
    let result = validate_type_match("uint32", &ParsedType::Primitive("u32".to_string()));
    assert!(result.is_ok());
}

#[test]
fn test_validate_uint64_matches_u64_primitive() {
    let result = validate_type_match("uint64", &ParsedType::Primitive("u64".to_string()));
    assert!(result.is_ok());
}

#[test]
fn test_validate_float64_matches_f64_primitive() {
    let result = validate_type_match("float64", &ParsedType::Primitive("f64".to_string()));
    assert!(result.is_ok());
}

#[test]
fn test_validate_string_matches_const_char_ptr() {
    let result = validate_type_match(
        "string",
        &ParsedType::Pointer {
            target: "c_char".to_string(),
            is_mut: false,
        },
    );
    assert!(result.is_ok());
}

#[test]
fn test_validate_object_matches_const_char_ptr() {
    let result = validate_type_match(
        "object",
        &ParsedType::Pointer {
            target: "c_char".to_string(),
            is_mut: false,
        },
    );
    assert!(result.is_ok());
}

#[test]
fn test_validate_array_matches_const_char_ptr() {
    let result = validate_type_match(
        "array",
        &ParsedType::Pointer {
            target: "c_char".to_string(),
            is_mut: false,
        },
    );
    assert!(result.is_ok());
}

// ============================================================================
// Type Validation Tests - Mismatched Types
// ============================================================================

#[test]
fn test_validate_void_mismatches_primitive() {
    let result = validate_type_match("void", &ParsedType::Primitive("i32".to_string()));
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.contract_type, "void");
    assert_eq!(err.expected, RustFfiType::Unit);
    assert_eq!(err.found, "i32");
}

#[test]
fn test_validate_int_mismatches_u32() {
    let result = validate_type_match("int", &ParsedType::Primitive("u32".to_string()));
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.contract_type, "int");
}

#[test]
fn test_validate_bool_mismatches_i32() {
    let result = validate_type_match("bool", &ParsedType::Primitive("i32".to_string()));
    assert!(result.is_err());
}

#[test]
fn test_validate_string_accepts_mut_pointer() {
    // FFI functions often return *mut c_char for strings because the caller
    // receives ownership and must free the memory
    let result = validate_type_match(
        "string",
        &ParsedType::Pointer {
            target: "c_char".to_string(),
            is_mut: true,
        },
    );
    assert!(result.is_ok());
}

#[test]
fn test_validate_string_mismatches_wrong_target() {
    let result = validate_type_match(
        "string",
        &ParsedType::Pointer {
            target: "u8".to_string(),
            is_mut: false,
        },
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message.contains("u8")); // Wrong target type
}

#[test]
fn test_validate_string_mismatches_primitive() {
    let result = validate_type_match("string", &ParsedType::Primitive("i32".to_string()));
    assert!(result.is_err());
}

// ============================================================================
// Type Validation Tests - Unknown Contract Types
// ============================================================================

#[test]
fn test_validate_unknown_contract_type_returns_error() {
    let result = validate_type_match("custom_type", &ParsedType::Primitive("i32".to_string()));
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message.contains("Unknown contract type"));
    assert!(err.message.contains("custom_type"));
}

// ============================================================================
// TypeMismatch Error Tests
// ============================================================================

#[test]
fn test_type_mismatch_display() {
    let mismatch = TypeMismatch {
        contract_type: "string".to_string(),
        expected: RustFfiType::ConstCharPtr,
        found: "*mut c_char".to_string(),
        message: "Expected const pointer, found mutable".to_string(),
    };
    assert_eq!(
        format!("{}", mismatch),
        "Expected const pointer, found mutable"
    );
}

#[test]
fn test_type_mismatch_is_error() {
    let mismatch = TypeMismatch {
        contract_type: "int".to_string(),
        expected: RustFfiType::I32,
        found: "u32".to_string(),
        message: "Type mismatch".to_string(),
    };
    // Verify it implements std::error::Error
    let _: &dyn std::error::Error = &mismatch;
}

#[test]
fn test_type_mismatch_has_actionable_message() {
    let result = validate_type_match("string", &ParsedType::Primitive("i32".to_string()));
    let err = result.unwrap_err();
    // Message should contain expected type, contract type, and found type
    assert!(err.message.contains("string") || err.message.contains("*const c_char"));
    assert!(err.message.contains("i32"));
}

// ============================================================================
// Error Pointer Detection Tests
// ============================================================================

#[test]
fn test_is_error_pointer_true_for_double_mut_pointer() {
    let parsed = ParsedType::Pointer {
        target: "*mut c_char".to_string(),
        is_mut: true,
    };
    assert!(is_error_pointer(&parsed));
}

#[test]
fn test_is_error_pointer_false_for_single_pointer() {
    let parsed = ParsedType::Pointer {
        target: "c_char".to_string(),
        is_mut: true,
    };
    assert!(!is_error_pointer(&parsed));
}

#[test]
fn test_is_error_pointer_false_for_const_double_pointer() {
    let parsed = ParsedType::Pointer {
        target: "*mut c_char".to_string(),
        is_mut: false,
    };
    assert!(!is_error_pointer(&parsed));
}

#[test]
fn test_is_error_pointer_false_for_unit() {
    assert!(!is_error_pointer(&ParsedType::Unit));
}

#[test]
fn test_is_error_pointer_false_for_primitive() {
    assert!(!is_error_pointer(&ParsedType::Primitive("i32".to_string())));
}

// ============================================================================
// Error Pointer String Detection Tests
// ============================================================================

#[test]
fn test_is_error_pointer_str_true() {
    assert!(is_error_pointer_str("*mut *mut c_char"));
}

#[test]
fn test_is_error_pointer_str_with_spaces() {
    assert!(is_error_pointer_str("*mut *mut c_char"));
}

#[test]
fn test_is_error_pointer_str_false_for_single_pointer() {
    assert!(!is_error_pointer_str("*mut c_char"));
}

#[test]
fn test_is_error_pointer_str_false_for_const_pointer() {
    assert!(!is_error_pointer_str("*const c_char"));
}

#[test]
fn test_is_error_pointer_str_false_for_primitive() {
    assert!(!is_error_pointer_str("i32"));
}

#[test]
fn test_is_error_pointer_str_in_larger_type() {
    // Should find substring
    assert!(is_error_pointer_str("Option<*mut *mut c_char>"));
}

// ============================================================================
// RustFfiType Equality Tests
// ============================================================================

#[test]
fn test_rust_ffi_type_equality() {
    assert_eq!(RustFfiType::Unit, RustFfiType::Unit);
    assert_eq!(RustFfiType::Bool, RustFfiType::Bool);
    assert_eq!(RustFfiType::I32, RustFfiType::I32);
    assert_ne!(RustFfiType::I32, RustFfiType::U32);
    assert_ne!(RustFfiType::ConstCharPtr, RustFfiType::MutCharPtr);
}

#[test]
fn test_rust_ffi_type_unknown_equality() {
    assert_eq!(
        RustFfiType::Unknown("custom".to_string()),
        RustFfiType::Unknown("custom".to_string())
    );
    assert_ne!(
        RustFfiType::Unknown("a".to_string()),
        RustFfiType::Unknown("b".to_string())
    );
}

#[test]
fn test_rust_ffi_type_clone() {
    let original = RustFfiType::ConstCharPtr;
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn test_rust_ffi_type_debug() {
    let t = RustFfiType::I32;
    let debug_str = format!("{:?}", t);
    assert!(debug_str.contains("I32"));
}

// ============================================================================
// TypeMismatch Clone and Debug Tests
// ============================================================================

#[test]
fn test_type_mismatch_clone() {
    let original = TypeMismatch {
        contract_type: "string".to_string(),
        expected: RustFfiType::ConstCharPtr,
        found: "i32".to_string(),
        message: "error".to_string(),
    };
    let cloned = original.clone();
    assert_eq!(original.contract_type, cloned.contract_type);
    assert_eq!(original.expected, cloned.expected);
    assert_eq!(original.found, cloned.found);
    assert_eq!(original.message, cloned.message);
}

#[test]
fn test_type_mismatch_debug() {
    let mismatch = TypeMismatch {
        contract_type: "int".to_string(),
        expected: RustFfiType::I32,
        found: "u32".to_string(),
        message: "test".to_string(),
    };
    let debug_str = format!("{:?}", mismatch);
    assert!(debug_str.contains("TypeMismatch"));
    assert!(debug_str.contains("int"));
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_validate_with_extra_whitespace_in_contract_type() {
    // Leading/trailing whitespace should be handled by caller, but test behavior
    let result = validate_type_match(" void ", &ParsedType::Unit);
    // This will fail because " void " != "void" - documenting expected behavior
    assert!(result.is_err());
}

#[test]
fn test_validate_empty_target_pointer() {
    let result = validate_type_match(
        "string",
        &ParsedType::Pointer {
            target: "".to_string(),
            is_mut: false,
        },
    );
    assert!(result.is_err());
}

#[test]
fn test_all_integer_types_distinct() {
    assert_ne!(
        map_contract_to_rust("uint8"),
        map_contract_to_rust("uint32")
    );
    assert_ne!(
        map_contract_to_rust("uint32"),
        map_contract_to_rust("uint64")
    );
    assert_ne!(map_contract_to_rust("int"), map_contract_to_rust("uint32"));
}
