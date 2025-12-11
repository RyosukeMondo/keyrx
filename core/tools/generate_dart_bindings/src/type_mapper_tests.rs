//! Comprehensive unit tests for type mapper
//!
//! This module tests all type mappings from contract types to Dart FFI types
//! and native Dart types, including nullable type handling.

use crate::type_mapper::{
    base_type, is_complex_type, is_nullable, map_to_dart_ffi_type, map_to_dart_native_type,
    TypeMappingError,
};
use crate::types::DartFfiType;

// =============================================================================
// FFI Type Mapping Tests
// =============================================================================

mod ffi_type_mapping {
    use super::*;

    #[test]
    fn void_maps_correctly() {
        assert_eq!(map_to_dart_ffi_type("void").unwrap(), DartFfiType::Void);
    }

    #[test]
    fn bool_variants_map_correctly() {
        assert_eq!(map_to_dart_ffi_type("bool").unwrap(), DartFfiType::Bool);
        assert_eq!(map_to_dart_ffi_type("boolean").unwrap(), DartFfiType::Bool);
    }

    mod signed_integers {
        use super::*;

        #[test]
        fn int8_variants() {
            assert_eq!(map_to_dart_ffi_type("int8").unwrap(), DartFfiType::Int8);
            assert_eq!(map_to_dart_ffi_type("i8").unwrap(), DartFfiType::Int8);
        }

        #[test]
        fn int16_variants() {
            assert_eq!(map_to_dart_ffi_type("int16").unwrap(), DartFfiType::Int16);
            assert_eq!(map_to_dart_ffi_type("i16").unwrap(), DartFfiType::Int16);
        }

        #[test]
        fn int32_variants() {
            assert_eq!(map_to_dart_ffi_type("int32").unwrap(), DartFfiType::Int32);
            assert_eq!(map_to_dart_ffi_type("i32").unwrap(), DartFfiType::Int32);
            assert_eq!(map_to_dart_ffi_type("int").unwrap(), DartFfiType::Int32);
        }

        #[test]
        fn int64_variants() {
            assert_eq!(map_to_dart_ffi_type("int64").unwrap(), DartFfiType::Int64);
            assert_eq!(map_to_dart_ffi_type("i64").unwrap(), DartFfiType::Int64);
        }
    }

    mod unsigned_integers {
        use super::*;

        #[test]
        fn uint8_variants() {
            assert_eq!(map_to_dart_ffi_type("uint8").unwrap(), DartFfiType::Uint8);
            assert_eq!(map_to_dart_ffi_type("u8").unwrap(), DartFfiType::Uint8);
            assert_eq!(map_to_dart_ffi_type("byte").unwrap(), DartFfiType::Uint8);
        }

        #[test]
        fn uint16_variants() {
            assert_eq!(map_to_dart_ffi_type("uint16").unwrap(), DartFfiType::Uint16);
            assert_eq!(map_to_dart_ffi_type("u16").unwrap(), DartFfiType::Uint16);
        }

        #[test]
        fn uint32_variants() {
            assert_eq!(map_to_dart_ffi_type("uint32").unwrap(), DartFfiType::Uint32);
            assert_eq!(map_to_dart_ffi_type("u32").unwrap(), DartFfiType::Uint32);
        }

        #[test]
        fn uint64_variants() {
            assert_eq!(map_to_dart_ffi_type("uint64").unwrap(), DartFfiType::Uint64);
            assert_eq!(map_to_dart_ffi_type("u64").unwrap(), DartFfiType::Uint64);
        }
    }

    mod floating_point {
        use super::*;

        #[test]
        fn float_variants() {
            assert_eq!(map_to_dart_ffi_type("float").unwrap(), DartFfiType::Float);
            assert_eq!(map_to_dart_ffi_type("float32").unwrap(), DartFfiType::Float);
            assert_eq!(map_to_dart_ffi_type("f32").unwrap(), DartFfiType::Float);
        }

