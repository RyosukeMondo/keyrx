//! Comprehensive unit tests for the validator module.
//!
//! These tests verify the reliability of FFI contract validation, including:
//! - Single function signature validation (parameter counts, types, return types)
//! - Batch validation across multiple contracts and implementations
//! - Detection of missing functions and uncontracted functions
//! - Validation report generation and statistics

use std::collections::HashMap;
use std::path::PathBuf;

use keyrx_core::ffi::contract::{FfiContract, FunctionContract, ParameterContract, TypeDefinition};

use super::parser::{ParsedFunction, ParsedParam, ParsedType};
use super::validator::{
    validate_all_functions, validate_function, FileLocation, ValidationError, ValidationReport,
};

// ============================================================================
// Test Helpers
// ============================================================================

fn make_contract(name: &str, params: Vec<(&str, &str)>, return_type: &str) -> FunctionContract {
    FunctionContract {
        name: name.to_string(),
        description: "Test function".to_string(),
        rust_name: Some(name.to_string()),
        parameters: params
            .into_iter()
            .map(|(n, t)| ParameterContract {
                name: n.to_string(),
                param_type: t.to_string(),
                description: "Test param".to_string(),
                required: true,
                constraints: None,
            })
            .collect(),
        returns: TypeDefinition::Primitive {
            type_name: return_type.to_string(),
            description: None,
            constraints: None,
        },
        errors: vec![],
        events_emitted: vec![],
        example: None,
        deprecated: false,
        since_version: None,
    }
}

fn make_parsed_fn(
    name: &str,
    params: Vec<(&str, &str, bool, bool)>,
    return_type: ParsedType,
) -> ParsedFunction {
    ParsedFunction::new(
        name.to_string(),
        params
            .into_iter()
            .map(|(n, t, is_ptr, is_mut)| {
                ParsedParam::new(n.to_string(), t.to_string(), is_ptr, is_mut)
            })
            .collect(),
        return_type,
        PathBuf::from("test.rs"),
        10,
    )
}

fn make_ffi_contract(domain: &str, functions: Vec<FunctionContract>) -> FfiContract {
    FfiContract {
        schema: "".to_string(),
        version: "1.0.0".to_string(),
        domain: domain.to_string(),
        description: "Test contract".to_string(),
        protocol_version: 1,
        functions,
        types: HashMap::new(),
        events: vec![],
    }
}

// ============================================================================
// FileLocation Tests
// ============================================================================

#[test]
fn test_file_location_new() {
    let loc = FileLocation::new(PathBuf::from("src/lib.rs"), 42);
    assert_eq!(loc.file, PathBuf::from("src/lib.rs"));
    assert_eq!(loc.line, 42);
}

#[test]
fn test_file_location_display_format() {
    let loc = FileLocation::new(PathBuf::from("src/lib.rs"), 42);
    assert_eq!(loc.to_string(), "src/lib.rs:42");
}

#[test]
fn test_file_location_display_with_nested_path() {
    let loc = FileLocation::new(PathBuf::from("core/src/ffi/exports.rs"), 100);
    assert_eq!(loc.to_string(), "core/src/ffi/exports.rs:100");
}

#[test]
fn test_file_location_equality() {
    let loc1 = FileLocation::new(PathBuf::from("test.rs"), 10);
    let loc2 = FileLocation::new(PathBuf::from("test.rs"), 10);
    let loc3 = FileLocation::new(PathBuf::from("test.rs"), 20);

    assert_eq!(loc1, loc2);
    assert_ne!(loc1, loc3);
}

#[test]
fn test_file_location_clone() {
    let loc = FileLocation::new(PathBuf::from("test.rs"), 42);
    let cloned = loc.clone();
    assert_eq!(loc, cloned);
}

// ============================================================================
// ValidationError Display Tests
// ============================================================================

#[test]
fn test_missing_function_error_display() {
    let err = ValidationError::MissingFunction {
        name: "keyrx_init".to_string(),
        contract_file: "engine.ffi-contract.json".to_string(),
    };
    let display = err.to_string();
    assert!(display.contains("keyrx_init"));
    assert!(display.contains("engine.ffi-contract.json"));
    assert!(display.contains("Missing function"));
}

#[test]
fn test_parameter_count_mismatch_display() {
    let loc = FileLocation::new(PathBuf::from("exports.rs"), 100);
    let err = ValidationError::ParameterCountMismatch {
        function: "keyrx_test".to_string(),
        expected: 3,
        found: 2,
        location: loc,
    };
    let display = err.to_string();
    assert!(display.contains("keyrx_test"));
    assert!(display.contains("expected 3"));
    assert!(display.contains("found 2"));
    assert!(display.contains("exports.rs:100"));
}

