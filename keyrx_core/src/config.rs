//! Configuration data structures for KeyRx
//!
//! This module defines all configuration types using rkyv for zero-copy deserialization.
//! All types use #[repr(C)] for stable binary layout.

extern crate alloc;

use alloc::vec::Vec;
use core::fmt;
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize, Serialize};

/// Version information for binary compatibility checking
///
/// Uses semantic versioning with major.minor.patch format.
/// All fields are u8 to keep the struct compact.
#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Debug,
)]
#[repr(C)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl Version {
    /// Returns the current version (1.0.0)
    pub const fn current() -> Self {
        Self {
            major: 1,
            minor: 0,
            patch: 0,
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Platform-agnostic keyboard key codes
///
/// All variants have explicit discriminants to prevent reordering issues.
/// Keys are organized by category for maintainability.
#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Debug,
)]
#[repr(u16)]
pub enum KeyCode {
    // Letters: A-Z (0x00-0x19)
    A = 0x00,
    B = 0x01,
    C = 0x02,
    D = 0x03,
    E = 0x04,
    F = 0x05,
    G = 0x06,
    H = 0x07,
    I = 0x08,
    J = 0x09,
    K = 0x0A,
    L = 0x0B,
    M = 0x0C,
    N = 0x0D,
    O = 0x0E,
    P = 0x0F,
    Q = 0x10,
    R = 0x11,
    S = 0x12,
    T = 0x13,
    U = 0x14,
    V = 0x15,
    W = 0x16,
    X = 0x17,
    Y = 0x18,
    Z = 0x19,

    // Numbers: 0-9 (0x20-0x29)
    Num0 = 0x20,
    Num1 = 0x21,
    Num2 = 0x22,
    Num3 = 0x23,
    Num4 = 0x24,
    Num5 = 0x25,
    Num6 = 0x26,
    Num7 = 0x27,
    Num8 = 0x28,
    Num9 = 0x29,

    // Function keys: F1-F12 (0x30-0x3B)
    F1 = 0x30,
    F2 = 0x31,
    F3 = 0x32,
    F4 = 0x33,
    F5 = 0x34,
    F6 = 0x35,
    F7 = 0x36,
    F8 = 0x37,
    F9 = 0x38,
    F10 = 0x39,
    F11 = 0x3A,
    F12 = 0x3B,

    // Physical modifier keys (0x100-0x107)
    LShift = 0x100,
    RShift = 0x101,
    LCtrl = 0x102,
    RCtrl = 0x103,
    LAlt = 0x104,
    RAlt = 0x105,
    LMeta = 0x106,
    RMeta = 0x107,

    // Special keys (0x200+)
    Escape = 0x200,
    Enter = 0x201,
    Backspace = 0x202,
    Tab = 0x203,
    Space = 0x204,
    CapsLock = 0x205,
    NumLock = 0x206,
    ScrollLock = 0x207,
    PrintScreen = 0x208,
    Pause = 0x209,
    Insert = 0x20A,
    Delete = 0x20B,
    Home = 0x20C,
    End = 0x20D,
    PageUp = 0x20E,
    PageDown = 0x20F,

    // Arrow keys (0x210-0x213)
    Left = 0x210,
    Right = 0x211,
    Up = 0x212,
    Down = 0x213,

    // Additional special keys
    LeftBracket = 0x220,
    RightBracket = 0x221,
    Backslash = 0x222,
    Semicolon = 0x223,
    Quote = 0x224,
    Comma = 0x225,
    Period = 0x226,
    Slash = 0x227,
    Grave = 0x228,
    Minus = 0x229,
    Equal = 0x22A,

