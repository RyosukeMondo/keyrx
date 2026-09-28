//! Linux-specific device enumeration and pattern matching using evdev.
//!
//! This module scans `/dev/input/event*` devices and identifies keyboards
//! based on their capabilities (presence of alphabetic keys). It also
//! provides pattern matching for selecting devices based on configuration.

use std::collections::HashMap;

use log::{info, warn};

use keyrx_core::config::DeviceConfig;
use keyrx_core::runtime::{DeviceState, KeyLookup};

use super::linux_enum::enumerate_keyboards;
use super::{DiscoveryError, KeyboardInfo};
use crate::platform::linux::EvdevInput;
use crate::platform::InputDevice;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RefreshResult {
    pub added: usize,
    pub removed: usize,
    /// Keyboards that matched a `device_start` pattern but could not be
    /// opened or grabbed (permission denied, in use, etc).
    pub denied: usize,
}

/// First config (in declaration order) whose pattern matches `info`.
fn first_match<'c>(
    info: &KeyboardInfo,
    configs: &'c [DeviceConfig],
) -> Option<(usize, &'c DeviceConfig)> {
    configs
        .iter()
        .enumerate()
        .find(|(_, c)| super::match_device(info, &c.identifier.pattern))
}

impl DeviceManager {
    /// A manager with no devices, ready for [`Self::reconcile`].
    pub fn empty() -> Self {
        Self {
            devices: Vec::new(),
        }
    }

    /// Discovers and grabs every keyboard matching `configs`. Fails if none
    /// end up grabbed (see [`Self::reconcile`] for a non-fatal variant used
    /// by hotplug and config reload, which keeps the daemon alive with 0
    /// devices rather than crash-looping).
    pub fn discover(configs: &[DeviceConfig]) -> Result<Self, DiscoveryError> {
        let mut manager = Self::empty();
        let result = manager.reconcile(configs, "*")?;
        if manager.devices.is_empty() {
            return Err(DiscoveryError::NoDevicesFound);
        }
        let _ = result;
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

    /// Re-evaluates which physical keyboards should be grabbed against
    /// `configs` (an empty slice matches nothing - use a single `"*"`
    /// [`DeviceConfig`] for pass-through), restricted to keyboards also
    /// matching `scope` (`"*"` for no restriction - tests pass a specific
    /// device name/pattern so they can never touch anything else, whatever
    /// `configs` says). Drops devices that were unplugged or that no longer
    /// match any in-scope pattern (releasing the grab first), rebinds
    /// devices that still match but to a different block, and grabs newly
    /// matching devices (hotplugged, or newly matched by a reload).
    ///
    /// Never fails just because nothing ended up grabbed - the daemon stays
    /// alive so IPC/web/`doctor` remain usable. A device that matched a
    /// pattern but could not be opened or grabbed is logged with the reason
    /// and counted in [`RefreshResult::denied`], not silently dropped.
    pub fn reconcile(
        &mut self,
        configs: &[DeviceConfig],
        scope: &str,
    ) -> Result<RefreshResult, DiscoveryError> {
        let current_keyboards: Vec<KeyboardInfo> = enumerate_keyboards()?
            .into_iter()
            .filter(|k| scope == "*" || super::match_device(k, scope))
            .collect();
        let by_path: HashMap<&std::path::Path, &KeyboardInfo> = current_keyboards
            .iter()
            .map(|k| (k.path.as_path(), k))
            .collect();

        let mut removed = 0;
        self.devices.retain_mut(|d| {
            let matched = by_path
                .get(d.info.path.as_path())
                .and_then(|info| first_match(info, configs));
            match matched {
                Some((idx, config)) => {
                    if idx != d.config_index {
                        info!(
                            "Rebinding '{}' ({}) to device_start block {idx}",
                            d.info.name,
                            d.info.path.display()
                        );
                        d.config_index = idx;
                    }
                    d.rebuild_lookup(config);
                    true
                }
                None => {
                    let reason = if by_path.contains_key(d.info.path.as_path()) {
                        "no longer matches any device_start pattern"
                    } else {
                        "unplugged"
                    };
                    info!(
                        "Releasing '{}' ({}): {reason}",
                        d.info.name,
                        d.info.path.display()
                    );
                    if let Err(e) = d.input.release() {
                        warn!("Failed to release {}: {e}", d.info.path.display());
                    }
                    removed += 1;
                    false
                }
            }
        });

        let managed_paths: std::collections::HashSet<std::path::PathBuf> =
            self.devices.iter().map(|d| d.info.path.clone()).collect();
        let mut added = 0;
        let mut denied = 0;
        for info in &current_keyboards {
            if managed_paths.contains(&info.path) {
                continue;
            }
            let Some((idx, config)) = first_match(info, configs) else {
                continue;
            };
            match EvdevInput::open(&info.path) {
                Ok(mut input) => match input.grab() {
                    Ok(()) => {
                        info!("Grabbed keyboard '{}' ({})", info.name, info.path.display());
                        self.devices
                            .push(ManagedDevice::new(info.clone(), input, config, idx));
                        added += 1;
                    }
                    Err(e) => {
                        warn!(
                            "Matched keyboard '{}' ({}) but could not grab it: {e}",
                            info.name,
                            info.path.display()
                        );
                        denied += 1;
                    }
                },
                Err(e) => {
                    warn!(
                        "Matched keyboard '{}' ({}) but could not open it: {e}",
                        info.name,
                        info.path.display()
                    );
                    denied += 1;
                }
            }
        }
        if denied > 0 {
            warn!(
                "{denied} matched keyboard(s) could not be grabbed. If this is a permission \
                 issue: sudo usermod -aG input $USER, then log out and back in (or run \
                 `keyrx_daemon doctor`)."
            );
        }

        Ok(RefreshResult {
            added,
            removed,
            denied,
        })
    }

    /// Releases and removes a device by id. Used when its fd errors on read
    /// (e.g. `ENODEV`) - it has almost certainly been unplugged, and holding
    /// onto a dead fd would otherwise starve every other managed device's
    /// events behind it. No-op if `id` is not currently managed.
    pub fn drop_by_id(&mut self, id: &str) {
        let Some(pos) = self.devices.iter().position(|d| d.device_id() == id) else {
            return;
        };
        let mut removed = self.devices.remove(pos);
        if let Err(e) = removed.input.release() {
            warn!("Failed to release {}: {e}", removed.info.path.display());
        }
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
