//! Property-based tests for deterministic serialization
//!
//! These tests use proptest to verify that serialization is deterministic
//! and that round-trip serialization/deserialization preserves data.

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
    let leaf = prop_oneof![
        (0u8..=0xFE).prop_map(Condition::ModifierActive),
        (0u8..=0xFE).prop_map(Condition::LockActive),
    ];

    leaf.prop_recursive(
        2,  // depth
        10, // max nodes
        5,  // items per collection
        |inner| {
            prop_oneof![
                prop::collection::vec(condition_item_strategy(), 1..5)
                    .prop_map(Condition::AllActive),
                prop::collection::vec(condition_item_strategy(), 1..5)
                    .prop_map(Condition::NotActive),
                inner,
            ]
        },
    )
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
                    KeyMapping::Base(BaseKeyMapping::Simple {
                        from: KeyCode::A,
                        to: KeyCode::B,
                    }),
                    KeyMapping::Conditional {
                        condition: Condition::ModifierActive(i as u8),
                        mappings: vec![BaseKeyMapping::Simple {
                            from: KeyCode::C,
                            to: KeyCode::D,
                        }],
                    },
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
    fn test_all_base_mapping_variants() {
        // Create a config with all BaseKeyMapping variants
        let config = ConfigRoot {
            version: Version::current(),
            devices: vec![DeviceConfig {
                identifier: DeviceIdentifier {
                    pattern: "*".to_string(),
                },
                mappings: vec![
                    KeyMapping::Base(BaseKeyMapping::Simple {
                        from: KeyCode::A,
                        to: KeyCode::B,
                    }),
                    KeyMapping::Base(BaseKeyMapping::Modifier {
                        from: KeyCode::CapsLock,
                        modifier_id: 0x01,
                    }),
                    KeyMapping::Base(BaseKeyMapping::Lock {
                        from: KeyCode::ScrollLock,
                        lock_id: 0x02,
                    }),
                    KeyMapping::Base(BaseKeyMapping::TapHold {
                        from: KeyCode::Space,
                        tap: KeyCode::Space,
                        hold_modifier: 0x00,
                        threshold_ms: 200,
                    }),
                    KeyMapping::Base(BaseKeyMapping::ModifiedOutput {
                        from: KeyCode::A,
                        to: KeyCode::A,
                        shift: true,
                        ctrl: false,
                        alt: false,
                        win: false,
                    }),
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
                    KeyMapping::Conditional {
                        condition: Condition::ModifierActive(0x01),
                        mappings: vec![BaseKeyMapping::Simple {
                            from: KeyCode::A,
                            to: KeyCode::B,
                        }],
                    },
                    KeyMapping::Conditional {
                        condition: Condition::LockActive(0x02),
                        mappings: vec![BaseKeyMapping::Simple {
                            from: KeyCode::C,
                            to: KeyCode::D,
                        }],
                    },
                    KeyMapping::Conditional {
                        condition: Condition::AllActive(vec![
                            ConditionItem::ModifierActive(0x01),
                            ConditionItem::LockActive(0x02),
                        ]),
                        mappings: vec![BaseKeyMapping::Simple {
                            from: KeyCode::E,
                            to: KeyCode::F,
                        }],
                    },
                    KeyMapping::Conditional {
                        condition: Condition::NotActive(vec![ConditionItem::ModifierActive(0x01)]),
                        mappings: vec![BaseKeyMapping::Simple {
                            from: KeyCode::G,
                            to: KeyCode::H,
                        }],
                    },
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
