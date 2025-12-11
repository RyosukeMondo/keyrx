//! C string parsing utilities for FFI.
//!
//! Provides safe conversion from C strings (`*const c_char`) to Rust strings,
//! with proper null pointer checking and UTF-8 validation.

use std::ffi::{c_char, CStr};

/// Parse a C string pointer into a Rust `String`.
///
/// # Safety
///
/// The caller must ensure that `ptr` points to a valid, null-terminated C string
/// if it is not null.
///
/// # Arguments
///
/// * `ptr` - Pointer to a null-terminated C string, or null
/// * `name` - Name of the parameter for error messages
///
/// # Returns
///
/// * `Ok(String)` - Successfully parsed string
/// * `Err(String)` - Error message if null pointer or invalid UTF-8
///
/// # Example
///
/// ```ignore
/// let result = unsafe { parse_c_string(ptr, "profile_id") };
/// ```
pub unsafe fn parse_c_string(ptr: *const c_char, name: &str) -> Result<String, String> {
    if ptr.is_null() {
        return Err(format!("Null pointer for parameter '{}'", name));
    }
    // SAFETY: Caller guarantees ptr is valid if not null
    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .map(|s| s.to_string())
        .map_err(|_| format!("Invalid UTF-8 in parameter '{}'", name))
}
