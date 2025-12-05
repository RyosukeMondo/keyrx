//! Device Definitions domain FFI implementation.
//!
//! Implements FFI exports for device definitions library,
//! exposing read-only access to device definitions for Flutter.
#![allow(unsafe_code)]

use crate::definitions::{DeviceDefinition, DeviceDefinitionLibrary};
use crate::ffi::error::{FfiError, FfiResult};
use std::ffi::{c_char, CString};

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

/// List all loaded device definitions.
///
/// Returns JSON string: `ok:<json>` or `error:<message>`.
///
/// # Safety
/// This function catches panics and returns an error string if a panic occurs.
#[no_mangle]
pub extern "C" fn keyrx_definitions_list_all() -> *mut c_char {
    std::panic::catch_unwind(|| {
        // TODO: This needs access to the DeviceDefinitionLibrary instance from the engine
        // For now, return an error indicating this needs integration
        let error_msg = "error:DeviceDefinitionLibrary not yet integrated with FFI context";
        CString::new(error_msg)
            .map(CString::into_raw)
            .unwrap_or(std::ptr::null_mut())
    })
    .unwrap_or_else(|_| {
        CString::new("error:panic in keyrx_definitions_list_all")
            .map(CString::into_raw)
            .unwrap_or(std::ptr::null_mut())
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
        // TODO: This needs access to the DeviceDefinitionLibrary instance from the engine
        // For now, return an error indicating this needs integration
        let error_msg = format!(
            "error:DeviceDefinitionLibrary not yet integrated (would get definition for {:04x}:{:04x})",
            vendor_id, product_id
        );
        CString::new(error_msg)
            .map(CString::into_raw)
            .unwrap_or(std::ptr::null_mut())
    })
    .unwrap_or_else(|_| {
        CString::new("error:panic in keyrx_definitions_get_for_device")
            .map(CString::into_raw)
            .unwrap_or(std::ptr::null_mut())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definitions::DeviceDefinitionLibrary;
    use std::ffi::CStr;
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
        let result = keyrx_definitions_list_all();
        assert!(!result.is_null());

        unsafe {
            let c_str = CStr::from_ptr(result);
            let msg = c_str.to_str().unwrap();
            // Should indicate integration needed
            assert!(msg.contains("not yet integrated"));
            drop(CString::from_raw(result));
        }
    }

    #[test]
    fn test_c_api_get_for_device() {
        let result = keyrx_definitions_get_for_device(0x1234, 0x5678);
        assert!(!result.is_null());

        unsafe {
            let c_str = CStr::from_ptr(result);
            let msg = c_str.to_str().unwrap();
            // Should indicate integration needed and include VID:PID
            assert!(msg.contains("not yet integrated"));
            assert!(msg.contains("1234:5678"));
            drop(CString::from_raw(result));
        }
    }

    #[test]
    fn test_c_api_get_for_device_valid_ids() {
        // Test with various valid VID:PID combinations
        let test_cases = vec![(0x0001, 0x0001), (0xFFFF, 0xFFFF), (0x0fd9, 0x0080)];

        for (vid, pid) in test_cases {
            let result = keyrx_definitions_get_for_device(vid, pid);
            assert!(!result.is_null());

            unsafe {
                let c_str = CStr::from_ptr(result);
                let msg = c_str.to_str().unwrap();
                assert!(msg.starts_with("error:"));
                drop(CString::from_raw(result));
            }
        }
    }
}