        #[test]
        fn double_variants() {
            assert_eq!(map_to_dart_ffi_type("double").unwrap(), DartFfiType::Double);
            assert_eq!(
                map_to_dart_ffi_type("float64").unwrap(),
                DartFfiType::Double
            );
            assert_eq!(map_to_dart_ffi_type("f64").unwrap(), DartFfiType::Double);
        }
    }

    mod string_types {
        use super::*;

        #[test]
        fn string_variants() {
            assert_eq!(
                map_to_dart_ffi_type("string").unwrap(),
                DartFfiType::PointerUtf8
            );
            assert_eq!(
                map_to_dart_ffi_type("str").unwrap(),
                DartFfiType::PointerUtf8
            );
        }
    }

    mod complex_types {
        use super::*;

        #[test]
        fn object_maps_to_pointer_utf8() {
            assert_eq!(
                map_to_dart_ffi_type("object").unwrap(),
                DartFfiType::PointerUtf8
            );
        }

        #[test]
        fn array_maps_to_pointer_utf8() {
            assert_eq!(
                map_to_dart_ffi_type("array").unwrap(),
                DartFfiType::PointerUtf8
            );
        }
    }
}

// =============================================================================
// Native Type Mapping Tests
// =============================================================================

mod native_type_mapping {
    use super::*;

    #[test]
    fn void_maps_correctly() {
        assert_eq!(map_to_dart_native_type("void").unwrap(), "void");
    }

    #[test]
    fn bool_variants_map_correctly() {
        assert_eq!(map_to_dart_native_type("bool").unwrap(), "bool");
        assert_eq!(map_to_dart_native_type("boolean").unwrap(), "bool");
    }

    #[test]
    fn all_integers_map_to_int() {
        // Signed
        assert_eq!(map_to_dart_native_type("int8").unwrap(), "int");
        assert_eq!(map_to_dart_native_type("i8").unwrap(), "int");
        assert_eq!(map_to_dart_native_type("int16").unwrap(), "int");
        assert_eq!(map_to_dart_native_type("i16").unwrap(), "int");
        assert_eq!(map_to_dart_native_type("int32").unwrap(), "int");
        assert_eq!(map_to_dart_native_type("i32").unwrap(), "int");
        assert_eq!(map_to_dart_native_type("int").unwrap(), "int");
        assert_eq!(map_to_dart_native_type("int64").unwrap(), "int");
        assert_eq!(map_to_dart_native_type("i64").unwrap(), "int");

        // Unsigned
        assert_eq!(map_to_dart_native_type("uint8").unwrap(), "int");
        assert_eq!(map_to_dart_native_type("u8").unwrap(), "int");
        assert_eq!(map_to_dart_native_type("byte").unwrap(), "int");
        assert_eq!(map_to_dart_native_type("uint16").unwrap(), "int");
        assert_eq!(map_to_dart_native_type("u16").unwrap(), "int");
        assert_eq!(map_to_dart_native_type("uint32").unwrap(), "int");
        assert_eq!(map_to_dart_native_type("u32").unwrap(), "int");
        assert_eq!(map_to_dart_native_type("uint64").unwrap(), "int");
        assert_eq!(map_to_dart_native_type("u64").unwrap(), "int");
    }

    #[test]
    fn all_floats_map_to_double() {
        assert_eq!(map_to_dart_native_type("float").unwrap(), "double");
        assert_eq!(map_to_dart_native_type("float32").unwrap(), "double");
        assert_eq!(map_to_dart_native_type("f32").unwrap(), "double");
        assert_eq!(map_to_dart_native_type("double").unwrap(), "double");
        assert_eq!(map_to_dart_native_type("float64").unwrap(), "double");
        assert_eq!(map_to_dart_native_type("f64").unwrap(), "double");
    }

    #[test]
    fn string_maps_correctly() {
        assert_eq!(map_to_dart_native_type("string").unwrap(), "String");
        assert_eq!(map_to_dart_native_type("str").unwrap(), "String");
    }

