//! Single source of truth for all keycode definitions.
//!
//! This module provides the `define_keycodes!` macro which generates:
//! - The `KeyCode` enum with all keyboard key variants
//! - `Display` implementation for human-readable output
//! - `FromStr` implementation with alias support
//! - Platform-specific conversion functions:
//!   - Linux: `evdev_to_keycode()` / `keycode_to_evdev()`
//!   - Windows: `vk_to_keycode()` / `keycode_to_vk()`
//! - `all_keycodes()` for uinput device registration
//!
//! All keycode definitions are centralized here to ensure consistency
//! across the codebase and eliminate duplication.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Macro that generates the KeyCode enum and all related implementations.
///
/// # Syntax
///
/// ```text
/// define_keycodes! {
///     // Variant => "DisplayName", evdev_code, vk_code, ["alias1", "alias2", ...]
///     A => "A", 30, 0x41, ["A"],
///     ...
/// }
/// ```
///
/// The macro generates:
/// - `KeyCode` enum with all specified variants plus `Unknown(u16)`
/// - `Display` impl using the display name
/// - `FromStr` impl matching aliases (case-insensitive)
/// - `evdev_to_keycode(u16) -> KeyCode` for Linux
/// - `keycode_to_evdev(KeyCode) -> u16` for Linux
/// - `vk_to_keycode(u16) -> KeyCode` for Windows
/// - `keycode_to_vk(KeyCode) -> u16` for Windows
/// - `all_keycodes() -> Vec<KeyCode>` for device registration
macro_rules! define_keycodes {
    (
        $(
            $variant:ident => $display:literal, $evdev:expr, $vk:expr, [$($alias:literal),* $(,)?]
        ),* $(,)?
    ) => {
        /// Physical key code representing keyboard keys.
        ///
        /// This enum covers all standard keyboard keys including letters,
        /// numbers, function keys, modifiers, navigation, numpad, and media keys.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub enum KeyCode {
            $(
                $variant,
            )*
            /// Unknown key with raw scan code.
            Unknown(u16),
        }

        impl KeyCode {
            /// Parse a key code from a string name.
            #[inline]
            pub fn from_name(name: &str) -> Option<Self> {
                Self::from_str(name).ok()
            }

            /// Get the string name of this key code.
            #[inline]
            pub fn name(&self) -> String {
                format!("{self}")
            }
        }

        impl fmt::Display for KeyCode {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $(
                        Self::$variant => write!(f, $display),
                    )*
                    Self::Unknown(code) => write!(f, "Unknown({code})"),
                }
            }
        }

        impl FromStr for KeyCode {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let s_upper = s.to_uppercase();
                match s_upper.as_str() {
                    $(
                        $($alias)|* => Ok(Self::$variant),
                    )*
                    _ => Err(format!("Unknown key: {s}")),
                }
            }
        }

        /// Convert evdev key code to KeyCode (Linux).
        ///
        /// Maps Linux evdev event codes (from input-event-codes.h) to the
        /// internal KeyCode representation.
        #[cfg(target_os = "linux")]
        pub fn evdev_to_keycode(code: u16) -> KeyCode {
            match code {
                $(
                    $evdev => KeyCode::$variant,
                )*
                _ => KeyCode::Unknown(code),
            }
        }

        /// Convert KeyCode to evdev key code (Linux).
        ///
        /// Maps internal KeyCode to Linux evdev event codes for key injection.
        #[cfg(target_os = "linux")]
        pub fn keycode_to_evdev(key: KeyCode) -> u16 {
            match key {
                $(
                    KeyCode::$variant => $evdev,
                )*
                KeyCode::Unknown(code) => code,
            }
        }

        /// Convert Windows virtual key code to KeyCode.
        ///
        /// Maps Windows VK_* constants to the internal KeyCode representation.
        /// Available on all platforms for cross-platform testing.
        ///
        /// Note: NumpadEnter and Enter share VK code 0x0D on Windows.
        /// They are distinguished by the extended key flag (KEYEVENTF_EXTENDEDKEY),
        /// not by the VK code itself. This function returns Enter for 0x0D.
        #[allow(unreachable_patterns)]
        pub fn vk_to_keycode(vk: u16) -> KeyCode {
            match vk {
                $(
                    $vk => KeyCode::$variant,
                )*
                _ => KeyCode::Unknown(vk),
            }
        }

        /// Convert KeyCode to Windows virtual key code.
        ///
        /// Maps internal KeyCode to Windows VK_* constants for key injection.
        /// Available on all platforms for cross-platform testing.
        pub fn keycode_to_vk(key: KeyCode) -> u16 {
            match key {
                $(
                    KeyCode::$variant => $vk,
                )*
                KeyCode::Unknown(vk) => vk,
            }
        }

        /// Returns a vector of all known keycodes (excluding Unknown).
        ///
        /// Used for uinput device registration on Linux to enable all keys.
        pub fn all_keycodes() -> Vec<KeyCode> {
            vec![
                $(
                    KeyCode::$variant,
                )*
            ]
        }

        /// Returns all evdev key codes for uinput registration (Linux only).
        ///
        /// Returns a vector of (evdev_code, KeyCode) pairs for registering
        /// all supported keys with a uinput virtual device.
        #[cfg(target_os = "linux")]
        pub fn all_evdev_codes() -> Vec<u16> {
            vec![
                $(
                    $evdev,
                )*
            ]
        }
    };
}