#[test]
fn test_parameter_type_mismatch_display() {
    let loc = FileLocation::new(PathBuf::from("exports.rs"), 50);
    let err = ValidationError::ParameterTypeMismatch {
        function: "keyrx_test".to_string(),
        param_name: "input".to_string(),
        param_index: 0,
        expected_type: "*const c_char".to_string(),
        found_type: "i32".to_string(),
        location: loc,
    };
    let display = err.to_string();
    assert!(display.contains("input"));
    assert!(display.contains("index 0"));
    assert!(display.contains("*const c_char"));
    assert!(display.contains("i32"));
}

#[test]
fn test_return_type_mismatch_display() {
    let loc = FileLocation::new(PathBuf::from("exports.rs"), 75);
    let err = ValidationError::ReturnTypeMismatch {
        function: "keyrx_get_value".to_string(),
        expected_type: "*const c_char".to_string(),
        found_type: "()".to_string(),
        location: loc,
    };
    let display = err.to_string();
    assert!(display.contains("Return type mismatch"));
    assert!(display.contains("keyrx_get_value"));
    assert!(display.contains("*const c_char"));
    assert!(display.contains("()"));
}

#[test]
fn test_uncontracted_function_display() {
    let loc = FileLocation::new(PathBuf::from("exports.rs"), 200);
    let err = ValidationError::UncontractedFunction {
        name: "keyrx_orphan".to_string(),
        location: loc,
    };
    let display = err.to_string();
    assert!(display.contains("Uncontracted"));
    assert!(display.contains("keyrx_orphan"));
    assert!(display.contains("no contract"));
}

#[test]
fn test_missing_error_pointer_display() {
    let loc = FileLocation::new(PathBuf::from("exports.rs"), 30);
    let err = ValidationError::MissingErrorPointer {
        function: "keyrx_no_error".to_string(),
        location: loc,
    };
    let display = err.to_string();
    assert!(display.contains("Missing error pointer"));
    assert!(display.contains("keyrx_no_error"));
    assert!(display.contains("*mut *mut c_char"));
}

#[test]
fn test_invalid_error_pointer_display() {
    let loc = FileLocation::new(PathBuf::from("exports.rs"), 60);
    let err = ValidationError::InvalidErrorPointer {
        function: "keyrx_bad_error".to_string(),
        found_type: "*mut c_char".to_string(),
        location: loc,
    };
    let display = err.to_string();
    assert!(display.contains("Invalid error pointer"));
    assert!(display.contains("*mut c_char"));
}

// ============================================================================
// ValidationError Method Tests
// ============================================================================

#[test]
fn test_function_name_extraction() {
    let loc = FileLocation::new(PathBuf::from("test.rs"), 1);

    let err1 = ValidationError::MissingFunction {
        name: "func_missing".to_string(),
        contract_file: "test.json".to_string(),
    };
    assert_eq!(err1.function_name(), "func_missing");

    let err2 = ValidationError::ParameterCountMismatch {
        function: "func_count".to_string(),
        expected: 1,
        found: 2,
        location: loc.clone(),
    };
    assert_eq!(err2.function_name(), "func_count");

    let err3 = ValidationError::UncontractedFunction {
        name: "func_orphan".to_string(),
        location: loc,
    };
    assert_eq!(err3.function_name(), "func_orphan");
}

#[test]
fn test_location_extraction() {
    let loc = FileLocation::new(PathBuf::from("test.rs"), 42);

    let err_no_loc = ValidationError::MissingFunction {
        name: "test".to_string(),
        contract_file: "test.json".to_string(),
    };
    assert!(err_no_loc.location().is_none());

    let err_with_loc = ValidationError::ParameterCountMismatch {
        function: "test".to_string(),
        expected: 1,
        found: 2,
        location: loc.clone(),
    };
    assert_eq!(err_with_loc.location(), Some(&loc));
}

// ============================================================================
// Fix Suggestion Tests
// ============================================================================

#[test]
fn test_fix_suggestion_missing_function() {
    let err = ValidationError::MissingFunction {
        name: "keyrx_init".to_string(),
        contract_file: "engine.json".to_string(),
    };
    let suggestion = err.fix_suggestion();
    assert!(suggestion.contains("Implement"));
    assert!(suggestion.contains("keyrx_init"));
    assert!(suggestion.contains("#[no_mangle]"));
}

