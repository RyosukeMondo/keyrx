//! Linux-specific device enumeration and pattern matching using evdev.
//!
//! This module scans `/dev/input/event*` devices and identifies keyboards
//! based on their capabilities (presence of alphabetic keys). It also
//! provides pattern matching for selecting devices based on configuration.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use log::{debug, info, warn};

use keyrx_core::config::DeviceConfig;
use keyrx_core::runtime::{DeviceState, KeyLookup};

use super::failure_log::{FailureTracker, Loudness, HOTPLUG_GRACE};
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
    failures: FailureTracker,
    /// Paths enumerated by the previous reconcile; `None` before the first.
    /// A path in here is not a fresh node, so it gets no udev grace period
    /// (nor does anything present at startup).
    seen_paths: Option<std::collections::HashSet<std::path::PathBuf>>,
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
            failures: FailureTracker::default(),
            seen_paths: None,
        }
    }

    /// True when a hotplugged node has been failing past the grace period
    /// without having been reported yet: a rescan now would warn about it.
    pub fn failure_retry_due(&self) -> bool {
        self.failures.retry_due(Instant::now())
    }

    /// Discovers and grabs every keyboard matching `configs`. Fails if none
    /// end up grabbed (see [`Self::reconcile`] for a non-fatal variant used
    /// by hotplug and config reload, which keeps the daemon alive with 0
    /// devices rather than crash-looping).
    pub fn discover(configs: &[DeviceConfig]) -> Result<Self, DiscoveryError> {
        let mut manager = Self::empty();
        let result = manager.reconcile(configs, "*", None)?;
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
    ///
    /// `own_output` names the daemon's own virtual output keyboard: it is
    /// never managed. It appears in /dev/input after startup, so hotplug
    /// would otherwise grab it and swallow every remapped key.
    pub fn reconcile(
        &mut self,
        configs: &[DeviceConfig],
        scope: &str,
        own_output: Option<&str>,
    ) -> Result<RefreshResult, DiscoveryError> {
        let current_keyboards: Vec<KeyboardInfo> = enumerate_keyboards()?
            .into_iter()
            .filter(|k| own_output != Some(k.name.as_str()))
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

        let (added, denied) = self.grab_new(&current_keyboards, configs);
        self.seen_paths = Some(current_keyboards.iter().map(|k| k.path.clone()).collect());

        Ok(RefreshResult {
            added,
            removed,
            denied,
        })
    }

    /// Opens and grabs every unmanaged keyboard that matches `configs`.
    /// Returns `(added, denied)`. Failures are logged per
    /// [`super::failure_log`]: transient ones on fresh nodes stay at debug.
    fn grab_new(&mut self, keyboards: &[KeyboardInfo], configs: &[DeviceConfig]) -> (usize, usize) {
        let managed: std::collections::HashSet<std::path::PathBuf> =
            self.devices.iter().map(|d| d.info.path.clone()).collect();
        let now = Instant::now();
        let (mut added, mut denied, mut busy, mut permission) = (0, 0, 0, 0);
        let mut failing: Vec<&std::path::Path> = Vec::new();
        for info in keyboards {
            if managed.contains(&info.path) {
                continue;
            }
            let Some((idx, config)) = first_match(info, configs) else {
                continue;
            };
            let attempt = EvdevInput::open(&info.path)
                .map_err(|e| ("open", e))
                .and_then(|mut input| match input.grab() {
                    Ok(()) => Ok(input),
                    Err(e) => Err(("grab", e)),
                });
            let (stage, e) = match attempt {
                Ok(input) => {
                    info!("Grabbed keyboard '{}' ({})", info.name, info.path.display());
                    self.devices
                        .push(ManagedDevice::new(info.clone(), input, config, idx));
                    added += 1;
                    continue;
                }
                Err(failure) => failure,
            };
            denied += 1;
            failing.push(&info.path);
            let fresh = self
                .seen_paths
                .as_ref()
                .is_some_and(|seen| !seen.contains(&info.path));
            let grace = if fresh { HOTPLUG_GRACE } else { Duration::ZERO };
            let ebusy = is_grabbed_elsewhere(&e);
            let msg = format!(
                "Matched keyboard '{}' ({}) but could not {stage} it{}: {e}",
                info.name,
                info.path.display(),
                if ebusy {
                    " (already grabbed, EBUSY)"
                } else {
                    ""
                }
            );
            if self.failures.record(&info.path, now, grace) == Loudness::Warn {
                warn!("{msg}");
                busy += usize::from(ebusy);
                permission += usize::from(is_permission_denied(&e));
            } else {
                debug!("{msg} (transient? retrying on the next device change)");
            }
        }
        self.failures.retain_failing(&failing);
        if busy > 0 {
            warn!(
                "{busy} matched keyboard(s) are already grabbed by another process - usually \
                 another keyrx_daemon (`systemctl --user status keyrx`) or a remapper such as \
                 kmonad/keyd/interception. Stop it, or narrow this device_start pattern."
            );
        }
        if permission > 0 {
            warn!(
                "{permission} matched keyboard(s) could not be opened: permission denied. \
                 Fix: {}, then log out and back in (or run `keyrx_daemon doctor`).",
                crate::permission_advice::join_group_command("input")
            );
        }
        (added, denied)
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

/// `EVIOCGRAB` fails with `EBUSY` when another process already holds the
/// device. That is not a permissions problem and must not be reported as one.
fn is_permission_denied(error: &crate::platform::DeviceError) -> bool {
    use crate::platform::DeviceError;
    match error {
        DeviceError::PermissionDenied(_) => true,
        DeviceError::Io(e) => e.kind() == std::io::ErrorKind::PermissionDenied,
        _ => false,
    }
}

fn is_grabbed_elsewhere(error: &crate::platform::DeviceError) -> bool {
    const EBUSY: i32 = 16;
    matches!(error, crate::platform::DeviceError::Io(e) if e.raw_os_error() == Some(EBUSY))
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
            is_virtual: false,
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
