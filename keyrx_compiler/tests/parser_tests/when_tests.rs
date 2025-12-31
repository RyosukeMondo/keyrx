//! Tests for when() function

use super::*;

use keyrx_core::config::{Condition, ConditionItem};

/// Test when() with single modifier condition creates Conditional mapping
#[test]
fn test_when_single_modifier_creates_conditional() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
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
    assert_eq!(config.devices[0].mappings.len(), 1);

    match &config.devices[0].mappings[0] {
        KeyMapping::Conditional {
            condition,
            mappings,
        } => {
            // Verify condition is ModifierActive(0x00)
            match condition {
                Condition::ModifierActive(id) => {
                    assert_eq!(*id, 0x00);
                }
                _ => panic!("Expected ModifierActive condition, got {:?}", condition),
            }
            // Verify nested mappings
            assert_eq!(mappings.len(), 2);
        }
        _ => panic!(
            "Expected Conditional mapping, got {:?}",
            config.devices[0].mappings[0]
        ),
    }
}

/// Test when() with single lock condition creates Conditional mapping
#[test]
fn test_when_single_lock_creates_conditional() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        when_start("LK_01");
        map("K", "VK_Up");
        when_end();
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
                Condition::LockActive(id) => {
                    assert_eq!(*id, 0x01);
                }
                _ => panic!("Expected LockActive condition, got {:?}", condition),
            }
            assert_eq!(mappings.len(), 1);
        }
        _ => panic!("Expected Conditional mapping"),
    }
}

/// Test when() with array of conditions creates AllActive conditional
#[test]
fn test_when_array_creates_all_active() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        when_start(["MD_00", "LK_01"]);
        map("A", "VK_B");
        when_end();
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
                Condition::AllActive(items) => {
                    assert_eq!(items.len(), 2);
                    // Verify both conditions present
                    assert!(items.contains(&ConditionItem::ModifierActive(0x00)));
                    assert!(items.contains(&ConditionItem::LockActive(0x01)));
                }
                _ => panic!("Expected AllActive condition, got {:?}", condition),
            }
            assert_eq!(mappings.len(), 1);
        }
        _ => panic!("Expected Conditional mapping"),
    }
}

/// Test when() with multiple modifiers in array
#[test]
fn test_when_multiple_modifiers() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        when_start(["MD_00", "MD_01", "MD_02"]);
        map("H", "VK_Left");
        when_end();
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    match &config.devices[0].mappings[0] {
        KeyMapping::Conditional { condition, .. } => match condition {
            Condition::AllActive(items) => {
                assert_eq!(items.len(), 3);
                assert!(items.contains(&ConditionItem::ModifierActive(0x00)));
                assert!(items.contains(&ConditionItem::ModifierActive(0x01)));
                assert!(items.contains(&ConditionItem::ModifierActive(0x02)));
            }
            _ => panic!("Expected AllActive condition"),
        },
        _ => panic!("Expected Conditional mapping"),
    }
}

/// Test when() collects multiple nested mappings
#[test]
fn test_when_multiple_nested_mappings() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        when_start("MD_00");
        map("H", "VK_Left");
        map("J", "VK_Down");
        map("K", "VK_Up");
        map("L", "VK_Right");
        when_end();
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    match &config.devices[0].mappings[0] {
        KeyMapping::Conditional { mappings, .. } => {
            assert_eq!(mappings.len(), 4);
        }
        _ => panic!("Expected Conditional mapping"),
    }
}

/// Test when() with different mapping types
#[test]
fn test_when_mixed_mapping_types() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        when_start("MD_00");
        map("A", "VK_B");
        map("CapsLock", "MD_01");
        tap_hold("Space", "VK_Space", "MD_02", 200);
        when_end();
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    match &config.devices[0].mappings[0] {
        KeyMapping::Conditional { mappings, .. } => {
            assert_eq!(mappings.len(), 3);
            // Verify different mapping types
            match &mappings[0] {
                BaseKeyMapping::Simple { .. } => {}
                _ => panic!("Expected Simple mapping"),
            }
            match &mappings[1] {
                BaseKeyMapping::Modifier { .. } => {}
                _ => panic!("Expected Modifier mapping"),
            }
            match &mappings[2] {
                BaseKeyMapping::TapHold { .. } => {}
                _ => panic!("Expected TapHold mapping"),
            }
        }
        _ => panic!("Expected Conditional mapping"),
    }
}