#[test]
fn test_fix_suggestion_add_parameters() {
    let loc = FileLocation::new(PathBuf::from("test.rs"), 1);
    let err = ValidationError::ParameterCountMismatch {
        function: "test".to_string(),
        expected: 5,
        found: 3,
        location: loc,
    };
    let suggestion = err.fix_suggestion();
    assert!(suggestion.contains("Add 2 missing"));
}

#[test]
fn test_fix_suggestion_remove_parameters() {
    let loc = FileLocation::new(PathBuf::from("test.rs"), 1);
    let err = ValidationError::ParameterCountMismatch {
        function: "test".to_string(),
        expected: 2,
        found: 4,
        location: loc,
    };
    let suggestion = err.fix_suggestion();
    assert!(suggestion.contains("Remove 2 extra"));
}

#[test]
fn test_fix_suggestion_parameter_type() {
    let loc = FileLocation::new(PathBuf::from("test.rs"), 1);
    let err = ValidationError::ParameterTypeMismatch {
        function: "test".to_string(),
        param_name: "input".to_string(),
        param_index: 0,
        expected_type: "*const c_char".to_string(),
        found_type: "i32".to_string(),
        location: loc,
    };
    let suggestion = err.fix_suggestion();
    assert!(suggestion.contains("Change type"));
    assert!(suggestion.contains("input"));
    assert!(suggestion.contains("*const c_char"));
}

#[test]
fn test_fix_suggestion_return_type() {
    let loc = FileLocation::new(PathBuf::from("test.rs"), 1);
    let err = ValidationError::ReturnTypeMismatch {
        function: "test".to_string(),
        expected_type: "i32".to_string(),
        found_type: "()".to_string(),
        location: loc,
    };
    let suggestion = err.fix_suggestion();
    assert!(suggestion.contains("Change return type"));
    assert!(suggestion.contains("i32"));
}

#[test]
fn test_fix_suggestion_uncontracted() {
    let loc = FileLocation::new(PathBuf::from("test.rs"), 1);
    let err = ValidationError::UncontractedFunction {
        name: "orphan_func".to_string(),
        location: loc,
    };
    let suggestion = err.fix_suggestion();
    assert!(suggestion.contains("Add a contract"));
    assert!(suggestion.contains("orphan_func"));
}

#[test]
fn test_fix_suggestion_missing_error_pointer() {
    let loc = FileLocation::new(PathBuf::from("test.rs"), 1);
    let err = ValidationError::MissingErrorPointer {
        function: "test".to_string(),
        location: loc,
    };
    let suggestion = err.fix_suggestion();
    assert!(suggestion.contains("error_out"));
    assert!(suggestion.contains("*mut *mut c_char"));
}

#[test]
fn test_fix_suggestion_invalid_error_pointer() {
    let loc = FileLocation::new(PathBuf::from("test.rs"), 1);
    let err = ValidationError::InvalidErrorPointer {
        function: "test".to_string(),
        found_type: "*mut c_char".to_string(),
        location: loc,
    };
    let suggestion = err.fix_suggestion();
    assert!(suggestion.contains("*mut *mut c_char"));
}

// ============================================================================
// validate_function Tests - Matching Signatures
// ============================================================================

#[test]
fn test_validate_matching_void_function() {
    let contract = make_contract("keyrx_test", vec![], "void");
    let parsed = make_parsed_fn(
        "keyrx_test",
        vec![("error_out", "*mut *mut c_char", true, true)],
        ParsedType::Unit,
    );

    let result = validate_function(&contract, &parsed);
    assert!(result.is_ok());
}

#[test]
fn test_validate_matching_int_return() {
    let contract = make_contract("keyrx_test", vec![], "int");
    let parsed = make_parsed_fn(
        "keyrx_test",
        vec![("error_out", "*mut *mut c_char", true, true)],
        ParsedType::Primitive("i32".to_string()),
    );

    let result = validate_function(&contract, &parsed);
    assert!(result.is_ok());
}

#[test]
fn test_validate_matching_string_param() {
    let contract = make_contract("keyrx_test", vec![("input", "string")], "int");
    let parsed = make_parsed_fn(
        "keyrx_test",
        vec![
            ("input", "*const c_char", true, false),
            ("error_out", "*mut *mut c_char", true, true),
        ],
        ParsedType::Primitive("i32".to_string()),
    );

    let result = validate_function(&contract, &parsed);
    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
}

