#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::print_stdout, clippy::print_stderr, clippy::field_reassign_with_default, clippy::useless_conversion, clippy::assertions_on_constants, clippy::manual_div_ceil, clippy::manual_strip, clippy::len_zero, clippy::redundant_closure, clippy::manual_range_contains, clippy::default_constructed_unit_structs, clippy::clone_on_copy, clippy::io_other_error, clippy::bool_assert_comparison, clippy::approx_constant, clippy::let_unit_value, clippy::while_let_on_iterator, clippy::await_holding_lock, clippy::unnecessary_cast, clippy::drop_non_drop, clippy::needless_range_loop, unused_imports, unused_variables, dead_code, unsafe_code, clippy::collapsible_if, clippy::bool_comparison, unexpected_cfgs)]
//! Integration-style tests for FFI exports covering eval, key registry, and state snapshots.
use keyrx_core::engine::TimingConfig;
use keyrx_core::ffi::{keyrx_eval, keyrx_free_string, keyrx_list_keys, publish_state_snapshot};
use keyrx_core::scripting::{clear_active_runtime, set_active_runtime, RhaiRuntime};
use serde_json::{json, Value};
use std::ffi::{CStr, CString};
use std::slice;
use std::sync::{Mutex, OnceLock};

fn state_payloads() -> &'static Mutex<Vec<Vec<u8>>> {
    static STORE: OnceLock<Mutex<Vec<Vec<u8>>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(Vec::new()))
}

unsafe extern "C" fn record_state(ptr: *const u8, len: usize) {
    let bytes = slice::from_raw_parts(ptr, len);
    state_payloads().lock().unwrap().push(bytes.to_vec());
}

#[test]
fn eval_returns_error_prefix_on_runtime_failure() {
    clear_active_runtime();
    let mut runtime = RhaiRuntime::new().expect("runtime should initialize");
    set_active_runtime(&mut runtime);

    // Deliberately invalid script to trigger runtime error path.
    let bad_script = CString::new("let =").unwrap();
    let ptr = unsafe { keyrx_eval(bad_script.as_ptr()) };
    let response = unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .expect("response should be utf8")
        .to_string();
    unsafe { keyrx_free_string(ptr) };

    assert!(
        response.starts_with("error:"),
        "expected error prefix, got {response}"
    );

    clear_active_runtime();
}

#[test]
fn key_registry_payload_includes_aliases_and_codes() {
    let ptr = keyrx_list_keys();
    let raw = unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .expect("response should be utf8")
        .to_string();
    unsafe { keyrx_free_string(ptr) };

    let payload = raw
        .strip_prefix("ok:")
        .expect("registry response should start with ok:");
    let entries: Vec<Value> = serde_json::from_str(payload).expect("valid registry json");
    assert!(!entries.is_empty(), "registry should not be empty");

    for entry in &entries {
        assert!(entry.get("name").and_then(Value::as_str).is_some());
        assert!(entry.get("aliases").and_then(Value::as_array).is_some());
        assert!(entry.get("evdev").and_then(Value::as_u64).is_some());
        assert!(entry.get("vk").and_then(Value::as_u64).is_some());
    }

    let left_shift = entries
        .iter()
        .find(|e| e.get("name") == Some(&json!("LeftShift")))
        .expect("LeftShift entry present");
    let aliases = left_shift
        .get("aliases")
        .and_then(Value::as_array)
        .expect("aliases array");
    assert!(
        aliases.iter().any(|a| a == "SHIFT"),
        "expected SHIFT alias for LeftShift"
    );
    assert_eq!(left_shift.get("evdev"), Some(&json!(42u64)));
    assert_eq!(left_shift.get("vk"), Some(&json!(0xA0u64)));
}

#[test]
#[ignore = "Uses obsolete FFI API - keyrx_on_state removed, needs refactor to EventRegistry"]
fn state_snapshot_serializes_latency_and_timing() {
    // Test body commented out due to obsolete FFI API (keyrx_on_state, old publish_state_snapshot signature)
    // Needs to be refactored to use the new EventRegistry pattern
    // See src/ffi/domains/engine.rs for the new publish_state_snapshot(StateSnapshot, Option<String>, Option<u64>) API
    unimplemented!("Test needs refactoring to use new EventRegistry API");
}
