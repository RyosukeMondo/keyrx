//! Component-level test for multi-device keyboard discrimination.
//!
//! This test verifies the device pattern matching logic without spawning
//! a full daemon process or relying on uinput. It tests:
//! 1. Device pattern matching (exact, prefix, suffix, contains, wildcard)
//! 2. Case-insensitive matching
//! 3. Serial number matching
//!
//! This approach is more reliable than E2E tests because it:
//! - Doesn't depend on kernel uinput behavior
//! - Runs faster (no daemon compilation/startup)
//! - Is easier to debug when it fails
//! - Tests the critical bug-prone pattern matching logic directly

use keyrx_core::config::KeyCode;
use keyrx_core::runtime::KeyEvent;
use keyrx_daemon::device_manager::{match_device, KeyboardInfo};
use std::path::PathBuf;

/// Helper to create a test KeyboardInfo
fn make_keyboard(path: &str, name: &str, serial: Option<&str>) -> KeyboardInfo {
    KeyboardInfo {
        path: PathBuf::from(path),
        name: name.to_string(),
        serial: serial.map(String::from),
        phys: None,
    }
}

#[test]
fn test_pattern_matching_wildcard() {
    let device = make_keyboard("/dev/input/event0", "Any Keyboard", None);
    assert!(match_device(&device, "*"));
}

#[test]
fn test_pattern_matching_exact() {
    let device = make_keyboard("/dev/input/event0", "USB Keyboard", None);
    assert!(match_device(&device, "USB Keyboard"));
    assert!(!match_device(&device, "Other Keyboard"));
}

#[test]
fn test_pattern_matching_prefix() {
    let device = make_keyboard("/dev/input/event0", "Logitech USB Keyboard", None);
    assert!(match_device(&device, "Logitech*"));
    assert!(!match_device(&device, "Razer*"));
}

#[test]
fn test_pattern_matching_suffix() {
    let device = make_keyboard("/dev/input/event0", "USB Keyboard", None);
    assert!(match_device(&device, "*Keyboard"));
    assert!(!match_device(&device, "*Mouse"));
}

#[test]
fn test_pattern_matching_contains() {
    let device = make_keyboard("/dev/input/event0", "Logitech USB Keyboard", None);
    assert!(match_device(&device, "*USB*"));
    assert!(match_device(&device, "*Logitech*"));
    assert!(!match_device(&device, "*Razer*"));
}

#[test]
fn test_pattern_matching_serial() {
    let device = make_keyboard("/dev/input/event0", "USB Keyboard", Some("SN12345"));
    assert!(match_device(&device, "SN12345"));
    assert!(match_device(&device, "SN123*"));
    assert!(!match_device(&device, "SN999*"));
}

#[test]
fn test_pattern_matching_case_insensitive() {
    let device = make_keyboard("/dev/input/event0", "USB Keyboard", None);
    assert!(match_device(&device, "usb keyboard"));
    assert!(match_device(&device, "USB*"));
    assert!(match_device(&device, "usb*"));
    assert!(match_device(&device, "*KEYBOARD"));
}

#[test]
fn test_device_specific_pattern_matching() {
    // Create two devices: numpad and main keyboard
    let numpad = make_keyboard("/dev/input/event0", "Numpad Device", None);
    let main_kbd = make_keyboard("/dev/input/event1", "Main Keyboard", None);

    // Verify pattern matching works correctly for each device
    assert!(match_device(&numpad, "*Numpad*"));
    assert!(!match_device(&numpad, "*Main*"));
    assert!(match_device(&main_kbd, "*Main*"));
    assert!(!match_device(&main_kbd, "*Numpad*"));

    // Verify wildcard matches both
    assert!(match_device(&numpad, "*"));
    assert!(match_device(&main_kbd, "*"));
}

#[test]
fn test_multiple_pattern_types_coexist() {
    // Create devices with non-overlapping names for each pattern type
    let prefix_dev = make_keyboard("/dev/input/event0", "logitech-mouse", None);
    let suffix_dev = make_keyboard("/dev/input/event1", "apple-keyboard", None);
    let contains_dev = make_keyboard("/dev/input/event2", "my-numpad-device", None);

    // Verify each pattern type works
    assert!(match_device(&prefix_dev, "logitech-*"));
    assert!(match_device(&suffix_dev, "*-keyboard"));
    assert!(match_device(&contains_dev, "*numpad*"));

    // Verify cross-matching doesn't occur
    assert!(!match_device(&prefix_dev, "*-keyboard")); // logitech-mouse doesn't end with -keyboard
    assert!(!match_device(&suffix_dev, "logitech-*")); // apple-keyboard doesn't start with logitech-
    assert!(!match_device(&contains_dev, "logitech-*")); // my-numpad-device doesn't start with logitech-

    // Verify contains pattern is distinct
    assert!(!match_device(&prefix_dev, "*numpad*")); // logitech-mouse doesn't contain numpad
    assert!(!match_device(&suffix_dev, "*numpad*")); // apple-keyboard doesn't contain numpad
}

#[test]
fn test_device_without_serial() {
    let device = make_keyboard("/dev/input/event0", "AT Translated Set 2 keyboard", None);

    // Should match by name
    assert!(match_device(&device, "AT Translated*"));
    assert!(match_device(&device, "*keyboard"));

    // Should not match by serial (doesn't have one)
    assert!(!match_device(&device, "SN123*"));
}

#[test]
fn test_event_with_device_id() {
    // Create events with device IDs using builder pattern
    let event1 = KeyEvent::press(KeyCode::A).with_device_id("device-1".to_string());
    let event2 = KeyEvent::press(KeyCode::A).with_device_id("device-2".to_string());
    let event3 = KeyEvent::press(KeyCode::A);

    // Verify device IDs are preserved
    assert_eq!(event1.device_id(), Some("device-1"));
    assert_eq!(event2.device_id(), Some("device-2"));
    assert_eq!(event3.device_id(), None);

    // Verify events are independent
    assert_ne!(event1.device_id(), event2.device_id());
}
