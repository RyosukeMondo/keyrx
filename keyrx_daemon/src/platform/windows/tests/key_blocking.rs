//! Integration tests for dynamic key blocking
//!
//! Tests the complete pipeline:
//! 1. Load .krx config
//! 2. Extract all source keys
//! 3. Convert to scan codes
//! 4. Configure blocker

#[cfg(test)]
mod tests {
    use keyrx_compiler::serialize::{deserialize, serialize};
    use keyrx_core::config::{BaseKeyMapping, ConfigRoot, DeviceConfig, KeyCode, KeyMapping};
    use keyrx_core::config::{Condition, Metadata, Version};
    use std::collections::HashSet;

    /// Create a test config with multiple layers and mappings
    fn create_comprehensive_test_config() -> ConfigRoot {
        use alloc::string::ToString;
        use alloc::vec;

        // Base layer mappings
        let base_mappings = vec![
            // Simple mappings
            KeyMapping::Base(BaseKeyMapping::Simple {
                from: KeyCode::W,
                to: KeyCode::A,
            }),
            KeyMapping::Base(BaseKeyMapping::Simple {
                from: KeyCode::E,
                to: KeyCode::O,
            }),
            KeyMapping::Base(BaseKeyMapping::Simple {
                from: KeyCode::O,
                to: KeyCode::T,
            }),
            // Tap-hold (these should also be blocked)
            KeyMapping::Base(BaseKeyMapping::TapHold {
                from: KeyCode::B,
                tap: KeyCode::Enter,
                hold_modifier: 0, // MD_00
                threshold_us: 200_000,
            }),
            KeyMapping::Base(BaseKeyMapping::TapHold {
                from: KeyCode::V,
                tap: KeyCode::Delete,
                hold_modifier: 1, // MD_01
                threshold_us: 200_000,
            }),
        ];

        // MD_00 layer (B held)
        let md00_mappings = vec![
            KeyMapping::Conditional {
                condition: Condition::ModifierActive(0),
                mappings: vec![
                    BaseKeyMapping::Simple {
                        from: KeyCode::W,
                        to: KeyCode::Num1,
                    },
                    BaseKeyMapping::Simple {
                        from: KeyCode::E,
                        to: KeyCode::Num2,
                    },
                    BaseKeyMapping::Simple {
                        from: KeyCode::O,
                        to: KeyCode::Num8,
                    },
                ],
            },
        ];

        // MD_01 layer (V held)
        let md01_mappings = vec![
            KeyMapping::Conditional {
                condition: Condition::ModifierActive(1),
                mappings: vec![
                    BaseKeyMapping::Simple {
                        from: KeyCode::W,
                        to: KeyCode::S,
                    },
                    BaseKeyMapping::Simple {
                        from: KeyCode::E,
                        to: KeyCode::N,
                    },
                    BaseKeyMapping::Simple {
                        from: KeyCode::O,
                        to: KeyCode::E,
                    },
                ],
            },
        ];

        // Combine all mappings
        let mut all_mappings = base_mappings;
        all_mappings.extend(md00_mappings);
        all_mappings.extend(md01_mappings);

        ConfigRoot {
            version: Version {
                major: 1,
                minor: 0,
                patch: 0,
            },
            devices: vec![DeviceConfig {
                identifier: keyrx_core::config::DeviceIdentifier::Any,
                mappings: all_mappings,
            }],
            metadata: Metadata {
                name: "test".to_string(),
                description: "Test config".to_string(),
                author: "test".to_string(),
                version: "1.0.0".to_string(),
                created_at: "2024-01-01T00:00:00Z".to_string(),
                compiler_version: "0.1.0".to_string(),
                source_hash: "test".to_string(),
            },
        }
    }

    /// Extract all source keys from a config (mimics platform_state logic)
    fn extract_source_keys(config: &ConfigRoot) -> HashSet<KeyCode> {
        let mut keys = HashSet::new();

        for device in &config.devices {
            for mapping in &device.mappings {
                extract_from_mapping(mapping, &mut keys);
            }
        }

        keys
    }

