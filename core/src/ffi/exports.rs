use crate::config::models::DeviceInstanceId;
use crate::config::models::{HardwareProfile, Keymap, VirtualLayout};
use crate::definitions::DeviceDefinitionLibrary;
use crate::ffi::domains::engine::global_event_registry;
use crate::ffi::error::{serialize_ffi_result, FfiError, FfiResult};
use crate::ffi::events::EventType;
use crate::ffi::runtime::{
    clear_revolutionary_runtime, set_revolutionary_runtime, RevolutionaryRuntime,
};
use crate::registry::{DeviceRegistry, ProfileRegistry};
use std::ffi::{c_char, CStr, CString};
use std::ptr;
use std::sync::{Arc, Mutex};

// Re-export specific helpers for other FFI modules
pub(crate) fn parse_c_string(ptr: *const c_char, param_name: &str) -> FfiResult<String> {
    if ptr.is_null() {
        return Err(FfiError::null_pointer(param_name));
    }
    unsafe {
        CStr::from_ptr(ptr)
            .to_str()
            .map(|s| s.to_string())
            .map_err(|_| FfiError::invalid_utf8(param_name))
    }
}

pub(crate) fn parse_device_json(json_str: &str) -> FfiResult<DeviceInstanceId> {
    serde_json::from_str(json_str)
        .map_err(|e| FfiError::invalid_input(format!("Invalid device JSON: {}", e)))
}

fn init_revolutionary_runtime() {
    // Initialize with default/empty components for now.
    // In a real app complexity, this would load config from disk.
    let (registry, _rx) = DeviceRegistry::new();
    let profiles = Arc::new(ProfileRegistry::new());
    let definitions = Arc::new(DeviceDefinitionLibrary::new());
    let rhai = crate::scripting::RhaiRuntime::new().expect("failed to initialize rhai runtime");
    let rhai_runtime = Arc::new(Mutex::new(rhai));

    let runtime = RevolutionaryRuntime::new(registry, profiles, definitions, rhai_runtime);
    let _ = set_revolutionary_runtime(runtime);
}

#[no_mangle]
pub unsafe extern "C" fn keyrx_init() -> i32 {
    init_revolutionary_runtime();
    0
}