#[test]
fn test_validate_matching_bool_param() {
    let contract = make_contract("keyrx_test", vec![("flag", "bool")], "void");
    let parsed = make_parsed_fn(
        "keyrx_test",
        vec![
            ("flag", "bool", false, false),
            ("error_out", "*mut *mut c_char", true, true),
        ],
        ParsedType::Unit,
    );

    let result = validate_function(&contract, &parsed);
    assert!(result.is_ok());
}

#[test]
fn test_validate_matching_object_param() {
    let contract = make_contract("keyrx_test", vec![("config", "object")], "string");
    let parsed = make_parsed_fn(
        "keyrx_test",
        vec![
            ("config", "*const c_char", true, false),
            ("error_out", "*mut *mut c_char", true, true),
        ],
        ParsedType::Pointer {
            target: "c_char".to_string(),
            is_mut: false,
        },
    );

    let result = validate_function(&contract, &parsed);
    assert!(result.is_ok());
}

#[test]
fn test_validate_matching_multiple_params() {
    let contract = make_contract(
        "keyrx_test",
        vec![("config", "object"), ("count", "int"), ("enabled", "bool")],
        "string",
    );
    let parsed = make_parsed_fn(
        "keyrx_test",
        vec![
            ("config", "*const c_char", true, false),
            ("count", "i32", false, false),
            ("enabled", "bool", false, false),
            ("error_out", "*mut *mut c_char", true, true),
        ],
        ParsedType::Pointer {
            target: "c_char".to_string(),
            is_mut: false,
        },
    );

    let result = validate_function(&contract, &parsed);
    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
}

// ============================================================================
// validate_function Tests - Parameter Count Mismatches
// ============================================================================

#[test]
fn test_validate_too_few_params() {
    let contract = make_contract("keyrx_test", vec![("a", "string"), ("b", "int")], "void");
    let parsed = make_parsed_fn(
        "keyrx_test",
        vec![
            ("a", "*const c_char", true, false),
            ("error_out", "*mut *mut c_char", true, true),
        ],
        ParsedType::Unit,
    );

    let result = validate_function(&contract, &parsed);
    assert!(matches!(
        result,
        Err(ValidationError::ParameterCountMismatch {
            expected: 2,
            found: 1,
            ..
        })
    ));
}

#[test]
fn test_validate_too_many_params() {
    let contract = make_contract("keyrx_test", vec![("a", "string")], "void");
    let parsed = make_parsed_fn(
        "keyrx_test",
        vec![
            ("a", "*const c_char", true, false),
            ("b", "i32", false, false),
            ("c", "bool", false, false),
            ("error_out", "*mut *mut c_char", true, true),
        ],
        ParsedType::Unit,
    );

    let result = validate_function(&contract, &parsed);
    assert!(matches!(
        result,
        Err(ValidationError::ParameterCountMismatch {
            expected: 1,
            found: 3,
            ..
        })
    ));
}

#[test]
fn test_validate_no_params_vs_some_params() {
    let contract = make_contract("keyrx_test", vec![("input", "string")], "void");
    let parsed = make_parsed_fn(
        "keyrx_test",
        vec![("error_out", "*mut *mut c_char", true, true)],
        ParsedType::Unit,
    );

    let result = validate_function(&contract, &parsed);
    assert!(matches!(
        result,
        Err(ValidationError::ParameterCountMismatch {
            expected: 1,
            found: 0,
            ..
        })
    ));
}

// ============================================================================
// validate_function Tests - Parameter Type Mismatches
// ============================================================================

#[test]
fn test_validate_wrong_param_type_primitive() {
    let contract = make_contract("keyrx_test", vec![("input", "string")], "void");
    let parsed = make_parsed_fn(
        "keyrx_test",
        vec![
            ("input", "i32", false, false),
            ("error_out", "*mut *mut c_char", true, true),
        ],
        ParsedType::Unit,
    );

    let result = validate_function(&contract, &parsed);
    assert!(matches!(
        result,
        Err(ValidationError::ParameterTypeMismatch { param_index: 0, .. })
    ));
}

#[test]
fn test_validate_wrong_param_type_pointer_mutability() {
    let contract = make_contract("keyrx_test", vec![("input", "string")], "void");
    let parsed = make_parsed_fn(
        "keyrx_test",
        vec![
            ("input", "*mut c_char", true, true), // Should be *const
            ("error_out", "*mut *mut c_char", true, true),
        ],
        ParsedType::Unit,
    );

    let result = validate_function(&contract, &parsed);
    assert!(matches!(
        result,
        Err(ValidationError::ParameterTypeMismatch { .. })
    ));
}

