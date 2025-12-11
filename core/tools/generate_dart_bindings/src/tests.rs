//! Centralized unit tests for code generators
//!
//! This module provides integration-style tests that verify the code generators
//! work together correctly, as well as edge case tests that complement the
//! per-module unit tests.

use crate::bindings_gen::{
    generate_ffi_signatures, generate_function_pointers_block, generate_typedefs_block,
    generate_wrapper_functions, generate_wrappers_block,
};
use crate::models_gen::{generate_all_models, generate_models_block};
use crate::templates::{to_camel_case, to_pascal_case, to_snake_case};
use crate::type_mapper::{map_to_dart_ffi_type, map_to_dart_native_type};
use crate::types::DartFfiType;
use keyrx_core::ffi::contract::{FfiContract, FunctionContract, ParameterContract, TypeDefinition};
use std::collections::HashMap;

// =============================================================================
// Test Helpers
// =============================================================================

/// Create a minimal test contract with no functions
fn create_empty_contract() -> FfiContract {
    FfiContract {
        schema: "https://keyrx.dev/schemas/ffi-contract-v1.json".to_string(),
        version: "1.0.0".to_string(),
        domain: "empty".to_string(),
        description: "Empty contract for testing".to_string(),
        protocol_version: 1,
        functions: vec![],
        types: HashMap::new(),
        events: vec![],
    }
}

/// Create a contract with multiple parameter types
fn create_multi_param_contract() -> FfiContract {
    FfiContract {
        schema: "https://keyrx.dev/schemas/ffi-contract-v1.json".to_string(),
        version: "1.0.0".to_string(),
        domain: "multi".to_string(),
        description: "Contract with multiple param types".to_string(),
        protocol_version: 1,
        functions: vec![FunctionContract {
            name: "complex_operation".to_string(),
            description: "Operation with multiple params".to_string(),
            rust_name: Some("keyrx_multi_complex_operation".to_string()),
            parameters: vec![
                ParameterContract {
                    name: "text".to_string(),
                    param_type: "string".to_string(),
                    description: "Text input".to_string(),
                    required: true,
                    constraints: None,
                },
                ParameterContract {
                    name: "count".to_string(),
                    param_type: "int32".to_string(),
                    description: "Count value".to_string(),
                    required: true,
                    constraints: None,
                },
                ParameterContract {
                    name: "enabled".to_string(),
                    param_type: "bool".to_string(),
                    description: "Flag".to_string(),
                    required: true,
                    constraints: None,
                },
            ],
            returns: TypeDefinition::Primitive {
                type_name: "bool".to_string(),
                description: None,
                constraints: None,
            },
            errors: vec![],
            events_emitted: vec![],
            example: None,
            deprecated: false,
            since_version: None,
        }],
        types: HashMap::new(),
        events: vec![],
    }
}

/// Create a contract with void return type
fn create_void_return_contract() -> FfiContract {
    FfiContract {
        schema: "https://keyrx.dev/schemas/ffi-contract-v1.json".to_string(),
        version: "1.0.0".to_string(),
        domain: "void_test".to_string(),
        description: "Contract with void return".to_string(),
        protocol_version: 1,
        functions: vec![FunctionContract {
            name: "do_something".to_string(),
            description: "Does something without returning".to_string(),
            rust_name: None,
            parameters: vec![],
            returns: TypeDefinition::Primitive {
                type_name: "void".to_string(),
                description: None,
                constraints: None,
            },
            errors: vec![],
            events_emitted: vec![],
            example: None,
            deprecated: false,
            since_version: None,
        }],
        types: HashMap::new(),
        events: vec![],
    }
}

/// Create a contract with array return type
fn create_array_return_contract() -> FfiContract {
    FfiContract {
        schema: "https://keyrx.dev/schemas/ffi-contract-v1.json".to_string(),
        version: "1.0.0".to_string(),
        domain: "array_test".to_string(),
        description: "Contract with array return".to_string(),
        protocol_version: 1,
        functions: vec![FunctionContract {
            name: "get_items".to_string(),
            description: "Get list of items".to_string(),
            rust_name: None,
            parameters: vec![],
            returns: TypeDefinition::Array {
                type_name: "array".to_string(),
                items: Box::new(TypeDefinition::Primitive {
                    type_name: "string".to_string(),
                    description: None,
                    constraints: None,
                }),
                constraints: None,
            },
            errors: vec![],
            events_emitted: vec![],
            example: None,
            deprecated: false,
            since_version: None,
        }],
        types: HashMap::new(),
        events: vec![],
    }
}

