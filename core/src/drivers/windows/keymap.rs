//! Windows keycode mappings.
//!
//! This module provides Windows virtual key code conversion functions.
//! The functions are re-exported from the centralized `keycodes` module
//! for SSOT (Single Source of Truth) compliance.

// Re-export VK conversion functions from the centralized keycodes module (SSOT)
pub use crate::drivers::keycodes::{keycode_to_vk, vk_to_keycode};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::KeyCode;

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