/// Test when() with invalid condition string
#[test]
fn test_when_invalid_condition() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        when_start("INVALID_00");
        map("A", "VK_B");
        when_end();
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_err(), "Should fail with invalid condition");

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("MD_") || err_msg.contains("LK_"),
        "Error should mention valid condition formats: {}",
        err_msg
    );
}

/// Test when() with out-of-range modifier ID
#[test]
fn test_when_out_of_range_modifier() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        when_start("MD_FF");
        map("A", "VK_B");
        when_end();
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_err(), "Should fail with out-of-range modifier");
}

/// Test when() requires device context
#[test]
fn test_when_requires_device_context() {
    let mut parser = Parser::new();
    let script = r#"
        when_start("MD_00");
        map("A", "VK_B");
        when_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_err(), "Should fail - when() outside device block");

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("device"),
        "Error should mention device requirement: {}",
        err_msg
    );
}

/// Test unclosed when block (auto-closed by device_end, mappings inside are lost)
#[test]
fn test_unclosed_when_block_auto_closes() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        when_start("MD_00");
        map("A", "VK_B");
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    // Unclosed when blocks are auto-closed when device_end is called
    // but mappings inside are discarded
    assert!(
        result.is_ok(),
        "Should auto-close when block: {:?}",
        result.err()
    );

    let config = result.unwrap();
    assert_eq!(config.devices.len(), 1);
    // The when block was auto-closed but mappings inside were lost
    assert_eq!(config.devices[0].mappings.len(), 0);
}

/// Test when_end without when_start
#[test]
fn test_when_end_without_start() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        when_end();
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_err(), "Should fail without matching when_start");

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("without") || err_msg.contains("matching"),
        "Error should mention missing when_start: {}",
        err_msg
    );
}

/// Test nested when blocks (should not be allowed)
#[test]
fn test_nested_when_blocks_error() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        when_start("MD_00");
        when_start("LK_01");
        map("A", "VK_B");
        when_end();
        when_end();
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    // Nested when blocks are not supported
    assert!(result.is_err(), "Should fail with nested when blocks");

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Nested") || err_msg.contains("nested") || err_msg.contains("when"),
        "Error should mention nested conditional blocks: {}",
        err_msg
    );
}

/// Test realistic vim-style navigation
#[test]
fn test_realistic_vim_navigation() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        map("CapsLock", "MD_00");
        when_start("MD_00");
        map("H", "VK_Left");
        map("J", "VK_Down");
        map("K", "VK_Up");
        map("L", "VK_Right");
        when_end();
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    assert_eq!(config.devices.len(), 1);
    assert_eq!(config.devices[0].mappings.len(), 2);

    // First mapping: CapsLock -> MD_00
    match &config.devices[0].mappings[0] {
        KeyMapping::Base(BaseKeyMapping::Modifier { modifier_id, .. }) => {
            assert_eq!(*modifier_id, 0x00);
        }
        _ => panic!("Expected Modifier mapping"),
    }

    // Second mapping: Conditional with 4 arrow key mappings
    match &config.devices[0].mappings[1] {
        KeyMapping::Conditional { mappings, .. } => {
            assert_eq!(mappings.len(), 4);
        }
        _ => panic!("Expected Conditional mapping"),
    }
}

/// Test multiple when blocks in same device
#[test]
fn test_multiple_when_blocks() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        when_start("MD_00");
        map("H", "VK_Left");
        when_end();
        when_start("MD_01");
        map("L", "VK_Right");
        when_end();
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    assert_eq!(config.devices.len(), 1);
    assert_eq!(config.devices[0].mappings.len(), 2);

    // Both should be Conditional mappings
    match &config.devices[0].mappings[0] {
        KeyMapping::Conditional { .. } => {}
        _ => panic!("Expected Conditional mapping"),
    }
    match &config.devices[0].mappings[1] {
        KeyMapping::Conditional { .. } => {}
        _ => panic!("Expected Conditional mapping"),
    }
}

/// Test empty when block
#[test]
fn test_empty_when_block() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        when_start("MD_00");
        when_end();
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
