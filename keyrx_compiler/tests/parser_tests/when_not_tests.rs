//! Tests for when_not() function

use super::*;

use keyrx_core::config::{Condition, ConditionItem};

/// Test when_not() with modifier creates NotActive condition
#[test]
fn test_when_not_modifier_creates_not_active() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        when_not_start("MD_00");
        map("K", "VK_Up");
        when_not_end();
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
                Condition::NotActive(items) => {
                    assert_eq!(items.len(), 1);
                    assert!(items.contains(&ConditionItem::ModifierActive(0x00)));
                }
                _ => panic!("Expected NotActive condition, got {:?}", condition),
            }
            assert_eq!(mappings.len(), 1);
        }
        _ => panic!("Expected Conditional mapping"),
    }
}

/// Test when_not() with lock creates NotActive condition
#[test]
fn test_when_not_lock_creates_not_active() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        when_not_start("LK_01");
        map("A", "VK_B");
        when_not_end();
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    match &config.devices[0].mappings[0] {
        KeyMapping::Conditional {
            condition,
            mappings,
        } => {
            match condition {
                Condition::NotActive(items) => {
                    assert_eq!(items.len(), 1);
                    assert!(items.contains(&ConditionItem::LockActive(0x01)));
                }
                _ => panic!("Expected NotActive condition"),
            }
            assert_eq!(mappings.len(), 1);
        }
        _ => panic!("Expected Conditional mapping"),
    }
}

/// Test when_not() collects multiple nested mappings
#[test]
fn test_when_not_multiple_mappings() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        when_not_start("MD_00");
        map("A", "VK_B");
        map("C", "VK_D");
        map("E", "VK_F");
        when_not_end();
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    match &config.devices[0].mappings[0] {
        KeyMapping::Conditional { mappings, .. } => {
            assert_eq!(mappings.len(), 3);
        }
        _ => panic!("Expected Conditional mapping"),
    }
}

/// Test when_not() with invalid condition
#[test]
fn test_when_not_invalid_condition() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        when_not_start("INVALID_XX");
        map("A", "VK_B");
        when_not_end();
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_err(), "Should fail with invalid condition");
}

/// Test when_not() with out-of-range ID
#[test]
fn test_when_not_out_of_range() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        when_not_start("MD_FF");
        map("A", "VK_B");
        when_not_end();
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_err(), "Should fail with out-of-range ID");
}

/// Test when_not() requires device context
#[test]
fn test_when_not_requires_device_context() {
    let mut parser = Parser::new();
    let script = r#"
        when_not_start("MD_00");
        map("A", "VK_B");
        when_not_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(
        result.is_err(),
        "Should fail - when_not() outside device block"
    );

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("device"),
        "Error should mention device requirement: {}",
        err_msg
    );
}

/// Test unclosed when_not block (auto-closed by device_end, mappings inside are lost)
#[test]
fn test_unclosed_when_not_block_auto_closes() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        when_not_start("MD_00");
        map("A", "VK_B");
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    // Unclosed when_not blocks are auto-closed when device_end is called
    // but mappings inside are discarded
    assert!(
        result.is_ok(),
        "Should auto-close when_not block: {:?}",
        result.err()
    );

    let config = result.unwrap();
    assert_eq!(config.devices.len(), 1);
    // The when_not block was auto-closed but mappings inside were lost
    assert_eq!(config.devices[0].mappings.len(), 0);
}

/// Test when_not_end without when_not_start
#[test]
fn test_when_not_end_without_start() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        when_not_end();
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(
        result.is_err(),
        "Should fail without matching when_not_start"
    );

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("without") || err_msg.contains("matching"),
        "Error should mention missing when_not_start: {}",
        err_msg
    );
}

/// Test realistic example: disable remapping when gaming mode inactive
#[test]
fn test_realistic_when_not_gaming_mode() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        map("ScrollLock", "LK_00");
        when_not_start("LK_00");
        map("CapsLock", "VK_Escape");
        when_not_end();
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    assert_eq!(config.devices.len(), 1);
    assert_eq!(config.devices[0].mappings.len(), 2);

    // First mapping: ScrollLock -> LK_00
    match &config.devices[0].mappings[0] {
        KeyMapping::Base(BaseKeyMapping::Lock { lock_id, .. }) => {
            assert_eq!(*lock_id, 0x00);
        }
        _ => panic!("Expected Lock mapping"),
    }

    // Second mapping: when_not LK_00, CapsLock -> Escape
    match &config.devices[0].mappings[1] {
        KeyMapping::Conditional {
            condition,
            mappings,
        } => {
            match condition {
                Condition::NotActive(items) => {
                    assert_eq!(items.len(), 1);
                    assert!(items.contains(&ConditionItem::LockActive(0x00)));
                }
                _ => panic!("Expected NotActive condition"),
            }
            assert_eq!(mappings.len(), 1);
        }
        _ => panic!("Expected Conditional mapping"),
    }
}

/// Test when_not() with different mapping types
#[test]
fn test_when_not_mixed_mapping_types() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        when_not_start("MD_00");
        map("A", "VK_B");
        tap_hold("Space", "VK_Space", "MD_01", 200);
        when_not_end();
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    match &config.devices[0].mappings[0] {
        KeyMapping::Conditional { mappings, .. } => {
            assert_eq!(mappings.len(), 2);
        }
        _ => panic!("Expected Conditional mapping"),
    }
}

/// Test multiple when_not blocks in same device
#[test]
fn test_multiple_when_not_blocks() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        when_not_start("MD_00");
        map("A", "VK_B");
        when_not_end();
        when_not_start("LK_01");
        map("C", "VK_D");
        when_not_end();
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    assert_eq!(config.devices.len(), 1);
    assert_eq!(config.devices[0].mappings.len(), 2);

    // Both should be Conditional mappings with NotActive
    for mapping in &config.devices[0].mappings {
        match mapping {
            KeyMapping::Conditional { condition, .. } => match condition {
                Condition::NotActive(_) => {}
                _ => panic!("Expected NotActive condition"),
            },
            _ => panic!("Expected Conditional mapping"),
        }
    }
}

/// Test empty when_not block
#[test]
fn test_empty_when_not_block() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        when_not_start("MD_00");
        when_not_end();
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    assert_eq!(config.devices[0].mappings.len(), 1);

    match &config.devices[0].mappings[0] {
        KeyMapping::Conditional { mappings, .. } => {
            assert_eq!(mappings.len(), 0);
        }
        _ => panic!("Expected Conditional mapping"),
    }
}

/// Test combining when() and when_not() in same device
#[test]
fn test_when_and_when_not_combined() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        when_start("MD_00");
        map("H", "VK_Left");
        when_end();
        when_not_start("MD_00");
        map("J", "VK_Down");
        when_not_end();
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    assert_eq!(config.devices.len(), 1);
    assert_eq!(config.devices[0].mappings.len(), 2);

    // First should be when (ModifierActive or AllActive)
    match &config.devices[0].mappings[0] {
        KeyMapping::Conditional { condition, .. } => match condition {
            Condition::ModifierActive(_) | Condition::AllActive(_) => {}
            _ => panic!("Expected positive condition for when()"),
        },
        _ => panic!("Expected Conditional mapping"),
    }

    // Second should be when_not (NotActive)
    match &config.devices[0].mappings[1] {
        KeyMapping::Conditional { condition, .. } => match condition {
            Condition::NotActive(_) => {}
            _ => panic!("Expected NotActive condition for when_not()"),
        },
        _ => panic!("Expected Conditional mapping"),
    }
}
