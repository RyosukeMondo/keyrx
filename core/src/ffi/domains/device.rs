//! Device domain FFI implementation.
//!
//! Implements the FfiExportable trait for device management.
//! Handles device listing, selection, and key registry.
#![allow(unsafe_code)]

use crate::discovery::storage::profile_path;
use crate::discovery::types::DeviceId;
use crate::drivers;
use crate::drivers::keycodes::key_definitions;
use crate::ffi::context::FfiContext;
use crate::ffi::error::{FfiError, FfiResult};
use crate::ffi::traits::FfiExportable;
// use keyrx_ffi_macros::ffi_export; // TODO: Uncomment when exports_*.rs files are removed (task 20)
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Device domain FFI implementation.
pub struct DeviceFfi;

/// Device state for FFI.
#[derive(Debug, Default)]
pub struct DeviceState {
    /// Currently selected device path
    pub selected_device: Option<PathBuf>,
}

impl FfiExportable for DeviceFfi {
    const DOMAIN: &'static str = "device";

    fn init(ctx: &mut FfiContext) -> Result<(), FfiError> {
        if ctx.has_domain(Self::DOMAIN) {
            return Err(FfiError::invalid_input("device domain already initialized"));
        }

        // Initialize device domain state
        ctx.set_domain(Self::DOMAIN, DeviceState::default());

        Ok(())
    }

    fn cleanup(ctx: &mut FfiContext) {
        // Remove domain state
        ctx.remove_domain(Self::DOMAIN);
    }
}

impl DeviceFfi {
    /// Get the currently selected device path.
    ///
    /// Used internally when starting the engine to determine which device to use.
    pub fn get_selected_device(ctx: &FfiContext) -> Option<PathBuf> {
        ctx.get_domain::<DeviceState>(Self::DOMAIN)
            .and_then(|state_guard| {
                state_guard
                    .downcast_ref::<DeviceState>()
                    .and_then(|state| state.selected_device.clone())
            })
    }
}

