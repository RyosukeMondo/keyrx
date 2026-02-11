use keyrx_core::config::KeyCode;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;

/// Bidirectional mapping between Windows Virtual Key codes and KeyRx KeyCode enum.
/// Uses const arrays for O(1) lookup performance.
// Mapping from VK to KeyCode
const VK_TO_KEYCODE: [(u16, KeyCode); 141] = [
    (VK_A, KeyCode::A),
    (VK_B, KeyCode::B),
    (VK_C, KeyCode::C),
    (VK_D, KeyCode::D),
    (VK_E, KeyCode::E),
    (VK_F, KeyCode::F),
    (VK_G, KeyCode::G),
    (VK_H, KeyCode::H),
    (VK_I, KeyCode::I),
    (VK_J, KeyCode::J),
    (VK_K, KeyCode::K),
    (VK_L, KeyCode::L),
    (VK_M, KeyCode::M),
    (VK_N, KeyCode::N),
    (VK_O, KeyCode::O),
    (VK_P, KeyCode::P),
    (VK_Q, KeyCode::Q),
    (VK_R, KeyCode::R),
    (VK_S, KeyCode::S),
    (VK_T, KeyCode::T),
    (VK_U, KeyCode::U),
    (VK_V, KeyCode::V),
    (VK_W, KeyCode::W),
    (VK_X, KeyCode::X),
    (VK_Y, KeyCode::Y),
    (VK_Z, KeyCode::Z),
    (VK_0, KeyCode::Num0),
    (VK_1, KeyCode::Num1),
    (VK_2, KeyCode::Num2),
    (VK_3, KeyCode::Num3),
    (VK_4, KeyCode::Num4),
    (VK_5, KeyCode::Num5),
    (VK_6, KeyCode::Num6),
    (VK_7, KeyCode::Num7),
    (VK_8, KeyCode::Num8),
    (VK_9, KeyCode::Num9),
    (VK_F1, KeyCode::F1),
    (VK_F2, KeyCode::F2),
    (VK_F3, KeyCode::F3),
    (VK_F4, KeyCode::F4),
    (VK_F5, KeyCode::F5),
    (VK_F6, KeyCode::F6),
    (VK_F7, KeyCode::F7),
    (VK_F8, KeyCode::F8),
    (VK_F9, KeyCode::F9),
    (VK_F10, KeyCode::F10),
    (VK_F11, KeyCode::F11),
    (VK_F12, KeyCode::F12),
    (VK_F13, KeyCode::F13),
    (VK_F14, KeyCode::F14),
    (VK_F15, KeyCode::F15),
    (VK_F16, KeyCode::F16),
    (VK_F17, KeyCode::F17),
    (VK_F18, KeyCode::F18),
    (VK_F19, KeyCode::F19),
    (VK_F20, KeyCode::F20),
    (VK_F21, KeyCode::F21),
    (VK_F22, KeyCode::F22),
    (VK_F23, KeyCode::F23),
    (VK_F24, KeyCode::F24),
    (VK_LSHIFT, KeyCode::LShift),
    (VK_RSHIFT, KeyCode::RShift),
    (VK_LCONTROL, KeyCode::LCtrl),
    (VK_RCONTROL, KeyCode::RCtrl),
    (VK_LMENU, KeyCode::LAlt),
    (VK_RMENU, KeyCode::RAlt),
    (VK_LWIN, KeyCode::LMeta),
    (VK_RWIN, KeyCode::RMeta),
    (VK_ESCAPE, KeyCode::Escape),
    (VK_RETURN, KeyCode::Enter),
    (VK_BACK, KeyCode::Backspace),
    (VK_TAB, KeyCode::Tab),
    (VK_SPACE, KeyCode::Space),
    (VK_CAPITAL, KeyCode::CapsLock),
    (VK_NUMLOCK, KeyCode::NumLock),
    (VK_SCROLL, KeyCode::ScrollLock),
    (VK_SNAPSHOT, KeyCode::PrintScreen),
    (VK_PAUSE, KeyCode::Pause),
    (VK_INSERT, KeyCode::Insert),
    (VK_DELETE, KeyCode::Delete),
    (VK_HOME, KeyCode::Home),
    (VK_END, KeyCode::End),
    (VK_PRIOR, KeyCode::PageUp),
    (VK_NEXT, KeyCode::PageDown),
    (VK_LEFT, KeyCode::Left),
    (VK_RIGHT, KeyCode::Right),
    (VK_UP, KeyCode::Up),
    (VK_DOWN, KeyCode::Down),
    (VK_OEM_4, KeyCode::LeftBracket),
    (VK_OEM_6, KeyCode::RightBracket),
    (VK_OEM_5, KeyCode::Backslash),
    (VK_OEM_1, KeyCode::Semicolon),
    (VK_OEM_7, KeyCode::Quote),
    (VK_OEM_COMMA, KeyCode::Comma),
    (VK_OEM_PERIOD, KeyCode::Period),
    (VK_OEM_2, KeyCode::Slash),
    (VK_OEM_3, KeyCode::Grave),
    (VK_OEM_MINUS, KeyCode::Minus),
    (VK_OEM_PLUS, KeyCode::Equal),
    (VK_NUMPAD0, KeyCode::Numpad0),
    (VK_NUMPAD1, KeyCode::Numpad1),
    (VK_NUMPAD2, KeyCode::Numpad2),
    (VK_NUMPAD3, KeyCode::Numpad3),
    (VK_NUMPAD4, KeyCode::Numpad4),
    (VK_NUMPAD5, KeyCode::Numpad5),
    (VK_NUMPAD6, KeyCode::Numpad6),
    (VK_NUMPAD7, KeyCode::Numpad7),
    (VK_NUMPAD8, KeyCode::Numpad8),
    (VK_NUMPAD9, KeyCode::Numpad9),
    (VK_DIVIDE, KeyCode::NumpadDivide),
    (VK_MULTIPLY, KeyCode::NumpadMultiply),
    (VK_SUBTRACT, KeyCode::NumpadSubtract),
    (VK_ADD, KeyCode::NumpadAdd),
    (VK_DECIMAL, KeyCode::NumpadDecimal),
    (VK_VOLUME_MUTE, KeyCode::Mute),
    (VK_VOLUME_DOWN, KeyCode::VolumeDown),
    (VK_VOLUME_UP, KeyCode::VolumeUp),
    (VK_MEDIA_PLAY_PAUSE, KeyCode::MediaPlayPause),
    (VK_MEDIA_STOP, KeyCode::MediaStop),
    (VK_MEDIA_PREV_TRACK, KeyCode::MediaPrevious),
    (VK_MEDIA_NEXT_TRACK, KeyCode::MediaNext),
    (VK_SLEEP, KeyCode::Sleep),
    (VK_BROWSER_BACK, KeyCode::BrowserBack),
    (VK_BROWSER_FORWARD, KeyCode::BrowserForward),
    (VK_BROWSER_REFRESH, KeyCode::BrowserRefresh),
    (VK_BROWSER_STOP, KeyCode::BrowserStop),
    (VK_BROWSER_SEARCH, KeyCode::BrowserSearch),
    (VK_BROWSER_FAVORITES, KeyCode::BrowserFavorites),
    (VK_BROWSER_HOME, KeyCode::BrowserHome),
    (VK_LAUNCH_MAIL, KeyCode::AppMail),
    (VK_LAUNCH_APP2, KeyCode::AppCalculator),
    (VK_LAUNCH_APP1, KeyCode::AppMyComputer),
    (VK_APPS, KeyCode::Menu),
    (VK_HELP, KeyCode::Help),
    (VK_SELECT, KeyCode::Select),
    (VK_EXECUTE, KeyCode::Execute),
    (VK_KANJI, KeyCode::Zenkaku), // Best effort mapping
    (VK_KANA, KeyCode::KatakanaHiragana),
    (VK_CONVERT, KeyCode::Henkan),
    (VK_NONCONVERT, KeyCode::Muhenkan),
    (VK_OEM_102, KeyCode::Iso102nd),
];

