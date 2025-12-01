//! C-ABI exports for FFI.
//!
//! This module provides C-compatible functions for FFI integration.
//! Unsafe code is required for FFI interoperability.
#![allow(unsafe_code)]

use crate::discovery::{session::set_session_update_sink, SessionUpdate};
use crate::drivers::keycodes::key_definitions;
use crate::engine::TimingConfig;
use crate::scripting::with_active_runtime;
use crate::traits::ScriptRuntime;
use serde::Serialize;
use std::ffi::{c_char, CStr, CString};
use std::path::Path;
use std::ptr;
use std::sync::{Arc, Mutex, OnceLock};

/// Initialize the KeyRx engine.
///
/// # Safety
/// This function is safe to call from any thread.
#[no_mangle]
pub extern "C" fn keyrx_init() -> i32 {
    let _ = tracing_subscriber::fmt::try_init();
    tracing::info!(
        service = "keyrx",
        event = "ffi_init",
        component = "ffi_exports",
        status = "ok",
        "KeyRx Core initialized"
    );
    0 // Success
}

/// Get the version string.
///
/// # Safety
/// The returned pointer is valid until the next call to this function.
#[no_mangle]
pub extern "C" fn keyrx_version() -> *const c_char {
    static VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "\0");
    VERSION.as_ptr() as *const c_char
}

/// Load a script file.
///
/// # Safety
/// `path` must be a valid null-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn keyrx_load_script(path: *const c_char) -> i32 {
    if path.is_null() {
        return -1;
    }

    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };

    let script_name = Path::new(path_str)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("<unknown>");

    tracing::info!(
        service = "keyrx",
        event = "ffi_load_script",
        component = "ffi_exports",
        script = script_name,
        "Loading script"
    );
    // TODO: Actually load the script
    0
}

/// Free a string allocated by KeyRx.
///
/// # Safety
/// `ptr` must be a pointer returned by a KeyRx function, or null.
#[no_mangle]
pub unsafe extern "C" fn keyrx_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(CString::from_raw(ptr));
    }
}

