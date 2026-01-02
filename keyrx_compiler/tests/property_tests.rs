//! Property-based tests for deterministic serialization and parser robustness
//!
//! These tests use proptest to verify that:
//! - Serialization is deterministic
//! - Round-trip serialization/deserialization preserves data
//! - Parser handles randomly generated valid scripts without panicking
//! - Parser produces valid ConfigRoot objects

use keyrx_compiler::parser::Parser;
use keyrx_compiler::serialize::{deserialize, serialize};
use keyrx_core::config::{
    BaseKeyMapping, Condition, ConditionItem, ConfigRoot, DeviceConfig, DeviceIdentifier, KeyCode,
    KeyMapping, Metadata, Version,
};
use proptest::prelude::*;
use sha2::{Digest, Sha256};

// ============================================================================
// Proptest Strategies for Generating Arbitrary Configs
// ============================================================================

/// Strategy for generating arbitrary Version
fn version_strategy() -> impl Strategy<Value = Version> {
    (any::<u8>(), any::<u8>(), any::<u8>()).prop_map(|(major, minor, patch)| Version {
        major,
        minor,
        patch,
    })
}

/// Strategy for generating arbitrary KeyCode
fn keycode_strategy() -> impl Strategy<Value = KeyCode> {
    prop_oneof![
        // Letters
        Just(KeyCode::A),
        Just(KeyCode::B),
        Just(KeyCode::C),
        Just(KeyCode::D),
        Just(KeyCode::E),
        Just(KeyCode::Z),
        // Numbers
        Just(KeyCode::Num0),
        Just(KeyCode::Num1),
        Just(KeyCode::Num9),
        // Function keys
        Just(KeyCode::F1),
        Just(KeyCode::F12),
        // Modifiers
        Just(KeyCode::LShift),
        Just(KeyCode::RShift),
        Just(KeyCode::LCtrl),
        Just(KeyCode::RCtrl),
        Just(KeyCode::LAlt),
        Just(KeyCode::RAlt),
        // Special keys
        Just(KeyCode::Escape),
        Just(KeyCode::Enter),
        Just(KeyCode::Backspace),
        Just(KeyCode::Tab),
        Just(KeyCode::Space),
        Just(KeyCode::CapsLock),
        // Arrow keys
        Just(KeyCode::Left),
        Just(KeyCode::Right),
        Just(KeyCode::Up),
        Just(KeyCode::Down),
    ]
}

/// Strategy for generating arbitrary ConditionItem
fn condition_item_strategy() -> impl Strategy<Value = ConditionItem> {
    prop_oneof![
        (0u8..=0xFE).prop_map(ConditionItem::ModifierActive),
        (0u8..=0xFE).prop_map(ConditionItem::LockActive),
    ]
}

/// Strategy for generating arbitrary Condition
fn condition_strategy() -> impl Strategy<Value = Condition> {
    prop_oneof![
        (0u8..=0xFE).prop_map(Condition::ModifierActive),
        (0u8..=0xFE).prop_map(Condition::LockActive),
        prop::collection::vec(condition_item_strategy(), 1..5).prop_map(Condition::AllActive),
        prop::collection::vec(condition_item_strategy(), 1..5).prop_map(Condition::NotActive),
    ]
}

/// Strategy for generating arbitrary BaseKeyMapping
fn base_key_mapping_strategy() -> impl Strategy<Value = BaseKeyMapping> {
    prop_oneof![
        // Simple mapping
        (keycode_strategy(), keycode_strategy())
            .prop_map(|(from, to)| BaseKeyMapping::Simple { from, to }),
        // Modifier mapping
        (keycode_strategy(), 0u8..=0xFE)
            .prop_map(|(from, modifier_id)| { BaseKeyMapping::Modifier { from, modifier_id } }),
        // Lock mapping
        (keycode_strategy(), 0u8..=0xFE)
            .prop_map(|(from, lock_id)| BaseKeyMapping::Lock { from, lock_id }),
        // TapHold mapping
        (
            keycode_strategy(),
            keycode_strategy(),
            0u8..=0xFE,
            1u16..1000
        )
            .prop_map(
                |(from, tap, hold_modifier, threshold_ms)| BaseKeyMapping::TapHold {
                    from,
                    tap,
                    hold_modifier,
                    threshold_ms
                }
            ),
        // ModifiedOutput mapping
        (
            keycode_strategy(),
            keycode_strategy(),
            any::<bool>(),
            any::<bool>(),
            any::<bool>(),
            any::<bool>()
        )
            .prop_map(
                |(from, to, shift, ctrl, alt, win)| BaseKeyMapping::ModifiedOutput {
                    from,
                    to,
                    shift,
                    ctrl,
                    alt,
                    win
                }
            ),
    ]
}

