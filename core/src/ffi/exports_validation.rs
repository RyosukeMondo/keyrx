//! Validation FFI exports.
//!
//! Provides C-ABI functions for script validation and key name suggestions.
#![allow(unsafe_code)]

use std::ffi::{c_char, CStr, CString};
use std::ptr;

use crate::drivers::keycodes::key_definitions;
use crate::validation::config::ValidationConfig;
use crate::validation::engine::ValidationEngine;
use crate::validation::suggestions::suggest_similar_keys;
use crate::validation::types::ValidationOptions;

/// Validate a script and return JSON result.
///
/// Returns JSON: `ok:{"is_valid":bool,"errors":[...],"warnings":[...]}`
/// or `error:<message>` on failure.
///
/// Caller must free with `keyrx_free_string`.
///
/// # Safety
/// `script` must be a valid null-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn keyrx_validate_script(script: *const c_char) -> *mut c_char {
    if script.is_null() {
        return error_response("null pointer");
    }

    let script_str = match CStr::from_ptr(script).to_str() {
        Ok(s) => s,
        Err(_) => return error_response("invalid utf8"),
    };

    let engine = ValidationEngine::new();
    let options = ValidationOptions::new().with_coverage();
    let result = engine.validate(script_str, options);

    match serde_json::to_string(&result) {
        Ok(json) => ok_response(&json),
        Err(e) => error_response(&format!("serialization error: {e}")),
    }
}

/// Validate a script with custom options and return JSON result.
///
/// Options JSON format: `{"strict":bool,"no_warnings":bool,"include_coverage":bool}`
///
/// Returns JSON: `ok:{"is_valid":bool,"errors":[...],"warnings":[...]}`
/// or `error:<message>` on failure.
///
/// Caller must free with `keyrx_free_string`.
///
/// # Safety
/// `script` and `options_json` must be valid null-terminated UTF-8 strings.
#[no_mangle]
pub unsafe extern "C" fn keyrx_validate_script_with_options(
    script: *const c_char,
    options_json: *const c_char,
) -> *mut c_char {
    if script.is_null() {
        return error_response("null script pointer");
    }

    let script_str = match CStr::from_ptr(script).to_str() {
        Ok(s) => s,
        Err(_) => return error_response("invalid utf8 in script"),
    };

    let options: ValidationOptions = if options_json.is_null() {
        ValidationOptions::new()
    } else {
        match CStr::from_ptr(options_json).to_str() {
            Ok(s) => serde_json::from_str(s).unwrap_or_default(),
            Err(_) => return error_response("invalid utf8 in options"),
        }
    };

    let engine = ValidationEngine::new();
    let result = engine.validate(script_str, options);

    match serde_json::to_string(&result) {
        Ok(json) => ok_response(&json),
        Err(e) => error_response(&format!("serialization error: {e}")),
    }
}

/// Suggest similar key names for autocomplete.
///
/// Returns JSON: `ok:["KeyName1","KeyName2",...]` or `error:<message>`.
///
/// Caller must free with `keyrx_free_string`.
///
/// # Safety
/// `partial` must be a valid null-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn keyrx_suggest_keys(partial: *const c_char) -> *mut c_char {
    if partial.is_null() {
        return error_response("null pointer");
    }

    let partial_str = match CStr::from_ptr(partial).to_str() {
        Ok(s) => s,
        Err(_) => return error_response("invalid utf8"),
    };

    let config = ValidationConfig::load();
    let suggestions = suggest_similar_keys(partial_str, &config);

    match serde_json::to_string(&suggestions) {
        Ok(json) => ok_response(&json),
        Err(e) => error_response(&format!("serialization error: {e}")),
    }
}

/// Get all valid key names for autocomplete.
///
/// Returns JSON: `ok:["A","B","CapsLock",...]` or `error:<message>`.
///
/// Caller must free with `keyrx_free_string`.
///
/// # Safety
/// This function is safe to call from any thread.
#[no_mangle]
pub extern "C" fn keyrx_all_key_names() -> *mut c_char {
    let definitions = key_definitions();
    let names: Vec<&str> = definitions.iter().map(|d| d.name).collect();

    match serde_json::to_string(&names) {
        Ok(json) => ok_response(&json),
        Err(e) => error_response(&format!("serialization error: {e}")),
    }
}

