//! Linux-specific device enumeration and pattern matching using evdev.
//!
//! This module scans `/dev/input/event*` devices and identifies keyboards
//! based on their capabilities (presence of alphabetic keys). It also
//! provides pattern matching for selecting devices based on configuration.
//!
//! # Device Management
//!
//! The [`DeviceManager`] struct orchestrates multi-device management:
//!
//! 1. Enumerate available keyboards via [`enumerate_keyboards`]
//! 2. Match each device against configuration patterns via [`match_device`]
//! 3. Create [`ManagedDevice`] instances for matched devices
//! 4. Provide access to managed devices for event processing
//!
//! # Example
//!
//! ```ignore
//! use keyrx_daemon::device_manager::{DeviceManager, enumerate_keyboards};
//! use keyrx_core::config::DeviceConfig;
//!
//! // Create device configurations
//! let configs = vec![/* ... */];
//!
//! // Discover and match devices
//! let manager = DeviceManager::discover(&configs)?;
//!
//! // Access managed devices
//! for device in manager.devices() {
//!     println!("Managing: {}", device.info().name);
//! }
//! ```

use std::fs;
use std::path::Path;

use evdev::{Device, EventType, Key};
use log::{debug, info, warn};

use keyrx_core::config::DeviceConfig;
use keyrx_core::runtime::{DeviceState, KeyLookup};

use super::{DiscoveryError, KeyboardInfo};
use crate::platform::linux::EvdevInput;

/// Required alphabetic keys that a keyboard must have.
/// If a device has EV_KEY capability and these keys, it's considered a keyboard.
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

/// Minimum number of required keys that must be present to consider a device a keyboard.
/// This threshold helps filter out devices that might have a few key events but aren't
/// full keyboards (like power buttons or multimedia remotes).
const MIN_REQUIRED_KEYS: usize = 20;

/// Checks if a device has keyboard capabilities.
///
/// A device is considered a keyboard if it:
/// 1. Supports the EV_KEY event type
/// 2. Has at least `MIN_REQUIRED_KEYS` of the required alphabetic keys
///
/// This filtering excludes mice, touchpads, power buttons, and other input
/// devices that may report some key events but are not keyboards.
fn is_keyboard(device: &Device) -> bool {
    // Must support key events
    let supported_events = device.supported_events();
    if !supported_events.contains(EventType::KEY) {
        return false;
    }

    // Check for alphabetic keys
    let Some(supported_keys) = device.supported_keys() else {
        return false;
    };

    let key_count = REQUIRED_KEYS
        .iter()
        .filter(|key| supported_keys.contains(**key))
        .count();

    key_count >= MIN_REQUIRED_KEYS
}

/// Enumerates all keyboard devices on the system.
///
/// This function scans `/dev/input/event*` devices and returns information
/// about each device that appears to be a keyboard (has EV_KEY capability
/// with alphabetic keys).
///
/// # Returns
///
/// * `Ok(Vec<KeyboardInfo>)` - List of discovered keyboard devices
/// * `Err(DiscoveryError::Io)` - Failed to read `/dev/input` directory
///
/// # Permissions
///
/// This function attempts to open each device to read its capabilities.
/// Devices that cannot be opened (due to permissions) are skipped with
/// a debug log message. To enumerate all devices, the user typically needs:
/// - Root access, OR
/// - Membership in the `input` group, OR
/// - Appropriate udev rules
///
/// # Example
///
/// ```no_run
/// use keyrx_daemon::device_manager::enumerate_keyboards;
///
/// match enumerate_keyboards() {
///     Ok(keyboards) => {
///         println!("Found {} keyboard(s):", keyboards.len());
///         for kb in keyboards {
///             println!("  {} ({})", kb.name, kb.path.display());
///         }
///     }
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn enumerate_keyboards() -> Result<Vec<KeyboardInfo>, DiscoveryError> {
    let input_dir = Path::new("/dev/input");

    // Read directory entries
    let entries = fs::read_dir(input_dir)?;

    let mut keyboards = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                debug!("Failed to read directory entry: {}", e);
                continue;
            }
        };

        let path = entry.path();

        // Only consider event* devices
        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) if name.starts_with("event") => name,
            _ => continue,
        };

        // Try to open the device
        let device = match Device::open(&path) {
            Ok(d) => d,
            Err(e) => {
                // Permission denied is common for devices not accessible to user
                debug!("Skipping {} ({}): {}", file_name, path.display(), e);
                continue;
            }
        };

        // Check if it's a keyboard
        if !is_keyboard(&device) {
            debug!("Skipping {} (not a keyboard)", file_name);
            continue;
        }

        // Collect device information
        let name = device.name().unwrap_or("Unknown Device").to_string();
        let serial = device
            .unique_name()
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty());
        let phys = device
            .physical_path()
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty());

        debug!("Found keyboard: {} at {}", name, path.display());

        keyboards.push(KeyboardInfo {
            path,
            name,
            serial,
            phys,
        });
    }

    // Sort by path for consistent ordering
    keyboards.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(keyboards)
}

