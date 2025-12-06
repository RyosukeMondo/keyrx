//! Device Registry domain FFI implementation.
//!
//! Implements FFI exports for the revolutionary mapping device registry,
//! exposing device management operations to Flutter.
#![allow(unsafe_code)]

use crate::drivers;
use crate::ffi::error::{serialize_ffi_result, FfiError, FfiResult};
use crate::ffi::runtime::{block_on_ffi, with_revolutionary_runtime};
use crate::identity::DeviceIdentity;
use crate::registry::{DeviceRegistry, DeviceState};
use serde::{Deserialize, Serialize};
use std::ffi::{c_char, CStr, CString};
use std::ptr;

/// Device state for FFI serialization.
///
/// This mirrors the DeviceState from registry but is explicitly designed for FFI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FfiDeviceState {
    pub identity: DeviceIdentity,
    pub remap_enabled: bool,
    pub profile_id: Option<String>,
    pub connected_at: String,
    pub updated_at: String,
}

impl From<DeviceState> for FfiDeviceState {
    fn from(state: DeviceState) -> Self {
        Self {
            identity: state.identity,
            remap_enabled: state.remap_enabled,
            profile_id: state.profile_id,
            connected_at: state.connected_at,
            updated_at: state.updated_at,
        }
    }
}

fn extract_serial(path: &std::path::Path) -> String {
    let path_str = path.to_string_lossy();
    // On Windows, paths are like \\?\HID#VID_...&PID_...&MI_...#<InstanceID>#...
    // We want the InstanceID (3rd component) as the serial/unique ID.
    if cfg!(windows) && path_str.contains('#') {
        let parts: Vec<&str> = path_str.split('#').collect();
        if parts.len() >= 3 {
            return parts[2].to_string();
        }
    }
    path_str.to_string()
}

/// List all registered devices.
///
/// Returns JSON array of device states.
///
/// # Returns
/// * `Ok(Vec<FfiDeviceState>)` - List of registered devices
pub async fn list_devices(registry: &DeviceRegistry) -> FfiResult<Vec<FfiDeviceState>> {
    // 1. Scan for physical devices
    // We use spawn_blocking because list_keyboards might do I/O
    let physical_devices = tokio::task::spawn_blocking(|| drivers::list_keyboards())
        .await
        .map_err(|e| FfiError::internal(format!("JoinError in list_devices: {}", e)))?
        .unwrap_or_default();

    // 2. Register found devices and track them
    let mut found_identities = std::collections::HashSet::new();
    for device in physical_devices {
        let serial = extract_serial(&device.path);
        let identity = DeviceIdentity::new(device.vendor_id, device.product_id, serial);
        found_identities.insert(identity.clone());
        registry.register_device(identity).await;
    }

    // 3. Unregister disconnected devices
    // Get current list from registry to check against found physical devices
    let registered_devices = registry.list_devices().await;
    for device in registered_devices {
        if !found_identities.contains(&device.identity) {
            registry.unregister_device(&device.identity).await;
        }
    }

    // 4. Return current registry state (now synchronized)
    let devices = registry.list_devices().await;
    let ffi_devices = devices.into_iter().map(FfiDeviceState::from).collect();
    Ok(ffi_devices)
}

/// Set remap enabled state for a device.
///
/// # Arguments
/// * `registry` - Device registry
/// * `device_key` - Device identity key (format: "VID:PID:SERIAL")
/// * `enabled` - Whether remapping should be enabled
///
/// # Returns
/// * `Ok(())` on success
/// * `Err(FfiError)` if device not found or key is invalid
pub async fn set_remap_enabled(
    registry: &DeviceRegistry,
    device_key: &str,
    enabled: bool,
) -> FfiResult<()> {
    let identity = DeviceIdentity::from_key(device_key)
        .map_err(|e| FfiError::invalid_input(format!("Invalid device key: {}", e)))?;

    registry
        .set_remap_enabled(&identity, enabled)
        .await
        .map_err(|e| FfiError::not_found(format!("Device not found: {}", e)))?;

    Ok(())
}

