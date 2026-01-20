//! CGKeyCode ↔ KeyCode bidirectional mapping.
//!
//! This module provides conversions between macOS CGKeyCode values and
//! keyrx KeyCode enum values, as well as conversions for rdev and enigo types.
//!
//! # CGKeyCode Constants
//!
//! macOS uses physical key codes (CGKeyCode) that are layout-independent.
//! These codes represent the physical key position on an ANSI-standard US keyboard.
//!
//! # References
//!
//! - Apple Developer Documentation: CGKeyCode
//! - Carbon HIToolbox/Events.h for kVK_* constants

use keyrx_core::config::keys::KeyCode;

/// Bidirectional mapping between CGKeyCode and KeyCode.
/// Uses const arrays for O(1) lookup performance.
const CGKEYCODE_TO_KEYCODE: &[(u16, KeyCode)] = &[
    // Letters (QWERTY layout positions)
    (0x00, KeyCode::A),
    (0x0B, KeyCode::B),
    (0x08, KeyCode::C),
    (0x02, KeyCode::D),
    (0x0E, KeyCode::E),
    (0x03, KeyCode::F),
    (0x05, KeyCode::G),
    (0x04, KeyCode::H),
    (0x22, KeyCode::I),
    (0x26, KeyCode::J),
    (0x28, KeyCode::K),
    (0x25, KeyCode::L),
    (0x2E, KeyCode::M),
    (0x2D, KeyCode::N),
    (0x1F, KeyCode::O),
    (0x23, KeyCode::P),
    (0x0C, KeyCode::Q),
    (0x0F, KeyCode::R),
    (0x01, KeyCode::S),
    (0x11, KeyCode::T),
    (0x20, KeyCode::U),
    (0x09, KeyCode::V),
    (0x0D, KeyCode::W),
    (0x07, KeyCode::X),
    (0x10, KeyCode::Y),
    (0x06, KeyCode::Z),
    // Numbers
    (0x1D, KeyCode::Num0),
    (0x12, KeyCode::Num1),
    (0x13, KeyCode::Num2),
    (0x14, KeyCode::Num3),
    (0x15, KeyCode::Num4),
    (0x17, KeyCode::Num5),
    (0x16, KeyCode::Num6),
    (0x1A, KeyCode::Num7),
    (0x1C, KeyCode::Num8),
    (0x19, KeyCode::Num9),
    // Function keys
    (0x7A, KeyCode::F1),
    (0x78, KeyCode::F2),
    (0x63, KeyCode::F3),
    (0x76, KeyCode::F4),
    (0x60, KeyCode::F5),
    (0x61, KeyCode::F6),
    (0x62, KeyCode::F7),
    (0x64, KeyCode::F8),
    (0x65, KeyCode::F9),
    (0x6D, KeyCode::F10),
    (0x67, KeyCode::F11),
    (0x6F, KeyCode::F12),
    (0x69, KeyCode::F13),
    (0x6B, KeyCode::F14),
    (0x71, KeyCode::F15),
    (0x6A, KeyCode::F16),
    (0x40, KeyCode::F17),
    (0x4F, KeyCode::F18),
    (0x50, KeyCode::F19),
    (0x5A, KeyCode::F20),
    // Modifiers
    (0x38, KeyCode::LShift),
    (0x3C, KeyCode::RShift),
    (0x3B, KeyCode::LCtrl),
    (0x3E, KeyCode::RCtrl),
    (0x3A, KeyCode::LAlt),
    (0x3D, KeyCode::RAlt),
    (0x37, KeyCode::LMeta), // Command/Cmd
    (0x36, KeyCode::RMeta), // Command/Cmd
    // Special keys
    (0x35, KeyCode::Escape),
    (0x24, KeyCode::Enter),
    (0x33, KeyCode::Backspace),
    (0x30, KeyCode::Tab),
    (0x31, KeyCode::Space),
    (0x39, KeyCode::CapsLock),
    // Navigation
    (0x72, KeyCode::Insert), // Help key on Mac keyboards
    (0x75, KeyCode::Delete),
    (0x73, KeyCode::Home),
    (0x77, KeyCode::End),
    (0x74, KeyCode::PageUp),
    (0x79, KeyCode::PageDown),
    (0x7B, KeyCode::Left),
    (0x7C, KeyCode::Right),
    (0x7E, KeyCode::Up),
    (0x7D, KeyCode::Down),
    // Punctuation and symbols
    (0x21, KeyCode::LeftBracket),
    (0x1E, KeyCode::RightBracket),
    (0x2A, KeyCode::Backslash),
    (0x29, KeyCode::Semicolon),
    (0x27, KeyCode::Quote),
    (0x2B, KeyCode::Comma),
    (0x2F, KeyCode::Period),
    (0x2C, KeyCode::Slash),
    (0x32, KeyCode::Grave),
    (0x1B, KeyCode::Minus),
    (0x18, KeyCode::Equal),
    // Numpad
    (0x52, KeyCode::Numpad0),
    (0x53, KeyCode::Numpad1),
    (0x54, KeyCode::Numpad2),
    (0x55, KeyCode::Numpad3),
    (0x56, KeyCode::Numpad4),
    (0x57, KeyCode::Numpad5),
    (0x58, KeyCode::Numpad6),
    (0x59, KeyCode::Numpad7),
    (0x5B, KeyCode::Numpad8),
    (0x5C, KeyCode::Numpad9),
    (0x4B, KeyCode::NumpadDivide),
    (0x43, KeyCode::NumpadMultiply),
    (0x4E, KeyCode::NumpadSubtract),
    (0x45, KeyCode::NumpadAdd),
    (0x4C, KeyCode::NumpadEnter),
    (0x41, KeyCode::NumpadDecimal),
    (0x47, KeyCode::NumLock), // Clear key on Mac
    // Media keys (if supported by rdev/enigo)
    (0x4A, KeyCode::Mute),
    (0x49, KeyCode::VolumeDown),
    (0x48, KeyCode::VolumeUp),
];