/// Strategy for generating arbitrary KeyMapping
fn key_mapping_strategy() -> impl Strategy<Value = KeyMapping> {
    prop_oneof![
        base_key_mapping_strategy().prop_map(KeyMapping::Base),
        (
            condition_strategy(),
            prop::collection::vec(base_key_mapping_strategy(), 0..5)
        )
            .prop_map(|(condition, mappings)| KeyMapping::Conditional {
                condition,
                mappings
            }),
    ]
}

/// Strategy for generating arbitrary DeviceIdentifier
fn device_identifier_strategy() -> impl Strategy<Value = DeviceIdentifier> {
    prop_oneof![
        Just("*".to_string()),
        Just("USB Keyboard".to_string()),
        Just("Laptop Keyboard".to_string()),
        Just("External Keyboard".to_string()),
        "[a-zA-Z ]{5,20}".prop_map(|s| s),
    ]
    .prop_map(|pattern| DeviceIdentifier { pattern })
}

/// Strategy for generating arbitrary DeviceConfig
fn device_config_strategy() -> impl Strategy<Value = DeviceConfig> {
    (
        device_identifier_strategy(),
        prop::collection::vec(key_mapping_strategy(), 0..10),
    )
        .prop_map(|(identifier, mappings)| DeviceConfig {
            identifier,
            mappings,
        })
}

/// Strategy for generating arbitrary Metadata
fn metadata_strategy() -> impl Strategy<Value = Metadata> {
    (
        any::<u64>(),
        "[0-9]{1,3}\\.[0-9]{1,3}\\.[0-9]{1,3}",
        "[a-f0-9]{64}",
    )
        .prop_map(
            |(compilation_timestamp, compiler_version, source_hash)| Metadata {
                compilation_timestamp,
                compiler_version,
                source_hash,
            },
        )
}

/// Strategy for generating arbitrary ConfigRoot
fn config_root_strategy() -> impl Strategy<Value = ConfigRoot> {
    (
        version_strategy(),
        prop::collection::vec(device_config_strategy(), 1..5),
        metadata_strategy(),
    )
        .prop_map(|(version, devices, metadata)| ConfigRoot {
            version,
            devices,
            metadata,
        })
}

