//! Compatibility FFI exports expected by the Flutter UI.
//!
//! These wrappers expose the legacy `keyrx_*` symbols that the Dart bindings
//! call. They delegate to the newer domain implementations where available.
//! Returned values follow the `ok:<json>` / `error:<json>` convention used
//! throughout the project.
#![allow(unsafe_code)]

use crate::ffi::context::FfiContext;
use crate::ffi::domains::{
    analysis, device, diagnostics, discovery, engine, recording, script, testing, DeviceFfi,
    RecordingFfi,
};
use crate::ffi::error::{serialize_ffi_result, FfiError, FfiResult};
use crate::ffi::traits::FfiExportable;
use crate::validation::{
    suggestions::suggest_similar_keys, types::ValidationOptions, ValidationEngine,
};
use std::ffi::{c_char, CStr, CString};
use std::sync::{Mutex, OnceLock};

// ── Helpers ───────────────────────────────────────────────────────────────────

static GLOBAL_CTX: OnceLock<Mutex<FfiContext>> = OnceLock::new();

fn with_ctx<F, T>(f: F) -> FfiResult<T>
where
    F: FnOnce(&mut FfiContext) -> FfiResult<T>,
{
    let lock = GLOBAL_CTX.get_or_init(|| Mutex::new(FfiContext::new()));
    let mut guard = lock
        .lock()
        .map_err(|_| FfiError::internal("context lock poisoned"))?;
    f(&mut guard)
}

fn ensure_domain<D: FfiExportable>(ctx: &mut FfiContext) -> Result<(), FfiError> {
    if !ctx.has_domain(D::DOMAIN) {
        D::init(ctx)?;
    }
    Ok(())
}

fn ffi_json<T: serde::Serialize>(result: FfiResult<T>) -> *mut c_char {
    let payload = serialize_ffi_result(&result).unwrap_or_else(|e| {
        format!("error:{{\"code\":\"SERIALIZATION_FAILED\",\"message\":\"{e}\"}}")
    });
    CString::new(payload)
        .map(CString::into_raw)
        .unwrap_or(std::ptr::null_mut())
}

fn ffi_error(err: FfiError) -> *mut c_char {
    ffi_json::<()>(Err(err))
}

/// # Safety
///
/// The `ptr` must be a valid, nul-terminated C string if non-null.
unsafe fn cstr_to_str(ptr: *const c_char) -> Result<&'static str, i32> {
    if ptr.is_null() {
        return Err(-1);
    }
    CStr::from_ptr(ptr).to_str().map_err(|_| -2)
}

// ── Device ────────────────────────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn keyrx_list_devices() -> *mut c_char {
    ffi_json(device::list_devices())
}

/// # Safety
///
/// The `path` pointer must be a valid, non-null, nul-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn keyrx_select_device(path: *const c_char) -> i32 {
    let path = match cstr_to_str(path) {
        Ok(s) => s,
        Err(code) => return code,
    };

    let res = with_ctx(|ctx| {
        ensure_domain::<DeviceFfi>(ctx)?;
        device::select_device(ctx, path)
    });

    match res {
        Ok(_) => 0,
        Err(err) => {
            if err.code == "NOT_FOUND" {
                -3
            } else {
                -4
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn keyrx_list_keys() -> *mut c_char {
    ffi_json(device::list_keys())
}

// ── Discovery ─────────────────────────────────────────────────────────────────

/// # Safety
///
/// Both `device_id` and `cols_per_row_json` must be valid, non-null, nul-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn keyrx_start_discovery(
    device_id: *const c_char,
    rows: u8,
    cols_per_row_json: *const c_char,
) -> *mut c_char {
    let device_id = match cstr_to_str(device_id) {
        Ok(s) => s,
        Err(code) => {
            return ffi_error(FfiError::new(
                format!("INVALID_DEVICE_ID_{code}"),
                "invalid device id",
            ))
        }
    };
    let cols_json = match cstr_to_str(cols_per_row_json) {
        Ok(s) => s,
        Err(code) => {
            return ffi_error(FfiError::new(
                format!("INVALID_COLS_{code}"),
                "invalid cols_per_row json",
            ))
        }
    };
    ffi_json(discovery::start_discovery(device_id, rows, cols_json))
}

#[no_mangle]
pub extern "C" fn keyrx_cancel_discovery() -> i32 {
    discovery::cancel_discovery().unwrap_or(-4)
}

// ── Engine ───────────────────────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn keyrx_is_bypass_active() -> bool {
    engine::is_bypass_mode_active().unwrap_or(false)
}

#[no_mangle]
pub extern "C" fn keyrx_set_bypass(active: bool) {
    let _ = engine::set_bypass_mode_state(active);
}

// ── Script / Validation ──────────────────────────────────────────────────────

/// # Safety
///
/// The `path` pointer must be a valid, non-null, nul-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn keyrx_load_script(path: *const c_char) -> i32 {
    let path = match cstr_to_str(path) {
        Ok(s) => s,
        Err(code) => return code,
    };
    match script::load_script(path) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

/// # Safety
///
/// The `command` pointer must be a valid, non-null, nul-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn keyrx_eval(command: *const c_char) -> *mut c_char {
    let cmd = match cstr_to_str(command) {
        Ok(s) => s,
        Err(code) => {
            let err = FfiError::invalid_utf8(format!("command ({code})"));
            return ffi_error(err);
        }
    };
    ffi_json(script::eval(cmd))
}

