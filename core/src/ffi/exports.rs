//! Core C-ABI exports for FFI.
//!
//! This module provides core init/common C-compatible functions for FFI integration.
//! Unsafe code is required for FFI interoperability.
#![allow(unsafe_code)]

use crate::config;
use crate::config::models::{
    DeviceInstanceId, DeviceSlots, HardwareProfile, Keymap, ProfileSlot, RuntimeConfig,
    VirtualLayout,
};
use crate::config::{ConfigManager, StorageError};
use crate::definitions::DeviceDefinitionLibrary;
use crate::ffi::domains::discovery::global_event_registry;
use crate::ffi::domains::engine::global_event_registry as engine_event_registry;
use crate::ffi::error::{serialize_ffi_result, FfiError, FfiResult};
use crate::ffi::events::{EventCallback, EventType};
use crate::ffi::runtime::{
    clear_revolutionary_runtime, set_revolutionary_runtime, RevolutionaryRuntime,
};
use crate::registry::ProfileRegistry;
use serde::Serialize;
use std::ffi::{c_char, CStr, CString};
use std::path::PathBuf;
use std::sync::Arc;

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

/// Get the ABI protocol version.
///
/// This integer is incremented whenever the FFI data structures or contract changes.
/// The UI should check this on startup to ensure it is compatible with the loaded core library.
#[no_mangle]
pub extern "C" fn keyrx_protocol_version() -> u32 {
    // Increment this whenever FFI structs/signatures change
    1
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

// ---------------------------------------------------------------------------
// Revolutionary runtime lifecycle (for FFI consumers like Flutter)
// ---------------------------------------------------------------------------

fn default_device_definitions() -> Arc<DeviceDefinitionLibrary> {
    let mut library = DeviceDefinitionLibrary::new();
    let mut loaded = 0usize;

    // Preferred search paths (in order):
    // 1) cwd/device_definitions
    // 2) config dir: ~/.config/keyrx/device_definitions
    let mut paths: Vec<PathBuf> = vec![];
    if let Ok(cwd) = std::env::current_dir() {
        paths.push(cwd.join("device_definitions"));
    }
    paths.push(config::config_dir().join("device_definitions"));

    for path in paths {
        if path.exists() {
            match library.load_from_directory(&path) {
                Ok(count) => {
                    loaded += count;
                    tracing::info!(
                        service = "keyrx",
                        component = "ffi_exports",
                        event = "device_definitions_loaded",
                        path = %path.display(),
                        count,
                        "Loaded device definitions for FFI runtime"
                    );
                }
                Err(err) => {
                    tracing::warn!(
                        service = "keyrx",
                        component = "ffi_exports",
                        event = "device_definitions_load_failed",
                        path = %path.display(),
                        error = %err,
                        "Failed to load device definitions for FFI runtime"
                    );
                }
            }
        }
    }

    if loaded == 0 {
        tracing::warn!(
            service = "keyrx",
            component = "ffi_exports",
            event = "device_definitions_empty",
            "No device definitions loaded for FFI runtime; definition calls may return NOT_FOUND"
        );
    }

    Arc::new(library)
}

fn init_revolutionary_runtime() -> i32 {
    // Create registries using default locations.
    let (device_registry, _rx) = crate::registry::DeviceRegistry::new();
    let profile_registry = Arc::new(ProfileRegistry::new());
    let device_definitions = default_device_definitions();

    match set_revolutionary_runtime(RevolutionaryRuntime::new(
        device_registry,
        profile_registry,
        device_definitions,
    )) {
        Ok(_) => {
            tracing::info!(
                service = "keyrx",
                component = "ffi_exports",
                event = "revolutionary_runtime_initialized",
                "Revolutionary runtime initialized for FFI consumers"
            );
            0
        }
        Err(err) => {
            tracing::error!(
                service = "keyrx",
                component = "ffi_exports",
                event = "revolutionary_runtime_init_failed",
                error = %err,
                "Failed to initialize revolutionary runtime for FFI consumers"
            );
            -1
        }
    }
}

fn shutdown_revolutionary_runtime() -> i32 {
    match clear_revolutionary_runtime() {
        Ok(_) => {
            tracing::info!(
                service = "keyrx",
                component = "ffi_exports",
                event = "revolutionary_runtime_cleared",
                "Revolutionary runtime cleared for FFI consumers"
            );
            0
        }
        Err(err) => {
            tracing::error!(
                service = "keyrx",
                component = "ffi_exports",
                event = "revolutionary_runtime_clear_failed",
                error = %err,
                "Failed to clear revolutionary runtime for FFI consumers"
            );
            -1
        }
    }
}

/// Initialize the revolutionary runtime for FFI consumers (e.g., Flutter).
///
/// Returns 0 on success, negative on failure.
#[no_mangle]
pub extern "C" fn keyrx_revolutionary_runtime_init() -> i32 {
    std::panic::catch_unwind(init_revolutionary_runtime).unwrap_or(-2)
}

/// Shutdown/clear the revolutionary runtime.
///
/// Returns 0 on success, negative on failure.
#[no_mangle]
pub extern "C" fn keyrx_revolutionary_runtime_shutdown() -> i32 {
    std::panic::catch_unwind(shutdown_revolutionary_runtime).unwrap_or(-2)
}

/// Register a unified event callback.
///
/// This is the new unified API for registering callbacks across all domains.
/// It replaces domain-specific callback registration functions.
///
/// # Event Types (by integer code)
/// - 0: DiscoveryProgress
/// - 1: DiscoveryDuplicate
/// - 2: DiscoverySummary
/// - 3: EngineState
/// - 4: ValidationProgress
/// - 5: ValidationResult
/// - 6: DeviceConnected
/// - 7: DeviceDisconnected
/// - 8: TestProgress
/// - 9: TestResult
/// - 10: AnalysisProgress
/// - 11: AnalysisResult
/// - 12: DiagnosticsLog
/// - 13: DiagnosticsMetric
/// - 14: RecordingStarted
/// - 15: RecordingStopped
///
/// # Arguments
/// * `event_type_code` - Integer code for the event type (see list above)
/// * `callback` - Optional callback function. Pass NULL to unregister.
///
/// # Returns
/// - 0: Success
/// - -1: Invalid event type code
///
/// # Safety
/// The callback function must be valid for the lifetime of the registration.
#[no_mangle]
pub extern "C" fn keyrx_register_event_callback(
    event_type_code: i32,
    callback: Option<EventCallback>,
) -> i32 {
    let event_type = match event_type_code {
        0 => EventType::DiscoveryProgress,
        1 => EventType::DiscoveryDuplicate,
        2 => EventType::DiscoverySummary,
        3 => EventType::EngineState,
        4 => EventType::ValidationProgress,
        5 => EventType::ValidationResult,
        6 => EventType::DeviceConnected,
        7 => EventType::DeviceDisconnected,
        8 => EventType::TestProgress,
        9 => EventType::TestResult,
        10 => EventType::AnalysisProgress,
        11 => EventType::AnalysisResult,
        12 => EventType::DiagnosticsLog,
        13 => EventType::DiagnosticsMetric,
        14 => EventType::RecordingStarted,
        15 => EventType::RecordingStopped,
        _ => {
            tracing::warn!(
                service = "keyrx",
                component = "ffi_exports",
                event = "invalid_event_type",
                code = event_type_code,
                "Invalid event type code provided to keyrx_register_event_callback"
            );
            return -1;
        }
    };

    let registry = match event_type {
        EventType::EngineState => engine_event_registry(),
        _ => global_event_registry(),
    };

    registry.register(event_type, callback);

    // Refresh discovery sink if registering discovery events
    if matches!(
        event_type,
        EventType::DiscoveryProgress | EventType::DiscoveryDuplicate | EventType::DiscoverySummary
    ) {
        crate::ffi::domains::discovery::refresh_discovery_sink();
    }

    0
}

// ---------------------------------------------------------------------------
// Configuration management (layouts, hardware profiles, keymaps, runtime)
// ---------------------------------------------------------------------------

fn ffi_json<T: Serialize>(result: FfiResult<T>) -> *mut c_char {
    let payload = serialize_ffi_result(&result).unwrap_or_else(|e| {
        format!("error:{{\"code\":\"SERIALIZATION_FAILED\",\"message\":\"{e}\"}}")
    });
    CString::new(payload)
        .map(CString::into_raw)
        .unwrap_or(std::ptr::null_mut())
}

fn storage_error(err: StorageError) -> FfiError {
    match err {
        StorageError::CreateDir(path, source) => FfiError::new(
            "STORAGE_ERROR",
            format!("failed to create directory {}: {}", path.display(), source),
        ),
        StorageError::ReadDir(path, source) => FfiError::new(
            "STORAGE_ERROR",
            format!("failed to read directory {}: {}", path.display(), source),
        ),
        StorageError::ReadFile(path, source) => FfiError::new(
            "STORAGE_ERROR",
            format!("failed to read file {}: {}", path.display(), source),
        ),
        StorageError::WriteFile(path, source) => FfiError::new(
            "STORAGE_ERROR",
            format!("failed to write file {}: {}", path.display(), source),
        ),
        StorageError::Parse(path, source) => FfiError::deserialization_failed(format!(
            "failed to parse JSON {}: {}",
            path.display(),
            source
        )),
    }
}

unsafe fn parse_c_string(ptr: *const c_char, name: &str) -> FfiResult<String> {
    if ptr.is_null() {
        return Err(FfiError::null_pointer(name));
    }

    CStr::from_ptr(ptr)
        .to_str()
        .map(|s| s.to_owned())
        .map_err(|_| FfiError::invalid_utf8(name))
}

fn config_manager() -> ConfigManager {
    ConfigManager::default()
}

fn update_runtime_config<F, T>(mutate: F) -> FfiResult<T>
where
    F: FnOnce(&mut RuntimeConfig) -> FfiResult<T>,
{
    let manager = config_manager();
    let mut runtime = manager.load_runtime_config().map_err(storage_error)?;

    let result = mutate(&mut runtime)?;

    manager
        .save_runtime_config(&runtime)
        .map_err(storage_error)?;

    Ok(result)
}

fn reorder_slots(slots: &mut [ProfileSlot]) {
    slots.sort_by(|a, b| b.priority.cmp(&a.priority));
}

#[no_mangle]
pub extern "C" fn keyrx_config_list_virtual_layouts() -> *mut c_char {
    std::panic::catch_unwind(|| {
        let result: FfiResult<Vec<VirtualLayout>> = config_manager()
            .load_virtual_layouts()
            .map(|m| m.into_values().collect())
            .map_err(storage_error);
        ffi_json(result)
    })
    .unwrap_or_else(|_| ffi_json::<()>(Err(FfiError::internal("panic in list_virtual_layouts"))))
}

#[no_mangle]
/// Save or update a virtual layout definition.
///
/// # Safety
/// `layout_json` must be a valid, non-null, UTF-8 C string containing a serialized `VirtualLayout`.
pub unsafe extern "C" fn keyrx_config_save_virtual_layout(
    layout_json: *const c_char,
) -> *mut c_char {
    std::panic::catch_unwind(|| {
        let json = match parse_c_string(layout_json, "layout_json") {
            Ok(s) => s,
            Err(err) => return ffi_json::<()>(Err(err)),
        };

        let layout: VirtualLayout = match serde_json::from_str(&json) {
            Ok(v) => v,
            Err(err) => {
                return ffi_json::<()>(Err(FfiError::deserialization_failed(err.to_string())))
            }
        };

        let result: FfiResult<VirtualLayout> = config_manager()
            .save_virtual_layout(&layout)
            .map_err(storage_error)
            .map(|_| layout);
        ffi_json(result)
    })
    .unwrap_or_else(|_| ffi_json::<()>(Err(FfiError::internal("panic in save_virtual_layout"))))
}

#[no_mangle]
/// Delete a persisted virtual layout by id.
///
/// # Safety
/// `id` must be a valid, non-null, UTF-8 C string.
pub unsafe extern "C" fn keyrx_config_delete_virtual_layout(id: *const c_char) -> *mut c_char {
    std::panic::catch_unwind(|| {
        let id = match parse_c_string(id, "layout_id") {
            Ok(s) => s,
            Err(err) => return ffi_json::<()>(Err(err)),
        };

        let result: FfiResult<()> = config_manager()
            .delete_virtual_layout(&id)
            .map_err(storage_error);
        ffi_json(result)
    })
    .unwrap_or_else(|_| ffi_json::<()>(Err(FfiError::internal("panic in delete_virtual_layout"))))
}

#[no_mangle]
pub extern "C" fn keyrx_config_list_hardware_profiles() -> *mut c_char {
    std::panic::catch_unwind(|| {
        let result: FfiResult<Vec<HardwareProfile>> = config_manager()
            .load_hardware_profiles()
            .map(|m| m.into_values().collect())
            .map_err(storage_error);
        ffi_json(result)
    })
    .unwrap_or_else(|_| ffi_json::<()>(Err(FfiError::internal("panic in list_hardware_profiles"))))
}

#[no_mangle]
/// Save or update a hardware wiring profile.
///
/// # Safety
/// `profile_json` must be a valid, non-null, UTF-8 C string containing a serialized `HardwareProfile`.
pub unsafe extern "C" fn keyrx_config_save_hardware_profile(
    profile_json: *const c_char,
) -> *mut c_char {
    std::panic::catch_unwind(|| {
        let json = match parse_c_string(profile_json, "hardware_profile_json") {
            Ok(s) => s,
            Err(err) => return ffi_json::<()>(Err(err)),
        };

        let profile: HardwareProfile = match serde_json::from_str(&json) {
            Ok(v) => v,
            Err(err) => {
                return ffi_json::<()>(Err(FfiError::deserialization_failed(err.to_string())))
            }
        };

        let result: FfiResult<HardwareProfile> = config_manager()
            .save_hardware_profile(&profile)
            .map_err(storage_error)
            .map(|_| profile);
        ffi_json(result)
    })
    .unwrap_or_else(|_| ffi_json::<()>(Err(FfiError::internal("panic in save_hardware_profile"))))
}

#[no_mangle]
/// Delete a hardware profile by id.
///
/// # Safety
/// `id` must be a valid, non-null, UTF-8 C string.
pub unsafe extern "C" fn keyrx_config_delete_hardware_profile(id: *const c_char) -> *mut c_char {
    std::panic::catch_unwind(|| {
        let id = match parse_c_string(id, "hardware_profile_id") {
            Ok(s) => s,
            Err(err) => return ffi_json::<()>(Err(err)),
        };

        let result: FfiResult<()> = config_manager()
            .delete_hardware_profile(&id)
            .map_err(storage_error);
        ffi_json(result)
    })
    .unwrap_or_else(|_| ffi_json::<()>(Err(FfiError::internal("panic in delete_hardware_profile"))))
}

#[no_mangle]
pub extern "C" fn keyrx_config_list_keymaps() -> *mut c_char {
    std::panic::catch_unwind(|| {
        let result: FfiResult<Vec<Keymap>> = config_manager()
            .load_keymaps()
            .map(|m| m.into_values().collect())
            .map_err(storage_error);
        ffi_json(result)
    })
    .unwrap_or_else(|_| ffi_json::<()>(Err(FfiError::internal("panic in list_keymaps"))))
}

#[no_mangle]
/// Save or update a keymap definition.
///
/// # Safety
/// `keymap_json` must be a valid, non-null, UTF-8 C string containing a serialized `Keymap`.
pub unsafe extern "C" fn keyrx_config_save_keymap(keymap_json: *const c_char) -> *mut c_char {
    std::panic::catch_unwind(|| {
        let json = match parse_c_string(keymap_json, "keymap_json") {
            Ok(s) => s,
            Err(err) => return ffi_json::<()>(Err(err)),
        };

        let keymap: Keymap = match serde_json::from_str(&json) {
            Ok(v) => v,
            Err(err) => {
                return ffi_json::<()>(Err(FfiError::deserialization_failed(err.to_string())))
            }
        };

        let result: FfiResult<Keymap> = config_manager()
            .save_keymap(&keymap)
            .map_err(storage_error)
            .map(|_| keymap);
        ffi_json(result)
    })
    .unwrap_or_else(|_| ffi_json::<()>(Err(FfiError::internal("panic in save_keymap"))))
}

#[no_mangle]
/// Delete a keymap by id.
///
/// # Safety
/// `id` must be a valid, non-null, UTF-8 C string.
pub unsafe extern "C" fn keyrx_config_delete_keymap(id: *const c_char) -> *mut c_char {
    std::panic::catch_unwind(|| {
        let id = match parse_c_string(id, "keymap_id") {
            Ok(s) => s,
            Err(err) => return ffi_json::<()>(Err(err)),
        };

        let result: FfiResult<()> = config_manager().delete_keymap(&id).map_err(storage_error);
        ffi_json(result)
    })
    .unwrap_or_else(|_| ffi_json::<()>(Err(FfiError::internal("panic in delete_keymap"))))
}

fn parse_device_json(json: &str) -> FfiResult<DeviceInstanceId> {
    serde_json::from_str(json)
        .map_err(|err| FfiError::deserialization_failed(format!("device: {err}")))
}

fn parse_slot_json(json: &str) -> FfiResult<ProfileSlot> {
    serde_json::from_str(json)
        .map_err(|err| FfiError::deserialization_failed(format!("slot: {err}")))
}

#[no_mangle]
pub extern "C" fn keyrx_runtime_get_config() -> *mut c_char {
    std::panic::catch_unwind(|| {
        let result: FfiResult<RuntimeConfig> = config_manager()
            .load_runtime_config()
            .map_err(storage_error);
        ffi_json(result)
    })
    .unwrap_or_else(|_| ffi_json::<()>(Err(FfiError::internal("panic in get_runtime_config"))))
}

#[no_mangle]
/// Add or upsert a runtime profile slot for a device and persist the configuration.
///
/// # Safety
/// `device_json` and `slot_json` must be valid, non-null, UTF-8 C strings containing `DeviceInstanceId`
/// and `ProfileSlot` JSON payloads.
pub unsafe extern "C" fn keyrx_runtime_add_slot(
    device_json: *const c_char,
    slot_json: *const c_char,
) -> *mut c_char {
    std::panic::catch_unwind(|| {
        let device_json = match parse_c_string(device_json, "device_json") {
            Ok(s) => s,
            Err(err) => return ffi_json::<()>(Err(err)),
        };
        let slot_json = match parse_c_string(slot_json, "slot_json") {
            Ok(s) => s,
            Err(err) => return ffi_json::<()>(Err(err)),
        };

        let device = match parse_device_json(&device_json) {
            Ok(d) => d,
            Err(err) => return ffi_json::<()>(Err(err)),
        };
        let slot = match parse_slot_json(&slot_json) {
            Ok(s) => s,
            Err(err) => return ffi_json::<()>(Err(err)),
        };

        let result: FfiResult<RuntimeConfig> = update_runtime_config(|runtime| {
            let device_index = runtime
                .devices
                .iter()
                .position(|d| d.device == device)
                .unwrap_or_else(|| {
                    runtime.devices.push(DeviceSlots {
                        device: device.clone(),
                        slots: vec![],
                    });
                    runtime.devices.len() - 1
                });

            let slots = {
                let device_slots = runtime
                    .devices
                    .get_mut(device_index)
                    .ok_or_else(|| FfiError::internal("device slot missing after insertion"))?;
                &mut device_slots.slots
            };
            // Replace existing slot with same id to keep updates idempotent.
            if let Some(existing_idx) = slots.iter().position(|s| s.id == slot.id) {
                slots[existing_idx] = slot.clone();
            } else {
                slots.push(slot.clone());
            }
            reorder_slots(slots);
            Ok(runtime.clone())
        });
        ffi_json(result)
    })
    .unwrap_or_else(|_| ffi_json::<()>(Err(FfiError::internal("panic in runtime_add_slot"))))
}

#[no_mangle]
/// Remove a runtime slot for a device.
///
/// # Safety
/// `device_json` and `slot_id` must be valid, non-null, UTF-8 C strings.
pub unsafe extern "C" fn keyrx_runtime_remove_slot(
    device_json: *const c_char,
    slot_id: *const c_char,
) -> *mut c_char {
    std::panic::catch_unwind(|| {
        let device_json = match parse_c_string(device_json, "device_json") {
            Ok(s) => s,
            Err(err) => return ffi_json::<()>(Err(err)),
        };
        let slot_id = match parse_c_string(slot_id, "slot_id") {
            Ok(s) => s,
            Err(err) => return ffi_json::<()>(Err(err)),
        };

        let device = match parse_device_json(&device_json) {
            Ok(d) => d,
            Err(err) => return ffi_json::<()>(Err(err)),
        };

        let result: FfiResult<RuntimeConfig> = update_runtime_config(|runtime| {
            let Some(slots) = runtime
                .devices
                .iter_mut()
                .find(|d| d.device == device)
                .map(|d| &mut d.slots)
            else {
                return Err(FfiError::not_found("device slots"));
            };

            let before_len = slots.len();
            slots.retain(|slot| slot.id != slot_id);

            if before_len == slots.len() {
                return Err(FfiError::not_found(format!(
                    "slot '{}' for device",
                    slot_id
                )));
            }

            Ok(runtime.clone())
        });
        ffi_json(result)
    })
    .unwrap_or_else(|_| ffi_json::<()>(Err(FfiError::internal("panic in runtime_remove_slot"))))
}

#[no_mangle]
/// Update slot priority and re-order the runtime configuration for a device.
///
/// # Safety
/// `device_json` and `slot_id` must be valid, non-null, UTF-8 C strings.
pub unsafe extern "C" fn keyrx_runtime_reorder_slot(
    device_json: *const c_char,
    slot_id: *const c_char,
    priority: u32,
) -> *mut c_char {
    std::panic::catch_unwind(|| {
        let device_json = match parse_c_string(device_json, "device_json") {
            Ok(s) => s,
            Err(err) => return ffi_json::<()>(Err(err)),
        };
        let slot_id = match parse_c_string(slot_id, "slot_id") {
            Ok(s) => s,
            Err(err) => return ffi_json::<()>(Err(err)),
        };

        let device = match parse_device_json(&device_json) {
            Ok(d) => d,
            Err(err) => return ffi_json::<()>(Err(err)),
        };

        let result: FfiResult<RuntimeConfig> = update_runtime_config(|runtime| {
            let Some(slots) = runtime
                .devices
                .iter_mut()
                .find(|d| d.device == device)
                .map(|d| &mut d.slots)
            else {
                return Err(FfiError::not_found("device slots"));
            };

            let Some(slot) = slots.iter_mut().find(|s| s.id == slot_id) else {
                return Err(FfiError::not_found("slot"));
            };

            slot.priority = priority;
            reorder_slots(slots);
            Ok(runtime.clone())
        });
        ffi_json(result)
    })
    .unwrap_or_else(|_| ffi_json::<()>(Err(FfiError::internal("panic in runtime_reorder_slot"))))
}

#[no_mangle]
/// Toggle a runtime slot active flag for a device.
///
/// # Safety
/// `device_json` and `slot_id` must be valid, non-null, UTF-8 C strings.
pub unsafe extern "C" fn keyrx_runtime_set_slot_active(
    device_json: *const c_char,
    slot_id: *const c_char,
    active: bool,
) -> *mut c_char {
    std::panic::catch_unwind(|| {
        let device_json = match parse_c_string(device_json, "device_json") {
            Ok(s) => s,
            Err(err) => return ffi_json::<()>(Err(err)),
        };
        let slot_id = match parse_c_string(slot_id, "slot_id") {
            Ok(s) => s,
            Err(err) => return ffi_json::<()>(Err(err)),
        };

        let device = match parse_device_json(&device_json) {
            Ok(d) => d,
            Err(err) => return ffi_json::<()>(Err(err)),
        };

        let result: FfiResult<RuntimeConfig> = update_runtime_config(|runtime| {
            let Some(slots) = runtime
                .devices
                .iter_mut()
                .find(|d| d.device == device)
                .map(|d| &mut d.slots)
            else {
                return Err(FfiError::not_found("device slots"));
            };

            let Some(slot) = slots.iter_mut().find(|s| s.id == slot_id) else {
                return Err(FfiError::not_found("slot"));
            };

            slot.active = active;
            reorder_slots(slots);
            Ok(runtime.clone())
        });
        ffi_json(result)
    })
    .unwrap_or_else(|_| ffi_json::<()>(Err(FfiError::internal("panic in runtime_set_slot_active"))))
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

    #[test]
    fn register_event_callback_accepts_valid_codes() {
        // Test valid event type codes
        for code in 0..=15 {
            assert_eq!(keyrx_register_event_callback(code, None), 0);
        }
    }

    #[test]
    fn register_event_callback_rejects_invalid_codes() {
        // Test invalid event type codes
        assert_eq!(keyrx_register_event_callback(-1, None), -1);
        assert_eq!(keyrx_register_event_callback(16, None), -1);
        assert_eq!(keyrx_register_event_callback(100, None), -1);
    }

    #[test]
    fn register_event_callback_registers_callback() {
        unsafe extern "C" fn test_cb(_ptr: *const u8, _len: usize) {}

        // Clear registry first
        global_event_registry().clear();

        // Register callback
        assert_eq!(keyrx_register_event_callback(0, Some(test_cb)), 0);
        assert!(global_event_registry().is_registered(EventType::DiscoveryProgress));

        // Unregister
        assert_eq!(keyrx_register_event_callback(0, None), 0);
        assert!(!global_event_registry().is_registered(EventType::DiscoveryProgress));
    }
}