/// Create a contract with custom types for model generation
fn create_contract_with_types() -> FfiContract {
    let mut types = HashMap::new();

    // Add a simple type
    let mut simple_props = HashMap::new();
    simple_props.insert(
        "id".to_string(),
        Box::new(TypeDefinition::Primitive {
            type_name: "string".to_string(),
            description: Some("Unique identifier".to_string()),
            constraints: None,
        }),
    );
    simple_props.insert(
        "name".to_string(),
        Box::new(TypeDefinition::Primitive {
            type_name: "string".to_string(),
            description: Some("Name".to_string()),
            constraints: None,
        }),
    );

    types.insert(
        "SimpleType".to_string(),
        TypeDefinition::Object {
            type_name: "object".to_string(),
            description: Some("A simple type".to_string()),
            properties: simple_props,
        },
    );

    // Add a complex type with various field types
    let mut complex_props = HashMap::new();
    complex_props.insert(
        "count".to_string(),
        Box::new(TypeDefinition::Primitive {
            type_name: "int32".to_string(),
            description: None,
            constraints: None,
        }),
    );
    complex_props.insert(
        "ratio".to_string(),
        Box::new(TypeDefinition::Primitive {
            type_name: "float64".to_string(),
            description: None,
            constraints: None,
        }),
    );
    complex_props.insert(
        "active".to_string(),
        Box::new(TypeDefinition::Primitive {
            type_name: "bool".to_string(),
            description: None,
            constraints: None,
        }),
    );
    complex_props.insert(
        "optional_field".to_string(),
        Box::new(TypeDefinition::Primitive {
            type_name: "string?".to_string(),
            description: Some("Optional field".to_string()),
            constraints: None,
        }),
    );

    types.insert(
        "ComplexType".to_string(),
        TypeDefinition::Object {
            type_name: "object".to_string(),
            description: Some("A complex type with various fields".to_string()),
            properties: complex_props,
        },
    );

    FfiContract {
        schema: "https://keyrx.dev/schemas/ffi-contract-v1.json".to_string(),
        version: "1.0.0".to_string(),
        domain: "types_test".to_string(),
        description: "Contract with custom types".to_string(),
        protocol_version: 1,
        functions: vec![],
        types,
        events: vec![],
    }
}

// =============================================================================
// FFI Signature Generation Tests
// =============================================================================

#[test]
fn test_ffi_signatures_empty_contract() {
    let contract = create_empty_contract();
    let signatures = generate_ffi_signatures(&contract).unwrap();
    assert!(signatures.is_empty());
}

#[test]
fn test_ffi_signatures_multi_param_function() {
    let contract = create_multi_param_contract();
    let signatures = generate_ffi_signatures(&contract).unwrap();

    assert_eq!(signatures.len(), 1);
    let sig = &signatures[0];

    // Verify the native typedef contains all params
    assert!(sig.native_typedef.contains("Pointer<Utf8> text"));
    assert!(sig.native_typedef.contains("Int32 count"));
    assert!(sig.native_typedef.contains("Bool enabled"));
    assert!(sig.native_typedef.contains("Pointer<Pointer<Utf8>> error"));
}

#[test]
fn test_ffi_signatures_void_return() {
    let contract = create_void_return_contract();
    let signatures = generate_ffi_signatures(&contract).unwrap();

    assert_eq!(signatures.len(), 1);
    let sig = &signatures[0];

    // Verify void return type
    assert!(sig.native_typedef.contains("Void Function"));
}

#[test]
fn test_ffi_signatures_auto_generated_name() {
    let contract = create_void_return_contract();
    let signatures = generate_ffi_signatures(&contract).unwrap();

    // Without rust_name, should auto-generate: keyrx_{domain}_{function_name}
    assert_eq!(signatures[0].ffi_name, "keyrx_void_test_do_something");
}