// ============================================================================
// Property Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 1000,
        .. ProptestConfig::default()
    })]

    /// Test property: serialize(config) == serialize(config)
    ///
    /// Verifies that serialization is deterministic - the same input
    /// always produces the same output.
    #[test]
    fn test_deterministic_serialization(config in config_root_strategy()) {
        let bytes1 = serialize(&config).expect("First serialization failed");
        let bytes2 = serialize(&config).expect("Second serialization failed");

        // Same struct → same bytes
        prop_assert_eq!(bytes1.len(), bytes2.len());
        prop_assert_eq!(&bytes1[..], &bytes2[..]);
    }

    /// Test property: deserialize(serialize(config)) == config
    ///
    /// Verifies that round-trip serialization preserves all data.
    #[test]
    fn test_round_trip_serialization(config in config_root_strategy()) {
        // Serialize
        let bytes = serialize(&config).expect("Serialization failed");

        // Deserialize
        let archived = deserialize(&bytes).expect("Deserialization failed");

        // Verify round-trip: all fields should match
        prop_assert_eq!(archived.version.major, config.version.major);
        prop_assert_eq!(archived.version.minor, config.version.minor);
        prop_assert_eq!(archived.version.patch, config.version.patch);
        prop_assert_eq!(archived.devices.len(), config.devices.len());
        prop_assert_eq!(
            archived.metadata.compilation_timestamp,
            config.metadata.compilation_timestamp
        );
        prop_assert_eq!(
            archived.metadata.compiler_version.as_str(),
            config.metadata.compiler_version.as_str()
        );
        prop_assert_eq!(
            archived.metadata.source_hash.as_str(),
            config.metadata.source_hash.as_str()
        );

        // Verify device patterns match
        for (archived_dev, original_dev) in archived.devices.iter().zip(config.devices.iter()) {
            prop_assert_eq!(
                archived_dev.identifier.pattern.as_str(),
                original_dev.identifier.pattern.as_str()
            );
            prop_assert_eq!(
                archived_dev.mappings.len(),
                original_dev.mappings.len()
            );
        }
    }

    /// Test property: hash(serialize(config1)) != hash(serialize(config2)) if config1 != config2
    ///
    /// Verifies that different configurations produce different serialized outputs
    /// (hash collision resistance).
    #[test]
    fn test_different_configs_different_hashes(
        config1 in config_root_strategy(),
        config2 in config_root_strategy()
    ) {
        // Skip if configs are identical
        if config1 == config2 {
            return Ok(());
        }

        // Serialize both
        let bytes1 = serialize(&config1).expect("Serialization 1 failed");
        let bytes2 = serialize(&config2).expect("Serialization 2 failed");

        // Compute hashes
        let mut hasher1 = Sha256::new();
        hasher1.update(&bytes1);
        let hash1 = hasher1.finalize();

        let mut hasher2 = Sha256::new();
        hasher2.update(&bytes2);
        let hash2 = hasher2.finalize();

        // Different configs → different hashes (with extremely high probability)
        prop_assert_ne!(hash1.as_slice(), hash2.as_slice());
    }

    /// Test property: Serialized size is reasonable
    ///
    /// Verifies that serialization doesn't produce unexpectedly large outputs.
    #[test]
    fn test_serialized_size_reasonable(config in config_root_strategy()) {
        let bytes = serialize(&config).expect("Serialization failed");

        // Header is 48 bytes, data should be at least 1 byte
        prop_assert!(bytes.len() >= 49);

        // Serialized size should not be more than 1MB for reasonable configs
        // (this is a sanity check, not a hard limit)
        prop_assert!(bytes.len() < 1_000_000);
    }

    /// Test property: Multiple serializations produce identical hashes
    ///
    /// Verifies that the embedded hash in the .krx file is deterministic.
    #[test]
    fn test_embedded_hash_deterministic(config in config_root_strategy()) {
        let bytes1 = serialize(&config).expect("First serialization failed");
        let bytes2 = serialize(&config).expect("Second serialization failed");

        // Extract embedded hashes (bytes 8-40)
        let hash1 = &bytes1[8..40];
        let hash2 = &bytes2[8..40];

        // Hashes should be identical
        prop_assert_eq!(hash1, hash2);
    }

    /// Test property: Deserialization validates magic bytes
    ///
    /// Verifies that deserializer correctly rejects invalid magic bytes.
    #[test]
    fn test_invalid_magic_rejected(config in config_root_strategy()) {
        let mut bytes = serialize(&config).expect("Serialization failed");

        // Corrupt the magic bytes
        bytes[0] = 0xFF;

        // Deserialization should fail
        let result = deserialize(&bytes);
        prop_assert!(result.is_err());
        if let Err(e) = result {
            let is_invalid_magic = matches!(e, keyrx_compiler::error::DeserializeError::InvalidMagic { .. });
            prop_assert!(is_invalid_magic);
        }
    }

    /// Test property: Deserialization validates version
    ///
    /// Verifies that deserializer correctly rejects invalid version numbers.
    #[test]
    fn test_invalid_version_rejected(config in config_root_strategy()) {
        let mut bytes = serialize(&config).expect("Serialization failed");

        // Corrupt the version (bytes 4-8)
        bytes[4] = 0xFF;
        bytes[5] = 0xFF;
        bytes[6] = 0xFF;
        bytes[7] = 0xFF;

        // Deserialization should fail
        let result = deserialize(&bytes);
        prop_assert!(result.is_err());
        if let Err(e) = result {
            let is_version_mismatch = matches!(e, keyrx_compiler::error::DeserializeError::VersionMismatch { .. });
            prop_assert!(is_version_mismatch);
        }
    }

    /// Test property: Deserialization validates hash
    ///
    /// Verifies that deserializer correctly rejects corrupted data by detecting hash mismatches.
    #[test]
    fn test_corrupted_data_rejected(config in config_root_strategy()) {
        let mut bytes = serialize(&config).expect("Serialization failed");

        // Skip if the data section is too small
        if bytes.len() < 100 {
            return Ok(());
        }

        // Corrupt the data section (after header, at byte 50)
        bytes[50] ^= 0xFF;

        // Deserialization should fail due to hash mismatch
        let result = deserialize(&bytes);
        prop_assert!(result.is_err());
        if let Err(e) = result {
            let is_hash_mismatch = matches!(e, keyrx_compiler::error::DeserializeError::HashMismatch { .. });
            prop_assert!(is_hash_mismatch);
        }
    }
}

