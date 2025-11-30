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

fn evdev_to_keycode_alphanumeric(code: u16) -> Option<KeyCode> {
    match code {
        2 => Some(KeyCode::Key1),
        3 => Some(KeyCode::Key2),
        4 => Some(KeyCode::Key3),
        5 => Some(KeyCode::Key4),
        6 => Some(KeyCode::Key5),
        7 => Some(KeyCode::Key6),
        8 => Some(KeyCode::Key7),
        9 => Some(KeyCode::Key8),
        10 => Some(KeyCode::Key9),
        11 => Some(KeyCode::Key0),
        16 => Some(KeyCode::Q),
        17 => Some(KeyCode::W),
        18 => Some(KeyCode::E),
        19 => Some(KeyCode::R),
        20 => Some(KeyCode::T),
        21 => Some(KeyCode::Y),
        22 => Some(KeyCode::U),
        23 => Some(KeyCode::I),
        24 => Some(KeyCode::O),
        25 => Some(KeyCode::P),
        30 => Some(KeyCode::A),
        31 => Some(KeyCode::S),
        32 => Some(KeyCode::D),
        33 => Some(KeyCode::F),
        34 => Some(KeyCode::G),
        35 => Some(KeyCode::H),
        36 => Some(KeyCode::J),
        37 => Some(KeyCode::K),
        38 => Some(KeyCode::L),
        44 => Some(KeyCode::Z),
        45 => Some(KeyCode::X),
        46 => Some(KeyCode::C),
        47 => Some(KeyCode::V),
        48 => Some(KeyCode::B),
        49 => Some(KeyCode::N),
        50 => Some(KeyCode::M),
        _ => None,
    }
}

fn evdev_to_keycode_function_and_modifiers(code: u16) -> Option<KeyCode> {
    match code {
        59 => Some(KeyCode::F1),
        60 => Some(KeyCode::F2),
        61 => Some(KeyCode::F3),
        62 => Some(KeyCode::F4),
        63 => Some(KeyCode::F5),
        64 => Some(KeyCode::F6),
        65 => Some(KeyCode::F7),
        66 => Some(KeyCode::F8),
        67 => Some(KeyCode::F9),
        68 => Some(KeyCode::F10),
        87 => Some(KeyCode::F11),
        88 => Some(KeyCode::F12),
        29 => Some(KeyCode::LeftCtrl),
        97 => Some(KeyCode::RightCtrl),
        42 => Some(KeyCode::LeftShift),
        54 => Some(KeyCode::RightShift),
        56 => Some(KeyCode::LeftAlt),
        100 => Some(KeyCode::RightAlt),
        125 => Some(KeyCode::LeftMeta),
        126 => Some(KeyCode::RightMeta),
        _ => None,
    }
}

fn evdev_to_keycode_navigation_and_editing(code: u16) -> Option<KeyCode> {
    match code {
        102 => Some(KeyCode::Home),
        103 => Some(KeyCode::Up),
        104 => Some(KeyCode::PageUp),
        105 => Some(KeyCode::Left),
        106 => Some(KeyCode::Right),
        107 => Some(KeyCode::End),
        108 => Some(KeyCode::Down),
        109 => Some(KeyCode::PageDown),
        110 => Some(KeyCode::Insert),
        111 => Some(KeyCode::Delete),
        1 => Some(KeyCode::Escape),
        14 => Some(KeyCode::Backspace),
        15 => Some(KeyCode::Tab),
        28 => Some(KeyCode::Enter),
        57 => Some(KeyCode::Space),
        58 => Some(KeyCode::CapsLock),
        69 => Some(KeyCode::NumLock),
        70 => Some(KeyCode::ScrollLock),
        99 => Some(KeyCode::PrintScreen),
        119 => Some(KeyCode::Pause),
        _ => None,
    }
}

