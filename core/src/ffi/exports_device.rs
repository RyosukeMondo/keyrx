//! Device-related FFI exports.
//!
//! Functions for querying device information and key definitions.
#![allow(unsafe_code)]

use crate::drivers::keycodes::key_definitions;
use std::ffi::{c_char, CString};
use std::ptr;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::keyrx_free_string;
    use std::ffi::CStr;

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
}