/// Helper to create an ok response.
fn ok_response(payload: &str) -> *mut c_char {
    let response = format!("ok:{payload}");
    CString::new(response).map_or_else(|_| ptr::null_mut(), CString::into_raw)
}

/// Helper to create an error response.
fn error_response(message: &str) -> *mut c_char {
    let response = format!("error:{message}");
    CString::new(response).map_or_else(|_| ptr::null_mut(), CString::into_raw)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::keyrx_free_string;
    use std::ffi::CStr;

    fn parse_response(ptr: *mut c_char) -> (bool, String) {
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
    fn validate_script_valid() {
        let script = CString::new(r#"remap("CapsLock", "Escape");"#).unwrap();
        let ptr = unsafe { keyrx_validate_script(script.as_ptr()) };
        let (ok, payload) = parse_response(ptr);
        assert!(ok);

        let result: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(result["is_valid"], true);
        assert!(result["errors"].as_array().unwrap().is_empty());
    }

    #[test]
    fn validate_script_with_errors() {
        let script = CString::new(r#"layer_push("undefined_layer");"#).unwrap();
        let ptr = unsafe { keyrx_validate_script(script.as_ptr()) };
        let (ok, payload) = parse_response(ptr);
        assert!(ok);

        let result: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(result["is_valid"], false);
        assert!(!result["errors"].as_array().unwrap().is_empty());
    }

    #[test]
    fn validate_script_parse_error() {
        let script = CString::new("this is not valid {{{").unwrap();
        let ptr = unsafe { keyrx_validate_script(script.as_ptr()) };
        let (ok, payload) = parse_response(ptr);
        assert!(ok);

        let result: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(result["is_valid"], false);
        assert!(result["errors"][0]["code"].as_str().unwrap() == "E000");
    }

    #[test]
    fn validate_script_null_pointer() {
        let ptr = unsafe { keyrx_validate_script(ptr::null()) };
        let (ok, _) = parse_response(ptr);
        assert!(!ok);
    }

    #[test]
    fn validate_script_with_options_strict() {
        // Script with conflicting remaps (produces warnings)
        let script = CString::new("remap(\"A\", \"B\");\nremap(\"A\", \"C\");").unwrap();
        let options = CString::new(r#"{"strict":true}"#).unwrap();

        let ptr = unsafe { keyrx_validate_script_with_options(script.as_ptr(), options.as_ptr()) };
        let (ok, payload) = parse_response(ptr);
        assert!(ok);

        let result: serde_json::Value = serde_json::from_str(&payload).unwrap();
        // In strict mode, warnings make script invalid
        assert_eq!(result["is_valid"], false);
        assert!(!result["warnings"].as_array().unwrap().is_empty());
    }

    #[test]
    fn validate_script_with_options_no_warnings() {
        // Script with conflicting remaps - warnings will be suppressed
        let script = CString::new("remap(\"A\", \"B\");\nremap(\"A\", \"C\");").unwrap();
        let options = CString::new(r#"{"no_warnings":true}"#).unwrap();

        let ptr = unsafe { keyrx_validate_script_with_options(script.as_ptr(), options.as_ptr()) };
        let (ok, payload) = parse_response(ptr);
        assert!(ok);

        let result: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert!(result["warnings"].as_array().unwrap().is_empty());
    }

    #[test]
    fn suggest_keys_typo() {
        let partial = CString::new("Escpe").unwrap();
        let ptr = unsafe { keyrx_suggest_keys(partial.as_ptr()) };
        let (ok, payload) = parse_response(ptr);
        assert!(ok);

        let suggestions: Vec<String> = serde_json::from_str(&payload).unwrap();
        assert!(suggestions.contains(&"Escape".to_string()));
    }

    #[test]
    fn suggest_keys_null_pointer() {
        let ptr = unsafe { keyrx_suggest_keys(ptr::null()) };
        let (ok, _) = parse_response(ptr);
        assert!(!ok);
    }

    #[test]
    fn all_key_names_returns_array() {
        let ptr = keyrx_all_key_names();
        let (ok, payload) = parse_response(ptr);
        assert!(ok);

        let names: Vec<String> = serde_json::from_str(&payload).unwrap();
        assert!(names.contains(&"A".to_string()));
        assert!(names.contains(&"Escape".to_string()));
        assert!(names.contains(&"CapsLock".to_string()));
        assert!(names.len() > 90); // Should have many keys
    }
}