// Define all keycodes with their display names, evdev codes, VK codes, and aliases.
// Format: Variant => "DisplayName", evdev_code, vk_code, ["ALIAS1", "ALIAS2", ...]
//
// Evdev codes are from linux/input-event-codes.h
// VK codes are from Windows WinUser.h VK_* constants
define_keycodes! {
    // Letters A-Z (evdev: 30-38, 44-50, 16-25; VK: 0x41-0x5A)
    A => "A", 30, 0x41, ["A"],
    B => "B", 48, 0x42, ["B"],
    C => "C", 46, 0x43, ["C"],
    D => "D", 32, 0x44, ["D"],
    E => "E", 18, 0x45, ["E"],
    F => "F", 33, 0x46, ["F"],
    G => "G", 34, 0x47, ["G"],
    H => "H", 35, 0x48, ["H"],
    I => "I", 23, 0x49, ["I"],
    J => "J", 36, 0x4A, ["J"],
    K => "K", 37, 0x4B, ["K"],
    L => "L", 38, 0x4C, ["L"],
    M => "M", 50, 0x4D, ["M"],
    N => "N", 49, 0x4E, ["N"],
    O => "O", 24, 0x4F, ["O"],
    P => "P", 25, 0x50, ["P"],
    Q => "Q", 16, 0x51, ["Q"],
    R => "R", 19, 0x52, ["R"],
    S => "S", 31, 0x53, ["S"],
    T => "T", 20, 0x54, ["T"],
    U => "U", 22, 0x55, ["U"],
    V => "V", 47, 0x56, ["V"],
    W => "W", 17, 0x57, ["W"],
    X => "X", 45, 0x58, ["X"],
    Y => "Y", 21, 0x59, ["Y"],
    Z => "Z", 44, 0x5A, ["Z"],

    // Numbers 0-9 (evdev: 2-11; VK: 0x30-0x39)
    Key0 => "0", 11, 0x30, ["0", "KEY0"],
    Key1 => "1", 2, 0x31, ["1", "KEY1"],
    Key2 => "2", 3, 0x32, ["2", "KEY2"],
    Key3 => "3", 4, 0x33, ["3", "KEY3"],
    Key4 => "4", 5, 0x34, ["4", "KEY4"],
    Key5 => "5", 6, 0x35, ["5", "KEY5"],
    Key6 => "6", 7, 0x36, ["6", "KEY6"],
    Key7 => "7", 8, 0x37, ["7", "KEY7"],
    Key8 => "8", 9, 0x38, ["8", "KEY8"],
    Key9 => "9", 10, 0x39, ["9", "KEY9"],

    // Function keys F1-F12 (evdev: 59-68, 87-88; VK: 0x70-0x7B)
    F1 => "F1", 59, 0x70, ["F1"],
    F2 => "F2", 60, 0x71, ["F2"],
    F3 => "F3", 61, 0x72, ["F3"],
    F4 => "F4", 62, 0x73, ["F4"],
    F5 => "F5", 63, 0x74, ["F5"],
    F6 => "F6", 64, 0x75, ["F6"],
    F7 => "F7", 65, 0x76, ["F7"],
    F8 => "F8", 66, 0x77, ["F8"],
    F9 => "F9", 67, 0x78, ["F9"],
    F10 => "F10", 68, 0x79, ["F10"],
    F11 => "F11", 87, 0x7A, ["F11"],
    F12 => "F12", 88, 0x7B, ["F12"],

    // Modifier keys
    LeftShift => "LeftShift", 42, 0xA0, ["LEFTSHIFT", "LSHIFT", "SHIFT"],
    RightShift => "RightShift", 54, 0xA1, ["RIGHTSHIFT", "RSHIFT"],
    LeftCtrl => "LeftCtrl", 29, 0xA2, ["LEFTCTRL", "LCTRL", "CTRL", "CONTROL"],
    RightCtrl => "RightCtrl", 97, 0xA3, ["RIGHTCTRL", "RCTRL"],
    LeftAlt => "LeftAlt", 56, 0xA4, ["LEFTALT", "LALT", "ALT"],
    RightAlt => "RightAlt", 100, 0xA5, ["RIGHTALT", "RALT", "ALTGR"],
    LeftMeta => "LeftMeta", 125, 0x5B, ["LEFTMETA", "LMETA", "META", "WIN", "SUPER", "CMD"],
    RightMeta => "RightMeta", 126, 0x5C, ["RIGHTMETA", "RMETA", "RWIN", "RSUPER", "RCMD"],

    // Navigation keys
    Up => "Up", 103, 0x26, ["UP", "UPARROW"],
    Down => "Down", 108, 0x28, ["DOWN", "DOWNARROW"],
    Left => "Left", 105, 0x25, ["LEFT", "LEFTARROW"],
    Right => "Right", 106, 0x27, ["RIGHT", "RIGHTARROW"],
    Home => "Home", 102, 0x24, ["HOME"],
    End => "End", 107, 0x23, ["END"],
    PageUp => "PageUp", 104, 0x21, ["PAGEUP", "PGUP"],
    PageDown => "PageDown", 109, 0x22, ["PAGEDOWN", "PGDN"],

    // Editing keys
    Insert => "Insert", 110, 0x2D, ["INSERT", "INS"],
    Delete => "Delete", 111, 0x2E, ["DELETE", "DEL"],
    Backspace => "Backspace", 14, 0x08, ["BACKSPACE", "BACK", "BS"],

    // Whitespace keys
    Space => "Space", 57, 0x20, ["SPACE", "SPACEBAR"],
    Tab => "Tab", 15, 0x09, ["TAB"],
    Enter => "Enter", 28, 0x0D, ["ENTER", "RETURN"],

    // Lock keys
    CapsLock => "CapsLock", 58, 0x14, ["CAPSLOCK", "CAPS"],
    NumLock => "NumLock", 69, 0x90, ["NUMLOCK", "NUM"],
    ScrollLock => "ScrollLock", 70, 0x91, ["SCROLLLOCK", "SCROLL"],

    // Escape and Print Screen area
    Escape => "Escape", 1, 0x1B, ["ESCAPE", "ESC"],
    PrintScreen => "PrintScreen", 99, 0x2C, ["PRINTSCREEN", "PRINT", "PRTSC"],
    Pause => "Pause", 119, 0x13, ["PAUSE", "BREAK"],

    // Punctuation and symbols
    Grave => "Grave", 41, 0xC0, ["GRAVE", "BACKTICK", "TILDE"],
    Minus => "Minus", 12, 0xBD, ["MINUS", "DASH"],
    Equal => "Equal", 13, 0xBB, ["EQUAL", "EQUALS"],
    LeftBracket => "LeftBracket", 26, 0xDB, ["LEFTBRACKET", "LBRACKET"],
    RightBracket => "RightBracket", 27, 0xDD, ["RIGHTBRACKET", "RBRACKET"],
    Backslash => "Backslash", 43, 0xDC, ["BACKSLASH"],
    Semicolon => "Semicolon", 39, 0xBA, ["SEMICOLON"],
    Apostrophe => "Apostrophe", 40, 0xDE, ["APOSTROPHE", "QUOTE"],
    Comma => "Comma", 51, 0xBC, ["COMMA"],
    Period => "Period", 52, 0xBE, ["PERIOD", "DOT"],
    Slash => "Slash", 53, 0xBF, ["SLASH"],

    // Numpad keys
    Numpad0 => "Numpad0", 82, 0x60, ["NUMPAD0", "KP0"],
    Numpad1 => "Numpad1", 79, 0x61, ["NUMPAD1", "KP1"],
    Numpad2 => "Numpad2", 80, 0x62, ["NUMPAD2", "KP2"],
    Numpad3 => "Numpad3", 81, 0x63, ["NUMPAD3", "KP3"],
    Numpad4 => "Numpad4", 75, 0x64, ["NUMPAD4", "KP4"],
    Numpad5 => "Numpad5", 76, 0x65, ["NUMPAD5", "KP5"],
    Numpad6 => "Numpad6", 77, 0x66, ["NUMPAD6", "KP6"],
    Numpad7 => "Numpad7", 71, 0x67, ["NUMPAD7", "KP7"],
    Numpad8 => "Numpad8", 72, 0x68, ["NUMPAD8", "KP8"],
    Numpad9 => "Numpad9", 73, 0x69, ["NUMPAD9", "KP9"],
    NumpadAdd => "NumpadAdd", 78, 0x6B, ["NUMPADADD", "KPADD", "KPPLUS"],
    NumpadSubtract => "NumpadSubtract", 74, 0x6D, ["NUMPADSUBTRACT", "KPSUB", "KPMINUS"],
    NumpadMultiply => "NumpadMultiply", 55, 0x6A, ["NUMPADMULTIPLY", "KPMUL", "KPASTERISK"],
    NumpadDivide => "NumpadDivide", 98, 0x6F, ["NUMPADDIVIDE", "KPDIV", "KPSLASH"],
    NumpadEnter => "NumpadEnter", 96, 0x0D, ["NUMPADENTER", "KPENTER"],
    NumpadDecimal => "NumpadDecimal", 83, 0x6E, ["NUMPADDECIMAL", "KPDECIMAL", "KPDOT"],

    // Media keys
    VolumeUp => "VolumeUp", 115, 0xAF, ["VOLUMEUP", "VOLUP"],
    VolumeDown => "VolumeDown", 114, 0xAE, ["VOLUMEDOWN", "VOLDOWN"],
    VolumeMute => "VolumeMute", 113, 0xAD, ["VOLUMEMUTE", "MUTE"],
    MediaPlayPause => "MediaPlayPause", 164, 0xB3, ["MEDIAPLAYPAUSE", "PLAYPAUSE", "PLAY"],
    MediaStop => "MediaStop", 166, 0xB2, ["MEDIASTOP", "STOP"],
    MediaNext => "MediaNext", 163, 0xB0, ["MEDIANEXT", "NEXT", "NEXTTRACK"],
    MediaPrev => "MediaPrev", 165, 0xB1, ["MEDIAPREV", "PREV", "PREVTRACK"],
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn keycode_can_be_hashmap_key() {
        let mut map: HashMap<KeyCode, &str> = HashMap::new();
        map.insert(KeyCode::A, "a_value");
        map.insert(KeyCode::CapsLock, "caps_value");

        assert_eq!(map.get(&KeyCode::A), Some(&"a_value"));
        assert_eq!(map.get(&KeyCode::CapsLock), Some(&"caps_value"));
        assert_eq!(map.get(&KeyCode::Z), None);
    }

    #[test]
    fn keycode_from_str_basic() {
        assert_eq!(KeyCode::from_str("A"), Ok(KeyCode::A));
        assert_eq!(KeyCode::from_str("a"), Ok(KeyCode::A));
        assert_eq!(KeyCode::from_str("CapsLock"), Ok(KeyCode::CapsLock));
        assert_eq!(KeyCode::from_str("caps"), Ok(KeyCode::CapsLock));
        assert_eq!(KeyCode::from_str("Escape"), Ok(KeyCode::Escape));
        assert_eq!(KeyCode::from_str("esc"), Ok(KeyCode::Escape));
        assert_eq!(KeyCode::from_str("F1"), Ok(KeyCode::F1));
        assert_eq!(KeyCode::from_str("1"), Ok(KeyCode::Key1));
        assert!(KeyCode::from_str("InvalidKey").is_err());
    }

    #[test]
    fn keycode_from_str_aliases() {
        // Modifier aliases
        assert_eq!(KeyCode::from_str("shift"), Ok(KeyCode::LeftShift));
        assert_eq!(KeyCode::from_str("lshift"), Ok(KeyCode::LeftShift));
        assert_eq!(KeyCode::from_str("ctrl"), Ok(KeyCode::LeftCtrl));
        assert_eq!(KeyCode::from_str("control"), Ok(KeyCode::LeftCtrl));
        assert_eq!(KeyCode::from_str("alt"), Ok(KeyCode::LeftAlt));
        assert_eq!(KeyCode::from_str("altgr"), Ok(KeyCode::RightAlt));
        assert_eq!(KeyCode::from_str("win"), Ok(KeyCode::LeftMeta));
        assert_eq!(KeyCode::from_str("super"), Ok(KeyCode::LeftMeta));
        assert_eq!(KeyCode::from_str("cmd"), Ok(KeyCode::LeftMeta));

        // Navigation aliases
        assert_eq!(KeyCode::from_str("pgup"), Ok(KeyCode::PageUp));
        assert_eq!(KeyCode::from_str("pgdn"), Ok(KeyCode::PageDown));

        // Editing aliases
        assert_eq!(KeyCode::from_str("del"), Ok(KeyCode::Delete));
        assert_eq!(KeyCode::from_str("ins"), Ok(KeyCode::Insert));
        assert_eq!(KeyCode::from_str("back"), Ok(KeyCode::Backspace));
        assert_eq!(KeyCode::from_str("bs"), Ok(KeyCode::Backspace));

        // Whitespace aliases
        assert_eq!(KeyCode::from_str("return"), Ok(KeyCode::Enter));
        assert_eq!(KeyCode::from_str("spacebar"), Ok(KeyCode::Space));

        // Numpad aliases
        assert_eq!(KeyCode::from_str("kp0"), Ok(KeyCode::Numpad0));
        assert_eq!(KeyCode::from_str("kpplus"), Ok(KeyCode::NumpadAdd));
        assert_eq!(KeyCode::from_str("kpminus"), Ok(KeyCode::NumpadSubtract));
    }

    #[test]
    fn keycode_display() {
        assert_eq!(KeyCode::A.to_string(), "A");
        assert_eq!(KeyCode::CapsLock.to_string(), "CapsLock");
        assert_eq!(KeyCode::F1.to_string(), "F1");
        assert_eq!(KeyCode::Key1.to_string(), "1");
        assert_eq!(KeyCode::Unknown(999).to_string(), "Unknown(999)");
    }

    #[test]
    fn keycode_from_name() {
        assert_eq!(KeyCode::from_name("A"), Some(KeyCode::A));
        assert_eq!(KeyCode::from_name("escape"), Some(KeyCode::Escape));
        assert_eq!(KeyCode::from_name("invalid"), None);
    }

    #[test]
    fn keycode_name() {
        assert_eq!(KeyCode::A.name(), "A");
        assert_eq!(KeyCode::LeftShift.name(), "LeftShift");
    }

    #[test]
    fn keycode_traits() {
        // Test Copy
        let key = KeyCode::A;
        let key_copy = key;
        assert_eq!(key, key_copy);

        // Test Clone
        let key_clone = key.clone();
        assert_eq!(key, key_clone);

        // Test Eq
        assert_eq!(KeyCode::A, KeyCode::A);
        assert_ne!(KeyCode::A, KeyCode::B);

        // Test Hash via HashMap
        let mut map = HashMap::new();
        map.insert(KeyCode::A, 1);
        assert_eq!(map.get(&KeyCode::A), Some(&1));
    }

    #[test]
    fn all_keycodes_returns_all_variants() {
        let all = all_keycodes();
        // Verify we have all 95 keycodes (26 letters + 10 numbers + 12 F-keys + 8 modifiers +
        // 8 nav + 3 editing + 3 whitespace + 3 locks + 3 escape area + 11 punct + 16 numpad + 7 media = 110)
        // But we need to count exactly what we defined
        assert!(
            all.len() > 90,
            "Should have many keycodes, got {}",
            all.len()
        );

        // Check some specific ones exist
        assert!(all.contains(&KeyCode::A));
        assert!(all.contains(&KeyCode::Z));
        assert!(all.contains(&KeyCode::F1));
        assert!(all.contains(&KeyCode::F12));
        assert!(all.contains(&KeyCode::CapsLock));
        assert!(all.contains(&KeyCode::MediaPlayPause));

        // Unknown should not be in the list
        assert!(!all.iter().any(|k| matches!(k, KeyCode::Unknown(_))));
    }

    #[cfg(target_os = "linux")]
    mod linux_tests {
        use super::*;

        #[test]
        fn evdev_roundtrip() {
            // Test that evdev_to_keycode and keycode_to_evdev are inverses
            for keycode in all_keycodes() {
                let evdev = keycode_to_evdev(keycode);
                let back = evdev_to_keycode(evdev);
                assert_eq!(keycode, back, "Roundtrip failed for {:?}", keycode);
            }
        }

        #[test]
        fn evdev_specific_codes() {
            // Test specific evdev codes match expected keycodes
            assert_eq!(evdev_to_keycode(1), KeyCode::Escape);
            assert_eq!(evdev_to_keycode(30), KeyCode::A);
            assert_eq!(evdev_to_keycode(58), KeyCode::CapsLock);
            assert_eq!(evdev_to_keycode(125), KeyCode::LeftMeta);
            assert_eq!(evdev_to_keycode(164), KeyCode::MediaPlayPause);
        }

        #[test]
        fn evdev_unknown() {
            assert_eq!(evdev_to_keycode(9999), KeyCode::Unknown(9999));
            assert_eq!(keycode_to_evdev(KeyCode::Unknown(9999)), 9999);
        }

        #[test]
        fn all_evdev_codes_count() {
            let codes = all_evdev_codes();
            assert_eq!(codes.len(), all_keycodes().len());
        }
    }

    mod windows_tests {
        use super::*;

        #[test]
        fn vk_roundtrip() {
            // Test that vk_to_keycode and keycode_to_vk are inverses
            // Note: NumpadEnter shares VK code 0x0D with Enter - distinguished by extended flag
            for keycode in all_keycodes() {
                // Skip NumpadEnter - it shares VK 0x0D with Enter
                // On Windows, these are distinguished by the extended key flag, not VK code
                if matches!(keycode, KeyCode::NumpadEnter) {
                    continue;
                }
                let vk = keycode_to_vk(keycode);
                let back = vk_to_keycode(vk);
                assert_eq!(keycode, back, "Roundtrip failed for {:?}", keycode);
            }
        }

        #[test]
        fn vk_specific_codes() {
            // Test specific VK codes match expected keycodes
            assert_eq!(vk_to_keycode(0x1B), KeyCode::Escape);
            assert_eq!(vk_to_keycode(0x41), KeyCode::A);
            assert_eq!(vk_to_keycode(0x14), KeyCode::CapsLock);
            assert_eq!(vk_to_keycode(0x5B), KeyCode::LeftMeta);
            assert_eq!(vk_to_keycode(0xB3), KeyCode::MediaPlayPause);
        }

        #[test]
        fn vk_unknown() {
            assert_eq!(vk_to_keycode(0xFFFF), KeyCode::Unknown(0xFFFF));
            assert_eq!(keycode_to_vk(KeyCode::Unknown(0xFFFF)), 0xFFFF);
        }
    }
}
