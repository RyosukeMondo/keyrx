//! Comprehensive unit tests for the AST parser module.
//!
//! These tests verify the reliability of FFI function signature extraction
//! from Rust source code using the syn crate.

use std::path::PathBuf;

use super::parser::{
    parse_ffi_exports_from_str, ParseError, ParsedFunction, ParsedParam, ParsedType,
};

// ============================================================================
// Basic Parsing Tests
// ============================================================================

#[test]
fn test_parse_empty_source() {
    let source = "";
    let funcs = parse_ffi_exports_from_str(source, PathBuf::from("empty.rs"))
        .expect("empty source should parse");
    assert!(funcs.is_empty(), "empty source should yield no functions");
}

#[test]
fn test_parse_source_with_only_comments_and_items() {
    // A valid Rust file with comments and a non-FFI struct
    let source = r#"
        // This is a comment
        /* This is a block comment */
        /// This is a doc comment
        struct Placeholder;
    "#;
    let funcs = parse_ffi_exports_from_str(source, PathBuf::from("comments.rs"))
        .expect("source with comments should parse");
    assert!(funcs.is_empty(), "non-FFI items should yield no functions");
}

#[test]
fn test_parse_source_with_use_statements_only() {
    let source = r#"
        use std::ffi::{c_char, CStr, CString};
        use std::ptr;
    "#;
    let funcs = parse_ffi_exports_from_str(source, PathBuf::from("uses.rs"))
        .expect("use-only source should parse");
    assert!(funcs.is_empty());
}

// ============================================================================
// Attribute Detection Tests
// ============================================================================

#[test]
fn test_function_requires_no_mangle_attribute() {
    let source = r#"
        // Missing #[no_mangle] - should NOT be extracted
        pub unsafe extern "C" fn missing_no_mangle() -> i32 { 0 }
    "#;
    let funcs = parse_ffi_exports_from_str(source, PathBuf::from("test.rs")).unwrap();
    assert!(
        funcs.is_empty(),
        "function without #[no_mangle] should be ignored"
    );
}

#[test]
fn test_function_requires_pub_visibility() {
    let source = r#"
        // Private function - should NOT be extracted
        #[no_mangle]
        unsafe extern "C" fn private_fn() -> i32 { 0 }
    "#;
    let funcs = parse_ffi_exports_from_str(source, PathBuf::from("test.rs")).unwrap();
    assert!(funcs.is_empty(), "non-public function should be ignored");
}

#[test]
fn test_function_requires_extern_c_abi() {
    let source = r#"
        // Missing extern "C" - should NOT be extracted
        #[no_mangle]
        pub fn missing_extern_c() -> i32 { 0 }

        // Wrong ABI - should NOT be extracted
        #[no_mangle]
        pub extern "system" fn wrong_abi() -> i32 { 0 }
    "#;
    let funcs = parse_ffi_exports_from_str(source, PathBuf::from("test.rs")).unwrap();
    assert!(
        funcs.is_empty(),
        "function without extern \"C\" should be ignored"
    );
}

#[test]
fn test_function_with_multiple_attributes() {
    let source = r#"
        #[doc = "Documentation"]
        #[no_mangle]
        #[allow(unused)]
        pub unsafe extern "C" fn multi_attr_fn() -> i32 { 0 }
    "#;
    let funcs = parse_ffi_exports_from_str(source, PathBuf::from("test.rs")).unwrap();
    assert_eq!(funcs.len(), 1);
    assert_eq!(funcs[0].name, "multi_attr_fn");
}

#[test]
fn test_no_mangle_attribute_position_varies() {
    let source = r#"
        #[no_mangle]
        #[doc = "After no_mangle"]
        pub unsafe extern "C" fn attr_first() -> i32 { 0 }

        #[allow(unused)]
        #[no_mangle]
        pub unsafe extern "C" fn attr_middle() -> i32 { 0 }

        #[doc = "Before no_mangle"]
        #[allow(unused)]
        #[no_mangle]
        pub unsafe extern "C" fn attr_last() -> i32 { 0 }
    "#;
    let funcs = parse_ffi_exports_from_str(source, PathBuf::from("test.rs")).unwrap();
    assert_eq!(funcs.len(), 3);
}

