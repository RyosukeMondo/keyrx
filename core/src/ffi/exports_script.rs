//! Script loading and validation FFI exports.
//!
//! Functions for loading scripts, validating syntax, and evaluating commands.
#![allow(unsafe_code)]

use crate::scripting::with_active_runtime;
use crate::traits::ScriptRuntime;
use serde::Serialize;
use std::ffi::{c_char, CStr, CString};
use std::path::Path;
use std::ptr;

/// Script validation error detail.
#[derive(Serialize)]
pub struct ValidationError {
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub message: String,
}

/// Script validation result.
#[derive(Serialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
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

    // Load and run the script using the active runtime
    match with_active_runtime(|runtime| {
        runtime.load_file(path_str)?;
        runtime.run_script()
    }) {
        Some(Ok(())) => 0,
        Some(Err(err)) => {
            tracing::error!(
                service = "keyrx",
                event = "ffi_load_script_error",
                component = "ffi_exports",
                script = script_name,
                error = %err,
                "Script syntax/execution error"
            );
            -3
        }
        None => {
            tracing::error!(
                service = "keyrx",
                event = "ffi_load_script_error",
                component = "ffi_exports",
                script = script_name,
                "Engine not initialized"
            );
            -4
        }
    }
}

/// Validate a Rhai script without executing it.
///
/// Returns JSON: `ok:{valid: bool, errors: [{line, column, message}]}`
///
/// Caller must free with `keyrx_free_string`.
///
/// # Safety
/// `path` must be a valid null-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn keyrx_check_script(path: *const c_char) -> *mut c_char {
    if path.is_null() {
        return CString::new("error:null pointer")
            .map_or_else(|_| ptr::null_mut(), CString::into_raw);
    }

    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => {
            return CString::new("error:invalid utf8")
                .map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    let script = match std::fs::read_to_string(path_str) {
        Ok(s) => s,
        Err(err) => {
            let result = ValidationResult {
                valid: false,
                errors: vec![ValidationError {
                    line: None,
                    column: None,
                    message: format!("Failed to read file: {err}"),
                }],
            };
            let payload = serde_json::to_string(&result)
                .map(|json| format!("ok:{json}"))
                .unwrap_or_else(|e| format!("error:{e}"));
            return CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    let engine = rhai::Engine::new();
    let result = match engine.compile(&script) {
        Ok(_) => ValidationResult {
            valid: true,
            errors: vec![],
        },
        Err(e) => {
            let position = e.position();
            let (line, column) = if position.is_none() {
                (None, None)
            } else {
                (position.line(), position.position())
            };
            ValidationResult {
                valid: false,
                errors: vec![ValidationError {
                    line,
                    column,
                    message: e.to_string(),
                }],
            }
        }
    };

    let payload = serde_json::to_string(&result)
        .map(|json| format!("ok:{json}"))
        .unwrap_or_else(|err| format!("error:{err}"));

    CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw)
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