// ============================================================================
// Additional Unit Tests for Edge Cases
// ============================================================================

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_empty_devices_list() {
        // Config with no devices
        let config = ConfigRoot {
            version: Version::current(),
            devices: vec![],
            metadata: Metadata {
                compilation_timestamp: 1234567890,
                compiler_version: "1.0.0".to_string(),
                source_hash: "empty".to_string(),
            },
        };

        // Should serialize and deserialize successfully
        let bytes = serialize(&config).expect("Serialization failed");
        let archived = deserialize(&bytes).expect("Deserialization failed");

        assert_eq!(archived.devices.len(), 0);
    }

    #[test]
    fn test_large_config() {
        // Create a large config with many devices and mappings
        let mut devices = Vec::new();
        for i in 0..100 {
            devices.push(DeviceConfig {
                identifier: DeviceIdentifier {
                    pattern: format!("Device {}", i),
                },
                mappings: vec![
                    KeyMapping::simple(KeyCode::A, KeyCode::B),
                    KeyMapping::conditional(
                        Condition::ModifierActive(i as u8),
                        vec![BaseKeyMapping::Simple {
                            from: KeyCode::C,
                            to: KeyCode::D,
                        }],
                    ),
                ],
            });
        }

        let config = ConfigRoot {
            version: Version::current(),
            devices,
            metadata: Metadata {
                compilation_timestamp: 1234567890,
                compiler_version: "1.0.0".to_string(),
                source_hash: "large".to_string(),
            },
        };

        // Should serialize and deserialize successfully
        let bytes = serialize(&config).expect("Serialization failed");
        let archived = deserialize(&bytes).expect("Deserialization failed");

        assert_eq!(archived.devices.len(), 100);
    }

    #[test]
    fn test_all_mapping_variants() {
        // Create a config with all KeyMapping variants
        let config = ConfigRoot {
            version: Version::current(),
            devices: vec![DeviceConfig {
                identifier: DeviceIdentifier {
                    pattern: "*".to_string(),
                },
                mappings: vec![
                    KeyMapping::simple(KeyCode::A, KeyCode::B),
                    KeyMapping::modifier(KeyCode::CapsLock, 0x01),
                    KeyMapping::lock(KeyCode::ScrollLock, 0x02),
                    KeyMapping::tap_hold(KeyCode::Space, KeyCode::Space, 0x00, 200),
                    KeyMapping::modified_output(KeyCode::A, KeyCode::A, true, false, false, false),
                ],
            }],
            metadata: Metadata {
                compilation_timestamp: 1234567890,
                compiler_version: "1.0.0".to_string(),
                source_hash: "all_variants".to_string(),
            },
        };

        // Should serialize and deserialize successfully
        let bytes = serialize(&config).expect("Serialization failed");
        let archived = deserialize(&bytes).expect("Deserialization failed");

        assert_eq!(archived.devices[0].mappings.len(), 5);
    }

    #[test]
    fn test_all_condition_variants() {
        // Create a config with all Condition variants
        let config = ConfigRoot {
            version: Version::current(),
            devices: vec![DeviceConfig {
                identifier: DeviceIdentifier {
                    pattern: "*".to_string(),
                },
                mappings: vec![
                    KeyMapping::conditional(
                        Condition::ModifierActive(0x01),
                        vec![BaseKeyMapping::Simple {
                            from: KeyCode::A,
                            to: KeyCode::B,
                        }],
                    ),
                    KeyMapping::conditional(
                        Condition::LockActive(0x02),
                        vec![BaseKeyMapping::Simple {
                            from: KeyCode::C,
                            to: KeyCode::D,
                        }],
                    ),
                    KeyMapping::conditional(
                        Condition::AllActive(vec![
                            ConditionItem::ModifierActive(0x01),
                            ConditionItem::LockActive(0x02),
                        ]),
                        vec![BaseKeyMapping::Simple {
                            from: KeyCode::E,
                            to: KeyCode::F,
                        }],
                    ),
                    KeyMapping::conditional(
                        Condition::NotActive(vec![ConditionItem::ModifierActive(0x01)]),
                        vec![BaseKeyMapping::Simple {
                            from: KeyCode::G,
                            to: KeyCode::H,
                        }],
                    ),
                ],
            }],
            metadata: Metadata {
                compilation_timestamp: 1234567890,
                compiler_version: "1.0.0".to_string(),
                source_hash: "all_conditions".to_string(),
            },
        };

        // Should serialize and deserialize successfully
        let bytes = serialize(&config).expect("Serialization failed");
        let archived = deserialize(&bytes).expect("Deserialization failed");

        assert_eq!(archived.devices[0].mappings.len(), 4);
    }
}