// ============================================================================
// Parameter Type Tests
// ============================================================================

#[test]
fn test_parse_const_pointer_param() {
    let source = r#"
        use std::ffi::c_char;

        #[no_mangle]
        pub unsafe extern "C" fn with_const_ptr(s: *const c_char) -> i32 { 0 }
    "#;
    let funcs = parse_ffi_exports_from_str(source, PathBuf::from("test.rs")).unwrap();
    assert_eq!(funcs.len(), 1);

    let param = &funcs[0].params[0];
    assert_eq!(param.name, "s");
    assert_eq!(param.rust_type, "*const c_char");
    assert!(param.is_pointer);
    assert!(!param.is_mutable);
}

#[test]
fn test_parse_mut_pointer_param() {
    let source = r#"
        use std::ffi::c_char;

        #[no_mangle]
        pub unsafe extern "C" fn with_mut_ptr(s: *mut c_char) { }
    "#;
    let funcs = parse_ffi_exports_from_str(source, PathBuf::from("test.rs")).unwrap();
    assert_eq!(funcs.len(), 1);

    let param = &funcs[0].params[0];
    assert_eq!(param.rust_type, "*mut c_char");
    assert!(param.is_pointer);
    assert!(param.is_mutable);
}

#[test]
fn test_parse_double_pointer_param() {
    let source = r#"
        use std::ffi::c_char;

        #[no_mangle]
        pub unsafe extern "C" fn with_double_ptr(err: *mut *mut c_char) -> i32 { 0 }
    "#;
    let funcs = parse_ffi_exports_from_str(source, PathBuf::from("test.rs")).unwrap();
    assert_eq!(funcs.len(), 1);

    let param = &funcs[0].params[0];
    assert_eq!(param.name, "err");
    assert_eq!(param.rust_type, "*mut *mut c_char");
    assert!(param.is_pointer);
    assert!(param.is_mutable);
}

#[test]
fn test_parse_primitive_params() {
    let source = r#"
        #[no_mangle]
        pub unsafe extern "C" fn primitives(a: i32, b: u32, c: i64, d: bool) -> i32 { 0 }
    "#;
    let funcs = parse_ffi_exports_from_str(source, PathBuf::from("test.rs")).unwrap();
    assert_eq!(funcs.len(), 1);
    assert_eq!(funcs[0].params.len(), 4);

    assert_eq!(funcs[0].params[0].rust_type, "i32");
    assert!(!funcs[0].params[0].is_pointer);

    assert_eq!(funcs[0].params[1].rust_type, "u32");
    assert_eq!(funcs[0].params[2].rust_type, "i64");
    assert_eq!(funcs[0].params[3].rust_type, "bool");
}

#[test]
fn test_parse_usize_param() {
    let source = r#"
        #[no_mangle]
        pub unsafe extern "C" fn with_usize(len: usize) -> usize { len }
    "#;
    let funcs = parse_ffi_exports_from_str(source, PathBuf::from("test.rs")).unwrap();
    assert_eq!(funcs[0].params[0].rust_type, "usize");
}

#[test]
fn test_parse_option_callback_param() {
    let source = r#"
        #[no_mangle]
        pub unsafe extern "C" fn with_callback(
            cb: Option<unsafe extern "C" fn(*const u8, usize)>,
        ) -> i32 { 0 }
    "#;
    let funcs = parse_ffi_exports_from_str(source, PathBuf::from("test.rs")).unwrap();
    assert_eq!(funcs.len(), 1);

    let param = &funcs[0].params[0];
    assert_eq!(param.name, "cb");
    assert!(param.rust_type.contains("Option"));
}

