//! Tests for when_device() function

use super::*;

use keyrx_core::config::Condition;

/// Test when_device_start() with exact pattern
#[test]
fn test_when_device_exact_pattern() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        when_device_start("usb-numpad-123");
        map("Numpad1", "VK_F13");
        when_device_end();
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    assert_eq!(config.devices.len(), 1);
    assert_eq!(config.devices[0].mappings.len(), 1);

    match &config.devices[0].mappings[0] {
        KeyMapping::Conditional {
            condition,
            mappings,
        } => {
            match condition {
                Condition::DeviceMatches(pattern) => {
                    assert_eq!(pattern, "usb-numpad-123");
                }
                _ => panic!("Expected DeviceMatches condition"),
            }
            assert_eq!(mappings.len(), 1);
            match &mappings[0] {
                BaseKeyMapping::Simple { from, to } => {
                    assert_eq!(*from, KeyCode::Numpad1);
                    assert_eq!(*to, KeyCode::F13);
                }
                _ => panic!("Expected Simple mapping"),
            }
        }
        _ => panic!("Expected Conditional mapping"),
    }
}

/// Test when_device_start() with wildcard pattern
#[test]
fn test_when_device_wildcard_pattern() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        when_device_start("*numpad*");
        map("Numpad1", "VK_F13");
        map("Numpad2", "VK_F14");
        when_device_end();
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    assert_eq!(config.devices.len(), 1);
    assert_eq!(config.devices[0].mappings.len(), 1);

    match &config.devices[0].mappings[0] {
        KeyMapping::Conditional {
            condition,
            mappings,
        } => {
            match condition {
                Condition::DeviceMatches(pattern) => {
                    assert_eq!(pattern, "*numpad*");
                }
                _ => panic!("Expected DeviceMatches condition"),
            }
            assert_eq!(mappings.len(), 2);
        }
        _ => panic!("Expected Conditional mapping"),
    }
}

/// Test when_device_start() with prefix pattern
#[test]
fn test_when_device_prefix_pattern() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        when_device_start("usb-*");
        map("A", "VK_B");
        when_device_end();
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    match &config.devices[0].mappings[0] {
        KeyMapping::Conditional { condition, .. } => match condition {
            Condition::DeviceMatches(pattern) => {
                assert_eq!(pattern, "usb-*");
            }
            _ => panic!("Expected DeviceMatches condition"),
        },
        _ => panic!("Expected Conditional mapping"),
    }
}

/// Test empty pattern returns error
#[test]
fn test_when_device_empty_pattern_error() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        when_device_start("");
        map("A", "VK_B");
        when_device_end();
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_err(), "Should fail with empty pattern");

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("empty"),
        "Error should mention empty pattern: {}",
        err_msg
    );
}

/// Test when_device_end() without when_device_start() returns error
#[test]
fn test_when_device_end_without_start_error() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        when_device_end();
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(
        result.is_err(),
        "Should fail without matching when_device_start"
    );

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("without") || err_msg.contains("matching"),
        "Error should mention missing when_device_start: {}",
        err_msg
    );
}

/// Test when_device_start() outside device block returns error
#[test]
fn test_when_device_outside_device_block_error() {
    let mut parser = Parser::new();
    let script = r#"
        when_device_start("*numpad*");
        map("A", "VK_B");
        when_device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_err(), "Should fail outside device block");

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("device"),
        "Error should mention device block requirement: {}",
        err_msg
    );
}

/// Test multiple device conditions in same device block
#[test]
fn test_multiple_device_conditions() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        when_device_start("*numpad*");
        map("Numpad1", "VK_F13");
        when_device_end();
        when_device_start("*keyboard*");
        map("A", "VK_B");
        when_device_end();
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    assert_eq!(config.devices[0].mappings.len(), 2);

    // First condition: *numpad*
    match &config.devices[0].mappings[0] {
        KeyMapping::Conditional { condition, .. } => match condition {
            Condition::DeviceMatches(pattern) => {
                assert_eq!(pattern, "*numpad*");
            }
            _ => panic!("Expected DeviceMatches condition"),
        },
        _ => panic!("Expected Conditional mapping"),
    }

    // Second condition: *keyboard*
    match &config.devices[0].mappings[1] {
        KeyMapping::Conditional { condition, .. } => match condition {
            Condition::DeviceMatches(pattern) => {
                assert_eq!(pattern, "*keyboard*");
            }
            _ => panic!("Expected DeviceMatches condition"),
        },
        _ => panic!("Expected Conditional mapping"),
    }
}