    // Numpad keys (0x230+)
    Numpad0 = 0x230,
    Numpad1 = 0x231,
    Numpad2 = 0x232,
    Numpad3 = 0x233,
    Numpad4 = 0x234,
    Numpad5 = 0x235,
    Numpad6 = 0x236,
    Numpad7 = 0x237,
    Numpad8 = 0x238,
    Numpad9 = 0x239,
    NumpadDivide = 0x23A,
    NumpadMultiply = 0x23B,
    NumpadSubtract = 0x23C,
    NumpadAdd = 0x23D,
    NumpadEnter = 0x23E,
    NumpadDecimal = 0x23F,

    // Extended function keys (0x240+)
    F13 = 0x240,
    F14 = 0x241,
    F15 = 0x242,
    F16 = 0x243,
    F17 = 0x244,
    F18 = 0x245,
    F19 = 0x246,
    F20 = 0x247,
    F21 = 0x248,
    F22 = 0x249,
    F23 = 0x24A,
    F24 = 0x24B,

    // Media keys (0x250+)
    Mute = 0x250,
    VolumeDown = 0x251,
    VolumeUp = 0x252,
    MediaPlayPause = 0x253,
    MediaStop = 0x254,
    MediaPrevious = 0x255,
    MediaNext = 0x256,

    // System keys (0x260+)
    Power = 0x260,
    Sleep = 0x261,
    Wake = 0x262,

    // Browser keys (0x270+)
    BrowserBack = 0x270,
    BrowserForward = 0x271,
    BrowserRefresh = 0x272,
    BrowserStop = 0x273,
    BrowserSearch = 0x274,
    BrowserFavorites = 0x275,
    BrowserHome = 0x276,

    // Application keys (0x280+)
    AppMail = 0x280,
    AppCalculator = 0x281,
    AppMyComputer = 0x282,

    // Additional keys
    Menu = 0x290,
    Help = 0x291,
    Select = 0x292,
    Execute = 0x293,
    Undo = 0x294,
    Redo = 0x295,
    Cut = 0x296,
    Copy = 0x297,
    Paste = 0x298,
    Find = 0x299,
}

/// Basic condition check for a single modifier or lock
///
/// Used in composite conditions.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, Serialize, Deserialize, Clone, PartialEq, Eq, Debug,
)]
pub enum ConditionItem {
    /// Custom modifier is active (MD_XX)
    ModifierActive(u8),
    /// Custom lock is active (LK_XX)
    LockActive(u8),
}

/// Conditional mapping support for when/when_not blocks
///
/// Supports single conditions, AND combinations, and negation.
/// To avoid recursive Box issues with rkyv, NotActive contains a Vec
/// of conditions which must ALL be false (implemented as NOT(AND(...))).
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, Serialize, Deserialize, Clone, PartialEq, Eq, Debug,
)]
pub enum Condition {
    /// Single custom modifier active (MD_XX)
    ModifierActive(u8),
    /// Single custom lock active (LK_XX)
    LockActive(u8),
    /// All conditions must be true (AND logic) - for when() with multiple conditions
    AllActive(Vec<ConditionItem>),
    /// All conditions must be false (when_not with AND logic) - negated AllActive
    /// For single condition negation, use vec with one item
    NotActive(Vec<ConditionItem>),
}

/// Base key mapping without conditional nesting
///
/// Used as the leaf mappings within conditional blocks.
/// This prevents infinite recursion in rkyv serialization.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, Serialize, Deserialize, Clone, PartialEq, Eq, Debug,
)]
#[repr(C)]
pub enum BaseKeyMapping {
    /// Simple 1:1 key remapping
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
    /// Output with physical modifiers
    ModifiedOutput {
        from: KeyCode,
        to: KeyCode,
        shift: bool,
        ctrl: bool,
        alt: bool,
        win: bool,
    },
}

