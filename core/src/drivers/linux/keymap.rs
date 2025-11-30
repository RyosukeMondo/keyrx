//! Linux keycode mappings.
//!
//! This module provides evdev-specific keycode conversion functions.
//! These functions use the engine's KeyCode type directly.
//!
//! The `all_evdev_codes()` function is re-exported from the centralized
//! `keycodes` module for SSOT compliance on key registration.

use crate::engine::KeyCode;

// Re-export all_evdev_codes from the centralized keycodes module
// for uinput device registration (SSOT)
pub use crate::drivers::keycodes::all_evdev_codes;

/// Convert evdev key code to KeyCode.
pub fn evdev_to_keycode(code: u16) -> KeyCode {
    match code {
        1 => KeyCode::Escape,
        2 => KeyCode::Key1,
        3 => KeyCode::Key2,
        4 => KeyCode::Key3,
        5 => KeyCode::Key4,
        6 => KeyCode::Key5,
        7 => KeyCode::Key6,
        8 => KeyCode::Key7,
        9 => KeyCode::Key8,
        10 => KeyCode::Key9,
        11 => KeyCode::Key0,
        12 => KeyCode::Minus,
        13 => KeyCode::Equal,
        14 => KeyCode::Backspace,
        15 => KeyCode::Tab,
        16 => KeyCode::Q,
        17 => KeyCode::W,
        18 => KeyCode::E,
        19 => KeyCode::R,
        20 => KeyCode::T,
        21 => KeyCode::Y,
        22 => KeyCode::U,
        23 => KeyCode::I,
        24 => KeyCode::O,
        25 => KeyCode::P,
        26 => KeyCode::LeftBracket,
        27 => KeyCode::RightBracket,
        28 => KeyCode::Enter,
        29 => KeyCode::LeftCtrl,
        30 => KeyCode::A,
        31 => KeyCode::S,
        32 => KeyCode::D,
        33 => KeyCode::F,
        34 => KeyCode::G,
        35 => KeyCode::H,
        36 => KeyCode::J,
        37 => KeyCode::K,
        38 => KeyCode::L,
        39 => KeyCode::Semicolon,
        40 => KeyCode::Apostrophe,
        41 => KeyCode::Grave,
        42 => KeyCode::LeftShift,
        43 => KeyCode::Backslash,
        44 => KeyCode::Z,
        45 => KeyCode::X,
        46 => KeyCode::C,
        47 => KeyCode::V,
        48 => KeyCode::B,
        49 => KeyCode::N,
        50 => KeyCode::M,
        51 => KeyCode::Comma,
        52 => KeyCode::Period,
        53 => KeyCode::Slash,
        54 => KeyCode::RightShift,
        55 => KeyCode::NumpadMultiply,
        56 => KeyCode::LeftAlt,
        57 => KeyCode::Space,
        58 => KeyCode::CapsLock,
        59 => KeyCode::F1,
        60 => KeyCode::F2,
        61 => KeyCode::F3,
        62 => KeyCode::F4,
        63 => KeyCode::F5,
        64 => KeyCode::F6,
        65 => KeyCode::F7,
        66 => KeyCode::F8,
        67 => KeyCode::F9,
        68 => KeyCode::F10,
        69 => KeyCode::NumLock,
        70 => KeyCode::ScrollLock,
        71 => KeyCode::Numpad7,
        72 => KeyCode::Numpad8,
        73 => KeyCode::Numpad9,
        74 => KeyCode::NumpadSubtract,
        75 => KeyCode::Numpad4,
        76 => KeyCode::Numpad5,
        77 => KeyCode::Numpad6,
        78 => KeyCode::NumpadAdd,
        79 => KeyCode::Numpad1,
        80 => KeyCode::Numpad2,
        81 => KeyCode::Numpad3,
        82 => KeyCode::Numpad0,
        83 => KeyCode::NumpadDecimal,
        87 => KeyCode::F11,
        88 => KeyCode::F12,
        96 => KeyCode::NumpadEnter,
        97 => KeyCode::RightCtrl,
        98 => KeyCode::NumpadDivide,
        99 => KeyCode::PrintScreen,
        100 => KeyCode::RightAlt,
        102 => KeyCode::Home,
        103 => KeyCode::Up,
        104 => KeyCode::PageUp,
        105 => KeyCode::Left,
        106 => KeyCode::Right,
        107 => KeyCode::End,
        108 => KeyCode::Down,
        109 => KeyCode::PageDown,
        110 => KeyCode::Insert,
        111 => KeyCode::Delete,
        113 => KeyCode::VolumeMute,
        114 => KeyCode::VolumeDown,
        115 => KeyCode::VolumeUp,
        119 => KeyCode::Pause,
        125 => KeyCode::LeftMeta,
        126 => KeyCode::RightMeta,
        163 => KeyCode::MediaNext,
        164 => KeyCode::MediaPlayPause,
        165 => KeyCode::MediaPrev,
        166 => KeyCode::MediaStop,
        _ => KeyCode::Unknown(code),
    }
}

