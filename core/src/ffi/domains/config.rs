//! Config domain FFI implementation using the keyrx_ffi macro.
//!
//! This module provides FFI functions for configuration management operations
//! including virtual layouts, hardware profiles, and keymaps. The FFI wrapper
//! functions are automatically generated from the config.ffi-contract.json contract.

use crate::config::manager::ConfigManager;
use crate::config::models::{HardwareProfile, Keymap, VirtualLayout};
use keyrx_ffi_macro::keyrx_ffi;
use std::sync::OnceLock;

/// Global config manager instance.
///
/// Stores configuration in `~/.keyrx` (or platform equivalent).
pub fn global_config_manager() -> &'static ConfigManager {
    static MANAGER: OnceLock<ConfigManager> = OnceLock::new();
    MANAGER.get_or_init(ConfigManager::default)
}

/// ConfigDomain provides configuration management FFI functions.
///
/// This impl block is annotated with `#[keyrx_ffi]` which generates
/// extern "C" FFI wrapper functions for each method based on the
/// config.ffi-contract.json contract.
#[keyrx_ffi(domain = "config")]
impl ConfigDomain {
    /// List all virtual layouts.
    fn list_virtual_layouts() -> Result<Vec<VirtualLayout>, String> {
        global_config_manager()
            .load_virtual_layouts()
            .map(|map| map.into_values().collect())
            .map_err(|e| e.to_string())
    }

    /// Save or update a virtual layout.
    fn save_virtual_layout(layout: VirtualLayout) -> Result<VirtualLayout, String> {
        global_config_manager()
            .save_virtual_layout(&layout)
            .map_err(|e| e.to_string())?;
        Ok(layout)
    }

    /// Delete a virtual layout by ID.
    fn delete_virtual_layout(id: String) -> Result<(), String> {
        global_config_manager()
            .delete_virtual_layout(&id)
            .map_err(|e| e.to_string())
    }

    /// List all hardware profiles.
    fn list_hardware_profiles() -> Result<Vec<HardwareProfile>, String> {
        global_config_manager()
            .load_hardware_profiles()
            .map(|map| map.into_values().collect())
            .map_err(|e| e.to_string())
    }

    /// Save or update a hardware profile.
    fn save_hardware_profile(profile: HardwareProfile) -> Result<HardwareProfile, String> {
        global_config_manager()
            .save_hardware_profile(&profile)
            .map_err(|e| e.to_string())?;
        Ok(profile)
    }

    /// Delete a hardware profile by ID.
    fn delete_hardware_profile(id: String) -> Result<(), String> {
        global_config_manager()
            .delete_hardware_profile(&id)
            .map_err(|e| e.to_string())
    }

    /// List all keymaps.
    fn list_keymaps() -> Result<Vec<Keymap>, String> {
        global_config_manager()
            .load_keymaps()
            .map(|map| map.into_values().collect())
            .map_err(|e| e.to_string())
    }

    /// Save or update a keymap.
    fn save_keymap(keymap: Keymap) -> Result<Keymap, String> {
        global_config_manager()
            .save_keymap(&keymap)
            .map_err(|e| e.to_string())?;
        Ok(keymap)
    }

    /// Delete a keymap by ID.
    fn delete_keymap(id: String) -> Result<(), String> {
        global_config_manager()
            .delete_keymap(&id)
            .map_err(|e| e.to_string())
    }

    /// Get the content root path.
    fn get_config_root() -> Result<String, String> {
        let path = global_config_manager().root_path().to_string_lossy();
        Ok(path.into_owned())
    }
}

/// Placeholder struct for the impl block.
struct ConfigDomain;