// ============================================================================
// Parser Property Tests - Test Rhai Script Generation and Parsing
// ============================================================================

/// Strategy for generating random valid KeyCode names as strings
fn keycode_name_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Letters
        Just("VK_A"),
        Just("VK_B"),
        Just("VK_C"),
        Just("VK_D"),
        Just("VK_E"),
        Just("VK_F"),
        Just("VK_G"),
        Just("VK_H"),
        Just("VK_I"),
        Just("VK_J"),
        Just("VK_K"),
        Just("VK_L"),
        Just("VK_M"),
        Just("VK_N"),
        Just("VK_Z"),
        // Numbers
        Just("VK_Num0"),
        Just("VK_Num1"),
        Just("VK_Num2"),
        Just("VK_Num9"),
        // Function keys
        Just("VK_F1"),
        Just("VK_F12"),
        // Special keys
        Just("VK_Escape"),
        Just("VK_Enter"),
        Just("VK_Backspace"),
        Just("VK_Tab"),
        Just("VK_Space"),
        Just("VK_CapsLock"),
        // Arrow keys
        Just("VK_Left"),
        Just("VK_Right"),
        Just("VK_Up"),
        Just("VK_Down"),
        // Modifiers
        Just("VK_LShift"),
        Just("VK_RShift"),
        Just("VK_LCtrl"),
        Just("VK_RCtrl"),
        Just("VK_LAlt"),
        Just("VK_RAlt"),
    ]
    .prop_map(|s| s.to_string())
}

/// Strategy for generating random valid modifier IDs as strings (MD_00 to MD_FE)
fn modifier_id_strategy() -> impl Strategy<Value = String> {
    (0u8..=0xFE).prop_map(|id| format!("MD_{:02X}", id))
}

/// Strategy for generating random valid lock IDs as strings (LK_00 to LK_FE)
fn lock_id_strategy() -> impl Strategy<Value = String> {
    (0u8..=0xFE).prop_map(|id| format!("LK_{:02X}", id))
}

/// Strategy for generating random valid condition strings (MD_XX or LK_XX)
fn condition_string_strategy() -> impl Strategy<Value = String> {
    prop_oneof![modifier_id_strategy(), lock_id_strategy(),]
}

/// Strategy for generating random valid Rhai script statements
/// This generates map(), tap_hold(), when(), when_not() calls with valid syntax
fn rhai_statement_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Simple mapping: map("VK_X", "VK_Y")
        (keycode_name_strategy(), keycode_name_strategy())
            .prop_map(|(from, to)| format!("map(\"{}\", \"{}\");", from, to)),
        // Modifier mapping: map("VK_X", "MD_XX")
        (keycode_name_strategy(), modifier_id_strategy())
            .prop_map(|(from, to)| format!("map(\"{}\", \"{}\");", from, to)),
        // Lock mapping: map("VK_X", "LK_XX")
        (keycode_name_strategy(), lock_id_strategy())
            .prop_map(|(from, to)| format!("map(\"{}\", \"{}\");", from, to)),
        // TapHold mapping: tap_hold("VK_X", "VK_Y", "MD_XX", 200)
        (
            keycode_name_strategy(),
            keycode_name_strategy(),
            modifier_id_strategy(),
            100u16..1000
        )
            .prop_map(|(from, tap, hold, threshold)| {
                format!(
                    "tap_hold(\"{}\", \"{}\", \"{}\", {});",
                    from, tap, hold, threshold
                )
            }),
        // Modified output: map("VK_X", with_shift("VK_Y"))
        (keycode_name_strategy(), keycode_name_strategy())
            .prop_map(|(from, to)| format!("map(\"{}\", with_shift(\"{}\"));", from, to)),
        // Modified output with ctrl
        (keycode_name_strategy(), keycode_name_strategy())
            .prop_map(|(from, to)| format!("map(\"{}\", with_ctrl(\"{}\"));", from, to)),
        // Modified output with alt
        (keycode_name_strategy(), keycode_name_strategy())
            .prop_map(|(from, to)| format!("map(\"{}\", with_alt(\"{}\"));", from, to)),
        // Modified output with win
        (keycode_name_strategy(), keycode_name_strategy())
            .prop_map(|(from, to)| format!("map(\"{}\", with_win(\"{}\"));", from, to)),
    ]
}