#[test]
fn test_parse_multiple_params() {
    let source = r#"
        use std::ffi::c_char;

        #[no_mangle]
        pub unsafe extern "C" fn multi_params(
            id: i32,
            name: *const c_char,
            buffer: *mut u8,
            len: usize,
        ) -> i32 { 0 }
    "#;
    let funcs = parse_ffi_exports_from_str(source, PathBuf::from("test.rs")).unwrap();
    assert_eq!(funcs.len(), 1);
    assert_eq!(funcs[0].params.len(), 4);

    assert_eq!(funcs[0].params[0].name, "id");
    assert_eq!(funcs[0].params[1].name, "name");
    assert_eq!(funcs[0].params[2].name, "buffer");
    assert_eq!(funcs[0].params[3].name, "len");
}

// ============================================================================
// Return Type Tests
// ============================================================================

#[test]
fn test_parse_void_return() {
    let source = r#"
        #[no_mangle]
        pub unsafe extern "C" fn void_return() { }
    "#;
    let funcs = parse_ffi_exports_from_str(source, PathBuf::from("test.rs")).unwrap();
    assert_eq!(funcs[0].return_type, ParsedType::Unit);
}

#[test]
fn test_parse_i32_return() {
    let source = r#"
        #[no_mangle]
        pub unsafe extern "C" fn i32_return() -> i32 { 0 }
    "#;
    let funcs = parse_ffi_exports_from_str(source, PathBuf::from("test.rs")).unwrap();
    assert_eq!(
        funcs[0].return_type,
        ParsedType::Primitive("i32".to_string())
    );
}

#[test]
fn test_parse_u32_return() {
    let source = r#"
        #[no_mangle]
        pub unsafe extern "C" fn u32_return() -> u32 { 0 }
    "#;
    let funcs = parse_ffi_exports_from_str(source, PathBuf::from("test.rs")).unwrap();
    assert_eq!(
        funcs[0].return_type,
        ParsedType::Primitive("u32".to_string())
    );
}

#[test]
fn test_parse_const_pointer_return() {
    let source = r#"
        use std::ffi::c_char;

        #[no_mangle]
        pub unsafe extern "C" fn const_ptr_return() -> *const c_char {
            std::ptr::null()
        }
    "#;
    let funcs = parse_ffi_exports_from_str(source, PathBuf::from("test.rs")).unwrap();
    assert_eq!(
        funcs[0].return_type,
        ParsedType::Pointer {
            target: "c_char".to_string(),
            is_mut: false,
        }
    );
}

#[test]
fn test_parse_mut_pointer_return() {
    let source = r#"
        use std::ffi::c_char;

        #[no_mangle]
        pub unsafe extern "C" fn mut_ptr_return() -> *mut c_char {
            std::ptr::null_mut()
        }
    "#;
    let funcs = parse_ffi_exports_from_str(source, PathBuf::from("test.rs")).unwrap();
    assert_eq!(
        funcs[0].return_type,
        ParsedType::Pointer {
            target: "c_char".to_string(),
            is_mut: true,
        }
    );
}

// ============================================================================
// Real-World FFI Pattern Tests
// ============================================================================

#[test]
fn test_parse_keyrx_init_pattern() {
    let source = r#"
        #[no_mangle]
        pub unsafe extern "C" fn keyrx_init() -> i32 {
            0
        }
    "#;
    let funcs = parse_ffi_exports_from_str(source, PathBuf::from("exports.rs")).unwrap();
    assert_eq!(funcs.len(), 1);
    assert_eq!(funcs[0].name, "keyrx_init");
    assert_eq!(funcs[0].params.len(), 0);
    assert_eq!(
        funcs[0].return_type,
        ParsedType::Primitive("i32".to_string())
    );
}