#[test]
fn test_validate_second_param_type_mismatch() {
    let contract = make_contract("keyrx_test", vec![("a", "string"), ("b", "int")], "void");
    let parsed = make_parsed_fn(
        "keyrx_test",
        vec![
            ("a", "*const c_char", true, false),
            ("b", "bool", false, false), // Should be i32
            ("error_out", "*mut *mut c_char", true, true),
        ],
        ParsedType::Unit,
    );

    let result = validate_function(&contract, &parsed);
    assert!(matches!(
        result,
        Err(ValidationError::ParameterTypeMismatch { param_index: 1, .. })
    ));
}

// ============================================================================
// validate_function Tests - Return Type Mismatches
// ============================================================================

#[test]
fn test_validate_wrong_return_type_void_vs_int() {
    let contract = make_contract("keyrx_test", vec![], "int");
    let parsed = make_parsed_fn(
        "keyrx_test",
        vec![("error_out", "*mut *mut c_char", true, true)],
        ParsedType::Unit,
    );

    let result = validate_function(&contract, &parsed);
    assert!(matches!(
        result,
        Err(ValidationError::ReturnTypeMismatch { .. })
    ));
}

#[test]
fn test_validate_wrong_return_type_int_vs_void() {
    let contract = make_contract("keyrx_test", vec![], "void");
    let parsed = make_parsed_fn(
        "keyrx_test",
        vec![("error_out", "*mut *mut c_char", true, true)],
        ParsedType::Primitive("i32".to_string()),
    );

    let result = validate_function(&contract, &parsed);
    assert!(matches!(
        result,
        Err(ValidationError::ReturnTypeMismatch { .. })
    ));
}

#[test]
fn test_validate_wrong_return_type_string_vs_int() {
    let contract = make_contract("keyrx_test", vec![], "string");
    let parsed = make_parsed_fn(
        "keyrx_test",
        vec![("error_out", "*mut *mut c_char", true, true)],
        ParsedType::Primitive("i32".to_string()),
    );

    let result = validate_function(&contract, &parsed);
    assert!(matches!(
        result,
        Err(ValidationError::ReturnTypeMismatch { .. })
    ));
}

// ============================================================================
// validate_function Tests - Error Pointer
// ============================================================================

#[test]
fn test_validate_missing_error_pointer() {
    let contract = make_contract("keyrx_test", vec![("input", "string")], "void");
    let parsed = make_parsed_fn(
        "keyrx_test",
        vec![("input", "*const c_char", true, false)],
        ParsedType::Unit,
    );

    let result = validate_function(&contract, &parsed);
    assert!(matches!(
        result,
        Err(ValidationError::MissingErrorPointer { .. })
    ));
}

// ============================================================================
// ValidationReport Tests
// ============================================================================

#[test]
fn test_validation_report_new() {
    let report = ValidationReport::new();
    assert!(report.is_success());
    assert_eq!(report.passed, 0);
    assert_eq!(report.failed_count(), 0);
    assert_eq!(report.total_contracts, 0);
    assert_eq!(report.total_parsed, 0);
    assert!(report.errors.is_empty());
}

#[test]
fn test_validation_report_default() {
    let report = ValidationReport::default();
    assert!(report.is_success());
    assert_eq!(report.passed, 0);
}

#[test]
fn test_validation_report_is_success_with_errors() {
    let mut report = ValidationReport::new();
    report.errors.push(ValidationError::MissingFunction {
        name: "test".to_string(),
        contract_file: "test.json".to_string(),
    });
    assert!(!report.is_success());
}

#[test]
fn test_validation_report_failed_count() {
    let mut report = ValidationReport::new();
    report.errors.push(ValidationError::MissingFunction {
        name: "test1".to_string(),
        contract_file: "test.json".to_string(),
    });
    report.errors.push(ValidationError::MissingFunction {
        name: "test2".to_string(),
        contract_file: "test.json".to_string(),
    });
    assert_eq!(report.failed_count(), 2);
}

// ============================================================================
// validate_all_functions Tests - Matching
// ============================================================================