pub fn vk_to_keycode(vk: u16) -> Option<KeyCode> {
    for (v, k) in VK_TO_KEYCODE.iter() {
        if *v == vk {
            return Some(*k);
        }
    }
    None
}

pub fn scancode_to_keycode(scancode: u32) -> Option<KeyCode> {
    // WIN-BUG #7: Layout-dependent mapping.
    // Use a fixed table for common scancodes to ensure consistency across layouts.
    // Only fall back to MapVirtualKeyW for less common keys.
    match scancode {
        0x1E => Some(KeyCode::A),
        0x30 => Some(KeyCode::B),
        0x2E => Some(KeyCode::C),
        0x20 => Some(KeyCode::D),
        0x12 => Some(KeyCode::E),
        0x21 => Some(KeyCode::F),
        0x22 => Some(KeyCode::G),
        0x23 => Some(KeyCode::H),
        0x17 => Some(KeyCode::I),
        0x24 => Some(KeyCode::J),
        0x25 => Some(KeyCode::K),
        0x26 => Some(KeyCode::L),
        0x32 => Some(KeyCode::M),
        0x31 => Some(KeyCode::N),
        0x18 => Some(KeyCode::O),
        0x19 => Some(KeyCode::P),
        0x10 => Some(KeyCode::Q),
        0x13 => Some(KeyCode::R),
        0x1F => Some(KeyCode::S),
        0x14 => Some(KeyCode::T),
        0x16 => Some(KeyCode::U),
        0x2F => Some(KeyCode::V),
        0x11 => Some(KeyCode::W),
        0x2D => Some(KeyCode::X),
        0x15 => Some(KeyCode::Y),
        0x2C => Some(KeyCode::Z),
        // Numbers
        0x0B => Some(KeyCode::Num0),
        0x02 => Some(KeyCode::Num1),
        0x03 => Some(KeyCode::Num2),
        0x04 => Some(KeyCode::Num3),
        0x05 => Some(KeyCode::Num4),
        0x06 => Some(KeyCode::Num5),
        0x07 => Some(KeyCode::Num6),
        0x08 => Some(KeyCode::Num7),
        0x09 => Some(KeyCode::Num8),
        0x0A => Some(KeyCode::Num9),
        // Control keys
        0x01 => Some(KeyCode::Escape),
        0x1C => Some(KeyCode::Enter),
        0x39 => Some(KeyCode::Space),
        0x0E => Some(KeyCode::Backspace),
        0x0F => Some(KeyCode::Tab),
        0x2A => Some(KeyCode::LShift),
        0x36 => Some(KeyCode::RShift),
        0x1D => Some(KeyCode::LCtrl),
        0x38 => Some(KeyCode::LAlt),
        // Navigation (E0 prefix handling in rawinput.rs makes these scan codes)
        0xE01D => Some(KeyCode::RCtrl),
        0xE038 => Some(KeyCode::RAlt),
        0xE047 => Some(KeyCode::Home),
        0xE048 => Some(KeyCode::Up),
        0xE049 => Some(KeyCode::PageUp),
        0xE04B => Some(KeyCode::Left),
        0xE04D => Some(KeyCode::Right),
        0xE04F => Some(KeyCode::End),
        0xE050 => Some(KeyCode::Down),
        0xE051 => Some(KeyCode::PageDown),
        0xE052 => Some(KeyCode::Insert),
        0xE053 => Some(KeyCode::Delete),
        0xE01C => Some(KeyCode::Enter),
        0xE035 => Some(KeyCode::NumpadDivide),
        // Function keys
        0x3B => Some(KeyCode::F1),
        0x3C => Some(KeyCode::F2),
        0x3D => Some(KeyCode::F3),
        0x3E => Some(KeyCode::F4),
        0x3F => Some(KeyCode::F5),
        0x40 => Some(KeyCode::F6),
        0x41 => Some(KeyCode::F7),
        0x42 => Some(KeyCode::F8),
        0x43 => Some(KeyCode::F9),
        0x44 => Some(KeyCode::F10),
        0x57 => Some(KeyCode::F11),
        0x58 => Some(KeyCode::F12),

        _ => {
            // Fallback to MapVirtualKeyW for other keys
            use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
                MapVirtualKeyW, MAPVK_VSC_TO_VK_EX,
            };
            unsafe {
                let vk = MapVirtualKeyW(scancode, MAPVK_VSC_TO_VK_EX);
                if vk == 0 {
                    return None;
                }
                vk_to_keycode(vk as u16)
            }
        }
    }
}