    #[test]
    fn object_maps_to_map() {
        assert_eq!(
            map_to_dart_native_type("object").unwrap(),
            "Map<String, dynamic>"
        );
    }

    #[test]
    fn array_maps_to_list() {
        assert_eq!(map_to_dart_native_type("array").unwrap(), "List<dynamic>");
    }
}

// =============================================================================
// Error Handling Tests
// =============================================================================

mod error_handling {
    use super::*;

    #[test]
    fn unknown_type_returns_error_for_ffi() {
        let result = map_to_dart_ffi_type("unknown_type");
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.contract_type, "unknown_type");
        assert!(err.message.contains("Unknown"));
    }

    #[test]
    fn unknown_type_returns_error_for_native() {
        let result = map_to_dart_native_type("custom_struct");
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.contract_type, "custom_struct");
    }

    #[test]
    fn empty_type_returns_error() {
        let result = map_to_dart_ffi_type("");
        assert!(result.is_err());

        let result = map_to_dart_native_type("");
        assert!(result.is_err());
    }

    #[test]
    fn error_display_is_readable() {
        let err = TypeMappingError {
            contract_type: "bad_type".to_string(),
            message: "Unknown type".to_string(),
        };
        let display = format!("{}", err);
        assert!(display.contains("bad_type"));
        assert!(display.contains("Unknown type"));
    }

    #[test]
    fn error_implements_std_error() {
        let err: Box<dyn std::error::Error> = Box::new(TypeMappingError {
            contract_type: "test".to_string(),
            message: "test error".to_string(),
        });
        // Just verify it compiles and can be used as dyn Error
        assert!(!err.to_string().is_empty());
    }
}

// =============================================================================
// Case Insensitivity Tests
// =============================================================================

mod case_insensitivity {
    use super::*;

    #[test]
    fn uppercase_types_work() {
        assert_eq!(
            map_to_dart_ffi_type("STRING").unwrap(),
            DartFfiType::PointerUtf8
        );
        assert_eq!(map_to_dart_ffi_type("INT32").unwrap(), DartFfiType::Int32);
        assert_eq!(map_to_dart_ffi_type("BOOL").unwrap(), DartFfiType::Bool);
        assert_eq!(map_to_dart_ffi_type("DOUBLE").unwrap(), DartFfiType::Double);
    }

    #[test]
    fn mixed_case_types_work() {
        assert_eq!(
            map_to_dart_ffi_type("String").unwrap(),
            DartFfiType::PointerUtf8
        );
        assert_eq!(map_to_dart_ffi_type("Int32").unwrap(), DartFfiType::Int32);
        assert_eq!(map_to_dart_ffi_type("UInt64").unwrap(), DartFfiType::Uint64);
        assert_eq!(
            map_to_dart_ffi_type("Float64").unwrap(),
            DartFfiType::Double
        );
    }

    #[test]
    fn native_mapping_case_insensitive() {
        assert_eq!(map_to_dart_native_type("STRING").unwrap(), "String");
        assert_eq!(
            map_to_dart_native_type("OBJECT").unwrap(),
            "Map<String, dynamic>"
        );
        assert_eq!(map_to_dart_native_type("ARRAY").unwrap(), "List<dynamic>");
    }
}

// =============================================================================
// Whitespace Handling Tests
// =============================================================================

mod whitespace_handling {
    use super::*;

    #[test]
    fn leading_whitespace_trimmed() {
        assert_eq!(
            map_to_dart_ffi_type("  string").unwrap(),
            DartFfiType::PointerUtf8
        );
        assert_eq!(map_to_dart_ffi_type("\tint32").unwrap(), DartFfiType::Int32);
    }

    #[test]
    fn trailing_whitespace_trimmed() {
        assert_eq!(
            map_to_dart_ffi_type("string  ").unwrap(),
            DartFfiType::PointerUtf8
        );
        assert_eq!(map_to_dart_ffi_type("int32\n").unwrap(), DartFfiType::Int32);
    }