fn validate_script_internal(
    script: &str,
    options: ValidationOptions,
) -> FfiResult<crate::validation::types::ValidationResult> {
    let engine = ValidationEngine::new();
    Ok(engine.validate(script, options))
}

/// # Safety
///
/// The `script` pointer must be a valid, non-null, nul-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn keyrx_validate_script(script: *const c_char) -> *mut c_char {
    let script = match cstr_to_str(script) {
        Ok(s) => s,
        Err(code) => {
            let err = FfiError::invalid_utf8(format!("script ({code})"));
            return ffi_error(err);
        }
    };
    ffi_json(validate_script_internal(script, ValidationOptions::new()))
}

/// # Safety
///
/// Both `script` and `options_json` must be valid, non-null, nul-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn keyrx_validate_script_with_options(
    script: *const c_char,
    options_json: *const c_char,
) -> *mut c_char {
    let script = match cstr_to_str(script) {
        Ok(s) => s,
        Err(code) => {
            let err = FfiError::invalid_utf8(format!("script ({code})"));
            return ffi_error(err);
        }
    };
    let options_json = match cstr_to_str(options_json) {
        Ok(s) => s,
        Err(code) => {
            let err = FfiError::invalid_utf8(format!("options ({code})"));
            return ffi_error(err);
        }
    };

    let options: ValidationOptions = match serde_json::from_str(options_json) {
        Ok(o) => o,
        Err(e) => {
            let err = FfiError::invalid_input(format!("invalid options json: {e}"));
            return ffi_error(err);
        }
    };

    ffi_json(validate_script_internal(script, options))
}

/// # Safety
///
/// The `partial` pointer must be a valid, non-null, nul-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn keyrx_suggest_keys(partial: *const c_char) -> *mut c_char {
    let partial = match cstr_to_str(partial) {
        Ok(s) => s,
        Err(code) => {
            let err = FfiError::invalid_utf8(format!("partial ({code})"));
            return ffi_error(err);
        }
    };
    let cfg = crate::validation::config::ValidationConfig::load();
    let suggestions = suggest_similar_keys(partial, &cfg);
    ffi_json(Ok(suggestions))
}

#[no_mangle]
pub extern "C" fn keyrx_all_key_names() -> *mut c_char {
    let names: Vec<String> = crate::drivers::keycodes::key_definitions()
        .iter()
        .map(|k| k.name.to_string())
        .collect();
    ffi_json(Ok(names))
}

/// # Safety
///
/// The `path` pointer must be a valid, non-null, nul-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn keyrx_check_script(path: *const c_char) -> *mut c_char {
    let path = match cstr_to_str(path) {
        Ok(s) => s,
        Err(code) => {
            let err = FfiError::invalid_utf8(format!("path ({code})"));
            return ffi_error(err);
        }
    };
    ffi_json(script::check_script(path))
}

// ── Testing / Simulation / Diagnostics ───────────────────────────────────────

/// # Safety
///
/// The `path` pointer must be a valid, non-null, nul-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn keyrx_discover_tests(path: *const c_char) -> *mut c_char {
    let path = match cstr_to_str(path) {
        Ok(s) => s,
        Err(code) => {
            let err = FfiError::invalid_utf8(format!("path ({code})"));
            return ffi_error(err);
        }
    };
    ffi_json(testing::discover_tests(path))
}