/// Device info with profile status for FFI.
#[derive(Clone, Serialize, Deserialize, keyrx_ffi_macros::FfiMarshaler)]
#[ffi(strategy = "json")]
pub struct DeviceInfoWithProfile {
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
// #[ffi_export] // TODO: Uncomment when exports_*.rs files are removed (task 20)
pub fn list_devices() -> FfiResult<Vec<DeviceInfoWithProfile>> {
    let devices = drivers::list_keyboards()
        .map_err(|e| FfiError::internal(format!("Failed to list keyboards: {}", e)))?;

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

    Ok(enriched)
}

/// Select a device by path for use when starting the engine.
///
/// # Arguments
/// * `ctx` - FFI context containing device state
/// * `path` - Device path to select
///
/// # Returns
/// * `Ok(())` on success
/// * `Err(FfiError)` if device path does not exist
// #[ffi_export] // TODO: Uncomment when exports_*.rs files are removed (task 20)
pub fn select_device(ctx: &mut FfiContext, path: &str) -> FfiResult<()> {
    let device_path = PathBuf::from(path);

    // On Linux, device paths are files in /dev/input/, so we can check if they exist.
    // On Windows, device paths are interface strings (\\?\HID...) which are not
    // valid filesystem paths for std::fs::metadata, so exists() fails.
    #[cfg(not(target_os = "windows"))]
    if !device_path.exists() {
        return Err(FfiError::not_found(format!(
            "Device path does not exist: {}",
            path
        )));
    }

    if let Some(mut state_guard) = ctx.get_domain_mut::<DeviceState>(DeviceFfi::DOMAIN) {
        if let Some(state) = state_guard.downcast_mut::<DeviceState>() {
            state.selected_device = Some(device_path);
            return Ok(());
        }
    }

    Err(FfiError::internal("Failed to access device state"))
}

/// Return canonical key registry as `ok:<json>` (or `error:<message>`).
// #[ffi_export] // TODO: Uncomment when exports_*.rs files are removed (task 20)
pub fn list_keys() -> FfiResult<Vec<serde_json::Value>> {
    let keys = key_definitions();
    let json_keys: Result<Vec<serde_json::Value>, _> =
        keys.iter().map(serde_json::to_value).collect();

    json_keys.map_err(|e| FfiError::internal(format!("Failed to serialize key definitions: {}", e)))
}

/// Get device profile for a specific device by vendor and product ID.
///
/// Returns the complete device profile including keymap, layout configuration,
/// and discovery metadata.
///
/// # Arguments
/// * `vendor_id` - USB vendor ID
/// * `product_id` - USB product ID
///
/// # Returns
/// * `Ok(DeviceProfile)` if profile exists
/// * `Err(FfiError)` if profile not found or cannot be loaded
// #[ffi_export] // TODO: Uncomment when exports_*.rs files are removed (task 20)
pub fn get_device_profile(
    vendor_id: u16,
    product_id: u16,
) -> FfiResult<crate::discovery::types::DeviceProfile> {
    use crate::discovery::storage;
    use crate::discovery::types::DeviceId;

    let device_id = DeviceId::new(vendor_id, product_id);
    let profile = storage::read_profile(device_id).map_err(|e| {
        FfiError::not_found(format!("Device profile not found for {}: {}", device_id, e))
    })?;

    Ok(profile)
}

/// Check if a device profile exists for given vendor and product ID.
///
/// # Arguments
/// * `vendor_id` - USB vendor ID
/// * `product_id` - USB product ID
///
/// # Returns
/// * `Ok(true)` if profile exists
/// * `Ok(false)` if profile does not exist
// #[ffi_export] // TODO: Uncomment when exports_*.rs files are removed (task 20)
pub fn has_device_profile(vendor_id: u16, product_id: u16) -> FfiResult<bool> {
    use crate::discovery::storage::profile_path;
    use crate::discovery::types::DeviceId;

    let device_id = DeviceId::new(vendor_id, product_id);
    Ok(profile_path(device_id).exists())
}

/// Save a device profile to disk.
///
/// # Arguments
/// * `profile_json` - JSON representation of the DeviceProfile
///
/// # Returns
/// * `Ok(())` on success
/// * `Err(FfiError)` if JSON is invalid or write fails
// #[ffi_export] // TODO: Uncomment when exports_*.rs files are removed (task 20)
pub fn save_device_profile(profile_json: &str) -> FfiResult<()> {
    let profile: crate::discovery::types::DeviceProfile = serde_json::from_str(profile_json)
        .map_err(|e| FfiError::invalid_input(format!("invalid profile json: {}", e)))?;

    crate::discovery::storage::write_profile(&profile)
        .map_err(|e| FfiError::internal(format!("failed to write profile: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use tempfile::NamedTempFile;

    fn setup_context() -> FfiContext {
        let mut ctx = FfiContext::new();
        DeviceFfi::init(&mut ctx).expect("init should succeed");
        ctx
    }

    #[test]
    #[serial]
    fn select_device_with_valid_path() {
        let mut ctx = setup_context();

        // Create a temp file to use as a valid path
        let temp = NamedTempFile::new().unwrap();
        let path = temp.path().to_str().unwrap();

        let result = select_device(&mut ctx, path);
        assert!(result.is_ok());

        let selected = DeviceFfi::get_selected_device(&ctx);
        assert_eq!(selected, Some(temp.path().to_path_buf()));
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn select_device_rejects_nonexistent_path() {
        let mut ctx = setup_context();
        let result = select_device(&mut ctx, "/nonexistent/device/path");
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("does not exist"));
    }

    #[test]
    fn list_devices_returns_json_array() {
        let result = list_devices();
        assert!(result.is_ok());
        // Device list may be empty on systems without keyboards
        // Just verify it returns successfully
    }

    #[test]
    fn list_keys_returns_registry_objects() {
        let result = list_keys();
        assert!(result.is_ok());
        let keys = result.unwrap();
        assert!(!keys.is_empty());

        // Verify at least the 'A' key exists
        let has_a = keys.iter().any(|k| {
            k.get("name")
                .and_then(|v| v.as_str())
                .map(|s| s == "A")
                .unwrap_or(false)
        });
        assert!(has_a, "Key registry should contain 'A'");
    }
}
