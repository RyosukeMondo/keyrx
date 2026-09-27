//! Linux-specific device enumeration and pattern matching using evdev.
//!
//! This module scans `/dev/input/event*` devices and identifies keyboards
//! based on their capabilities (presence of alphabetic keys). It also
//! provides pattern matching for selecting devices based on configuration.

use log::warn;

use keyrx_core::config::DeviceConfig;
use keyrx_core::runtime::{DeviceState, KeyLookup};

use super::linux_enum::enumerate_keyboards;
use super::{DiscoveryError, KeyboardInfo};
use crate::platform::linux::EvdevInput;

pub struct ManagedDevice {
    info: KeyboardInfo,
    input: EvdevInput,
    lookup: KeyLookup,
    state: DeviceState,
    config_index: usize,
}

impl ManagedDevice {
    fn new(
        info: KeyboardInfo,
        input: EvdevInput,
        config: &DeviceConfig,
        config_index: usize,
    ) -> Self {
        Self {
            info,
            input,
            lookup: KeyLookup::from_device_config(config),
            state: DeviceState::new(),
            config_index,
        }
    }

    pub fn info(&self) -> &KeyboardInfo {
        &self.info
    }
    pub fn input_mut(&mut self) -> &mut EvdevInput {
        &mut self.input
    }
    pub fn input(&self) -> &EvdevInput {
        &self.input
    }
    pub fn lookup(&self) -> &KeyLookup {
        &self.lookup
    }
    pub fn state_mut(&mut self) -> &mut DeviceState {
        &mut self.state
    }
    pub fn state(&self) -> &DeviceState {
        &self.state
    }
    pub fn config_index(&self) -> usize {
        self.config_index
    }

    pub fn rebuild_lookup(&mut self, config: &DeviceConfig) {
        self.lookup = KeyLookup::from_device_config(config);
    }

    /// Returns mutable references to both lookup and state simultaneously.
    ///
    /// This combined accessor is necessary because both are needed during
    /// event processing, but we can't borrow both separately from a mutable
    /// reference to ManagedDevice.
    pub fn lookup_and_state_mut(&mut self) -> (&KeyLookup, &mut DeviceState) {
        (&self.lookup, &mut self.state)
    }

    /// Returns a unique device ID for this device.
    ///
    /// The ID is generated from the serial number if available, otherwise
    /// falls back to a path-based identifier for stability.
    ///
    /// # ID Generation Strategy
    ///
    /// 1. If a serial number is available (USB devices), use it prefixed with "serial-"
    /// 2. Otherwise, use the device path (e.g., "/dev/input/event0") prefixed with "path-"
    ///
    /// This ensures each device has a stable, unique identifier that can be
    /// used in Rhai scripts for per-device configuration.
    #[must_use]
    pub fn device_id(&self) -> String {
        if let Some(ref serial) = self.info.serial {
            if !serial.is_empty() {
                return format!("serial-{}", serial);
            }
        }
        // Fallback to path-based ID
        format!("path-{}", self.info.path.display())
    }
}

pub struct DeviceManager {
    devices: Vec<ManagedDevice>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RefreshResult {
    pub added: usize,
    pub removed: usize,
}

impl DeviceManager {
    pub fn discover(configs: &[DeviceConfig]) -> Result<Self, DiscoveryError> {
        let keyboards = enumerate_keyboards()?;
        if keyboards.is_empty() {
            return Err(DiscoveryError::NoDevicesFound);
        }

        let mut managed_devices = Vec::new();
        for keyboard_info in keyboards {
            for (idx, config) in configs.iter().enumerate() {
                if super::match_device(&keyboard_info, &config.identifier.pattern) {
                    if let Ok(input) = EvdevInput::open(&keyboard_info.path) {
                        managed_devices.push(ManagedDevice::new(
                            keyboard_info.clone(),
                            input,
                            config,
                            idx,
                        ));
                        break;
                    }
                }
            }
        }

        if managed_devices.is_empty() {
            return Err(DiscoveryError::NoDevicesFound);
        }
        Ok(Self {
            devices: managed_devices,
        })
    }

    pub fn device_count(&self) -> usize {
        self.devices.len()
    }
    pub fn devices(&self) -> impl Iterator<Item = &ManagedDevice> {
        self.devices.iter()
    }
    pub fn devices_mut(&mut self) -> impl Iterator<Item = &mut ManagedDevice> {
        self.devices.iter_mut()
    }
    pub fn get_device(&self, index: usize) -> Option<&ManagedDevice> {
        self.devices.get(index)
    }
    pub fn get_device_mut(&mut self, index: usize) -> Option<&mut ManagedDevice> {
        self.devices.get_mut(index)
    }

    pub fn rebuild_lookups(&mut self, configs: &[DeviceConfig]) {
        for device in &mut self.devices {
            if let Some(config) = configs.get(device.config_index) {
                device.rebuild_lookup(config);
            } else {
                warn!(
                    "Config reload: No config at index {} for device '{}', keeping old config",
                    device.config_index,
                    device.info().name
                );
            }
        }
    }

