//! Compatibility FFI exports expected by the Flutter UI.
//!
//! These wrappers expose the legacy `keyrx_*` symbols that the Dart bindings
//! call. They delegate to the newer domain implementations where available.
//! Returned values follow the `ok:<json>` / `error:<json>` convention used
//! throughout the project.
#![allow(unsafe_code)]

use crate::ffi::context::FfiContext;
use crate::ffi::domains::{
    analysis, device, diagnostics, engine, recording, script, testing, DeviceFfi, RecordingFfi,
};
use crate::ffi::error::{serialize_ffi_result, FfiError, FfiResult};
use crate::ffi::traits::FfiExportable;
use crate::validation::{
    suggestions::suggest_similar_keys, types::ValidationOptions, ValidationEngine,
};
use ffi_export::ffi_export;
use std::ffi::{c_char, CStr, CString};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;
use tracing::instrument;

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

/// # Safety
///
/// The `path` pointer must be a valid, non-null, nul-terminated C string.
#[instrument(skip_all, fields(correlation_id = %crate::ffi::tracing::correlation_id(), path))]
#[no_mangle]
pub unsafe extern "C" fn keyrx_select_device(path: *const c_char) -> i32 {
    let start = Instant::now();
    crate::ffi::tracing::new_correlation_id();

    let result = std::panic::catch_unwind(|| {
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
    })
    .unwrap_or(-5); // -5 for panic

    crate::ffi_record_metrics!("keyrx_select_device", "compat", start, &Ok::<_, ()>(result));
    result
}

#[instrument(skip_all, fields(correlation_id = %crate::ffi::tracing::correlation_id()))]
#[ffi_export(domain = "compat", version = "1.0.0")]
#[no_mangle]
pub extern "C" fn keyrx_list_keys() -> *mut c_char {
    let start = Instant::now();
    crate::ffi::tracing::new_correlation_id();

    let result = device::list_keys();
    crate::ffi_record_metrics!("keyrx_list_keys", "compat", start, &result);

    std::panic::catch_unwind(|| ffi_json(result))
        .unwrap_or_else(|_| ffi_error(FfiError::internal("panic in keyrx_list_keys")))
}

/// Get device profile for a specific device.
///
/// Returns JSON containing the complete device profile including:
/// - Keyboard layout (rows, cols_per_row)
/// - Keymap (scan_code -> row/col/alias mapping)
/// - Discovery metadata
///
/// # Arguments
/// * `vendor_id` - USB vendor ID
/// * `product_id` - USB product ID
///
/// # Returns
/// * `ok:<json>` on success with DeviceProfile JSON
/// * `error:<message>` if profile not found
///
/// # Example JSON Response
/// ```json
/// {
///   "schema_version": 1,
///   "vendor_id": 1234,
///   "product_id": 5678,
///   "name": "My Keyboard",
///   "discovered_at": "2024-01-15T10:30:00Z",
///   "rows": 6,
///   "cols_per_row": [15, 15, 15, 13, 11, 8],
///   "keymap": {
///     "1": {"scan_code": 1, "row": 0, "col": 0, "alias": "Esc"},
///     "2": {"scan_code": 2, "row": 0, "col": 1, "alias": "1"}
///   },
///   "aliases": {
///     "Esc": 1,
///     "1": 2
///   },
///   "source": "Discovered"
/// }
/// ```
#[instrument(skip_all, fields(correlation_id = %crate::ffi::tracing::correlation_id(), vendor_id, product_id))]
#[no_mangle]
#[ffi_export(domain = "compat", version = "1.0.0")]
pub extern "C" fn keyrx_get_device_profile(vendor_id: u16, product_id: u16) -> *mut c_char {
    let start = Instant::now();
    crate::ffi::tracing::new_correlation_id();

    let result = device::get_device_profile(vendor_id, product_id);
    crate::ffi_record_metrics!("keyrx_get_device_profile", "compat", start, &result);

    std::panic::catch_unwind(|| ffi_json(result))
        .unwrap_or_else(|_| ffi_error(FfiError::internal("panic in keyrx_get_device_profile")))
}

