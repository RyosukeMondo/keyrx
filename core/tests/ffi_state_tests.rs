//! Integration-style tests for FFI exports covering eval, key registry, and state snapshots.
use keyrx_core::engine::TimingConfig;
use keyrx_core::ffi::{
    keyrx_eval, keyrx_free_string, keyrx_list_keys, keyrx_on_state, publish_state_snapshot,
};
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
fn state_snapshot_serializes_latency_and_timing() {
    state_payloads().lock().unwrap().clear();
    keyrx_on_state(Some(record_state));

    // keyrx_on_state emits an initial snapshot immediately.
    let initial = state_payloads().lock().unwrap().clone();
    assert_eq!(initial.len(), 1, "initial snapshot should be emitted");
    let initial_json: Value = serde_json::from_slice(&initial[0]).expect("valid initial json");
    assert_eq!(initial_json.get("event"), Some(&json!("engine_ready")));

    // Now capture a custom snapshot and assert fields.
    state_payloads().lock().unwrap().clear();
    let timing = TimingConfig {
        tap_timeout_ms: 150,
        combo_timeout_ms: 60,
        hold_delay_ms: 10,
        eager_tap: true,
        permissive_hold: false,
        retro_tap: true,
    };
    publish_state_snapshot(
        vec!["fn".into()],
        vec!["1".into(), "3".into()],
        vec!["KeyA".into()],
        vec!["pending:KeyB".into()],
        Some("decision".into()),
        Some(1234),
        timing.clone(),
    );

    let payloads = state_payloads().lock().unwrap();
    assert_eq!(payloads.len(), 1, "expected one custom snapshot");
    let snapshot: Value = serde_json::from_slice(&payloads[0]).expect("valid snapshot json");
    assert_eq!(snapshot.get("layers"), Some(&json!(["fn"])));
    assert_eq!(snapshot.get("modifiers"), Some(&json!(["1", "3"])));
    assert_eq!(snapshot.get("held"), Some(&json!(["KeyA"])));
    assert_eq!(snapshot.get("pending"), Some(&json!(["pending:KeyB"])));
    assert_eq!(snapshot.get("event"), Some(&json!("decision")));
    assert_eq!(snapshot.get("latency_us"), Some(&json!(1234)));
    assert_eq!(snapshot.get("timing"), Some(&json!(timing)));

    keyrx_on_state(None);
}
