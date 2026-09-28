//! Device management service.
//!
//! This service provides device management operations including listing devices,
//! renaming, setting scope, and forgetting devices. It integrates the device
//! registry with platform-specific device enumeration.

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use crate::config::device_registry::DeviceRegistry;
use crate::daemon::DaemonSharedState;

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
    /// The running daemon in this process, attached once by the web layer
    /// (mirrors `ProfileService`). `active` in [`DeviceInfo`] means "this
    /// daemon currently has the device captured" - read from here, THE one
    /// source of truth (`DaemonSharedState::is_device_active`, published by
    /// `daemon::publish_device_state`); unattached (test mode, no daemon in
    /// this process) means nothing is captured, so every device reports
    /// `active: false` (H8).
    daemon_state: OnceLock<Arc<DaemonSharedState>>,
}

impl DeviceService {
    /// Create a new DeviceService with the given registry path
    pub fn new(config_dir: PathBuf) -> Self {
        let registry_path = config_dir.join("devices.json");
        Self {
            registry_path,
            daemon_state: OnceLock::new(),
        }
    }

    /// Returns the path to the device registry file
    pub fn registry_path(&self) -> &std::path::Path {
        &self.registry_path
    }

    /// Attaches the running daemon so `active` reflects what it actually
    /// has captured. Only the first attachment counts. Never attached in
    /// test mode, where nothing is captured.
    pub fn attach_daemon_state(&self, state: Arc<DaemonSharedState>) {
        if self.daemon_state.set(state).is_err() {
            log::debug!("DeviceService already attached to a daemon");
        }
    }

    /// Whether the running daemon (if any) currently has `id` captured.
    fn is_active(&self, id: &str) -> bool {
        self.daemon_state
            .get()
            .is_some_and(|state| state.is_device_active(id))
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
                    active: self.is_active(&id),
                    id: id.clone(),
                    name: registry_entry
                        .map(|e| e.name.clone())
                        .unwrap_or_else(|| kb.name.clone()),
                    path: kb.path.display().to_string(),
                    serial: kb.serial,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;

    /// H8: no daemon attached (test mode, or before the web layer wires
    /// one) must report every device inactive, never `true` by default.
    #[test]
    fn is_active_defaults_false_without_a_daemon() {
        let service = DeviceService::new(PathBuf::from("/tmp/keyrx-device-service-test"));
        assert!(!service.is_active("dev-a"));
    }

    /// H8: `active` reflects exactly what the attached daemon published as
    /// captured - not "every enumerated device", and not stale after a
    /// republish.
    #[test]
    fn is_active_reflects_the_attached_daemon_state() {
        let service = DeviceService::new(PathBuf::from("/tmp/keyrx-device-service-test"));
        let shared = Arc::new(DaemonSharedState::new(
            Arc::new(AtomicBool::new(true)),
            None,
            PathBuf::new(),
            0,
        ));
        service.attach_daemon_state(Arc::clone(&shared));

        assert!(!service.is_active("dev-a"), "nothing published yet");

        shared.set_active_devices(["dev-a".to_string()]);
        assert!(service.is_active("dev-a"));
        assert!(!service.is_active("dev-b"));

        shared.set_active_devices(["dev-b".to_string()]);
        assert!(
            !service.is_active("dev-a"),
            "must not report a device active after it drops out of the published set"
        );
        assert!(service.is_active("dev-b"));
    }
}