/// Key mapping configuration
///
/// Defines all possible mapping types.
/// Conditional mappings contain base mappings to limit recursion depth to 1 level.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, Serialize, Deserialize, Clone, PartialEq, Eq, Debug,
)]
#[repr(C)]
pub enum KeyMapping {
    /// Base mapping (non-conditional)
    Base(BaseKeyMapping),
    /// Conditional mappings (when/when_not)
    /// Contains only base mappings to avoid infinite recursion
    Conditional {
        condition: Condition,
        mappings: Vec<BaseKeyMapping>,
    },
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

/// Metadata about the compiled configuration
///
/// Contains information about when and how the configuration was compiled.
/// This is useful for debugging and verification purposes.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, Serialize, Deserialize, Clone, PartialEq, Eq, Debug,
)]
#[repr(C)]
pub struct Metadata {
    /// Unix timestamp (seconds since epoch) when the config was compiled
    pub compilation_timestamp: u64,
    /// Version of the compiler that created this file
    pub compiler_version: alloc::string::String,
    /// SHA256 hash of the source Rhai script(s)
    pub source_hash: alloc::string::String,
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
    use alloc::string::ToString;

    #[test]
    fn test_version_current() {
        let version = Version::current();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 0);
        assert_eq!(version.patch, 0);
    }

    #[test]
    fn test_version_display() {
        let version = Version::current();
        assert_eq!(version.to_string(), "1.0.0");
    }

    #[test]
    fn test_keycode_has_all_expected_variants() {
        // Test letters
        assert_eq!(KeyCode::A as u16, 0x00);
        assert_eq!(KeyCode::Z as u16, 0x19);

        // Test numbers
        assert_eq!(KeyCode::Num0 as u16, 0x20);
        assert_eq!(KeyCode::Num9 as u16, 0x29);

        // Test function keys
        assert_eq!(KeyCode::F1 as u16, 0x30);
        assert_eq!(KeyCode::F12 as u16, 0x3B);

        // Test modifiers
        assert_eq!(KeyCode::LShift as u16, 0x100);
        assert_eq!(KeyCode::RMeta as u16, 0x107);

        // Test special keys
        assert_eq!(KeyCode::Escape as u16, 0x200);
        assert_eq!(KeyCode::Enter as u16, 0x201);

        // Test arrow keys
        assert_eq!(KeyCode::Left as u16, 0x210);
        assert_eq!(KeyCode::Down as u16, 0x213);
    }

    #[test]
    fn test_condition_variants() {
        // Test ModifierActive variant
        let cond1 = Condition::ModifierActive(0x01);
        assert_eq!(cond1, Condition::ModifierActive(0x01));

        // Test LockActive variant
        let cond2 = Condition::LockActive(0x02);
        assert_eq!(cond2, Condition::LockActive(0x02));

        // Test AllActive variant with multiple conditions
        let cond3 = Condition::AllActive(alloc::vec![
            ConditionItem::ModifierActive(0x01),
            ConditionItem::LockActive(0x02),
        ]);
        if let Condition::AllActive(items) = &cond3 {
            assert_eq!(items.len(), 2);
        } else {
            panic!("Expected AllActive variant");
        }

        // Test NotActive variant (negation)
        let cond4 = Condition::NotActive(alloc::vec![ConditionItem::ModifierActive(0x01),]);
        if let Condition::NotActive(items) = &cond4 {
            assert_eq!(items.len(), 1);
        } else {
            panic!("Expected NotActive variant");
        }
    }

    #[test]
    fn test_base_key_mapping_variants() {
        // Test Simple mapping
        let mapping1 = BaseKeyMapping::Simple {
            from: KeyCode::A,
            to: KeyCode::B,
        };
        assert_eq!(
            mapping1,
            BaseKeyMapping::Simple {
                from: KeyCode::A,
                to: KeyCode::B,
            }
        );

        // Test Modifier mapping
        let mapping2 = BaseKeyMapping::Modifier {
            from: KeyCode::CapsLock,
            modifier_id: 0x01,
        };
        if let BaseKeyMapping::Modifier { from, modifier_id } = mapping2 {
            assert_eq!(from, KeyCode::CapsLock);
            assert_eq!(modifier_id, 0x01);
        } else {
            panic!("Expected Modifier variant");
        }

        // Test Lock mapping
        let mapping3 = BaseKeyMapping::Lock {
            from: KeyCode::ScrollLock,
            lock_id: 0x02,
        };
        if let BaseKeyMapping::Lock { from, lock_id } = mapping3 {
            assert_eq!(from, KeyCode::ScrollLock);
            assert_eq!(lock_id, 0x02);
        } else {
            panic!("Expected Lock variant");
        }

        // Test TapHold mapping
        let mapping4 = BaseKeyMapping::TapHold {
            from: KeyCode::Space,
            tap: KeyCode::Space,
            hold_modifier: 0x00,
            threshold_ms: 200,
        };
        if let BaseKeyMapping::TapHold {
            from,
            tap,
            hold_modifier,
            threshold_ms,
        } = mapping4
        {
            assert_eq!(from, KeyCode::Space);
            assert_eq!(tap, KeyCode::Space);
            assert_eq!(hold_modifier, 0x00);
            assert_eq!(threshold_ms, 200);
        } else {
            panic!("Expected TapHold variant");
        }

        // Test ModifiedOutput mapping
        let mapping5 = BaseKeyMapping::ModifiedOutput {
            from: KeyCode::A,
            to: KeyCode::A,
            shift: true,
            ctrl: false,
            alt: false,
            win: false,
        };
        if let BaseKeyMapping::ModifiedOutput {
            shift,
            ctrl,
            alt,
            win,
            ..
        } = mapping5
        {
            assert!(shift);
            assert!(!ctrl);
            assert!(!alt);
            assert!(!win);
        } else {
            panic!("Expected ModifiedOutput variant");
        }
    }

    #[test]
    fn test_key_mapping_variants() {
        // Test Base mapping
        let mapping1 = KeyMapping::Base(BaseKeyMapping::Simple {
            from: KeyCode::A,
            to: KeyCode::B,
        });
        assert!(matches!(mapping1, KeyMapping::Base(_)));

        // Test Conditional mapping
        let mapping2 = KeyMapping::Conditional {
            condition: Condition::ModifierActive(0x01),
            mappings: alloc::vec![BaseKeyMapping::Simple {
                from: KeyCode::H,
                to: KeyCode::Left,
            },],
        };
        if let KeyMapping::Conditional {
            condition,
            mappings,
        } = &mapping2
        {
            assert_eq!(*condition, Condition::ModifierActive(0x01));
            assert_eq!(mappings.len(), 1);
        } else {
            panic!("Expected Conditional variant");
        }
    }

    #[test]
    fn test_device_config_creation() {
        use alloc::string::String;

        let device_config = DeviceConfig {
            identifier: DeviceIdentifier {
                pattern: String::from("*"),
            },
            mappings: alloc::vec![
                KeyMapping::Base(BaseKeyMapping::Simple {
                    from: KeyCode::A,
                    to: KeyCode::B,
                }),
                KeyMapping::Base(BaseKeyMapping::Modifier {
                    from: KeyCode::CapsLock,
                    modifier_id: 0x01,
                }),
            ],
        };

        assert_eq!(device_config.identifier.pattern, "*");
        assert_eq!(device_config.mappings.len(), 2);
    }

    #[test]
    fn test_config_root_serialization_round_trip() {
        use alloc::string::String;

        // Create a complete ConfigRoot
        let config = ConfigRoot {
            version: Version::current(),
            devices: alloc::vec![DeviceConfig {
                identifier: DeviceIdentifier {
                    pattern: String::from("*"),
                },
                mappings: alloc::vec![KeyMapping::Base(BaseKeyMapping::Simple {
                    from: KeyCode::A,
                    to: KeyCode::B,
                }),],
            },],
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
        use alloc::string::String;

        // Create the same config twice
        let create_config = || ConfigRoot {
            version: Version::current(),
            devices: alloc::vec![DeviceConfig {
                identifier: DeviceIdentifier {
                    pattern: String::from("USB Keyboard"),
                },
                mappings: alloc::vec![
                    KeyMapping::Base(BaseKeyMapping::Simple {
                        from: KeyCode::A,
                        to: KeyCode::B,
                    }),
                    KeyMapping::Conditional {
                        condition: Condition::ModifierActive(0x01),
                        mappings: alloc::vec![BaseKeyMapping::Simple {
                            from: KeyCode::H,
                            to: KeyCode::Left,
                        },],
                    },
                ],
            },],
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

        // Verify deterministic output (same struct → same bytes)
        assert_eq!(bytes1.len(), bytes2.len());
        assert_eq!(&bytes1[..], &bytes2[..]);
    }

    #[test]
    fn test_complex_config_serialization() {
        use alloc::string::String;

        // Create a complex configuration with all mapping types
        let config = ConfigRoot {
            version: Version::current(),
            devices: alloc::vec![DeviceConfig {
                identifier: DeviceIdentifier {
                    pattern: String::from("*"),
                },
                mappings: alloc::vec![
                    // Simple mapping
                    KeyMapping::Base(BaseKeyMapping::Simple {
                        from: KeyCode::A,
                        to: KeyCode::B,
                    }),
                    // Modifier mapping
                    KeyMapping::Base(BaseKeyMapping::Modifier {
                        from: KeyCode::CapsLock,
                        modifier_id: 0x01,
                    }),
                    // Lock mapping
                    KeyMapping::Base(BaseKeyMapping::Lock {
                        from: KeyCode::ScrollLock,
                        lock_id: 0x02,
                    }),
                    // TapHold mapping
                    KeyMapping::Base(BaseKeyMapping::TapHold {
                        from: KeyCode::Space,
                        tap: KeyCode::Space,
                        hold_modifier: 0x00,
                        threshold_ms: 200,
                    }),
                    // ModifiedOutput mapping
                    KeyMapping::Base(BaseKeyMapping::ModifiedOutput {
                        from: KeyCode::A,
                        to: KeyCode::A,
                        shift: true,
                        ctrl: false,
                        alt: false,
                        win: false,
                    }),
                    // Conditional mapping with single condition
                    KeyMapping::Conditional {
                        condition: Condition::ModifierActive(0x01),
                        mappings: alloc::vec![BaseKeyMapping::Simple {
                            from: KeyCode::H,
                            to: KeyCode::Left,
                        },],
                    },
                    // Conditional mapping with AllActive
                    KeyMapping::Conditional {
                        condition: Condition::AllActive(alloc::vec![
                            ConditionItem::ModifierActive(0x01),
                            ConditionItem::LockActive(0x02),
                        ]),
                        mappings: alloc::vec![BaseKeyMapping::Simple {
                            from: KeyCode::J,
                            to: KeyCode::Down,
                        },],
                    },
                    // Conditional mapping with NotActive
                    KeyMapping::Conditional {
                        condition: Condition::NotActive(alloc::vec![
                            ConditionItem::ModifierActive(0x01),
                        ]),
                        mappings: alloc::vec![BaseKeyMapping::Simple {
                            from: KeyCode::K,
                            to: KeyCode::Up,
                        },],
                    },
                ],
            },],
            metadata: Metadata {
                compilation_timestamp: 1234567890,
                compiler_version: String::from("1.0.0"),
                source_hash: String::from("complex_test_hash"),
            },
        };

        // Serialize
        let bytes = rkyv::to_bytes::<_, 4096>(&config).expect("Serialization failed");

        // Deserialize
        let archived = unsafe { rkyv::archived_root::<ConfigRoot>(&bytes[..]) };

        // Verify all mappings are preserved
        assert_eq!(archived.devices.len(), 1);
        assert_eq!(archived.devices[0].mappings.len(), 8);
        assert_eq!(archived.metadata.compilation_timestamp, 1234567890);
    }
}