/// Evaluate a console command against the active runtime.
///
/// Responses are prefixed with `ok:` or `error:`. Callers must free the returned pointer with
/// `keyrx_free_string`.
///
/// # Safety
/// `command` must be a valid, null-terminated UTF-8 string pointer or null.
#[no_mangle]
pub unsafe extern "C" fn keyrx_eval(command: *const c_char) -> *mut c_char {
    if command.is_null() {
        return CString::new("error: null command")
            .map_or_else(|_| ptr::null_mut(), CString::into_raw);
    }

    let cmd_str = match CStr::from_ptr(command).to_str() {
        Ok(s) => s,
        Err(_) => {
            return CString::new("error: invalid utf8")
                .map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    let response = match with_active_runtime(|runtime| runtime.execute(cmd_str)) {
        Some(Ok(_)) => "ok:".to_string(),
        Some(Err(err)) => format!("error: {err}"),
        None => "error: engine not initialized".to_string(),
    };

    CString::new(response).map_or_else(|_| ptr::null_mut(), CString::into_raw)
}

type DiscoveryEventCallback = unsafe extern "C" fn(*const u8, usize);
type StateEventCallback = unsafe extern "C" fn(*const u8, usize);

fn progress_callback() -> &'static Mutex<Option<DiscoveryEventCallback>> {
    static SLOT: OnceLock<Mutex<Option<DiscoveryEventCallback>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

fn duplicate_callback() -> &'static Mutex<Option<DiscoveryEventCallback>> {
    static SLOT: OnceLock<Mutex<Option<DiscoveryEventCallback>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

fn summary_callback() -> &'static Mutex<Option<DiscoveryEventCallback>> {
    static SLOT: OnceLock<Mutex<Option<DiscoveryEventCallback>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

fn state_callback() -> &'static Mutex<Option<StateEventCallback>> {
    static SLOT: OnceLock<Mutex<Option<StateEventCallback>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

fn any_callback_registered() -> bool {
    [
        progress_callback(),
        duplicate_callback(),
        summary_callback(),
    ]
    .iter()
    .any(|slot| slot.lock().map(|guard| guard.is_some()).unwrap_or(false))
}

fn register_callback(
    slot: &'static Mutex<Option<DiscoveryEventCallback>>,
    callback: Option<DiscoveryEventCallback>,
) {
    if let Ok(mut guard) = slot.lock() {
        *guard = callback;
    }
    refresh_discovery_sink();
}

/// Register a callback for discovery progress updates.
/// The provided pointer/length pair references a JSON payload that is only valid for the duration of the callback.
#[no_mangle]
pub extern "C" fn keyrx_on_discovery_progress(callback: Option<DiscoveryEventCallback>) {
    register_callback(progress_callback(), callback);
}

/// Register a callback for duplicate key warnings during discovery.
/// The provided pointer/length pair references a JSON payload that is only valid for the duration of the callback.
#[no_mangle]
pub extern "C" fn keyrx_on_discovery_duplicate(callback: Option<DiscoveryEventCallback>) {
    register_callback(duplicate_callback(), callback);
}

/// Register a callback for discovery summaries (completed, cancelled, or bypassed).
/// The provided pointer/length pair references a JSON payload that is only valid for the duration of the callback.
#[no_mangle]
pub extern "C" fn keyrx_on_discovery_summary(callback: Option<DiscoveryEventCallback>) {
    register_callback(summary_callback(), callback);
}

/// Register a callback for engine state snapshots.
///
/// The payload is a JSON blob with fields: layers, modifiers, held, pending, event, latency_us, timing.
#[no_mangle]
pub extern "C" fn keyrx_on_state(callback: Option<StateEventCallback>) {
    if let Ok(mut guard) = state_callback().lock() {
        *guard = callback;
    }

    emit_state_snapshot(FfiState {
        layers: vec!["base".into()],
        modifiers: Vec::new(),
        held: Vec::new(),
        pending: Vec::new(),
        event: Some("engine_ready".into()),
        latency_us: Some(0),
        timing: TimingConfig::default(),
    });
}

/// Return canonical key registry as `ok:<json>` (or `error:<message>`).
///
/// Caller must free with `keyrx_free_string`.
#[no_mangle]
pub extern "C" fn keyrx_list_keys() -> *mut c_char {
    let payload = serde_json::to_string(&key_definitions())
        .map(|json| format!("ok:{json}"))
        .unwrap_or_else(|err| format!("error:{err}"));

    CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw)
}

fn refresh_discovery_sink() {
    if any_callback_registered() {
        set_session_update_sink(Some(discovery_sink()));
    } else {
        set_session_update_sink(None);
    }
}

#[derive(Serialize)]
struct FfiState {
    layers: Vec<String>,
    modifiers: Vec<String>,
    held: Vec<String>,
    pending: Vec<String>,
    event: Option<String>,
    latency_us: Option<u64>,
    timing: TimingConfig,
}

fn emit_state_snapshot(state: FfiState) {
    let callback = state_callback().lock().ok().and_then(|guard| *guard);

    let Some(cb) = callback else { return };

    match serde_json::to_vec(&state) {
        Ok(bytes) => unsafe {
            cb(bytes.as_ptr(), bytes.len());
        },
        Err(err) => tracing::warn!(
            service = "keyrx",
            component = "ffi_exports",
            event = "state",
            error = %err,
            "Failed to serialize state payload for FFI"
        ),
    }
}

/// Expose a safe-ish API for internal callers to emit state snapshots to the FFI listeners.
/// Engine subsystems can call this when state changes.
pub fn publish_state_snapshot(
    layers: Vec<String>,
    modifiers: Vec<String>,
    held: Vec<String>,
    pending: Vec<String>,
    event: Option<String>,
    latency_us: Option<u64>,
    timing: TimingConfig,
) {
    emit_state_snapshot(FfiState {
        layers,
        modifiers,
        held,
        pending,
        event,
        latency_us,
        timing,
    });
}

fn discovery_sink() -> Arc<dyn Fn(&SessionUpdate) + Send + Sync + 'static> {
    Arc::new(|update| match update {
        SessionUpdate::Ignored => {}
        SessionUpdate::Progress(progress) => {
            serialize_and_invoke(progress_callback(), progress, "progress")
        }
        SessionUpdate::Duplicate(dup) => {
            serialize_and_invoke(duplicate_callback(), dup, "duplicate")
        }
        SessionUpdate::Finished(summary) => {
            serialize_and_invoke(summary_callback(), summary, "summary")
        }
    })
}

