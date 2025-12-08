//! Device Definitions domain FFI implementation.
//!
//! Implements FFI exports for device definitions library,
//! exposing read-only access to device definitions for Flutter.
#![allow(unsafe_code)]

use crate::definitions::{DeviceDefinition, DeviceDefinitionLibrary};
use crate::ffi::error::{serialize_ffi_result, FfiError, FfiResult};
use crate::ffi::runtime::with_revolutionary_runtime;
use serde::Serialize;
use std::ffi::{c_char, CString};
use std::ptr;

/// List all loaded device definitions.
///
/// Returns JSON array of device definitions.
///
/// # Returns
/// * `Ok(Vec<DeviceDefinition>)` - List of all definitions
pub fn list_all(library: &DeviceDefinitionLibrary) -> FfiResult<Vec<DeviceDefinition>> {
    let definitions: Vec<DeviceDefinition> = library.list_definitions().cloned().collect();
    Ok(definitions)
}

/// Get device definition for a specific VID:PID.
///
/// # Arguments
/// * `library` - Device definition library
/// * `vendor_id` - USB Vendor ID
/// * `product_id` - USB Product ID
///
/// # Returns
/// * `Ok(DeviceDefinition)` on success
/// * `Err(FfiError)` if definition not found
pub fn get_for_device(
    library: &DeviceDefinitionLibrary,
    vendor_id: u16,
    product_id: u16,
) -> FfiResult<DeviceDefinition> {
    let definition = library
        .find_definition(vendor_id, product_id)
        .ok_or_else(|| {
            FfiError::not_found(format!(
                "No definition found for device {:04x}:{:04x}",
                vendor_id, product_id
            ))
        })?;

    Ok(definition.clone())
}

// C-ABI exports with panic guards

fn ffi_response<T: Serialize>(result: FfiResult<T>) -> *mut c_char {
    let payload = serialize_ffi_result(&result).unwrap_or_else(|e| {
        format!("error:{{\"code\":\"SERIALIZATION_FAILED\",\"message\":\"{e}\"}}")
    });
    CString::new(payload)
        .map(CString::into_raw)
        .unwrap_or(ptr::null_mut())
}

/// List all loaded device definitions.
///
/// Returns JSON string: `ok:<json>` or `error:<message>`.
///
/// # Safety
/// This function catches panics and returns an error string if a panic occurs.
#[no_mangle]
pub extern "C" fn keyrx_definitions_list_all() -> *mut c_char {
    std::panic::catch_unwind(|| {
        let result: FfiResult<Vec<DeviceDefinition>> =
            with_revolutionary_runtime(|runtime| list_all(runtime.device_definitions()));
        ffi_response(result)
    })
    .unwrap_or_else(|_| {
        ffi_response::<()>(Err(FfiError::internal(
            "panic in keyrx_definitions_list_all",
        )))
    })
}

