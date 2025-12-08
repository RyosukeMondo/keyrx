//! Comprehensive FFI tests for revolutionary mapping domains.
//!
//! Tests all FFI functions from device_registry, profile_registry, and device_definitions
//! domains for:
//! - Valid and invalid inputs
//! - Panic safety (panics don't cross FFI boundary)
//! - Memory safety (no leaks, proper cleanup)
//! - Null pointer handling
//! - UTF-8 validation
//! - Error propagation
//!
//! Tests are divided into two categories:
//! 1. Rust API tests: Test the async Rust functions that provide FFI functionality
//! 2. C API tests: Test the extern "C" functions that wrap the Rust API

use keyrx_core::definitions::DeviceDefinitionLibrary;
use keyrx_core::ffi::domains::device_definitions;
use keyrx_core::ffi::domains::device_registry;
use keyrx_core::ffi::domains::profile_registry;
use keyrx_core::identity::DeviceIdentity;
use keyrx_core::registry::profile::{LayoutType, Profile, ProfileRegistry};
use keyrx_core::registry::DeviceRegistry;
use keyrx_core::{clear_revolutionary_runtime, set_revolutionary_runtime, RevolutionaryRuntime};
use serde_json::Value;
use serial_test::serial;
use std::ffi::{CStr, CString};
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::tempdir;

// ============================================================================
// Helper Functions
// ============================================================================

fn test_identity(serial: &str) -> DeviceIdentity {
    DeviceIdentity::new(0x1234, 0x5678, serial.to_string())
}

fn test_profile(name: &str, layout: LayoutType) -> Profile {
    Profile::new(name, layout)
}

/// Helper to safely convert a C string result and free it
#[allow(dead_code)]
unsafe fn get_result_string(ptr: *mut i8) -> String {
    assert!(!ptr.is_null(), "FFI function returned null pointer");
    let c_str = CStr::from_ptr(ptr);
    let result = c_str
        .to_str()
        .expect("Result is not valid UTF-8")
        .to_string();
    drop(CString::from_raw(ptr));
    result
}

fn load_test_definitions() -> Arc<DeviceDefinitionLibrary> {
    let mut library = DeviceDefinitionLibrary::new();
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or(manifest_dir);
    let definitions_path = workspace_root.join("device_definitions");

    if definitions_path.exists() {
        library
            .load_from_directory(&definitions_path)
            .expect("device definitions should load for tests");
    }

    Arc::new(library)
}

fn setup_shared_runtime() -> (DeviceRegistry, Arc<ProfileRegistry>, tempfile::TempDir) {
    let (device_registry, _rx) = DeviceRegistry::new();
    let temp_dir = tempdir().unwrap();
    let profile_registry = Arc::new(ProfileRegistry::with_directory(
        temp_dir.path().to_path_buf(),
    ));

    let device_definitions = load_test_definitions();

    set_revolutionary_runtime(RevolutionaryRuntime::new(
        device_registry.clone(),
        profile_registry.clone(),
        device_definitions,
        Arc::new(std::sync::Mutex::new(keyrx_core::scripting::RhaiRuntime::new().unwrap()))
    ))
    .unwrap();

    (device_registry, profile_registry, temp_dir)
}

// ============================================================================
// Device Registry Rust API Tests
// ============================================================================