    #[test]
    fn both_sides_whitespace_trimmed() {
        assert_eq!(
            map_to_dart_ffi_type("  string  ").unwrap(),
            DartFfiType::PointerUtf8
        );
        assert_eq!(
            map_to_dart_ffi_type("\t\nint32\t\n").unwrap(),
            DartFfiType::Int32
        );
    }

    #[test]
    fn native_mapping_trims_whitespace() {
        assert_eq!(map_to_dart_native_type("  string  ").unwrap(), "String");
        assert_eq!(map_to_dart_native_type("\tbool\n").unwrap(), "bool");
    }
}

// =============================================================================
// Nullable Type Tests
// =============================================================================

mod nullable_types {
    use super::*;

    #[test]
    fn nullable_suffix_detected() {
        assert!(is_nullable("string?"));
        assert!(is_nullable("int32?"));
        assert!(is_nullable("bool?"));
        assert!(is_nullable("object?"));
    }

    #[test]
    fn non_nullable_types_not_detected() {
        assert!(!is_nullable("string"));
        assert!(!is_nullable("int32"));
        assert!(!is_nullable("bool"));
    }

    #[test]
    fn nullable_with_whitespace() {
        // Trailing whitespace before ? is trimmed
        assert!(is_nullable("string?  ")); // still has ?
        assert!(!is_nullable("  string")); // no ?
    }

    #[test]
    fn base_type_extraction_works() {
        assert_eq!(base_type("string?"), "string");
        assert_eq!(base_type("int32?"), "int32");
        assert_eq!(base_type("bool?"), "bool");
    }

    #[test]
    fn base_type_unchanged_for_non_nullable() {
        assert_eq!(base_type("string"), "string");
        assert_eq!(base_type("int32"), "int32");
    }

    #[test]
    fn base_type_trims_whitespace() {
        assert_eq!(base_type("  string?  "), "string");
        assert_eq!(base_type("  int32  "), "int32");
    }

    #[test]
    fn nullable_base_type_maps_correctly() {
        // Verify we can strip ? and map the base type
        let nullable_type = "string?";
        let base = base_type(nullable_type);
        assert_eq!(
            map_to_dart_ffi_type(base).unwrap(),
            DartFfiType::PointerUtf8
        );
        assert_eq!(map_to_dart_native_type(base).unwrap(), "String");
    }

    #[test]
    fn all_nullable_base_types_map_correctly() {
        let nullable_types = [
            ("void?", DartFfiType::Void, "void"),
            ("bool?", DartFfiType::Bool, "bool"),
            ("int32?", DartFfiType::Int32, "int"),
            ("double?", DartFfiType::Double, "double"),
            ("string?", DartFfiType::PointerUtf8, "String"),
            ("object?", DartFfiType::PointerUtf8, "Map<String, dynamic>"),
            ("array?", DartFfiType::PointerUtf8, "List<dynamic>"),
        ];

        for (nullable, expected_ffi, expected_native) in nullable_types {
            assert!(
                is_nullable(nullable),
                "Expected {} to be nullable",
                nullable
            );
            let base = base_type(nullable);
            assert_eq!(
                map_to_dart_ffi_type(base).unwrap(),
                expected_ffi,
                "FFI mapping failed for {}",
                nullable
            );
            assert_eq!(
                map_to_dart_native_type(base).unwrap(),
                expected_native,
                "Native mapping failed for {}",
                nullable
            );
        }
    }
}

// =============================================================================
// Complex Type Detection Tests
// =============================================================================

mod complex_type_detection {
    use super::*;

    #[test]
    fn object_is_complex() {
        assert!(is_complex_type("object"));
    }

    #[test]
    fn array_is_complex() {
        assert!(is_complex_type("array"));
    }

    #[test]
    fn primitives_are_not_complex() {
        assert!(!is_complex_type("string"));
        assert!(!is_complex_type("int32"));
        assert!(!is_complex_type("bool"));
        assert!(!is_complex_type("double"));
        assert!(!is_complex_type("void"));
    }

