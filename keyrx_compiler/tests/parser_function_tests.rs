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

#[cfg(test)]
mod tap_hold_function_tests {
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
}

#[cfg(test)]
mod modifier_helper_tests {
    use super::*;

    /// Test with_shift() creates ModifiedOutput mapping with shift=true
    #[test]
    fn test_with_shift_creates_modified_output() {
        let mut parser = Parser::new();
        let script = r#"
            device_start("Test");
            map("VK_2", with_shift("VK_1"));
            device_end();
        "#;

        let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let config = result.unwrap();
        assert_eq!(config.devices.len(), 1);
        assert_eq!(config.devices[0].mappings.len(), 1);

        match &config.devices[0].mappings[0] {
            KeyMapping::Base(BaseKeyMapping::ModifiedOutput {
                from,
                to,
                shift,
                ctrl,
                alt,
                win,
            }) => {
                assert_eq!(*from, KeyCode::Num2);
                assert_eq!(*to, KeyCode::Num1);
                assert_eq!(*shift, true);
                assert_eq!(*ctrl, false);
                assert_eq!(*alt, false);
                assert_eq!(*win, false);
            }
            _ => panic!(
                "Expected ModifiedOutput mapping, got {:?}",
                config.devices[0].mappings[0]
            ),
        }
    }

    /// Test with_ctrl() creates ModifiedOutput mapping with ctrl=true
    #[test]
    fn test_with_ctrl_creates_modified_output() {
        let mut parser = Parser::new();
        let script = r#"
            device_start("Test");
            map("VK_A", with_ctrl("VK_C"));
            device_end();
        "#;

        let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let config = result.unwrap();
        match &config.devices[0].mappings[0] {
            KeyMapping::Base(BaseKeyMapping::ModifiedOutput {
                from,
                to,
                shift,
                ctrl,
                alt,
                win,
            }) => {
                assert_eq!(*from, KeyCode::A);
                assert_eq!(*to, KeyCode::C);
                assert_eq!(*shift, false);
                assert_eq!(*ctrl, true);
                assert_eq!(*alt, false);
                assert_eq!(*win, false);
            }
            _ => panic!("Expected ModifiedOutput mapping"),
        }
    }

    /// Test with_alt() creates ModifiedOutput mapping with alt=true
    #[test]
    fn test_with_alt_creates_modified_output() {
        let mut parser = Parser::new();
        let script = r#"
            device_start("Test");
            map("VK_F1", with_alt("VK_F4"));
            device_end();
        "#;

        let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let config = result.unwrap();
        match &config.devices[0].mappings[0] {
            KeyMapping::Base(BaseKeyMapping::ModifiedOutput {
                from,
                to,
                shift,
                ctrl,
                alt,
                win,
            }) => {
                assert_eq!(*from, KeyCode::F1);
                assert_eq!(*to, KeyCode::F4);
                assert_eq!(*shift, false);
                assert_eq!(*ctrl, false);
                assert_eq!(*alt, true);
                assert_eq!(*win, false);
            }
            _ => panic!("Expected ModifiedOutput mapping"),
        }
    }

    /// Test with_win() creates ModifiedOutput mapping with win=true
    #[test]
    fn test_with_win_creates_modified_output() {
        let mut parser = Parser::new();
        let script = r#"
            device_start("Test");
            map("VK_L", with_win("VK_L"));
            device_end();
        "#;

        let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let config = result.unwrap();
        match &config.devices[0].mappings[0] {
            KeyMapping::Base(BaseKeyMapping::ModifiedOutput {
                from,
                to,
                shift,
                ctrl,
                alt,
                win,
            }) => {
                assert_eq!(*from, KeyCode::L);
                assert_eq!(*to, KeyCode::L);
                assert_eq!(*shift, false);
                assert_eq!(*ctrl, false);
                assert_eq!(*alt, false);
                assert_eq!(*win, true);
            }
            _ => panic!("Expected ModifiedOutput mapping"),
        }
    }

