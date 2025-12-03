//! Device-related FFI exports.
//!
//! Functions for querying device information and key definitions.
#![allow(unsafe_code)]

use crate::discovery::storage::profile_path;
use crate::discovery::types::DeviceId;
use crate::drivers;
use crate::drivers::keycodes::key_definitions;
use serde::Serialize;
use std::ffi::{c_char, CStr, CString};
use std::path::PathBuf;
use std::ptr;
use std::sync::RwLock;

/// Global storage for the selected device path.
static SELECTED_DEVICE: RwLock<Option<PathBuf>> = RwLock::new(None);

/// Device info with profile status for FFI.
#[derive(Serialize)]
struct DeviceInfoWithProfile {
    name: String,
    #[serde(rename = "vendorId")]
    vendor_id: u16,
    #[serde(rename = "productId")]
    product_id: u16,
    path: String,
    #[serde(rename = "hasProfile")]
    has_profile: bool,
}

/// Return list of keyboard devices as `ok:<json>` (or `error:<message>`).
///
/// Returns JSON array: `[{name, vendorId, productId, path, hasProfile}, ...]`
///
/// Caller must free with `keyrx_free_string`.
#[no_mangle]
pub extern "C" fn keyrx_list_devices() -> *mut c_char {
    let payload = match drivers::list_keyboards() {
        Ok(devices) => {
            let enriched: Vec<DeviceInfoWithProfile> = devices
                .into_iter()
                .map(|d| {
                    let device_id = DeviceId::new(d.vendor_id, d.product_id);
                    let has_profile = profile_path(device_id).exists();
                    DeviceInfoWithProfile {
                        name: d.name,
                        vendor_id: d.vendor_id,
                        product_id: d.product_id,
                        path: d.path.display().to_string(),
                        has_profile,
                    }
                })
                .collect();
            serde_json::to_string(&enriched)
                .map(|json| format!("ok:{json}"))
                .unwrap_or_else(|err| format!("error:{err}"))
        }
        Err(err) => format!("error:{err}"),
    };

    CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw)
}

/// Select a device by path for use when starting the engine.
///
/// # Returns
/// - 0: Success
/// - -1: Null pointer
/// - -2: Invalid UTF-8
/// - -3: Device path does not exist
///
/// # Safety
/// `path` must be a valid null-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn keyrx_select_device(path: *const c_char) -> i32 {
    if path.is_null() {
        return -1;
    }

    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };

    let device_path = PathBuf::from(path_str);
    if !device_path.exists() {
        return -3;
    }

    if let Ok(mut guard) = SELECTED_DEVICE.write() {
        *guard = Some(device_path);
        0
    } else {
        -4 // Lock poisoned
    }
}

/// Get the currently selected device path.
///
/// Used internally when starting the engine to determine which device to use.
pub fn get_selected_device() -> Option<PathBuf> {
    SELECTED_DEVICE.read().ok().and_then(|guard| guard.clone())
}

/// Clear the selected device (for testing).
#[cfg(test)]
pub fn clear_selected_device() {
    if let Ok(mut guard) = SELECTED_DEVICE.write() {
        *guard = None;
    }
}

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
    use serial_test::serial;
    use std::ffi::CStr;
    use tempfile::NamedTempFile;

    #[test]
    #[serial]
    fn select_device_with_valid_path() {
        clear_selected_device();

        // Create a temp file to use as a valid path
        let temp = NamedTempFile::new().unwrap();
        let path_cstr = CString::new(temp.path().to_str().unwrap()).expect("valid CString");

        let result = unsafe { keyrx_select_device(path_cstr.as_ptr()) };
        assert_eq!(result, 0);

        let selected = get_selected_device();
        assert_eq!(selected, Some(temp.path().to_path_buf()));

        clear_selected_device();
    }

    #[test]
    fn select_device_rejects_null() {
        let result = unsafe { keyrx_select_device(ptr::null()) };
        assert_eq!(result, -1);
    }

    #[test]
    fn select_device_rejects_invalid_utf8() {
        static INVALID_UTF8: [u8; 2] = [0xFF, 0x00];
        let result = unsafe { keyrx_select_device(INVALID_UTF8.as_ptr() as *const c_char) };
        assert_eq!(result, -2);
    }

    #[test]
    fn select_device_rejects_nonexistent_path() {
        let path = CString::new("/nonexistent/device/path").unwrap();
        let result = unsafe { keyrx_select_device(path.as_ptr()) };
        assert_eq!(result, -3);
    }

    #[test]
    fn list_devices_returns_json_array() {
        let ptr = keyrx_list_devices();
        assert!(!ptr.is_null());
        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };

        // Result should start with ok: or error:
        assert!(raw.starts_with("ok:") || raw.starts_with("error:"));

        if raw.starts_with("ok:") {
            let payload = &raw["ok:".len()..];
            let devices: Vec<serde_json::Value> =
                serde_json::from_str(payload).expect("valid device list");
            // Devices array may be empty if no keyboards found
            for device in devices {
                assert!(device.get("name").is_some());
                assert!(device.get("vendorId").is_some());
                assert!(device.get("productId").is_some());
                assert!(device.get("path").is_some());
                assert!(device.get("hasProfile").is_some());
            }
        }
    }

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