#[allow(dead_code)]
pub fn keycode_to_vk(keycode: KeyCode) -> Option<u16> {
    for (v, k) in VK_TO_KEYCODE.iter() {
        if *k == keycode {
            return Some(*v);
        }
    }
    None
}

/// Convert KeyCode to Windows scan code for key blocking
///
/// Uses MapVirtualKeyW to get the hardware scan code from virtual key code.
/// Returns the scan code with extended flag if applicable (e.g., 0xE01D for Right Ctrl).
pub fn keycode_to_scancode(keycode: KeyCode) -> Option<u32> {
    let vk = keycode_to_vk(keycode)?;

    // MapVirtualKeyW with MAPVK_VK_TO_VSC (0) returns scan code
    let scan_code =
        unsafe { windows_sys::Win32::UI::Input::KeyboardAndMouse::MapVirtualKeyW(vk as u32, 0) };

    if scan_code == 0 {
        return None;
    }

    // Check if this is an extended key (arrows, insert, delete, home, end, page up/down, etc.)
    // Extended keys need the 0xE000 prefix
    let is_extended = matches!(
        keycode,
        KeyCode::Insert
            | KeyCode::Delete
            | KeyCode::Home
            | KeyCode::End
            | KeyCode::PageUp
            | KeyCode::PageDown
            | KeyCode::Left
            | KeyCode::Right
            | KeyCode::Up
            | KeyCode::Down
            | KeyCode::RCtrl
            | KeyCode::RAlt
            | KeyCode::RMeta
            | KeyCode::NumpadDivide
            | KeyCode::NumpadEnter
    );

    let full_scan_code = if is_extended {
        scan_code | 0xE000
    } else {
        scan_code
    };

    Some(full_scan_code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vk_to_keycode() {
        assert_eq!(vk_to_keycode(VK_A), Some(KeyCode::A));
        assert_eq!(vk_to_keycode(VK_RETURN), Some(KeyCode::Enter));
        assert_eq!(vk_to_keycode(0xFFFF), None);
    }

    #[test]
    fn test_keycode_to_vk() {
        assert_eq!(keycode_to_vk(KeyCode::A), Some(VK_A));
        assert_eq!(keycode_to_vk(KeyCode::Enter), Some(VK_RETURN));
    }

    #[test]
    fn test_roundtrip() {
        for (_, keycode) in VK_TO_KEYCODE.iter() {
            let vk = keycode_to_vk(*keycode).unwrap();
            let keycode2 = vk_to_keycode(vk).unwrap();
            assert_eq!(*keycode, keycode2);
        }
    }
}