/// Opens a device by path and returns it if it's a valid keyboard.
///
/// # Arguments
///
/// * `path` - Path to the device node (e.g., `/dev/input/event0`)
///
/// # Returns
///
/// * `Ok(Some(KeyboardInfo))` - Device is a keyboard
/// * `Ok(None)` - Device exists but is not a keyboard
/// * `Err(DiscoveryError)` - Failed to access device
#[allow(dead_code)] // Will be used in task #12 (DeviceManager)
pub fn open_keyboard(path: &Path) -> Result<Option<KeyboardInfo>, DiscoveryError> {
    let device = Device::open(path).map_err(|e| {
        let kind = e.kind();
        match kind {
            std::io::ErrorKind::NotFound => DiscoveryError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("device not found: {}", path.display()),
            )),
            std::io::ErrorKind::PermissionDenied => DiscoveryError::Io(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                format!("permission denied: {}", path.display()),
            )),
            _ => DiscoveryError::Io(e),
        }
    })?;

    if !is_keyboard(&device) {
        return Ok(None);
    }

    let name = device.name().unwrap_or("Unknown Device").to_string();
    let serial = device
        .unique_name()
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty());
    let phys = device
        .physical_path()
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty());

    Ok(Some(KeyboardInfo {
        path: path.to_path_buf(),
        name,
        serial,
        phys,
    }))
}

/// Matches a device against a pattern string.
///
/// This function checks if a device (identified by `KeyboardInfo`) matches
/// the given pattern. Patterns are matched against both the device name
/// and serial number (if available).
///
/// # Pattern Syntax
///
/// - `"*"` - Wildcard: matches any device
/// - `"prefix*"` - Prefix pattern: matches devices whose name or serial
///   starts with "prefix" (case-insensitive)
/// - `"exact"` - Exact match: matches devices whose name or serial
///   equals "exact" (case-insensitive)
///
/// # Arguments
///
/// * `device` - Information about the keyboard device to match
/// * `pattern` - Pattern string from configuration
///
/// # Returns
///
/// `true` if the device matches the pattern, `false` otherwise.
///
/// # Examples
///
/// ```ignore
/// use keyrx_daemon::device_manager::{KeyboardInfo, match_device};
/// use std::path::PathBuf;
///
/// let device = KeyboardInfo {
///     path: PathBuf::from("/dev/input/event0"),
///     name: "USB Keyboard".to_string(),
///     serial: Some("ABC123".to_string()),
///     phys: None,
/// };
///
/// // Wildcard matches all
/// assert!(match_device(&device, "*"));
///
/// // Prefix pattern
/// assert!(match_device(&device, "USB*"));
///
/// // Exact match
/// assert!(match_device(&device, "USB Keyboard"));
///
/// // Case-insensitive
/// assert!(match_device(&device, "usb keyboard"));
/// ```
pub fn match_device(device: &KeyboardInfo, pattern: &str) -> bool {
    // Wildcard pattern matches everything
    if pattern == "*" {
        return true;
    }

    // Check for prefix pattern (ends with *)
    if let Some(prefix) = pattern.strip_suffix('*') {
        let prefix_lower = prefix.to_lowercase();

        // Match against device name
        if device.name.to_lowercase().starts_with(&prefix_lower) {
            return true;
        }

        // Match against serial if available
        if let Some(ref serial) = device.serial {
            if serial.to_lowercase().starts_with(&prefix_lower) {
                return true;
            }
        }

        // Match against physical path if available
        if let Some(ref phys) = device.phys {
            if phys.to_lowercase().starts_with(&prefix_lower) {
                return true;
            }
        }

        return false;
    }

    // Exact match (case-insensitive)
    let pattern_lower = pattern.to_lowercase();

    // Match against device name
    if device.name.to_lowercase() == pattern_lower {
        return true;
    }

    // Match against serial if available
    if let Some(ref serial) = device.serial {
        if serial.to_lowercase() == pattern_lower {
            return true;
        }
    }

    // Match against physical path if available
    if let Some(ref phys) = device.phys {
        if phys.to_lowercase() == pattern_lower {
            return true;
        }
    }

    false
}