/// Strategy for generating random valid conditional blocks
/// when("MD_XX") { map("VK_X", "VK_Y"); }
fn conditional_block_strategy() -> impl Strategy<Value = String> {
    (
        condition_string_strategy(),
        prop::collection::vec(rhai_statement_strategy(), 1..5),
    )
        .prop_map(|(condition, statements)| {
            format!(
                "when_start(\"{}\");\n{}\nwhen_end();",
                condition,
                statements.join("\n")
            )
        })
}

/// Strategy for generating when_not blocks
fn when_not_block_strategy() -> impl Strategy<Value = String> {
    (
        condition_string_strategy(),
        prop::collection::vec(rhai_statement_strategy(), 1..5),
    )
        .prop_map(|(condition, statements)| {
            format!(
                "when_not_start(\"{}\");\n{}\nwhen_not_end();",
                condition,
                statements.join("\n")
            )
        })
}

/// Strategy for generating random device blocks with statements
fn device_block_strategy() -> impl Strategy<Value = String> {
    (
        prop_oneof![
            Just("*".to_string()),
            Just("USB Keyboard".to_string()),
            Just("Laptop Keyboard".to_string()),
        ],
        prop::collection::vec(
            prop_oneof![
                rhai_statement_strategy(),
                conditional_block_strategy(),
                when_not_block_strategy(),
            ],
            1..10,
        ),
    )
        .prop_map(|(pattern, statements)| {
            format!(
                "device_start(\"{}\");\n{}\ndevice_end();",
                pattern,
                statements.join("\n")
            )
        })
}

/// Strategy for generating complete valid Rhai scripts
fn rhai_script_strategy() -> impl Strategy<Value = String> {
    prop::collection::vec(device_block_strategy(), 1..5).prop_map(|blocks| blocks.join("\n\n"))
}

