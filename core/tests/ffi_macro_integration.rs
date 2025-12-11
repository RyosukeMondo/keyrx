#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, unsafe_code)]
//! Integration tests for the keyrx_ffi_macro procedural macro.
//!
//! Tests that the macro correctly generates FFI functions from real contracts.
//! Verifies:
//! - Generated code compiles correctly
//! - FFI functions follow contract signatures
//! - Parameter parsing works
//! - Result serialization works
//! - Error handling via error pointer works

use keyrx_ffi_macro::keyrx_ffi;
use serde::{Deserialize, Serialize};
use serial_test::serial;
use std::ffi::{c_char, CStr, CString};

// ============================================================================
// Test Domain Implementation
// ============================================================================

/// Simple config storage for testing.
#[derive(Debug, Default)]
struct TestConfigStore {
    layouts: Vec<VirtualLayout>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VirtualLayout {
    id: String,
    name: String,
}

impl TestConfigStore {
    fn new() -> Self {
        Self::default()
    }

    fn list_virtual_layouts(&self) -> Result<String, String> {
        serde_json::to_string(&self.layouts).map_err(|e| e.to_string())
    }

    fn save_virtual_layout(&mut self, json: &str) -> Result<String, String> {
        let layout: VirtualLayout =
            serde_json::from_str(json).map_err(|e| format!("Invalid JSON: {e}"))?;
        self.layouts.retain(|l| l.id != layout.id);
        self.layouts.push(layout.clone());
        serde_json::to_string(&layout).map_err(|e| e.to_string())
    }

    fn delete_virtual_layout(&mut self, id: &str) -> Result<String, String> {
        let initial_len = self.layouts.len();
        self.layouts.retain(|l| l.id != id);
        if self.layouts.len() < initial_len {
            Ok(r#"{"success": true}"#.to_string())
        } else {
            Err(format!("Layout '{id}' not found"))
        }
    }
}

// ============================================================================
// Config Domain with FFI Macro
// ============================================================================

/// Static test config store for FFI functions.
/// In real code, this would be managed properly.
static mut TEST_CONFIG_STORE: Option<TestConfigStore> = None;

fn get_store() -> &'static TestConfigStore {
    unsafe { TEST_CONFIG_STORE.get_or_insert_with(TestConfigStore::new) }
}

fn get_store_mut() -> &'static mut TestConfigStore {
    unsafe { TEST_CONFIG_STORE.get_or_insert_with(TestConfigStore::new) }
}

fn reset_store() {
    unsafe {
        TEST_CONFIG_STORE = Some(TestConfigStore::new());
    }
}

/// ConfigDomain provides configuration management FFI functions.
///
/// This impl block is annotated with `#[keyrx_ffi]` which generates
/// extern "C" FFI wrapper functions for each method based on the
/// config.ffi-contract.json contract.
#[keyrx_ffi(domain = "config")]
impl ConfigDomain {
    /// List all virtual layouts.
    fn list_virtual_layouts() -> Result<String, String> {
        get_store().list_virtual_layouts()
    }

    /// Save or update a virtual layout.
    fn save_virtual_layout(json: String) -> Result<String, String> {
        get_store_mut().save_virtual_layout(&json)
    }

    /// Delete a virtual layout by ID.
    fn delete_virtual_layout(id: String) -> Result<String, String> {
        get_store_mut().delete_virtual_layout(&id)
    }

    /// List all hardware profiles.
    fn list_hardware_profiles() -> Result<String, String> {
        Ok("[]".to_string())
    }