/// A managed device with its associated configuration and runtime state.
///
/// `ManagedDevice` bundles together:
/// - The device information ([`KeyboardInfo`])
/// - The evdev input device ([`EvdevInput`])
/// - The key lookup table ([`KeyLookup`]) built from the device's configuration
/// - The device state ([`DeviceState`]) for tracking modifiers and locks
///
/// This struct is created by [`DeviceManager::discover`] when a device matches
/// a configuration pattern.
///
/// # Example
///
/// ```ignore
/// let manager = DeviceManager::discover(&configs)?;
/// for device in manager.devices() {
///     println!("Device: {}", device.info().name);
///     // Process events through device.lookup() and device.state()
/// }
/// ```
pub struct ManagedDevice {
    /// Information about the keyboard device.
    info: KeyboardInfo,
    /// The evdev input device for reading events.
    input: EvdevInput,
    /// Key lookup table for mapping resolution.
    lookup: KeyLookup,
    /// Runtime state tracking modifiers and locks.
    state: DeviceState,
    /// Index of the matching configuration (for reference).
    config_index: usize,
}

impl ManagedDevice {
    /// Creates a new managed device.
    ///
    /// # Arguments
    ///
    /// * `info` - Information about the keyboard device
    /// * `input` - The evdev input device
    /// * `config` - The device configuration to use for lookup table
    /// * `config_index` - Index of the configuration in the original config list
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

    /// Returns information about the keyboard device.
    #[must_use]
    pub fn info(&self) -> &KeyboardInfo {
        &self.info
    }

    /// Returns a mutable reference to the evdev input device.
    pub fn input_mut(&mut self) -> &mut EvdevInput {
        &mut self.input
    }

    /// Returns a reference to the evdev input device.
    #[must_use]
    pub fn input(&self) -> &EvdevInput {
        &self.input
    }

    /// Returns the key lookup table for this device.
    #[must_use]
    pub fn lookup(&self) -> &KeyLookup {
        &self.lookup
    }

    /// Returns a mutable reference to the device state.
    pub fn state_mut(&mut self) -> &mut DeviceState {
        &mut self.state
    }

    /// Returns a reference to the device state.
    #[must_use]
    pub fn state(&self) -> &DeviceState {
        &self.state
    }

    /// Returns the index of the matching configuration.
    #[must_use]
    pub fn config_index(&self) -> usize {
        self.config_index
    }

    /// Rebuilds the lookup table from a new configuration.
    ///
    /// This is used during configuration reload to update mappings
    /// without recreating the device.
    pub fn rebuild_lookup(&mut self, config: &DeviceConfig) {
        self.lookup = KeyLookup::from_device_config(config);
    }
}

/// Manager for multiple keyboard devices with configuration matching.
///
/// `DeviceManager` discovers available keyboard devices and matches them
/// against provided configurations. It creates [`ManagedDevice`] instances
/// for each matched device and provides access to them for event processing.
///
/// # Discovery Process
///
/// 1. Enumerate all keyboard devices on the system
/// 2. For each device, iterate through configurations in order
/// 3. First matching configuration wins (priority order)
/// 4. Create a `ManagedDevice` for matched devices
/// 5. Unmatched devices are logged but not grabbed
///
/// # Example
///
/// ```ignore
/// use keyrx_daemon::device_manager::DeviceManager;
/// use keyrx_core::config::{DeviceConfig, DeviceIdentifier, KeyMapping, KeyCode};
///
/// // Create configurations
/// let configs = vec![
///     DeviceConfig {
///         identifier: DeviceIdentifier { pattern: "USB Keyboard".to_string() },
///         mappings: vec![/* ... */],
///     },
///     DeviceConfig {
///         identifier: DeviceIdentifier { pattern: "*".to_string() },
///         mappings: vec![/* ... */],
///     },
/// ];
///
/// // Discover and match devices
/// let manager = DeviceManager::discover(&configs)?;
///
/// println!("Managing {} device(s)", manager.device_count());
/// for device in manager.devices() {
///     println!("  - {}", device.info().name);
/// }
/// ```
pub struct DeviceManager {
    /// Managed devices with their configurations and state.
    devices: Vec<ManagedDevice>,
}

