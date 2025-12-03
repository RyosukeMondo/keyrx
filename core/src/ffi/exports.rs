//! Core C-ABI exports for FFI.
//!
//! This module provides core init/common C-compatible functions for FFI integration.
//! Unsafe code is required for FFI interoperability.
#![allow(unsafe_code)]

use std::ffi::{c_char, CString};

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;
    use std::ptr;

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
    fn free_string_handles_null_pointer() {
        unsafe {
            keyrx_free_string(ptr::null_mut());
        }
    }
}
