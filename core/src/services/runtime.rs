use crate::config::models::{DeviceInstanceId, DeviceSlots, ProfileSlot, RuntimeConfig};
use crate::config::{ConfigManager, StorageError};
use thiserror::Error;

use super::traits::RuntimeServiceTrait;

#[derive(Error, Debug)]
pub enum RuntimeServiceError {
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
    #[error("Device not found: {0:?}")]
    DeviceNotFound(DeviceInstanceId),
    #[error("Slot not found: {0}")]
    SlotNotFound(String),
}

pub struct RuntimeService {
    config_manager: ConfigManager,
}

impl Default for RuntimeService {
    fn default() -> Self {
        Self::with_defaults()
    }
}

impl RuntimeService {
    pub fn new(config_manager: ConfigManager) -> Self {
        Self { config_manager }
    }

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