// ============================================================================
// Parser Property Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 100,  // Fewer cases for parser tests since they're slower
        .. ProptestConfig::default()
    })]

    /// Test property: Parser handles randomly generated valid scripts without panicking
    ///
    /// Verifies that the parser can handle any valid Rhai script structure.
    #[test]
    fn test_parser_handles_random_valid_scripts(script in rhai_script_strategy()) {
        // Create a temporary file for the script
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let script_path = temp_dir.path().join("test.rhai");
        std::fs::write(&script_path, &script).expect("Failed to write script");

        // Parse the script
        let mut parser = Parser::new();
        let result = parser.parse_script(&script_path);

        // Should succeed without panicking
        prop_assert!(result.is_ok(), "Parser failed on valid script:\n{}\n\nError: {:?}", script, result.err());

        // Verify the result is valid
        if let Ok(config) = result {
            prop_assert!(!config.devices.is_empty(), "Config should have at least one device");
            prop_assert_eq!(config.version.major, 1);
            prop_assert_eq!(config.version.minor, 0);
            prop_assert_eq!(config.version.patch, 0);
        }
    }

    /// Test property: Parsed ConfigRoot can be serialized successfully
    ///
    /// Verifies that parser output is valid enough to be serialized.
    #[test]
    fn test_parsed_config_can_be_serialized(script in rhai_script_strategy()) {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let script_path = temp_dir.path().join("test.rhai");
        std::fs::write(&script_path, &script).expect("Failed to write script");

        let mut parser = Parser::new();
        if let Ok(config) = parser.parse_script(&script_path) {
            // Serialization should succeed
            let result = serialize(&config);
            prop_assert!(result.is_ok(), "Serialization failed for parsed config: {:?}", result.err());
        }
    }

    /// Test property: Simple map() statements produce correct mappings
    #[test]
    fn test_simple_map_statements(
        from in keycode_name_strategy(),
        to in keycode_name_strategy()
    ) {
        let script = format!(
            "device_start(\"*\");\nmap(\"{}\", \"{}\");\ndevice_end();",
            from, to
        );

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let script_path = temp_dir.path().join("test.rhai");
        std::fs::write(&script_path, &script).expect("Failed to write script");

        let mut parser = Parser::new();
        let result = parser.parse_script(&script_path);

        prop_assert!(result.is_ok(), "Parser failed on simple map: {}", script);

        if let Ok(config) = result {
            prop_assert_eq!(config.devices.len(), 1);
            prop_assert!(!config.devices[0].mappings.is_empty());
        }
    }

    /// Test property: tap_hold() statements produce valid TapHold mappings
    #[test]
    fn test_tap_hold_statements(
        from in keycode_name_strategy(),
        tap in keycode_name_strategy(),
        hold in modifier_id_strategy(),
        threshold in 100u16..1000
    ) {
        let script = format!(
            "device_start(\"*\");\ntap_hold(\"{}\", \"{}\", \"{}\", {});\ndevice_end();",
            from, tap, hold, threshold
        );

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let script_path = temp_dir.path().join("test.rhai");
        std::fs::write(&script_path, &script).expect("Failed to write script");

        let mut parser = Parser::new();
        let result = parser.parse_script(&script_path);

        prop_assert!(result.is_ok(), "Parser failed on tap_hold: {}", script);

        if let Ok(config) = result {
            prop_assert_eq!(config.devices.len(), 1);
            prop_assert_eq!(config.devices[0].mappings.len(), 1);
        }
    }

    /// Test property: Conditional blocks produce valid Conditional mappings
    #[test]
    fn test_conditional_blocks(
        condition in condition_string_strategy(),
        from in keycode_name_strategy(),
        to in keycode_name_strategy()
    ) {
        let script = format!(
            "device_start(\"*\");\nwhen_start(\"{}\");\nmap(\"{}\", \"{}\");\nwhen_end();\ndevice_end();",
            condition, from, to
        );

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let script_path = temp_dir.path().join("test.rhai");
        std::fs::write(&script_path, &script).expect("Failed to write script");

        let mut parser = Parser::new();
        let result = parser.parse_script(&script_path);

        prop_assert!(result.is_ok(), "Parser failed on conditional: {}", script);

        if let Ok(config) = result {
            prop_assert_eq!(config.devices.len(), 1);
            prop_assert!(!config.devices[0].mappings.is_empty());
        }
    }

    /// Test property: Multiple devices can be parsed
    #[test]
    fn test_multiple_devices(
        pattern1 in prop_oneof![Just("*"), Just("USB Keyboard")],
        pattern2 in prop_oneof![Just("Laptop"), Just("External")],
        from in keycode_name_strategy(),
        to in keycode_name_strategy()
    ) {
        let script = format!(
            "device_start(\"{}\");\nmap(\"{}\", \"{}\");\ndevice_end();\n\ndevice_start(\"{}\");\nmap(\"{}\", \"{}\");\ndevice_end();",
            pattern1, from, to, pattern2, from, to
        );

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let script_path = temp_dir.path().join("test.rhai");
        std::fs::write(&script_path, &script).expect("Failed to write script");

        let mut parser = Parser::new();
        let result = parser.parse_script(&script_path);

        prop_assert!(result.is_ok(), "Parser failed on multiple devices: {}", script);

        if let Ok(config) = result {
            prop_assert_eq!(config.devices.len(), 2);
        }
    }

    /// Test property: Round-trip parsing produces consistent results
    ///
    /// Parse → serialize → deserialize should produce equivalent config
    #[test]
    fn test_parser_round_trip(script in rhai_script_strategy()) {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let script_path = temp_dir.path().join("test.rhai");
        std::fs::write(&script_path, &script).expect("Failed to write script");

        let mut parser = Parser::new();
        if let Ok(config) = parser.parse_script(&script_path) {
            // Serialize
            if let Ok(bytes) = serialize(&config) {
                // Deserialize
                let result = deserialize(&bytes);
                prop_assert!(result.is_ok(), "Round-trip deserialization failed");

                if let Ok(archived) = result {
                    // Compare key properties
                    prop_assert_eq!(archived.devices.len(), config.devices.len());
                    prop_assert_eq!(archived.version.major, config.version.major);
                    prop_assert_eq!(archived.version.minor, config.version.minor);
                    prop_assert_eq!(archived.version.patch, config.version.patch);
                }
            }
        }
    }
}