#[test]
fn test_parse_keyrx_version_pattern() {
    let source = r#"
        use std::ffi::c_char;

        #[no_mangle]
        pub unsafe extern "C" fn keyrx_version() -> *mut c_char {
            std::ptr::null_mut()
        }
    "#;
    let funcs = parse_ffi_exports_from_str(source, PathBuf::from("exports.rs")).unwrap();
    assert_eq!(funcs.len(), 1);
    assert_eq!(funcs[0].name, "keyrx_version");
    assert_eq!(
        funcs[0].return_type,
        ParsedType::Pointer {
            target: "c_char".to_string(),
            is_mut: true,
        }
    );
}

#[test]
fn test_parse_keyrx_free_string_pattern() {
    let source = r#"
        use std::ffi::c_char;

        #[no_mangle]
        pub unsafe extern "C" fn keyrx_free_string(s: *mut c_char) {
            // free the string
        }
    "#;
    let funcs = parse_ffi_exports_from_str(source, PathBuf::from("exports.rs")).unwrap();
    assert_eq!(funcs.len(), 1);
    assert_eq!(funcs[0].name, "keyrx_free_string");
    assert_eq!(funcs[0].params.len(), 1);
    assert_eq!(funcs[0].params[0].rust_type, "*mut c_char");
    assert_eq!(funcs[0].return_type, ParsedType::Unit);
}

#[test]
fn test_parse_keyrx_register_callback_pattern() {
    let source = r#"
        #[no_mangle]
        pub unsafe extern "C" fn keyrx_register_event_callback(
            event_type: i32,
            callback: Option<unsafe extern "C" fn(*const u8, usize)>,
        ) -> i32 {
            0
        }
    "#;
    let funcs = parse_ffi_exports_from_str(source, PathBuf::from("exports.rs")).unwrap();
    assert_eq!(funcs.len(), 1);
    assert_eq!(funcs[0].name, "keyrx_register_event_callback");
    assert_eq!(funcs[0].params.len(), 2);
}

#[test]
fn test_parse_keyrx_free_event_payload_pattern() {
    let source = r#"
        #[no_mangle]
        pub unsafe extern "C" fn keyrx_free_event_payload(ptr: *mut u8, len: usize) {
            // free the payload
        }
    "#;
    let funcs = parse_ffi_exports_from_str(source, PathBuf::from("exports.rs")).unwrap();
    assert_eq!(funcs.len(), 1);
    assert_eq!(funcs[0].name, "keyrx_free_event_payload");
    assert_eq!(funcs[0].params.len(), 2);
    assert_eq!(funcs[0].params[0].rust_type, "*mut u8");
    assert_eq!(funcs[0].params[1].rust_type, "usize");
}

