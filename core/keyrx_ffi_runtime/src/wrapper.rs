//! Complete FFI wrapper functionality.
//!
//! Provides a unified wrapper that combines panic catching, error handling,
//! and JSON serialization for FFI functions.

use crate::json::serialize_to_c_string;
use crate::panic::handle_panic;
use serde::Serialize;
use std::ffi::{c_char, CString};

/// Complete FFI wrapper that handles panics, errors, and serialization.
///
/// This function wraps an implementation closure with all necessary FFI safety:
/// - Catches panics to prevent UB at FFI boundaries
/// - Handles `Result` return types, setting error pointer on `Err`
/// - Serializes successful results to JSON C strings
///
/// # Safety
///
/// The `error` pointer, if not null, must point to valid, writable memory.
///
/// # Arguments
///
/// * `error` - Pointer to error output pointer (set on error, null on success)
/// * `f` - The implementation closure returning `Result<T, String>`
///
/// # Returns
///
/// * On success: Pointer to JSON C string (caller must free)
/// * On error: Null pointer (error message written to `error` if not null)
///
/// # Example
///
/// ```ignore
/// let result = unsafe {
///     ffi_wrapper(error_ptr, || {
///         Ok(my_struct)
///     })
/// };
/// ```
pub unsafe fn ffi_wrapper<F, T>(error: *mut *mut c_char, f: F) -> *const c_char
where
    F: FnOnce() -> Result<T, String>,
    T: Serialize,
{
    // Helper to set error pointer
    let set_error = |err_msg: &str| {
        if !error.is_null() {
            if let Ok(cs) = CString::new(err_msg) {
                // SAFETY: error is not null, checked above
                unsafe { *error = cs.into_raw() };
            }
        }
    };

    // Catch panics
    let result = handle_panic(f);

    match result {
        Ok(Ok(value)) => {
            // Success path: serialize to JSON
            match serialize_to_c_string(&value) {
                Ok(ptr) => ptr,
                Err(e) => {
                    set_error(&e);
                    std::ptr::null()
                }
            }
        }
        Ok(Err(e)) => {
            // Implementation returned an error
            set_error(&e);
            std::ptr::null()
        }
        Err(panic_msg) => {
            // Panic was caught
            set_error(&panic_msg);
            std::ptr::null()
        }
    }
    }


/// Specialized wrapper for returning raw strings (without JSON serialization).
///
/// Use this when the function contract specifies a String return and the implementation
/// returns a `String` that should be passed directly to C (e.g. pre-formatted JSON).
pub unsafe fn ffi_string_wrapper<F>(error: *mut *mut c_char, f: F) -> *const c_char
where
    F: FnOnce() -> Result<String, String>,
{
     // Helper to set error pointer
    let set_error = |err_msg: &str| {
        if !error.is_null() {
            if let Ok(cs) = CString::new(err_msg) {
                // SAFETY: error is not null, checked above
                unsafe { *error = cs.into_raw() };
            }
        }
    };

    // Catch panics
    let result = handle_panic(f);

    match result {
        Ok(Ok(value)) => {
            // Success path: convert String directly to CString
            match CString::new(value) {
                Ok(cs) => cs.into_raw(),
                Err(_) => {
                    set_error("String contains null byte");
                    std::ptr::null()
                }
            }
        }
        Ok(Err(e)) => {
            // Implementation returned an error
            set_error(&e);
            std::ptr::null()
        }
        Err(panic_msg) => {
            // Panic was caught
            set_error(&panic_msg);
            std::ptr::null()
        }
    }
}