impl DeviceManager {
    /// Discovers keyboard devices and matches them against configurations.
    ///
    /// This method enumerates all available keyboard devices, attempts to match
    /// each against the provided configurations in priority order (first match wins),
    /// and creates managed devices for those that match.
    ///
    /// # Arguments
    ///
    /// * `configs` - Device configurations to match against, in priority order
    ///
    /// # Returns
    ///
    /// * `Ok(DeviceManager)` - Successfully discovered and matched at least one device
    /// * `Err(DiscoveryError::NoDevicesFound)` - No keyboard devices found on the system
    /// * `Err(DiscoveryError)` - If no devices match any configuration
    ///
    /// # Device Matching
    ///
    /// Devices are matched against configurations in the order provided.
    /// The first matching configuration for each device wins. This allows
    /// specific device configurations to take precedence over wildcards.
    ///
    /// Example configuration order:
    /// 1. "USB\\VID_04D9*" - Specific vendor match
    /// 2. "AT Translated*" - Laptop keyboard match
    /// 3. "*" - Catch-all for remaining keyboards
    ///
    /// # Example
    ///
    /// ```ignore
    /// let configs = vec![
    ///     DeviceConfig {
    ///         identifier: DeviceIdentifier { pattern: "*".to_string() },
    ///         mappings: vec![/* ... */],
    ///     },
    /// ];
    ///
    /// match DeviceManager::discover(&configs) {
    ///     Ok(manager) => {
    ///         println!("Managing {} devices", manager.device_count());
    ///     }
    ///     Err(DiscoveryError::NoDevicesFound) => {
    ///         eprintln!("No keyboards found");
    ///     }
    ///     Err(e) => {
    ///         eprintln!("Discovery error: {}", e);
    ///     }
    /// }
    /// ```
    pub fn discover(configs: &[DeviceConfig]) -> Result<Self, DiscoveryError> {
        // Enumerate available keyboards
        let keyboards = enumerate_keyboards()?;

        if keyboards.is_empty() {
            warn!("No keyboard devices found on the system");
            return Err(DiscoveryError::NoDevicesFound);
        }

        info!("Found {} keyboard device(s)", keyboards.len());

        let mut managed_devices = Vec::new();
        let mut unmatched_devices = Vec::new();

        // Try to match each keyboard against configurations
        for keyboard_info in keyboards {
            let mut matched = false;

            // Iterate through configs in priority order
            for (config_idx, config) in configs.iter().enumerate() {
                let pattern = &config.identifier.pattern;

                if match_device(&keyboard_info, pattern) {
                    info!(
                        "Device '{}' matches pattern '{}' (config #{})",
                        keyboard_info.name, pattern, config_idx
                    );

                    // Try to open the device
                    match EvdevInput::open(&keyboard_info.path) {
                        Ok(input) => {
                            let managed = ManagedDevice::new(
                                keyboard_info.clone(),
                                input,
                                config,
                                config_idx,
                            );
                            managed_devices.push(managed);
                            matched = true;
                            break; // First matching config wins
                        }
                        Err(e) => {
                            warn!("Failed to open device '{}': {}", keyboard_info.name, e);
                            // Continue to try other devices even if one fails
                        }
                    }
                }
            }

            if !matched {
                debug!(
                    "Device '{}' does not match any configuration pattern",
                    keyboard_info.name
                );
                unmatched_devices.push(keyboard_info);
            }
        }

        // Check if we matched any devices
        if managed_devices.is_empty() {
            // Build helpful error message with available devices
            let available: Vec<_> = unmatched_devices.iter().map(|d| d.name.as_str()).collect();

            warn!(
                "No devices matched any configuration. Available devices: {:?}",
                available
            );

            return Err(DiscoveryError::NoDevicesFound);
        }

        info!(
            "Successfully matched {} device(s), {} unmatched",
            managed_devices.len(),
            unmatched_devices.len()
        );

        Ok(Self {
            devices: managed_devices,
        })
    }

    /// Returns the number of managed devices.
    #[must_use]
    pub fn device_count(&self) -> usize {
        self.devices.len()
    }

    /// Returns an iterator over managed devices.
    pub fn devices(&self) -> impl Iterator<Item = &ManagedDevice> {
        self.devices.iter()
    }

    /// Returns a mutable iterator over managed devices.
    pub fn devices_mut(&mut self) -> impl Iterator<Item = &mut ManagedDevice> {
        self.devices.iter_mut()
    }