/// Test when_device alongside modifier conditions
#[test]
fn test_when_device_alongside_modifier() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        when_device_start("*numpad*");
        map("Numpad1", "VK_F13");
        when_device_end();
        when_start("MD_00");
        map("H", "VK_Left");
        when_end();
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    assert_eq!(config.devices[0].mappings.len(), 2);

    // First: DeviceMatches
    match &config.devices[0].mappings[0] {
        KeyMapping::Conditional { condition, .. } => {
            assert!(matches!(condition, Condition::DeviceMatches(_)));
        }
        _ => panic!("Expected Conditional mapping"),
    }

    // Second: ModifierActive
    match &config.devices[0].mappings[1] {
        KeyMapping::Conditional { condition, .. } => {
            assert!(matches!(
                condition,
                Condition::ModifierActive(_) | Condition::AllActive(_)
            ));
        }
        _ => panic!("Expected Conditional mapping"),
    }
}

/// Test when_device_start() with suffix pattern (*-keyboard)
#[test]
fn test_when_device_suffix_pattern() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        when_device_start("*-keyboard");
        map("A", "VK_B");
        when_device_end();
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    match &config.devices[0].mappings[0] {
        KeyMapping::Conditional { condition, .. } => match condition {
            Condition::DeviceMatches(pattern) => {
                assert_eq!(pattern, "*-keyboard");
            }
            _ => panic!("Expected DeviceMatches condition"),
        },
        _ => panic!("Expected Conditional mapping"),
    }
}

/// Test device and modifier conditions separately
///
/// Verifies that device patterns can exist alongside modifier layers
/// in the same device block (not nested, which is unsupported).
#[test]
fn test_device_and_modifier_separate() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        // Device-specific mappings
        when_device_start("*numpad*");
            map("Numpad1", "VK_F13");
            map("Numpad2", "VK_F14");
        when_device_end();

        // Modifier layer (separate from device matching)
        map("CapsLock", "MD_00");
        when_start("MD_00");
            map("H", "VK_Left");
            map("J", "VK_Down");
        when_end();

        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    assert_eq!(config.devices.len(), 1);

    // Should have device conditional mapping
    let has_device_conditional = config.devices[0].mappings.iter().any(|m| {
        matches!(m, KeyMapping::Conditional { condition, .. }
            if matches!(condition, Condition::DeviceMatches(_)))
    });
    assert!(
        has_device_conditional,
        "Should have DeviceMatches conditional"
    );

    // Should have modifier conditional mapping
    let has_modifier_conditional = config.devices[0].mappings.iter().any(|m| {
        matches!(m, KeyMapping::Conditional { condition, .. }
            if matches!(condition, Condition::AllActive(_) | Condition::ModifierActive(_)))
    });
    assert!(has_modifier_conditional, "Should have modifier conditional");
}

/// Test tap_hold with device pattern
///
/// Verifies that tap-hold configurations work within device contexts.
#[test]
fn test_tap_hold_with_device_pattern() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        when_device_start("*gaming*");
            tap_hold("Space", "VK_Space", "MD_00", 200);
        when_device_end();
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    assert_eq!(config.devices.len(), 1);

    // Verify the device conditional wraps the tap-hold
    match &config.devices[0].mappings[0] {
        KeyMapping::Conditional {
            condition,
            mappings,
        } => {
            match condition {
                Condition::DeviceMatches(pattern) => {
                    assert_eq!(pattern, "*gaming*");
                }
                _ => panic!("Expected DeviceMatches condition"),
            }
            // Tap-hold should be inside the conditional
            assert!(
                mappings.len() > 0,
                "Should have mappings inside conditional"
            );
        }
        _ => panic!("Expected Conditional mapping wrapping tap-hold"),
    }
}

