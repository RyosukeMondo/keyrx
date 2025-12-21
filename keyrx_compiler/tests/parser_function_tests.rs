//! Unit tests for DSL functions (map, tap_hold, helpers, when, when_not, device)
//!
//! These tests verify that each DSL function works correctly in isolation.

use keyrx_compiler::parser::core::Parser;
use keyrx_core::config::{BaseKeyMapping, KeyCode, KeyMapping};
use std::path::PathBuf;

#[cfg(test)]
mod map_function_tests {
    use super::*;

    /// Test map() with VK_ output creates Simple mapping
    #[test]
    fn test_map_vk_to_vk_creates_simple_mapping() {
        let mut parser = Parser::new();
        let script = r#"
            device_start("Test");
            map("A", "VK_B");
            device_end();
        "#;

        let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let config = result.unwrap();
        assert_eq!(config.devices.len(), 1);
        assert_eq!(config.devices[0].mappings.len(), 1);

        match &config.devices[0].mappings[0] {
            KeyMapping::Base(BaseKeyMapping::Simple { from, to }) => {
                assert_eq!(*from, KeyCode::A);
                assert_eq!(*to, KeyCode::B);
            }
            _ => panic!(
                "Expected Simple mapping, got {:?}",
                config.devices[0].mappings[0]
            ),
        }
    }

    /// Test map() with MD_ output creates Modifier mapping
    #[test]
    fn test_map_vk_to_md_creates_modifier_mapping() {
        let mut parser = Parser::new();
        let script = r#"
            device_start("Test");
            map("CapsLock", "MD_00");
            device_end();
        "#;

        let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let config = result.unwrap();
        assert_eq!(config.devices.len(), 1);
        assert_eq!(config.devices[0].mappings.len(), 1);

        match &config.devices[0].mappings[0] {
            KeyMapping::Base(BaseKeyMapping::Modifier { from, modifier_id }) => {
                assert_eq!(*from, KeyCode::CapsLock);
                assert_eq!(*modifier_id, 0x00);
            }
            _ => panic!(
                "Expected Modifier mapping, got {:?}",
                config.devices[0].mappings[0]
            ),
        }
    }

    /// Test map() with LK_ output creates Lock mapping
    #[test]
    fn test_map_vk_to_lk_creates_lock_mapping() {
        let mut parser = Parser::new();
        let script = r#"
            device_start("Test");
            map("ScrollLock", "LK_01");
            device_end();
        "#;

        let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let config = result.unwrap();
        assert_eq!(config.devices.len(), 1);
        assert_eq!(config.devices[0].mappings.len(), 1);

        match &config.devices[0].mappings[0] {
            KeyMapping::Base(BaseKeyMapping::Lock { from, lock_id }) => {
                assert_eq!(*from, KeyCode::ScrollLock);
                assert_eq!(*lock_id, 0x01);
            }
            _ => panic!(
                "Expected Lock mapping, got {:?}",
                config.devices[0].mappings[0]
            ),
        }
    }

    /// Test map() with VK_ prefix on input key (should work due to parse_physical_key)
    #[test]
    fn test_map_accepts_vk_prefix_on_input() {
        let mut parser = Parser::new();
        let script = r#"
            device_start("Test");
            map("VK_A", "VK_B");
            device_end();
        "#;

        let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let config = result.unwrap();
        assert_eq!(config.devices.len(), 1);
        assert_eq!(config.devices[0].mappings.len(), 1);
    }

    /// Test map() without VK_ prefix on input key (should work)
    #[test]
    fn test_map_accepts_no_prefix_on_input() {
        let mut parser = Parser::new();
        let script = r#"
            device_start("Test");
            map("A", "VK_B");
            device_end();
        "#;

        let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let config = result.unwrap();
        assert_eq!(config.devices.len(), 1);
        assert_eq!(config.devices[0].mappings.len(), 1);
    }