/// Check if a device profile exists.
///
/// # Arguments
/// * `vendor_id` - USB vendor ID
/// * `product_id` - USB product ID
///
/// # Returns
/// * `ok:true` if profile exists
/// * `ok:false` if profile does not exist
#[instrument(skip_all, fields(correlation_id = %crate::ffi::tracing::correlation_id(), vendor_id, product_id))]
#[no_mangle]
#[ffi_export(domain = "compat", version = "1.0.0")]
pub extern "C" fn keyrx_has_device_profile(vendor_id: u16, product_id: u16) -> *mut c_char {
    let start = Instant::now();
    crate::ffi::tracing::new_correlation_id();

    let result = device::has_device_profile(vendor_id, product_id);
    crate::ffi_record_metrics!("keyrx_has_device_profile", "compat", start, &result);

    std::panic::catch_unwind(|| ffi_json(result))
        .unwrap_or_else(|_| ffi_error(FfiError::internal("panic in keyrx_has_device_profile")))
}

/// Save a device profile.
///
/// # Safety
/// `profile_json` must be a valid, non-null, nul-terminated C string.
#[instrument(skip_all, fields(correlation_id = %crate::ffi::tracing::correlation_id()))]
#[no_mangle]
pub unsafe extern "C" fn keyrx_save_device_profile(profile_json: *const c_char) -> *mut c_char {
    let start = Instant::now();
    crate::ffi::tracing::new_correlation_id();

    let result = std::panic::catch_unwind(|| {
        let json = match cstr_to_str(profile_json) {
            Ok(s) => s,
            Err(code) => {
                let err = FfiError::invalid_utf8(format!("profile_json ({code})"));
                crate::ffi_record_metrics!(
                    "keyrx_save_device_profile",
                    "compat",
                    start,
                    &Err::<(), _>(&err)
                );
                return ffi_error(err);
            }
        };
        let res = device::save_device_profile(json);
        crate::ffi_record_metrics!("keyrx_save_device_profile", "compat", start, &res);
        ffi_json(res)
    })
    .unwrap_or_else(|_| {
        crate::ffi_record_metrics!(
            "keyrx_save_device_profile",
            "compat",
            start,
            &Err::<(), _>(FfiError::internal("panic"))
        );
        ffi_error(FfiError::internal("panic in keyrx_save_device_profile"))
    });

    result
}

// ── Discovery ─────────────────────────────────────────────────────────────────

// NOTE: Legacy discovery functions (start_discovery, cancel_discovery, process_event)
// have been removed to enforce Single Source of Truth (SSOT) via DeviceRegistry.

// ── Engine ───────────────────────────────────────────────────────────────────

#[instrument(skip_all, fields(correlation_id = %crate::ffi::tracing::correlation_id()))]
#[no_mangle]
#[ffi_export(domain = "compat", version = "1.0.0")]
pub extern "C" fn keyrx_is_bypass_active() -> bool {
    let start = Instant::now();
    crate::ffi::tracing::new_correlation_id();

    let result = engine::is_bypass_mode_active().unwrap_or(false);
    crate::ffi_record_metrics!(
        "keyrx_is_bypass_active",
        "compat",
        start,
        &Ok::<_, ()>(result)
    );

    result
}

#[instrument(skip_all, fields(correlation_id = %crate::ffi::tracing::correlation_id(), active))]
#[no_mangle]
#[ffi_export(domain = "compat", version = "1.0.0")]
pub extern "C" fn keyrx_set_bypass(active: bool) {
    let start = Instant::now();
    crate::ffi::tracing::new_correlation_id();

    let result = engine::set_bypass_mode_state(active);
    crate::ffi_record_metrics!("keyrx_set_bypass", "compat", start, &result);
}

// ── Script / Validation ──────────────────────────────────────────────────────

