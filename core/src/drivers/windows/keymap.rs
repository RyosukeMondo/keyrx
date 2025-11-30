//! Windows keycode mappings.
//!
//! This module provides Windows virtual key code conversion functions.
//! These functions use the engine's KeyCode type directly.

use crate::engine::KeyCode;

/// Convert Windows virtual key code to KeyRx KeyCode.
pub fn vk_to_keycode(vk: u16) -> KeyCode {
    // Windows virtual key codes (from WinUser.h / VK_* constants)
    match vk {
        // Letters A-Z (0x41-0x5A)
        0x41 => KeyCode::A,
        0x42 => KeyCode::B,
        0x43 => KeyCode::C,
        0x44 => KeyCode::D,
        0x45 => KeyCode::E,
        0x46 => KeyCode::F,
        0x47 => KeyCode::G,
        0x48 => KeyCode::H,
        0x49 => KeyCode::I,
        0x4A => KeyCode::J,
        0x4B => KeyCode::K,
        0x4C => KeyCode::L,
        0x4D => KeyCode::M,
        0x4E => KeyCode::N,
        0x4F => KeyCode::O,
        0x50 => KeyCode::P,
        0x51 => KeyCode::Q,
        0x52 => KeyCode::R,
        0x53 => KeyCode::S,
        0x54 => KeyCode::T,
        0x55 => KeyCode::U,
        0x56 => KeyCode::V,
        0x57 => KeyCode::W,
        0x58 => KeyCode::X,
        0x59 => KeyCode::Y,
        0x5A => KeyCode::Z,
        // Numbers 0-9 (0x30-0x39)
        0x30 => KeyCode::Key0,
        0x31 => KeyCode::Key1,
        0x32 => KeyCode::Key2,
        0x33 => KeyCode::Key3,
        0x34 => KeyCode::Key4,
        0x35 => KeyCode::Key5,
        0x36 => KeyCode::Key6,
        0x37 => KeyCode::Key7,
        0x38 => KeyCode::Key8,
        0x39 => KeyCode::Key9,
        // Function keys F1-F12 (0x70-0x7B)
        0x70 => KeyCode::F1,
        0x71 => KeyCode::F2,
        0x72 => KeyCode::F3,
        0x73 => KeyCode::F4,
        0x74 => KeyCode::F5,
        0x75 => KeyCode::F6,
        0x76 => KeyCode::F7,
        0x77 => KeyCode::F8,
        0x78 => KeyCode::F9,
        0x79 => KeyCode::F10,
        0x7A => KeyCode::F11,
        0x7B => KeyCode::F12,
        // Modifier keys
        0x10 => KeyCode::LeftShift,  // VK_SHIFT (use left by default)
        0xA0 => KeyCode::LeftShift,  // VK_LSHIFT
        0xA1 => KeyCode::RightShift, // VK_RSHIFT
        0x11 => KeyCode::LeftCtrl,   // VK_CONTROL (use left by default)
        0xA2 => KeyCode::LeftCtrl,   // VK_LCONTROL
        0xA3 => KeyCode::RightCtrl,  // VK_RCONTROL
        0x12 => KeyCode::LeftAlt,    // VK_MENU (use left by default)
        0xA4 => KeyCode::LeftAlt,    // VK_LMENU
        0xA5 => KeyCode::RightAlt,   // VK_RMENU
        0x5B => KeyCode::LeftMeta,   // VK_LWIN
        0x5C => KeyCode::RightMeta,  // VK_RWIN
        // Navigation keys
        0x26 => KeyCode::Up,       // VK_UP
        0x28 => KeyCode::Down,     // VK_DOWN
        0x25 => KeyCode::Left,     // VK_LEFT
        0x27 => KeyCode::Right,    // VK_RIGHT
        0x24 => KeyCode::Home,     // VK_HOME
        0x23 => KeyCode::End,      // VK_END
        0x21 => KeyCode::PageUp,   // VK_PRIOR
        0x22 => KeyCode::PageDown, // VK_NEXT
        // Editing keys
        0x2D => KeyCode::Insert,    // VK_INSERT
        0x2E => KeyCode::Delete,    // VK_DELETE
        0x08 => KeyCode::Backspace, // VK_BACK
        // Whitespace
        0x20 => KeyCode::Space, // VK_SPACE
        0x09 => KeyCode::Tab,   // VK_TAB
        0x0D => KeyCode::Enter, // VK_RETURN
        // Lock keys
        0x14 => KeyCode::CapsLock,   // VK_CAPITAL
        0x90 => KeyCode::NumLock,    // VK_NUMLOCK
        0x91 => KeyCode::ScrollLock, // VK_SCROLL
        // Escape area
        0x1B => KeyCode::Escape,      // VK_ESCAPE
        0x2C => KeyCode::PrintScreen, // VK_SNAPSHOT
        0x13 => KeyCode::Pause,       // VK_PAUSE
        // Punctuation and symbols
        0xC0 => KeyCode::Grave,        // VK_OEM_3 (` ~)
        0xBD => KeyCode::Minus,        // VK_OEM_MINUS (- _)
        0xBB => KeyCode::Equal,        // VK_OEM_PLUS (= +)
        0xDB => KeyCode::LeftBracket,  // VK_OEM_4 ([ {)
        0xDD => KeyCode::RightBracket, // VK_OEM_6 (] })
        0xDC => KeyCode::Backslash,    // VK_OEM_5 (\ |)
        0xBA => KeyCode::Semicolon,    // VK_OEM_1 (; :)
        0xDE => KeyCode::Apostrophe,   // VK_OEM_7 (' ")
        0xBC => KeyCode::Comma,        // VK_OEM_COMMA (, <)
        0xBE => KeyCode::Period,       // VK_OEM_PERIOD (. >)
        0xBF => KeyCode::Slash,        // VK_OEM_2 (/ ?)
        // Numpad keys
        0x60 => KeyCode::Numpad0,        // VK_NUMPAD0
        0x61 => KeyCode::Numpad1,        // VK_NUMPAD1
        0x62 => KeyCode::Numpad2,        // VK_NUMPAD2
        0x63 => KeyCode::Numpad3,        // VK_NUMPAD3
        0x64 => KeyCode::Numpad4,        // VK_NUMPAD4
        0x65 => KeyCode::Numpad5,        // VK_NUMPAD5
        0x66 => KeyCode::Numpad6,        // VK_NUMPAD6
        0x67 => KeyCode::Numpad7,        // VK_NUMPAD7
        0x68 => KeyCode::Numpad8,        // VK_NUMPAD8
        0x69 => KeyCode::Numpad9,        // VK_NUMPAD9
        0x6B => KeyCode::NumpadAdd,      // VK_ADD
        0x6D => KeyCode::NumpadSubtract, // VK_SUBTRACT
        0x6A => KeyCode::NumpadMultiply, // VK_MULTIPLY
        0x6F => KeyCode::NumpadDivide,   // VK_DIVIDE
        0x6E => KeyCode::NumpadDecimal,  // VK_DECIMAL
        // Note: NumpadEnter shares VK_RETURN (0x0D) but has extended flag
        // Media keys
        0xAF => KeyCode::VolumeUp,       // VK_VOLUME_UP
        0xAE => KeyCode::VolumeDown,     // VK_VOLUME_DOWN
        0xAD => KeyCode::VolumeMute,     // VK_VOLUME_MUTE
        0xB3 => KeyCode::MediaPlayPause, // VK_MEDIA_PLAY_PAUSE
        0xB2 => KeyCode::MediaStop,      // VK_MEDIA_STOP
        0xB0 => KeyCode::MediaNext,      // VK_MEDIA_NEXT_TRACK
        0xB1 => KeyCode::MediaPrev,      // VK_MEDIA_PREV_TRACK
        // Unknown
        _ => KeyCode::Unknown(vk),
    }
}

