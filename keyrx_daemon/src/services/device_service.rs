//! Device management service.
//!
//! This service provides device management operations including listing devices,
//! renaming, setting scope, and forgetting devices. It integrates the device
//! registry with platform-specific device enumeration.

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use crate::config::device_registry::{DeviceEntry, DeviceRegistry, DeviceValidationError};
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

/// Why a device edit failed - REST and RPC map this to their own errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceEditError {
    /// No such device in the registry.
    NotFound(String),
    /// The requested name/layout is not valid.
    Invalid(String),
    /// The registry file could not be read or written.
    Storage(String),
}

impl std::fmt::Display for DeviceEditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(m) | Self::Invalid(m) | Self::Storage(m) => f.write_str(m),
        }
    }
}

impl From<DeviceValidationError> for DeviceEditError {
    fn from(e: DeviceValidationError) -> Self {
        match e {
            DeviceValidationError::DeviceNotFound(msg) => Self::NotFound(msg),
            other => Self::Invalid(other.to_string()),
        }
    }
}

/// The name a connected device reports, used when a device is registered by
/// its first edit; the id otherwise.
fn connected_device_name(device_id: &str) -> String {
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    if let Ok(keyboards) = crate::device_manager::enumerate_keyboards() {
        if let Some(kb) = keyboards.into_iter().find(|kb| kb.device_id() == device_id) {
            return kb.name;
        }
    }
    device_id.to_string()
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

    /// Renames a device. A connected keyboard that was never edited is
    /// registered first (named after what it reports), so any listed device
    /// can be renamed.
    pub async fn rename_device(
        &self,
        id: &str,
        name: &str,
    ) -> Result<DeviceEntry, DeviceEditError> {
        self.edit(id, |registry| registry.rename(id, name))
    }

    /// Sets a device's keyboard layout (registering it first, as above).
    pub async fn set_layout(&self, id: &str, layout: &str) -> Result<DeviceEntry, DeviceEditError> {
        self.edit(id, |registry| registry.set_layout(id, layout))
    }

    /// The layout of a registered device (`None` = inherits the default).
    pub async fn get_layout(&self, id: &str) -> Result<Option<String>, DeviceEditError> {
        let registry = self.load()?;
        let device = registry
            .get(id)
            .ok_or_else(|| DeviceEditError::NotFound(format!("Device not found: {id}")))?;
        Ok(device.layout.clone())
    }

    /// Removes a device's saved name and layout.
    pub async fn forget_device(&self, id: &str) -> Result<(), DeviceEditError> {
        let mut registry = self.load()?;
        registry.forget(id).map_err(DeviceEditError::from)?;
        registry
            .save()
            .map_err(|e| DeviceEditError::Storage(e.to_string()))
    }

    fn load(&self) -> Result<DeviceRegistry, DeviceEditError> {
        DeviceRegistry::load(&self.registry_path)
            .map_err(|e| DeviceEditError::Storage(e.to_string()))
    }

    /// Load, register `id` if needed, apply `change`, save; returns the entry.
    fn edit(
        &self,
        id: &str,
        change: impl FnOnce(&mut DeviceRegistry) -> Result<(), DeviceValidationError>,
    ) -> Result<DeviceEntry, DeviceEditError> {
        let mut registry = self.load()?;
        registry
            .ensure_registered(id, &connected_device_name(id))
            .map_err(DeviceEditError::from)?;
        change(&mut registry).map_err(DeviceEditError::from)?;
        registry
            .save()
            .map_err(|e| DeviceEditError::Storage(e.to_string()))?;
        registry
            .get(id)
            .cloned()
            .ok_or_else(|| DeviceEditError::Storage(format!("device {id} vanished")))
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

    /// Every transport (REST, WS-RPC) edits through these methods: renaming
    /// or re-laying-out a keyboard that was never edited must register it
    /// instead of failing (the RPC path used to skip that step).
    #[tokio::test]
    async fn editing_an_unregistered_device_registers_it() {
        let dir = tempfile::tempdir().unwrap();
        let service = DeviceService::new(dir.path().to_path_buf());
        let entry = service.rename_device("usb-kbd-1", "Desk").await.unwrap();
        assert_eq!(entry.name, "Desk");
        let entry = service.set_layout("usb-kbd-2", "jis_109").await.unwrap();
        assert_eq!(entry.layout.as_deref(), Some("jis_109"));
        assert_eq!(
            service.get_layout("usb-kbd-2").await.unwrap().as_deref(),
            Some("jis_109")
        );
        service.forget_device("usb-kbd-1").await.unwrap();
        assert!(matches!(
            service.forget_device("usb-kbd-1").await,
            Err(DeviceEditError::NotFound(_))
        ));
        assert!(matches!(
            service.rename_device("usb-kbd-3", "").await,
            Err(DeviceEditError::Invalid(_))
        ));
    }
}
