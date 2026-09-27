use super::*;
use alloc::string::String;

#[test]
fn test_exact_match() {
    // Exact match should work
    assert!(DeviceState::matches_device_pattern(
        Some("usb-numpad-123"),
        "usb-numpad-123"
    ));
    // Different string should not match
    assert!(!DeviceState::matches_device_pattern(
        Some("usb-keyboard-456"),
        "usb-numpad-123"
    ));
    // None device_id should not match
    assert!(!DeviceState::matches_device_pattern(None, "usb-numpad-123"));
}

#[test]
fn test_wildcard_matches_all() {
    // Pattern "*" matches everything
    assert!(DeviceState::matches_device_pattern(Some("anything"), "*"));
    assert!(DeviceState::matches_device_pattern(Some(""), "*"));
    assert!(DeviceState::matches_device_pattern(
        Some("usb-numpad-123"),
        "*"
    ));
    // But still not None
    assert!(!DeviceState::matches_device_pattern(None, "*"));
}

#[test]
fn test_prefix_pattern() {
    // Pattern "usb-*" matches anything starting with "usb-"
    assert!(DeviceState::matches_device_pattern(
        Some("usb-numpad"),
        "usb-*"
    ));
    assert!(DeviceState::matches_device_pattern(
        Some("usb-keyboard"),
        "usb-*"
    ));
    assert!(DeviceState::matches_device_pattern(Some("usb-"), "usb-*"));
    // But not things that don't start with "usb-"
    assert!(!DeviceState::matches_device_pattern(
        Some("bt-keyboard"),
        "usb-*"
    ));
    assert!(!DeviceState::matches_device_pattern(
        Some("keyboard-usb"),
        "usb-*"
    ));
}

#[test]
fn test_suffix_pattern() {
    // Pattern "*-keyboard" matches anything ending with "-keyboard"
    assert!(DeviceState::matches_device_pattern(
        Some("usb-keyboard"),
        "*-keyboard"
    ));
    assert!(DeviceState::matches_device_pattern(
        Some("bt-keyboard"),
        "*-keyboard"
    ));
    assert!(DeviceState::matches_device_pattern(
        Some("-keyboard"),
        "*-keyboard"
    ));
    // But not things that don't end with "-keyboard"
    assert!(!DeviceState::matches_device_pattern(
        Some("keyboard-usb"),
        "*-keyboard"
    ));
    assert!(!DeviceState::matches_device_pattern(
        Some("usb-numpad"),
        "*-keyboard"
    ));
}

#[test]
fn test_contains_pattern() {
    // Pattern "*numpad*" matches anything containing "numpad"
    assert!(DeviceState::matches_device_pattern(
        Some("usb-numpad-123"),
        "*numpad*"
    ));
    assert!(DeviceState::matches_device_pattern(
        Some("numpad"),
        "*numpad*"
    ));
    assert!(DeviceState::matches_device_pattern(
        Some("my-numpad-device"),
        "*numpad*"
    ));
    // But not things that don't contain "numpad"
    assert!(!DeviceState::matches_device_pattern(
        Some("usb-keyboard"),
        "*numpad*"
    ));
    assert!(!DeviceState::matches_device_pattern(
        Some("numpd"),
        "*numpad*"
    ));
}

#[test]
fn test_prefix_and_suffix_pattern() {
    // Pattern "usb-*-keyboard" matches "usb-...-keyboard"
    assert!(DeviceState::matches_device_pattern(
        Some("usb-logitech-keyboard"),
        "usb-*-keyboard"
    ));
    assert!(DeviceState::matches_device_pattern(
        Some("usb-keyboard"),
        "usb-*-keyboard"
    ));
    // But not mismatched
    assert!(!DeviceState::matches_device_pattern(
        Some("bt-logitech-keyboard"),
        "usb-*-keyboard"
    ));
    assert!(!DeviceState::matches_device_pattern(
        Some("usb-logitech-numpad"),
        "usb-*-keyboard"
    ));
}