#[test]
fn test_typedefs_block_generation() {
    let contract = create_multi_param_contract();
    let signatures = generate_ffi_signatures(&contract).unwrap();
    let block = generate_typedefs_block(&signatures);

    // Should contain both native and dart typedefs
    assert!(block.contains("_keyrx_multi_complex_operation_native"));
    assert!(block.contains("typedef _keyrx_multi_complex_operation ="));
}

#[test]
fn test_function_pointers_block_generation() {
    let contract = create_multi_param_contract();
    let signatures = generate_ffi_signatures(&contract).unwrap();
    let block = generate_function_pointers_block(&signatures);

    // Should contain late final declaration with correct lookup
    assert!(block.contains("late final _complexOperation"));
    assert!(block.contains("'keyrx_multi_complex_operation'"));
}

// =============================================================================
// Wrapper Function Generation Tests
// =============================================================================

#[test]
fn test_wrapper_functions_empty_contract() {
    let contract = create_empty_contract();
    let wrappers = generate_wrapper_functions(&contract).unwrap();
    assert!(wrappers.is_empty());
}

#[test]
fn test_wrapper_function_multi_param() {
    let contract = create_multi_param_contract();
    let wrappers = generate_wrapper_functions(&contract).unwrap();

    assert_eq!(wrappers.len(), 1);
    let wrapper = &wrappers[0];

    // Check function name is camelCase
    assert_eq!(wrapper.dart_name, "complexOperation");

    // Check parameters are included
    assert!(wrapper.code.contains("String text"));
    assert!(wrapper.code.contains("int count"));
    assert!(wrapper.code.contains("bool enabled"));
}

#[test]
fn test_wrapper_function_void_return() {
    let contract = create_void_return_contract();
    let wrappers = generate_wrapper_functions(&contract).unwrap();

    let wrapper = &wrappers[0];
    assert!(wrapper.code.contains("void doSomething()"));
}

#[test]
fn test_wrapper_function_array_return() {
    let contract = create_array_return_contract();
    let wrappers = generate_wrapper_functions(&contract).unwrap();

    let wrapper = &wrappers[0];
    // Array return should be List<dynamic>
    assert!(wrapper.code.contains("List<dynamic> getItems()"));
    assert!(wrapper.code.contains("jsonDecode"));
}

#[test]
fn test_wrappers_block_generation() {
    let contract = create_multi_param_contract();
    let wrappers = generate_wrapper_functions(&contract).unwrap();
    let block = generate_wrappers_block(&wrappers);

    // Should contain the function implementation
    assert!(block.contains("complexOperation"));
    assert!(block.contains("errorPtr"));
}

// =============================================================================
// Model Class Generation Tests
// =============================================================================

#[test]
fn test_models_from_empty_contract() {
    let contract = create_empty_contract();
    let models = generate_all_models(&contract).unwrap();
    assert!(models.is_empty());
}

#[test]
fn test_models_from_types() {
    let contract = create_contract_with_types();
    let models = generate_all_models(&contract).unwrap();

    assert_eq!(models.len(), 2);

    let names: Vec<_> = models.iter().map(|m| m.class_name.as_str()).collect();
    assert!(names.contains(&"SimpleType"));
    assert!(names.contains(&"ComplexType"));
}

#[test]
fn test_model_field_types() {
    let contract = create_contract_with_types();
    let models = generate_all_models(&contract).unwrap();

    let complex = models.iter().find(|m| m.class_name == "ComplexType").unwrap();

    // Check field type mappings
    assert!(complex.code.contains("final int count"));
    assert!(complex.code.contains("final double ratio"));
    assert!(complex.code.contains("final bool active"));
    assert!(complex.code.contains("final String? optionalField")); // nullable
}

#[test]
fn test_model_json_serialization() {
    let contract = create_contract_with_types();
    let models = generate_all_models(&contract).unwrap();

    let simple = models.iter().find(|m| m.class_name == "SimpleType").unwrap();

    // Check fromJson
    assert!(simple.code.contains("factory SimpleType.fromJson"));
    assert!(simple.code.contains("json['id']"));
    assert!(simple.code.contains("json['name']"));

    // Check toJson
    assert!(simple.code.contains("Map<String, dynamic> toJson()"));
    assert!(simple.code.contains("'id': id"));
    assert!(simple.code.contains("'name': name"));
}

