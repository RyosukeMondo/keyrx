//! Error code FFI exports.
//!
//! Functions for querying error codes and definitions through FFI.
//! Provides Flutter access to error codes, messages, hints, and severity levels.
#![allow(unsafe_code)]

use crate::errors::{ErrorCategory, ErrorCode, ErrorRegistry};
use serde::Serialize;
use std::ffi::{c_char, CStr, CString};
use std::ptr;

/// Error definition for FFI serialization.
#[derive(Serialize)]
struct ErrorDefFFI {
    code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    hint: Option<String>,
    severity: String,
    category: String,
}

/// Category summary for FFI.
#[derive(Serialize)]
struct CategoryInfoFFI {
    name: String,
    prefix: char,
    #[serde(rename = "errorCount")]
    error_count: usize,
}

/// Get all registered error categories with their error counts.
///
/// Returns JSON array: `[{name, prefix, errorCount}, ...]` as `ok:<json>` or `error:<message>`.
///
/// Caller must free with `keyrx_free_string`.
///
/// # Safety
/// Returns a newly allocated C string that must be freed.
#[no_mangle]
pub extern "C" fn keyrx_error_categories() -> *mut c_char {
    let categories = ErrorRegistry::categories();
    let info: Vec<CategoryInfoFFI> = categories
        .into_iter()
        .map(|cat| {
            let error_count = ErrorRegistry::get_by_category(cat).len();
            CategoryInfoFFI {
                name: format!("{:?}", cat),
                prefix: cat.prefix(),
                error_count,
            }
        })
        .collect();

    let payload = serde_json::to_string(&info)
        .map(|json| format!("ok:{json}"))
        .unwrap_or_else(|err| format!("error:{err}"));

    CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw)
}