#[test]
fn test_parse_config_save_pattern() {
    let source = r#"
        use std::ffi::c_char;

        #[no_mangle]
        pub unsafe extern "C" fn keyrx_config_save_keymap(json: *const c_char) -> *mut c_char {
            std::ptr::null_mut()
        }
    "#;
    let funcs = parse_ffi_exports_from_str(source, PathBuf::from("exports.rs")).unwrap();
    assert_eq!(funcs.len(), 1);
    assert_eq!(funcs[0].name, "keyrx_config_save_keymap");
    assert_eq!(funcs[0].params.len(), 1);
    assert_eq!(funcs[0].params[0].name, "json");
    assert_eq!(funcs[0].params[0].rust_type, "*const c_char");
    assert_eq!(
        funcs[0].return_type,
        ParsedType::Pointer {
            target: "c_char".to_string(),
            is_mut: true,
        }
    );
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_parse_invalid_syntax_returns_error() {
    let source = r#"
        // Invalid Rust syntax - missing closing brace
        pub fn broken( {
    "#;
    let result = parse_ffi_exports_from_str(source, PathBuf::from("broken.rs"));
    assert!(result.is_err());
}

#[test]
fn test_parse_error_is_syn_error() {
    let source = "fn broken(";
    let result = parse_ffi_exports_from_str(source, PathBuf::from("test.rs"));
    match result {
        Err(ParseError::SynError(_)) => {} // expected
        _ => panic!("expected SynError"),
    }
}

#[test]
fn test_parse_incomplete_function() {
    let source = r#"
        #[no_mangle]
        pub unsafe extern "C" fn incomplete
    "#;
    let result = parse_ffi_exports_from_str(source, PathBuf::from("test.rs"));
    assert!(result.is_err(), "incomplete function should fail to parse");
}

#[test]
fn test_parse_unclosed_string() {
    let source = r#"
        #[no_mangle]
        pub unsafe extern "C" fn bad() -> i32 {
            let s = "unclosed string;
            0
        }
    "#;
    let result = parse_ffi_exports_from_str(source, PathBuf::from("test.rs"));
    assert!(result.is_err(), "unclosed string should fail to parse");
}

// ============================================================================
// Multiple Functions Tests
// ============================================================================

#[test]
fn test_parse_multiple_functions_preserves_order() {
    let source = r#"
        #[no_mangle]
        pub unsafe extern "C" fn alpha() -> i32 { 0 }

        #[no_mangle]
        pub unsafe extern "C" fn beta() -> i32 { 0 }

        #[no_mangle]
        pub unsafe extern "C" fn gamma() -> i32 { 0 }
    "#;
    let funcs = parse_ffi_exports_from_str(source, PathBuf::from("test.rs")).unwrap();
    assert_eq!(funcs.len(), 3);
    assert_eq!(funcs[0].name, "alpha");
    assert_eq!(funcs[1].name, "beta");
    assert_eq!(funcs[2].name, "gamma");
}

#[test]
fn test_parse_mixed_ffi_and_regular_functions() {
    let source = r#"
        fn helper() -> i32 { 42 }

        #[no_mangle]
        pub unsafe extern "C" fn ffi_func() -> i32 { helper() }

        fn another_helper() {}

        #[no_mangle]
        pub unsafe extern "C" fn another_ffi() {}
    "#;
    let funcs = parse_ffi_exports_from_str(source, PathBuf::from("test.rs")).unwrap();
    assert_eq!(funcs.len(), 2);
    assert_eq!(funcs[0].name, "ffi_func");
    assert_eq!(funcs[1].name, "another_ffi");
}

// ============================================================================
// ParsedFunction Helper Method Tests
// ============================================================================

#[test]
fn test_parsed_function_param_count() {
    let func = ParsedFunction::new(
        "test".to_string(),
        vec![
            ParsedParam::new("a".to_string(), "i32".to_string(), false, false),
            ParsedParam::new("b".to_string(), "i32".to_string(), false, false),
            ParsedParam::new("c".to_string(), "i32".to_string(), false, false),
        ],
        ParsedType::Unit,
        PathBuf::from("test.rs"),
        1,
    );
    assert_eq!(func.param_count(), 3);
}

#[test]
fn test_parsed_function_has_error_pointer_true() {
    let func = ParsedFunction::new(
        "test".to_string(),
        vec![
            ParsedParam::new(
                "input".to_string(),
                "*const c_char".to_string(),
                true,
                false,
            ),
            ParsedParam::new(
                "error".to_string(),
                "*mut *mut c_char".to_string(),
                true,
                true,
            ),
        ],
        ParsedType::Primitive("i32".to_string()),
        PathBuf::from("test.rs"),
        1,
    );
    assert!(func.has_error_pointer());
}

#[test]
fn test_parsed_function_has_error_pointer_false() {
    let func = ParsedFunction::new(
        "test".to_string(),
        vec![ParsedParam::new(
            "input".to_string(),
            "*const c_char".to_string(),
            true,
            false,
        )],
        ParsedType::Unit,
        PathBuf::from("test.rs"),
        1,
    );
    assert!(!func.has_error_pointer());
}

#[test]
fn test_parsed_function_has_error_pointer_empty_params() {
    let func = ParsedFunction::new(
        "test".to_string(),
        vec![],
        ParsedType::Unit,
        PathBuf::from("test.rs"),
        1,
    );
    assert!(!func.has_error_pointer());
}

// ============================================================================
// ParsedType Display Tests
// ============================================================================

#[test]
fn test_parsed_type_to_string_unit() {
    assert_eq!(ParsedType::Unit.to_type_string(), "()");
}

#[test]
fn test_parsed_type_to_string_primitive() {
    assert_eq!(
        ParsedType::Primitive("i32".to_string()).to_type_string(),
        "i32"
    );
}

#[test]
fn test_parsed_type_to_string_const_pointer() {
    let t = ParsedType::Pointer {
        target: "c_char".to_string(),
        is_mut: false,
    };
    assert_eq!(t.to_type_string(), "*const c_char");
}

#[test]
fn test_parsed_type_to_string_mut_pointer() {
    let t = ParsedType::Pointer {
        target: "c_char".to_string(),
        is_mut: true,
    };
    assert_eq!(t.to_type_string(), "*mut c_char");
}

// ============================================================================
// Line Number Tests
// ============================================================================

#[test]
fn test_line_number_tracking() {
    let source = r#"// Line 1
// Line 2
// Line 3
#[no_mangle]
pub unsafe extern "C" fn on_line_five() -> i32 { 0 }
"#;
    let funcs = parse_ffi_exports_from_str(source, PathBuf::from("test.rs")).unwrap();
    assert_eq!(funcs.len(), 1);
    // Line numbers are 1-indexed, function is on line 5
    assert!(funcs[0].line_number >= 4 && funcs[0].line_number <= 6);
}

// ============================================================================
// File Path Tests
// ============================================================================

#[test]
fn test_file_path_preserved() {
    let source = r#"
        #[no_mangle]
        pub unsafe extern "C" fn test_fn() -> i32 { 0 }
    "#;
    let path = PathBuf::from("/path/to/source/exports.rs");
    let funcs = parse_ffi_exports_from_str(source, path.clone()).unwrap();
    assert_eq!(funcs[0].file_path, path);
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_function_name_with_underscores() {
    let source = r#"
        #[no_mangle]
        pub unsafe extern "C" fn __keyrx__internal__func__() -> i32 { 0 }
    "#;
    let funcs = parse_ffi_exports_from_str(source, PathBuf::from("test.rs")).unwrap();
    assert_eq!(funcs[0].name, "__keyrx__internal__func__");
}

#[test]
fn test_qualified_type_path() {
    let source = r#"
        #[no_mangle]
        pub unsafe extern "C" fn with_qualified_type(ptr: *const std::ffi::c_char) -> i32 { 0 }
    "#;
    let funcs = parse_ffi_exports_from_str(source, PathBuf::from("test.rs")).unwrap();
    assert_eq!(funcs.len(), 1);
    // The type should be parsed (exact format may vary based on implementation)
    assert!(funcs[0].params[0].is_pointer);
}

#[test]
fn test_param_with_underscore_prefix() {
    let source = r#"
        use std::ffi::c_char;

        #[no_mangle]
        pub unsafe extern "C" fn with_unused_param(_unused: *const c_char) -> i32 { 0 }
    "#;
    let funcs = parse_ffi_exports_from_str(source, PathBuf::from("test.rs")).unwrap();
    assert_eq!(funcs[0].params[0].name, "_unused");
}

#[test]
fn test_parse_function_without_unsafe() {
    // Functions without `unsafe` keyword should still be parsed if they meet other criteria
    let source = r#"
        #[no_mangle]
        pub extern "C" fn safe_ffi() -> i32 { 0 }
    "#;
    let funcs = parse_ffi_exports_from_str(source, PathBuf::from("test.rs")).unwrap();
    assert_eq!(funcs.len(), 1);
    assert_eq!(funcs[0].name, "safe_ffi");
}
