//! Mock service implementations for fast, isolated unit testing.
//!
//! This module provides mock implementations of all service traits, enabling:
//! - Pure in-memory testing without I/O
//! - Configurable success and error responses
//! - Call tracking for verification
//!
//! # Example
//!
//! ```rust,ignore
//! use keyrx_core::services::MockDeviceService;
//!
//! let mock = MockDeviceService::new()
//!     .with_devices(vec![test_device])
//!     .with_list_error(Some(DeviceServiceError::Io(std::io::Error::other("test"))));
//!
//! let api = ApiContext::new(Arc::new(mock), ...);
//! ```

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use crate::config::models::{
    DeviceInstanceId, DeviceSlots, HardwareProfile, Keymap, ProfileSlot, RuntimeConfig,
    VirtualLayout,
};

use super::device::{DeviceServiceError, DeviceView};
use super::profile::ProfileServiceError;
use super::runtime::RuntimeServiceError;
use super::traits::{DeviceServiceTrait, ProfileServiceTrait, RuntimeServiceTrait};

/// Mock implementation of DeviceServiceTrait for testing.
///
/// Provides configurable responses and call tracking for all device operations.
/// All operations are pure in-memory with no I/O.
pub struct MockDeviceService {
    /// Devices to return from list_devices and get_device
    devices: Vec<DeviceView>,
    /// Error to return from list_devices
    list_error: Option<DeviceServiceError>,
    /// Error to return from get_device
    get_error: Option<DeviceServiceError>,
    /// Error to return from set_remap_enabled
    set_remap_error: Option<DeviceServiceError>,
    /// Error to return from assign_profile
    assign_error: Option<DeviceServiceError>,
    /// Error to return from unassign_profile
    unassign_error: Option<DeviceServiceError>,
    /// Error to return from set_label
    set_label_error: Option<DeviceServiceError>,
    /// Tracks method call counts for verification
    call_counts: Arc<Mutex<HashMap<String, usize>>>,
}

impl MockDeviceService {
    /// Creates a new empty MockDeviceService.
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
            list_error: None,
            get_error: None,
            set_remap_error: None,
            assign_error: None,
            unassign_error: None,
            set_label_error: None,
            call_counts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Configures the devices to return from list_devices and get_device.
    pub fn with_devices(mut self, devices: Vec<DeviceView>) -> Self {
        self.devices = devices;
        self
    }

    /// Configures an error to return from list_devices.
    pub fn with_list_error(mut self, error: DeviceServiceError) -> Self {
        self.list_error = Some(error);
        self
    }

    /// Configures an error to return from get_device.
    pub fn with_get_error(mut self, error: DeviceServiceError) -> Self {
        self.get_error = Some(error);
        self
    }

    /// Configures an error to return from set_remap_enabled.
    pub fn with_set_remap_error(mut self, error: DeviceServiceError) -> Self {
        self.set_remap_error = Some(error);
        self
    }

    /// Configures an error to return from assign_profile.
    pub fn with_assign_error(mut self, error: DeviceServiceError) -> Self {
        self.assign_error = Some(error);
        self
    }

    /// Configures an error to return from unassign_profile.
    pub fn with_unassign_error(mut self, error: DeviceServiceError) -> Self {
        self.unassign_error = Some(error);
        self
    }

    /// Configures an error to return from set_label.
    pub fn with_set_label_error(mut self, error: DeviceServiceError) -> Self {
        self.set_label_error = Some(error);
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

    fn find_device(&self, device_key: &str) -> Option<DeviceView> {
        self.devices.iter().find(|d| d.key == device_key).cloned()
    }
}

impl Default for MockDeviceService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DeviceServiceTrait for MockDeviceService {
    async fn list_devices(&self) -> Result<Vec<DeviceView>, DeviceServiceError> {
        self.increment_call("list_devices");
        if let Some(ref error) = self.list_error {
            return Err(make_io_error(&error.to_string()));
        }
        Ok(self.devices.clone())
    }

    async fn get_device(&self, device_key: &str) -> Result<DeviceView, DeviceServiceError> {
        self.increment_call("get_device");
        if let Some(ref error) = self.get_error {
            return Err(make_io_error(&error.to_string()));
        }
        self.find_device(device_key)
            .ok_or_else(|| DeviceServiceError::DeviceNotFound(device_key.to_string()))
    }