fn evdev_to_keycode_numpad_and_punctuation(code: u16) -> Option<KeyCode> {
    match code {
        71 => Some(KeyCode::Numpad7),
        72 => Some(KeyCode::Numpad8),
        73 => Some(KeyCode::Numpad9),
        74 => Some(KeyCode::NumpadSubtract),
        75 => Some(KeyCode::Numpad4),
        76 => Some(KeyCode::Numpad5),
        77 => Some(KeyCode::Numpad6),
        78 => Some(KeyCode::NumpadAdd),
        79 => Some(KeyCode::Numpad1),
        80 => Some(KeyCode::Numpad2),
        81 => Some(KeyCode::Numpad3),
        82 => Some(KeyCode::Numpad0),
        83 => Some(KeyCode::NumpadDecimal),
        55 => Some(KeyCode::NumpadMultiply),
        96 => Some(KeyCode::NumpadEnter),
        98 => Some(KeyCode::NumpadDivide),
        12 => Some(KeyCode::Minus),
        13 => Some(KeyCode::Equal),
        26 => Some(KeyCode::LeftBracket),
        27 => Some(KeyCode::RightBracket),
        39 => Some(KeyCode::Semicolon),
        40 => Some(KeyCode::Apostrophe),
        41 => Some(KeyCode::Grave),
        43 => Some(KeyCode::Backslash),
        51 => Some(KeyCode::Comma),
        52 => Some(KeyCode::Period),
        53 => Some(KeyCode::Slash),
        _ => None,
    }
}

fn evdev_to_keycode_media(code: u16) -> Option<KeyCode> {
    match code {
        113 => Some(KeyCode::VolumeMute),
        114 => Some(KeyCode::VolumeDown),
        115 => Some(KeyCode::VolumeUp),
        163 => Some(KeyCode::MediaNext),
        164 => Some(KeyCode::MediaPlayPause),
        165 => Some(KeyCode::MediaPrev),
        166 => Some(KeyCode::MediaStop),
        _ => None,
    }
}

/// Convert evdev key code to KeyCode.
pub fn evdev_to_keycode(code: u16) -> KeyCode {
    evdev_to_keycode_alphanumeric(code)
        .or_else(|| evdev_to_keycode_function_and_modifiers(code))
        .or_else(|| evdev_to_keycode_navigation_and_editing(code))
        .or_else(|| evdev_to_keycode_numpad_and_punctuation(code))
        .or_else(|| evdev_to_keycode_media(code))
        .unwrap_or(KeyCode::Unknown(code))
}

fn keycode_to_evdev_alphanumeric(key: KeyCode) -> Option<u16> {
    match key {
        KeyCode::Key1 => Some(2),
        KeyCode::Key2 => Some(3),
        KeyCode::Key3 => Some(4),
        KeyCode::Key4 => Some(5),
        KeyCode::Key5 => Some(6),
        KeyCode::Key6 => Some(7),
        KeyCode::Key7 => Some(8),
        KeyCode::Key8 => Some(9),
        KeyCode::Key9 => Some(10),
        KeyCode::Key0 => Some(11),
        KeyCode::Q => Some(16),
        KeyCode::W => Some(17),
        KeyCode::E => Some(18),
        KeyCode::R => Some(19),
        KeyCode::T => Some(20),
        KeyCode::Y => Some(21),
        KeyCode::U => Some(22),
        KeyCode::I => Some(23),
        KeyCode::O => Some(24),
        KeyCode::P => Some(25),
        KeyCode::A => Some(30),
        KeyCode::S => Some(31),
        KeyCode::D => Some(32),
        KeyCode::F => Some(33),
        KeyCode::G => Some(34),
        KeyCode::H => Some(35),
        KeyCode::J => Some(36),
        KeyCode::K => Some(37),
        KeyCode::L => Some(38),
        KeyCode::Z => Some(44),
        KeyCode::X => Some(45),
        KeyCode::C => Some(46),
        KeyCode::V => Some(47),
        KeyCode::B => Some(48),
        KeyCode::N => Some(49),
        KeyCode::M => Some(50),
        _ => None,
    }
}