#[test]
fn test_validate_all_single_matching_function() {
    let contract = make_contract("keyrx_test", vec![("input", "string")], "int");
    let ffi_contract = make_ffi_contract("test", vec![contract]);

    let parsed = make_parsed_fn(
        "keyrx_test",
        vec![
            ("input", "*const c_char", true, false),
            ("error_out", "*mut *mut c_char", true, true),
        ],
        ParsedType::Primitive("i32".to_string()),
    );

    let report = validate_all_functions(&[ffi_contract], &[parsed]);

    assert!(report.is_success());
    assert_eq!(report.passed, 1);
    assert_eq!(report.total_contracts, 1);
    assert_eq!(report.total_parsed, 1);
}

#[test]
fn test_validate_all_multiple_matching_functions() {
    let contract1 = make_contract("keyrx_init", vec![], "int");
    let contract2 = make_contract("keyrx_shutdown", vec![], "void");
    let ffi_contract = make_ffi_contract("test", vec![contract1, contract2]);

    let parsed1 = make_parsed_fn(
        "keyrx_init",
        vec![("error_out", "*mut *mut c_char", true, true)],
        ParsedType::Primitive("i32".to_string()),
    );
    let parsed2 = make_parsed_fn(
        "keyrx_shutdown",
        vec![("error_out", "*mut *mut c_char", true, true)],
        ParsedType::Unit,
    );

    let report = validate_all_functions(&[ffi_contract], &[parsed1, parsed2]);

    assert!(report.is_success());
    assert_eq!(report.passed, 2);
    assert_eq!(report.total_contracts, 2);
    assert_eq!(report.total_parsed, 2);
}

#[test]
fn test_validate_all_multiple_contracts() {
    let contract1 = make_contract("keyrx_a", vec![], "void");
    let contract2 = make_contract("keyrx_b", vec![("x", "int")], "int");

    let ffi_contract1 = make_ffi_contract("domain1", vec![contract1]);
    let ffi_contract2 = make_ffi_contract("domain2", vec![contract2]);

    let parsed_a = make_parsed_fn(
        "keyrx_a",
        vec![("error_out", "*mut *mut c_char", true, true)],
        ParsedType::Unit,
    );
    let parsed_b = make_parsed_fn(
        "keyrx_b",
        vec![
            ("x", "i32", false, false),
            ("error_out", "*mut *mut c_char", true, true),
        ],
        ParsedType::Primitive("i32".to_string()),
    );

    let report = validate_all_functions(&[ffi_contract1, ffi_contract2], &[parsed_a, parsed_b]);

    assert!(report.is_success());
    assert_eq!(report.passed, 2);
    assert_eq!(report.total_contracts, 2);
}

// ============================================================================
// validate_all_functions Tests - Missing Functions
// ============================================================================

#[test]
fn test_validate_all_missing_implementation() {
    let contract = make_contract("keyrx_missing", vec![], "void");
    let ffi_contract = make_ffi_contract("test", vec![contract]);

    let report = validate_all_functions(&[ffi_contract], &[]);

    assert!(!report.is_success());
    assert_eq!(report.failed_count(), 1);
    assert!(matches!(
        &report.errors[0],
        ValidationError::MissingFunction { name, .. } if name == "keyrx_missing"
    ));
}

#[test]
fn test_validate_all_multiple_missing() {
    let contract1 = make_contract("keyrx_missing1", vec![], "void");
    let contract2 = make_contract("keyrx_missing2", vec![], "int");
    let ffi_contract = make_ffi_contract("test", vec![contract1, contract2]);

    let report = validate_all_functions(&[ffi_contract], &[]);

    assert_eq!(report.failed_count(), 2);
}

#[test]
fn test_validate_all_partial_missing() {
    let contract1 = make_contract("keyrx_exists", vec![], "void");
    let contract2 = make_contract("keyrx_missing", vec![], "void");
    let ffi_contract = make_ffi_contract("test", vec![contract1, contract2]);

    let parsed = make_parsed_fn(
        "keyrx_exists",
        vec![("error_out", "*mut *mut c_char", true, true)],
        ParsedType::Unit,
    );

    let report = validate_all_functions(&[ffi_contract], &[parsed]);

    assert_eq!(report.passed, 1);
    assert_eq!(report.failed_count(), 1);
    assert!(report.errors.iter().any(|e| matches!(
        e,
        ValidationError::MissingFunction { name, .. } if name == "keyrx_missing"
    )));
}

// ============================================================================
// validate_all_functions Tests - Uncontracted Functions
// ============================================================================

