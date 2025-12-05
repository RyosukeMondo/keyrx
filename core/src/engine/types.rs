//! Core type definitions for input/output events.

use serde::{Deserialize, Serialize};

// Re-export KeyCode from the single source of truth (keycodes.rs)
pub use crate::drivers::keycodes::KeyCode;

/// Action to take when a key is pressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum RemapAction {
    /// Remap this key to another key.
    Remap(KeyCode),
    /// Block this key (consume it, don't pass through).
    Block,
    /// Pass this key through unchanged.
    #[default]
    Pass,
}

/// Input event from keyboard.
///
/// Contains the core key information plus metadata about the event source,
/// timing, and nature. Metadata fields enable advanced remapping features
/// like tap-hold detection, device-specific configurations, and synthetic
/// event filtering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputEvent {
    /// Key that was pressed/released.
    pub key: KeyCode,
    /// True if key down, false if key up.
    pub pressed: bool,
    /// Timestamp in microseconds since driver start.
    ///
    /// **Linux**: Derived from `struct timeval` in evdev events (sec * 1_000_000 + usec).
    /// **Windows**: Derived from `KBDLLHOOKSTRUCT.time` (ms * 1_000).
    pub timestamp_us: u64,
    /// Identifier of the source keyboard device.
    ///
    /// **Linux**: Device path (e.g., "/dev/input/event3").
    /// **Windows**: Not available (None), as hooks don't identify source device.
    /// **Mock**: Set to "mock" for test events.
    pub device_id: Option<String>,
    /// Whether this is an auto-repeat event (key held down).
    ///
    /// **Linux**: Detected via evdev `value == 2`.
    /// **Windows**: Detected by tracking key state (key down while already down).
    /// Useful for tap-hold detection to distinguish initial press from repeat.
    pub is_repeat: bool,
    /// Whether this event was injected by software (including KeyRx itself).
    ///
    /// **Linux**: Detected by comparing event source to uinput device.
    /// **Windows**: Detected via `LLKHF_INJECTED` flag in hook callback.
    /// Critical for preventing infinite loops when our injected keys are recaptured.
    pub is_synthetic: bool,
    /// Raw hardware scan code from the input device.
    ///
    /// **Linux**: The evdev event code (e.g., KEY_A = 30).
    /// **Windows**: The `scanCode` field from `KBDLLHOOKSTRUCT`.
    /// Useful for handling keys that may have the same virtual key code but
    /// different physical locations (e.g., numpad Enter vs main Enter).
    pub scan_code: u16,
    /// Device serial number for unique device identification.
    ///
    /// **Linux**: Extracted via EVIOCGUNIQ ioctl, udev properties, or synthetic ID.
    /// **Windows**: Extracted via HidD_GetSerialNumberString or Instance ID from device path.
    /// Used with vendor_id:product_id to create unique DeviceIdentity for per-device configuration.
    pub serial_number: Option<String>,
    /// USB Vendor ID for device identification.
    ///
    /// **Linux**: Extracted from udev properties.
    /// **Windows**: Parsed from device instance path.
    /// Used with product_id and serial_number to create unique DeviceIdentity.
    pub vendor_id: Option<u16>,
    /// USB Product ID for device identification.
    ///
    /// **Linux**: Extracted from udev properties.
    /// **Windows**: Parsed from device instance path.
    /// Used with vendor_id and serial_number to create unique DeviceIdentity.
    pub product_id: Option<u16>,
}

/// Output action to send to OS.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputAction {
    /// Press a key.
    KeyDown(KeyCode),
    /// Release a key.
    KeyUp(KeyCode),
    /// Press and release a key.
    KeyTap(KeyCode),
    /// Block the original input (consume it).
    Block,
    /// Pass through the original input unchanged.
    PassThrough,
}

impl Default for InputEvent {
    fn default() -> Self {
        Self {
            key: KeyCode::Unknown(0),
            pressed: false,
            timestamp_us: 0,
            device_id: None,
            is_repeat: false,
            is_synthetic: false,
            scan_code: 0,
            serial_number: None,
            vendor_id: None,
            product_id: None,
        }
    }
}

impl InputEvent {
    /// Create a new key down event with default metadata.
    ///
    /// For production use, prefer `with_metadata()` or populate fields directly.
    pub fn key_down(key: KeyCode, timestamp_us: u64) -> Self {
        Self {
            key,
            pressed: true,
            timestamp_us,
            ..Default::default()
        }
    }

    /// Create a new key up event with default metadata.
    ///
    /// For production use, prefer `with_metadata()` or populate fields directly.
    pub fn key_up(key: KeyCode, timestamp_us: u64) -> Self {
        Self {
            key,
            pressed: false,
            timestamp_us,
            ..Default::default()
        }
    }

    /// Create a new event with full metadata.
    #[allow(clippy::too_many_arguments)]
    pub fn with_metadata(
        key: KeyCode,
        pressed: bool,
        timestamp_us: u64,
        device_id: Option<String>,
        is_repeat: bool,
        is_synthetic: bool,
        scan_code: u16,
        serial_number: Option<String>,
    ) -> Self {
        Self {
            key,
            pressed,
            timestamp_us,
            device_id,
            is_repeat,
            is_synthetic,
            scan_code,
            serial_number,
            vendor_id: None,
            product_id: None,
        }
    }

    /// Create a new event with complete identity metadata.
    #[allow(clippy::too_many_arguments)]
    pub fn with_full_identity(
        key: KeyCode,
        pressed: bool,
        timestamp_us: u64,
        device_id: Option<String>,
        is_repeat: bool,
        is_synthetic: bool,
        scan_code: u16,
        serial_number: Option<String>,
        vendor_id: Option<u16>,
        product_id: Option<u16>,
    ) -> Self {
        Self {
            key,
            pressed,
            timestamp_us,
            device_id,
            is_repeat,
            is_synthetic,
            scan_code,
            serial_number,
            vendor_id,
            product_id,
        }
    }

    /// Returns the legacy timestamp field (alias for timestamp_us).
    ///
    /// Provided for backward compatibility with code expecting `timestamp`.
    #[inline]
    pub fn timestamp(&self) -> u64 {
        self.timestamp_us
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn keycode_can_be_hashmap_key_with_remap_action() {
        let mut map: HashMap<KeyCode, RemapAction> = HashMap::new();
        map.insert(KeyCode::A, RemapAction::Remap(KeyCode::B));
        map.insert(KeyCode::CapsLock, RemapAction::Block);

        assert_eq!(map.get(&KeyCode::A), Some(&RemapAction::Remap(KeyCode::B)));
        assert_eq!(map.get(&KeyCode::CapsLock), Some(&RemapAction::Block));
        assert_eq!(map.get(&KeyCode::Z), None);
    }

    #[test]
    fn remap_action_default_is_pass() {
        assert_eq!(RemapAction::default(), RemapAction::Pass);
    }

    #[test]
    fn input_event_key_down() {
        let event = InputEvent::key_down(KeyCode::A, 1000);
        assert_eq!(event.key, KeyCode::A);
        assert!(event.pressed);
        assert_eq!(event.timestamp_us, 1000);
    }

    #[test]
    fn input_event_key_up() {
        let event = InputEvent::key_up(KeyCode::B, 2000);
        assert_eq!(event.key, KeyCode::B);
        assert!(!event.pressed);
        assert_eq!(event.timestamp_us, 2000);
    }

    #[test]
    fn input_event_timestamp_alias() {
        let event = InputEvent::key_down(KeyCode::A, 5000);
        assert_eq!(event.timestamp(), 5000);
        assert_eq!(event.timestamp(), event.timestamp_us);
    }
}