/// Test complex realistic multi-device configuration
///
/// Simulates a real-world setup with:
/// - Main keyboard with standard mappings and modifier layer
/// - Numpad as macro pad for OBS/streaming
/// - Gaming keyboard with custom key mappings
#[test]
fn test_complex_realistic_multi_device() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("*");

        // Numpad as Stream Deck
        when_device_start("*numpad*");
            map("Numpad1", "VK_F13");  // Scene 1
            map("Numpad2", "VK_F14");  // Scene 2
            map("Numpad3", "VK_F15");  // Scene 3
            map("NumpadEnter", "VK_F23");  // Go Live
        when_device_end();

        // Gaming keyboard with WASD remapping
        when_device_start("*gaming*");
            map("W", "VK_Up");
            map("A", "VK_Left");
            map("S", "VK_Down");
            map("D", "VK_Right");
        when_device_end();

        // Standard mappings for all devices
        map("Escape", "VK_Grave");
        map("CapsLock", "MD_00");

        // Modifier layer for all devices
        when_start("MD_00");
            map("H", "VK_Left");
            map("J", "VK_Down");
            map("K", "VK_Up");
            map("L", "VK_Right");
        when_end();

        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(
        result.is_ok(),
        "Failed to parse complex config: {:?}",
        result.err()
    );

    let config = result.unwrap();
    assert_eq!(config.devices.len(), 1);

    // Should have device conditionals
    let device_conditionals = config.devices[0]
        .mappings
        .iter()
        .filter(|m| {
            matches!(m, KeyMapping::Conditional { condition, .. }
            if matches!(condition, Condition::DeviceMatches(_)))
        })
        .count();
    assert_eq!(
        device_conditionals, 2,
        "Should have 2 device conditionals (numpad + gaming)"
    );

    // Should have modifier conditional
    let modifier_conditionals = config.devices[0]
        .mappings
        .iter()
        .filter(|m| {
            matches!(m, KeyMapping::Conditional { condition, .. }
            if matches!(condition, Condition::AllActive(_) | Condition::ModifierActive(_)))
        })
        .count();
    assert_eq!(
        modifier_conditionals, 1,
        "Should have 1 modifier conditional"
    );

    // Should have base mappings (Escape, CapsLock)
    let base_mappings = config.devices[0]
        .mappings
        .iter()
        .filter(|m| matches!(m, KeyMapping::Base(_)))
        .count();
    assert!(base_mappings >= 2, "Should have at least 2 base mappings");
}

/// Test case sensitivity in device patterns
#[test]
fn test_device_pattern_case_sensitivity() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        when_device_start("USB-Keyboard");
        map("A", "VK_B");
        when_device_end();
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    match &config.devices[0].mappings[0] {
        KeyMapping::Conditional { condition, .. } => match condition {
            Condition::DeviceMatches(pattern) => {
                // Pattern should preserve case
                assert_eq!(pattern, "USB-Keyboard");
                assert_ne!(pattern, &"usb-keyboard");
            }
            _ => panic!("Expected DeviceMatches condition"),
        },
        _ => panic!("Expected Conditional mapping"),
    }
}

/// Test special characters in device patterns
#[test]
fn test_device_pattern_special_chars() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        when_device_start("usb-0000:00:14.0-1/input0");
        map("A", "VK_B");
        when_device_end();
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    match &config.devices[0].mappings[0] {
        KeyMapping::Conditional { condition, .. } => match condition {
            Condition::DeviceMatches(pattern) => {
                assert_eq!(pattern, "usb-0000:00:14.0-1/input0");
            }
            _ => panic!("Expected DeviceMatches condition"),
        },
        _ => panic!("Expected Conditional mapping"),
    }
}