    /// Test with_mods() with multiple modifiers
    #[test]
    fn test_with_mods_multiple_modifiers() {
        let mut parser = Parser::new();
        let script = r#"
            device_start("Test");
            map("VK_1", with_mods("VK_2", true, true, false, false));
            device_end();
        "#;

        let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let config = result.unwrap();
        match &config.devices[0].mappings[0] {
            KeyMapping::Base(BaseKeyMapping::ModifiedOutput {
                from,
                to,
                shift,
                ctrl,
                alt,
                win,
            }) => {
                assert_eq!(*from, KeyCode::Num1);
                assert_eq!(*to, KeyCode::Num2);
                assert_eq!(*shift, true);
                assert_eq!(*ctrl, true);
                assert_eq!(*alt, false);
                assert_eq!(*win, false);
            }
            _ => panic!("Expected ModifiedOutput mapping"),
        }
    }

    /// Test with_mods() with all modifiers enabled
    #[test]
    fn test_with_mods_all_modifiers() {
        let mut parser = Parser::new();
        let script = r#"
            device_start("Test");
            map("VK_A", with_mods("VK_Z", true, true, true, true));
            device_end();
        "#;

        let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let config = result.unwrap();
        match &config.devices[0].mappings[0] {
            KeyMapping::Base(BaseKeyMapping::ModifiedOutput {
                from,
                to,
                shift,
                ctrl,
                alt,
                win,
            }) => {
                assert_eq!(*from, KeyCode::A);
                assert_eq!(*to, KeyCode::Z);
                assert_eq!(*shift, true);
                assert_eq!(*ctrl, true);
                assert_eq!(*alt, true);
                assert_eq!(*win, true);
            }
            _ => panic!("Expected ModifiedOutput mapping"),
        }
    }

    /// Test with_mods() with no modifiers
    #[test]
    fn test_with_mods_no_modifiers() {
        let mut parser = Parser::new();
        let script = r#"
            device_start("Test");
            map("VK_X", with_mods("VK_Y", false, false, false, false));
            device_end();
        "#;

        let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let config = result.unwrap();
        match &config.devices[0].mappings[0] {
            KeyMapping::Base(BaseKeyMapping::ModifiedOutput {
                from,
                to,
                shift,
                ctrl,
                alt,
                win,
            }) => {
                assert_eq!(*from, KeyCode::X);
                assert_eq!(*to, KeyCode::Y);
                assert_eq!(*shift, false);
                assert_eq!(*ctrl, false);
                assert_eq!(*alt, false);
                assert_eq!(*win, false);
            }
            _ => panic!("Expected ModifiedOutput mapping"),
        }
    }

    /// Test multiple modifier helper calls in same device
    #[test]
    fn test_multiple_modifier_helpers() {
        let mut parser = Parser::new();
        let script = r#"
            device_start("Test");
            map("VK_1", with_shift("VK_1"));
            map("VK_2", with_ctrl("VK_2"));
            map("VK_3", with_alt("VK_3"));
            device_end();
        "#;

        let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let config = result.unwrap();
        assert_eq!(config.devices.len(), 1);
        assert_eq!(config.devices[0].mappings.len(), 3);
    }