#[test]
fn test_evaluate_condition_device_matches() {
    let state = DeviceState::new();

    // Exact match condition
    let cond = Condition::DeviceMatches(String::from("usb-numpad-123"));
    assert!(state.evaluate_condition_with_device(&cond, Some("usb-numpad-123")));
    assert!(!state.evaluate_condition_with_device(&cond, Some("usb-keyboard")));
    assert!(!state.evaluate_condition_with_device(&cond, None));

    // Wildcard pattern condition
    let cond_wildcard = Condition::DeviceMatches(String::from("*numpad*"));
    assert!(state.evaluate_condition_with_device(&cond_wildcard, Some("usb-numpad-123")));
    assert!(state.evaluate_condition_with_device(&cond_wildcard, Some("my-numpad")));
    assert!(!state.evaluate_condition_with_device(&cond_wildcard, Some("keyboard")));

    // evaluate_condition (without device) always returns false for DeviceMatches
    assert!(!state.evaluate_condition(&cond));
    assert!(!state.evaluate_condition(&cond_wildcard));
}

#[test]
fn test_device_pattern_prefix() {
    let state = DeviceState::new();

    // Prefix pattern (usb-*)
    let cond = Condition::DeviceMatches(String::from("usb-*"));
    assert!(state.evaluate_condition_with_device(&cond, Some("usb-keyboard")));
    assert!(state.evaluate_condition_with_device(&cond, Some("usb-numpad-123")));
    assert!(state.evaluate_condition_with_device(&cond, Some("usb-")));
    assert!(!state.evaluate_condition_with_device(&cond, Some("serial-usb-device")));
    assert!(!state.evaluate_condition_with_device(&cond, Some("usb"))); // No hyphen
}

#[test]
fn test_device_pattern_suffix() {
    let state = DeviceState::new();

    // Suffix pattern (*-keyboard)
    let cond = Condition::DeviceMatches(String::from("*-keyboard"));
    assert!(state.evaluate_condition_with_device(&cond, Some("usb-keyboard")));
    assert!(state.evaluate_condition_with_device(&cond, Some("at-translated-keyboard")));
    assert!(state.evaluate_condition_with_device(&cond, Some("-keyboard")));
    assert!(!state.evaluate_condition_with_device(&cond, Some("keyboard-usb")));
    assert!(!state.evaluate_condition_with_device(&cond, Some("keyboard")));
    // No hyphen
}

#[test]
fn test_device_pattern_contains() {
    let state = DeviceState::new();

    // Contains pattern (*numpad*)
    let cond = Condition::DeviceMatches(String::from("*numpad*"));
    assert!(state.evaluate_condition_with_device(&cond, Some("usb-numpad-123")));
    assert!(state.evaluate_condition_with_device(&cond, Some("numpad")));
    assert!(state.evaluate_condition_with_device(&cond, Some("my-numpad-device")));
    assert!(state.evaluate_condition_with_device(&cond, Some("anumpadb"))); // Contains substring
    assert!(!state.evaluate_condition_with_device(&cond, Some("keyboard")));
}

#[test]
fn test_device_pattern_case_sensitive() {
    let state = DeviceState::new();

    // Pattern matching should be case-sensitive
    let cond = Condition::DeviceMatches(String::from("USB-Keyboard"));
    assert!(state.evaluate_condition_with_device(&cond, Some("USB-Keyboard")));
    assert!(!state.evaluate_condition_with_device(&cond, Some("usb-keyboard")));
    assert!(!state.evaluate_condition_with_device(&cond, Some("USB-KEYBOARD")));
    assert!(!state.evaluate_condition_with_device(&cond, Some("Usb-Keyboard")));
}

#[test]
fn test_device_pattern_empty_and_whitespace() {
    let state = DeviceState::new();

    // Empty pattern should only match empty device_id
    let cond_empty = Condition::DeviceMatches(String::from(""));
    assert!(state.evaluate_condition_with_device(&cond_empty, Some("")));
    assert!(!state.evaluate_condition_with_device(&cond_empty, Some("any")));
    assert!(!state.evaluate_condition_with_device(&cond_empty, None));

    // Whitespace should be treated literally
    let cond_space = Condition::DeviceMatches(String::from(" "));
    assert!(state.evaluate_condition_with_device(&cond_space, Some(" ")));
    assert!(!state.evaluate_condition_with_device(&cond_space, Some("")));
    assert!(!state.evaluate_condition_with_device(&cond_space, Some("space")));
}

#[test]
fn test_device_pattern_wildcard_only() {
    let state = DeviceState::new();

    // Single wildcard should match everything
    let cond = Condition::DeviceMatches(String::from("*"));
    assert!(state.evaluate_condition_with_device(&cond, Some("any-device")));
    assert!(state.evaluate_condition_with_device(&cond, Some("")));
    assert!(state.evaluate_condition_with_device(&cond, Some("usb-keyboard")));
    assert!(!state.evaluate_condition_with_device(&cond, None)); // Still requires Some()
}