    #[test]
    fn complex_detection_case_insensitive() {
        assert!(is_complex_type("OBJECT"));
        assert!(is_complex_type("ARRAY"));
        assert!(is_complex_type("Object"));
        assert!(is_complex_type("Array"));
    }

    #[test]
    fn complex_detection_trims_whitespace() {
        assert!(is_complex_type("  object  "));
        assert!(is_complex_type("\tarray\n"));
    }
}

// =============================================================================
// Comprehensive Type Coverage Tests
// =============================================================================

mod comprehensive_coverage {
    use super::*;

    /// Test all supported type aliases map to expected FFI types
    #[test]
    fn all_type_aliases_covered() {
        let type_mappings: &[(&str, DartFfiType)] = &[
            // Void
            ("void", DartFfiType::Void),
            // Boolean
            ("bool", DartFfiType::Bool),
            ("boolean", DartFfiType::Bool),
            // Signed integers
            ("int8", DartFfiType::Int8),
            ("i8", DartFfiType::Int8),
            ("int16", DartFfiType::Int16),
            ("i16", DartFfiType::Int16),
            ("int32", DartFfiType::Int32),
            ("i32", DartFfiType::Int32),
            ("int", DartFfiType::Int32),
            ("int64", DartFfiType::Int64),
            ("i64", DartFfiType::Int64),
            // Unsigned integers
            ("uint8", DartFfiType::Uint8),
            ("u8", DartFfiType::Uint8),
            ("byte", DartFfiType::Uint8),
            ("uint16", DartFfiType::Uint16),
            ("u16", DartFfiType::Uint16),
            ("uint32", DartFfiType::Uint32),
            ("u32", DartFfiType::Uint32),
            ("uint64", DartFfiType::Uint64),
            ("u64", DartFfiType::Uint64),
            // Floating point
            ("float", DartFfiType::Float),
            ("float32", DartFfiType::Float),
            ("f32", DartFfiType::Float),
            ("double", DartFfiType::Double),
            ("float64", DartFfiType::Double),
            ("f64", DartFfiType::Double),
            // String types
            ("string", DartFfiType::PointerUtf8),
            ("str", DartFfiType::PointerUtf8),
            // Complex types
            ("object", DartFfiType::PointerUtf8),
            ("array", DartFfiType::PointerUtf8),
        ];

        for (contract_type, expected_ffi) in type_mappings {
            assert_eq!(
                map_to_dart_ffi_type(contract_type).unwrap(),
                *expected_ffi,
                "FFI mapping failed for '{}'",
                contract_type
            );
        }
    }

    /// Test FFI types have correct Dart representations
    #[test]
    fn ffi_types_to_dart_types_consistency() {
        let test_cases: &[(&str, &str, &str)] = &[
            // (contract_type, expected_ffi_string, expected_native)
            ("void", "Void", "void"),
            ("bool", "Bool", "bool"),
            ("int8", "Int8", "int"),
            ("int16", "Int16", "int"),
            ("int32", "Int32", "int"),
            ("int64", "Int64", "int"),
            ("uint8", "Uint8", "int"),
            ("uint16", "Uint16", "int"),
            ("uint32", "Uint32", "int"),
            ("uint64", "Uint64", "int"),
            ("float", "Float", "double"),
            ("double", "Double", "double"),
            ("string", "Pointer<Utf8>", "String"),
        ];

        for (contract_type, expected_ffi_string, expected_native) in test_cases {
            let ffi_type = map_to_dart_ffi_type(contract_type).unwrap();
            assert_eq!(
                ffi_type.ffi_type(),
                *expected_ffi_string,
                "FFI string representation wrong for '{}'",
                contract_type
            );
            assert_eq!(
                map_to_dart_native_type(contract_type).unwrap(),
                *expected_native,
                "Native type wrong for '{}'",
                contract_type
            );
        }
    }
}