#[test]
fn test_model_constructor() {
    let contract = create_contract_with_types();
    let models = generate_all_models(&contract).unwrap();

    let complex = models.iter().find(|m| m.class_name == "ComplexType").unwrap();

    // Required fields should have 'required' keyword
    assert!(complex.code.contains("required this.count"));
    assert!(complex.code.contains("required this.ratio"));
    assert!(complex.code.contains("required this.active"));

    // Optional field should not have 'required'
    assert!(complex.code.contains("this.optionalField,"));
    // And should NOT have 'required this.optionalField'
    assert!(!complex.code.contains("required this.optionalField"));
}

#[test]
fn test_models_block_generation() {
    let contract = create_contract_with_types();
    let models = generate_all_models(&contract).unwrap();
    let block = generate_models_block(&models);

    // Should contain both class definitions
    assert!(block.contains("class SimpleType"));
    assert!(block.contains("class ComplexType"));
}

// =============================================================================
// Type Mapping Edge Cases
// =============================================================================

#[test]
fn test_type_mapping_all_integers() {
    // Test all integer types map correctly
    let types = ["int8", "int16", "int32", "int64", "uint8", "uint16", "uint32", "uint64"];

    for t in types {
        let ffi = map_to_dart_ffi_type(t).expect(&format!("Should map {}", t));
        let native = map_to_dart_native_type(t).expect(&format!("Should map native {}", t));

        // All map to int in native Dart
        assert_eq!(native, "int", "Type {} should map to int", t);

        // FFI types should be specific
        assert!(
            matches!(
                ffi,
                DartFfiType::Int8
                    | DartFfiType::Int16
                    | DartFfiType::Int32
                    | DartFfiType::Int64
                    | DartFfiType::Uint8
                    | DartFfiType::Uint16
                    | DartFfiType::Uint32
                    | DartFfiType::Uint64
            ),
            "Type {} should map to integer FFI type",
            t
        );
    }
}

#[test]
fn test_type_mapping_floats() {
    let types = ["float32", "float64", "f32", "f64", "double"];

    for t in types {
        let native = map_to_dart_native_type(t).expect(&format!("Should map native {}", t));
        assert_eq!(native, "double", "Type {} should map to double", t);
    }
}

#[test]
fn test_type_mapping_nullable() {
    use crate::type_mapper::{base_type, is_nullable};

    // Nullable types need to use base_type() first before mapping
    let type_str = "string?";
    assert!(is_nullable(type_str));

    let base = base_type(type_str);
    assert_eq!(base, "string");

    // Map the base type
    let ffi = map_to_dart_ffi_type(base);
    let native = map_to_dart_native_type(base);

    assert!(ffi.is_ok());
    assert!(native.is_ok());
    assert_eq!(native.unwrap(), "String");
}

// =============================================================================
// Case Conversion Tests
// =============================================================================

#[test]
fn test_case_conversion_edge_cases() {
    // Empty string
    assert_eq!(to_camel_case(""), "");
    assert_eq!(to_pascal_case(""), "");
    assert_eq!(to_snake_case(""), "");

    // Single character
    assert_eq!(to_camel_case("a"), "a");
    assert_eq!(to_pascal_case("a"), "A");
    assert_eq!(to_snake_case("a"), "a");

    // Multiple underscores
    assert_eq!(to_camel_case("a__b"), "aB");
    assert_eq!(to_pascal_case("a__b"), "AB");

    // Leading/trailing underscores
    assert_eq!(to_camel_case("_leading"), "Leading");
    assert_eq!(to_camel_case("trailing_"), "trailing");

    // All caps
    assert_eq!(to_snake_case("ABC"), "a_b_c");
}

#[test]
fn test_case_conversion_roundtrip() {
    let original = "device_count";

    let pascal = to_pascal_case(original);
    assert_eq!(pascal, "DeviceCount");

    let snake = to_snake_case(&pascal);
    assert_eq!(snake, "device_count");

    let camel = to_camel_case(original);
    assert_eq!(camel, "deviceCount");
}

