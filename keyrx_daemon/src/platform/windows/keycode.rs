use keyrx_core::config::KeyCode;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;

/// Bidirectional mapping between Windows Virtual Key codes and KeyRx KeyCode enum.
/// Uses const arrays for O(1) lookup performance.
// Mapping from VK to KeyCode
const VK_TO_KEYCODE: [(u16, KeyCode); 141] = [
    (VK_A as u16, KeyCode::A),
    (VK_B as u16, KeyCode::B),
    (VK_C as u16, KeyCode::C),
    (VK_D as u16, KeyCode::D),
    (VK_E as u16, KeyCode::E),
    (VK_F as u16, KeyCode::F),
    (VK_G as u16, KeyCode::G),
    (VK_H as u16, KeyCode::H),
    (VK_I as u16, KeyCode::I),
    (VK_J as u16, KeyCode::J),
    (VK_K as u16, KeyCode::K),
    (VK_L as u16, KeyCode::L),
    (VK_M as u16, KeyCode::M),
    (VK_N as u16, KeyCode::N),
    (VK_O as u16, KeyCode::O),
    (VK_P as u16, KeyCode::P),
    (VK_Q as u16, KeyCode::Q),
    (VK_R as u16, KeyCode::R),
    (VK_S as u16, KeyCode::S),
    (VK_T as u16, KeyCode::T),
    (VK_U as u16, KeyCode::U),
    (VK_V as u16, KeyCode::V),
    (VK_W as u16, KeyCode::W),
    (VK_X as u16, KeyCode::X),
    (VK_Y as u16, KeyCode::Y),
    (VK_Z as u16, KeyCode::Z),
    (VK_0 as u16, KeyCode::Num0),
    (VK_1 as u16, KeyCode::Num1),
    (VK_2 as u16, KeyCode::Num2),
    (VK_3 as u16, KeyCode::Num3),
    (VK_4 as u16, KeyCode::Num4),
    (VK_5 as u16, KeyCode::Num5),
    (VK_6 as u16, KeyCode::Num6),
    (VK_7 as u16, KeyCode::Num7),
    (VK_8 as u16, KeyCode::Num8),
    (VK_9 as u16, KeyCode::Num9),
    (VK_F1 as u16, KeyCode::F1),
    (VK_F2 as u16, KeyCode::F2),
    (VK_F3 as u16, KeyCode::F3),
    (VK_F4 as u16, KeyCode::F4),
    (VK_F5 as u16, KeyCode::F5),
    (VK_F6 as u16, KeyCode::F6),
    (VK_F7 as u16, KeyCode::F7),
    (VK_F8 as u16, KeyCode::F8),
    (VK_F9 as u16, KeyCode::F9),
    (VK_F10 as u16, KeyCode::F10),
    (VK_F11 as u16, KeyCode::F11),
    (VK_F12 as u16, KeyCode::F12),
    (VK_F13 as u16, KeyCode::F13),
    (VK_F14 as u16, KeyCode::F14),
    (VK_F15 as u16, KeyCode::F15),
    (VK_F16 as u16, KeyCode::F16),
    (VK_F17 as u16, KeyCode::F17),
    (VK_F18 as u16, KeyCode::F18),
    (VK_F19 as u16, KeyCode::F19),
    (VK_F20 as u16, KeyCode::F20),
    (VK_F21 as u16, KeyCode::F21),
    (VK_F22 as u16, KeyCode::F22),
    (VK_F23 as u16, KeyCode::F23),
    (VK_F24 as u16, KeyCode::F24),
    (VK_LSHIFT as u16, KeyCode::LShift),
    (VK_RSHIFT as u16, KeyCode::RShift),
    (VK_LCONTROL as u16, KeyCode::LCtrl),
    (VK_RCONTROL as u16, KeyCode::RCtrl),
    (VK_LMENU as u16, KeyCode::LAlt),
    (VK_RMENU as u16, KeyCode::RAlt),
    (VK_LWIN as u16, KeyCode::LMeta),
    (VK_RWIN as u16, KeyCode::RMeta),
    (VK_ESCAPE as u16, KeyCode::Escape),
    (VK_RETURN as u16, KeyCode::Enter),
    (VK_BACK as u16, KeyCode::Backspace),
    (VK_TAB as u16, KeyCode::Tab),
    (VK_SPACE as u16, KeyCode::Space),
    (VK_CAPITAL as u16, KeyCode::CapsLock),
    (VK_NUMLOCK as u16, KeyCode::NumLock),
    (VK_SCROLL as u16, KeyCode::ScrollLock),
    (VK_SNAPSHOT as u16, KeyCode::PrintScreen),
    (VK_PAUSE as u16, KeyCode::Pause),
    (VK_INSERT as u16, KeyCode::Insert),
    (VK_DELETE as u16, KeyCode::Delete),
    (VK_HOME as u16, KeyCode::Home),
    (VK_END as u16, KeyCode::End),
    (VK_PRIOR as u16, KeyCode::PageUp),
    (VK_NEXT as u16, KeyCode::PageDown),
    (VK_LEFT as u16, KeyCode::Left),
    (VK_RIGHT as u16, KeyCode::Right),
    (VK_UP as u16, KeyCode::Up),
    (VK_DOWN as u16, KeyCode::Down),
    (VK_OEM_4 as u16, KeyCode::LeftBracket),
    (VK_OEM_6 as u16, KeyCode::RightBracket),
    (VK_OEM_5 as u16, KeyCode::Backslash),
    (VK_OEM_1 as u16, KeyCode::Semicolon),
    (VK_OEM_7 as u16, KeyCode::Quote),
    (VK_OEM_COMMA as u16, KeyCode::Comma),
    (VK_OEM_PERIOD as u16, KeyCode::Period),
    (VK_OEM_2 as u16, KeyCode::Slash),
    (VK_OEM_3 as u16, KeyCode::Grave),
    (VK_OEM_MINUS as u16, KeyCode::Minus),
    (VK_OEM_PLUS as u16, KeyCode::Equal),
    (VK_NUMPAD0 as u16, KeyCode::Numpad0),
    (VK_NUMPAD1 as u16, KeyCode::Numpad1),
    (VK_NUMPAD2 as u16, KeyCode::Numpad2),
    (VK_NUMPAD3 as u16, KeyCode::Numpad3),
    (VK_NUMPAD4 as u16, KeyCode::Numpad4),
    (VK_NUMPAD5 as u16, KeyCode::Numpad5),
    (VK_NUMPAD6 as u16, KeyCode::Numpad6),
    (VK_NUMPAD7 as u16, KeyCode::Numpad7),
    (VK_NUMPAD8 as u16, KeyCode::Numpad8),
    (VK_NUMPAD9 as u16, KeyCode::Numpad9),
    (VK_DIVIDE as u16, KeyCode::NumpadDivide),
    (VK_MULTIPLY as u16, KeyCode::NumpadMultiply),
    (VK_SUBTRACT as u16, KeyCode::NumpadSubtract),
    (VK_ADD as u16, KeyCode::NumpadAdd),
    (VK_DECIMAL as u16, KeyCode::NumpadDecimal),
    (VK_VOLUME_MUTE as u16, KeyCode::Mute),
    (VK_VOLUME_DOWN as u16, KeyCode::VolumeDown),
    (VK_VOLUME_UP as u16, KeyCode::VolumeUp),
    (VK_MEDIA_PLAY_PAUSE as u16, KeyCode::MediaPlayPause),
    (VK_MEDIA_STOP as u16, KeyCode::MediaStop),
    (VK_MEDIA_PREV_TRACK as u16, KeyCode::MediaPrevious),
    (VK_MEDIA_NEXT_TRACK as u16, KeyCode::MediaNext),
    (VK_SLEEP as u16, KeyCode::Sleep),
    (VK_BROWSER_BACK as u16, KeyCode::BrowserBack),
    (VK_BROWSER_FORWARD as u16, KeyCode::BrowserForward),
    (VK_BROWSER_REFRESH as u16, KeyCode::BrowserRefresh),
    (VK_BROWSER_STOP as u16, KeyCode::BrowserStop),
    (VK_BROWSER_SEARCH as u16, KeyCode::BrowserSearch),
    (VK_BROWSER_FAVORITES as u16, KeyCode::BrowserFavorites),
    (VK_BROWSER_HOME as u16, KeyCode::BrowserHome),
    (VK_LAUNCH_MAIL as u16, KeyCode::AppMail),
    (VK_LAUNCH_APP2 as u16, KeyCode::AppCalculator),
    (VK_LAUNCH_APP1 as u16, KeyCode::AppMyComputer),
    (VK_APPS as u16, KeyCode::Menu),
    (VK_HELP as u16, KeyCode::Help),
    (VK_SELECT as u16, KeyCode::Select),
    (VK_EXECUTE as u16, KeyCode::Execute),
    (VK_KANJI as u16, KeyCode::Zenkaku), // Best effort mapping
    (VK_KANA as u16, KeyCode::KatakanaHiragana),
    (VK_CONVERT as u16, KeyCode::Henkan),
    (VK_NONCONVERT as u16, KeyCode::Muhenkan),
    (VK_OEM_102 as u16, KeyCode::Iso102nd),
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
    let scan_code = unsafe { windows_sys::Win32::UI::Input::KeyboardAndMouse::MapVirtualKeyW(vk as u32, 0) };

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
        assert_eq!(vk_to_keycode(VK_A as u16), Some(KeyCode::A));
        assert_eq!(vk_to_keycode(VK_RETURN as u16), Some(KeyCode::Enter));
        assert_eq!(vk_to_keycode(0xFFFF), None);
    }

    #[test]
    fn test_keycode_to_vk() {
        assert_eq!(keycode_to_vk(KeyCode::A), Some(VK_A as u16));
        assert_eq!(keycode_to_vk(KeyCode::Enter), Some(VK_RETURN as u16));
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