/// Get device definition for a specific VID:PID.
///
/// Returns JSON string: `ok:<definition_json>` or `error:<message>`.
///
/// # Arguments
/// * `vendor_id` - USB Vendor ID (hex format, e.g., 0x1234)
/// * `product_id` - USB Product ID (hex format, e.g., 0x5678)
///
/// # Safety
/// This function catches panics and returns an error string if a panic occurs.
#[no_mangle]
pub extern "C" fn keyrx_definitions_get_for_device(vendor_id: u16, product_id: u16) -> *mut c_char {
    std::panic::catch_unwind(move || {
        let result: FfiResult<DeviceDefinition> = with_revolutionary_runtime(|runtime| {
            get_for_device(runtime.device_definitions(), vendor_id, product_id)
        });
        ffi_response(result)
    })
    .unwrap_or_else(|_| {
        ffi_response::<()>(Err(FfiError::internal(
            "panic in keyrx_definitions_get_for_device",
        )))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definitions::DeviceDefinitionLibrary;
    use crate::ffi::runtime::{
        clear_revolutionary_runtime, set_revolutionary_runtime, RevolutionaryRuntime,
    };
    use crate::registry::profile::ProfileRegistry;
    use crate::registry::DeviceRegistry;
    use std::ffi::{CStr, CString};
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;

    fn create_test_definition_toml() -> String {
        r#"
name = "Test Device"
vendor_id = 0x1234
product_id = 0x5678
manufacturer = "Test Manufacturer"

[layout]
layout_type = "matrix"
rows = 2
cols = 3

[matrix_map]
"1" = { row = 0, col = 0 }
"2" = { row = 0, col = 1 }
"3" = { row = 0, col = 2 }
"4" = { row = 1, col = 0 }
"5" = { row = 1, col = 1 }
"6" = { row = 1, col = 2 }

[visual]
key_width = 80
key_height = 80
key_spacing = 4
"#
        .to_string()
    }

    fn setup_runtime_with_definition() -> TempDir {
        let (device_registry, _rx) = DeviceRegistry::new();
        let temp_dir = TempDir::new().unwrap();
        let profile_registry = Arc::new(ProfileRegistry::with_directory(
            temp_dir.path().to_path_buf(),
        ));

        let mut library = DeviceDefinitionLibrary::new();
        let file_path = temp_dir.path().join("test.toml");
        std::fs::write(&file_path, create_test_definition_toml()).unwrap();
        library.load_from_directory(temp_dir.path()).unwrap();
        let library = Arc::new(library);

        set_revolutionary_runtime(RevolutionaryRuntime::new(
            device_registry,
            profile_registry,
            library,
            Arc::new(Mutex::new(crate::scripting::RhaiRuntime::new().unwrap())),
        ))
        .unwrap();

        temp_dir
    }

    unsafe fn c_string_result(ptr: *mut c_char) -> String {
        assert!(!ptr.is_null());
        let msg = CStr::from_ptr(ptr).to_str().unwrap().to_string();
        drop(CString::from_raw(ptr));
        msg
    }

    #[test]
    fn test_list_all_empty() {
        let library = DeviceDefinitionLibrary::new();
        let result = list_all(&library);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_list_all_with_definitions() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.toml");
        std::fs::write(&file_path, create_test_definition_toml()).unwrap();

        let mut library = DeviceDefinitionLibrary::new();
        library.load_from_directory(temp_dir.path()).unwrap();

        let result = list_all(&library);
        assert!(result.is_ok());
        let definitions = result.unwrap();
        assert_eq!(definitions.len(), 1);
        assert_eq!(definitions[0].name, "Test Device");
    }

    #[test]
    fn test_get_for_device_valid() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.toml");
        std::fs::write(&file_path, create_test_definition_toml()).unwrap();

        let mut library = DeviceDefinitionLibrary::new();
        library.load_from_directory(temp_dir.path()).unwrap();

        let result = get_for_device(&library, 0x1234, 0x5678);
        assert!(result.is_ok());
        let definition = result.unwrap();
        assert_eq!(definition.name, "Test Device");
        assert_eq!(definition.vendor_id, 0x1234);
        assert_eq!(definition.product_id, 0x5678);
    }

    #[test]
    fn test_get_for_device_not_found() {
        let library = DeviceDefinitionLibrary::new();
        let result = get_for_device(&library, 0x9999, 0x8888);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_for_device_error_message() {
        let library = DeviceDefinitionLibrary::new();
        let result = get_for_device(&library, 0x1234, 0x5678);
        assert!(result.is_err());

        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("1234:5678"));
        assert!(msg.contains("not found"));
    }

    #[test]
    fn test_list_all_multiple_definitions() {
        let temp_dir = TempDir::new().unwrap();

        // Create first definition
        let file1 = temp_dir.path().join("device1.toml");
        std::fs::write(&file1, create_test_definition_toml()).unwrap();

        // Create second definition with different VID:PID
        let toml2 = create_test_definition_toml()
            .replace("0x5678", "0x9999")
            .replace("Test Device", "Test Device 2");
        let file2 = temp_dir.path().join("device2.toml");
        std::fs::write(&file2, toml2).unwrap();

        let mut library = DeviceDefinitionLibrary::new();
        library.load_from_directory(temp_dir.path()).unwrap();

        let result = list_all(&library);
        assert!(result.is_ok());
        let definitions = result.unwrap();
        assert_eq!(definitions.len(), 2);

        // Verify both definitions are present
        let names: Vec<_> = definitions.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"Test Device"));
        assert!(names.contains(&"Test Device 2"));
    }

    #[test]
    fn test_c_api_list_all() {
        let temp_dir = setup_runtime_with_definition();
        let msg = unsafe { c_string_result(keyrx_definitions_list_all()) };
        assert!(msg.starts_with("ok:"));

        let payload = msg.trim_start_matches("ok:");
        let definitions: Vec<DeviceDefinition> = serde_json::from_str(payload).unwrap();
        assert_eq!(definitions.len(), 1);

        clear_revolutionary_runtime().unwrap();
        drop(temp_dir);
    }

    #[test]
    fn test_c_api_get_for_device() {
        let temp_dir = setup_runtime_with_definition();
        let msg = unsafe { c_string_result(keyrx_definitions_get_for_device(0x1234, 0x5678)) };
        assert!(msg.starts_with("ok:"));

        let payload = msg.trim_start_matches("ok:");
        let definition: DeviceDefinition = serde_json::from_str(payload).unwrap();
        assert_eq!(definition.vendor_id, 0x1234);
        assert_eq!(definition.product_id, 0x5678);

        clear_revolutionary_runtime().unwrap();
        drop(temp_dir);
    }

    #[test]
    fn test_c_api_get_for_device_valid_ids() {
        let temp_dir = setup_runtime_with_definition();
        let msg = unsafe { c_string_result(keyrx_definitions_get_for_device(0x0001, 0x0001)) };
        assert!(msg.starts_with("error:"));

        clear_revolutionary_runtime().unwrap();
        drop(temp_dir);
    }
}
