//! Runtime configuration service.
//!
//! This module provides the [`RuntimeService`] for managing runtime configuration,
//! including device-to-profile slot assignments and their priorities.

use crate::config::models::{DeviceInstanceId, DeviceSlots, ProfileSlot, RuntimeConfig};
use crate::config::{ConfigManager, StorageError};
use thiserror::Error;

use super::traits::RuntimeServiceTrait;

/// Errors that can occur during runtime service operations.
///
/// This error type covers failures when managing runtime configuration,
/// such as storage errors and invalid references.
#[derive(Error, Debug)]
pub enum RuntimeServiceError {
    /// Error from the underlying storage layer.
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    /// The specified device was not found in the runtime configuration.
    #[error("Device not found: {0:?}")]
    DeviceNotFound(DeviceInstanceId),

    /// The specified profile slot was not found for the device.
    #[error("Slot not found: {0}")]
    SlotNotFound(String),
}

/// Service for managing runtime configuration.
///
/// The runtime service manages the relationship between devices and profile slots,
/// controlling which profiles are active for each device and their priority ordering.
///
/// # Dependency Injection
///
/// Use [`RuntimeService::new`] to inject a custom [`ConfigManager`] for testing,
/// or [`RuntimeService::with_defaults`] for production use.
///
/// # Example
///
/// ```no_run
/// use keyrx_core::services::{RuntimeService, RuntimeServiceTrait};
///
/// let service = RuntimeService::with_defaults();
/// let config = service.get_config().expect("Failed to load config");
/// for device in &config.devices {
///     println!("Device: {:?}, {} slots", device.device, device.slots.len());
/// }
/// ```
pub struct RuntimeService {
    config_manager: ConfigManager,
}

impl Default for RuntimeService {
    fn default() -> Self {
        Self::with_defaults()
    }
}

impl RuntimeService {
    /// Creates a new RuntimeService with the provided ConfigManager.
    ///
    /// Use this constructor for dependency injection, allowing tests to
    /// inject mock or custom ConfigManager implementations.
    ///
    /// # Arguments
    ///
    /// * `config_manager` - The configuration manager to use for storage
    pub fn new(config_manager: ConfigManager) -> Self {
        Self { config_manager }
    }

    /// Creates a RuntimeService with default dependencies.
    ///
    /// This is the convenience constructor for production use, creating
    /// a ConfigManager with default settings.
    pub fn with_defaults() -> Self {
        Self::new(ConfigManager::default())
    }

    fn update_runtime_config<F>(&self, mutate: F) -> Result<RuntimeConfig, RuntimeServiceError>
    where
        F: FnOnce(&mut RuntimeConfig) -> Result<(), RuntimeServiceError>,
    {
        let mut runtime = self.config_manager.load_runtime_config()?;
        mutate(&mut runtime)?;
        self.config_manager.save_runtime_config(&runtime)?;
        Ok(runtime)
    }
}

impl RuntimeServiceTrait for RuntimeService {
    fn get_config(&self) -> Result<RuntimeConfig, RuntimeServiceError> {
        self.config_manager
            .load_runtime_config()
            .map_err(Into::into)
    }

    fn add_slot(
        &self,
        device: DeviceInstanceId,
        slot: ProfileSlot,
    ) -> Result<RuntimeConfig, RuntimeServiceError> {
        self.update_runtime_config(|runtime| {
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

            let slots = &mut runtime.devices[device_index].slots;

            // Upsert
            if let Some(idx) = slots.iter().position(|s| s.id == slot.id) {
                slots[idx] = slot.clone();
            } else {
                slots.push(slot.clone());
            }

            // Sort by priority (descending)
            slots.sort_by(|a, b| b.priority.cmp(&a.priority));

            Ok(())
        })
    }

    fn remove_slot(
        &self,
        device: DeviceInstanceId,
        slot_id: &str,
    ) -> Result<RuntimeConfig, RuntimeServiceError> {
        self.update_runtime_config(|runtime| {
            let slots = runtime
                .devices
                .iter_mut()
                .find(|d| d.device == device)
                .map(|d| &mut d.slots)
                .ok_or_else(|| RuntimeServiceError::DeviceNotFound(device.clone()))?;

            let initial_len = slots.len();
            slots.retain(|s| s.id != slot_id);

            if slots.len() == initial_len {
                return Err(RuntimeServiceError::SlotNotFound(slot_id.to_string()));
            }

            Ok(())
        })
    }

    fn reorder_slot(
        &self,
        device: DeviceInstanceId,
        slot_id: &str,
        new_priority: u32,
    ) -> Result<RuntimeConfig, RuntimeServiceError> {
        self.update_runtime_config(|runtime| {
            let slots = runtime
                .devices
                .iter_mut()
                .find(|d| d.device == device)
                .map(|d| &mut d.slots)
                .ok_or_else(|| RuntimeServiceError::DeviceNotFound(device.clone()))?;

            let slot = slots
                .iter_mut()
                .find(|s| s.id == slot_id)
                .ok_or_else(|| RuntimeServiceError::SlotNotFound(slot_id.to_string()))?;

            slot.priority = new_priority;

            // Re-sort
            slots.sort_by(|a, b| b.priority.cmp(&a.priority));

            Ok(())
        })
    }