fn keycode_to_evdev_function_and_modifiers(key: KeyCode) -> Option<u16> {
    match key {
        KeyCode::F1 => Some(59),
        KeyCode::F2 => Some(60),
        KeyCode::F3 => Some(61),
        KeyCode::F4 => Some(62),
        KeyCode::F5 => Some(63),
        KeyCode::F6 => Some(64),
        KeyCode::F7 => Some(65),
        KeyCode::F8 => Some(66),
        KeyCode::F9 => Some(67),
        KeyCode::F10 => Some(68),
        KeyCode::F11 => Some(87),
        KeyCode::F12 => Some(88),
        KeyCode::LeftCtrl => Some(29),
        KeyCode::RightCtrl => Some(97),
        KeyCode::LeftShift => Some(42),
        KeyCode::RightShift => Some(54),
        KeyCode::LeftAlt => Some(56),
        KeyCode::RightAlt => Some(100),
        KeyCode::LeftMeta => Some(125),
        KeyCode::RightMeta => Some(126),
        _ => None,
    }
}

fn keycode_to_evdev_navigation_and_editing(key: KeyCode) -> Option<u16> {
    match key {
        KeyCode::Home => Some(102),
        KeyCode::Up => Some(103),
        KeyCode::PageUp => Some(104),
        KeyCode::Left => Some(105),
        KeyCode::Right => Some(106),
        KeyCode::End => Some(107),
        KeyCode::Down => Some(108),
        KeyCode::PageDown => Some(109),
        KeyCode::Insert => Some(110),
        KeyCode::Delete => Some(111),
        KeyCode::Escape => Some(1),
        KeyCode::Backspace => Some(14),
        KeyCode::Tab => Some(15),
        KeyCode::Enter => Some(28),
        KeyCode::Space => Some(57),
        KeyCode::CapsLock => Some(58),
        KeyCode::NumLock => Some(69),
        KeyCode::ScrollLock => Some(70),
        KeyCode::PrintScreen => Some(99),
        KeyCode::Pause => Some(119),
        _ => None,
    }
}

fn keycode_to_evdev_numpad_and_punctuation(key: KeyCode) -> Option<u16> {
    match key {
        KeyCode::Numpad7 => Some(71),
        KeyCode::Numpad8 => Some(72),
        KeyCode::Numpad9 => Some(73),
        KeyCode::NumpadSubtract => Some(74),
        KeyCode::Numpad4 => Some(75),
        KeyCode::Numpad5 => Some(76),
        KeyCode::Numpad6 => Some(77),
        KeyCode::NumpadAdd => Some(78),
        KeyCode::Numpad1 => Some(79),
        KeyCode::Numpad2 => Some(80),
        KeyCode::Numpad3 => Some(81),
        KeyCode::Numpad0 => Some(82),
        KeyCode::NumpadDecimal => Some(83),
        KeyCode::NumpadMultiply => Some(55),
        KeyCode::NumpadEnter => Some(96),
        KeyCode::NumpadDivide => Some(98),
        KeyCode::Minus => Some(12),
        KeyCode::Equal => Some(13),
        KeyCode::LeftBracket => Some(26),
        KeyCode::RightBracket => Some(27),
        KeyCode::Semicolon => Some(39),
        KeyCode::Apostrophe => Some(40),
        KeyCode::Grave => Some(41),
        KeyCode::Backslash => Some(43),
        KeyCode::Comma => Some(51),
        KeyCode::Period => Some(52),
        KeyCode::Slash => Some(53),
        _ => None,
    }
}

fn keycode_to_evdev_media(key: KeyCode) -> Option<u16> {
    match key {
        KeyCode::VolumeMute => Some(113),
        KeyCode::VolumeDown => Some(114),
        KeyCode::VolumeUp => Some(115),
        KeyCode::MediaNext => Some(163),
        KeyCode::MediaPlayPause => Some(164),
        KeyCode::MediaPrev => Some(165),
        KeyCode::MediaStop => Some(166),
        _ => None,
    }
}

/// Convert KeyCode to evdev key code.
pub fn keycode_to_evdev(key: KeyCode) -> u16 {
    if let KeyCode::Unknown(code) = key {
        return code;
    }

    if let Some(code) = keycode_to_evdev_alphanumeric(key) {
        return code;
    }
    if let Some(code) = keycode_to_evdev_function_and_modifiers(key) {
        return code;
    }
    if let Some(code) = keycode_to_evdev_navigation_and_editing(key) {
        return code;
    }
    if let Some(code) = keycode_to_evdev_numpad_and_punctuation(key) {
        return code;
    }
    if let Some(code) = keycode_to_evdev_media(key) {
        return code;
    }

    unreachable!("All KeyCode variants should be handled");
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
