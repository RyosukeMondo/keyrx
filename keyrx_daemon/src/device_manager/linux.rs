//! Linux-specific device enumeration and pattern matching using evdev.
//!
//! This module scans `/dev/input/event*` devices and identifies keyboards
//! based on their capabilities (presence of alphabetic keys). It also
//! provides pattern matching for selecting devices based on configuration.

use std::fs;
use std::path::Path;

use evdev::{Device, EventType, Key};

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
}