    async fn set_remap_enabled(
        &self,
        device_key: &str,
        _enabled: bool,
    ) -> Result<DeviceView, DeviceServiceError> {
        self.increment_call("set_remap_enabled");
        if let Some(ref error) = self.set_remap_error {
            return Err(make_io_error(&error.to_string()));
        }
        self.find_device(device_key)
            .ok_or_else(|| DeviceServiceError::DeviceNotFound(device_key.to_string()))
    }

    async fn assign_profile(
        &self,
        device_key: &str,
        _profile_id: &str,
    ) -> Result<DeviceView, DeviceServiceError> {
        self.increment_call("assign_profile");
        if let Some(ref error) = self.assign_error {
            return Err(make_io_error(&error.to_string()));
        }
        self.find_device(device_key)
            .ok_or_else(|| DeviceServiceError::DeviceNotFound(device_key.to_string()))
    }

    async fn unassign_profile(&self, device_key: &str) -> Result<DeviceView, DeviceServiceError> {
        self.increment_call("unassign_profile");
        if let Some(ref error) = self.unassign_error {
            return Err(make_io_error(&error.to_string()));
        }
        self.find_device(device_key)
            .ok_or_else(|| DeviceServiceError::DeviceNotFound(device_key.to_string()))
    }

    async fn set_label(
        &self,
        device_key: &str,
        _label: Option<String>,
    ) -> Result<DeviceView, DeviceServiceError> {
        self.increment_call("set_label");
        if let Some(ref error) = self.set_label_error {
            return Err(make_io_error(&error.to_string()));
        }
        self.find_device(device_key)
            .ok_or_else(|| DeviceServiceError::DeviceNotFound(device_key.to_string()))
    }
}

/// Helper to create IO errors from strings (since DeviceServiceError variants aren't Clone)
fn make_io_error(msg: &str) -> DeviceServiceError {
    DeviceServiceError::Io(std::io::Error::other(msg.to_string()))
}

/// Mock implementation of ProfileServiceTrait for testing.
///
/// Provides configurable responses and call tracking for all profile operations.
/// All operations are pure in-memory with no I/O.
pub struct MockProfileService {
    /// Virtual layouts to store and return
    virtual_layouts: Arc<Mutex<Vec<VirtualLayout>>>,
    /// Hardware profiles to store and return
    hardware_profiles: Arc<Mutex<Vec<HardwareProfile>>>,
    /// Keymaps to store and return
    keymaps: Arc<Mutex<Vec<Keymap>>>,
    /// Error to return from list_virtual_layouts
    list_layouts_error: Option<String>,
    /// Error to return from save_virtual_layout
    save_layout_error: Option<String>,
    /// Error to return from delete_virtual_layout
    delete_layout_error: Option<String>,
    /// Error to return from list_hardware_profiles
    list_profiles_error: Option<String>,
    /// Error to return from save_hardware_profile
    save_profile_error: Option<String>,
    /// Error to return from delete_hardware_profile
    delete_profile_error: Option<String>,
    /// Error to return from list_keymaps
    list_keymaps_error: Option<String>,
    /// Error to return from save_keymap
    save_keymap_error: Option<String>,
    /// Error to return from delete_keymap
    delete_keymap_error: Option<String>,
    /// Tracks method call counts for verification
    call_counts: Arc<Mutex<HashMap<String, usize>>>,
}

impl MockProfileService {
    /// Creates a new empty MockProfileService.
    pub fn new() -> Self {
        Self {
            virtual_layouts: Arc::new(Mutex::new(Vec::new())),
            hardware_profiles: Arc::new(Mutex::new(Vec::new())),
            keymaps: Arc::new(Mutex::new(Vec::new())),
            list_layouts_error: None,
            save_layout_error: None,
            delete_layout_error: None,
            list_profiles_error: None,
            save_profile_error: None,
            delete_profile_error: None,
            list_keymaps_error: None,
            save_keymap_error: None,
            delete_keymap_error: None,
            call_counts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Configures the virtual layouts to return.
    pub fn with_virtual_layouts(self, layouts: Vec<VirtualLayout>) -> Self {
        *self.virtual_layouts.lock().unwrap() = layouts;
        self
    }

    /// Configures the hardware profiles to return.
    pub fn with_hardware_profiles(self, profiles: Vec<HardwareProfile>) -> Self {
        *self.hardware_profiles.lock().unwrap() = profiles;
        self
    }

    /// Configures the keymaps to return.
    pub fn with_keymaps(self, keymaps: Vec<Keymap>) -> Self {
        *self.keymaps.lock().unwrap() = keymaps;
        self
    }

    /// Configures an error to return from list_virtual_layouts.
    pub fn with_list_layouts_error(mut self, error: &str) -> Self {
        self.list_layouts_error = Some(error.to_string());
        self
    }

    /// Configures an error to return from save_virtual_layout.
    pub fn with_save_layout_error(mut self, error: &str) -> Self {
        self.save_layout_error = Some(error.to_string());
        self
    }

    /// Configures an error to return from delete_virtual_layout.
    pub fn with_delete_layout_error(mut self, error: &str) -> Self {
        self.delete_layout_error = Some(error.to_string());
        self
    }

    /// Configures an error to return from list_hardware_profiles.
    pub fn with_list_profiles_error(mut self, error: &str) -> Self {
        self.list_profiles_error = Some(error.to_string());
        self
    }

    /// Configures an error to return from save_hardware_profile.
    pub fn with_save_profile_error(mut self, error: &str) -> Self {
        self.save_profile_error = Some(error.to_string());
        self
    }

    /// Configures an error to return from delete_hardware_profile.
    pub fn with_delete_profile_error(mut self, error: &str) -> Self {
        self.delete_profile_error = Some(error.to_string());
        self
    }

    /// Configures an error to return from list_keymaps.
    pub fn with_list_keymaps_error(mut self, error: &str) -> Self {
        self.list_keymaps_error = Some(error.to_string());
        self
    }

    /// Configures an error to return from save_keymap.
    pub fn with_save_keymap_error(mut self, error: &str) -> Self {
        self.save_keymap_error = Some(error.to_string());
        self
    }

    /// Configures an error to return from delete_keymap.
    pub fn with_delete_keymap_error(mut self, error: &str) -> Self {
        self.delete_keymap_error = Some(error.to_string());
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

impl Default for MockProfileService {
    fn default() -> Self {
        Self::new()
    }
}

impl ProfileServiceTrait for MockProfileService {
    fn list_virtual_layouts(&self) -> Result<Vec<VirtualLayout>, ProfileServiceError> {
        self.increment_call("list_virtual_layouts");
        if let Some(ref error) = self.list_layouts_error {
            return Err(make_profile_error(error));
        }
        Ok(self.virtual_layouts.lock().unwrap().clone())
    }

    fn save_virtual_layout(
        &self,
        layout: VirtualLayout,
    ) -> Result<VirtualLayout, ProfileServiceError> {
        self.increment_call("save_virtual_layout");
        if let Some(ref error) = self.save_layout_error {
            return Err(make_profile_error(error));
        }
        let mut layouts = self.virtual_layouts.lock().unwrap();
        // Update or add
        if let Some(existing) = layouts.iter_mut().find(|l| l.id == layout.id) {
            *existing = layout.clone();
        } else {
            layouts.push(layout.clone());
        }
        Ok(layout)
    }

    fn delete_virtual_layout(&self, id: &str) -> Result<(), ProfileServiceError> {
        self.increment_call("delete_virtual_layout");
        if let Some(ref error) = self.delete_layout_error {
            return Err(make_profile_error(error));
        }
        let mut layouts = self.virtual_layouts.lock().unwrap();
        layouts.retain(|l| l.id != id);
        Ok(())
    }

    fn list_hardware_profiles(&self) -> Result<Vec<HardwareProfile>, ProfileServiceError> {
        self.increment_call("list_hardware_profiles");
        if let Some(ref error) = self.list_profiles_error {
            return Err(make_profile_error(error));
        }
        Ok(self.hardware_profiles.lock().unwrap().clone())
    }

    fn save_hardware_profile(
        &self,
        profile: HardwareProfile,
    ) -> Result<HardwareProfile, ProfileServiceError> {
        self.increment_call("save_hardware_profile");
        if let Some(ref error) = self.save_profile_error {
            return Err(make_profile_error(error));
        }
        let mut profiles = self.hardware_profiles.lock().unwrap();
        if let Some(existing) = profiles.iter_mut().find(|p| p.id == profile.id) {
            *existing = profile.clone();
        } else {
            profiles.push(profile.clone());
        }
        Ok(profile)
    }

    fn delete_hardware_profile(&self, id: &str) -> Result<(), ProfileServiceError> {
        self.increment_call("delete_hardware_profile");
        if let Some(ref error) = self.delete_profile_error {
            return Err(make_profile_error(error));
        }
        let mut profiles = self.hardware_profiles.lock().unwrap();
        profiles.retain(|p| p.id != id);
        Ok(())
    }

    fn list_keymaps(&self) -> Result<Vec<Keymap>, ProfileServiceError> {
        self.increment_call("list_keymaps");
        if let Some(ref error) = self.list_keymaps_error {
            return Err(make_profile_error(error));
        }
        Ok(self.keymaps.lock().unwrap().clone())
    }

    fn save_keymap(&self, keymap: Keymap) -> Result<Keymap, ProfileServiceError> {
        self.increment_call("save_keymap");
        if let Some(ref error) = self.save_keymap_error {
            return Err(make_profile_error(error));
        }
        let mut keymaps = self.keymaps.lock().unwrap();
        if let Some(existing) = keymaps.iter_mut().find(|k| k.id == keymap.id) {
            *existing = keymap.clone();
        } else {
            keymaps.push(keymap.clone());
        }
        Ok(keymap)
    }

    fn delete_keymap(&self, id: &str) -> Result<(), ProfileServiceError> {
        self.increment_call("delete_keymap");
        if let Some(ref error) = self.delete_keymap_error {
            return Err(make_profile_error(error));
        }
        let mut keymaps = self.keymaps.lock().unwrap();
        keymaps.retain(|k| k.id != id);
        Ok(())
    }
}

/// Helper to create profile errors from strings
fn make_profile_error(msg: &str) -> ProfileServiceError {
    ProfileServiceError::NotFound(msg.to_string())
}

/// Mock implementation of RuntimeServiceTrait for testing.
///
/// Provides configurable responses and call tracking for all runtime operations.
/// All operations are pure in-memory with no I/O.
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

/// Helper to create runtime errors from strings
fn make_runtime_error(msg: &str) -> RuntimeServiceError {
    RuntimeServiceError::SlotNotFound(msg.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::models::LayoutType;

    fn test_device(key: &str) -> DeviceView {
        DeviceView {
            key: key.to_string(),
            vendor_id: 0x1234,
            product_id: 0x5678,
            serial_number: "test".to_string(),
            label: None,
            remap_enabled: false,
            profile_id: None,
            connected: true,
        }
    }

    fn test_layout(id: &str) -> VirtualLayout {
        VirtualLayout {
            id: id.to_string(),
            name: format!("Layout {}", id),
            layout_type: LayoutType::Semantic,
            keys: vec![],
        }
    }

    fn test_hardware_profile(id: &str) -> HardwareProfile {
        HardwareProfile {
            id: id.to_string(),
            vendor_id: 0x1234,
            product_id: 0x5678,
            name: Some(format!("Profile {}", id)),
            virtual_layout_id: "layout-1".to_string(),
            wiring: HashMap::new(),
        }
    }

    fn test_keymap(id: &str) -> Keymap {
        Keymap {
            id: id.to_string(),
            name: format!("Keymap {}", id),
            virtual_layout_id: "layout-1".to_string(),
            layers: vec![],
        }
    }

    // MockDeviceService tests
    #[tokio::test]
    async fn test_mock_device_service_list_devices() {
        let devices = vec![test_device("1234:5678:test")];
        let mock = MockDeviceService::new().with_devices(devices.clone());

        let result = mock.list_devices().await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].key, "1234:5678:test");
        assert_eq!(mock.get_call_count("list_devices"), 1);
    }

    #[tokio::test]
    async fn test_mock_device_service_list_error() {
        let mock = MockDeviceService::new()
            .with_list_error(DeviceServiceError::Io(std::io::Error::other("test error")));

        let result = mock.list_devices().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_device_service_get_device() {
        let devices = vec![test_device("1234:5678:test")];
        let mock = MockDeviceService::new().with_devices(devices);

        let result = mock.get_device("1234:5678:test").await.unwrap();
        assert_eq!(result.key, "1234:5678:test");
        assert_eq!(mock.get_call_count("get_device"), 1);
    }

    #[tokio::test]
    async fn test_mock_device_service_get_device_not_found() {
        let mock = MockDeviceService::new();

        let result = mock.get_device("unknown").await;
        assert!(matches!(result, Err(DeviceServiceError::DeviceNotFound(_))));
    }

    #[tokio::test]
    async fn test_mock_device_service_call_tracking() {
        let mock = MockDeviceService::new().with_devices(vec![test_device("key")]);

        let _ = mock.list_devices().await;
        let _ = mock.list_devices().await;
        let _ = mock.get_device("key").await;

        assert_eq!(mock.get_call_count("list_devices"), 2);
        assert_eq!(mock.get_call_count("get_device"), 1);
        assert_eq!(mock.get_call_count("set_remap_enabled"), 0);
    }

    // MockProfileService tests
    #[test]
    fn test_mock_profile_service_list_virtual_layouts() {
        let layouts = vec![test_layout("1"), test_layout("2")];
        let mock = MockProfileService::new().with_virtual_layouts(layouts);

        let result = mock.list_virtual_layouts().unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(mock.get_call_count("list_virtual_layouts"), 1);
    }

    #[test]
    fn test_mock_profile_service_save_virtual_layout() {
        let mock = MockProfileService::new();

        let layout = test_layout("new");
        let result = mock.save_virtual_layout(layout.clone()).unwrap();
        assert_eq!(result.id, "new");
        assert_eq!(mock.get_call_count("save_virtual_layout"), 1);

        // Verify it was stored
        let layouts = mock.list_virtual_layouts().unwrap();
        assert_eq!(layouts.len(), 1);
    }

    #[test]
    fn test_mock_profile_service_delete_virtual_layout() {
        let layouts = vec![test_layout("1"), test_layout("2")];
        let mock = MockProfileService::new().with_virtual_layouts(layouts);

        mock.delete_virtual_layout("1").unwrap();
        assert_eq!(mock.get_call_count("delete_virtual_layout"), 1);

        let remaining = mock.list_virtual_layouts().unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id, "2");
    }

    #[test]
    fn test_mock_profile_service_hardware_profiles() {
        let profiles = vec![test_hardware_profile("hp1")];
        let mock = MockProfileService::new().with_hardware_profiles(profiles);

        let result = mock.list_hardware_profiles().unwrap();
        assert_eq!(result.len(), 1);

        let new_profile = test_hardware_profile("hp2");
        mock.save_hardware_profile(new_profile).unwrap();

        let result = mock.list_hardware_profiles().unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_mock_profile_service_keymaps() {
        let keymaps = vec![test_keymap("km1")];
        let mock = MockProfileService::new().with_keymaps(keymaps);

        let result = mock.list_keymaps().unwrap();
        assert_eq!(result.len(), 1);

        mock.delete_keymap("km1").unwrap();
        let result = mock.list_keymaps().unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_mock_profile_service_error_handling() {
        let mock = MockProfileService::new().with_list_layouts_error("storage failure");

        let result = mock.list_virtual_layouts();
        assert!(result.is_err());
    }

    #[test]
    fn test_mock_profile_service_call_tracking() {
        let mock = MockProfileService::new();

        let _ = mock.list_virtual_layouts();
        let _ = mock.list_virtual_layouts();
        let _ = mock.list_hardware_profiles();
        let _ = mock.save_keymap(test_keymap("1"));

        assert_eq!(mock.get_call_count("list_virtual_layouts"), 2);
        assert_eq!(mock.get_call_count("list_hardware_profiles"), 1);
        assert_eq!(mock.get_call_count("save_keymap"), 1);
        assert_eq!(mock.get_call_count("delete_keymap"), 0);
    }

    // MockRuntimeService tests
    fn test_device_id() -> DeviceInstanceId {
        DeviceInstanceId {
            vendor_id: 0x1234,
            product_id: 0x5678,
            serial: Some("test-serial".to_string()),
        }
    }

    fn test_slot(id: &str, priority: u32) -> ProfileSlot {
        ProfileSlot {
            id: id.to_string(),
            hardware_profile_id: "hw-profile-1".to_string(),
            keymap_id: "keymap-1".to_string(),
            active: true,
            priority,
        }
    }

    #[test]
    fn test_mock_runtime_service_get_config() {
        let mock = MockRuntimeService::new();

        let result = mock.get_config().unwrap();
        assert!(result.devices.is_empty());
        assert_eq!(mock.get_call_count("get_config"), 1);
    }

    #[test]
    fn test_mock_runtime_service_add_slot() {
        let mock = MockRuntimeService::new();
        let device = test_device_id();
        let slot = test_slot("slot-1", 100);

        let result = mock.add_slot(device.clone(), slot).unwrap();
        assert_eq!(result.devices.len(), 1);
        assert_eq!(result.devices[0].slots.len(), 1);
        assert_eq!(result.devices[0].slots[0].id, "slot-1");
        assert_eq!(mock.get_call_count("add_slot"), 1);
    }

    #[test]
    fn test_mock_runtime_service_add_slot_sorts_by_priority() {
        let mock = MockRuntimeService::new();
        let device = test_device_id();

        // Add slots in reverse priority order
        mock.add_slot(device.clone(), test_slot("low", 10)).unwrap();
        mock.add_slot(device.clone(), test_slot("high", 100))
            .unwrap();
        mock.add_slot(device.clone(), test_slot("mid", 50)).unwrap();

        let config = mock.get_config().unwrap();
        let slots = &config.devices[0].slots;

        // Should be sorted descending by priority
        assert_eq!(slots[0].id, "high");
        assert_eq!(slots[1].id, "mid");
        assert_eq!(slots[2].id, "low");
    }

    #[test]
    fn test_mock_runtime_service_remove_slot() {
        let mock = MockRuntimeService::new();
        let device = test_device_id();

        mock.add_slot(device.clone(), test_slot("slot-1", 100))
            .unwrap();
        mock.add_slot(device.clone(), test_slot("slot-2", 50))
            .unwrap();

        let result = mock.remove_slot(device.clone(), "slot-1").unwrap();
        assert_eq!(result.devices[0].slots.len(), 1);
        assert_eq!(result.devices[0].slots[0].id, "slot-2");
        assert_eq!(mock.get_call_count("remove_slot"), 1);
    }

    #[test]
    fn test_mock_runtime_service_remove_slot_not_found() {
        let mock = MockRuntimeService::new();
        let device = test_device_id();

        mock.add_slot(device.clone(), test_slot("slot-1", 100))
            .unwrap();

        let result = mock.remove_slot(device.clone(), "nonexistent");
        assert!(matches!(result, Err(RuntimeServiceError::SlotNotFound(_))));
    }

    #[test]
    fn test_mock_runtime_service_remove_slot_device_not_found() {
        let mock = MockRuntimeService::new();
        let device = test_device_id();

        let result = mock.remove_slot(device, "slot-1");
        assert!(matches!(
            result,
            Err(RuntimeServiceError::DeviceNotFound(_))
        ));
    }

    #[test]
    fn test_mock_runtime_service_reorder_slot() {
        let mock = MockRuntimeService::new();
        let device = test_device_id();

        mock.add_slot(device.clone(), test_slot("slot-1", 100))
            .unwrap();
        mock.add_slot(device.clone(), test_slot("slot-2", 50))
            .unwrap();

        // Reorder slot-2 to highest priority
        let result = mock.reorder_slot(device.clone(), "slot-2", 200).unwrap();
        assert_eq!(result.devices[0].slots[0].id, "slot-2");
        assert_eq!(result.devices[0].slots[0].priority, 200);
        assert_eq!(mock.get_call_count("reorder_slot"), 1);
    }

    #[test]
    fn test_mock_runtime_service_set_slot_active() {
        let mock = MockRuntimeService::new();
        let device = test_device_id();

        mock.add_slot(device.clone(), test_slot("slot-1", 100))
            .unwrap();

        let result = mock
            .set_slot_active(device.clone(), "slot-1", false)
            .unwrap();
        assert!(!result.devices[0].slots[0].active);
        assert_eq!(mock.get_call_count("set_slot_active"), 1);

        // Toggle back
        let result = mock
            .set_slot_active(device.clone(), "slot-1", true)
            .unwrap();
        assert!(result.devices[0].slots[0].active);
    }

    #[test]
    fn test_mock_runtime_service_error_handling() {
        let mock = MockRuntimeService::new().with_get_config_error("storage failure");

        let result = mock.get_config();
        assert!(result.is_err());
    }

    #[test]
    fn test_mock_runtime_service_call_tracking() {
        let mock = MockRuntimeService::new();
        let device = test_device_id();

        let _ = mock.get_config();
        let _ = mock.get_config();
        let _ = mock.add_slot(device.clone(), test_slot("1", 100));
        let _ = mock.remove_slot(device.clone(), "1");

        assert_eq!(mock.get_call_count("get_config"), 2);
        assert_eq!(mock.get_call_count("add_slot"), 1);
        assert_eq!(mock.get_call_count("remove_slot"), 1);
        assert_eq!(mock.get_call_count("reorder_slot"), 0);
    }
}