#[test]
fn test_validate_all_uncontracted_function() {
    let ffi_contract = make_ffi_contract("test", vec![]);

    let parsed = make_parsed_fn(
        "keyrx_orphan",
        vec![("error_out", "*mut *mut c_char", true, true)],
        ParsedType::Unit,
    );

    let report = validate_all_functions(&[ffi_contract], &[parsed]);

    assert!(!report.is_success());
    assert_eq!(report.failed_count(), 1);
    assert!(matches!(
        &report.errors[0],
        ValidationError::UncontractedFunction { name, .. } if name == "keyrx_orphan"
    ));
}

#[test]
fn test_validate_all_multiple_uncontracted() {
    let ffi_contract = make_ffi_contract("test", vec![]);

    let parsed1 = make_parsed_fn(
        "keyrx_orphan1",
        vec![("error_out", "*mut *mut c_char", true, true)],
        ParsedType::Unit,
    );
    let parsed2 = make_parsed_fn(
        "keyrx_orphan2",
        vec![("error_out", "*mut *mut c_char", true, true)],
        ParsedType::Unit,
    );

    let report = validate_all_functions(&[ffi_contract], &[parsed1, parsed2]);

    assert_eq!(report.failed_count(), 2);
    assert_eq!(report.total_parsed, 2);
}

// ============================================================================
// validate_all_functions Tests - Bidirectional Detection
// ============================================================================

#[test]
fn test_validate_all_bidirectional_mismatch() {
    let contract = make_contract("keyrx_in_contract", vec![], "void");
    let ffi_contract = make_ffi_contract("test", vec![contract]);

    let parsed = make_parsed_fn(
        "keyrx_in_impl",
        vec![("error_out", "*mut *mut c_char", true, true)],
        ParsedType::Unit,
    );

    let report = validate_all_functions(&[ffi_contract], &[parsed]);

    assert!(!report.is_success());
    assert_eq!(report.failed_count(), 2);

    let has_missing = report.errors.iter().any(|e| {
        matches!(
            e,
            ValidationError::MissingFunction { name, .. } if name == "keyrx_in_contract"
        )
    });
    let has_uncontracted = report.errors.iter().any(|e| {
        matches!(
            e,
            ValidationError::UncontractedFunction { name, .. } if name == "keyrx_in_impl"
        )
    });

    assert!(has_missing, "Should detect missing function");
    assert!(has_uncontracted, "Should detect uncontracted function");
}

// ============================================================================
// validate_all_functions Tests - Error Collection
// ============================================================================

#[test]
fn test_validate_all_collects_all_errors() {
    let missing_contract = make_contract("keyrx_missing", vec![], "void");
    let mismatch_contract = make_contract("keyrx_mismatch", vec![("x", "string")], "void");
    let ffi_contract = make_ffi_contract("test", vec![missing_contract, mismatch_contract]);

    let parsed = make_parsed_fn(
        "keyrx_mismatch",
        vec![
            ("x", "i32", false, false), // Wrong type
            ("error_out", "*mut *mut c_char", true, true),
        ],
        ParsedType::Unit,
    );

    let report = validate_all_functions(&[ffi_contract], &[parsed]);

    assert!(!report.is_success());
    assert_eq!(report.failed_count(), 2);

    let has_missing = report
        .errors
        .iter()
        .any(|e| matches!(e, ValidationError::MissingFunction { .. }));
    let has_type_mismatch = report
        .errors
        .iter()
        .any(|e| matches!(e, ValidationError::ParameterTypeMismatch { .. }));

    assert!(has_missing, "Should have MissingFunction error");
    assert!(has_type_mismatch, "Should have ParameterTypeMismatch error");
}

#[test]
fn test_validate_all_does_not_fail_fast() {
    let contract1 = make_contract("keyrx_bad1", vec![("x", "int")], "void");
    let contract2 = make_contract("keyrx_bad2", vec![("y", "string")], "void");
    let contract3 = make_contract("keyrx_missing", vec![], "void");
    let ffi_contract = make_ffi_contract("test", vec![contract1, contract2, contract3]);

    let parsed1 = make_parsed_fn(
        "keyrx_bad1",
        vec![
            ("x", "bool", false, false), // Wrong type
            ("error_out", "*mut *mut c_char", true, true),
        ],
        ParsedType::Unit,
    );
    let parsed2 = make_parsed_fn(
        "keyrx_bad2",
        vec![
            ("y", "i32", false, false), // Wrong type
            ("error_out", "*mut *mut c_char", true, true),
        ],
        ParsedType::Unit,
    );
    let parsed_orphan = make_parsed_fn(
        "keyrx_orphan",
        vec![("error_out", "*mut *mut c_char", true, true)],
        ParsedType::Unit,
    );

    let report = validate_all_functions(&[ffi_contract], &[parsed1, parsed2, parsed_orphan]);

    // Should have collected all errors:
    // - keyrx_bad1 type mismatch
    // - keyrx_bad2 type mismatch
    // - keyrx_missing is missing
    // - keyrx_orphan is uncontracted
    assert_eq!(report.failed_count(), 4);
    assert_eq!(report.passed, 0);
}

