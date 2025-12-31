//! Tests for tap_hold() function

use super::*;

/// Test tap_hold() creates TapHold mapping
#[test]
fn test_tap_hold_creates_tap_hold_mapping() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        tap_hold("Space", "VK_Space", "MD_00", 200);
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    assert_eq!(config.devices.len(), 1);
    assert_eq!(config.devices[0].mappings.len(), 1);

    match &config.devices[0].mappings[0] {
        KeyMapping::Base(BaseKeyMapping::TapHold {
            from,
            tap,
            hold_modifier,
            threshold_ms,
        }) => {
            assert_eq!(*from, KeyCode::Space);
            assert_eq!(*tap, KeyCode::Space);
            assert_eq!(*hold_modifier, 0x00);
            assert_eq!(*threshold_ms, 200);
        }
        _ => panic!(
            "Expected TapHold mapping, got {:?}",
            config.devices[0].mappings[0]
        ),
    }
}

/// Test tap_hold() with different keys
#[test]
fn test_tap_hold_different_tap_and_hold() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        tap_hold("CapsLock", "VK_Escape", "MD_01", 250);
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    match &config.devices[0].mappings[0] {
        KeyMapping::Base(BaseKeyMapping::TapHold {
            from,
            tap,
            hold_modifier,
            threshold_ms,
        }) => {
            assert_eq!(*from, KeyCode::CapsLock);
            assert_eq!(*tap, KeyCode::Escape);
            assert_eq!(*hold_modifier, 0x01);
            assert_eq!(*threshold_ms, 250);
        }
        _ => panic!("Expected TapHold mapping"),
    }
}

/// Test tap_hold() rejects tap without VK_ prefix
#[test]
fn test_tap_hold_rejects_tap_without_vk_prefix() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        tap_hold("Space", "MD_00", "MD_01", 200);
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(
        result.is_err(),
        "Should have failed - tap must have VK_ prefix"
    );

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("VK_") || err_msg.contains("tap"),
        "Error should mention VK_ prefix requirement for tap: {}",
        err_msg
    );
}

/// Test tap_hold() rejects hold without MD_ prefix
#[test]
fn test_tap_hold_rejects_hold_without_md_prefix() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        tap_hold("Space", "VK_Space", "VK_LShift", 200);
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(
        result.is_err(),
        "Should have failed - hold must have MD_ prefix"
    );

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("MD_") || err_msg.contains("hold"),
        "Error should mention MD_ prefix requirement for hold: {}",
        err_msg
    );
}

/// Test tap_hold() rejects physical modifier names in hold parameter
#[test]
fn test_tap_hold_rejects_physical_modifier_in_hold() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        tap_hold("Space", "VK_Space", "MD_LShift", 200);
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(
        result.is_err(),
        "Should have failed - physical modifier name in hold"
    );

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("LShift") || err_msg.contains("physical"),
        "Error should mention physical modifier rejection: {}",
        err_msg
    );
}

/// Test tap_hold() with various modifier IDs
#[test]
fn test_tap_hold_various_modifier_ids() {
    let test_ids = vec![0x00, 0x01, 0x7F, 0xFE];

    for id in test_ids {
        let mut parser = Parser::new();

        let script = format!(
            r#"
            device_start("Test");
            tap_hold("Space", "VK_Space", "MD_{:02X}", 200);
            device_end();
            "#,
            id
        );

        let result = parser.parse_string(&script, &PathBuf::from("test.rhai"));
        assert!(
            result.is_ok(),
            "Failed to parse with MD_{:02X}: {:?}",
            id,
            result.err()
        );

        let config = result.unwrap();
        match &config.devices[0].mappings[0] {
            KeyMapping::Base(BaseKeyMapping::TapHold { hold_modifier, .. }) => {
                assert_eq!(*hold_modifier, id);
            }
            _ => panic!("Expected TapHold mapping"),
        }
    }
}

/// Test tap_hold() with different threshold values
#[test]
fn test_tap_hold_different_thresholds() {
    let thresholds = vec![100, 200, 300, 500, 1000];

    for threshold in thresholds {
        let mut parser = Parser::new();

        let script = format!(
            r#"
            device_start("Test");
            tap_hold("Space", "VK_Space", "MD_00", {});
            device_end();
            "#,
            threshold
        );

        let result = parser.parse_string(&script, &PathBuf::from("test.rhai"));
        assert!(
            result.is_ok(),
            "Failed to parse with threshold {}: {:?}",
            threshold,
            result.err()
        );

        let config = result.unwrap();
        match &config.devices[0].mappings[0] {
            KeyMapping::Base(BaseKeyMapping::TapHold { threshold_ms, .. }) => {
                assert_eq!(*threshold_ms, threshold as u16);
            }
            _ => panic!("Expected TapHold mapping"),
        }
    }
}

/// Test tap_hold() must be called inside device block
#[test]
fn test_tap_hold_requires_device_context() {
    let mut parser = Parser::new();
    let script = r#"
        tap_hold("Space", "VK_Space", "MD_00", 200);
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(
        result.is_err(),
        "Should have failed - tap_hold() outside device block"
    );

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("device") || err_msg.contains("block"),
        "Error should mention device requirement: {}",
        err_msg
    );
}

/// Test tap_hold() with invalid key parameter
#[test]
fn test_tap_hold_rejects_invalid_key() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        tap_hold("InvalidKey999", "VK_Space", "MD_00", 200);
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(
        result.is_err(),
        "Should have failed due to invalid key parameter"
    );
}

/// Test tap_hold() with invalid tap key
#[test]
fn test_tap_hold_rejects_invalid_tap_key() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        tap_hold("Space", "VK_InvalidKey999", "MD_00", 200);
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_err(), "Should have failed due to invalid tap key");
}

/// Test tap_hold() with out-of-range modifier ID
#[test]
fn test_tap_hold_rejects_out_of_range_modifier() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        tap_hold("Space", "VK_Space", "MD_FF", 200);
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(
        result.is_err(),
        "Should have failed - MD_FF is out of range"
    );
}

/// Test multiple tap_hold() calls in one device
#[test]
fn test_multiple_tap_hold_calls() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        tap_hold("Space", "VK_Space", "MD_00", 200);
        tap_hold("CapsLock", "VK_Escape", "MD_01", 250);
        tap_hold("Tab", "VK_Tab", "MD_02", 300);
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    assert_eq!(config.devices.len(), 1);
    assert_eq!(config.devices[0].mappings.len(), 3);
}

/// Test tap_hold() with VK_ prefix on key parameter (should work)
#[test]
fn test_tap_hold_accepts_vk_prefix_on_key() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        tap_hold("VK_Space", "VK_Space", "MD_00", 200);
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    assert_eq!(config.devices[0].mappings.len(), 1);
}

/// Test tap_hold() rejects LK_ prefix in hold parameter
#[test]
fn test_tap_hold_rejects_lock_in_hold() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        tap_hold("Space", "VK_Space", "LK_00", 200);
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(
        result.is_err(),
        "Should have failed - hold must have MD_ prefix, not LK_"
    );

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("MD_") || err_msg.contains("hold"),
        "Error should mention MD_ prefix requirement: {}",
        err_msg
    );
}
