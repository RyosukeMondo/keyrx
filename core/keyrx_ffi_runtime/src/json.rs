//! JSON serialization utilities for FFI.
//!
//! Provides conversion from Rust types to C strings via JSON serialization.

use serde::Serialize;
use std::ffi::{c_char, CString};

/// Serialize a Rust value to a C string via JSON.
///
/// # Arguments
///
/// * `value` - The value to serialize (must implement `Serialize`)
///
/// # Returns
///
/// * `Ok(*const c_char)` - Pointer to a newly allocated C string containing the JSON
/// * `Err(String)` - Error message if serialization fails
///
/// # Memory
///
/// The returned pointer must be freed by the caller using `keyrx_free_string` or
/// equivalent to avoid memory leaks.
///
/// # Example
///
/// ```ignore
/// let ptr = serialize_to_c_string(&my_struct)?;
/// ```
pub fn serialize_to_c_string<T: Serialize>(value: &T) -> Result<*const c_char, String> {
    let json = serde_json::to_string(value).map_err(|e| format!("Serialization failed: {}", e))?;

    CString::new(json)
        .map(|cs| cs.into_raw() as *const c_char)
        .map_err(|_| "JSON contains null byte".to_string())
}