    /// Save or update a hardware profile.
    fn save_hardware_profile(json: String) -> Result<String, String> {
        let _: serde_json::Value =
            serde_json::from_str(&json).map_err(|e| format!("Invalid JSON: {e}"))?;
        Ok(r#"{"success": true}"#.to_string())
    }

    /// Delete a hardware profile by ID.
    fn delete_hardware_profile(id: String) -> Result<String, String> {
        Err(format!("Profile '{id}' not found"))
    }

    /// List all keymaps.
    fn list_keymaps() -> Result<String, String> {
        Ok("[]".to_string())
    }

    /// Save or update a keymap.
    fn save_keymap(json: String) -> Result<String, String> {
        let _: serde_json::Value =
            serde_json::from_str(&json).map_err(|e| format!("Invalid JSON: {e}"))?;
        Ok(r#"{"success": true}"#.to_string())
    }

    /// Delete a keymap by ID.
    fn delete_keymap(id: String) -> Result<String, String> {
        Err(format!("Keymap '{id}' not found"))
    }
}

/// Placeholder struct for the impl block.
struct ConfigDomain;

// ============================================================================
// Helper Functions
// ============================================================================

/// Safely get result string and free memory.
unsafe fn get_result_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    let c_str = CStr::from_ptr(ptr);
    let result = c_str.to_str().ok()?.to_string();
    // Free the memory allocated by serialize_to_c_string
    drop(CString::from_raw(ptr as *mut c_char));
    Some(result)
}

/// Get error string and free memory.
unsafe fn get_error_string(error: *mut c_char) -> Option<String> {
    if error.is_null() {
        return None;
    }
    let c_str = CStr::from_ptr(error);
    let result = c_str.to_str().ok()?.to_string();
    drop(CString::from_raw(error));
    Some(result)
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
#[serial]
fn test_list_virtual_layouts_empty() {
    reset_store();

    let mut error: *mut c_char = std::ptr::null_mut();
    let result = unsafe { keyrx_config_list_virtual_layouts(&mut error) };

    assert!(error.is_null(), "Expected no error");

    let json = unsafe { get_result_string(result) };
    assert!(json.is_some());
    let json = json.unwrap();

    // The result is double-serialized: the method returns a JSON string,
    // which is then serialized again by ffi_wrapper. So we need to parse
    // the outer JSON string first, then parse the inner JSON.
    let inner_json: String = serde_json::from_str(&json).unwrap();
    let layouts: Vec<VirtualLayout> = serde_json::from_str(&inner_json).unwrap();
    assert!(layouts.is_empty());
}

#[test]
#[serial]
fn test_save_and_list_virtual_layout() {
    reset_store();

    // Save a layout
    let layout = VirtualLayout {
        id: "test-layout-1".to_string(),
        name: "Test Layout".to_string(),
    };
    let layout_json = serde_json::to_string(&layout).unwrap();
    let layout_json_c = CString::new(layout_json).unwrap();

    let mut error: *mut c_char = std::ptr::null_mut();
    let result = unsafe { keyrx_config_save_virtual_layout(&mut error, layout_json_c.as_ptr()) };

    assert!(error.is_null(), "Expected no error on save");
    let saved = unsafe { get_result_string(result) };
    assert!(saved.is_some());

    // List layouts
    let mut error: *mut c_char = std::ptr::null_mut();
    let result = unsafe { keyrx_config_list_virtual_layouts(&mut error) };

    assert!(error.is_null(), "Expected no error on list");
    let json = unsafe { get_result_string(result) }.unwrap();

    // Parse the double-serialized result
    let inner_json: String = serde_json::from_str(&json).unwrap();
    let layouts: Vec<VirtualLayout> = serde_json::from_str(&inner_json).unwrap();
    assert_eq!(layouts.len(), 1);
    assert_eq!(layouts[0].id, "test-layout-1");
    assert_eq!(layouts[0].name, "Test Layout");
}

#[test]
#[serial]
fn test_delete_virtual_layout() {
    reset_store();

    // Save a layout first
    let layout = VirtualLayout {
        id: "delete-me".to_string(),
        name: "To Delete".to_string(),
    };
    let layout_json = serde_json::to_string(&layout).unwrap();
    let layout_json_c = CString::new(layout_json).unwrap();

    let mut error: *mut c_char = std::ptr::null_mut();
    let save_result =
        unsafe { keyrx_config_save_virtual_layout(&mut error, layout_json_c.as_ptr()) };
    assert!(error.is_null(), "Expected no error on save");
    // Free the save result
    if !save_result.is_null() {
        unsafe { drop(CString::from_raw(save_result as *mut c_char)) };
    }

    // Delete the layout
    let id_c = CString::new("delete-me").unwrap();
    let mut error: *mut c_char = std::ptr::null_mut();
    let result = unsafe { keyrx_config_delete_virtual_layout(&mut error, id_c.as_ptr()) };

    assert!(error.is_null(), "Expected no error on delete");
    let response = unsafe { get_result_string(result) }.unwrap();
    // Result is double-serialized
    let inner_response: String = serde_json::from_str(&response).unwrap();
    assert!(inner_response.contains("success"));

    // Verify it's gone
    let mut error: *mut c_char = std::ptr::null_mut();
    let result = unsafe { keyrx_config_list_virtual_layouts(&mut error) };
    let json = unsafe { get_result_string(result) }.unwrap();
    let inner_json: String = serde_json::from_str(&json).unwrap();
    let layouts: Vec<VirtualLayout> = serde_json::from_str(&inner_json).unwrap();
    assert!(layouts.is_empty());
}

#[test]
#[serial]
fn test_delete_nonexistent_layout_returns_error() {
    reset_store();

    let id_c = CString::new("nonexistent").unwrap();
    let mut error: *mut c_char = std::ptr::null_mut();
    let result = unsafe { keyrx_config_delete_virtual_layout(&mut error, id_c.as_ptr()) };

    // Result should be null and error should be set
    assert!(result.is_null(), "Expected null result on error");
    assert!(!error.is_null(), "Expected error to be set");

    let error_msg = unsafe { get_error_string(error) }.unwrap();
    assert!(error_msg.contains("not found"));
}

#[test]
#[serial]
fn test_save_invalid_json_returns_error() {
    reset_store();

    let invalid_json = CString::new("not valid json").unwrap();
    let mut error: *mut c_char = std::ptr::null_mut();
    let result = unsafe { keyrx_config_save_virtual_layout(&mut error, invalid_json.as_ptr()) };

    assert!(result.is_null(), "Expected null result on error");
    assert!(!error.is_null(), "Expected error to be set");

    let error_msg = unsafe { get_error_string(error) }.unwrap();
    assert!(error_msg.contains("Invalid JSON"));
}

#[test]
#[serial]
fn test_null_parameter_returns_error() {
    reset_store();

    let mut error: *mut c_char = std::ptr::null_mut();
    let result = unsafe { keyrx_config_save_virtual_layout(&mut error, std::ptr::null()) };

    assert!(result.is_null(), "Expected null result on null param");
    assert!(!error.is_null(), "Expected error for null param");

    let error_msg = unsafe { get_error_string(error) }.unwrap();
    assert!(error_msg.contains("null") || error_msg.contains("Null"));
}

#[test]
fn test_list_hardware_profiles() {
    let mut error: *mut c_char = std::ptr::null_mut();
    let result = unsafe { keyrx_config_list_hardware_profiles(&mut error) };

    assert!(error.is_null());
    let json = unsafe { get_result_string(result) }.unwrap();
    // Result is double-serialized
    let inner: String = serde_json::from_str(&json).unwrap();
    assert_eq!(inner, "[]");
}

#[test]
fn test_list_keymaps() {
    let mut error: *mut c_char = std::ptr::null_mut();
    let result = unsafe { keyrx_config_list_keymaps(&mut error) };

    assert!(error.is_null());
    let json = unsafe { get_result_string(result) }.unwrap();
    // Result is double-serialized
    let inner: String = serde_json::from_str(&json).unwrap();
    assert_eq!(inner, "[]");
}

#[test]
fn test_delete_hardware_profile_not_found() {
    let id_c = CString::new("nonexistent").unwrap();
    let mut error: *mut c_char = std::ptr::null_mut();
    let result = unsafe { keyrx_config_delete_hardware_profile(&mut error, id_c.as_ptr()) };

    assert!(result.is_null());
    assert!(!error.is_null());

    let error_msg = unsafe { get_error_string(error) }.unwrap();
    assert!(error_msg.contains("not found"));
}

#[test]
fn test_delete_keymap_not_found() {
    let id_c = CString::new("nonexistent").unwrap();
    let mut error: *mut c_char = std::ptr::null_mut();
    let result = unsafe { keyrx_config_delete_keymap(&mut error, id_c.as_ptr()) };

    assert!(result.is_null());
    assert!(!error.is_null());

    let error_msg = unsafe { get_error_string(error) }.unwrap();
    assert!(error_msg.contains("not found"));
}

#[test]
fn test_save_hardware_profile_valid_json() {
    let profile_json = CString::new(r#"{"id": "test", "name": "Test Profile"}"#).unwrap();
    let mut error: *mut c_char = std::ptr::null_mut();
    let result = unsafe { keyrx_config_save_hardware_profile(&mut error, profile_json.as_ptr()) };

    assert!(error.is_null());
    let response = unsafe { get_result_string(result) }.unwrap();
    // Result is double-serialized
    let inner: String = serde_json::from_str(&response).unwrap();
    assert!(inner.contains("success"));
}

#[test]
fn test_save_keymap_valid_json() {
    let keymap_json = CString::new(r#"{"id": "test", "mappings": []}"#).unwrap();
    let mut error: *mut c_char = std::ptr::null_mut();
    let result = unsafe { keyrx_config_save_keymap(&mut error, keymap_json.as_ptr()) };

    assert!(error.is_null());
    let response = unsafe { get_result_string(result) }.unwrap();
    // Result is double-serialized
    let inner: String = serde_json::from_str(&response).unwrap();
    assert!(inner.contains("success"));
}

// ============================================================================
// FFI Safety Tests
// ============================================================================

#[test]
#[serial]
fn test_repeated_calls_no_memory_issues() {
    reset_store();

    for i in 0..100 {
        let layout = VirtualLayout {
            id: format!("layout-{i}"),
            name: format!("Layout {i}"),
        };
        let layout_json = serde_json::to_string(&layout).unwrap();
        let layout_json_c = CString::new(layout_json).unwrap();

        let mut error: *mut c_char = std::ptr::null_mut();
        let result =
            unsafe { keyrx_config_save_virtual_layout(&mut error, layout_json_c.as_ptr()) };

        if !result.is_null() {
            unsafe { drop(CString::from_raw(result as *mut c_char)) };
        }
        if !error.is_null() {
            unsafe { drop(CString::from_raw(error)) };
        }
    }

    // Verify all layouts are saved
    let mut error: *mut c_char = std::ptr::null_mut();
    let result = unsafe { keyrx_config_list_virtual_layouts(&mut error) };
    let json = unsafe { get_result_string(result) }.unwrap();
    // Result is double-serialized
    let inner: String = serde_json::from_str(&json).unwrap();
    let layouts: Vec<VirtualLayout> = serde_json::from_str(&inner).unwrap();
    assert_eq!(layouts.len(), 100);
}

#[test]
fn test_null_error_pointer_handled() {
    // Passing null error pointer should not cause a crash
    let result = unsafe { keyrx_config_list_virtual_layouts(std::ptr::null_mut()) };

    // Result should still be valid
    if !result.is_null() {
        unsafe { drop(CString::from_raw(result as *mut c_char)) };
    }
}

#[test]
#[serial]
fn test_error_pointer_not_modified_on_success() {
    reset_store();

    let mut error: *mut c_char = std::ptr::null_mut();
    let result = unsafe { keyrx_config_list_virtual_layouts(&mut error) };

    // On success, error pointer should remain null
    assert!(error.is_null());
    assert!(!result.is_null());

    unsafe { drop(CString::from_raw(result as *mut c_char)) };
}
