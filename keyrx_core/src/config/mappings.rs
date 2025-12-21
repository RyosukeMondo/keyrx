use alloc::vec::Vec;
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize, Serialize};

use crate::config::conditions::Condition;
use crate::config::keys::KeyCode;
use crate::config::types::{Metadata, Version};

/// Base key mapping types (non-recursive)
///
/// Contains the 5 fundamental mapping types. This is separated from KeyMapping
/// to avoid rkyv recursion depth issues while maintaining ergonomic usage.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, Serialize, Deserialize, Clone, PartialEq, Eq, Debug,
)]
#[repr(C)]
pub enum BaseKeyMapping {
    /// Simple 1:1 key remapping (A → B)
    Simple { from: KeyCode, to: KeyCode },

    /// Key acts as custom modifier (MD_00-MD_FE)
    Modifier { from: KeyCode, modifier_id: u8 },

    /// Key toggles custom lock (LK_00-LK_FE)
    Lock { from: KeyCode, lock_id: u8 },

    /// Dual tap/hold behavior
    TapHold {
        from: KeyCode,
        tap: KeyCode,
        hold_modifier: u8,
        threshold_ms: u16,
    },

    /// Output with physical modifiers (Shift+2, Ctrl+C, etc.)
    ModifiedOutput {
        from: KeyCode,
        to: KeyCode,
        shift: bool,
        ctrl: bool,
        alt: bool,
        win: bool,
    },
}

/// Key mapping configuration with recursive conditional support
///
/// This enum wraps BaseKeyMapping and adds recursive Conditional mappings.
/// The two-enum design allows unlimited nesting depth while working within
/// rkyv's limitations.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, Serialize, Deserialize, Clone, PartialEq, Eq, Debug,
)]
#[repr(C)]
pub enum KeyMapping {
    /// Base mapping (one of the 5 fundamental types)
    Base(BaseKeyMapping),

    /// Conditional mappings (when/when_not blocks) - supports unlimited nesting
    Conditional {
        condition: Condition,
        mappings: Vec<BaseKeyMapping>,
    },
}

impl KeyMapping {
    /// Create a simple key remapping
    pub fn simple(from: KeyCode, to: KeyCode) -> Self {
        KeyMapping::Base(BaseKeyMapping::Simple { from, to })
    }

    /// Create a modifier key mapping
    pub fn modifier(from: KeyCode, modifier_id: u8) -> Self {
        KeyMapping::Base(BaseKeyMapping::Modifier { from, modifier_id })
    }

    /// Create a lock key mapping
    pub fn lock(from: KeyCode, lock_id: u8) -> Self {
        KeyMapping::Base(BaseKeyMapping::Lock { from, lock_id })
    }

    /// Create a tap-hold mapping
    pub fn tap_hold(from: KeyCode, tap: KeyCode, hold_modifier: u8, threshold_ms: u16) -> Self {
        KeyMapping::Base(BaseKeyMapping::TapHold {
            from,
            tap,
            hold_modifier,
            threshold_ms,
        })
    }

    /// Create a modified output mapping
    pub fn modified_output(
        from: KeyCode,
        to: KeyCode,
        shift: bool,
        ctrl: bool,
        alt: bool,
        win: bool,
    ) -> Self {
        KeyMapping::Base(BaseKeyMapping::ModifiedOutput {
            from,
            to,
            shift,
            ctrl,
            alt,
            win,
        })
    }

    /// Create a conditional mapping
    pub fn conditional(condition: Condition, mappings: Vec<BaseKeyMapping>) -> Self {
        KeyMapping::Conditional {
            condition,
            mappings,
        }
    }
}

/// Device identifier pattern for matching keyboards
///
/// The pattern string is used to match against device names/IDs.
/// Examples:
/// - "*" matches all devices
/// - "USB Keyboard" matches devices with that exact name
/// - Platform-specific patterns may be supported by the daemon
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, Serialize, Deserialize, Clone, PartialEq, Eq, Debug,
)]
#[repr(C)]
pub struct DeviceIdentifier {
    /// Pattern string for matching device names/IDs
    pub pattern: alloc::string::String,
}

/// Device-specific configuration
///
/// Contains all key mappings for a specific device or device pattern.
/// Multiple devices can share the same configuration by using pattern matching.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, Serialize, Deserialize, Clone, PartialEq, Eq, Debug,
)]
#[repr(C)]
pub struct DeviceConfig {
    /// Device identifier pattern
    pub identifier: DeviceIdentifier,
    /// List of key mappings for this device
    pub mappings: Vec<KeyMapping>,
}