/// Get all error definitions for a specific category.
///
/// Returns JSON array: `[{code, message, hint?, severity, category}, ...]` as `ok:<json>` or `error:<message>`.
///
/// # Arguments
/// * `category` - Category name (e.g., "Config", "Runtime", "Driver", "Validation", "Ffi", "Internal")
///
/// Caller must free with `keyrx_free_string`.
///
/// # Safety
/// `category` must be a valid null-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn keyrx_error_get_by_category(category: *const c_char) -> *mut c_char {
    if category.is_null() {
        return CString::new("error:null category pointer")
            .map_or_else(|_| ptr::null_mut(), CString::into_raw);
    }

    let category_str = match CStr::from_ptr(category).to_str() {
        Ok(s) => s,
        Err(_) => {
            return CString::new("error:invalid UTF-8 in category")
                .map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    // Parse category
    let cat = match category_str {
        "Config" => ErrorCategory::Config,
        "Runtime" => ErrorCategory::Runtime,
        "Driver" => ErrorCategory::Driver,
        "Validation" => ErrorCategory::Validation,
        "Ffi" => ErrorCategory::Ffi,
        "Internal" => ErrorCategory::Internal,
        _ => {
            return CString::new(format!("error:unknown category '{category_str}'"))
                .map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    let errors = ErrorRegistry::get_by_category(cat);
    let defs: Vec<ErrorDefFFI> = errors
        .into_iter()
        .map(|def| ErrorDefFFI {
            code: def.code().to_string(),
            message: def.message_template().to_string(),
            hint: def.hint().map(|h| h.to_string()),
            severity: format!("{:?}", def.severity()),
            category: format!("{:?}", def.code().category()),
        })
        .collect();

    let payload = serde_json::to_string(&defs)
        .map(|json| format!("ok:{json}"))
        .unwrap_or_else(|err| format!("error:{err}"));

    CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw)
}

/// Get a specific error definition by code.
///
/// Returns JSON: `{code, message, hint?, severity, category}` as `ok:<json>` or `error:<message>`.
///
/// # Arguments
/// * `code` - Error code string (e.g., "KRX-C1001", "KRX-R2005")
///
/// Caller must free with `keyrx_free_string`.
///
/// # Safety
/// `code` must be a valid null-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn keyrx_error_get_by_code(code: *const c_char) -> *mut c_char {
    if code.is_null() {
        return CString::new("error:null code pointer")
            .map_or_else(|_| ptr::null_mut(), CString::into_raw);
    }

    let code_str = match CStr::from_ptr(code).to_str() {
        Ok(s) => s,
        Err(_) => {
            return CString::new("error:invalid UTF-8 in code")
                .map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    // Parse error code (format: KRX-X####)
    if !code_str.starts_with("KRX-") || code_str.len() != 9 {
        return CString::new(format!("error:invalid code format '{code_str}'"))
            .map_or_else(|_| ptr::null_mut(), CString::into_raw);
    }

    let Some(prefix) = code_str.chars().nth(4) else {
        return CString::new("error:invalid code format")
            .map_or_else(|_| ptr::null_mut(), CString::into_raw);
    };
    let number_str = &code_str[5..9];

    let number: u16 = match number_str.parse() {
        Ok(n) => n,
        Err(_) => {
            return CString::new(format!("error:invalid code number '{number_str}'"))
                .map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    let category = match prefix {
        'C' => ErrorCategory::Config,
        'R' => ErrorCategory::Runtime,
        'D' => ErrorCategory::Driver,
        'V' => ErrorCategory::Validation,
        'F' => ErrorCategory::Ffi,
        'I' => ErrorCategory::Internal,
        _ => {
            return CString::new(format!("error:unknown category prefix '{prefix}'"))
                .map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    let error_code = ErrorCode::new(category, number);

    match ErrorRegistry::get(error_code) {
        Some(def) => {
            let error_def = ErrorDefFFI {
                code: def.code().to_string(),
                message: def.message_template().to_string(),
                hint: def.hint().map(|h| h.to_string()),
                severity: format!("{:?}", def.severity()),
                category: format!("{:?}", def.code().category()),
            };

            let payload = serde_json::to_string(&error_def)
                .map(|json| format!("ok:{json}"))
                .unwrap_or_else(|err| format!("error:{err}"));

            CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw)
        }
        None => CString::new(format!("error:code not found '{code_str}'"))
            .map_or_else(|_| ptr::null_mut(), CString::into_raw),
    }
}

/// Get all registered error definitions.
///
/// Returns JSON array: `[{code, message, hint?, severity, category}, ...]` as `ok:<json>` or `error:<message>`.
///
/// Caller must free with `keyrx_free_string`.
///
/// # Safety
/// Returns a newly allocated C string that must be freed.
#[no_mangle]
pub extern "C" fn keyrx_error_get_all() -> *mut c_char {
    let errors = ErrorRegistry::all();
    let defs: Vec<ErrorDefFFI> = errors
        .into_iter()
        .map(|def| ErrorDefFFI {
            code: def.code().to_string(),
            message: def.message_template().to_string(),
            hint: def.hint().map(|h| h.to_string()),
            severity: format!("{:?}", def.severity()),
            category: format!("{:?}", def.code().category()),
        })
        .collect();

    let payload = serde_json::to_string(&defs)
        .map(|json| format!("ok:{json}"))
        .unwrap_or_else(|err| format!("error:{err}"));

    CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw)
}

/// Get the total count of registered errors.
///
/// Returns the count as a non-negative integer, or -1 on error.
///
/// # Safety
/// This function is safe to call from any thread.
#[no_mangle]
pub extern "C" fn keyrx_error_count() -> i32 {
    ErrorRegistry::count() as i32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::keyrx_free_string;
    use std::ffi::CStr;

    #[test]
    fn error_count_returns_non_negative() {
        let count = keyrx_error_count();
        assert!(count >= 0, "Error count should be non-negative");
    }

    #[test]
    fn error_categories_returns_json() {
        let result = keyrx_error_categories();
        assert!(!result.is_null(), "Should return non-null pointer");

        unsafe {
            let c_str = CStr::from_ptr(result);
            let s = c_str.to_str().expect("Should be valid UTF-8");
            assert!(s.starts_with("ok:"), "Should return ok: prefix");

            keyrx_free_string(result);
        }
    }

    #[test]
    fn error_get_all_returns_json() {
        let result = keyrx_error_get_all();
        assert!(!result.is_null(), "Should return non-null pointer");

        unsafe {
            let c_str = CStr::from_ptr(result);
            let s = c_str.to_str().expect("Should be valid UTF-8");
            assert!(s.starts_with("ok:"), "Should return ok: prefix");

            keyrx_free_string(result);
        }
    }

    #[test]
    fn error_get_by_category_handles_null() {
        let result = unsafe { keyrx_error_get_by_category(ptr::null()) };
        assert!(!result.is_null(), "Should return error message");

        unsafe {
            let c_str = CStr::from_ptr(result);
            let s = c_str.to_str().expect("Should be valid UTF-8");
            assert!(s.starts_with("error:"), "Should return error: prefix");

            keyrx_free_string(result);
        }
    }

    #[test]
    fn error_get_by_category_handles_invalid_category() {
        let category = CString::new("InvalidCategory").unwrap();
        let result = unsafe { keyrx_error_get_by_category(category.as_ptr()) };
        assert!(!result.is_null(), "Should return error message");

        unsafe {
            let c_str = CStr::from_ptr(result);
            let s = c_str.to_str().expect("Should be valid UTF-8");
            assert!(s.starts_with("error:"), "Should return error: prefix");
            assert!(
                s.contains("unknown category"),
                "Should mention unknown category"
            );

            keyrx_free_string(result);
        }
    }

    #[test]
    fn error_get_by_code_handles_null() {
        let result = unsafe { keyrx_error_get_by_code(ptr::null()) };
        assert!(!result.is_null(), "Should return error message");

        unsafe {
            let c_str = CStr::from_ptr(result);
            let s = c_str.to_str().expect("Should be valid UTF-8");
            assert!(s.starts_with("error:"), "Should return error: prefix");

            keyrx_free_string(result);
        }
    }

    #[test]
    fn error_get_by_code_handles_invalid_format() {
        let code = CString::new("INVALID").unwrap();
        let result = unsafe { keyrx_error_get_by_code(code.as_ptr()) };
        assert!(!result.is_null(), "Should return error message");

        unsafe {
            let c_str = CStr::from_ptr(result);
            let s = c_str.to_str().expect("Should be valid UTF-8");
            assert!(s.starts_with("error:"), "Should return error: prefix");
            assert!(
                s.contains("invalid code format"),
                "Should mention invalid format"
            );

            keyrx_free_string(result);
        }
    }
}