// ============================================================================
// validate_all_functions Tests - Empty Cases
// ============================================================================

#[test]
fn test_validate_all_empty_contracts_empty_parsed() {
    let report = validate_all_functions(&[], &[]);
    assert!(report.is_success());
    assert_eq!(report.passed, 0);
    assert_eq!(report.total_contracts, 0);
    assert_eq!(report.total_parsed, 0);
}

#[test]
fn test_validate_all_empty_contracts_with_parsed() {
    let parsed = make_parsed_fn(
        "keyrx_orphan",
        vec![("error_out", "*mut *mut c_char", true, true)],
        ParsedType::Unit,
    );

    let report = validate_all_functions(&[], &[parsed]);

    assert!(!report.is_success());
    assert_eq!(report.failed_count(), 1);
    assert_eq!(report.total_parsed, 1);
}

// ============================================================================
// validate_all_functions Tests - rust_name Override
// ============================================================================

#[test]
fn test_validate_uses_rust_name_when_different() {
    let mut contract = make_contract("contractName", vec![], "void");
    contract.rust_name = Some("keyrx_rust_name".to_string());
    let ffi_contract = make_ffi_contract("test", vec![contract]);

    let parsed = make_parsed_fn(
        "keyrx_rust_name",
        vec![("error_out", "*mut *mut c_char", true, true)],
        ParsedType::Unit,
    );

    let report = validate_all_functions(&[ffi_contract], &[parsed]);

    assert!(report.is_success());
    assert_eq!(report.passed, 1);
}

#[test]
fn test_validate_falls_back_to_name_when_no_rust_name() {
    let mut contract = make_contract("keyrx_func", vec![], "void");
    contract.rust_name = None;
    let ffi_contract = make_ffi_contract("test", vec![contract]);

    let parsed = make_parsed_fn(
        "keyrx_func",
        vec![("error_out", "*mut *mut c_char", true, true)],
        ParsedType::Unit,
    );

    let report = validate_all_functions(&[ffi_contract], &[parsed]);

    assert!(report.is_success());
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_validate_function_with_many_params() {
    let contract = make_contract(
        "keyrx_multi",
        vec![
            ("p1", "string"),
            ("p2", "int"),
            ("p3", "bool"),
            ("p4", "object"),
            ("p5", "int"),
        ],
        "void",
    );

    let parsed = make_parsed_fn(
        "keyrx_multi",
        vec![
            ("p1", "*const c_char", true, false),
            ("p2", "i32", false, false),
            ("p3", "bool", false, false),
            ("p4", "*const c_char", true, false),
            ("p5", "i32", false, false),
            ("error_out", "*mut *mut c_char", true, true),
        ],
        ParsedType::Unit,
    );

    let result = validate_function(&contract, &parsed);
    assert!(result.is_ok());
}

#[test]
fn test_validation_error_is_std_error() {
    let err = ValidationError::MissingFunction {
        name: "test".to_string(),
        contract_file: "test.json".to_string(),
    };
    // Verify it implements std::error::Error
    let _: &dyn std::error::Error = &err;
}

#[test]
fn test_validation_error_clone() {
    let loc = FileLocation::new(PathBuf::from("test.rs"), 42);
    let err = ValidationError::ParameterCountMismatch {
        function: "test".to_string(),
        expected: 3,
        found: 2,
        location: loc,
    };
    let cloned = err.clone();
    assert_eq!(format!("{}", err), format!("{}", cloned));
}

#[test]
fn test_validation_report_clone() {
    let mut report = ValidationReport::new();
    report.passed = 5;
    report.total_contracts = 10;
    report.errors.push(ValidationError::MissingFunction {
        name: "test".to_string(),
        contract_file: "test.json".to_string(),
    });

    let cloned = report.clone();
    assert_eq!(report.passed, cloned.passed);
    assert_eq!(report.total_contracts, cloned.total_contracts);
    assert_eq!(report.errors.len(), cloned.errors.len());
}