/// Assign a profile to a device.
///
/// # Arguments
/// * `registry` - Device registry
/// * `device_key` - Device identity key (format: "VID:PID:SERIAL")
/// * `profile_id` - Profile ID to assign
///
/// # Returns
/// * `Ok(())` on success
/// * `Err(FfiError)` if device not found or key is invalid
pub async fn assign_profile(
    registry: &DeviceRegistry,
    device_key: &str,
    profile_id: &str,
) -> FfiResult<()> {
    let identity = DeviceIdentity::from_key(device_key)
        .map_err(|e| FfiError::invalid_input(format!("Invalid device key: {}", e)))?;

    registry
        .assign_profile(&identity, profile_id.to_string())
        .await
        .map_err(|e| FfiError::not_found(format!("Device not found: {}", e)))?;

    Ok(())
}

/// Set user label for a device.
///
/// # Arguments
/// * `registry` - Device registry
/// * `device_key` - Device identity key (format: "VID:PID:SERIAL")
/// * `label` - Optional user label (None to clear)
///
/// # Returns
/// * `Ok(())` on success
/// * `Err(FfiError)` if device not found or key is invalid
pub async fn set_user_label(
    registry: &DeviceRegistry,
    device_key: &str,
    label: Option<String>,
) -> FfiResult<()> {
    let identity = DeviceIdentity::from_key(device_key)
        .map_err(|e| FfiError::invalid_input(format!("Invalid device key: {}", e)))?;

    registry
        .set_user_label(&identity, label)
        .await
        .map_err(|e| FfiError::not_found(format!("Device not found: {}", e)))?;

    Ok(())
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

/// # Safety
///
/// `ptr` must be a valid, null-terminated C string.
unsafe fn parse_c_string(ptr: *const c_char, name: &str) -> FfiResult<String> {
    if ptr.is_null() {
        return Err(FfiError::null_pointer(name));
    }

    CStr::from_ptr(ptr)
        .to_str()
        .map(|s| s.to_string())
        .map_err(|_| FfiError::invalid_utf8(name))
}

/// # Safety
///
/// `ptr` may be null. If non-null, it must be a valid, null-terminated C string.
unsafe fn parse_optional_c_string(ptr: *const c_char, name: &str) -> FfiResult<Option<String>> {
    if ptr.is_null() {
        return Ok(None);
    }

    parse_c_string(ptr, name).map(Some)
}

/// List all registered devices.
///
/// Returns JSON string: `ok:<json>` or `error:<message>`.
///
/// # Safety
/// This function catches panics and returns an error string if a panic occurs.
#[no_mangle]
pub extern "C" fn keyrx_device_registry_list_devices() -> *mut c_char {
    std::panic::catch_unwind(|| {
        let result: FfiResult<Vec<FfiDeviceState>> = with_revolutionary_runtime(|runtime| {
            block_on_ffi(list_devices(runtime.device_registry()))
        });
        ffi_response(result)
    })
    .unwrap_or_else(|_| {
        ffi_response::<()>(Err(FfiError::internal(
            "panic in keyrx_device_registry_list_devices",
        )))
    })
}

/// Set remap enabled state for a device.
///
/// Returns `ok:` or `error:<message>`.
///
/// # Arguments
/// * `device_key` - Device identity key (format: "VID:PID:SERIAL")
/// * `enabled` - 1 for enabled, 0 for disabled
///
/// # Safety
/// * `device_key` must be a valid null-terminated C string
/// * This function catches panics and returns an error string if a panic occurs
#[no_mangle]
pub unsafe extern "C" fn keyrx_device_registry_set_remap_enabled(
    device_key: *const c_char,
    enabled: i32,
) -> *mut c_char {
    std::panic::catch_unwind(|| {
        let key_str = match parse_c_string(device_key, "device_key") {
            Ok(key) => key,
            Err(err) => return ffi_response::<()>(Err(err)),
        };

        let enabled = enabled != 0;
        let result: FfiResult<()> = with_revolutionary_runtime(|runtime| {
            block_on_ffi(set_remap_enabled(
                runtime.device_registry(),
                &key_str,
                enabled,
            ))
        });

        ffi_response(result)
    })
    .unwrap_or_else(|_| {
        ffi_response::<()>(Err(FfiError::internal(
            "panic in keyrx_device_registry_set_remap_enabled",
        )))
    })
}

/// Assign a profile to a device.
///
/// Returns `ok:` or `error:<message>`.
///
/// # Arguments
/// * `device_key` - Device identity key (format: "VID:PID:SERIAL")
/// * `profile_id` - Profile ID to assign
///
/// # Safety
/// * `device_key` and `profile_id` must be valid null-terminated C strings
/// * This function catches panics and returns an error string if a panic occurs
#[no_mangle]
pub unsafe extern "C" fn keyrx_device_registry_assign_profile(
    device_key: *const c_char,
    profile_id: *const c_char,
) -> *mut c_char {
    std::panic::catch_unwind(|| {
        let key_str = match parse_c_string(device_key, "device_key") {
            Ok(key) => key,
            Err(err) => return ffi_response::<()>(Err(err)),
        };

        let profile_str = match parse_c_string(profile_id, "profile_id") {
            Ok(profile) => profile,
            Err(err) => return ffi_response::<()>(Err(err)),
        };

        let result: FfiResult<()> = with_revolutionary_runtime(|runtime| {
            block_on_ffi(assign_profile(
                runtime.device_registry(),
                &key_str,
                &profile_str,
            ))
        });

        ffi_response(result)
    })
    .unwrap_or_else(|_| {
        ffi_response::<()>(Err(FfiError::internal(
            "panic in keyrx_device_registry_assign_profile",
        )))
    })
}

/// Set user label for a device.
///
/// Returns `ok:` or `error:<message>`.
///
/// # Arguments
/// * `device_key` - Device identity key (format: "VID:PID:SERIAL")
/// * `label` - User label (pass NULL to clear)
///
/// # Safety
/// * `device_key` must be a valid null-terminated C string
/// * `label` may be NULL to clear the label
/// * This function catches panics and returns an error string if a panic occurs
#[no_mangle]
pub unsafe extern "C" fn keyrx_device_registry_set_user_label(
    device_key: *const c_char,
    label: *const c_char,
) -> *mut c_char {
    std::panic::catch_unwind(|| {
        let key_str = match parse_c_string(device_key, "device_key") {
            Ok(key) => key,
            Err(err) => return ffi_response::<()>(Err(err)),
        };

        let label_opt = match parse_optional_c_string(label, "label") {
            Ok(label) => label,
            Err(err) => return ffi_response::<()>(Err(err)),
        };

        let result: FfiResult<()> = with_revolutionary_runtime(|runtime| {
            block_on_ffi(set_user_label(
                runtime.device_registry(),
                &key_str,
                label_opt,
            ))
        });

        ffi_response(result)
    })
    .unwrap_or_else(|_| {
        ffi_response::<()>(Err(FfiError::internal(
            "panic in keyrx_device_registry_set_user_label",
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
    use std::sync::Arc;
    use tempfile::tempdir;
    use tokio::runtime::Runtime;

    fn test_identity(serial: &str) -> DeviceIdentity {
        DeviceIdentity::new(0x1234, 0x5678, serial.to_string())
    }

    #[tokio::test]
    async fn test_list_devices_empty() {
        let (registry, _rx) = DeviceRegistry::new();
        let result = list_devices(&registry).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_list_devices_with_devices() {
        let (registry, _rx) = DeviceRegistry::new();
        let id1 = test_identity("TEST001");
        let id2 = test_identity("TEST002");

        registry.register_device(id1.clone()).await;
        registry.register_device(id2.clone()).await;

        let result = list_devices(&registry).await;
        assert!(result.is_ok());
        let devices = result.unwrap();
        assert_eq!(devices.len(), 2);
    }

    #[tokio::test]
    async fn test_set_remap_enabled_valid() {
        let (registry, _rx) = DeviceRegistry::new();
        let identity = test_identity("TEST001");
        registry.register_device(identity.clone()).await;

        let key = identity.to_key();
        let result = set_remap_enabled(&registry, &key, true).await;
        assert!(result.is_ok());

        let state = registry.get_device_state(&identity).await.unwrap();
        assert!(state.remap_enabled);
    }

    #[tokio::test]
    async fn test_set_remap_enabled_invalid_key() {
        let (registry, _rx) = DeviceRegistry::new();
        let result = set_remap_enabled(&registry, "invalid", true).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_set_remap_enabled_device_not_found() {
        let (registry, _rx) = DeviceRegistry::new();
        let identity = test_identity("TEST001");
        let key = identity.to_key();

        let result = set_remap_enabled(&registry, &key, true).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_assign_profile_valid() {
        let (registry, _rx) = DeviceRegistry::new();
        let identity = test_identity("TEST001");
        registry.register_device(identity.clone()).await;

        let key = identity.to_key();
        let result = assign_profile(&registry, &key, "profile-123").await;
        assert!(result.is_ok());

        let state = registry.get_device_state(&identity).await.unwrap();
        assert_eq!(state.profile_id, Some("profile-123".to_string()));
    }

    #[tokio::test]
    async fn test_assign_profile_invalid_key() {
        let (registry, _rx) = DeviceRegistry::new();
        let result = assign_profile(&registry, "invalid", "profile-123").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_set_user_label_valid() {
        let (registry, _rx) = DeviceRegistry::new();
        let identity = test_identity("TEST001");
        registry.register_device(identity.clone()).await;

        let key = identity.to_key();
        let result = set_user_label(&registry, &key, Some("My Device".to_string())).await;
        assert!(result.is_ok());

        let state = registry.get_device_state(&identity).await.unwrap();
        assert_eq!(state.identity.user_label, Some("My Device".to_string()));
    }

    #[tokio::test]
    async fn test_set_user_label_clear() {
        let (registry, _rx) = DeviceRegistry::new();
        let mut identity = test_identity("TEST001");
        identity.user_label = Some("Old Label".to_string());
        registry.register_device(identity.clone()).await;

        let key = identity.to_key();
        let result = set_user_label(&registry, &key, None).await;
        assert!(result.is_ok());

        let state = registry.get_device_state(&identity).await.unwrap();
        assert_eq!(state.identity.user_label, None);
    }

    #[test]
    fn test_c_api_null_device_key() {
        unsafe {
            let result = keyrx_device_registry_set_remap_enabled(std::ptr::null(), 1);
            assert!(!result.is_null());
            let c_str = CStr::from_ptr(result);
            let msg = c_str.to_str().unwrap();
            assert!(msg.starts_with("error:"));
            drop(CString::from_raw(result));
        }
    }

    #[test]
    fn test_c_api_null_profile_id() {
        unsafe {
            let device_key = CString::new("1234:5678:TEST001").unwrap();
            let result =
                keyrx_device_registry_assign_profile(device_key.as_ptr(), std::ptr::null());
            assert!(!result.is_null());
            let c_str = CStr::from_ptr(result);
            let msg = c_str.to_str().unwrap();
            assert!(msg.starts_with("error:"));
            drop(CString::from_raw(result));
        }
    }

    #[test]
    #[serial_test::serial]
    fn test_c_api_null_label_clears() {
        let (registry, _rx) = DeviceRegistry::new();
        let temp_dir = tempdir().unwrap();
        let profile_registry = Arc::new(ProfileRegistry::with_directory(
            temp_dir.path().to_path_buf(),
        ));
        let definitions = Arc::new(DeviceDefinitionLibrary::new());

        let _ = clear_revolutionary_runtime();
        set_revolutionary_runtime(RevolutionaryRuntime::new(
            registry.clone(),
            profile_registry,
            definitions,
        ))
        .unwrap();

        let identity = test_identity("TEST001");
        let key = identity.to_key();

        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            registry.register_device(identity.clone()).await;
            registry
                .set_user_label(&identity, Some("Existing".to_string()))
                .await
                .unwrap();
        });

        unsafe {
            let device_key = CString::new(key.clone()).unwrap();
            let result =
                keyrx_device_registry_set_user_label(device_key.as_ptr(), std::ptr::null());
            assert!(!result.is_null());
            let c_str = CStr::from_ptr(result);
            let msg = c_str.to_str().unwrap();
            assert!(msg.starts_with("ok:"));
            drop(CString::from_raw(result));
        }

        rt.block_on(async {
            let state = registry.get_device_state(&identity).await.unwrap();
            assert!(state.identity.user_label.is_none());
        });

        clear_revolutionary_runtime().unwrap();
    }
}
