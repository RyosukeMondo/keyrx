//! Linux keycode mappings (SSOT re-exports).
//!
//! This module delegates evdev conversions to the centralized keycodes module
//! to ensure a single source of truth for key definitions and mappings.

#[cfg(test)]
use crate::engine::KeyCode;

// Re-export conversion helpers from the centralized keycodes module (SSOT)
pub use crate::drivers::keycodes::{evdev_to_keycode, keycode_to_evdev};

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
        assert_eq!(keycode_to_evdev(KeyCode::Escape), 1);
        assert_eq!(keycode_to_evdev(KeyCode::A), 30);
        assert_eq!(keycode_to_evdev(KeyCode::CapsLock), 58);
        assert_eq!(keycode_to_evdev(KeyCode::Space), 57);
        assert_eq!(keycode_to_evdev(KeyCode::Enter), 28);
        assert_eq!(keycode_to_evdev(KeyCode::Unknown(9999)), 9999);
    }

    #[test]
    fn keycode_evdev_roundtrip() {
        let test_codes: Vec<u16> = vec![
            30, 48, 46, 32, 18, 33, 34, 35, 23, 36, 37, 38, 50, 49, 24, 25, 16, 19, 31, 20, 22, 47,
            17, 45, 21, 44, 11, 2, 3, 4, 5, 6, 7, 8, 9, 10, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68,
            87, 88, 42, 54, 29, 97, 56, 100, 125, 126, 103, 108, 105, 106, 102, 107, 104, 109, 110,
            111, 14, 57, 15, 28, 58, 69, 70, 1, 99, 119, 41, 12, 13, 26, 27, 43, 39, 40, 51, 52,
            53, 82, 79, 80, 81, 75, 76, 77, 71, 72, 73, 78, 74, 55, 98, 96, 83, 115, 114, 113, 164,
            166, 163, 165,
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
            KeyCode::LeftShift,
            KeyCode::RightAlt,
            KeyCode::LeftMeta,
            KeyCode::Up,
            KeyCode::PageDown,
            KeyCode::Insert,
            KeyCode::Delete,
            KeyCode::PrintScreen,
            KeyCode::Numpad7,
            KeyCode::NumpadDecimal,
            KeyCode::NumpadEnter,
            KeyCode::VolumeMute,
            KeyCode::MediaPlayPause,
        ];

        for keycode in test_keycodes {
            let evdev = keycode_to_evdev(keycode);
            let back = evdev_to_keycode(evdev);
            assert_eq!(
                keycode, back,
                "Roundtrip failed for KeyCode::{:?} -> {} -> {:?}",
                keycode, evdev, back
            );
        }
    }
}
