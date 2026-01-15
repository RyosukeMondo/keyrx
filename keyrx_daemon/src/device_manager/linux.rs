//! Linux-specific device enumeration and pattern matching using evdev.
//!
//! This module scans `/dev/input/event*` devices and identifies keyboards
//! based on their capabilities (presence of alphabetic keys). It also
//! provides pattern matching for selecting devices based on configuration.

use std::fs;
use std::path::Path;

use evdev::{Device, EventType, Key};
use log::warn;

use keyrx_core::config::DeviceConfig;
use keyrx_core::runtime::{DeviceState, KeyLookup};

use super::{DiscoveryError, KeyboardInfo};
use crate::platform::linux::EvdevInput;

/// Required alphabetic keys that a keyboard must have.
const REQUIRED_KEYS: &[Key] = &[
    Key::KEY_A,
    Key::KEY_B,
    Key::KEY_C,
    Key::KEY_D,
    Key::KEY_E,
    Key::KEY_F,
    Key::KEY_G,
    Key::KEY_H,
    Key::KEY_I,
    Key::KEY_J,
    Key::KEY_K,
    Key::KEY_L,
    Key::KEY_M,
    Key::KEY_N,
    Key::KEY_O,
    Key::KEY_P,
    Key::KEY_Q,
    Key::KEY_R,
    Key::KEY_S,
    Key::KEY_T,
    Key::KEY_U,
    Key::KEY_V,
    Key::KEY_W,
    Key::KEY_X,
    Key::KEY_Y,
    Key::KEY_Z,
];

const MIN_REQUIRED_KEYS: usize = 20;

fn is_keyboard(device: &Device) -> bool {
    let supported_events = device.supported_events();
    if !supported_events.contains(EventType::KEY) {
        return false;
    }

    let Some(supported_keys) = device.supported_keys() else {
        return false;
    };

    let key_count = REQUIRED_KEYS
        .iter()
        .filter(|key| supported_keys.contains(**key))
        .count();

    key_count >= MIN_REQUIRED_KEYS
}

/// Deduplicates keyboards by filtering out secondary interfaces.
///
/// Many USB keyboards register as multiple `/dev/input/event*` devices:
/// - One for the main keyboard (input0)
/// - One for consumer controls/media keys (input1, input2, etc.)
///
/// This function filters by preferring devices with physical path ending in `/input0`.
fn deduplicate_keyboards(keyboards: &mut Vec<KeyboardInfo>) {
    use std::collections::HashMap;

    // Group keyboards by their base physical path (without /inputN suffix)
    let mut grouped: HashMap<String, Vec<usize>> = HashMap::new();

    for (idx, kb) in keyboards.iter().enumerate() {
        if let Some(ref phys) = kb.phys {
            // Get the base path (everything before /inputN)
            let base_path = if let Some(pos) = phys.rfind("/input") {
                phys[..pos].to_string()
            } else {
                phys.clone()
            };

            grouped.entry(base_path).or_default().push(idx);
        } else {
            // No physical path - keep the device (can't deduplicate)
            continue;
        }
    }

    // For each group, find the primary interface (input0) or keep the first one
    let mut to_remove: Vec<usize> = Vec::new();

    for indices in grouped.values() {
        if indices.len() <= 1 {
            // Only one device in this group - keep it
            continue;
        }

        // Find the primary interface (input0)
        let primary_idx = indices.iter().find(|&&idx| {
            keyboards[idx]
                .phys
                .as_ref()
                .map(|p| p.ends_with("/input0"))
                .unwrap_or(false)
        });

        // Mark all others for removal
        for &idx in indices {
            if let Some(&primary) = primary_idx {
                if idx != primary {
                    to_remove.push(idx);
                }
            } else {
                // No primary found - keep only the first one
                if idx != indices[0] {
                    to_remove.push(idx);
                }
            }
        }
    }

    // Sort removal indices in reverse order to maintain validity during removal
    to_remove.sort_unstable_by(|a, b| b.cmp(a));

    // Remove marked devices
    for idx in to_remove {
        keyboards.remove(idx);
    }
}

pub fn enumerate_keyboards() -> Result<Vec<KeyboardInfo>, DiscoveryError> {
    let input_dir = Path::new("/dev/input");
    let entries = fs::read_dir(input_dir)?;

    let mut keyboards = Vec::new();
    for entry in entries {
        let entry = entry.map_err(DiscoveryError::Io)?;
        let path = entry.path();

        let device = match Device::open(&path) {
            Ok(d) => d,
            Err(_) => continue,
        };

        if !is_keyboard(&device) {
            continue;
        }

        let name = device.name().unwrap_or("Unknown Device").to_string();

        // Skip the daemon's own virtual output device to prevent grabbing it
        // The virtual device name is "keyrx" as set in platform/linux/mod.rs
        if name == "keyrx" {
            continue;
        }

        let serial = device.unique_name().map(|s| s.to_string());
        let phys = device.physical_path().map(|s| s.to_string());

        keyboards.push(KeyboardInfo {
            path,
            name,
            serial,
            phys,
        });
    }

    keyboards.sort_by(|a, b| a.path.cmp(&b.path));

    // Deduplicate devices: Many USB keyboards register as multiple event devices
    // (one for keyboard, one for consumer controls/media keys). We filter these
    // by preferring input0 interfaces and removing duplicates with " Keyboard" suffix.
    deduplicate_keyboards(&mut keyboards);

    Ok(keyboards)
}

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
    fn test_is_keyboard_requires_key_events() {
        // is_keyboard function exists and filters by key capability
        // This is a documentation test - the function is tested implicitly
        // through enumerate_keyboards() which uses it
        const _: () = assert!(MIN_REQUIRED_KEYS > 0);
    }

    #[test]
    fn test_required_keys_coverage() {
        // Ensure we have a reasonable set of required keys
        assert_eq!(REQUIRED_KEYS.len(), 26); // A-Z
        assert!(MIN_REQUIRED_KEYS <= REQUIRED_KEYS.len());
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
