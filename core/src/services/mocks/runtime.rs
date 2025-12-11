//! Mock implementation of RuntimeServiceTrait for testing.

// Allow unwrap on mutex locks in mocks - poison panic is acceptable in test infrastructure
#![allow(clippy::unwrap_used)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::config::models::{DeviceInstanceId, DeviceSlots, ProfileSlot, RuntimeConfig};
use crate::services::runtime::RuntimeServiceError;
use crate::services::traits::RuntimeServiceTrait;

/// Mock implementation of [`RuntimeServiceTrait`] for testing.
///
/// Provides configurable responses and call tracking for all runtime operations.
/// All operations are pure in-memory with no I/O.
///
/// This mock maintains actual state for slot operations:
/// - `add_slot` adds slots to devices (creating device entries as needed)
/// - `remove_slot` removes slots from devices
/// - `reorder_slot` updates slot priorities and re-sorts
/// - `set_slot_active` toggles slot active state
/// - Slots are automatically sorted by priority (descending)
///
/// # Example
///
/// ```rust,ignore
/// let mock = MockRuntimeService::new();
/// let device = DeviceInstanceId { vendor_id: 0x1234, product_id: 0x5678, serial: None };
/// let slot = ProfileSlot { id: "slot-1".into(), priority: 100, active: true, .. };
///
/// // Add a slot - device entry is created automatically
/// let config = mock.add_slot(device.clone(), slot).unwrap();
/// assert_eq!(config.devices.len(), 1);
/// assert_eq!(config.devices[0].slots.len(), 1);
///
/// // Remove the slot
/// let config = mock.remove_slot(device.clone(), "slot-1").unwrap();
/// assert_eq!(config.devices[0].slots.len(), 0);
/// ```
pub struct MockRuntimeService {
    /// Runtime config state (devices and their slots)
    config: Arc<Mutex<RuntimeConfig>>,
    /// Error to return from get_config
    get_config_error: Option<String>,
    /// Error to return from add_slot
    add_slot_error: Option<String>,
    /// Error to return from remove_slot
    remove_slot_error: Option<String>,
    /// Error to return from reorder_slot
    reorder_slot_error: Option<String>,
    /// Error to return from set_slot_active
    set_slot_active_error: Option<String>,
    /// Tracks method call counts for verification
    call_counts: Arc<Mutex<HashMap<String, usize>>>,
}

impl MockRuntimeService {
    /// Creates a new MockRuntimeService with empty configuration.
    pub fn new() -> Self {
        Self {
            config: Arc::new(Mutex::new(RuntimeConfig { devices: vec![] })),
            get_config_error: None,
            add_slot_error: None,
            remove_slot_error: None,
            reorder_slot_error: None,
            set_slot_active_error: None,
            call_counts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Configures the runtime config to return.
    pub fn with_config(self, config: RuntimeConfig) -> Self {
        *self.config.lock().unwrap() = config;
        self
    }

    /// Configures an error to return from get_config.
    pub fn with_get_config_error(mut self, error: &str) -> Self {
        self.get_config_error = Some(error.to_string());
        self
    }

    /// Configures an error to return from add_slot.
    pub fn with_add_slot_error(mut self, error: &str) -> Self {
        self.add_slot_error = Some(error.to_string());
        self
    }

    /// Configures an error to return from remove_slot.
    pub fn with_remove_slot_error(mut self, error: &str) -> Self {
        self.remove_slot_error = Some(error.to_string());
        self
    }

    /// Configures an error to return from reorder_slot.
    pub fn with_reorder_slot_error(mut self, error: &str) -> Self {
        self.reorder_slot_error = Some(error.to_string());
        self
    }

    /// Configures an error to return from set_slot_active.
    pub fn with_set_slot_active_error(mut self, error: &str) -> Self {
        self.set_slot_active_error = Some(error.to_string());
        self
    }

    /// Returns the number of times a method was called.
    pub fn get_call_count(&self, method: &str) -> usize {
        self.call_counts
            .lock()
            .unwrap()
            .get(method)
            .copied()
            .unwrap_or(0)
    }

    fn increment_call(&self, method: &str) {
        let mut counts = self.call_counts.lock().unwrap();
        *counts.entry(method.to_string()).or_insert(0) += 1;
    }
}

impl Default for MockRuntimeService {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to create runtime errors from strings
fn make_runtime_error(msg: &str) -> RuntimeServiceError {
    RuntimeServiceError::SlotNotFound(msg.to_string())
}

impl RuntimeServiceTrait for MockRuntimeService {
    fn get_config(&self) -> Result<RuntimeConfig, RuntimeServiceError> {
        self.increment_call("get_config");
        if let Some(ref error) = self.get_config_error {
            return Err(make_runtime_error(error));
        }
        Ok(self.config.lock().unwrap().clone())
    }

    fn add_slot(
        &self,
        device: DeviceInstanceId,
        slot: ProfileSlot,
    ) -> Result<RuntimeConfig, RuntimeServiceError> {
        self.increment_call("add_slot");
        if let Some(ref error) = self.add_slot_error {
            return Err(make_runtime_error(error));
        }

        let mut config = self.config.lock().unwrap();
        let device_index = config
            .devices
            .iter()
            .position(|d| d.device == device)
            .unwrap_or_else(|| {
                config.devices.push(DeviceSlots {
                    device: device.clone(),
                    slots: vec![],
                });
                config.devices.len() - 1
            });

        let slots = &mut config.devices[device_index].slots;

        // Upsert: update existing or add new
        if let Some(idx) = slots.iter().position(|s| s.id == slot.id) {
            slots[idx] = slot;
        } else {
            slots.push(slot);
        }

        // Sort by priority (descending)
        slots.sort_by(|a, b| b.priority.cmp(&a.priority));

        Ok(config.clone())
    }

    fn remove_slot(
        &self,
        device: DeviceInstanceId,
        slot_id: &str,
    ) -> Result<RuntimeConfig, RuntimeServiceError> {
        self.increment_call("remove_slot");
        if let Some(ref error) = self.remove_slot_error {
            return Err(make_runtime_error(error));
        }

        let mut config = self.config.lock().unwrap();
        let slots = config
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

        Ok(config.clone())
    }

    fn reorder_slot(
        &self,
        device: DeviceInstanceId,
        slot_id: &str,
        new_priority: u32,
    ) -> Result<RuntimeConfig, RuntimeServiceError> {
        self.increment_call("reorder_slot");
        if let Some(ref error) = self.reorder_slot_error {
            return Err(make_runtime_error(error));
        }

        let mut config = self.config.lock().unwrap();
        let slots = config
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

        // Re-sort by priority (descending)
        slots.sort_by(|a, b| b.priority.cmp(&a.priority));

        Ok(config.clone())
    }

    fn set_slot_active(
        &self,
        device: DeviceInstanceId,
        slot_id: &str,
        active: bool,
    ) -> Result<RuntimeConfig, RuntimeServiceError> {
        self.increment_call("set_slot_active");
        if let Some(ref error) = self.set_slot_active_error {
            return Err(make_runtime_error(error));
        }

        let mut config = self.config.lock().unwrap();
        let slots = config
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

        Ok(config.clone())
    }
}