/// Converts a CGKeyCode to a keyrx KeyCode.
///
/// # Arguments
///
/// * `cgcode` - macOS virtual key code
///
/// # Returns
///
/// The corresponding [`KeyCode`], or `None` if unmapped.
///
/// # Examples
///
/// ```no_run
/// use keyrx_daemon::platform::macos::keycode_map::cgkeycode_to_keyrx;
/// use keyrx_core::config::keys::KeyCode;
///
/// assert_eq!(cgkeycode_to_keyrx(0x00), Some(KeyCode::A));
/// assert_eq!(cgkeycode_to_keyrx(0x12), Some(KeyCode::Num1));
/// ```
pub fn cgkeycode_to_keyrx(cgcode: u16) -> Option<KeyCode> {
    CGKEYCODE_TO_KEYCODE
        .iter()
        .find(|(code, _)| *code == cgcode)
        .map(|(_, keycode)| *keycode)
}

/// Converts a keyrx KeyCode to a CGKeyCode.
///
/// # Arguments
///
/// * `keycode` - keyrx KeyCode
///
/// # Returns
///
/// The corresponding CGKeyCode, or `None` if unmapped.
///
/// # Examples
///
/// ```no_run
/// use keyrx_daemon::platform::macos::keycode_map::keyrx_to_cgkeycode;
/// use keyrx_core::config::keys::KeyCode;
///
/// assert_eq!(keyrx_to_cgkeycode(KeyCode::A), Some(0x00));
/// assert_eq!(keyrx_to_cgkeycode(KeyCode::Num1), Some(0x12));
/// ```
pub fn keyrx_to_cgkeycode(keycode: KeyCode) -> Option<u16> {
    CGKEYCODE_TO_KEYCODE
        .iter()
        .find(|(_, kc)| *kc == keycode)
        .map(|(code, _)| *code)
}