    /// Recursively extract source keys from a mapping
    fn extract_from_mapping(mapping: &KeyMapping, keys: &mut HashSet<KeyCode>) {
        match mapping {
            KeyMapping::Base(base) => {
                let key = match base {
                    BaseKeyMapping::Simple { from, .. } => *from,
                    BaseKeyMapping::Modifier { from, .. } => *from,
                    BaseKeyMapping::Lock { from, .. } => *from,
                    BaseKeyMapping::TapHold { from, .. } => *from,
                    BaseKeyMapping::ModifiedOutput { from, .. } => *from,
                };
                keys.insert(key);
            }
            KeyMapping::Conditional { mappings, .. } => {
                for base in mappings {
                    let key = match base {
                        BaseKeyMapping::Simple { from, .. } => *from,
                        BaseKeyMapping::Modifier { from, .. } => *from,
                        BaseKeyMapping::Lock { from, .. } => *from,
                        BaseKeyMapping::TapHold { from, .. } => *from,
                        BaseKeyMapping::ModifiedOutput { from, .. } => *from,
                    };
                    keys.insert(key);
                }
            }
        }
    }

    #[test]
    fn test_extract_unique_source_keys() {
        let config = create_comprehensive_test_config();
        let keys = extract_source_keys(&config);

        // W, E, O appear in multiple layers, but should be extracted once
        // B, V are tap-hold keys
        // Expected unique keys: W, E, O, B, V = 5 keys
        assert_eq!(
            keys.len(),
            5,
            "Should extract 5 unique source keys (W, E, O, B, V)"
        );

        assert!(keys.contains(&KeyCode::W), "Should extract W");
        assert!(keys.contains(&KeyCode::E), "Should extract E");
        assert!(keys.contains(&KeyCode::O), "Should extract O");
        assert!(keys.contains(&KeyCode::B), "Should extract B (tap-hold)");
        assert!(keys.contains(&KeyCode::V), "Should extract V (tap-hold)");
    }

    #[test]
    fn test_config_roundtrip() {
        let config = create_comprehensive_test_config();

        // Serialize to bytes
        let bytes = serialize(&config).expect("Serialization should succeed");

        // Deserialize back
        let archived = deserialize(&bytes).expect("Deserialization should succeed");

        // Verify structure
        assert_eq!(archived.devices.len(), 1);
        assert_eq!(archived.devices[0].mappings.len(), 7); // 5 base + 2 conditional blocks
    }

    #[test]
    fn test_keycode_to_scancode_coverage() {
        use crate::platform::windows::keycode::keycode_to_scancode;

        // Test keys from user_layout.rhai
        let test_keys = vec![
            (KeyCode::W, Some(0x11)),
            (KeyCode::E, Some(0x12)),
            (KeyCode::O, Some(0x18)),
            (KeyCode::B, Some(0x30)),
            (KeyCode::V, Some(0x2F)),
            (KeyCode::M, Some(0x32)),
            (KeyCode::X, Some(0x2D)),
        ];

        for (keycode, expected_scan) in test_keys {
            let scan = keycode_to_scancode(keycode);
            assert_eq!(
                scan, expected_scan,
                "{:?} should convert to scan code {:?}",
                keycode, expected_scan
            );
        }
    }

    #[test]
    fn test_extended_keys_have_e000_prefix() {
        use crate::platform::windows::keycode::keycode_to_scancode;

        // Extended keys should have 0xE000 prefix
        let extended_keys = vec![
            KeyCode::Insert,
            KeyCode::Delete,
            KeyCode::Home,
            KeyCode::End,
            KeyCode::PageUp,
            KeyCode::PageDown,
            KeyCode::Left,
            KeyCode::Right,
            KeyCode::Up,
            KeyCode::Down,
        ];

        for keycode in extended_keys {
            let scan = keycode_to_scancode(keycode).expect("Should have scan code");
            assert!(
                scan & 0xE000 == 0xE000,
                "{:?} should have 0xE000 prefix, got 0x{:04X}",
                keycode,
                scan
            );
        }
    }