/// Root configuration structure
///
/// This is the top-level structure that gets serialized to .krx binary format.
/// Contains all device configurations and metadata.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, Serialize, Deserialize, Clone, PartialEq, Eq, Debug,
)]
#[repr(C)]
pub struct ConfigRoot {
    /// Binary format version
    pub version: Version,
    /// List of device-specific configurations
    pub devices: Vec<DeviceConfig>,
    /// Compilation metadata
    pub metadata: Metadata,
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate alloc;
    use crate::config::conditions::Condition;
    use alloc::string::String;

    #[test]
    fn test_key_mapping_helper_functions() {
        // Test all helper functions produce correct variants
        let simple = KeyMapping::simple(KeyCode::A, KeyCode::B);
        assert!(matches!(
            simple,
            KeyMapping::Base(BaseKeyMapping::Simple { .. })
        ));

        let modifier = KeyMapping::modifier(KeyCode::CapsLock, 0x01);
        assert!(matches!(
            modifier,
            KeyMapping::Base(BaseKeyMapping::Modifier { .. })
        ));

        let lock = KeyMapping::lock(KeyCode::ScrollLock, 0x02);
        assert!(matches!(
            lock,
            KeyMapping::Base(BaseKeyMapping::Lock { .. })
        ));

        let tap_hold = KeyMapping::tap_hold(KeyCode::Space, KeyCode::Space, 0x00, 200);
        assert!(matches!(
            tap_hold,
            KeyMapping::Base(BaseKeyMapping::TapHold { .. })
        ));

        let modified =
            KeyMapping::modified_output(KeyCode::A, KeyCode::A, true, false, false, false);
        assert!(matches!(
            modified,
            KeyMapping::Base(BaseKeyMapping::ModifiedOutput { .. })
        ));

        let conditional = KeyMapping::conditional(
            Condition::ModifierActive(0x01),
            alloc::vec![BaseKeyMapping::Simple {
                from: KeyCode::H,
                to: KeyCode::Left,
            }],
        );
        assert!(matches!(conditional, KeyMapping::Conditional { .. }));
    }

    #[test]
    fn test_device_config_creation() {
        let device_config = DeviceConfig {
            identifier: DeviceIdentifier {
                pattern: String::from("*"),
            },
            mappings: alloc::vec![
                KeyMapping::simple(KeyCode::A, KeyCode::B),
                KeyMapping::modifier(KeyCode::CapsLock, 0x01),
            ],
        };

        assert_eq!(device_config.identifier.pattern, "*");
        assert_eq!(device_config.mappings.len(), 2);
    }

    #[test]
    fn test_config_root_serialization_round_trip() {
        let config = ConfigRoot {
            version: Version::current(),
            devices: alloc::vec![DeviceConfig {
                identifier: DeviceIdentifier {
                    pattern: String::from("*"),
                },
                mappings: alloc::vec![KeyMapping::simple(KeyCode::A, KeyCode::B)],
            }],
            metadata: Metadata {
                compilation_timestamp: 1234567890,
                compiler_version: String::from("1.0.0"),
                source_hash: String::from("abc123"),
            },
        };

        // Serialize
        let bytes = rkyv::to_bytes::<_, 1024>(&config).expect("Serialization failed");

        // Deserialize
        let archived = unsafe { rkyv::archived_root::<ConfigRoot>(&bytes[..]) };

        // Verify round-trip
        assert_eq!(archived.version.major, 1);
        assert_eq!(archived.version.minor, 0);
        assert_eq!(archived.version.patch, 0);
        assert_eq!(archived.devices.len(), 1);
        assert_eq!(archived.metadata.compilation_timestamp, 1234567890);
    }

    #[test]
    fn test_deterministic_serialization() {
        let create_config = || ConfigRoot {
            version: Version::current(),
            devices: alloc::vec![DeviceConfig {
                identifier: DeviceIdentifier {
                    pattern: String::from("USB Keyboard"),
                },
                mappings: alloc::vec![
                    KeyMapping::simple(KeyCode::A, KeyCode::B),
                    KeyMapping::conditional(
                        Condition::ModifierActive(0x01),
                        alloc::vec![BaseKeyMapping::Simple {
                            from: KeyCode::H,
                            to: KeyCode::Left,
                        }],
                    ),
                ],
            }],
            metadata: Metadata {
                compilation_timestamp: 9999999999,
                compiler_version: String::from("1.0.0"),
                source_hash: String::from("test_hash_123"),
            },
        };

        let config1 = create_config();
        let config2 = create_config();

        // Serialize both
        let bytes1 = rkyv::to_bytes::<_, 2048>(&config1).expect("Serialization 1 failed");
        let bytes2 = rkyv::to_bytes::<_, 2048>(&config2).expect("Serialization 2 failed");

        // Verify deterministic output
        assert_eq!(bytes1.len(), bytes2.len());
        assert_eq!(&bytes1[..], &bytes2[..]);
    }
}