/// Convert KeyRx KeyCode to Windows virtual key code.
pub fn keycode_to_vk(key: KeyCode) -> u16 {
    match key {
        // Letters A-Z
        KeyCode::A => 0x41,
        KeyCode::B => 0x42,
        KeyCode::C => 0x43,
        KeyCode::D => 0x44,
        KeyCode::E => 0x45,
        KeyCode::F => 0x46,
        KeyCode::G => 0x47,
        KeyCode::H => 0x48,
        KeyCode::I => 0x49,
        KeyCode::J => 0x4A,
        KeyCode::K => 0x4B,
        KeyCode::L => 0x4C,
        KeyCode::M => 0x4D,
        KeyCode::N => 0x4E,
        KeyCode::O => 0x4F,
        KeyCode::P => 0x50,
        KeyCode::Q => 0x51,
        KeyCode::R => 0x52,
        KeyCode::S => 0x53,
        KeyCode::T => 0x54,
        KeyCode::U => 0x55,
        KeyCode::V => 0x56,
        KeyCode::W => 0x57,
        KeyCode::X => 0x58,
        KeyCode::Y => 0x59,
        KeyCode::Z => 0x5A,
        // Numbers 0-9
        KeyCode::Key0 => 0x30,
        KeyCode::Key1 => 0x31,
        KeyCode::Key2 => 0x32,
        KeyCode::Key3 => 0x33,
        KeyCode::Key4 => 0x34,
        KeyCode::Key5 => 0x35,
        KeyCode::Key6 => 0x36,
        KeyCode::Key7 => 0x37,
        KeyCode::Key8 => 0x38,
        KeyCode::Key9 => 0x39,
        // Function keys F1-F12
        KeyCode::F1 => 0x70,
        KeyCode::F2 => 0x71,
        KeyCode::F3 => 0x72,
        KeyCode::F4 => 0x73,
        KeyCode::F5 => 0x74,
        KeyCode::F6 => 0x75,
        KeyCode::F7 => 0x76,
        KeyCode::F8 => 0x77,
        KeyCode::F9 => 0x78,
        KeyCode::F10 => 0x79,
        KeyCode::F11 => 0x7A,
        KeyCode::F12 => 0x7B,
        // Modifier keys
        KeyCode::LeftShift => 0xA0,
        KeyCode::RightShift => 0xA1,
        KeyCode::LeftCtrl => 0xA2,
        KeyCode::RightCtrl => 0xA3,
        KeyCode::LeftAlt => 0xA4,
        KeyCode::RightAlt => 0xA5,
        KeyCode::LeftMeta => 0x5B,
        KeyCode::RightMeta => 0x5C,
        // Navigation
        KeyCode::Up => 0x26,
        KeyCode::Down => 0x28,
        KeyCode::Left => 0x25,
        KeyCode::Right => 0x27,
        KeyCode::Home => 0x24,
        KeyCode::End => 0x23,
        KeyCode::PageUp => 0x21,
        KeyCode::PageDown => 0x22,
        // Editing
        KeyCode::Insert => 0x2D,
        KeyCode::Delete => 0x2E,
        KeyCode::Backspace => 0x08,
        // Whitespace
        KeyCode::Space => 0x20,
        KeyCode::Tab => 0x09,
        KeyCode::Enter => 0x0D,
        // Lock keys
        KeyCode::CapsLock => 0x14,
        KeyCode::NumLock => 0x90,
        KeyCode::ScrollLock => 0x91,
        // Escape area
        KeyCode::Escape => 0x1B,
        KeyCode::PrintScreen => 0x2C,
        KeyCode::Pause => 0x13,
        // Punctuation
        KeyCode::Grave => 0xC0,
        KeyCode::Minus => 0xBD,
        KeyCode::Equal => 0xBB,
        KeyCode::LeftBracket => 0xDB,
        KeyCode::RightBracket => 0xDD,
        KeyCode::Backslash => 0xDC,
        KeyCode::Semicolon => 0xBA,
        KeyCode::Apostrophe => 0xDE,
        KeyCode::Comma => 0xBC,
        KeyCode::Period => 0xBE,
        KeyCode::Slash => 0xBF,
        // Numpad
        KeyCode::Numpad0 => 0x60,
        KeyCode::Numpad1 => 0x61,
        KeyCode::Numpad2 => 0x62,
        KeyCode::Numpad3 => 0x63,
        KeyCode::Numpad4 => 0x64,
        KeyCode::Numpad5 => 0x65,
        KeyCode::Numpad6 => 0x66,
        KeyCode::Numpad7 => 0x67,
        KeyCode::Numpad8 => 0x68,
        KeyCode::Numpad9 => 0x69,
        KeyCode::NumpadAdd => 0x6B,
        KeyCode::NumpadSubtract => 0x6D,
        KeyCode::NumpadMultiply => 0x6A,
        KeyCode::NumpadDivide => 0x6F,
        KeyCode::NumpadEnter => 0x0D, // Same as Enter, distinguished by extended flag
        KeyCode::NumpadDecimal => 0x6E,
        // Media
        KeyCode::VolumeUp => 0xAF,
        KeyCode::VolumeDown => 0xAE,
        KeyCode::VolumeMute => 0xAD,
        KeyCode::MediaPlayPause => 0xB3,
        KeyCode::MediaStop => 0xB2,
        KeyCode::MediaNext => 0xB0,
        KeyCode::MediaPrev => 0xB1,
        // Unknown - return the stored code
        KeyCode::Unknown(vk) => vk,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vk_keycode_mapping_letters() {
        assert_eq!(vk_to_keycode(0x41), KeyCode::A);
        assert_eq!(vk_to_keycode(0x5A), KeyCode::Z);
    }

    #[test]
    fn vk_keycode_mapping_numbers() {
        assert_eq!(vk_to_keycode(0x30), KeyCode::Key0);
        assert_eq!(vk_to_keycode(0x39), KeyCode::Key9);
    }

    #[test]
    fn vk_keycode_mapping_function_keys() {
        assert_eq!(vk_to_keycode(0x70), KeyCode::F1);
        assert_eq!(vk_to_keycode(0x7B), KeyCode::F12);
    }

    #[test]
    fn vk_keycode_mapping_modifiers() {
        assert_eq!(vk_to_keycode(0xA0), KeyCode::LeftShift);
        assert_eq!(vk_to_keycode(0xA1), KeyCode::RightShift);
        assert_eq!(vk_to_keycode(0xA2), KeyCode::LeftCtrl);
        assert_eq!(vk_to_keycode(0xA3), KeyCode::RightCtrl);
        assert_eq!(vk_to_keycode(0xA4), KeyCode::LeftAlt);
        assert_eq!(vk_to_keycode(0xA5), KeyCode::RightAlt);
    }

    #[test]
    fn vk_keycode_mapping_special() {
        assert_eq!(vk_to_keycode(0x1B), KeyCode::Escape);
        assert_eq!(vk_to_keycode(0x14), KeyCode::CapsLock);
        assert_eq!(vk_to_keycode(0x20), KeyCode::Space);
        assert_eq!(vk_to_keycode(0x0D), KeyCode::Enter);
        assert_eq!(vk_to_keycode(0x09), KeyCode::Tab);
    }

    #[test]
    fn vk_keycode_mapping_unknown() {
        assert_eq!(vk_to_keycode(0xFF), KeyCode::Unknown(0xFF));
    }

    #[test]
    fn keycode_to_vk_roundtrip() {
        let keys = vec![
            KeyCode::A,
            KeyCode::Z,
            KeyCode::Key0,
            KeyCode::F1,
            KeyCode::Escape,
            KeyCode::CapsLock,
            KeyCode::Space,
        ];
        for key in keys {
            let vk = keycode_to_vk(key);
            let back = vk_to_keycode(vk);
            assert_eq!(key, back, "Roundtrip failed for {:?}", key);
        }
    }

    #[test]
    fn vk_keycode_comprehensive_roundtrip() {
        // Test roundtrip conversion: VK -> KeyCode -> VK
        // All known VK codes should roundtrip correctly
        let test_vks: Vec<u16> = vec![
            // Letters A-Z (0x41-0x5A)
            0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E,
            0x4F, 0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A,
            // Numbers 0-9 (0x30-0x39)
            0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39,
            // Function keys F1-F12 (0x70-0x7B)
            0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x7B,
            // Modifier keys (specific left/right variants)
            0xA0, 0xA1, // L/R Shift
            0xA2, 0xA3, // L/R Ctrl
            0xA4, 0xA5, // L/R Alt
            0x5B, 0x5C, // L/R Win
            // Navigation
            0x26, 0x28, 0x25, 0x27, // Arrows
            0x24, 0x23, 0x21, 0x22, // Home/End/PgUp/PgDn
            // Editing
            0x2D, 0x2E, 0x08, // Ins/Del/Backspace
            // Whitespace
            0x20, 0x09, 0x0D, // Space/Tab/Enter
            // Locks
            0x14, 0x90, 0x91, // Caps/Num/Scroll
            // Escape area
            0x1B, 0x2C, 0x13, // Esc/PrtSc/Pause
            // Punctuation
            0xC0, 0xBD, 0xBB, 0xDB, 0xDD, 0xDC, 0xBA, 0xDE, 0xBC, 0xBE, 0xBF, // Numpad
            0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6B, 0x6D, 0x6A, 0x6F,
            0x6E, // Media
            0xAF, 0xAE, 0xAD, 0xB3, 0xB2, 0xB0, 0xB1,
        ];

        for vk in test_vks {
            let keycode = vk_to_keycode(vk);
            let back_to_vk = keycode_to_vk(keycode);
            assert_eq!(
                vk, back_to_vk,
                "Roundtrip failed for VK {:#x}: got KeyCode::{:?} -> {:#x}",
                vk, keycode, back_to_vk
            );
        }
    }

    #[test]
    fn keycode_to_vk_comprehensive_roundtrip() {
        // Test roundtrip conversion: KeyCode -> VK -> KeyCode
        // (for non-Unknown variants)
        let test_keycodes = vec![
            // All letters
            KeyCode::A,
            KeyCode::B,
            KeyCode::C,
            KeyCode::D,
            KeyCode::E,
            KeyCode::F,
            KeyCode::G,
            KeyCode::H,
            KeyCode::I,
            KeyCode::J,
            KeyCode::K,
            KeyCode::L,
            KeyCode::M,
            KeyCode::N,
            KeyCode::O,
            KeyCode::P,
            KeyCode::Q,
            KeyCode::R,
            KeyCode::S,
            KeyCode::T,
            KeyCode::U,
            KeyCode::V,
            KeyCode::W,
            KeyCode::X,
            KeyCode::Y,
            KeyCode::Z,
            // All numbers
            KeyCode::Key0,
            KeyCode::Key1,
            KeyCode::Key2,
            KeyCode::Key3,
            KeyCode::Key4,
            KeyCode::Key5,
            KeyCode::Key6,
            KeyCode::Key7,
            KeyCode::Key8,
            KeyCode::Key9,
            // All function keys
            KeyCode::F1,
            KeyCode::F2,
            KeyCode::F3,
            KeyCode::F4,
            KeyCode::F5,
            KeyCode::F6,
            KeyCode::F7,
            KeyCode::F8,
            KeyCode::F9,
            KeyCode::F10,
            KeyCode::F11,
            KeyCode::F12,
            // Modifiers
            KeyCode::LeftShift,
            KeyCode::RightShift,
            KeyCode::LeftCtrl,
            KeyCode::RightCtrl,
            KeyCode::LeftAlt,
            KeyCode::RightAlt,
            KeyCode::LeftMeta,
            KeyCode::RightMeta,
            // Navigation
            KeyCode::Up,
            KeyCode::Down,
            KeyCode::Left,
            KeyCode::Right,
            KeyCode::Home,
            KeyCode::End,
            KeyCode::PageUp,
            KeyCode::PageDown,
            // Editing
            KeyCode::Insert,
            KeyCode::Delete,
            KeyCode::Backspace,
            // Whitespace
            KeyCode::Space,
            KeyCode::Tab,
            KeyCode::Enter,
            // Locks
            KeyCode::CapsLock,
            KeyCode::NumLock,
            KeyCode::ScrollLock,
            // Escape area
            KeyCode::Escape,
            KeyCode::PrintScreen,
            KeyCode::Pause,
            // Punctuation
            KeyCode::Grave,
            KeyCode::Minus,
            KeyCode::Equal,
            KeyCode::LeftBracket,
            KeyCode::RightBracket,
            KeyCode::Backslash,
            KeyCode::Semicolon,
            KeyCode::Apostrophe,
            KeyCode::Comma,
            KeyCode::Period,
            KeyCode::Slash,
            // Numpad
            KeyCode::Numpad0,
            KeyCode::Numpad1,
            KeyCode::Numpad2,
            KeyCode::Numpad3,
            KeyCode::Numpad4,
            KeyCode::Numpad5,
            KeyCode::Numpad6,
            KeyCode::Numpad7,
            KeyCode::Numpad8,
            KeyCode::Numpad9,
            KeyCode::NumpadAdd,
            KeyCode::NumpadSubtract,
            KeyCode::NumpadMultiply,
            KeyCode::NumpadDivide,
            KeyCode::NumpadDecimal,
            // Media
            KeyCode::VolumeUp,
            KeyCode::VolumeDown,
            KeyCode::VolumeMute,
            KeyCode::MediaPlayPause,
            KeyCode::MediaStop,
            KeyCode::MediaNext,
            KeyCode::MediaPrev,
        ];

        for keycode in test_keycodes {
            let vk = keycode_to_vk(keycode);
            let back_to_keycode = vk_to_keycode(vk);
            assert_eq!(
                keycode, back_to_keycode,
                "Roundtrip failed for {:?}: VK {:#x} -> {:?}",
                keycode, vk, back_to_keycode
            );
        }
    }

    #[test]
    fn unknown_keycode_vk_passthrough() {
        // Unknown keycodes should pass through unchanged
        let unknown = KeyCode::Unknown(0x1234);
        let vk = keycode_to_vk(unknown);
        assert_eq!(vk, 0x1234);

        // And when converting back, unknown VKs stay unknown
        let unknown_vk = 0xFE_u16;
        let keycode = vk_to_keycode(unknown_vk);
        assert_eq!(keycode, KeyCode::Unknown(0xFE));
    }
}