#[test]
fn test_device_pattern_special_characters() {
    let state = DeviceState::new();

    // Special characters should work (path-like patterns)
    let cond = Condition::DeviceMatches(String::from("usb-0000:00:14.0-1/input0"));
    assert!(state.evaluate_condition_with_device(&cond, Some("usb-0000:00:14.0-1/input0")));
    assert!(!state.evaluate_condition_with_device(&cond, Some("usb-0000:00:14.0-2/input0")));

    // Pattern with special chars and wildcard
    let cond_wildcard = Condition::DeviceMatches(String::from("usb-*:00:14.0-*/input0"));
    assert!(state.evaluate_condition_with_device(&cond_wildcard, Some("usb-0000:00:14.0-1/input0")));
    assert!(state.evaluate_condition_with_device(&cond_wildcard, Some("usb-1234:00:14.0-5/input0")));
    assert!(
        !state.evaluate_condition_with_device(&cond_wildcard, Some("usb-0000:00:14.0-1/input1"))
    );
}

#[test]
fn test_device_pattern_unicode() {
    let state = DeviceState::new();

    // Unicode characters should work
    let cond = Condition::DeviceMatches(String::from("キーボード-日本語"));
    assert!(state.evaluate_condition_with_device(&cond, Some("キーボード-日本語")));
    assert!(!state.evaluate_condition_with_device(&cond, Some("keyboard")));

    // Unicode with wildcard
    let cond_wildcard = Condition::DeviceMatches(String::from("*キーボード*"));
    assert!(state.evaluate_condition_with_device(&cond_wildcard, Some("usb-キーボード-123")));
    assert!(state.evaluate_condition_with_device(&cond_wildcard, Some("キーボード")));
    assert!(!state.evaluate_condition_with_device(&cond_wildcard, Some("keyboard")));
}

#[test]
fn test_device_pattern_multiple_wildcards() {
    let state = DeviceState::new();

    // Multiple wildcards in pattern
    let cond = Condition::DeviceMatches(String::from("*usb*keyboard*"));
    assert!(state.evaluate_condition_with_device(&cond, Some("my-usb-fancy-keyboard-123")));
    assert!(state.evaluate_condition_with_device(&cond, Some("usb-keyboard")));
    assert!(state.evaluate_condition_with_device(&cond, Some("usbkeyboard123"))); // No separators needed
    assert!(!state.evaluate_condition_with_device(&cond, Some("keyboard-usb"))); // Wrong order
    assert!(!state.evaluate_condition_with_device(&cond, Some("usb-only")));
}

#[test]
fn test_device_pattern_edge_cases_none() {
    let state = DeviceState::new();

    // None device_id should never match any pattern
    let patterns = vec!["*", "usb-*", "*-keyboard", "*numpad*", "", "exact-match"];

    for pattern_str in patterns {
        let cond = Condition::DeviceMatches(String::from(pattern_str));
        assert!(
            !state.evaluate_condition_with_device(&cond, None),
            "Pattern '{}' should not match None",
            pattern_str
        );
    }
}

#[test]
fn test_device_pattern_realistic_device_ids() {
    let state = DeviceState::new();

    // Test with realistic device IDs from actual hardware

    // Linux evdev path
    let linux_pattern = Condition::DeviceMatches(String::from("/dev/input/event*"));
    assert!(state.evaluate_condition_with_device(&linux_pattern, Some("/dev/input/event0")));
    assert!(state.evaluate_condition_with_device(&linux_pattern, Some("/dev/input/event15")));

    // Windows HID path
    let windows_pattern = Condition::DeviceMatches(String::from("*VID_046D&PID_C52B*"));
    assert!(state.evaluate_condition_with_device(
        &windows_pattern,
        Some(
            "\\\\?\\HID#VID_046D&PID_C52B#7&1a2b3c4d&0&0000#{884b96c3-56ef-11d1-bc8c-00a0c91405dd}"
        )
    ));

    // Serial number match
    let serial_pattern = Condition::DeviceMatches(String::from("*SN12345*"));
    assert!(state.evaluate_condition_with_device(&serial_pattern, Some("USB-Keyboard-SN12345-v2")));
}