#[no_mangle]
pub unsafe extern "C" fn keyrx_version() -> *mut c_char {
    let version = env!("CARGO_PKG_VERSION");
    CString::new(version).unwrap().into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn keyrx_free_string(s: *mut c_char) {
    if !s.is_null() {
        let _ = CString::from_raw(s);
    }
}

#[no_mangle]
pub unsafe extern "C" fn keyrx_last_error() -> *mut c_char {
    // This is a placeholder. Real implementation might use thread-local storage
    // or a global error registry if needed for legacy C interop style.
    // For now returning null or empty string as we use Result JSON returns.
    ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn keyrx_register_event_callback(
    event_type: i32,
    callback: Option<unsafe extern "C" fn(*const u8, usize)>,
) -> i32 {
    let event_type = match EventType::from_i32(event_type) {
        Some(t) => t,
        None => return -1,
    };

    global_event_registry().register(event_type, callback);
    0
}

#[no_mangle]
pub unsafe extern "C" fn keyrx_protocol_version() -> u32 {
    1
}

/// Get FFI introspection metadata (developer tools)
///
/// Returns JSON with all available FFI functions, parameters, and events.
/// This enables runtime discovery of FFI capabilities.
#[no_mangle]
pub unsafe extern "C" fn keyrx_introspection_metadata() -> *mut c_char {
    use crate::ffi::introspection::{generate_introspection_data, init_contracts};

    // Initialize contracts if not already done
    if let Err(e) = init_contracts() {
        // Already initialized is OK
        if !e.to_string().contains("already initialized") {
            let err_result: FfiResult<()> = Err(e);
            return match serialize_ffi_result(&err_result) {
                Ok(json) => CString::new(json).unwrap_or_default().into_raw(),
                Err(_) => ptr::null_mut(),
            };
        }
    }

    // Generate introspection data
    let result = generate_introspection_data();
    match serialize_ffi_result(&result) {
        Ok(json) => CString::new(json).unwrap_or_default().into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn keyrx_set_config_root(_path: *const c_char) -> i32 {
    // TODO: Implement config root override
    0
}

#[no_mangle]
pub unsafe extern "C" fn keyrx_revolutionary_runtime_init() -> i32 {
    init_revolutionary_runtime();
    0
}

#[no_mangle]
pub unsafe extern "C" fn keyrx_revolutionary_runtime_shutdown() -> i32 {
    let _ = clear_revolutionary_runtime();
    0
}

#[no_mangle]
pub unsafe extern "C" fn keyrx_free_event_payload(ptr: *mut u8, len: usize) {
    if !ptr.is_null() && len > 0 {
        let slice = std::ptr::slice_from_raw_parts_mut(ptr, len);
        let _ = Box::from_raw(slice);
    }
}

use crate::config::manager::ConfigManager;
use std::sync::OnceLock;

/// Global config manager instance.
///
/// Stores configuration in `~/.keyrx` (or platform equivalent).
pub(crate) fn global_config_manager() -> &'static ConfigManager {
    static MANAGER: OnceLock<ConfigManager> = OnceLock::new();
    MANAGER.get_or_init(|| {
        let home = dirs::home_dir().expect("failed to determine home directory");
        let config_root = home.join(".keyrx");
        ConfigManager::new(config_root)
    })
}

// Helper for JSON response
fn ffi_json<T: serde::Serialize>(result: FfiResult<T>) -> *mut c_char {
    let payload = serialize_ffi_result(&result).unwrap_or_else(|e| {
        format!("error:{{\"code\":\"SERIALIZATION_FAILED\",\"message\":\"{e}\"}}")
    });
    CString::new(payload)
        .unwrap_or(
            CString::new(r#"error:{"code":"INTERNAL_ERROR","message":"Nul byte in JSON"}"#)
                .unwrap(),
        )
        .into_raw()
}

// Config Exports

#[no_mangle]
pub unsafe extern "C" fn keyrx_config_list_virtual_layouts() -> *mut c_char {
    let result = global_config_manager()
        .load_virtual_layouts()
        .map(|map| map.into_values().collect::<Vec<_>>())
        .map_err(|e| FfiError::internal(e.to_string()));
    ffi_json(result)
}

#[no_mangle]
pub unsafe extern "C" fn keyrx_config_save_virtual_layout(json: *const c_char) -> *mut c_char {
    let result = (|| -> FfiResult<()> {
        let json_str = parse_c_string(json, "json")?;
        let layout: VirtualLayout =
            serde_json::from_str(&json_str).map_err(|e| FfiError::invalid_input(e.to_string()))?;
        global_config_manager()
            .save_virtual_layout(&layout)
            .map_err(|e| FfiError::internal(e.to_string()))?;
        Ok(())
    })();
    ffi_json(result)
}

#[no_mangle]
pub unsafe extern "C" fn keyrx_config_delete_virtual_layout(id: *const c_char) -> *mut c_char {
    let result = (|| -> FfiResult<()> {
        let id_str = parse_c_string(id, "id")?;
        global_config_manager()
            .delete_virtual_layout(&id_str)
            .map_err(|e| FfiError::internal(e.to_string()))?;
        Ok(())
    })();
    ffi_json(result)
}

#[no_mangle]
pub unsafe extern "C" fn keyrx_config_list_hardware_profiles() -> *mut c_char {
    let result = global_config_manager()
        .load_hardware_profiles()
        .map(|map| map.into_values().collect::<Vec<_>>())
        .map_err(|e| FfiError::internal(e.to_string()));
    ffi_json(result)
}

#[no_mangle]
pub unsafe extern "C" fn keyrx_config_save_hardware_profile(json: *const c_char) -> *mut c_char {
    let result = (|| -> FfiResult<()> {
        let json_str = parse_c_string(json, "json")?;
        let profile: HardwareProfile =
            serde_json::from_str(&json_str).map_err(|e| FfiError::invalid_input(e.to_string()))?;
        global_config_manager()
            .save_hardware_profile(&profile)
            .map_err(|e| FfiError::internal(e.to_string()))?;
        Ok(())
    })();
    ffi_json(result)
}

#[no_mangle]
pub unsafe extern "C" fn keyrx_config_delete_hardware_profile(id: *const c_char) -> *mut c_char {
    let result = (|| -> FfiResult<()> {
        let id_str = parse_c_string(id, "id")?;
        global_config_manager()
            .delete_hardware_profile(&id_str)
            .map_err(|e| FfiError::internal(e.to_string()))?;
        Ok(())
    })();
    ffi_json(result)
}

#[no_mangle]
pub unsafe extern "C" fn keyrx_config_list_keymaps() -> *mut c_char {
    let result = global_config_manager()
        .load_keymaps()
        .map(|map| map.into_values().collect::<Vec<_>>())
        .map_err(|e| FfiError::internal(e.to_string()));
    ffi_json(result)
}

#[no_mangle]
pub unsafe extern "C" fn keyrx_config_save_keymap(json: *const c_char) -> *mut c_char {
    let result = (|| -> FfiResult<()> {
        let json_str = parse_c_string(json, "json")?;
        let keymap: Keymap =
            serde_json::from_str(&json_str).map_err(|e| FfiError::invalid_input(e.to_string()))?;
        global_config_manager()
            .save_keymap(&keymap)
            .map_err(|e| FfiError::internal(e.to_string()))?;
        Ok(())
    })();
    ffi_json(result)
}

#[no_mangle]
pub unsafe extern "C" fn keyrx_config_delete_keymap(id: *const c_char) -> *mut c_char {
    let result = (|| -> FfiResult<()> {
        let id_str = parse_c_string(id, "id")?;
        global_config_manager()
            .delete_keymap(&id_str)
            .map_err(|e| FfiError::internal(e.to_string()))?;
        Ok(())
    })();
    ffi_json(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::events::EventType;

    #[test]
    fn init_is_idempotent() {
        unsafe {
            assert_eq!(keyrx_init(), 0);
            assert_eq!(keyrx_init(), 0);
        }
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
        for code in 0..=15 {
            unsafe {
                assert_eq!(keyrx_register_event_callback(code, None), 0);
            }
        }
    }

    #[test]
    fn register_event_callback_rejects_invalid_codes() {
        unsafe {
            assert_eq!(keyrx_register_event_callback(-1, None), -1);
            assert_eq!(keyrx_register_event_callback(18, None), -1);
            assert_eq!(keyrx_register_event_callback(100, None), -1);
        }
    }
}