    /// Returns a reference to a specific managed device by index.
    #[must_use]
    pub fn get_device(&self, index: usize) -> Option<&ManagedDevice> {
        self.devices.get(index)
    }

    /// Returns a mutable reference to a specific managed device by index.
    pub fn get_device_mut(&mut self, index: usize) -> Option<&mut ManagedDevice> {
        self.devices.get_mut(index)
    }

    /// Rebuilds all lookup tables from the provided configurations.
    ///
    /// This is used during configuration reload to update mappings
    /// for all managed devices without recreating them.
    ///
    /// # Arguments
    ///
    /// * `configs` - The new device configurations
    ///
    /// # Note
    ///
    /// Each device's lookup table is rebuilt using the configuration
    /// at its stored `config_index`. If the configuration at that index
    /// has changed, the device will use the new mappings.
    pub fn rebuild_lookups(&mut self, configs: &[DeviceConfig]) {
        for device in &mut self.devices {
            if let Some(config) = configs.get(device.config_index) {
                device.rebuild_lookup(config);
                debug!(
                    "Rebuilt lookup table for '{}' using config #{}",
                    device.info.name, device.config_index
                );
            } else {
                warn!(
                    "Config index {} out of bounds for device '{}', keeping old lookup",
                    device.config_index, device.info.name
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_required_keys_constant() {
        // Verify we have all 26 letters
        assert_eq!(REQUIRED_KEYS.len(), 26);
    }

    #[test]
    fn test_min_required_keys() {
        // Threshold should be reasonable (most of the alphabet)
        assert!(MIN_REQUIRED_KEYS >= 20);
        assert!(MIN_REQUIRED_KEYS <= REQUIRED_KEYS.len());
    }

    // Integration tests that require real devices
    #[test]
    #[ignore = "Requires access to /dev/input devices"]
    fn test_enumerate_keyboards_real_devices() {
        let result = enumerate_keyboards();
        assert!(result.is_ok(), "Should not error on enumeration");

        let keyboards = result.unwrap();
        println!("Found {} keyboard(s):", keyboards.len());
        for kb in &keyboards {
            println!("  Name: {}", kb.name);
            println!("  Path: {}", kb.path.display());
            if let Some(ref serial) = kb.serial {
                println!("  Serial: {}", serial);
            }
            if let Some(ref phys) = kb.phys {
                println!("  Phys: {}", phys);
            }
            println!();
        }
    }

    #[test]
    #[ignore = "Requires access to /dev/input devices"]
    fn test_open_keyboard_event0() {
        let path = Path::new("/dev/input/event0");
        let result = open_keyboard(path);

        match result {
            Ok(Some(kb)) => {
                println!("event0 is a keyboard: {}", kb.name);
            }
            Ok(None) => {
                println!("event0 exists but is not a keyboard");
            }
            Err(e) => {
                println!("Failed to open event0: {}", e);
            }
        }
    }

    #[test]
    fn test_open_keyboard_nonexistent() {
        let path = Path::new("/dev/input/event99999");
        let result = open_keyboard(path);

        assert!(matches!(result, Err(DiscoveryError::Io(_))));
    }

    #[test]
    fn test_keyboard_info_fields() {
        let info = KeyboardInfo {
            path: PathBuf::from("/dev/input/event0"),
            name: "AT Translated Set 2 keyboard".to_string(),
            serial: Some("0000:00:00".to_string()),
            phys: Some("isa0060/serio0/input0".to_string()),
        };

        assert_eq!(info.path, PathBuf::from("/dev/input/event0"));
        assert_eq!(info.name, "AT Translated Set 2 keyboard");
        assert_eq!(info.serial.as_deref(), Some("0000:00:00"));
        assert_eq!(info.phys.as_deref(), Some("isa0060/serio0/input0"));
    }

    // Pattern matching tests - these don't require real devices
    mod pattern_matching {
        use super::*;

        /// Creates a test device with all fields populated
        fn create_test_device() -> KeyboardInfo {
            KeyboardInfo {
                path: PathBuf::from("/dev/input/event0"),
                name: "USB Keyboard".to_string(),
                serial: Some("ABC123".to_string()),
                phys: Some("usb-0000:00:14.0-1/input0".to_string()),
            }
        }

        /// Creates a test device with no serial/phys
        fn create_minimal_device() -> KeyboardInfo {
            KeyboardInfo {
                path: PathBuf::from("/dev/input/event1"),
                name: "AT Translated Set 2 keyboard".to_string(),
                serial: None,
                phys: None,
            }
        }

        #[test]
        fn test_wildcard_matches_all() {
            let device = create_test_device();
            assert!(match_device(&device, "*"));
        }

        #[test]
        fn test_wildcard_matches_minimal_device() {
            let device = create_minimal_device();
            assert!(match_device(&device, "*"));
        }

        #[test]
        fn test_exact_match_name() {
            let device = create_test_device();
            assert!(match_device(&device, "USB Keyboard"));
        }

        #[test]
        fn test_exact_match_serial() {
            let device = create_test_device();
            assert!(match_device(&device, "ABC123"));
        }

        #[test]
        fn test_exact_match_phys() {
            let device = create_test_device();
            assert!(match_device(&device, "usb-0000:00:14.0-1/input0"));
        }

        #[test]
        fn test_exact_match_case_insensitive() {
            let device = create_test_device();
            assert!(match_device(&device, "usb keyboard"));
            assert!(match_device(&device, "USB KEYBOARD"));
            assert!(match_device(&device, "Usb Keyboard"));
        }

        #[test]
        fn test_prefix_pattern_name() {
            let device = create_test_device();
            assert!(match_device(&device, "USB*"));
            assert!(match_device(&device, "USB Key*"));
            assert!(match_device(&device, "USB Keyboard*"));
        }

        #[test]
        fn test_prefix_pattern_serial() {
            let device = create_test_device();
            assert!(match_device(&device, "ABC*"));
            assert!(match_device(&device, "ABC1*"));
        }

        #[test]
        fn test_prefix_pattern_phys() {
            let device = create_test_device();
            assert!(match_device(&device, "usb-*"));
            assert!(match_device(&device, "usb-0000:*"));
        }

        #[test]
        fn test_prefix_pattern_case_insensitive() {
            let device = create_test_device();
            assert!(match_device(&device, "usb*"));
            assert!(match_device(&device, "USB*"));
            assert!(match_device(&device, "abc*"));
            assert!(match_device(&device, "ABC*"));
        }

        #[test]
        fn test_no_match_exact() {
            let device = create_test_device();
            assert!(!match_device(&device, "Logitech Keyboard"));
            assert!(!match_device(&device, "XYZ789"));
        }

        #[test]
        fn test_no_match_prefix() {
            let device = create_test_device();
            assert!(!match_device(&device, "Logitech*"));
            assert!(!match_device(&device, "XYZ*"));
        }

        #[test]
        fn test_minimal_device_no_serial_match() {
            let device = create_minimal_device();
            // Serial pattern shouldn't match if device has no serial
            assert!(!match_device(&device, "ABC*"));
        }

        #[test]
        fn test_empty_prefix_pattern() {
            let device = create_test_device();
            // "*" alone is handled as wildcard, but prefix "" with * should also work
            assert!(match_device(&device, "*"));
        }

        #[test]
        fn test_vendor_id_style_pattern() {
            // Common pattern for matching USB devices by vendor ID
            let device = KeyboardInfo {
                path: PathBuf::from("/dev/input/event2"),
                name: "USB\\VID_04D9&PID_0024".to_string(),
                serial: None,
                phys: None,
            };
            assert!(match_device(&device, "USB\\VID_04D9*"));
        }

        #[test]
        fn test_at_keyboard_pattern() {
            let device = create_minimal_device();
            assert!(match_device(&device, "AT*"));
            assert!(match_device(&device, "AT Translated*"));
        }

        #[test]
        fn test_partial_match_is_not_exact() {
            let device = create_test_device();
            // "USB" alone shouldn't match "USB Keyboard" for exact match
            assert!(!match_device(&device, "USB"));
            // But "USB*" prefix should match
            assert!(match_device(&device, "USB*"));
        }

        #[test]
        fn test_asterisk_in_middle_is_literal() {
            // An asterisk in the middle is treated literally, not as a glob
            let device = KeyboardInfo {
                path: PathBuf::from("/dev/input/event3"),
                name: "Test*Device".to_string(),
                serial: None,
                phys: None,
            };
            // Exact match with literal asterisk
            assert!(match_device(&device, "Test*Device"));
            // Prefix pattern ending with asterisk
            assert!(match_device(&device, "Test**"));
        }

        #[test]
        fn test_multiple_devices_same_pattern() {
            let devices = vec![
                KeyboardInfo {
                    path: PathBuf::from("/dev/input/event0"),
                    name: "USB Keyboard 1".to_string(),
                    serial: None,
                    phys: None,
                },
                KeyboardInfo {
                    path: PathBuf::from("/dev/input/event1"),
                    name: "USB Keyboard 2".to_string(),
                    serial: None,
                    phys: None,
                },
            ];

            // Both should match the same prefix pattern
            for device in &devices {
                assert!(match_device(device, "USB*"));
            }
        }
    }

    // DeviceManager tests - pattern matching logic tests (no real devices needed)
    mod device_manager {
        use super::*;
        use keyrx_core::config::{DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping};

        /// Helper to create a test device config with a pattern
        fn create_config(pattern: &str) -> DeviceConfig {
            DeviceConfig {
                identifier: DeviceIdentifier {
                    pattern: pattern.to_string(),
                },
                mappings: vec![KeyMapping::simple(KeyCode::A, KeyCode::B)],
            }
        }

        /// Helper to create a test device config with specific mapping
        fn create_config_with_mapping(pattern: &str, from: KeyCode, to: KeyCode) -> DeviceConfig {
            DeviceConfig {
                identifier: DeviceIdentifier {
                    pattern: pattern.to_string(),
                },
                mappings: vec![KeyMapping::simple(from, to)],
            }
        }

        #[test]
        fn test_config_pattern_matching_priority() {
            // Test that pattern matching priority is correct
            // More specific patterns should be checked before wildcards
            let configs = vec![
                create_config("USB Keyboard"), // Specific match
                create_config("USB*"),         // Prefix match
                create_config("*"),            // Wildcard match
            ];

            let device = KeyboardInfo {
                path: PathBuf::from("/dev/input/event0"),
                name: "USB Keyboard".to_string(),
                serial: None,
                phys: None,
            };

            // Device should match the specific pattern first
            let mut matched_index = None;
            for (idx, config) in configs.iter().enumerate() {
                if match_device(&device, &config.identifier.pattern) {
                    matched_index = Some(idx);
                    break;
                }
            }

            assert_eq!(
                matched_index,
                Some(0),
                "Should match specific pattern first"
            );
        }

        #[test]
        fn test_config_pattern_matching_prefix_before_wildcard() {
            let configs = vec![
                create_config("USB*"), // Prefix match
                create_config("*"),    // Wildcard match
            ];

            let device = KeyboardInfo {
                path: PathBuf::from("/dev/input/event0"),
                name: "USB Gaming Keyboard".to_string(),
                serial: None,
                phys: None,
            };

            let mut matched_index = None;
            for (idx, config) in configs.iter().enumerate() {
                if match_device(&device, &config.identifier.pattern) {
                    matched_index = Some(idx);
                    break;
                }
            }

            assert_eq!(
                matched_index,
                Some(0),
                "Should match prefix pattern before wildcard"
            );
        }

        #[test]
        fn test_config_pattern_matching_fallback_to_wildcard() {
            let configs = vec![
                create_config("Logitech*"), // Won't match
                create_config("*"),         // Wildcard should match
            ];

            let device = KeyboardInfo {
                path: PathBuf::from("/dev/input/event0"),
                name: "Generic Keyboard".to_string(),
                serial: None,
                phys: None,
            };

            let mut matched_index = None;
            for (idx, config) in configs.iter().enumerate() {
                if match_device(&device, &config.identifier.pattern) {
                    matched_index = Some(idx);
                    break;
                }
            }

            assert_eq!(matched_index, Some(1), "Should fall back to wildcard");
        }

        #[test]
        fn test_config_pattern_no_match() {
            let configs = vec![create_config("Logitech*"), create_config("Razer*")];

            let device = KeyboardInfo {
                path: PathBuf::from("/dev/input/event0"),
                name: "Generic Keyboard".to_string(),
                serial: None,
                phys: None,
            };

            let mut matched_index = None;
            for (idx, config) in configs.iter().enumerate() {
                if match_device(&device, &config.identifier.pattern) {
                    matched_index = Some(idx);
                    break;
                }
            }

            assert!(matched_index.is_none(), "Should not match any pattern");
        }

        #[test]
        fn test_multiple_devices_different_configs() {
            // Simulate matching multiple devices to different configs
            let configs = vec![
                create_config_with_mapping("USB*", KeyCode::A, KeyCode::B),
                create_config_with_mapping("AT*", KeyCode::A, KeyCode::C),
                create_config_with_mapping("*", KeyCode::A, KeyCode::D),
            ];

            let devices = vec![
                KeyboardInfo {
                    path: PathBuf::from("/dev/input/event0"),
                    name: "USB Keyboard".to_string(),
                    serial: None,
                    phys: None,
                },
                KeyboardInfo {
                    path: PathBuf::from("/dev/input/event1"),
                    name: "AT Translated Set 2 keyboard".to_string(),
                    serial: None,
                    phys: None,
                },
                KeyboardInfo {
                    path: PathBuf::from("/dev/input/event2"),
                    name: "Random Keyboard".to_string(),
                    serial: None,
                    phys: None,
                },
            ];

            let expected_config_indices = vec![0, 1, 2];

            for (device_idx, device) in devices.iter().enumerate() {
                let mut matched_config_idx = None;
                for (config_idx, config) in configs.iter().enumerate() {
                    if match_device(device, &config.identifier.pattern) {
                        matched_config_idx = Some(config_idx);
                        break;
                    }
                }

                assert_eq!(
                    matched_config_idx,
                    Some(expected_config_indices[device_idx]),
                    "Device {} should match config {}",
                    device.name,
                    expected_config_indices[device_idx]
                );
            }
        }

        #[test]
        fn test_empty_configs_no_match() {
            let configs: Vec<DeviceConfig> = vec![];

            let device = KeyboardInfo {
                path: PathBuf::from("/dev/input/event0"),
                name: "USB Keyboard".to_string(),
                serial: None,
                phys: None,
            };

            let mut matched_index = None;
            for (idx, config) in configs.iter().enumerate() {
                if match_device(&device, &config.identifier.pattern) {
                    matched_index = Some(idx);
                    break;
                }
            }

            assert!(
                matched_index.is_none(),
                "Empty configs should not match anything"
            );
        }

        #[test]
        fn test_serial_based_matching() {
            let configs = vec![
                create_config("SERIAL123"), // Exact serial match
                create_config("*"),
            ];

            let device = KeyboardInfo {
                path: PathBuf::from("/dev/input/event0"),
                name: "Generic Keyboard".to_string(),
                serial: Some("SERIAL123".to_string()),
                phys: None,
            };

            let mut matched_index = None;
            for (idx, config) in configs.iter().enumerate() {
                if match_device(&device, &config.identifier.pattern) {
                    matched_index = Some(idx);
                    break;
                }
            }

            assert_eq!(matched_index, Some(0), "Should match by serial number");
        }

        // Integration tests that require real devices
        #[test]
        #[ignore = "Requires access to /dev/input devices"]
        fn test_device_manager_discover_real_devices() {
            let configs = vec![create_config("*")];

            match DeviceManager::discover(&configs) {
                Ok(manager) => {
                    println!("Discovered {} device(s)", manager.device_count());
                    for device in manager.devices() {
                        println!(
                            "  - {} (config #{})",
                            device.info().name,
                            device.config_index()
                        );
                    }
                    assert!(
                        manager.device_count() > 0,
                        "Should find at least one device"
                    );
                }
                Err(e) => {
                    // This is expected if we don't have permission or no keyboards
                    println!("Discovery error (expected if no permissions): {}", e);
                }
            }
        }

        #[test]
        #[ignore = "Requires access to /dev/input devices"]
        fn test_device_manager_specific_pattern() {
            // Test with a specific pattern that might not match any device
            let configs = vec![create_config("NonExistentKeyboard")];

            let result = DeviceManager::discover(&configs);

            // Should fail because no device matches
            assert!(
                matches!(result, Err(DiscoveryError::NoDevicesFound)),
                "Should return NoDevicesFound for non-matching pattern"
            );
        }

        #[test]
        #[ignore = "Requires access to /dev/input devices"]
        fn test_device_manager_rebuild_lookups() {
            let initial_configs = vec![create_config_with_mapping("*", KeyCode::A, KeyCode::B)];

            let updated_configs = vec![create_config_with_mapping("*", KeyCode::A, KeyCode::C)];

            match DeviceManager::discover(&initial_configs) {
                Ok(mut manager) => {
                    // Rebuild with new configs
                    manager.rebuild_lookups(&updated_configs);
                    println!("Successfully rebuilt lookup tables");
                }
                Err(e) => {
                    println!("Discovery error (expected if no permissions): {}", e);
                }
            }
        }
    }
}