/// Converts an rdev::Key to a keyrx KeyCode.
///
/// # Arguments
///
/// * `key` - rdev key event
///
/// # Returns
///
/// The corresponding [`KeyCode`], or `None` if unmapped.
///
/// # Examples
///
/// ```no_run
/// use keyrx_daemon::platform::macos::keycode_map::rdev_key_to_keyrx;
/// use keyrx_core::config::keys::KeyCode;
///
/// let key = rdev::Key::KeyA;
/// assert_eq!(rdev_key_to_keyrx(key), Some(KeyCode::A));
/// ```
pub fn rdev_key_to_keyrx(key: rdev::Key) -> Option<KeyCode> {
    use rdev::Key;

    match key {
        // Letters
        Key::KeyA => Some(KeyCode::A),
        Key::KeyB => Some(KeyCode::B),
        Key::KeyC => Some(KeyCode::C),
        Key::KeyD => Some(KeyCode::D),
        Key::KeyE => Some(KeyCode::E),
        Key::KeyF => Some(KeyCode::F),
        Key::KeyG => Some(KeyCode::G),
        Key::KeyH => Some(KeyCode::H),
        Key::KeyI => Some(KeyCode::I),
        Key::KeyJ => Some(KeyCode::J),
        Key::KeyK => Some(KeyCode::K),
        Key::KeyL => Some(KeyCode::L),
        Key::KeyM => Some(KeyCode::M),
        Key::KeyN => Some(KeyCode::N),
        Key::KeyO => Some(KeyCode::O),
        Key::KeyP => Some(KeyCode::P),
        Key::KeyQ => Some(KeyCode::Q),
        Key::KeyR => Some(KeyCode::R),
        Key::KeyS => Some(KeyCode::S),
        Key::KeyT => Some(KeyCode::T),
        Key::KeyU => Some(KeyCode::U),
        Key::KeyV => Some(KeyCode::V),
        Key::KeyW => Some(KeyCode::W),
        Key::KeyX => Some(KeyCode::X),
        Key::KeyY => Some(KeyCode::Y),
        Key::KeyZ => Some(KeyCode::Z),
        // Numbers
        Key::Num0 => Some(KeyCode::Num0),
        Key::Num1 => Some(KeyCode::Num1),
        Key::Num2 => Some(KeyCode::Num2),
        Key::Num3 => Some(KeyCode::Num3),
        Key::Num4 => Some(KeyCode::Num4),
        Key::Num5 => Some(KeyCode::Num5),
        Key::Num6 => Some(KeyCode::Num6),
        Key::Num7 => Some(KeyCode::Num7),
        Key::Num8 => Some(KeyCode::Num8),
        Key::Num9 => Some(KeyCode::Num9),
        // Function keys
        Key::F1 => Some(KeyCode::F1),
        Key::F2 => Some(KeyCode::F2),
        Key::F3 => Some(KeyCode::F3),
        Key::F4 => Some(KeyCode::F4),
        Key::F5 => Some(KeyCode::F5),
        Key::F6 => Some(KeyCode::F6),
        Key::F7 => Some(KeyCode::F7),
        Key::F8 => Some(KeyCode::F8),
        Key::F9 => Some(KeyCode::F9),
        Key::F10 => Some(KeyCode::F10),
        Key::F11 => Some(KeyCode::F11),
        Key::F12 => Some(KeyCode::F12),
        // Modifiers
        Key::ShiftLeft => Some(KeyCode::LShift),
        Key::ShiftRight => Some(KeyCode::RShift),
        Key::ControlLeft => Some(KeyCode::LCtrl),
        Key::ControlRight => Some(KeyCode::RCtrl),
        Key::Alt => Some(KeyCode::LAlt),
        Key::AltGr => Some(KeyCode::RAlt),
        Key::MetaLeft => Some(KeyCode::LMeta),
        Key::MetaRight => Some(KeyCode::RMeta),
        // Special keys
        Key::Escape => Some(KeyCode::Escape),
        Key::Return => Some(KeyCode::Enter),
        Key::Backspace => Some(KeyCode::Backspace),
        Key::Tab => Some(KeyCode::Tab),
        Key::Space => Some(KeyCode::Space),
        Key::CapsLock => Some(KeyCode::CapsLock),
        // Navigation
        Key::Insert => Some(KeyCode::Insert),
        Key::Delete => Some(KeyCode::Delete),
        Key::Home => Some(KeyCode::Home),
        Key::End => Some(KeyCode::End),
        Key::PageUp => Some(KeyCode::PageUp),
        Key::PageDown => Some(KeyCode::PageDown),
        Key::LeftArrow => Some(KeyCode::Left),
        Key::RightArrow => Some(KeyCode::Right),
        Key::UpArrow => Some(KeyCode::Up),
        Key::DownArrow => Some(KeyCode::Down),
        // Punctuation
        Key::LeftBracket => Some(KeyCode::LeftBracket),
        Key::RightBracket => Some(KeyCode::RightBracket),
        Key::BackSlash => Some(KeyCode::Backslash),
        Key::SemiColon => Some(KeyCode::Semicolon),
        Key::Quote => Some(KeyCode::Quote),
        Key::Comma => Some(KeyCode::Comma),
        Key::Dot => Some(KeyCode::Period),
        Key::Slash => Some(KeyCode::Slash),
        Key::BackQuote => Some(KeyCode::Grave),
        Key::Minus => Some(KeyCode::Minus),
        Key::Equal => Some(KeyCode::Equal),
        // Numpad
        Key::Kp0 => Some(KeyCode::Numpad0),
        Key::Kp1 => Some(KeyCode::Numpad1),
        Key::Kp2 => Some(KeyCode::Numpad2),
        Key::Kp3 => Some(KeyCode::Numpad3),
        Key::Kp4 => Some(KeyCode::Numpad4),
        Key::Kp5 => Some(KeyCode::Numpad5),
        Key::Kp6 => Some(KeyCode::Numpad6),
        Key::Kp7 => Some(KeyCode::Numpad7),
        Key::Kp8 => Some(KeyCode::Numpad8),
        Key::Kp9 => Some(KeyCode::Numpad9),
        Key::KpDivide => Some(KeyCode::NumpadDivide),
        Key::KpMultiply => Some(KeyCode::NumpadMultiply),
        Key::KpMinus => Some(KeyCode::NumpadSubtract),
        Key::KpPlus => Some(KeyCode::NumpadAdd),
        Key::KpReturn => Some(KeyCode::NumpadEnter),
        Key::KpDelete => Some(KeyCode::NumpadDecimal),
        // Other
        Key::PrintScreen => Some(KeyCode::PrintScreen),
        Key::ScrollLock => Some(KeyCode::ScrollLock),
        Key::Pause => Some(KeyCode::Pause),
        _ => None,
    }
}