/// # Safety
///
/// The `path` pointer must be a valid, non-null, nul-terminated C string.
#[instrument(skip_all, fields(correlation_id = %crate::ffi::tracing::correlation_id()))]
#[no_mangle]
pub unsafe extern "C" fn keyrx_load_script(path: *const c_char) -> i32 {
    let start = Instant::now();
    crate::ffi::tracing::new_correlation_id();

    let path = match cstr_to_str(path) {
        Ok(s) => s,
        Err(code) => {
            crate::ffi_record_metrics!("keyrx_load_script", "compat", start, &Err::<(), _>(code));
            return code;
        }
    };
    let result = match script::load_script(path) {
        Ok(_) => 0,
        Err(_) => -1,
    };
    crate::ffi_record_metrics!("keyrx_load_script", "compat", start, &Ok::<_, ()>(result));
    result
}

/// # Safety
///
/// The `command` pointer must be a valid, non-null, nul-terminated C string.
#[instrument(skip_all, fields(correlation_id = %crate::ffi::tracing::correlation_id()))]
#[no_mangle]
pub unsafe extern "C" fn keyrx_eval(command: *const c_char) -> *mut c_char {
    let start = Instant::now();
    crate::ffi::tracing::new_correlation_id();

    let cmd = match cstr_to_str(command) {
        Ok(s) => s,
        Err(code) => {
            let err = FfiError::invalid_utf8(format!("command ({code})"));
            crate::ffi_record_metrics!("keyrx_eval", "compat", start, &Err::<(), _>(&err));
            return ffi_error(err);
        }
    };
    let result = script::eval(cmd);
    crate::ffi_record_metrics!("keyrx_eval", "compat", start, &result);
    ffi_json(result)
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
#[instrument(skip_all, fields(correlation_id = %crate::ffi::tracing::correlation_id()))]
#[no_mangle]
pub unsafe extern "C" fn keyrx_validate_script(script: *const c_char) -> *mut c_char {
    let start = Instant::now();
    crate::ffi::tracing::new_correlation_id();

    let script = match cstr_to_str(script) {
        Ok(s) => s,
        Err(code) => {
            let err = FfiError::invalid_utf8(format!("script ({code})"));
            crate::ffi_record_metrics!(
                "keyrx_validate_script",
                "compat",
                start,
                &Err::<(), _>(&err)
            );
            return ffi_error(err);
        }
    };
    let result = validate_script_internal(script, ValidationOptions::new());
    crate::ffi_record_metrics!("keyrx_validate_script", "compat", start, &result);
    ffi_json(result)
}

/// # Safety
///
/// Both `script` and `options_json` must be valid, non-null, nul-terminated C strings.
#[instrument(skip_all, fields(correlation_id = %crate::ffi::tracing::correlation_id()))]
#[no_mangle]
pub unsafe extern "C" fn keyrx_validate_script_with_options(
    script: *const c_char,
    options_json: *const c_char,
) -> *mut c_char {
    let start = Instant::now();
    crate::ffi::tracing::new_correlation_id();

    let script = match cstr_to_str(script) {
        Ok(s) => s,
        Err(code) => {
            let err = FfiError::invalid_utf8(format!("script ({code})"));
            crate::ffi_record_metrics!(
                "keyrx_validate_script_with_options",
                "compat",
                start,
                &Err::<(), _>(&err)
            );
            return ffi_error(err);
        }
    };
    let options_json = match cstr_to_str(options_json) {
        Ok(s) => s,
        Err(code) => {
            let err = FfiError::invalid_utf8(format!("options ({code})"));
            crate::ffi_record_metrics!(
                "keyrx_validate_script_with_options",
                "compat",
                start,
                &Err::<(), _>(&err)
            );
            return ffi_error(err);
        }
    };

    let options: ValidationOptions = match serde_json::from_str(options_json) {
        Ok(o) => o,
        Err(e) => {
            let err = FfiError::invalid_input(format!("invalid options json: {e}"));
            crate::ffi_record_metrics!(
                "keyrx_validate_script_with_options",
                "compat",
                start,
                &Err::<(), _>(&err)
            );
            return ffi_error(err);
        }
    };

    let result = validate_script_internal(script, options);
    crate::ffi_record_metrics!(
        "keyrx_validate_script_with_options",
        "compat",
        start,
        &result
    );
    ffi_json(result)
}

/// # Safety
///
/// The `partial` pointer must be a valid, non-null, nul-terminated C string.
#[instrument(skip_all, fields(correlation_id = %crate::ffi::tracing::correlation_id()))]
#[no_mangle]
pub unsafe extern "C" fn keyrx_suggest_keys(partial: *const c_char) -> *mut c_char {
    let start = Instant::now();
    crate::ffi::tracing::new_correlation_id();

    let partial = match cstr_to_str(partial) {
        Ok(s) => s,
        Err(code) => {
            let err = FfiError::invalid_utf8(format!("partial ({code})"));
            crate::ffi_record_metrics!("keyrx_suggest_keys", "compat", start, &Err::<(), _>(&err));
            return ffi_error(err);
        }
    };
    let cfg = crate::validation::config::ValidationConfig::load();
    let suggestions = suggest_similar_keys(partial, &cfg);
    let result = Ok(suggestions);
    crate::ffi_record_metrics!("keyrx_suggest_keys", "compat", start, &result);
    ffi_json(result)
}

#[instrument(skip_all, fields(correlation_id = %crate::ffi::tracing::correlation_id()))]
#[no_mangle]
#[ffi_export(domain = "compat", version = "1.0.0")]
pub extern "C" fn keyrx_all_key_names() -> *mut c_char {
    let start = Instant::now();
    crate::ffi::tracing::new_correlation_id();

    let names: Vec<String> = crate::drivers::keycodes::key_definitions()
        .iter()
        .map(|k| k.name.to_string())
        .collect();
    let result = Ok(names);
    crate::ffi_record_metrics!("keyrx_all_key_names", "compat", start, &result);
    ffi_json(result)
}

/// # Safety
///
/// The `path` pointer must be a valid, non-null, nul-terminated C string.
#[instrument(skip_all, fields(correlation_id = %crate::ffi::tracing::correlation_id()))]
#[no_mangle]
pub unsafe extern "C" fn keyrx_check_script(path: *const c_char) -> *mut c_char {
    let start = Instant::now();
    crate::ffi::tracing::new_correlation_id();

    let path = match cstr_to_str(path) {
        Ok(s) => s,
        Err(code) => {
            let err = FfiError::invalid_utf8(format!("path ({code})"));
            crate::ffi_record_metrics!("keyrx_check_script", "compat", start, &Err::<(), _>(&err));
            return ffi_error(err);
        }
    };
    let result = script::check_script(path);
    crate::ffi_record_metrics!("keyrx_check_script", "compat", start, &result);
    ffi_json(result)
}

// ── Testing / Simulation / Diagnostics ───────────────────────────────────────

/// # Safety
///
/// The `path` pointer must be a valid, non-null, nul-terminated C string.
#[instrument(skip_all, fields(correlation_id = %crate::ffi::tracing::correlation_id()))]
#[no_mangle]
pub unsafe extern "C" fn keyrx_discover_tests(path: *const c_char) -> *mut c_char {
    let start = Instant::now();
    crate::ffi::tracing::new_correlation_id();

    let path = match cstr_to_str(path) {
        Ok(s) => s,
        Err(code) => {
            let err = FfiError::invalid_utf8(format!("path ({code})"));
            crate::ffi_record_metrics!(
                "keyrx_discover_tests",
                "compat",
                start,
                &Err::<(), _>(&err)
            );
            return ffi_error(err);
        }
    };
    let result = testing::discover_tests(path);
    crate::ffi_record_metrics!("keyrx_discover_tests", "compat", start, &result);
    ffi_json(result)
}

/// # Safety
///
/// The `path` pointer must be a valid, non-null, nul-terminated C string.
/// The `filter` pointer may be null, or a valid nul-terminated C string.
#[instrument(skip_all, fields(correlation_id = %crate::ffi::tracing::correlation_id()))]
#[no_mangle]
pub unsafe extern "C" fn keyrx_run_tests(
    path: *const c_char,
    filter: *const c_char,
) -> *mut c_char {
    let start = Instant::now();
    crate::ffi::tracing::new_correlation_id();

    let path = match cstr_to_str(path) {
        Ok(s) => s,
        Err(code) => {
            let err = FfiError::invalid_utf8(format!("path ({code})"));
            crate::ffi_record_metrics!("keyrx_run_tests", "compat", start, &Err::<(), _>(&err));
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
    let result = testing::run_tests(path, filter);
    crate::ffi_record_metrics!("keyrx_run_tests", "compat", start, &result);
    ffi_json(result)
}

/// # Safety
///
/// The `keys_json` pointer must be a valid, non-null, nul-terminated C string.
/// The `script_path` pointer may be null, or a valid nul-terminated C string.
#[instrument(skip_all, fields(correlation_id = %crate::ffi::tracing::correlation_id(), combo_mode))]
#[no_mangle]
pub unsafe extern "C" fn keyrx_simulate(
    keys_json: *const c_char,
    script_path: *const c_char,
    combo_mode: bool,
) -> *mut c_char {
    let start = Instant::now();
    crate::ffi::tracing::new_correlation_id();

    let keys_json = match cstr_to_str(keys_json) {
        Ok(s) => s,
        Err(code) => {
            let err = FfiError::invalid_utf8(format!("keys ({code})"));
            crate::ffi_record_metrics!("keyrx_simulate", "compat", start, &Err::<(), _>(&err));
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
    let result = testing::simulate(keys_json, script_path, combo_mode);
    crate::ffi_record_metrics!("keyrx_simulate", "compat", start, &result);
    ffi_json(result)
}

/// # Safety
///
/// The `script_path` pointer may be null, or a valid nul-terminated C string.
#[instrument(skip_all, fields(correlation_id = %crate::ffi::tracing::correlation_id(), iterations))]
#[no_mangle]
pub unsafe extern "C" fn keyrx_run_benchmark(
    iterations: u32,
    script_path: *const c_char,
) -> *mut c_char {
    let start = Instant::now();
    crate::ffi::tracing::new_correlation_id();

    let script_path = if script_path.is_null() {
        None
    } else {
        match CStr::from_ptr(script_path).to_str() {
            Ok(s) if !s.is_empty() => Some(s),
            _ => None,
        }
    };
    let result = diagnostics::run_benchmark(iterations, script_path);
    crate::ffi_record_metrics!("keyrx_run_benchmark", "compat", start, &result);
    ffi_json(result)
}

#[instrument(skip_all, fields(correlation_id = %crate::ffi::tracing::correlation_id()))]
#[no_mangle]
#[ffi_export(domain = "compat", version = "1.0.0")]
pub extern "C" fn keyrx_run_doctor() -> *mut c_char {
    let start = Instant::now();
    crate::ffi::tracing::new_correlation_id();

    let result = diagnostics::run_doctor();
    crate::ffi_record_metrics!("keyrx_run_doctor", "compat", start, &result);
    ffi_json(result)
}

// ── Recording / Sessions ─────────────────────────────────────────────────────

/// # Safety
///
/// The `path` pointer must be a valid, non-null, nul-terminated C string.
#[instrument(skip_all, fields(correlation_id = %crate::ffi::tracing::correlation_id()))]
#[no_mangle]
pub unsafe extern "C" fn keyrx_start_recording(path: *const c_char) -> *mut c_char {
    let start = Instant::now();
    crate::ffi::tracing::new_correlation_id();

    let path = match cstr_to_str(path) {
        Ok(s) => s,
        Err(code) => {
            let err = FfiError::invalid_utf8(format!("path ({code})"));
            crate::ffi_record_metrics!(
                "keyrx_start_recording",
                "compat",
                start,
                &Err::<(), _>(&err)
            );
            return ffi_error(err);
        }
    };
    let result = with_ctx(|ctx| {
        ensure_domain::<RecordingFfi>(ctx)?;
        recording::start_recording(ctx, path)
    });
    crate::ffi_record_metrics!("keyrx_start_recording", "compat", start, &result);
    ffi_json(result)
}

#[instrument(skip_all, fields(correlation_id = %crate::ffi::tracing::correlation_id()))]
#[no_mangle]
#[ffi_export(domain = "compat", version = "1.0.0")]
pub extern "C" fn keyrx_stop_recording() -> *mut c_char {
    let start = Instant::now();
    crate::ffi::tracing::new_correlation_id();

    let result = with_ctx(|ctx| {
        ensure_domain::<RecordingFfi>(ctx)?;
        recording::stop_recording(ctx)
    });
    crate::ffi_record_metrics!("keyrx_stop_recording", "compat", start, &result);
    ffi_json(result)
}

/// # Safety
///
/// The `dir_path` pointer must be a valid, non-null, nul-terminated C string.
#[instrument(skip_all, fields(correlation_id = %crate::ffi::tracing::correlation_id()))]
#[no_mangle]
pub unsafe extern "C" fn keyrx_list_sessions(dir_path: *const c_char) -> *mut c_char {
    let start = Instant::now();
    crate::ffi::tracing::new_correlation_id();

    let dir = match cstr_to_str(dir_path) {
        Ok(s) => s,
        Err(code) => {
            let err = FfiError::invalid_utf8(format!("dir ({code})"));
            crate::ffi_record_metrics!("keyrx_list_sessions", "compat", start, &Err::<(), _>(&err));
            return ffi_error(err);
        }
    };
    let result = analysis::list_sessions(dir);
    crate::ffi_record_metrics!("keyrx_list_sessions", "compat", start, &result);
    ffi_json(result)
}

/// # Safety
///
/// The `path` pointer must be a valid, non-null, nul-terminated C string.
#[instrument(skip_all, fields(correlation_id = %crate::ffi::tracing::correlation_id()))]
#[no_mangle]
pub unsafe extern "C" fn keyrx_analyze_session(path: *const c_char) -> *mut c_char {
    let start = Instant::now();
    crate::ffi::tracing::new_correlation_id();

    let path = match cstr_to_str(path) {
        Ok(s) => s,
        Err(code) => {
            let err = FfiError::invalid_utf8(format!("path ({code})"));
            crate::ffi_record_metrics!(
                "keyrx_analyze_session",
                "compat",
                start,
                &Err::<(), _>(&err)
            );
            return ffi_error(err);
        }
    };
    let result = analysis::analyze_session(path);
    crate::ffi_record_metrics!("keyrx_analyze_session", "compat", start, &result);
    ffi_json(result)
}

/// # Safety
///
/// The `path` pointer must be a valid, non-null, nul-terminated C string.
#[instrument(skip_all, fields(correlation_id = %crate::ffi::tracing::correlation_id(), verify))]
#[no_mangle]
pub unsafe extern "C" fn keyrx_replay_session(path: *const c_char, verify: bool) -> *mut c_char {
    let start = Instant::now();
    crate::ffi::tracing::new_correlation_id();

    let path = match cstr_to_str(path) {
        Ok(s) => s,
        Err(code) => {
            let err = FfiError::invalid_utf8(format!("path ({code})"));
            crate::ffi_record_metrics!(
                "keyrx_replay_session",
                "compat",
                start,
                &Err::<(), _>(&err)
            );
            return ffi_error(err);
        }
    };
    let result = analysis::replay_session(path, verify);
    crate::ffi_record_metrics!("keyrx_replay_session", "compat", start, &result);
    ffi_json(result)
}
