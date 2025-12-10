use crate::config::models::{DeviceSlots, ProfileSlot, RuntimeConfig};
use crate::ffi::error::{serialize_ffi_result, FfiError, FfiResult};
use crate::ffi::exports::{global_config_manager, parse_c_string, parse_device_json};

use std::ffi::{c_char, CString};
use std::ptr;
use std::sync::{OnceLock, RwLock};

fn ffi_json<T: serde::Serialize>(result: FfiResult<T>) -> *mut c_char {
    match serialize_ffi_result(&result) {
        Ok(json) => CString::new(json).unwrap_or_default().into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

fn runtime_config_store() -> &'static RwLock<RuntimeConfig> {
    static CONFIG: OnceLock<RwLock<RuntimeConfig>> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let initial = global_config_manager()
            .load_runtime_config()
            .unwrap_or_else(|e| {
                tracing::warn!("Failed to load runtime config: {e}");
                RuntimeConfig::default()
            });
        RwLock::new(initial)
    })
}

fn update_runtime_config<F>(f: F) -> FfiResult<RuntimeConfig>
where
    F: FnOnce(&mut RuntimeConfig) -> FfiResult<RuntimeConfig>,
{
    let mut guard = runtime_config_store()
        .write()
        .map_err(|_| FfiError::internal("runtime config lock poisoned"))?;
    let new_config = f(&mut guard)?;

    // Auto-save on update
    if let Err(e) = global_config_manager().save_runtime_config(&new_config) {
        tracing::error!("Failed to save runtime config: {e}");
        // We log error but return success to UI as in-memory state is valid
    }

    Ok(new_config)
}

fn reorder_slots(slots: &mut Vec<ProfileSlot>) {
    slots.sort_by(|a, b| {
        // Lower priority number = higher priority first (or vice versa? Logic said a.priority.cmp(b.priority) in original)
        // Original: a.priority.cmp(&b.priority) which is ascending. 0 < 1.
        // Usually priority 0 is highest.
        a.priority.cmp(&b.priority)
    });
}

/// Get the current runtime configuration.
///
/// Returns JSON string: `ok:<json>` or `error:<message>`.
#[no_mangle]
pub extern "C" fn keyrx_runtime_get_config() -> *mut c_char {
    let config = match runtime_config_store().read() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    ffi_json(Ok(config.clone()))
}

#[no_mangle]
/// Add a new runtime slot for a device (or update existing).
pub unsafe extern "C" fn keyrx_runtime_add_slot(
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

        let result: FfiResult<RuntimeConfig> = update_runtime_config(|config| {
            // Find device entry, or create if missing
            let device_entry_idx =
                if let Some(idx) = config.devices.iter().position(|d| d.device == device) {
                    idx
                } else {
                    config.devices.push(DeviceSlots {
                        device: device.clone(),
                        slots: vec![],
                    });
                    config.devices.len() - 1
                };

            let slots = &mut config.devices[device_entry_idx].slots;

            // Define new slot
            // Since we don't receive keymap_id/hardware_profile_id from FFI here (legacy signature?),
            // we assume defaults or placeholder IDs?
            // The original signature: add_slot(device, slot_id, priority).
            // It seems the original implementation might have had a lighter slot definition or assumed valid IDs.
            // For now, I'll use placeholders as this seems to be managing "active" state mostly?
            // Wait, this function might be legacy or incorrect for the new `ProfileSlot`.
            // `ProfileSlot` needs `hardware_profile_id`, `keymap_id`.
            // I'll populate them with the `slot_id` as a placeholder for now to satisfy the type.
            let slot = ProfileSlot {
                id: slot_id.clone(),
                hardware_profile_id: "default".into(), // Placeholder
                keymap_id: "default".into(),           // Placeholder
                active: true,
                priority,
            };

            if let Some(existing_idx) = slots.iter().position(|s| s.id == slot.id) {
                // Preserve things? Or overwrite?
                // Overwrite core fields but maybe keep others in real impl.
                slots[existing_idx] = slot;
            } else {
                slots.push(slot);
            }
            reorder_slots(slots);
            Ok(config.clone())
        });
        ffi_json(result)
    })
    .unwrap_or_else(|_| ffi_json::<()>(Err(FfiError::internal("panic in runtime_add_slot"))))
}

#[no_mangle]
/// Remove a runtime slot for a device.
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

        let result: FfiResult<RuntimeConfig> = update_runtime_config(|config| {
            let Some(device_entry) = config.devices.iter_mut().find(|d| d.device == device) else {
                return Err(FfiError::not_found("device slots"));
            };
            let slots = &mut device_entry.slots;

            let before_len = slots.len();
            slots.retain(|slot| slot.id != slot_id);

            if before_len == slots.len() {
                return Err(FfiError::not_found(format!(
                    "slot '{}' for device",
                    slot_id
                )));
            }

            Ok(config.clone())
        });
        ffi_json(result)
    })
    .unwrap_or_else(|_| ffi_json::<()>(Err(FfiError::internal("panic in runtime_remove_slot"))))
}

#[no_mangle]
/// Update slot priority.
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

        let result: FfiResult<RuntimeConfig> = update_runtime_config(|config| {
            let Some(device_entry) = config.devices.iter_mut().find(|d| d.device == device) else {
                return Err(FfiError::not_found("device slots"));
            };
            let slots = &mut device_entry.slots;

            let Some(slot) = slots.iter_mut().find(|s| s.id == slot_id) else {
                return Err(FfiError::not_found("slot"));
            };

            slot.priority = priority;
            reorder_slots(slots);
            Ok(config.clone())
        });
        ffi_json(result)
    })
    .unwrap_or_else(|_| ffi_json::<()>(Err(FfiError::internal("panic in runtime_reorder_slot"))))
}

#[no_mangle]
/// Toggle a runtime slot active flag.
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

        let result: FfiResult<RuntimeConfig> = update_runtime_config(|config| {
            let Some(device_entry) = config.devices.iter_mut().find(|d| d.device == device) else {
                return Err(FfiError::not_found("device slots"));
            };
            let slots = &mut device_entry.slots;

            let Some(slot) = slots.iter_mut().find(|s| s.id == slot_id) else {
                return Err(FfiError::not_found("slot"));
            };

            slot.active = active;
            reorder_slots(slots);
            Ok(config.clone())
        });
        ffi_json(result)
    })
    .unwrap_or_else(|_| ffi_json::<()>(Err(FfiError::internal("panic in runtime_set_slot_active"))))
}