    /// Test with_shift() rejects invalid key
    #[test]
    fn test_with_shift_rejects_invalid_key() {
        let mut parser = Parser::new();
        let script = r#"
            device_start("Test");
            map("VK_A", with_shift("VK_InvalidKey"));
            device_end();
        "#;

        let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
        assert!(result.is_err());

        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Invalid") || err_msg.contains("key"),
            "Error should mention invalid key: {}",
            err_msg
        );
    }

    /// Test with_ctrl() rejects missing VK_ prefix
    #[test]
    fn test_with_ctrl_rejects_missing_prefix() {
        let mut parser = Parser::new();
        let script = r#"
            device_start("Test");
            map("VK_A", with_ctrl("C"));
            device_end();
        "#;

        let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
        assert!(result.is_err());

        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("VK_") || err_msg.contains("prefix"),
            "Error should mention VK_ prefix requirement: {}",
            err_msg
        );
    }

    /// Test with_mods() rejects invalid key
    #[test]
    fn test_with_mods_rejects_invalid_key() {
        let mut parser = Parser::new();
        let script = r#"
            device_start("Test");
            map("VK_A", with_mods("VK_NoSuchKey", true, false, false, false));
            device_end();
        "#;

        let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
        assert!(result.is_err());

        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Invalid") || err_msg.contains("key"),
            "Error should mention invalid key: {}",
            err_msg
        );
    }

    /// Test map() with ModifiedKey requires device context
    #[test]
    fn test_modifier_helper_requires_device_context() {
        let mut parser = Parser::new();
        let script = r#"
            map("VK_A", with_shift("VK_B"));
        "#;

        let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
        assert!(result.is_err());

        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("device"),
            "Error should mention device context requirement: {}",
            err_msg
        );
    }

    /// Test with_shift() with function keys
    #[test]
    fn test_with_shift_function_keys() {
        let mut parser = Parser::new();
        let script = r#"
            device_start("Test");
            map("VK_F12", with_shift("VK_F1"));
            device_end();
        "#;

        let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let config = result.unwrap();
        match &config.devices[0].mappings[0] {
            KeyMapping::Base(BaseKeyMapping::ModifiedOutput { from, to, .. }) => {
                assert_eq!(*from, KeyCode::F12);
                assert_eq!(*to, KeyCode::F1);
            }
            _ => panic!("Expected ModifiedOutput mapping"),
        }
    }

    /// Test with_ctrl() with special keys
    #[test]
    fn test_with_ctrl_special_keys() {
        let mut parser = Parser::new();
        let script = r#"
            device_start("Test");
            map("VK_Tab", with_ctrl("VK_Tab"));
            device_end();
        "#;

        let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let config = result.unwrap();
        match &config.devices[0].mappings[0] {
            KeyMapping::Base(BaseKeyMapping::ModifiedOutput { from, to, .. }) => {
                assert_eq!(*from, KeyCode::Tab);
                assert_eq!(*to, KeyCode::Tab);
            }
            _ => panic!("Expected ModifiedOutput mapping"),
        }
    }

    /// Test realistic example: Shift+Number for symbols
    #[test]
    fn test_realistic_shift_number_for_symbol() {
        let mut parser = Parser::new();
        let script = r#"
            device_start("Test");
            map("VK_9", with_shift("VK_8"));  // Remap 9 to Shift+8 (*)
            device_end();
        "#;

        let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let config = result.unwrap();
        match &config.devices[0].mappings[0] {
            KeyMapping::Base(BaseKeyMapping::ModifiedOutput {
                from, to, shift, ..
            }) => {
                assert_eq!(*from, KeyCode::Num9);
                assert_eq!(*to, KeyCode::Num8);
                assert_eq!(*shift, true);
            }
            _ => panic!("Expected ModifiedOutput mapping"),
        }
    }

    /// Test realistic example: Ctrl+C for copy
    #[test]
    fn test_realistic_ctrl_c_copy() {
        let mut parser = Parser::new();
        let script = r#"
            device_start("Test");
            map("VK_F2", with_ctrl("VK_C"));  // F2 sends Ctrl+C
            device_end();
        "#;

        let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let config = result.unwrap();
        match &config.devices[0].mappings[0] {
            KeyMapping::Base(BaseKeyMapping::ModifiedOutput { from, to, ctrl, .. }) => {
                assert_eq!(*from, KeyCode::F2);
                assert_eq!(*to, KeyCode::C);
                assert_eq!(*ctrl, true);
            }
            _ => panic!("Expected ModifiedOutput mapping"),
        }
    }
}

#[cfg(test)]
mod device_function_tests {
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
}
