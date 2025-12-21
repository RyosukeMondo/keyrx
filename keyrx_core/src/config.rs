//! Configuration data structures for KeyRx
//!
//! This module defines all configuration types using rkyv for zero-copy deserialization.
//! All types use #[repr(C)] for stable binary layout.

extern crate alloc;

use alloc::vec::Vec;
use core::fmt;
use rkyv::{Archive, Deserialize, Serialize};

/// Version information for binary compatibility checking
///
/// Uses semantic versioning with major.minor.patch format.
/// All fields are u8 to keep the struct compact.
#[derive(Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
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
#[derive(Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
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
#[derive(Archive, Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
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
#[derive(Archive, Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
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
#[derive(Archive, Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
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
#[derive(Archive, Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
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
#[derive(Archive, Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct DeviceIdentifier {
    /// Pattern string for matching device names/IDs
    pub pattern: alloc::string::String,
}

/// Device-specific configuration
///
/// Contains all key mappings for a specific device or device pattern.
/// Multiple devices can share the same configuration by using pattern matching.
#[derive(Archive, Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct DeviceConfig {
    /// Device identifier pattern
    pub identifier: DeviceIdentifier,
    /// List of key mappings for this device
    pub mappings: Vec<KeyMapping>,
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
}