#[tokio::test]
async fn test_device_registry_list_devices_empty() {
    let (registry, _rx) = DeviceRegistry::new();
    let result = device_registry::list_devices(&registry).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[tokio::test]
async fn test_device_registry_list_devices_with_devices() {
    let (registry, _rx) = DeviceRegistry::new();
    let id1 = test_identity("TEST001");
    let id2 = test_identity("TEST002");

    registry.register_device(id1.clone()).await;
    registry.register_device(id2.clone()).await;

    let result = device_registry::list_devices(&registry).await;
    assert!(result.is_ok());
    let devices = result.unwrap();
    assert_eq!(devices.len(), 2);
}

#[tokio::test]
async fn test_device_registry_set_remap_enabled_valid() {
    let (registry, _rx) = DeviceRegistry::new();
    let identity = test_identity("TEST001");
    registry.register_device(identity.clone()).await;

    let key = identity.to_key();
    let result = device_registry::set_remap_enabled(&registry, &key, true).await;
    assert!(result.is_ok());

    let state = registry.get_device_state(&identity).await.unwrap();
    assert!(state.remap_enabled);
}

#[tokio::test]
async fn test_device_registry_set_remap_enabled_invalid_key() {
    let (registry, _rx) = DeviceRegistry::new();
    let result = device_registry::set_remap_enabled(&registry, "invalid", true).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_device_registry_set_remap_enabled_device_not_found() {
    let (registry, _rx) = DeviceRegistry::new();
    let identity = test_identity("TEST001");
    let key = identity.to_key();

    let result = device_registry::set_remap_enabled(&registry, &key, true).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_device_registry_assign_profile_valid() {
    let (registry, _rx) = DeviceRegistry::new();
    let identity = test_identity("TEST001");
    registry.register_device(identity.clone()).await;

    let key = identity.to_key();
    let result = device_registry::assign_profile(&registry, &key, "profile-123").await;
    assert!(result.is_ok());

    let state = registry.get_device_state(&identity).await.unwrap();
    assert_eq!(state.profile_id, Some("profile-123".to_string()));
}

#[tokio::test]
async fn test_device_registry_assign_profile_invalid_key() {
    let (registry, _rx) = DeviceRegistry::new();
    let result = device_registry::assign_profile(&registry, "invalid", "profile-123").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_device_registry_set_user_label_valid() {
    let (registry, _rx) = DeviceRegistry::new();
    let identity = test_identity("TEST001");
    registry.register_device(identity.clone()).await;

    let key = identity.to_key();
    let result =
        device_registry::set_user_label(&registry, &key, Some("My Device".to_string())).await;
    assert!(result.is_ok());

    let state = registry.get_device_state(&identity).await.unwrap();
    assert_eq!(state.identity.user_label, Some("My Device".to_string()));
}

#[tokio::test]
async fn test_device_registry_set_user_label_clear() {
    let (registry, _rx) = DeviceRegistry::new();
    let mut identity = test_identity("TEST001");
    identity.user_label = Some("Old Label".to_string());
    registry.register_device(identity.clone()).await;

    let key = identity.to_key();
    let result = device_registry::set_user_label(&registry, &key, None).await;
    assert!(result.is_ok());

    let state = registry.get_device_state(&identity).await.unwrap();
    assert_eq!(state.identity.user_label, None);
}

// ============================================================================
// Profile Registry Rust API Tests
// ============================================================================

#[tokio::test]
async fn test_profile_registry_list_profiles_empty() {
    let temp = tempdir().unwrap();
    let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

    let result = profile_registry::list_profiles(&registry).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[tokio::test]
async fn test_profile_registry_list_profiles_with_profiles() {
    let temp = tempdir().unwrap();
    let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

    let profile1 = test_profile("Profile 1", LayoutType::Matrix);
    let profile2 = test_profile("Profile 2", LayoutType::Standard);

    registry.save_profile(&profile1).await.unwrap();
    registry.save_profile(&profile2).await.unwrap();

    let result = profile_registry::list_profiles(&registry).await;
    assert!(result.is_ok());
    let profiles = result.unwrap();
    assert_eq!(profiles.len(), 2);
    assert!(profiles.contains(&profile1.id));
    assert!(profiles.contains(&profile2.id));
}

#[tokio::test]
async fn test_profile_registry_get_profile_valid() {
    let temp = tempdir().unwrap();
    let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

    let profile = test_profile("Test Profile", LayoutType::Matrix);
    registry.save_profile(&profile).await.unwrap();

    let result = profile_registry::get_profile(&registry, &profile.id).await;
    assert!(result.is_ok());
    let loaded = result.unwrap();
    assert_eq!(loaded.id, profile.id);
    assert_eq!(loaded.name, profile.name);
}

#[tokio::test]
async fn test_profile_registry_get_profile_not_found() {
    let temp = tempdir().unwrap();
    let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

    let result = profile_registry::get_profile(&registry, "nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_profile_registry_save_profile_valid() {
    let temp = tempdir().unwrap();
    let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

    let profile = test_profile("Test Profile", LayoutType::Standard);
    let json = serde_json::to_string(&profile).unwrap();

    let result = profile_registry::save_profile(&registry, &json).await;
    assert!(result.is_ok());

    // Verify it was saved
    let loaded = registry.get_profile(&profile.id).await.unwrap();
    assert_eq!(loaded.name, profile.name);
}

#[tokio::test]
async fn test_profile_registry_save_profile_invalid_json() {
    let temp = tempdir().unwrap();
    let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

    let result = profile_registry::save_profile(&registry, "invalid json").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_profile_registry_delete_profile_valid() {
    let temp = tempdir().unwrap();
    let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

    let profile = test_profile("Test Profile", LayoutType::Matrix);
    registry.save_profile(&profile).await.unwrap();

    let result = profile_registry::delete_profile(&registry, &profile.id).await;
    assert!(result.is_ok());

    // Verify it was deleted
    assert!(registry.get_profile(&profile.id).await.is_err());
}

#[tokio::test]
async fn test_profile_registry_delete_profile_not_found() {
    let temp = tempdir().unwrap();
    let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

    let result = profile_registry::delete_profile(&registry, "nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_profile_registry_find_compatible_profiles_valid() {
    let temp = tempdir().unwrap();
    let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

    let matrix_profile = test_profile("Matrix Profile", LayoutType::Matrix);
    let standard_profile = test_profile("Standard Profile", LayoutType::Standard);

    registry.save_profile(&matrix_profile).await.unwrap();
    registry.save_profile(&standard_profile).await.unwrap();

    let result = profile_registry::find_compatible_profiles(&registry, "matrix").await;
    assert!(result.is_ok());
    let compatible = result.unwrap();
    assert_eq!(compatible.len(), 1);
    assert_eq!(compatible[0].id, matrix_profile.id);
}

#[tokio::test]
async fn test_profile_registry_find_compatible_profiles_invalid_layout() {
    let temp = tempdir().unwrap();
    let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

    let result = profile_registry::find_compatible_profiles(&registry, "invalid").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_profile_registry_find_compatible_all_layout_types() {
    let temp = tempdir().unwrap();
    let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

    // Test all three layout types
    for layout_str in &["standard", "matrix", "split"] {
        let result = profile_registry::find_compatible_profiles(&registry, layout_str).await;
        assert!(
            result.is_ok(),
            "Layout type '{}' should be valid",
            layout_str
        );
    }
}

// ============================================================================
// Device Definitions Rust API Tests
// ============================================================================

#[test]
fn test_definitions_list_all_empty() {
    let library = DeviceDefinitionLibrary::new();
    let result = device_definitions::list_all(&library);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[test]
fn test_definitions_get_for_device_not_found() {
    let library = DeviceDefinitionLibrary::new();
    let result = device_definitions::get_for_device(&library, 0x9999, 0x8888);
    assert!(result.is_err());
}

#[test]
fn test_definitions_get_for_device_error_contains_vid_pid() {
    let library = DeviceDefinitionLibrary::new();
    let result = device_definitions::get_for_device(&library, 0x1234, 0x5678);
    assert!(result.is_err());

    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("1234:5678"));
    assert!(msg.contains("not found") || msg.contains("Not found"));
}

// ============================================================================
// C API Tests (Testing the extern "C" wrappers)
// ============================================================================

#[test]
fn test_c_api_device_registry_null_device_key() {
    unsafe {
        let result = device_registry::keyrx_device_registry_set_remap_enabled(std::ptr::null(), 1);
        assert!(!result.is_null());
        let c_str = CStr::from_ptr(result);
        let msg = c_str.to_str().unwrap();
        assert!(msg.starts_with("error:"));
        assert!(msg.contains("null"));
        drop(CString::from_raw(result));
    }
}

#[test]
fn test_c_api_device_registry_null_profile_id() {
    unsafe {
        let device_key = CString::new("1234:5678:TEST001").unwrap();
        let result = device_registry::keyrx_device_registry_assign_profile(
            device_key.as_ptr(),
            std::ptr::null(),
        );
        assert!(!result.is_null());
        let c_str = CStr::from_ptr(result);
        let msg = c_str.to_str().unwrap();
        assert!(msg.starts_with("error:"));
        assert!(msg.contains("profile_id") && msg.contains("null"));
        drop(CString::from_raw(result));
    }
}

#[test]
#[serial]
fn test_c_api_device_registry_null_label_clears() {
    let (registry, _profiles, temp_dir) = setup_shared_runtime();
    let identity = test_identity("TEST001");

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        registry.register_device(identity.clone()).await;
        registry
            .set_user_label(&identity, Some("Existing".to_string()))
            .await
            .unwrap();
    });

    unsafe {
        let device_key = CString::new(identity.to_key()).unwrap();
        let result = device_registry::keyrx_device_registry_set_user_label(
            device_key.as_ptr(),
            std::ptr::null(),
        );
        assert!(!result.is_null());
        let msg = get_result_string(result);
        assert!(msg.starts_with("ok:"));
    }

    rt.block_on(async {
        let state = registry.get_device_state(&identity).await.unwrap();
        assert!(state.identity.user_label.is_none());
    });

    clear_revolutionary_runtime().unwrap();
    drop(temp_dir);
}

#[test]
fn test_c_api_profile_registry_null_profile_id() {
    unsafe {
        let result = profile_registry::keyrx_profile_registry_get_profile(std::ptr::null());
        assert!(!result.is_null());
        let c_str = CStr::from_ptr(result);
        let msg = c_str.to_str().unwrap();
        assert!(msg.starts_with("error:"));
        assert!(msg.contains("profile_id") && msg.contains("null"));
        drop(CString::from_raw(result));
    }
}

#[test]
fn test_c_api_profile_registry_null_profile_json() {
    unsafe {
        let result = profile_registry::keyrx_profile_registry_save_profile(std::ptr::null());
        assert!(!result.is_null());
        let c_str = CStr::from_ptr(result);
        let msg = c_str.to_str().unwrap();
        assert!(msg.starts_with("error:"));
        assert!(msg.contains("profile_json") && msg.contains("null"));
        drop(CString::from_raw(result));
    }
}

#[test]
#[serial]
fn test_c_api_profile_registry_invalid_json() {
    let (_device_registry, _profile_registry, temp_dir) = setup_shared_runtime();

    unsafe {
        let invalid_json = CString::new("not valid json").unwrap();
        let result = profile_registry::keyrx_profile_registry_save_profile(invalid_json.as_ptr());
        assert!(!result.is_null());
        let c_str = CStr::from_ptr(result);
        let msg = c_str.to_str().unwrap();
        assert!(
            msg.starts_with("error:"),
            "expected error response for invalid JSON, got {msg}"
        );
        drop(CString::from_raw(result));
    }

    clear_revolutionary_runtime().unwrap();
    drop(temp_dir);
}

#[test]
fn test_c_api_profile_registry_null_layout_type() {
    unsafe {
        let result =
            profile_registry::keyrx_profile_registry_find_compatible_profiles(std::ptr::null());
        assert!(!result.is_null());
        let c_str = CStr::from_ptr(result);
        let msg = c_str.to_str().unwrap();
        assert!(msg.starts_with("error:"));
        assert!(msg.contains("layout_type") && msg.contains("null"));
        drop(CString::from_raw(result));
    }
}

#[test]
fn test_c_api_profile_registry_invalid_layout_type() {
    unsafe {
        let invalid_layout = CString::new("invalid").unwrap();
        let result = profile_registry::keyrx_profile_registry_find_compatible_profiles(
            invalid_layout.as_ptr(),
        );
        assert!(!result.is_null());
        let c_str = CStr::from_ptr(result);
        let msg = c_str.to_str().unwrap();
        assert!(msg.contains("Invalid layout type"));
        drop(CString::from_raw(result));
    }
}

#[test]
#[serial]
fn test_c_api_profile_registry_valid_layout_types() {
    let (_device_registry, _profile_registry, _temp_dir) = setup_shared_runtime();

    unsafe {
        for layout_type in &["standard", "matrix", "split"] {
            let layout = CString::new(*layout_type).unwrap();
            let result =
                profile_registry::keyrx_profile_registry_find_compatible_profiles(layout.as_ptr());
            let msg = get_result_string(result);
            assert!(
                msg.starts_with("ok:"),
                "expected ok response for layout {}, got {}",
                layout_type,
                msg
            );
        }
    }

    clear_revolutionary_runtime().unwrap();
}

#[test]
#[serial]
fn test_c_api_device_registry_list_devices_round_trip() {
    let (device_registry, _profile_registry, temp_dir) = setup_shared_runtime();
    let identity = test_identity("SERIAL123");

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        device_registry.register_device(identity.clone()).await;
        device_registry
            .set_remap_enabled(&identity, true)
            .await
            .unwrap();
    });

    unsafe {
        let result = device_registry::keyrx_device_registry_list_devices();
        let raw = get_result_string(result);
        assert!(raw.starts_with("ok:"));

        let payload = raw.trim_start_matches("ok:");
        let devices: Vec<Value> = serde_json::from_str(payload).unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0]["identity"]["serial_number"], "SERIAL123");
        assert_eq!(devices[0]["remap_enabled"], Value::Bool(true));
    }

    clear_revolutionary_runtime().unwrap();
    drop(temp_dir);
}

#[test]
#[serial]
fn test_c_api_profile_registry_save_and_get_round_trip() {
    let (_device_registry, _profile_registry, temp_dir) = setup_shared_runtime();

    let profile = test_profile("FFI Profile", LayoutType::Matrix);
    let profile_json = serde_json::to_string(&profile).unwrap();
    let profile_id = profile.id.clone();
    let profile_name = profile.name.clone();

    unsafe {
        let profile_json_c = CString::new(profile_json.clone()).unwrap();
        let save_result =
            profile_registry::keyrx_profile_registry_save_profile(profile_json_c.as_ptr());
        let msg = get_result_string(save_result);
        assert!(msg.starts_with("ok:"));
    }

    unsafe {
        let id_c = CString::new(profile_id.clone()).unwrap();
        let get_result = profile_registry::keyrx_profile_registry_get_profile(id_c.as_ptr());
        let msg = get_result_string(get_result);
        assert!(msg.starts_with("ok:"));
        let payload = msg.trim_start_matches("ok:");
        let loaded: Profile = serde_json::from_str(payload).unwrap();
        assert_eq!(loaded.id, profile_id);
        assert_eq!(loaded.name, profile_name);
    }

    unsafe {
        let list_result = profile_registry::keyrx_profile_registry_list_profiles();
        let msg = get_result_string(list_result);
        assert!(msg.starts_with("ok:"));
        let payload = msg.trim_start_matches("ok:");
        let ids: Vec<String> = serde_json::from_str(payload).unwrap();
        assert!(ids.contains(&profile_id));
    }

    clear_revolutionary_runtime().unwrap();
    drop(temp_dir);
}

#[test]
#[serial]
fn test_c_api_definitions_list_all() {
    let (_device_registry, _profile_registry, temp_dir) = setup_shared_runtime();
    let result = device_definitions::keyrx_definitions_list_all();
    assert!(!result.is_null());

    let msg = unsafe { get_result_string(result) };
    assert!(msg.starts_with("ok:"), "expected ok response, got {}", msg);

    let payload = msg.trim_start_matches("ok:");
    let definitions: Vec<Value> = serde_json::from_str(payload).unwrap();
    assert!(
        !definitions.is_empty(),
        "expected at least one device definition"
    );

    clear_revolutionary_runtime().unwrap();
    drop(temp_dir);
}

#[test]
#[serial]
fn test_c_api_definitions_get_for_device_valid_ids() {
    let (_device_registry, _profile_registry, temp_dir) = setup_shared_runtime();
    let test_cases = vec![
        ((0x0fd9, 0x0080), true),
        ((0x0fd9, 0x006c), true),
        ((0xFFFF, 0x0001), true),
        ((0x1234, 0x5678), false),
    ];

    for ((vid, pid), should_exist) in test_cases {
        let msg = unsafe {
            get_result_string(device_definitions::keyrx_definitions_get_for_device(
                vid, pid,
            ))
        };
        if should_exist {
            assert!(
                msg.starts_with("ok:"),
                "expected ok response for {:04x}:{:04x}, got {}",
                vid,
                pid,
                msg
            );
            let payload = msg.trim_start_matches("ok:");
            let definition: Value = serde_json::from_str(payload).unwrap();
            assert_eq!(definition["vendor_id"].as_u64(), Some(vid as u64));
            assert_eq!(definition["product_id"].as_u64(), Some(pid as u64));
        } else {
            assert!(
                msg.starts_with("error:"),
                "expected error for {:04x}:{:04x}, got {}",
                vid,
                pid,
                msg
            );
        }
    }

    clear_revolutionary_runtime().unwrap();
    drop(temp_dir);
}

// ============================================================================
// Panic Safety Tests
// ============================================================================

#[test]
fn test_panic_safety_no_segfault_on_repeated_c_calls() {
    // Call each C FFI function multiple times to ensure no state corruption
    unsafe {
        for _ in 0..10 {
            let result = device_registry::keyrx_device_registry_list_devices();
            drop(CString::from_raw(result));

            let result = profile_registry::keyrx_profile_registry_list_profiles();
            drop(CString::from_raw(result));

            let result = device_definitions::keyrx_definitions_list_all();
            drop(CString::from_raw(result));
        }
    }
}

#[test]
fn test_panic_safety_all_nulls() {
    // Verify that passing nulls doesn't cause segfaults
    unsafe {
        let result = device_registry::keyrx_device_registry_set_remap_enabled(std::ptr::null(), 0);
        drop(CString::from_raw(result));

        let result = device_registry::keyrx_device_registry_assign_profile(
            std::ptr::null(),
            std::ptr::null(),
        );
        drop(CString::from_raw(result));

        let result = device_registry::keyrx_device_registry_set_user_label(
            std::ptr::null(),
            std::ptr::null(),
        );
        drop(CString::from_raw(result));

        let result = profile_registry::keyrx_profile_registry_get_profile(std::ptr::null());
        drop(CString::from_raw(result));

        let result = profile_registry::keyrx_profile_registry_save_profile(std::ptr::null());
        drop(CString::from_raw(result));

        let result = profile_registry::keyrx_profile_registry_delete_profile(std::ptr::null());
        drop(CString::from_raw(result));

        let result =
            profile_registry::keyrx_profile_registry_find_compatible_profiles(std::ptr::null());
        drop(CString::from_raw(result));
    }
}

// ============================================================================
// Memory Safety Tests
// ============================================================================

#[test]
fn test_memory_safety_no_leaks_on_repeated_calls() {
    // Call C FFI functions many times to check for memory leaks
    unsafe {
        for _ in 0..100 {
            let device_key = CString::new("1234:5678:TEST").unwrap();
            let result =
                device_registry::keyrx_device_registry_set_remap_enabled(device_key.as_ptr(), 1);
            drop(CString::from_raw(result));

            let profile_id = CString::new("profile-123").unwrap();
            let result = profile_registry::keyrx_profile_registry_get_profile(profile_id.as_ptr());
            drop(CString::from_raw(result));

            let result = device_definitions::keyrx_definitions_get_for_device(0x1234, 0x5678);
            drop(CString::from_raw(result));
        }
    }
}

#[test]
fn test_memory_safety_strings_properly_freed() {
    // Ensure all returned strings are properly freed without double-free
    unsafe {
        let ptrs: Vec<_> = (0..10)
            .map(|_| device_registry::keyrx_device_registry_list_devices())
            .collect();

        for ptr in ptrs {
            drop(CString::from_raw(ptr));
        }
    }
}

// ============================================================================
// UTF-8 Validation Tests
// ============================================================================

#[tokio::test]
async fn test_utf8_validation_device_key_with_unicode() {
    let (registry, _rx) = DeviceRegistry::new();
    // Valid UTF-8 with unicode characters (but invalid device key format)
    let result = device_registry::set_remap_enabled(&registry, "1234:5678:键盘", true).await;
    // Should error on device not found, not UTF-8 validation
    assert!(result.is_err());
}

#[tokio::test]
async fn test_utf8_validation_profile_name_with_emoji() {
    let temp = tempdir().unwrap();
    let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

    // Create a profile with emoji in the name
    let profile = test_profile("🎮 Gaming Profile", LayoutType::Standard);
    let json = serde_json::to_string(&profile).unwrap();

    let result = profile_registry::save_profile(&registry, &json).await;
    // Should accept valid UTF-8 JSON
    assert!(result.is_ok());
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[tokio::test]
async fn test_edge_case_empty_strings() {
    let (registry, _rx) = DeviceRegistry::new();
    let result = device_registry::set_remap_enabled(&registry, "", true).await;
    assert!(result.is_err());

    let temp = tempdir().unwrap();
    let profile_registry = ProfileRegistry::with_directory(temp.path().to_path_buf());
    let result = profile_registry::get_profile(&profile_registry, "").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_edge_case_very_long_strings() {
    let temp = tempdir().unwrap();
    let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

    let long_id = "A".repeat(10000);
    let result = profile_registry::get_profile(&registry, &long_id).await;
    assert!(result.is_err());
}

#[test]
fn test_edge_case_special_vid_pid_values() {
    let library = DeviceDefinitionLibrary::new();

    // Test various edge case VID:PID values
    let test_cases = vec![
        (0x0000, 0x0000), // Zero
        (0xFFFF, 0xFFFF), // Max
        (0x0001, 0xFFFF), // Mixed
        (0xFFFF, 0x0001), // Mixed
    ];

    for (vid, pid) in test_cases {
        let result = device_definitions::get_for_device(&library, vid, pid);
        assert!(
            result.is_err(),
            "Should not find definition for {:04x}:{:04x}",
            vid,
            pid
        );
    }
}

// ============================================================================
// Concurrency Tests
// ============================================================================

#[test]
fn test_concurrency_parallel_c_api_calls() {
    use std::thread;

    // Spawn multiple threads calling C FFI functions simultaneously
    let handles: Vec<_> = (0..10)
        .map(|i| {
            thread::spawn(move || unsafe {
                let device_key = CString::new(format!("1234:5678:TEST{:03}", i)).unwrap();
                let result = device_registry::keyrx_device_registry_set_remap_enabled(
                    device_key.as_ptr(),
                    1,
                );
                drop(CString::from_raw(result));

                let result = profile_registry::keyrx_profile_registry_list_profiles();
                drop(CString::from_raw(result));

                let result = device_definitions::keyrx_definitions_list_all();
                drop(CString::from_raw(result));
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}

#[tokio::test]
async fn test_concurrency_parallel_async_calls() {
    let (registry, _rx) = DeviceRegistry::new();

    // Register multiple devices
    for i in 0..5 {
        let identity = DeviceIdentity::new(0x1234, 0x5678, format!("TEST{:03}", i));
        registry.register_device(identity).await;
    }

    // Spawn multiple concurrent tasks
    let mut handles = vec![];
    for i in 0..5 {
        let registry = registry.clone();
        let key = format!("1234:5678:TEST{:03}", i);
        handles.push(tokio::spawn(async move {
            device_registry::set_remap_enabled(&registry, &key, i % 2 == 0).await
        }));
    }

    // Wait for all tasks
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }
}
