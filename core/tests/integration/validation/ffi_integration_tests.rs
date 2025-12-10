#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::print_stdout, clippy::print_stderr, clippy::field_reassign_with_default, clippy::useless_conversion, clippy::assertions_on_constants, clippy::manual_div_ceil, clippy::manual_strip, clippy::len_zero, clippy::redundant_closure, clippy::manual_range_contains, clippy::default_constructed_unit_structs, clippy::clone_on_copy, clippy::io_other_error, clippy::bool_assert_comparison, clippy::approx_constant, clippy::let_unit_value, clippy::while_let_on_iterator, clippy::await_holding_lock, clippy::unnecessary_cast, clippy::drop_non_drop, clippy::needless_range_loop, unused_imports, unused_variables, dead_code, unsafe_code, clippy::collapsible_if, clippy::bool_comparison, unexpected_cfgs)]
//! FFI integration tests for validation.
//!
//! Tests FFI bindings for validation including:
//! - C API validation function
//! - Null pointer handling
//! - JSON response format
//! - Memory management

use keyrx_core::ffi::{keyrx_free_string, keyrx_validate_script};
use std::ffi::{CStr, CString};
use std::ptr;

fn parse_ffi_response(ptr: *mut std::ffi::c_char) -> (bool, String) {
    assert!(!ptr.is_null());
    let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
    unsafe { keyrx_free_string(ptr) };

    if let Some(payload) = raw.strip_prefix("ok:") {
        (true, payload.to_string())
    } else if let Some(msg) = raw.strip_prefix("error:") {
        (false, msg.to_string())
    } else {
        (false, raw)
    }
}

#[test]
fn ffi_validates_valid_script() {
    let script = CString::new(r#"remap("CapsLock", "Escape");"#).unwrap();
    let ptr = unsafe { keyrx_validate_script(script.as_ptr()) };
    let (ok, payload) = parse_ffi_response(ptr);

    assert!(ok);
    let result: serde_json::Value = serde_json::from_str(&payload).unwrap();
    assert_eq!(result["is_valid"], true);
}

#[test]
fn ffi_detects_errors() {
    let script = CString::new(r#"layer_push("undefined_layer");"#).unwrap();
    let ptr = unsafe { keyrx_validate_script(script.as_ptr()) };
    let (ok, payload) = parse_ffi_response(ptr);

    assert!(ok);
    let result: serde_json::Value = serde_json::from_str(&payload).unwrap();
    assert_eq!(result["is_valid"], false);
    assert!(!result["errors"].as_array().unwrap().is_empty());
}

#[test]
fn ffi_handles_null_pointer() {
    let ptr = unsafe { keyrx_validate_script(ptr::null()) };
    let (ok, _) = parse_ffi_response(ptr);
    assert!(!ok);
}

#[test]
fn ffi_json_is_parseable() {
    let script = CString::new(
        r#"
        remap("A", "B");
        block("C");
    "#,
    )
    .unwrap();
    let ptr = unsafe { keyrx_validate_script(script.as_ptr()) };
    let (ok, payload) = parse_ffi_response(ptr);

    assert!(ok);
    let result: serde_json::Value = serde_json::from_str(&payload).unwrap();
    assert_eq!(result["is_valid"], true);
    // Coverage is optional in FFI response
    if let Some(coverage) = result.get("coverage") {
        assert!(coverage.is_object());
    }
}
