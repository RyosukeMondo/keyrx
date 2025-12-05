//! Device Registry domain FFI implementation.
//!
//! Implements FFI exports for the revolutionary mapping device registry,
//! exposing device management operations to Flutter.
#![allow(unsafe_code)]

use crate::ffi::error::{FfiError, FfiResult};
use crate::identity::DeviceIdentity;
use crate::registry::{DeviceRegistry, DeviceState};
use serde::{Deserialize, Serialize};
use std::ffi::{c_char, CStr, CString};

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

/// List all registered devices.
///
/// Returns JSON array of device states.
///
/// # Returns
/// * `Ok(Vec<FfiDeviceState>)` - List of registered devices
pub async fn list_devices(registry: &DeviceRegistry) -> FfiResult<Vec<FfiDeviceState>> {
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

/// List all registered devices.
///
/// Returns JSON string: `ok:<json>` or `error:<message>`.
///
/// # Safety
/// This function catches panics and returns an error string if a panic occurs.
#[no_mangle]
pub extern "C" fn keyrx_device_registry_list_devices() -> *mut c_char {
    std::panic::catch_unwind(|| {
        // TODO: This needs access to the DeviceRegistry instance from the engine
        // For now, return an error indicating this needs integration
        let error_msg = "error:DeviceRegistry not yet integrated with FFI context";
        CString::new(error_msg)
            .map(CString::into_raw)
            .unwrap_or(std::ptr::null_mut())
    })
    .unwrap_or_else(|_| {
        CString::new("error:panic in keyrx_device_registry_list_devices")
            .map(CString::into_raw)
            .unwrap_or(std::ptr::null_mut())
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
        if device_key.is_null() {
            return CString::new("error:device_key is null")
                .map(CString::into_raw)
                .unwrap_or(std::ptr::null_mut());
        }

        let key_str = match CStr::from_ptr(device_key).to_str() {
            Ok(s) => s,
            Err(_) => {
                return CString::new("error:device_key is not valid UTF-8")
                    .map(CString::into_raw)
                    .unwrap_or(std::ptr::null_mut())
            }
        };

        // TODO: This needs access to the DeviceRegistry instance from the engine
        // For now, return an error indicating this needs integration
        let error_msg = format!(
            "error:DeviceRegistry not yet integrated (would set {} to {})",
            key_str,
            enabled != 0
        );
        CString::new(error_msg)
            .map(CString::into_raw)
            .unwrap_or(std::ptr::null_mut())
    })
    .unwrap_or_else(|_| {
        CString::new("error:panic in keyrx_device_registry_set_remap_enabled")
            .map(CString::into_raw)
            .unwrap_or(std::ptr::null_mut())
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
        if device_key.is_null() {
            return CString::new("error:device_key is null")
                .map(CString::into_raw)
                .unwrap_or(std::ptr::null_mut());
        }

        if profile_id.is_null() {
            return CString::new("error:profile_id is null")
                .map(CString::into_raw)
                .unwrap_or(std::ptr::null_mut());
        }

        let key_str = match CStr::from_ptr(device_key).to_str() {
            Ok(s) => s,
            Err(_) => {
                return CString::new("error:device_key is not valid UTF-8")
                    .map(CString::into_raw)
                    .unwrap_or(std::ptr::null_mut())
            }
        };

        let profile_str = match CStr::from_ptr(profile_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                return CString::new("error:profile_id is not valid UTF-8")
                    .map(CString::into_raw)
                    .unwrap_or(std::ptr::null_mut())
            }
        };

        // TODO: This needs access to the DeviceRegistry instance from the engine
        // For now, return an error indicating this needs integration
        let error_msg = format!(
            "error:DeviceRegistry not yet integrated (would assign {} to {})",
            profile_str, key_str
        );
        CString::new(error_msg)
            .map(CString::into_raw)
            .unwrap_or(std::ptr::null_mut())
    })
    .unwrap_or_else(|_| {
        CString::new("error:panic in keyrx_device_registry_assign_profile")
            .map(CString::into_raw)
            .unwrap_or(std::ptr::null_mut())
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
        if device_key.is_null() {
            return CString::new("error:device_key is null")
                .map(CString::into_raw)
                .unwrap_or(std::ptr::null_mut());
        }

        let key_str = match CStr::from_ptr(device_key).to_str() {
            Ok(s) => s,
            Err(_) => {
                return CString::new("error:device_key is not valid UTF-8")
                    .map(CString::into_raw)
                    .unwrap_or(std::ptr::null_mut())
            }
        };

        let label_opt = if label.is_null() {
            None
        } else {
            match CStr::from_ptr(label).to_str() {
                Ok(s) => Some(s.to_string()),
                Err(_) => {
                    return CString::new("error:label is not valid UTF-8")
                        .map(CString::into_raw)
                        .unwrap_or(std::ptr::null_mut())
                }
            }
        };

        // TODO: This needs access to the DeviceRegistry instance from the engine
        // For now, return an error indicating this needs integration
        let error_msg = format!(
            "error:DeviceRegistry not yet integrated (would set label {:?} on {})",
            label_opt, key_str
        );
        CString::new(error_msg)
            .map(CString::into_raw)
            .unwrap_or(std::ptr::null_mut())
    })
    .unwrap_or_else(|_| {
        CString::new("error:panic in keyrx_device_registry_set_user_label")
            .map(CString::into_raw)
            .unwrap_or(std::ptr::null_mut())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::DeviceRegistry;

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
    fn test_c_api_null_label_clears() {
        unsafe {
            let device_key = CString::new("1234:5678:TEST001").unwrap();
            let result =
                keyrx_device_registry_set_user_label(device_key.as_ptr(), std::ptr::null());
            assert!(!result.is_null());
            let c_str = CStr::from_ptr(result);
            let msg = c_str.to_str().unwrap();
            // Should indicate integration needed, not null pointer error
            assert!(msg.contains("not yet integrated"));
            drop(CString::from_raw(result));
        }
    }
}