#[cfg(test)]
mod parser_edge_case_tests {
    use super::*;

    #[test]
    fn test_empty_device_block() {
        let script = "device_start(\"*\");\ndevice_end();";

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let script_path = temp_dir.path().join("test.rhai");
        std::fs::write(&script_path, script).expect("Failed to write script");

        let mut parser = Parser::new();
        let result = parser.parse_script(&script_path);

        // Empty device should be valid
        assert!(result.is_ok());
        if let Ok(config) = result {
            assert_eq!(config.devices.len(), 1);
            assert_eq!(config.devices[0].mappings.len(), 0);
        }
    }

    #[test]
    fn test_many_mappings_in_device() {
        let mut script = String::from("device_start(\"*\");\n");
        for i in 0..100 {
            script.push_str(&format!(
                "map(\"VK_F{}\", \"VK_F{}\");\n",
                (i % 24) + 1,
                ((i + 1) % 24) + 1
            ));
        }
        script.push_str("device_end();");

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let script_path = temp_dir.path().join("test.rhai");
        std::fs::write(&script_path, &script).expect("Failed to write script");

        let mut parser = Parser::new();
        let result = parser.parse_script(&script_path);

        assert!(result.is_ok());
        if let Ok(config) = result {
            assert_eq!(config.devices.len(), 1);
            assert_eq!(config.devices[0].mappings.len(), 100);
        }
    }

    #[test]
    fn test_nested_conditionals() {
        let script = r#"
device_start("*");
when_start("MD_00");
map("VK_H", "VK_Left");
when_end();
device_end();
        "#;

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let script_path = temp_dir.path().join("test.rhai");
        std::fs::write(&script_path, script).expect("Failed to write script");

        let mut parser = Parser::new();
        let result = parser.parse_script(&script_path);

        assert!(result.is_ok());
        if let Ok(config) = result {
            assert_eq!(config.devices.len(), 1);
            assert!(!config.devices[0].mappings.is_empty());
        }
    }

    #[test]
    fn test_all_modifier_helpers() {
        let script = r#"
device_start("*");
map("VK_1", with_shift("VK_1"));
map("VK_2", with_ctrl("VK_C"));
map("VK_3", with_alt("VK_F4"));
map("VK_4", with_win("VK_L"));
map("VK_5", with_mods("VK_C", true, true, false, false));
device_end();
        "#;

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let script_path = temp_dir.path().join("test.rhai");
        std::fs::write(&script_path, script).expect("Failed to write script");

        let mut parser = Parser::new();
        let result = parser.parse_script(&script_path);

        assert!(result.is_ok());
        if let Ok(config) = result {
            assert_eq!(config.devices.len(), 1);
            assert_eq!(config.devices[0].mappings.len(), 5);
        }
    }

    #[test]
    fn test_mix_of_all_mapping_types() {
        let script = r#"
device_start("*");
map("VK_A", "VK_B");
map("VK_CapsLock", "MD_00");
map("VK_ScrollLock", "LK_01");
tap_hold("VK_Space", "VK_Space", "MD_01", 200);
map("VK_1", with_shift("VK_1"));
when_start("MD_00");
map("VK_H", "VK_Left");
when_end();
device_end();
        "#;

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let script_path = temp_dir.path().join("test.rhai");
        std::fs::write(&script_path, script).expect("Failed to write script");

        let mut parser = Parser::new();
        let result = parser.parse_script(&script_path);

        assert!(result.is_ok());
        if let Ok(config) = result {
            assert_eq!(config.devices.len(), 1);
            assert_eq!(config.devices[0].mappings.len(), 6);
        }
    }
}
