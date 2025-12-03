//! Platform-specific key code constants.
//!
//! This module provides constants for key codes used across the KeyRx engine:
//! - Linux evdev key codes (EVDEV_KEY_*) for modifier and escape keys
//! - Windows virtual key codes (VK_*) for modifier and escape keys
//!
//! These constants are primarily used for emergency exit detection (Ctrl+Alt+Shift+Escape)
//! and modifier key tracking in the input drivers.

// ============================================================================
// Linux evdev Key Codes
// ============================================================================
//
// These constants match the evdev key codes defined in linux/input-event-codes.h.
// They are used for modifier key tracking and emergency exit detection on Linux.

/// Evdev key code for Escape key.
///
/// Used in emergency exit detection (Ctrl+Alt+Shift+Escape).
pub const EVDEV_KEY_ESC: u16 = 1;

/// Evdev key code for Left Control.
///
/// One of the modifier keys used in emergency exit combo detection.
pub const EVDEV_KEY_LEFTCTRL: u16 = 29;

/// Evdev key code for Left Shift.
///
/// One of the modifier keys used in emergency exit combo detection.
pub const EVDEV_KEY_LEFTSHIFT: u16 = 42;

/// Evdev key code for Right Shift.
///
/// One of the modifier keys used in emergency exit combo detection.
pub const EVDEV_KEY_RIGHTSHIFT: u16 = 54;

/// Evdev key code for Left Alt.
///
/// One of the modifier keys used in emergency exit combo detection.
pub const EVDEV_KEY_LEFTALT: u16 = 56;

/// Evdev key code for Right Control.
///
/// One of the modifier keys used in emergency exit combo detection.
pub const EVDEV_KEY_RIGHTCTRL: u16 = 97;

/// Evdev key code for Right Alt (AltGr on some keyboards).
///
/// One of the modifier keys used in emergency exit combo detection.
pub const EVDEV_KEY_RIGHTALT: u16 = 100;

// ============================================================================
// Windows Virtual Key Codes
// ============================================================================
//
// These constants match the Windows virtual key codes defined in winuser.h.
// They are used for modifier key tracking and emergency exit detection on Windows.

/// Windows virtual key code for Escape key.
///
/// Used in emergency exit detection (Ctrl+Alt+Shift+Escape).
pub const VK_ESCAPE: i32 = 0x1B;

/// Windows virtual key code for Shift key (either left or right).
///
/// One of the modifier keys used in emergency exit combo detection.
/// Use `GetAsyncKeyState(VK_SHIFT)` to check if any Shift key is pressed.
pub const VK_SHIFT: i32 = 0x10;

/// Windows virtual key code for Control key (either left or right).
///
/// One of the modifier keys used in emergency exit combo detection.
/// Use `GetAsyncKeyState(VK_CONTROL)` to check if any Ctrl key is pressed.
pub const VK_CONTROL: i32 = 0x11;

/// Windows virtual key code for Alt key (Menu key, either left or right).
///
/// One of the modifier keys used in emergency exit combo detection.
/// Use `GetAsyncKeyState(VK_MENU)` to check if any Alt key is pressed.
pub const VK_MENU: i32 = 0x12;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evdev_key_codes_match_linux_definitions() {
        // These values are defined in linux/input-event-codes.h
        assert_eq!(EVDEV_KEY_ESC, 1);
        assert_eq!(EVDEV_KEY_LEFTCTRL, 29);
        assert_eq!(EVDEV_KEY_LEFTSHIFT, 42);
        assert_eq!(EVDEV_KEY_RIGHTSHIFT, 54);
        assert_eq!(EVDEV_KEY_LEFTALT, 56);
        assert_eq!(EVDEV_KEY_RIGHTCTRL, 97);
        assert_eq!(EVDEV_KEY_RIGHTALT, 100);
    }

    #[test]
    fn windows_vk_codes_match_winuser_definitions() {
        // These values are defined in winuser.h
        assert_eq!(VK_ESCAPE, 0x1B);
        assert_eq!(VK_SHIFT, 0x10);
        assert_eq!(VK_CONTROL, 0x11);
        assert_eq!(VK_MENU, 0x12);
    }

    #[test]
    fn evdev_key_types_are_u16() {
        // Verify evdev keys are u16 for compatibility with evdev crate
        let _esc: u16 = EVDEV_KEY_ESC;
        let _lctrl: u16 = EVDEV_KEY_LEFTCTRL;
        let _lshift: u16 = EVDEV_KEY_LEFTSHIFT;
        let _rshift: u16 = EVDEV_KEY_RIGHTSHIFT;
        let _lalt: u16 = EVDEV_KEY_LEFTALT;
        let _rctrl: u16 = EVDEV_KEY_RIGHTCTRL;
        let _ralt: u16 = EVDEV_KEY_RIGHTALT;
    }

    #[test]
    fn windows_vk_types_are_i32() {
        // Verify VK codes are i32 for compatibility with Windows API
        let _esc: i32 = VK_ESCAPE;
        let _shift: i32 = VK_SHIFT;
        let _ctrl: i32 = VK_CONTROL;
        let _menu: i32 = VK_MENU;
    }
}