fn serialize_and_invoke<T: Serialize>(
    slot: &'static Mutex<Option<DiscoveryEventCallback>>,
    payload: &T,
    event: &'static str,
) {
    let callback = slot.lock().ok().and_then(|guard| *guard);

    let Some(cb) = callback else {
        return;
    };

    match serde_json::to_vec(payload) {
        Ok(bytes) => unsafe {
            cb(bytes.as_ptr(), bytes.len());
        },
        Err(err) => tracing::warn!(
            service = "keyrx",
            component = "ffi_exports",
            event,
            error = %err,
            "Failed to serialize discovery payload for FFI"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::{
        session::publish_session_update, DeviceId, DiscoveryProgress, DiscoverySummary,
        ExpectedPosition, PhysicalKey, SessionStatus,
    };
    use crate::drivers::keycodes::KeyCode;
    use crate::engine::RemapAction;
    use crate::scripting::{clear_active_runtime, set_active_runtime, RhaiRuntime};
    use std::collections::HashMap;
    use std::ptr;
    use std::slice;
    use std::sync::Mutex;

    fn progress_store() -> &'static Mutex<Vec<Vec<u8>>> {
        static STORE: OnceLock<Mutex<Vec<Vec<u8>>>> = OnceLock::new();
        STORE.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn summary_store() -> &'static Mutex<Vec<Vec<u8>>> {
        static STORE: OnceLock<Mutex<Vec<Vec<u8>>>> = OnceLock::new();
        STORE.get_or_init(|| Mutex::new(Vec::new()))
    }

    unsafe extern "C" fn record_progress(ptr: *const u8, len: usize) {
        let slice = slice::from_raw_parts(ptr, len);
        progress_store().lock().unwrap().push(slice.to_vec());
    }

    unsafe extern "C" fn record_summary(ptr: *const u8, len: usize) {
        let slice = slice::from_raw_parts(ptr, len);
        summary_store().lock().unwrap().push(slice.to_vec());
    }

    #[test]
    fn list_keys_returns_registry_objects() {
        let ptr = keyrx_list_keys();
        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };

        assert!(raw.starts_with("ok:"));
        let payload = &raw["ok:".len()..];
        let keys: Vec<serde_json::Value> = serde_json::from_str(payload).expect("valid key list");
        assert!(!keys.is_empty());

        let alpha = keys
            .iter()
            .find(|entry| entry.get("name") == Some(&serde_json::Value::String("A".into())))
            .expect("contains A entry");
        let aliases = alpha
            .get("aliases")
            .and_then(|v| v.as_array())
            .expect("aliases array");

        assert!(aliases.iter().any(|a| a == "A"));
        assert!(alpha.get("evdev").and_then(|v| v.as_u64()).is_some());
        assert!(alpha.get("vk").and_then(|v| v.as_u64()).is_some());
    }

    #[test]
    fn init_is_idempotent() {
        assert_eq!(keyrx_init(), 0);
        assert_eq!(keyrx_init(), 0);
    }

    #[test]
    fn version_matches_package_version() {
        let version = unsafe { CStr::from_ptr(keyrx_version()) }
            .to_str()
            .expect("version string should be valid UTF-8");

        assert_eq!(version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn load_script_rejects_null_pointer() {
        let result = unsafe { keyrx_load_script(ptr::null()) };
        assert_eq!(result, -1);
    }

    #[test]
    fn load_script_rejects_invalid_utf8() {
        static INVALID_UTF8: [u8; 2] = [0xFF, 0x00];
        let result = unsafe { keyrx_load_script(INVALID_UTF8.as_ptr() as *const c_char) };
        assert_eq!(result, -2);
    }

    #[test]
    fn load_script_accepts_valid_path() {
        let path = CString::new("script.rhai").expect("CString should not contain nulls");
        let result = unsafe { keyrx_load_script(path.as_ptr()) };
        assert_eq!(result, 0);
    }

    #[test]
    fn free_string_handles_null_pointer() {
        unsafe {
            keyrx_free_string(ptr::null_mut());
        }
    }

    #[test]
    fn eval_returns_error_when_runtime_missing() {
        clear_active_runtime();
        let cmd = CString::new(r#"remap("A","B");"#).unwrap();
        let ptr = unsafe { keyrx_eval(cmd.as_ptr()) };
        assert!(!ptr.is_null());
        let response = unsafe { CStr::from_ptr(ptr) }
            .to_str()
            .expect("response should be utf8");
        assert!(
            response.starts_with("error: engine not initialized"),
            "unexpected response: {response}"
        );
        unsafe { keyrx_free_string(ptr) };
    }

    #[test]
    fn eval_executes_against_shared_runtime() {
        clear_active_runtime();
        let mut runtime = RhaiRuntime::new().expect("runtime should initialize");
        set_active_runtime(&mut runtime);

        let cmd = CString::new(r#"remap("A","B");"#).unwrap();
        let ptr = unsafe { keyrx_eval(cmd.as_ptr()) };
        assert!(!ptr.is_null());
        let response = unsafe { CStr::from_ptr(ptr) }
            .to_str()
            .expect("response should be utf8");
        assert!(
            response.starts_with("ok:"),
            "unexpected response: {response}"
        );
        unsafe { keyrx_free_string(ptr) };

        assert!(matches!(
            runtime.lookup_remap(KeyCode::A),
            RemapAction::Remap(KeyCode::B)
        ));
        clear_active_runtime();
    }

    #[test]
    fn discovery_callbacks_emit_json_payloads() {
        progress_store().lock().unwrap().clear();
        summary_store().lock().unwrap().clear();

        keyrx_on_discovery_progress(Some(record_progress));
        keyrx_on_discovery_summary(Some(record_summary));

        let progress = SessionUpdate::Progress(DiscoveryProgress {
            captured: 1,
            total: 3,
            next: Some(ExpectedPosition { row: 0, col: 1 }),
        });
        publish_session_update(&progress);

        let mut keymap = HashMap::new();
        keymap.insert(10, PhysicalKey::new(10, 0, 0));

        let mut aliases = HashMap::new();
        aliases.insert("r0_c0".to_string(), 10);

        let summary = SessionUpdate::Finished(DiscoverySummary {
            device_id: DeviceId::new(0x1, 0x2),
            status: SessionStatus::Completed,
            message: None,
            rows: 1,
            cols_per_row: vec![1],
            captured: 1,
            total: 1,
            next: None,
            unmapped: vec![],
            duplicates: vec![],
            keymap,
            aliases,
        });
        publish_session_update(&summary);

        let progress_payloads = progress_store().lock().unwrap();
        assert_eq!(progress_payloads.len(), 1);
        let progress_json: DiscoveryProgress =
            serde_json::from_slice(&progress_payloads[0]).expect("valid progress json");
        assert_eq!(progress_json.captured, 1);
        assert_eq!(progress_json.total, 3);

        let summary_payloads = summary_store().lock().unwrap();
        assert_eq!(summary_payloads.len(), 1);
        let summary_json: DiscoverySummary =
            serde_json::from_slice(&summary_payloads[0]).expect("valid summary json");
        assert_eq!(summary_json.status, SessionStatus::Completed);
        assert_eq!(summary_json.device_id.vendor_id, 0x1);

        keyrx_on_discovery_progress(None);
        keyrx_on_discovery_summary(None);
    }
}
