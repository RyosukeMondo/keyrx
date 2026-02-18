//! Device management service.
//!
//! This service provides device management operations including listing devices,
//! renaming, setting scope, and forgetting devices. It integrates the device
//! registry with platform-specific device enumeration.

use std::path::PathBuf;

use crate::config::device_registry::DeviceRegistry;

/// Device information returned by service methods
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub path: String,
    pub serial: Option<String>,
    pub active: bool,
    pub layout: Option<String>,
}

/// Device management service
pub struct DeviceService {
    registry_path: PathBuf,
}

impl DeviceService {
    /// Create a new DeviceService with the given registry path
    pub fn new(config_dir: PathBuf) -> Self {
        let registry_path = config_dir.join("devices.json");
        Self { registry_path }
    }

    /// Returns the path to the device registry file
    pub fn registry_path(&self) -> &std::path::Path {
        &self.registry_path
    }

    /// List all connected devices
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    pub async fn list_devices(&self) -> Result<Vec<DeviceInfo>, String> {
        use crate::device_manager::enumerate_keyboards;

        // Load registry
        let registry = DeviceRegistry::load(&self.registry_path)
            .map_err(|e| format!("Failed to load device registry: {}", e))?;

        // Enumerate actual connected devices
        let keyboards =
            enumerate_keyboards().map_err(|e| format!("Failed to enumerate keyboards: {}", e))?;

        let devices: Vec<DeviceInfo> = keyboards
            .into_iter()
            .map(|kb| {
                let id = kb.device_id();
                let registry_entry = registry.get(&id);

                DeviceInfo {
                    id: id.clone(),
                    name: registry_entry
                        .map(|e| e.name.clone())
                        .unwrap_or_else(|| kb.name.clone()),
                    path: kb.path.display().to_string(),
                    serial: kb.serial,
                    active: true,
                    layout: registry_entry.and_then(|e| e.layout.clone()),
                }
            })
            .collect();

        Ok(devices)
    }

    /// List all connected devices (stub for unsupported platforms)
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    pub async fn list_devices(&self) -> Result<Vec<DeviceInfo>, String> {
        Ok(Vec::new())
    }

    /// Rename a device
    pub async fn rename_device(&self, id: &str, name: &str) -> Result<(), String> {
        let mut registry = DeviceRegistry::load(&self.registry_path)
            .map_err(|e| format!("Failed to load device registry: {}", e))?;

        registry
            .rename(id, name)
            .map_err(|e| format!("Failed to rename device: {}", e))?;

        registry
            .save()
            .map_err(|e| format!("Failed to save device registry: {}", e))?;

        Ok(())
    }

    /// Forget a device
    pub async fn forget_device(&self, id: &str) -> Result<(), String> {
        let mut registry = DeviceRegistry::load(&self.registry_path)
            .map_err(|e| format!("Failed to load device registry: {}", e))?;

        registry
            .forget(id)
            .map_err(|e| format!("Failed to forget device: {}", e))?;

        registry
            .save()
            .map_err(|e| format!("Failed to save device registry: {}", e))?;

        Ok(())
    }
}