/// Convert KeyCode to evdev key code.
pub fn keycode_to_evdev(key: KeyCode) -> u16 {
    match key {
        KeyCode::Escape => 1,
        KeyCode::Key1 => 2,
        KeyCode::Key2 => 3,
        KeyCode::Key3 => 4,
        KeyCode::Key4 => 5,
        KeyCode::Key5 => 6,
        KeyCode::Key6 => 7,
        KeyCode::Key7 => 8,
        KeyCode::Key8 => 9,
        KeyCode::Key9 => 10,
        KeyCode::Key0 => 11,
        KeyCode::Minus => 12,
        KeyCode::Equal => 13,
        KeyCode::Backspace => 14,
        KeyCode::Tab => 15,
        KeyCode::Q => 16,
        KeyCode::W => 17,
        KeyCode::E => 18,
        KeyCode::R => 19,
        KeyCode::T => 20,
        KeyCode::Y => 21,
        KeyCode::U => 22,
        KeyCode::I => 23,
        KeyCode::O => 24,
        KeyCode::P => 25,
        KeyCode::LeftBracket => 26,
        KeyCode::RightBracket => 27,
        KeyCode::Enter => 28,
        KeyCode::LeftCtrl => 29,
        KeyCode::A => 30,
        KeyCode::S => 31,
        KeyCode::D => 32,
        KeyCode::F => 33,
        KeyCode::G => 34,
        KeyCode::H => 35,
        KeyCode::J => 36,
        KeyCode::K => 37,
        KeyCode::L => 38,
        KeyCode::Semicolon => 39,
        KeyCode::Apostrophe => 40,
        KeyCode::Grave => 41,
        KeyCode::LeftShift => 42,
        KeyCode::Backslash => 43,
        KeyCode::Z => 44,
        KeyCode::X => 45,
        KeyCode::C => 46,
        KeyCode::V => 47,
        KeyCode::B => 48,
        KeyCode::N => 49,
        KeyCode::M => 50,
        KeyCode::Comma => 51,
        KeyCode::Period => 52,
        KeyCode::Slash => 53,
        KeyCode::RightShift => 54,
        KeyCode::NumpadMultiply => 55,
        KeyCode::LeftAlt => 56,
        KeyCode::Space => 57,
        KeyCode::CapsLock => 58,
        KeyCode::F1 => 59,
        KeyCode::F2 => 60,
        KeyCode::F3 => 61,
        KeyCode::F4 => 62,
        KeyCode::F5 => 63,
        KeyCode::F6 => 64,
        KeyCode::F7 => 65,
        KeyCode::F8 => 66,
        KeyCode::F9 => 67,
        KeyCode::F10 => 68,
        KeyCode::NumLock => 69,
        KeyCode::ScrollLock => 70,
        KeyCode::Numpad7 => 71,
        KeyCode::Numpad8 => 72,
        KeyCode::Numpad9 => 73,
        KeyCode::NumpadSubtract => 74,
        KeyCode::Numpad4 => 75,
        KeyCode::Numpad5 => 76,
        KeyCode::Numpad6 => 77,
        KeyCode::NumpadAdd => 78,
        KeyCode::Numpad1 => 79,
        KeyCode::Numpad2 => 80,
        KeyCode::Numpad3 => 81,
        KeyCode::Numpad0 => 82,
        KeyCode::NumpadDecimal => 83,
        KeyCode::F11 => 87,
        KeyCode::F12 => 88,
        KeyCode::NumpadEnter => 96,
        KeyCode::RightCtrl => 97,
        KeyCode::NumpadDivide => 98,
        KeyCode::PrintScreen => 99,
        KeyCode::RightAlt => 100,
        KeyCode::Home => 102,
        KeyCode::Up => 103,
        KeyCode::PageUp => 104,
        KeyCode::Left => 105,
        KeyCode::Right => 106,
        KeyCode::End => 107,
        KeyCode::Down => 108,
        KeyCode::PageDown => 109,
        KeyCode::Insert => 110,
        KeyCode::Delete => 111,
        KeyCode::VolumeMute => 113,
        KeyCode::VolumeDown => 114,
        KeyCode::VolumeUp => 115,
        KeyCode::Pause => 119,
        KeyCode::LeftMeta => 125,
        KeyCode::RightMeta => 126,
        KeyCode::MediaNext => 163,
        KeyCode::MediaPlayPause => 164,
        KeyCode::MediaPrev => 165,
        KeyCode::MediaStop => 166,
        KeyCode::Unknown(code) => code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evdev_keycode_mapping() {
        assert_eq!(evdev_to_keycode(1), KeyCode::Escape);
        assert_eq!(evdev_to_keycode(30), KeyCode::A);
        assert_eq!(evdev_to_keycode(58), KeyCode::CapsLock);
        assert_eq!(evdev_to_keycode(57), KeyCode::Space);
        assert_eq!(evdev_to_keycode(28), KeyCode::Enter);
        assert_eq!(evdev_to_keycode(9999), KeyCode::Unknown(9999));
    }

    #[test]
    fn keycode_evdev_reverse_mapping() {
        // Test the reverse mapping: KeyCode -> evdev code
        assert_eq!(keycode_to_evdev(KeyCode::Escape), 1);
        assert_eq!(keycode_to_evdev(KeyCode::A), 30);
        assert_eq!(keycode_to_evdev(KeyCode::CapsLock), 58);
        assert_eq!(keycode_to_evdev(KeyCode::Space), 57);
        assert_eq!(keycode_to_evdev(KeyCode::Enter), 28);
        assert_eq!(keycode_to_evdev(KeyCode::Unknown(9999)), 9999);
    }

    #[test]
    fn keycode_evdev_roundtrip() {
        // Test roundtrip conversion: evdev -> KeyCode -> evdev
        // All known key codes should roundtrip correctly
        let test_codes: Vec<u16> = vec![
            // Letters
            30, 48, 46, 32, 18, 33, 34, 35, 23, 36, 37, 38, 50, 49, 24, 25, 16, 19, 31, 20, 22, 47,
            17, 45, 21, 44, // Numbers
            11, 2, 3, 4, 5, 6, 7, 8, 9, 10, // Function keys
            59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 87, 88, // Modifiers
            42, 54, 29, 97, 56, 100, 125, 126, // Navigation
            103, 108, 105, 106, 102, 107, 104, 109, // Editing
            110, 111, 14, // Whitespace
            57, 15, 28, // Locks
            58, 69, 70, // Escape area
            1, 99, 119, // Punctuation
            41, 12, 13, 26, 27, 43, 39, 40, 51, 52, 53, // Numpad
            82, 79, 80, 81, 75, 76, 77, 71, 72, 73, 78, 74, 55, 98, 96, 83, // Media
            115, 114, 113, 164, 166, 163, 165,
        ];

        for code in test_codes {
            let keycode = evdev_to_keycode(code);
            let back_to_code = keycode_to_evdev(keycode);
            assert_eq!(
                code, back_to_code,
                "Roundtrip failed for evdev code {}: got KeyCode::{:?} -> {}",
                code, keycode, back_to_code
            );
        }
    }

    #[test]
    fn keycode_evdev_roundtrip_from_keycode() {
        // Test roundtrip conversion: KeyCode -> evdev -> KeyCode
        // (for non-Unknown variants)
        let test_keycodes = vec![
            KeyCode::A,
            KeyCode::Z,
            KeyCode::Key0,
            KeyCode::Key9,
            KeyCode::F1,
            KeyCode::F12,
            KeyCode::Escape,
            KeyCode::CapsLock,
            KeyCode::Space,
            KeyCode::Enter,
            KeyCode::Tab,
            KeyCode::Backspace,
            KeyCode::LeftShift,
            KeyCode::RightShift,
            KeyCode::LeftCtrl,
            KeyCode::RightCtrl,
            KeyCode::LeftAlt,
            KeyCode::RightAlt,
            KeyCode::LeftMeta,
            KeyCode::RightMeta,
            KeyCode::Up,
            KeyCode::Down,
            KeyCode::Left,
            KeyCode::Right,
            KeyCode::Home,
            KeyCode::End,
            KeyCode::PageUp,
            KeyCode::PageDown,
            KeyCode::Insert,
            KeyCode::Delete,
            KeyCode::Numpad0,
            KeyCode::Numpad9,
            KeyCode::NumpadEnter,
            KeyCode::NumpadAdd,
            KeyCode::NumpadSubtract,
            KeyCode::NumpadMultiply,
            KeyCode::NumpadDivide,
            KeyCode::NumpadDecimal,
            KeyCode::VolumeUp,
            KeyCode::VolumeDown,
            KeyCode::VolumeMute,
            KeyCode::MediaPlayPause,
            KeyCode::MediaStop,
            KeyCode::MediaNext,
            KeyCode::MediaPrev,
        ];

        for keycode in test_keycodes {
            let evdev_code = keycode_to_evdev(keycode);
            let back_to_keycode = evdev_to_keycode(evdev_code);
            assert_eq!(
                keycode, back_to_keycode,
                "Roundtrip failed for {:?}: evdev {} -> {:?}",
                keycode, evdev_code, back_to_keycode
            );
        }
    }

    #[test]
    fn unknown_keycode_passthrough() {
        // Unknown keycodes should pass through unchanged
        let unknown = KeyCode::Unknown(12345);
        let evdev_code = keycode_to_evdev(unknown);
        assert_eq!(evdev_code, 12345);

        // And when converting back, unknown codes stay unknown
        let unknown_code = 9999u16;
        let keycode = evdev_to_keycode(unknown_code);
        assert_eq!(keycode, KeyCode::Unknown(9999));
    }

    #[test]
    fn all_evdev_codes_returns_all_codes() {
        let codes = all_evdev_codes();
        // Verify we have expected evdev codes
        assert!(codes.contains(&1)); // Escape
        assert!(codes.contains(&30)); // A
        assert!(codes.contains(&58)); // CapsLock
        assert!(codes.contains(&164)); // MediaPlayPause
    }
}