/// # Safety
///
/// The `path` pointer must be a valid, non-null, nul-terminated C string.
/// The `filter` pointer may be null, or a valid nul-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn keyrx_run_tests(
    path: *const c_char,
    filter: *const c_char,
) -> *mut c_char {
    let path = match cstr_to_str(path) {
        Ok(s) => s,
        Err(code) => {
            let err = FfiError::invalid_utf8(format!("path ({code})"));
            return ffi_error(err);
        }
    };
    let filter = if filter.is_null() {
        None
    } else {
        match CStr::from_ptr(filter).to_str() {
            Ok(s) if !s.is_empty() => Some(s),
            _ => None,
        }
    };
    ffi_json(testing::run_tests(path, filter))
}

/// # Safety
///
/// The `keys_json` pointer must be a valid, non-null, nul-terminated C string.
/// The `script_path` pointer may be null, or a valid nul-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn keyrx_simulate(
    keys_json: *const c_char,
    script_path: *const c_char,
    combo_mode: bool,
) -> *mut c_char {
    let keys_json = match cstr_to_str(keys_json) {
        Ok(s) => s,
        Err(code) => {
            let err = FfiError::invalid_utf8(format!("keys ({code})"));
            return ffi_error(err);
        }
    };
    let script_path = if script_path.is_null() {
        None
    } else {
        match CStr::from_ptr(script_path).to_str() {
            Ok(s) if !s.is_empty() => Some(s),
            _ => None,
        }
    };
    ffi_json(testing::simulate(keys_json, script_path, combo_mode))
}

/// # Safety
///
/// The `script_path` pointer may be null, or a valid nul-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn keyrx_run_benchmark(
    iterations: u32,
    script_path: *const c_char,
) -> *mut c_char {
    let script_path = if script_path.is_null() {
        None
    } else {
        match CStr::from_ptr(script_path).to_str() {
            Ok(s) if !s.is_empty() => Some(s),
            _ => None,
        }
    };
    ffi_json(diagnostics::run_benchmark(iterations, script_path))
}

#[no_mangle]
pub extern "C" fn keyrx_run_doctor() -> *mut c_char {
    ffi_json(diagnostics::run_doctor())
}

// ── Recording / Sessions ─────────────────────────────────────────────────────

/// # Safety
///
/// The `path` pointer must be a valid, non-null, nul-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn keyrx_start_recording(path: *const c_char) -> *mut c_char {
    let path = match cstr_to_str(path) {
        Ok(s) => s,
        Err(code) => {
            let err = FfiError::invalid_utf8(format!("path ({code})"));
            return ffi_error(err);
        }
    };
    ffi_json(with_ctx(|ctx| {
        ensure_domain::<RecordingFfi>(ctx)?;
        recording::start_recording(ctx, path)
    }))
}

#[no_mangle]
pub extern "C" fn keyrx_stop_recording() -> *mut c_char {
    ffi_json(with_ctx(|ctx| {
        ensure_domain::<RecordingFfi>(ctx)?;
        recording::stop_recording(ctx)
    }))
}

/// # Safety
///
/// The `dir_path` pointer must be a valid, non-null, nul-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn keyrx_list_sessions(dir_path: *const c_char) -> *mut c_char {
    let dir = match cstr_to_str(dir_path) {
        Ok(s) => s,
        Err(code) => {
            let err = FfiError::invalid_utf8(format!("dir ({code})"));
            return ffi_error(err);
        }
    };
    ffi_json(analysis::list_sessions(dir))
}

/// # Safety
///
/// The `path` pointer must be a valid, non-null, nul-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn keyrx_analyze_session(path: *const c_char) -> *mut c_char {
    let path = match cstr_to_str(path) {
        Ok(s) => s,
        Err(code) => {
            let err = FfiError::invalid_utf8(format!("path ({code})"));
            return ffi_error(err);
        }
    };
    ffi_json(analysis::analyze_session(path))
}

/// # Safety
///
/// The `path` pointer must be a valid, non-null, nul-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn keyrx_replay_session(path: *const c_char, verify: bool) -> *mut c_char {
    let path = match cstr_to_str(path) {
        Ok(s) => s,
        Err(code) => {
            let err = FfiError::invalid_utf8(format!("path ({code})"));
            return ffi_error(err);
        }
    };
    ffi_json(analysis::replay_session(path, verify))
}