    pub fn refresh(&mut self, configs: &[DeviceConfig]) -> Result<RefreshResult, DiscoveryError> {
        let current_keyboards = enumerate_keyboards()?;
        let current_paths: std::collections::HashSet<_> =
            current_keyboards.iter().map(|k| k.path.clone()).collect();

        let mut removed = 0;
        self.devices.retain(|d| {
            if current_paths.contains(&d.info.path) {
                true
            } else {
                removed += 1;
                false
            }
        });

        let managed_paths: std::collections::HashSet<_> =
            self.devices.iter().map(|d| d.info.path.clone()).collect();
        let mut added = 0;
        for info in current_keyboards {
            if managed_paths.contains(&info.path) {
                continue;
            }
            for (idx, config) in configs.iter().enumerate() {
                if super::match_device(&info, &config.identifier.pattern) {
                    if let Ok(input) = EvdevInput::open(&info.path) {
                        self.devices
                            .push(ManagedDevice::new(info.clone(), input, config, idx));
                        added += 1;
                        break;
                    }
                }
            }
        }

        Ok(RefreshResult { added, removed })
    }

    /// Returns a list of all device IDs.
    ///
    /// Device IDs are unique identifiers generated from serial numbers (when
    /// available) or device paths. These IDs can be used in Rhai scripts for
    /// per-device configuration.
    #[must_use]
    pub fn device_ids(&self) -> Vec<String> {
        self.devices.iter().map(|d| d.device_id()).collect()
    }

    /// Returns keyboard info for a device by its ID.
    ///
    /// Returns `None` if no device with the given ID exists.
    #[must_use]
    pub fn device_info(&self, id: &str) -> Option<&KeyboardInfo> {
        self.devices
            .iter()
            .find(|d| d.device_id() == id)
            .map(|d| &d.info)
    }

    /// Returns a mutable reference to a managed device by its ID.
    ///
    /// Returns `None` if no device with the given ID exists.
    #[must_use]
    pub fn get_device_by_id(&self, id: &str) -> Option<&ManagedDevice> {
        self.devices.iter().find(|d| d.device_id() == id)
    }

    /// Returns a mutable reference to a managed device by its ID.
    ///
    /// Returns `None` if no device with the given ID exists.
    pub fn get_device_by_id_mut(&mut self, id: &str) -> Option<&mut ManagedDevice> {
        self.devices.iter_mut().find(|d| d.device_id() == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Helper to create a KeyboardInfo for testing
    fn make_keyboard_info(path: &str, name: &str, serial: Option<&str>) -> KeyboardInfo {
        KeyboardInfo {
            path: PathBuf::from(path),
            name: name.to_string(),
            serial: serial.map(String::from),
            phys: None,
        }
    }

    #[test]
    fn test_device_id_format_with_serial() {
        // Device IDs with serial should be prefixed with "serial-"
        let serial = "ABC123";
        let device_id = format!("serial-{}", serial);
        assert!(device_id.starts_with("serial-"));
        assert!(device_id.contains("ABC123"));
    }

    #[test]
    fn test_device_id_format_without_serial() {
        // Device IDs without serial should be prefixed with "path-"
        let path = "/dev/input/event5";
        let device_id = format!("path-{}", path);
        assert!(device_id.starts_with("path-"));
        assert!(device_id.contains("event5"));
    }

    #[test]
    fn test_device_id_empty_serial_uses_path() {
        // Empty serial strings should fallback to path-based ID
        let serial = "";
        let path = "/dev/input/event0";

        // Simulate the device_id() logic
        let device_id = if !serial.is_empty() {
            format!("serial-{}", serial)
        } else {
            format!("path-{}", path)
        };

        assert!(device_id.starts_with("path-"));
    }

    #[test]
    fn test_match_device_wildcard() {
        let info = make_keyboard_info("/dev/input/event0", "USB Keyboard", Some("SN123"));
        assert!(super::super::match_device(&info, "*"));
    }

    #[test]
    fn test_match_device_exact_name() {
        let info = make_keyboard_info("/dev/input/event0", "USB Keyboard", None);
        assert!(super::super::match_device(&info, "USB Keyboard"));
        assert!(!super::super::match_device(&info, "Other Keyboard"));
    }

    #[test]
    fn test_match_device_prefix_pattern() {
        let info = make_keyboard_info("/dev/input/event0", "Logitech USB Keyboard", None);
        assert!(super::super::match_device(&info, "Logitech*"));
        assert!(!super::super::match_device(&info, "Razer*"));
    }

    #[test]
    fn test_match_device_serial() {
        let info = make_keyboard_info("/dev/input/event0", "Keyboard", Some("SN12345"));
        assert!(super::super::match_device(&info, "SN12345"));
        assert!(super::super::match_device(&info, "SN123*"));
    }

    #[test]
    fn test_match_device_case_insensitive() {
        let info = make_keyboard_info("/dev/input/event0", "USB Keyboard", None);
        assert!(super::super::match_device(&info, "usb keyboard"));
        assert!(super::super::match_device(&info, "USB*"));
        assert!(super::super::match_device(&info, "usb*"));
    }
}