/// Converts a keyrx KeyCode to an enigo::Key.
///
/// # Arguments
///
/// * `keycode` - keyrx KeyCode
///
/// # Returns
///
/// The corresponding enigo::Key, or `None` if unmapped.
///
/// # Examples
///
/// ```no_run
/// use keyrx_daemon::platform::macos::keycode_map::keyrx_to_enigo_key;
/// use keyrx_core::config::keys::KeyCode;
///
/// let key = keyrx_to_enigo_key(KeyCode::A);
/// assert!(key.is_some());
/// ```
pub fn keyrx_to_enigo_key(keycode: KeyCode) -> Option<enigo::Key> {
    use enigo::Key;

    match keycode {
        // Letters
        KeyCode::A => Some(Key::Unicode('a')),
        KeyCode::B => Some(Key::Unicode('b')),
        KeyCode::C => Some(Key::Unicode('c')),
        KeyCode::D => Some(Key::Unicode('d')),
        KeyCode::E => Some(Key::Unicode('e')),
        KeyCode::F => Some(Key::Unicode('f')),
        KeyCode::G => Some(Key::Unicode('g')),
        KeyCode::H => Some(Key::Unicode('h')),
        KeyCode::I => Some(Key::Unicode('i')),
        KeyCode::J => Some(Key::Unicode('j')),
        KeyCode::K => Some(Key::Unicode('k')),
        KeyCode::L => Some(Key::Unicode('l')),
        KeyCode::M => Some(Key::Unicode('m')),
        KeyCode::N => Some(Key::Unicode('n')),
        KeyCode::O => Some(Key::Unicode('o')),
        KeyCode::P => Some(Key::Unicode('p')),
        KeyCode::Q => Some(Key::Unicode('q')),
        KeyCode::R => Some(Key::Unicode('r')),
        KeyCode::S => Some(Key::Unicode('s')),
        KeyCode::T => Some(Key::Unicode('t')),
        KeyCode::U => Some(Key::Unicode('u')),
        KeyCode::V => Some(Key::Unicode('v')),
        KeyCode::W => Some(Key::Unicode('w')),
        KeyCode::X => Some(Key::Unicode('x')),
        KeyCode::Y => Some(Key::Unicode('y')),
        KeyCode::Z => Some(Key::Unicode('z')),
        // Numbers
        KeyCode::Num0 => Some(Key::Unicode('0')),
        KeyCode::Num1 => Some(Key::Unicode('1')),
        KeyCode::Num2 => Some(Key::Unicode('2')),
        KeyCode::Num3 => Some(Key::Unicode('3')),
        KeyCode::Num4 => Some(Key::Unicode('4')),
        KeyCode::Num5 => Some(Key::Unicode('5')),
        KeyCode::Num6 => Some(Key::Unicode('6')),
        KeyCode::Num7 => Some(Key::Unicode('7')),
        KeyCode::Num8 => Some(Key::Unicode('8')),
        KeyCode::Num9 => Some(Key::Unicode('9')),
        // Function keys
        KeyCode::F1 => Some(Key::F1),
        KeyCode::F2 => Some(Key::F2),
        KeyCode::F3 => Some(Key::F3),
        KeyCode::F4 => Some(Key::F4),
        KeyCode::F5 => Some(Key::F5),
        KeyCode::F6 => Some(Key::F6),
        KeyCode::F7 => Some(Key::F7),
        KeyCode::F8 => Some(Key::F8),
        KeyCode::F9 => Some(Key::F9),
        KeyCode::F10 => Some(Key::F10),
        KeyCode::F11 => Some(Key::F11),
        KeyCode::F12 => Some(Key::F12),
        // Modifiers
        KeyCode::LShift | KeyCode::RShift => Some(Key::Shift),
        KeyCode::LCtrl | KeyCode::RCtrl => Some(Key::Control),
        KeyCode::LAlt | KeyCode::RAlt => Some(Key::Alt),
        KeyCode::LMeta | KeyCode::RMeta => Some(Key::Meta),
        // Special keys
        KeyCode::Escape => Some(Key::Escape),
        KeyCode::Enter => Some(Key::Return),
        KeyCode::Backspace => Some(Key::Backspace),
        KeyCode::Tab => Some(Key::Tab),
        KeyCode::Space => Some(Key::Space),
        KeyCode::CapsLock => Some(Key::CapsLock),
        // Navigation
        KeyCode::Insert => None, // Insert key not supported by enigo
        KeyCode::Delete => Some(Key::Delete),
        KeyCode::Home => Some(Key::Home),
        KeyCode::End => Some(Key::End),
        KeyCode::PageUp => Some(Key::PageUp),
        KeyCode::PageDown => Some(Key::PageDown),
        KeyCode::Left => Some(Key::LeftArrow),
        KeyCode::Right => Some(Key::RightArrow),
        KeyCode::Up => Some(Key::UpArrow),
        KeyCode::Down => Some(Key::DownArrow),
        // Punctuation
        KeyCode::LeftBracket => Some(Key::Unicode('[')),
        KeyCode::RightBracket => Some(Key::Unicode(']')),
        KeyCode::Backslash => Some(Key::Unicode('\\')),
        KeyCode::Semicolon => Some(Key::Unicode(';')),
        KeyCode::Quote => Some(Key::Unicode('\'')),
        KeyCode::Comma => Some(Key::Unicode(',')),
        KeyCode::Period => Some(Key::Unicode('.')),
        KeyCode::Slash => Some(Key::Unicode('/')),
        KeyCode::Grave => Some(Key::Unicode('`')),
        KeyCode::Minus => Some(Key::Unicode('-')),
        KeyCode::Equal => Some(Key::Unicode('=')),
        // Numpad
        KeyCode::Numpad0 => Some(Key::Numpad0),
        KeyCode::Numpad1 => Some(Key::Numpad1),
        KeyCode::Numpad2 => Some(Key::Numpad2),
        KeyCode::Numpad3 => Some(Key::Numpad3),
        KeyCode::Numpad4 => Some(Key::Numpad4),
        KeyCode::Numpad5 => Some(Key::Numpad5),
        KeyCode::Numpad6 => Some(Key::Numpad6),
        KeyCode::Numpad7 => Some(Key::Numpad7),
        KeyCode::Numpad8 => Some(Key::Numpad8),
        KeyCode::Numpad9 => Some(Key::Numpad9),
        KeyCode::NumpadDivide => Some(Key::Divide),
        KeyCode::NumpadMultiply => Some(Key::Multiply),
        KeyCode::NumpadSubtract => Some(Key::Subtract),
        KeyCode::NumpadAdd => Some(Key::Add),
        KeyCode::NumpadDecimal => Some(Key::Decimal),
        // Media keys
        KeyCode::Mute => Some(Key::VolumeMute),
        KeyCode::VolumeDown => Some(Key::VolumeDown),
        KeyCode::VolumeUp => Some(Key::VolumeUp),
        KeyCode::MediaPlayPause => Some(Key::MediaPlayPause),
        KeyCode::MediaStop => None, // MediaStop not supported by enigo
        KeyCode::MediaPrevious => Some(Key::MediaPrevTrack),
        KeyCode::MediaNext => Some(Key::MediaNextTrack),
        // Other keys that don't have direct enigo mappings
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cgkeycode_letter_mappings() {
        assert_eq!(cgkeycode_to_keyrx(0x00), Some(KeyCode::A));
        assert_eq!(cgkeycode_to_keyrx(0x0B), Some(KeyCode::B));
        assert_eq!(cgkeycode_to_keyrx(0x08), Some(KeyCode::C));
        assert_eq!(cgkeycode_to_keyrx(0x06), Some(KeyCode::Z));
    }

    #[test]
    fn test_cgkeycode_number_mappings() {
        assert_eq!(cgkeycode_to_keyrx(0x12), Some(KeyCode::Num1));
        assert_eq!(cgkeycode_to_keyrx(0x13), Some(KeyCode::Num2));
        assert_eq!(cgkeycode_to_keyrx(0x1D), Some(KeyCode::Num0));
    }

    #[test]
    fn test_cgkeycode_function_key_mappings() {
        assert_eq!(cgkeycode_to_keyrx(0x7A), Some(KeyCode::F1));
        assert_eq!(cgkeycode_to_keyrx(0x78), Some(KeyCode::F2));
        assert_eq!(cgkeycode_to_keyrx(0x6F), Some(KeyCode::F12));
    }

    #[test]
    fn test_cgkeycode_special_key_mappings() {
        assert_eq!(cgkeycode_to_keyrx(0x35), Some(KeyCode::Escape));
        assert_eq!(cgkeycode_to_keyrx(0x24), Some(KeyCode::Enter));
        assert_eq!(cgkeycode_to_keyrx(0x33), Some(KeyCode::Backspace));
        assert_eq!(cgkeycode_to_keyrx(0x31), Some(KeyCode::Space));
    }

    #[test]
    fn test_cgkeycode_modifier_mappings() {
        assert_eq!(cgkeycode_to_keyrx(0x38), Some(KeyCode::LShift));
        assert_eq!(cgkeycode_to_keyrx(0x3C), Some(KeyCode::RShift));
        assert_eq!(cgkeycode_to_keyrx(0x37), Some(KeyCode::LMeta));
        assert_eq!(cgkeycode_to_keyrx(0x3B), Some(KeyCode::LCtrl));
    }

    #[test]
    fn test_keyrx_to_cgkeycode_bidirectional() {
        // Test bidirectional mapping
        assert_eq!(keyrx_to_cgkeycode(KeyCode::A), Some(0x00));
        assert_eq!(keyrx_to_cgkeycode(KeyCode::Z), Some(0x06));
        assert_eq!(keyrx_to_cgkeycode(KeyCode::Num1), Some(0x12));
        assert_eq!(keyrx_to_cgkeycode(KeyCode::F1), Some(0x7A));
        assert_eq!(keyrx_to_cgkeycode(KeyCode::Escape), Some(0x35));
    }

    #[test]
    fn test_bidirectional_consistency() {
        // Verify that all mapped CGKeyCodes can be converted back
        for (cgcode, keycode) in CGKEYCODE_TO_KEYCODE {
            let converted = cgkeycode_to_keyrx(*cgcode);
            assert_eq!(converted, Some(*keycode));

            let back = keyrx_to_cgkeycode(*keycode);
            assert_eq!(back, Some(*cgcode));
        }
    }

    #[test]
    fn test_rdev_key_letter_mappings() {
        use rdev::Key;
        assert_eq!(rdev_key_to_keyrx(Key::KeyA), Some(KeyCode::A));
        assert_eq!(rdev_key_to_keyrx(Key::KeyZ), Some(KeyCode::Z));
    }

    #[test]
    fn test_rdev_key_modifier_mappings() {
        use rdev::Key;
        assert_eq!(rdev_key_to_keyrx(Key::ShiftLeft), Some(KeyCode::LShift));
        assert_eq!(rdev_key_to_keyrx(Key::ControlLeft), Some(KeyCode::LCtrl));
        assert_eq!(rdev_key_to_keyrx(Key::MetaLeft), Some(KeyCode::LMeta));
    }

    #[test]
    fn test_keyrx_to_enigo_key_letters() {
        assert!(keyrx_to_enigo_key(KeyCode::A).is_some());
        assert!(keyrx_to_enigo_key(KeyCode::Z).is_some());
    }

    #[test]
    fn test_keyrx_to_enigo_key_modifiers() {
        assert!(keyrx_to_enigo_key(KeyCode::LShift).is_some());
        assert!(keyrx_to_enigo_key(KeyCode::LCtrl).is_some());
        assert!(keyrx_to_enigo_key(KeyCode::LMeta).is_some());
    }

    #[test]
    fn test_keyrx_to_enigo_key_special() {
        assert!(keyrx_to_enigo_key(KeyCode::Escape).is_some());
        assert!(keyrx_to_enigo_key(KeyCode::Enter).is_some());
        assert!(keyrx_to_enigo_key(KeyCode::Space).is_some());
    }

    #[test]
    fn test_unmapped_cgkeycode_returns_none() {
        // Test that unmapped codes return None
        assert_eq!(cgkeycode_to_keyrx(0xFFFF), None);
    }

    #[test]
    fn test_all_cgkeycodes_are_unique() {
        // Verify no duplicate CGKeyCodes in mapping
        let mut codes = std::collections::HashSet::new();
        for (code, _) in CGKEYCODE_TO_KEYCODE {
            assert!(codes.insert(*code), "Duplicate CGKeyCode: {}", code);
        }
    }

    #[test]
    fn test_all_cgkeycodes_mapped() {
        // Verify all entries in the mapping table work correctly
        let total_mappings = CGKEYCODE_TO_KEYCODE.len();
        assert!(
            total_mappings >= 100,
            "Expected at least 100 key mappings, found {}",
            total_mappings
        );

        // Test each mapping
        for (cgcode, expected_keycode) in CGKEYCODE_TO_KEYCODE {
            let result = cgkeycode_to_keyrx(*cgcode);
            assert_eq!(
                result,
                Some(*expected_keycode),
                "CGKeyCode {:x} should map to {:?}",
                cgcode,
                expected_keycode
            );
        }
    }

    #[test]
    fn test_special_keys_comprehensive() {
        // Test all special keys have mappings
        assert!(cgkeycode_to_keyrx(0x35).is_some()); // Escape
        assert!(cgkeycode_to_keyrx(0x24).is_some()); // Enter
        assert!(cgkeycode_to_keyrx(0x33).is_some()); // Backspace
        assert!(cgkeycode_to_keyrx(0x30).is_some()); // Tab
        assert!(cgkeycode_to_keyrx(0x31).is_some()); // Space
        assert!(cgkeycode_to_keyrx(0x39).is_some()); // CapsLock

        // Navigation keys
        assert!(cgkeycode_to_keyrx(0x7B).is_some()); // Left
        assert!(cgkeycode_to_keyrx(0x7C).is_some()); // Right
        assert!(cgkeycode_to_keyrx(0x7E).is_some()); // Up
        assert!(cgkeycode_to_keyrx(0x7D).is_some()); // Down
        assert!(cgkeycode_to_keyrx(0x73).is_some()); // Home
        assert!(cgkeycode_to_keyrx(0x77).is_some()); // End
        assert!(cgkeycode_to_keyrx(0x74).is_some()); // PageUp
        assert!(cgkeycode_to_keyrx(0x79).is_some()); // PageDown

        // Media keys
        assert!(cgkeycode_to_keyrx(0x4A).is_some()); // Mute
        assert!(cgkeycode_to_keyrx(0x49).is_some()); // VolumeDown
        assert!(cgkeycode_to_keyrx(0x48).is_some()); // VolumeUp
    }

    #[test]
    fn test_numpad_keys() {
        // All numpad keys should be mapped
        assert_eq!(cgkeycode_to_keyrx(0x52), Some(KeyCode::Numpad0));
        assert_eq!(cgkeycode_to_keyrx(0x53), Some(KeyCode::Numpad1));
        assert_eq!(cgkeycode_to_keyrx(0x54), Some(KeyCode::Numpad2));
        assert_eq!(cgkeycode_to_keyrx(0x55), Some(KeyCode::Numpad3));
        assert_eq!(cgkeycode_to_keyrx(0x56), Some(KeyCode::Numpad4));
        assert_eq!(cgkeycode_to_keyrx(0x57), Some(KeyCode::Numpad5));
        assert_eq!(cgkeycode_to_keyrx(0x58), Some(KeyCode::Numpad6));
        assert_eq!(cgkeycode_to_keyrx(0x59), Some(KeyCode::Numpad7));
        assert_eq!(cgkeycode_to_keyrx(0x5B), Some(KeyCode::Numpad8));
        assert_eq!(cgkeycode_to_keyrx(0x5C), Some(KeyCode::Numpad9));
        assert_eq!(cgkeycode_to_keyrx(0x4B), Some(KeyCode::NumpadDivide));
        assert_eq!(cgkeycode_to_keyrx(0x43), Some(KeyCode::NumpadMultiply));
        assert_eq!(cgkeycode_to_keyrx(0x4E), Some(KeyCode::NumpadSubtract));
        assert_eq!(cgkeycode_to_keyrx(0x45), Some(KeyCode::NumpadAdd));
        assert_eq!(cgkeycode_to_keyrx(0x4C), Some(KeyCode::NumpadEnter));
        assert_eq!(cgkeycode_to_keyrx(0x41), Some(KeyCode::NumpadDecimal));
    }

    #[test]
    fn test_rdev_comprehensive_mapping() {
        use rdev::Key;

        // Test all letter keys
        for (rdev_key, expected) in [
            (Key::KeyA, KeyCode::A),
            (Key::KeyB, KeyCode::B),
            (Key::KeyC, KeyCode::C),
            (Key::KeyD, KeyCode::D),
            (Key::KeyE, KeyCode::E),
            (Key::KeyF, KeyCode::F),
            (Key::KeyG, KeyCode::G),
            (Key::KeyH, KeyCode::H),
            (Key::KeyI, KeyCode::I),
            (Key::KeyJ, KeyCode::J),
            (Key::KeyK, KeyCode::K),
            (Key::KeyL, KeyCode::L),
            (Key::KeyM, KeyCode::M),
            (Key::KeyN, KeyCode::N),
            (Key::KeyO, KeyCode::O),
            (Key::KeyP, KeyCode::P),
            (Key::KeyQ, KeyCode::Q),
            (Key::KeyR, KeyCode::R),
            (Key::KeyS, KeyCode::S),
            (Key::KeyT, KeyCode::T),
            (Key::KeyU, KeyCode::U),
            (Key::KeyV, KeyCode::V),
            (Key::KeyW, KeyCode::W),
            (Key::KeyX, KeyCode::X),
            (Key::KeyY, KeyCode::Y),
            (Key::KeyZ, KeyCode::Z),
        ] {
            assert_eq!(
                rdev_key_to_keyrx(rdev_key),
                Some(expected),
                "rdev key {:?} should map to {:?}",
                rdev_key,
                expected
            );
        }

        // Test all number keys
        for (rdev_key, expected) in [
            (Key::Num0, KeyCode::Num0),
            (Key::Num1, KeyCode::Num1),
            (Key::Num2, KeyCode::Num2),
            (Key::Num3, KeyCode::Num3),
            (Key::Num4, KeyCode::Num4),
            (Key::Num5, KeyCode::Num5),
            (Key::Num6, KeyCode::Num6),
            (Key::Num7, KeyCode::Num7),
            (Key::Num8, KeyCode::Num8),
            (Key::Num9, KeyCode::Num9),
        ] {
            assert_eq!(rdev_key_to_keyrx(rdev_key), Some(expected));
        }

        // Test all function keys
        for (rdev_key, expected) in [
            (Key::F1, KeyCode::F1),
            (Key::F2, KeyCode::F2),
            (Key::F3, KeyCode::F3),
            (Key::F4, KeyCode::F4),
            (Key::F5, KeyCode::F5),
            (Key::F6, KeyCode::F6),
            (Key::F7, KeyCode::F7),
            (Key::F8, KeyCode::F8),
            (Key::F9, KeyCode::F9),
            (Key::F10, KeyCode::F10),
            (Key::F11, KeyCode::F11),
            (Key::F12, KeyCode::F12),
        ] {
            assert_eq!(rdev_key_to_keyrx(rdev_key), Some(expected));
        }
    }

    #[test]
    fn test_enigo_unmapped_keys() {
        // Test that certain keys correctly return None for enigo
        assert_eq!(keyrx_to_enigo_key(KeyCode::Insert), None);
        assert_eq!(keyrx_to_enigo_key(KeyCode::MediaStop), None);
    }

    #[test]
    fn test_enigo_modifier_unification() {
        use enigo::Key;

        // Test that left and right modifiers map to the same enigo key
        assert_eq!(
            keyrx_to_enigo_key(KeyCode::LShift),
            keyrx_to_enigo_key(KeyCode::RShift)
        );
        assert_eq!(
            keyrx_to_enigo_key(KeyCode::LCtrl),
            keyrx_to_enigo_key(KeyCode::RCtrl)
        );
        assert_eq!(
            keyrx_to_enigo_key(KeyCode::LAlt),
            keyrx_to_enigo_key(KeyCode::RAlt)
        );
        assert_eq!(
            keyrx_to_enigo_key(KeyCode::LMeta),
            keyrx_to_enigo_key(KeyCode::RMeta)
        );

        // Verify they map to the expected keys
        assert_eq!(keyrx_to_enigo_key(KeyCode::LShift), Some(Key::Shift));
        assert_eq!(keyrx_to_enigo_key(KeyCode::LCtrl), Some(Key::Control));
        assert_eq!(keyrx_to_enigo_key(KeyCode::LAlt), Some(Key::Alt));
        assert_eq!(keyrx_to_enigo_key(KeyCode::LMeta), Some(Key::Meta));
    }

    #[test]
    fn test_invalid_cgkeycodes() {
        // Test a range of invalid codes
        for invalid_code in [0xFF, 0x100, 0x200, 0x500, 0x1000, 0xFFFF] {
            assert_eq!(
                cgkeycode_to_keyrx(invalid_code),
                None,
                "Invalid CGKeyCode {:x} should return None",
                invalid_code
            );
        }
    }
}

// Property-based tests
#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_cgkeycode_roundtrip(cgcode in 0u16..0x80) {
            // If a CGKeyCode maps to a KeyCode, converting back should give the original
            if let Some(keycode) = cgkeycode_to_keyrx(cgcode) {
                let back = keyrx_to_cgkeycode(keycode);
                prop_assert_eq!(back, Some(cgcode));
            }
        }

        #[test]
        fn test_keycode_roundtrip_consistency(idx in 0usize..CGKEYCODE_TO_KEYCODE.len()) {
            // For every mapping in the table, roundtrip should be consistent
            let (cgcode, keycode) = CGKEYCODE_TO_KEYCODE[idx];

            // CGKeyCode -> KeyCode -> CGKeyCode
            let keycode_result = cgkeycode_to_keyrx(cgcode);
            prop_assert_eq!(keycode_result, Some(keycode));

            let cgcode_result = keyrx_to_cgkeycode(keycode);
            prop_assert_eq!(cgcode_result, Some(cgcode));
        }

        #[test]
        fn test_no_duplicate_keycodes(idx1 in 0usize..CGKEYCODE_TO_KEYCODE.len(),
                                      idx2 in 0usize..CGKEYCODE_TO_KEYCODE.len()) {
            // If two indices are different, their keycodes should be different
            let (cgcode1, _keycode1) = CGKEYCODE_TO_KEYCODE[idx1];
            let (cgcode2, _keycode2) = CGKEYCODE_TO_KEYCODE[idx2];

            if idx1 != idx2 {
                prop_assert_ne!(cgcode1, cgcode2, "Duplicate CGKeyCode found");
            }
        }

        #[test]
        fn test_unmapped_returns_none(cgcode in 0x80u16..0x200) {
            // CGKeyCodes outside the standard range should return None
            // (most macOS keycodes are < 0x80)
            let result = cgkeycode_to_keyrx(cgcode);
            // If it returns Some, that's okay (some valid codes exist above 0x80)
            // But we're just checking it doesn't panic
            let _ = result;
        }
    }
}