    /// Test map() rejects output without valid prefix
    #[test]
    fn test_map_rejects_missing_output_prefix() {
        let mut parser = Parser::new();
        let script = r#"
            device_start("Test");
            map("A", "B");
            device_end();
        "#;

        let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
        assert!(result.is_err(), "Should have failed due to missing prefix");

        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("VK_") || err_msg.contains("MD_") || err_msg.contains("LK_"),
            "Error should mention prefix requirement: {}",
            err_msg
        );
    }

    /// Test map() with invalid input key
    #[test]
    fn test_map_rejects_invalid_input_key() {
        let mut parser = Parser::new();
        let script = r#"
            device_start("Test");
            map("InvalidKey123", "VK_B");
            device_end();
        "#;

        let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
        assert!(
            result.is_err(),
            "Should have failed due to invalid input key"
        );
    }

    /// Test map() with all modifier IDs (00-FE)
    #[test]
    fn test_map_all_modifier_ids() {
        // Test first, middle, and last valid IDs
        let test_ids = vec![0x00, 0x7F, 0xFE];

        for id in test_ids {
            let mut parser = Parser::new(); // Create new parser for each iteration

            let script = format!(
                r#"
                device_start("Test");
                map("A", "MD_{:02X}");
                device_end();
                "#,
                id
            );

            let result = parser.parse_string(&script, &PathBuf::from("test.rhai"));
            assert!(
                result.is_ok(),
                "Failed to parse MD_{:02X}: {:?}",
                id,
                result.err()
            );

            let config = result.unwrap();
            match &config.devices[0].mappings[0] {
                KeyMapping::Base(BaseKeyMapping::Modifier { modifier_id, .. }) => {
                    assert_eq!(*modifier_id, id);
                }
                _ => panic!("Expected Modifier mapping"),
            }
        }
    }

    /// Test map() with all lock IDs (00-FE)
    #[test]
    fn test_map_all_lock_ids() {
        // Test first, middle, and last valid IDs
        let test_ids = vec![0x00, 0x7F, 0xFE];

        for id in test_ids {
            let mut parser = Parser::new(); // Create new parser for each iteration

            let script = format!(
                r#"
                device_start("Test");
                map("A", "LK_{:02X}");
                device_end();
                "#,
                id
            );

            let result = parser.parse_string(&script, &PathBuf::from("test.rhai"));
            assert!(
                result.is_ok(),
                "Failed to parse LK_{:02X}: {:?}",
                id,
                result.err()
            );

            let config = result.unwrap();
            match &config.devices[0].mappings[0] {
                KeyMapping::Base(BaseKeyMapping::Lock { lock_id, .. }) => {
                    assert_eq!(*lock_id, id);
                }
                _ => panic!("Expected Lock mapping"),
            }
        }
    }

    /// Test map() rejects physical modifier names in MD_ prefix
    #[test]
    fn test_map_rejects_physical_modifier_in_md() {
        let mut parser = Parser::new();
        let script = r#"
            device_start("Test");
            map("CapsLock", "MD_LShift");
            device_end();
        "#;

        let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
        assert!(
            result.is_err(),
            "Should have failed due to physical modifier in MD_"
        );

        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("LShift") || err_msg.contains("physical"),
            "Error should mention physical modifier: {}",
            err_msg
        );
    }

    /// Test map() must be called inside device block
    #[test]
    fn test_map_requires_device_context() {
        let mut parser = Parser::new();
        let script = r#"
            map("A", "VK_B");
        "#;

        let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
        assert!(
            result.is_err(),
            "Should have failed - map() outside device block"
        );

        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("device") || err_msg.contains("block"),
            "Error should mention device requirement: {}",
            err_msg
        );
    }

    /// Test multiple map() calls in one device
    #[test]
    fn test_multiple_map_calls() {
        let mut parser = Parser::new();
        let script = r#"
            device_start("Test");
            map("A", "VK_B");
            map("C", "VK_D");
            map("E", "MD_00");
            map("F", "LK_01");
            device_end();
        "#;

        let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let config = result.unwrap();
        assert_eq!(config.devices.len(), 1);
        assert_eq!(config.devices[0].mappings.len(), 4);
    }

    /// Test map() with special keys
    #[test]
    fn test_map_special_keys() {
        let mut parser = Parser::new();
        let script = r#"
            device_start("Test");
            map("Escape", "VK_CapsLock");
            map("CapsLock", "VK_Escape");
            map("Enter", "VK_Space");
            map("Backspace", "VK_Delete");
            device_end();
        "#;

        let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let config = result.unwrap();
        assert_eq!(config.devices[0].mappings.len(), 4);
    }

    /// Test map() with function keys
    #[test]
    fn test_map_function_keys() {
        let mut parser = Parser::new();
        let script = r#"
            device_start("Test");
            map("F1", "VK_F12");
            map("F12", "VK_F1");
            device_end();
        "#;

        let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let config = result.unwrap();
        assert_eq!(config.devices[0].mappings.len(), 2);
    }

    /// Test map() with arrow keys
    #[test]
    fn test_map_arrow_keys() {
        let mut parser = Parser::new();
        let script = r#"
            device_start("Test");
            map("H", "VK_Left");
            map("J", "VK_Down");
            map("K", "VK_Up");
            map("L", "VK_Right");
            device_end();
        "#;

        let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let config = result.unwrap();
        assert_eq!(config.devices[0].mappings.len(), 4);
    }

    /// Test map() rejects out-of-range modifier ID
    #[test]
    fn test_map_rejects_out_of_range_modifier_id() {
        let mut parser = Parser::new();
        let script = r#"
            device_start("Test");
            map("A", "MD_FF");
            device_end();
        "#;

        let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
        assert!(
            result.is_err(),
            "Should have failed - MD_FF is out of range"
        );
    }

    /// Test map() rejects out-of-range lock ID
    #[test]
    fn test_map_rejects_out_of_range_lock_id() {
        let mut parser = Parser::new();
        let script = r#"
            device_start("Test");
            map("A", "LK_FF");
            device_end();
        "#;

        let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
        assert!(
            result.is_err(),
            "Should have failed - LK_FF is out of range"
        );
    }
}
