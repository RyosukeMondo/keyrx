//! Linux-specific device enumeration and pattern matching using evdev.
//!
//! This module scans `/dev/input/event*` devices and identifies keyboards
//! based on their capabilities (presence of alphabetic keys). It also
//! provides pattern matching for selecting devices based on configuration.

use std::fs;
use std::path::Path;

use evdev::{Device, EventType, Key};
use log::debug;

use super::{DiscoveryError, KeyboardInfo};

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
}
