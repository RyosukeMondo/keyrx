use super::{DiscoveryError, KeyboardInfo};
use crate::platform::windows::{
    device_map::DeviceMap, rawinput::RawInputManager, WindowsKeyboardInput,
};
use keyrx_core::config::DeviceConfig;
use keyrx_core::runtime::{DeviceState, KeyLookup};
use log::{info, warn};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub fn enumerate_keyboards() -> Result<Vec<KeyboardInfo>, DiscoveryError> {
    let device_map = DeviceMap::new();
    device_map
        .enumerate()
        .map_err(|e| DiscoveryError::Io(std::io::Error::other(e)))?;

    let devices = device_map
        .all()
        .into_iter()
        .map(|d| {
            // Use serial if available, otherwise path as fallback
            let serial = d.serial.clone();

            // Improve name resolution:
            // 1. If serial exists, use it as a primary identifier in the name.
            // 2. Attempt to parse VID/PID from path for a more technical name if no friendly name is available.
            let name = if let Some(ref s) = serial {
                if s.contains('&') && s.len() > 10 {
                    // This looks like a generated instance ID, not a real serial.
                    // Try to extract a shorter version or just use "Keyboard" with partial ID.
                    format!("Keyboard ({})", &s[..std::cmp::min(s.len(), 8)])
                } else {
                    format!("Keyboard ({})", s)
                }
            } else {
                // Try to extract VID/PID from path: \\?\HID#VID_046D&PID_C52B...
                if let Some(vid_idx) = d.path.find("VID_") {
                    if let Some(pid_idx) = d.path.find("PID_") {
                        let vid = &d.path[vid_idx..vid_idx + 8];
                        let pid = &d.path[pid_idx..pid_idx + 8];
                        format!("Keyboard {} {}", vid, pid)
                    } else {
                        format!("Keyboard {:x}", d.handle)
                    }
                } else {
                    format!("Keyboard {:x}", d.handle)
                }
            };

            KeyboardInfo {
                path: PathBuf::from(d.path),
                name,
                serial,
                phys: None,
            }
        })
        .collect();

    Ok(devices)
}

pub struct ManagedDevice {
    info: KeyboardInfo,
    input: WindowsKeyboardInput,
    lookup: KeyLookup,
    state: DeviceState,
    config_index: usize,
    device_handle: usize, // Keep track to unsubscribe
}

impl ManagedDevice {
    pub fn new(
        info: KeyboardInfo,
        input: WindowsKeyboardInput,
        config: &DeviceConfig,
        config_index: usize,
        device_handle: usize,
    ) -> Self {
        Self {
            info,
            input,
            lookup: KeyLookup::from_device_config(config),
            state: DeviceState::new(),
            config_index,
            device_handle,
        }
    }

    pub fn info(&self) -> &KeyboardInfo {
        &self.info
    }

    pub fn input_mut(&mut self) -> &mut WindowsKeyboardInput {
        &mut self.input
    }

