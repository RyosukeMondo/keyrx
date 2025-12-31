//! Tests for device_start() and device_end() functions

use super::*;

/// Test device_start() and device_end() create DeviceConfig correctly
#[test]
fn test_device_creates_device_config() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test Device");
        map("A", "VK_B");
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    assert_eq!(config.devices.len(), 1);
    assert_eq!(config.devices[0].identifier.pattern, "Test Device");
    assert_eq!(config.devices[0].mappings.len(), 1);
}

/// Test multiple device blocks create separate DeviceConfig entries
#[test]
fn test_multiple_devices() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Device 1");
        map("A", "VK_B");
        device_end();

        device_start("Device 2");
        map("C", "VK_D");
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    assert_eq!(config.devices.len(), 2);
    assert_eq!(config.devices[0].identifier.pattern, "Device 1");
    assert_eq!(config.devices[1].identifier.pattern, "Device 2");
    assert_eq!(config.devices[0].mappings.len(), 1);
    assert_eq!(config.devices[1].mappings.len(), 1);
}

/// Test device with wildcard pattern
#[test]
fn test_device_wildcard_pattern() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("*");
        map("A", "VK_B");
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    assert_eq!(config.devices.len(), 1);
    assert_eq!(config.devices[0].identifier.pattern, "*");
}

/// Test device with multiple mappings
#[test]
fn test_device_with_multiple_mappings() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        map("A", "VK_B");
        map("C", "VK_D");
        map("E", "VK_F");
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    assert_eq!(config.devices.len(), 1);
    assert_eq!(config.devices[0].mappings.len(), 3);
}

/// Test device with different mapping types
#[test]
fn test_device_with_mixed_mapping_types() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        map("A", "VK_B");
        map("CapsLock", "MD_00");
        map("ScrollLock", "LK_01");
        tap_hold("Space", "VK_Space", "MD_01", 200);
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    assert_eq!(config.devices.len(), 1);
    assert_eq!(config.devices[0].mappings.len(), 4);

    // Verify mapping types
    match &config.devices[0].mappings[0] {
        KeyMapping::Base(BaseKeyMapping::Simple { .. }) => {}
        _ => panic!("Expected Simple mapping"),
    }
    match &config.devices[0].mappings[1] {
        KeyMapping::Base(BaseKeyMapping::Modifier { .. }) => {}
        _ => panic!("Expected Modifier mapping"),
    }
    match &config.devices[0].mappings[2] {
        KeyMapping::Base(BaseKeyMapping::Lock { .. }) => {}
        _ => panic!("Expected Lock mapping"),
    }
    match &config.devices[0].mappings[3] {
        KeyMapping::Base(BaseKeyMapping::TapHold { .. }) => {}
        _ => panic!("Expected TapHold mapping"),
    }
}

/// Test device with conditional mappings
#[test]
fn test_device_with_conditional() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        when_start("MD_00");
        map("H", "VK_Left");
        map("L", "VK_Right");
        when_end();
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    assert_eq!(config.devices.len(), 1);
    assert_eq!(config.devices[0].mappings.len(), 1);

    match &config.devices[0].mappings[0] {
        KeyMapping::Conditional { mappings, .. } => {
            assert_eq!(mappings.len(), 2);
        }
        _ => panic!("Expected Conditional mapping"),
    }
}

/// Test unclosed device block returns error
#[test]
fn test_unclosed_device_block_error() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        map("A", "VK_B");
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_err(), "Should fail with unclosed device block");

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Unclosed") || err_msg.contains("device"),
        "Error should mention unclosed device: {}",
        err_msg
    );
}

/// Test device_end() without device_start() returns error
#[test]
fn test_device_end_without_start_error() {
    let mut parser = Parser::new();
    let script = r#"
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_err(), "Should fail without matching device_start");

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("without") || err_msg.contains("matching"),
        "Error should mention missing device_start: {}",
        err_msg
    );
}

/// Test map() outside device block returns error
#[test]
fn test_map_outside_device_error() {
    let mut parser = Parser::new();
    let script = r#"
        map("A", "VK_B");
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(
        result.is_err(),
        "Should fail when map() called outside device"
    );

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("device"),
        "Error should mention device context requirement: {}",
        err_msg
    );
}

/// Test empty device (no mappings)
#[test]
fn test_empty_device() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Empty Device");
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    assert_eq!(config.devices.len(), 1);
    assert_eq!(config.devices[0].identifier.pattern, "Empty Device");
    assert_eq!(config.devices[0].mappings.len(), 0);
}

/// Test device pattern with special characters
#[test]
fn test_device_pattern_special_chars() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("USB:1234:5678");
        map("A", "VK_B");
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    assert_eq!(config.devices.len(), 1);
    assert_eq!(config.devices[0].identifier.pattern, "USB:1234:5678");
}

/// Test realistic example with multiple device blocks
#[test]
fn test_realistic_multi_device_config() {
    let mut parser = Parser::new();
    let script = r#"
        // Default device (all keyboards)
        device_start("*");
        map("CapsLock", "VK_Escape");
        device_end();

        // Specific keyboard
        device_start("Logitech Keyboard");
        map("Enter", "VK_Space");
        map("Space", "VK_Enter");
        device_end();

        // Gaming keyboard
        device_start("Gaming Keyboard");
        map("ScrollLock", "LK_00");
        when_start("LK_00");
        map("W", "VK_W");
        map("A", "VK_A");
        when_end();
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    assert_eq!(config.devices.len(), 3);
    assert_eq!(config.devices[0].identifier.pattern, "*");
    assert_eq!(config.devices[0].mappings.len(), 1);
    assert_eq!(config.devices[1].identifier.pattern, "Logitech Keyboard");
    assert_eq!(config.devices[1].mappings.len(), 2);
    assert_eq!(config.devices[2].identifier.pattern, "Gaming Keyboard");
    assert_eq!(config.devices[2].mappings.len(), 2);
}

/// Test sequential device_start() without device_end() completes previous device
#[test]
fn test_sequential_device_start_completes_previous() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Device 1");
        map("A", "VK_B");
        device_start("Device 2");
        map("C", "VK_D");
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    // device_start on Device 2 should finalize Device 1
    assert_eq!(config.devices.len(), 2);
    assert_eq!(config.devices[0].identifier.pattern, "Device 1");
    assert_eq!(config.devices[0].mappings.len(), 1);
    assert_eq!(config.devices[1].identifier.pattern, "Device 2");
    assert_eq!(config.devices[1].mappings.len(), 1);
}