// =============================================================================
// Integration Tests - Full Pipeline Components
// =============================================================================

#[test]
fn test_full_contract_code_generation() {
    // Create a realistic contract
    let mut types = HashMap::new();
    let mut props = HashMap::new();
    props.insert(
        "device_id".to_string(),
        Box::new(TypeDefinition::Primitive {
            type_name: "string".to_string(),
            description: None,
            constraints: None,
        }),
    );
    props.insert(
        "is_connected".to_string(),
        Box::new(TypeDefinition::Primitive {
            type_name: "bool".to_string(),
            description: None,
            constraints: None,
        }),
    );
    types.insert(
        "DeviceStatus".to_string(),
        TypeDefinition::Object {
            type_name: "object".to_string(),
            description: Some("Device status information".to_string()),
            properties: props,
        },
    );

    let contract = FfiContract {
        schema: "https://keyrx.dev/schemas/ffi-contract-v1.json".to_string(),
        version: "1.0.0".to_string(),
        domain: "device".to_string(),
        description: "Device management".to_string(),
        protocol_version: 1,
        functions: vec![
            FunctionContract {
                name: "get_status".to_string(),
                description: "Get device status".to_string(),
                rust_name: Some("keyrx_device_get_status".to_string()),
                parameters: vec![ParameterContract {
                    name: "device_id".to_string(),
                    param_type: "string".to_string(),
                    description: "Device identifier".to_string(),
                    required: true,
                    constraints: None,
                }],
                returns: TypeDefinition::Object {
                    type_name: "object".to_string(),
                    description: None,
                    properties: HashMap::new(),
                },
                errors: vec![],
                events_emitted: vec![],
                example: None,
                deprecated: false,
                since_version: None,
            },
            FunctionContract {
                name: "connect".to_string(),
                description: "Connect to device".to_string(),
                rust_name: Some("keyrx_device_connect".to_string()),
                parameters: vec![],
                returns: TypeDefinition::Primitive {
                    type_name: "bool".to_string(),
                    description: None,
                    constraints: None,
                },
                errors: vec![],
                events_emitted: vec![],
                example: None,
                deprecated: false,
                since_version: None,
            },
        ],
        types,
        events: vec![],
    };

    // Generate all components
    let signatures = generate_ffi_signatures(&contract).unwrap();
    let wrappers = generate_wrapper_functions(&contract).unwrap();
    let models = generate_all_models(&contract).unwrap();

    // Verify signatures
    assert_eq!(signatures.len(), 2);
    assert!(signatures.iter().any(|s| s.ffi_name == "keyrx_device_get_status"));
    assert!(signatures.iter().any(|s| s.ffi_name == "keyrx_device_connect"));

    // Verify wrappers
    assert_eq!(wrappers.len(), 2);
    assert!(wrappers.iter().any(|w| w.dart_name == "getStatus"));
    assert!(wrappers.iter().any(|w| w.dart_name == "connect"));

    // Verify models
    // Should have DeviceStatus type and GetStatusResult from return type
    assert!(!models.is_empty());
    let model_names: Vec<_> = models.iter().map(|m| m.class_name.as_str()).collect();
    assert!(model_names.contains(&"DeviceStatus"));
}

#[test]
fn test_generated_code_validity() {
    // Test that generated code has proper structure
    let contract = create_multi_param_contract();

    let signatures = generate_ffi_signatures(&contract).unwrap();
    let sig = &signatures[0];

    // Native typedef should start with typedef
    assert!(sig.native_typedef.starts_with("typedef"));

    // Dart typedef should start with typedef
    assert!(sig.dart_typedef.starts_with("typedef"));

    // Function pointer should have late final
    assert!(sig.function_pointer.contains("late final"));

    let wrappers = generate_wrapper_functions(&contract).unwrap();
    let wrapper = &wrappers[0];

    // Wrapper should have try-finally for cleanup
    assert!(wrapper.code.contains("try {"));
    assert!(wrapper.code.contains("} finally {"));

    // Wrapper should have error handling
    assert!(wrapper.code.contains("if (errorPtr.value.address != 0)"));
    assert!(wrapper.code.contains("throw FfiException"));
}