    pub fn input(&self) -> &WindowsKeyboardInput {
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

    pub fn lookup_and_state_mut(&mut self) -> (&KeyLookup, &mut DeviceState) {
        (&self.lookup, &mut self.state)
    }

    /// Returns a unique device ID for this device.
    #[must_use]
    pub fn device_id(&self) -> String {
        self.info.device_id()
    }
}

pub struct DeviceManager {
    devices: Vec<ManagedDevice>,
    device_map: DeviceMap,
    raw_input_manager: RawInputManager,
}

pub struct RefreshResult {
    pub added: usize,
    pub removed: usize,
}

impl DeviceManager {
    pub fn discover(configs: &[DeviceConfig]) -> Result<Self, DiscoveryError> {
        info!("Initializing Windows Raw Input Device Manager");

        let device_map = DeviceMap::new();
        // RawInputManager registers for WM_INPUT upon creation
        // We pass a dummy sender since DeviceManager uses per-device subscriptions
        let (sender, _) = crossbeam_channel::unbounded();
        let bridge_context = Arc::new(Mutex::new(None));
        let bridge_hook = Arc::new(Mutex::new(None));
        let raw_input_manager =
            RawInputManager::new(device_map.clone(), sender, bridge_context, bridge_hook)
                .map_err(|e| DiscoveryError::Io(std::io::Error::other(e)))?;

        // Populate initial device list
        device_map
            .enumerate()
            .map_err(|e| DiscoveryError::Io(std::io::Error::other(e)))?;

        let mut manager = Self {
            devices: Vec::new(),
            device_map,
            raw_input_manager,
        };

        manager.refresh(configs)?;

        Ok(manager)
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
        let mut added = 0;
        let mut removed = 0;

        let detected_devices = self.device_map.all();

        // 1. Identify removed devices
        // A device is removed if its handle is no longer in the map's current list.
        // Wait, device_map handles might be reused? Windows handles are pointers or similar.
        // Using handle as identity is standard for Raw Input session.

        let current_handles: Vec<usize> = detected_devices.iter().map(|d| d.handle).collect();

        let mut retained_indices = Vec::new();
        let mut devices_to_drop = Vec::new();

        for (i, device) in self.devices.iter().enumerate() {
            if current_handles.contains(&device.device_handle) {
                retained_indices.push(i);
            } else {
                devices_to_drop.push(i);
            }
        }

        // Process removals
        // We iterate backwards to remove safely if modifying in place, but here we reconstruct or retain.
        // Let's filter in place.
        let raw_input = &self.raw_input_manager;
        self.devices.retain(|d| {
            if current_handles.contains(&d.device_handle) {
                true
            } else {
                info!("Device removed: {} ({:?})", d.info.name, d.info.path);
                raw_input.unsubscribe(d.device_handle);
                removed += 1;
                false
            }
        });

        // 2. Identify new devices
        // A device is new if no ManagedDevice has its handle.
        let existing_handles: Vec<usize> = self.devices.iter().map(|d| d.device_handle).collect();

        for device_info in detected_devices {
            let handle = device_info.handle;
            if existing_handles.contains(&handle) {
                continue;
            }

            // Convert DeviceInfo to KeyboardInfo used by linker
            let keyboard_info = KeyboardInfo {
                path: PathBuf::from(&device_info.path), // Use path as unique ID usually, but here path string
                name: format!("Device {:x}", handle), // We might want better name if API gives it? currently path is \\?\...
                serial: device_info.serial.clone(),
                phys: None,
            };

            // Attempt to match
            let mut matched_config = None;
            for (idx, config) in configs.iter().enumerate() {
                if super::match_device(&keyboard_info, &config.identifier.pattern) {
                    matched_config = Some((idx, config));
                    break;
                }
            }

            if let Some((config_idx, config)) = matched_config {
                info!(
                    "Device matched: {} -> {}",
                    keyboard_info.path.display(),
                    config.identifier.pattern
                );

                // create subscription
                let receiver = self.raw_input_manager.subscribe(handle);
                let input = WindowsKeyboardInput::new(receiver);

                let managed = ManagedDevice::new(keyboard_info, input, config, config_idx, handle);

                self.devices.push(managed);
                added += 1;
            } else {
                // info!("Device ignored (no match): {:?}", device_info.path);
            }
        }

        Ok(RefreshResult { added, removed })
    }

    /// Returns a list of all device IDs.
    #[must_use]
    pub fn device_ids(&self) -> Vec<String> {
        self.devices.iter().map(|d| d.device_id()).collect()
    }

    /// Returns keyboard info for a device by its ID.
    #[must_use]
    pub fn device_info(&self, id: &str) -> Option<&KeyboardInfo> {
        self.devices
            .iter()
            .find(|d| d.device_id() == id)
            .map(|d| &d.info)
    }

    /// Returns a reference to a managed device by its ID.
    #[must_use]
    pub fn get_device_by_id(&self, id: &str) -> Option<&ManagedDevice> {
        self.devices.iter().find(|d| d.device_id() == id)
    }

    /// Returns a mutable reference to a managed device by its ID.
    pub fn get_device_by_id_mut(&mut self, id: &str) -> Option<&mut ManagedDevice> {
        self.devices.iter_mut().find(|d| d.device_id() == id)
    }
}