    fn set_slot_active(
        &self,
        device: DeviceInstanceId,
        slot_id: &str,
        active: bool,
    ) -> Result<RuntimeConfig, RuntimeServiceError> {
        self.update_runtime_config(|runtime| {
            let slots = runtime
                .devices
                .iter_mut()
                .find(|d| d.device == device)
                .map(|d| &mut d.slots)
                .ok_or_else(|| RuntimeServiceError::DeviceNotFound(device.clone()))?;

            let slot = slots
                .iter_mut()
                .find(|s| s.id == slot_id)
                .ok_or_else(|| RuntimeServiceError::SlotNotFound(slot_id.to_string()))?;

            slot.active = active;

            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn test_device_id() -> DeviceInstanceId {
        DeviceInstanceId {
            vendor_id: 0x1234,
            product_id: 0x5678,
            serial: None,
        }
    }

    fn test_slot(id: &str, priority: u32) -> ProfileSlot {
        ProfileSlot {
            id: id.into(),
            hardware_profile_id: "profile-1".into(),
            keymap_id: "keymap-1".into(),
            priority,
            active: true,
        }
    }

    #[test]
    fn service_new_creates_with_config_manager() {
        let dir = tempdir().unwrap();
        let config_manager = ConfigManager::new(dir.path());
        let service = RuntimeService::new(config_manager);

        // Verify service works by calling get_config
        let config = service.get_config().unwrap();
        assert!(config.devices.is_empty());
    }

    #[test]
    fn service_default_creates_with_defaults() {
        // Just verify Default impl works
        let _service = RuntimeService::default();
    }

    #[test]
    fn get_config_returns_empty_config_initially() {
        let dir = tempdir().unwrap();
        let service = RuntimeService::new(ConfigManager::new(dir.path()));

        let config = service.get_config().unwrap();
        assert!(config.devices.is_empty());
    }

    #[test]
    fn add_slot_creates_device_entry_if_not_exists() {
        let dir = tempdir().unwrap();
        let service = RuntimeService::new(ConfigManager::new(dir.path()));

        let device = test_device_id();
        let slot = test_slot("slot-1", 100);

        let config = service.add_slot(device.clone(), slot).unwrap();
        assert_eq!(config.devices.len(), 1);
        assert_eq!(config.devices[0].device, device);
        assert_eq!(config.devices[0].slots.len(), 1);
        assert_eq!(config.devices[0].slots[0].id, "slot-1");
    }

    #[test]
    fn add_slot_appends_to_existing_device() {
        let dir = tempdir().unwrap();
        let service = RuntimeService::new(ConfigManager::new(dir.path()));

        let device = test_device_id();

        // Add first slot
        service
            .add_slot(device.clone(), test_slot("slot-1", 100))
            .unwrap();

        // Add second slot
        let config = service
            .add_slot(device.clone(), test_slot("slot-2", 200))
            .unwrap();

        assert_eq!(config.devices.len(), 1);
        assert_eq!(config.devices[0].slots.len(), 2);
    }

    #[test]
    fn add_slot_updates_existing_slot_with_same_id() {
        let dir = tempdir().unwrap();
        let service = RuntimeService::new(ConfigManager::new(dir.path()));

        let device = test_device_id();

        // Add initial slot
        service
            .add_slot(device.clone(), test_slot("slot-1", 100))
            .unwrap();

        // Update same slot
        let config = service
            .add_slot(device.clone(), test_slot("slot-1", 200))
            .unwrap();

        assert_eq!(config.devices[0].slots.len(), 1);
        assert_eq!(config.devices[0].slots[0].priority, 200);
    }

    #[test]
    fn add_slot_sorts_by_priority_descending() {
        let dir = tempdir().unwrap();
        let service = RuntimeService::new(ConfigManager::new(dir.path()));

        let device = test_device_id();

        // Add slots in wrong order
        service
            .add_slot(device.clone(), test_slot("low", 10))
            .unwrap();
        service
            .add_slot(device.clone(), test_slot("high", 100))
            .unwrap();
        let config = service
            .add_slot(device.clone(), test_slot("medium", 50))
            .unwrap();

        // Should be sorted by priority descending
        assert_eq!(config.devices[0].slots[0].id, "high");
        assert_eq!(config.devices[0].slots[1].id, "medium");
        assert_eq!(config.devices[0].slots[2].id, "low");
    }

    #[test]
    fn remove_slot_removes_existing_slot() {
        let dir = tempdir().unwrap();
        let service = RuntimeService::new(ConfigManager::new(dir.path()));

        let device = test_device_id();

        // Add slots
        service
            .add_slot(device.clone(), test_slot("slot-1", 100))
            .unwrap();
        service
            .add_slot(device.clone(), test_slot("slot-2", 200))
            .unwrap();

        // Remove one
        let config = service.remove_slot(device.clone(), "slot-1").unwrap();

        assert_eq!(config.devices[0].slots.len(), 1);
        assert_eq!(config.devices[0].slots[0].id, "slot-2");
    }

    #[test]
    fn remove_slot_returns_error_for_unknown_device() {
        let dir = tempdir().unwrap();
        let service = RuntimeService::new(ConfigManager::new(dir.path()));

        let device = test_device_id();
        let result = service.remove_slot(device, "slot-1");

        assert!(matches!(
            result,
            Err(RuntimeServiceError::DeviceNotFound(_))
        ));
    }

    #[test]
    fn remove_slot_returns_error_for_unknown_slot() {
        let dir = tempdir().unwrap();
        let service = RuntimeService::new(ConfigManager::new(dir.path()));

        let device = test_device_id();
        service
            .add_slot(device.clone(), test_slot("slot-1", 100))
            .unwrap();

        let result = service.remove_slot(device, "nonexistent");

        assert!(matches!(result, Err(RuntimeServiceError::SlotNotFound(_))));
    }

    #[test]
    fn reorder_slot_updates_priority_and_sorts() {
        let dir = tempdir().unwrap();
        let service = RuntimeService::new(ConfigManager::new(dir.path()));

        let device = test_device_id();

        service
            .add_slot(device.clone(), test_slot("low", 10))
            .unwrap();
        service
            .add_slot(device.clone(), test_slot("high", 100))
            .unwrap();

        // Move "low" to top priority
        let config = service.reorder_slot(device.clone(), "low", 500).unwrap();

        assert_eq!(config.devices[0].slots[0].id, "low");
        assert_eq!(config.devices[0].slots[0].priority, 500);
    }

    #[test]
    fn reorder_slot_returns_error_for_unknown_device() {
        let dir = tempdir().unwrap();
        let service = RuntimeService::new(ConfigManager::new(dir.path()));

        let device = test_device_id();
        let result = service.reorder_slot(device, "slot-1", 100);

        assert!(matches!(
            result,
            Err(RuntimeServiceError::DeviceNotFound(_))
        ));
    }

    #[test]
    fn reorder_slot_returns_error_for_unknown_slot() {
        let dir = tempdir().unwrap();
        let service = RuntimeService::new(ConfigManager::new(dir.path()));

        let device = test_device_id();
        service
            .add_slot(device.clone(), test_slot("slot-1", 100))
            .unwrap();

        let result = service.reorder_slot(device, "nonexistent", 200);

        assert!(matches!(result, Err(RuntimeServiceError::SlotNotFound(_))));
    }

    #[test]
    fn set_slot_active_updates_active_state() {
        let dir = tempdir().unwrap();
        let service = RuntimeService::new(ConfigManager::new(dir.path()));

        let device = test_device_id();
        service
            .add_slot(device.clone(), test_slot("slot-1", 100))
            .unwrap();

        // Deactivate
        let config = service
            .set_slot_active(device.clone(), "slot-1", false)
            .unwrap();
        assert!(!config.devices[0].slots[0].active);

        // Reactivate
        let config = service
            .set_slot_active(device.clone(), "slot-1", true)
            .unwrap();
        assert!(config.devices[0].slots[0].active);
    }

    #[test]
    fn set_slot_active_returns_error_for_unknown_device() {
        let dir = tempdir().unwrap();
        let service = RuntimeService::new(ConfigManager::new(dir.path()));

        let device = test_device_id();
        let result = service.set_slot_active(device, "slot-1", false);

        assert!(matches!(
            result,
            Err(RuntimeServiceError::DeviceNotFound(_))
        ));
    }

    #[test]
    fn set_slot_active_returns_error_for_unknown_slot() {
        let dir = tempdir().unwrap();
        let service = RuntimeService::new(ConfigManager::new(dir.path()));

        let device = test_device_id();
        service
            .add_slot(device.clone(), test_slot("slot-1", 100))
            .unwrap();

        let result = service.set_slot_active(device, "nonexistent", false);

        assert!(matches!(result, Err(RuntimeServiceError::SlotNotFound(_))));
    }

    #[test]
    fn config_persists_across_service_instances() {
        let dir = tempdir().unwrap();

        // Create first service and add slot
        let service1 = RuntimeService::new(ConfigManager::new(dir.path()));
        let device = test_device_id();
        service1
            .add_slot(device.clone(), test_slot("slot-1", 100))
            .unwrap();

        // Create second service from same directory
        let service2 = RuntimeService::new(ConfigManager::new(dir.path()));
        let config = service2.get_config().unwrap();

        assert_eq!(config.devices.len(), 1);
        assert_eq!(config.devices[0].slots.len(), 1);
    }

    // ========== Error Type Tests ==========

    #[test]
    fn runtime_service_error_display() {
        let device_err = RuntimeServiceError::DeviceNotFound(test_device_id());
        assert!(device_err.to_string().contains("Device not found"));

        let slot_err = RuntimeServiceError::SlotNotFound("my-slot".into());
        assert!(slot_err.to_string().contains("Slot not found"));

        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let storage_err =
            RuntimeServiceError::Storage(StorageError::ReadFile("test.json".into(), io_err));
        assert!(storage_err.to_string().contains("Storage error"));
    }
}