    #[test]
    fn test_all_user_layout_keys_convertible() {
        use crate::platform::windows::keycode::keycode_to_scancode;

        // All keys from user_layout.rhai that appear in mappings
        let user_keys = vec![
            // Base layer
            KeyCode::W,
            KeyCode::E,
            KeyCode::R,
            KeyCode::T,
            KeyCode::Y,
            KeyCode::U,
            KeyCode::I,
            KeyCode::O,
            KeyCode::P,
            KeyCode::A,
            KeyCode::S,
            KeyCode::D,
            KeyCode::F,
            KeyCode::G,
            KeyCode::H,
            KeyCode::J,
            KeyCode::K,
            KeyCode::L,
            KeyCode::Z,
            KeyCode::X,
            KeyCode::C,
            KeyCode::V,
            KeyCode::B,
            KeyCode::N,
            KeyCode::M,
            // Tap-hold keys
            KeyCode::Q,
            KeyCode::LCtrl,
            KeyCode::Tab,
            // Numbers
            KeyCode::Num1,
            KeyCode::Num2,
            KeyCode::Num3,
            KeyCode::Num4,
            KeyCode::Num5,
            KeyCode::Num6,
            KeyCode::Num7,
            KeyCode::Num8,
            KeyCode::Num9,
            KeyCode::Num0,
            // Symbols
            KeyCode::Minus,
            KeyCode::Equal,
            KeyCode::LeftBracket,
            KeyCode::RightBracket,
            KeyCode::Backslash,
            KeyCode::Semicolon,
            KeyCode::Quote,
            KeyCode::Comma,
            KeyCode::Period,
            KeyCode::Slash,
            // Special
            KeyCode::Space,
            KeyCode::Enter,
            KeyCode::Backspace,
            KeyCode::Delete,
        ];

        let mut failed = Vec::new();
        for keycode in user_keys {
            if keycode_to_scancode(keycode).is_none() {
                failed.push(keycode);
            }
        }

        assert!(
            failed.is_empty(),
            "These keys from user_layout.rhai cannot be converted to scan codes: {:?}",
            failed
        );
    }

    #[test]
    fn test_load_real_user_layout() {
        use std::path::PathBuf;

        // Try to load the actual user_layout.krx
        let krx_path = PathBuf::from("examples/user_layout.krx");

        if !krx_path.exists() {
            println!("Skipping test: examples/user_layout.krx not found");
            return;
        }

        let bytes = std::fs::read(&krx_path).expect("Should read user_layout.krx");

        let archived = deserialize(&bytes).expect("Should deserialize user_layout.krx");

        // Extract source keys
        let owned: ConfigRoot = archived
            .deserialize(&mut rkyv::Infallible)
            .expect("Should convert to owned");

        let keys = extract_source_keys(&owned);

        println!("Extracted {} unique source keys from user_layout.krx", keys.len());

        // user_layout.rhai has many mappings, should have at least 30 unique source keys
        assert!(
            keys.len() >= 30,
            "user_layout.krx should have at least 30 unique source keys, got {}",
            keys.len()
        );

        // Verify O key is extracted
        assert!(
            keys.contains(&KeyCode::O),
            "O key should be extracted from user_layout.krx"
        );

        // Verify tap-hold keys are extracted
        assert!(
            keys.contains(&KeyCode::B),
            "B (tap-hold) should be extracted"
        );
        assert!(
            keys.contains(&KeyCode::V),
            "V (tap-hold) should be extracted"
        );
    }

    #[test]
    fn test_scan_code_conversion_for_extracted_keys() {
        use crate::platform::windows::keycode::keycode_to_scancode;
        use std::path::PathBuf;

        let krx_path = PathBuf::from("examples/user_layout.krx");

        if !krx_path.exists() {
            println!("Skipping test: examples/user_layout.krx not found");
            return;
        }

        let bytes = std::fs::read(&krx_path).expect("Should read user_layout.krx");
        let archived = deserialize(&bytes).expect("Should deserialize user_layout.krx");
        let owned: ConfigRoot = archived
            .deserialize(&mut rkyv::Infallible)
            .expect("Should convert to owned");

        let keys = extract_source_keys(&owned);

        let mut unconvertible = Vec::new();
        let mut scan_codes = HashSet::new();

        for key in &keys {
            match keycode_to_scancode(*key) {
                Some(scan) => {
                    scan_codes.insert(scan);
                }
                None => {
                    unconvertible.push(*key);
                }
            }
        }

        assert!(
            unconvertible.is_empty(),
            "These keys from user_layout.krx cannot be converted to scan codes: {:?}",
            unconvertible
        );

        println!(
            "Successfully converted {} keys to {} scan codes",
            keys.len(),
            scan_codes.len()
        );

        // Verify scan code count matches (should be 1:1 unless extended keys)
        assert_eq!(
            scan_codes.len(),
            keys.len(),
            "Scan code count should match key count"
        );
    }
}
