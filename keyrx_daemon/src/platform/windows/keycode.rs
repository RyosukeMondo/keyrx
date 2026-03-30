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
        // Punctuation (physical key positions — layout-independent)
        0x29 => Some(KeyCode::Grave),
        0x0C => Some(KeyCode::Minus),
        0x0D => Some(KeyCode::Equal),
        0x1A => Some(KeyCode::LeftBracket),
        0x1B => Some(KeyCode::RightBracket),
        0x2B => Some(KeyCode::Backslash),
        0x27 => Some(KeyCode::Semicolon),
        0x28 => Some(KeyCode::Quote),
        0x33 => Some(KeyCode::Comma),
        0x34 => Some(KeyCode::Period),
        0x35 => Some(KeyCode::Slash),
        // Lock keys
        0x3A => Some(KeyCode::CapsLock),
        0x45 => Some(KeyCode::NumLock),
        0x46 => Some(KeyCode::ScrollLock),
        // Numpad
        0x52 => Some(KeyCode::Numpad0),
        0x4F => Some(KeyCode::Numpad1),
        0x50 => Some(KeyCode::Numpad2),
        0x51 => Some(KeyCode::Numpad3),
        0x4B => Some(KeyCode::Numpad4),
        0x4C => Some(KeyCode::Numpad5),
        0x4D => Some(KeyCode::Numpad6),
        0x47 => Some(KeyCode::Numpad7),
        0x48 => Some(KeyCode::Numpad8),
        0x49 => Some(KeyCode::Numpad9),
        0x53 => Some(KeyCode::NumpadDecimal),
        0x37 => Some(KeyCode::NumpadMultiply),
        0x4A => Some(KeyCode::NumpadSubtract),
        0x4E => Some(KeyCode::NumpadAdd),
        // JIS-specific keys
        0x73 => Some(KeyCode::Ro),
        0x7D => Some(KeyCode::Yen),
        0x70 => Some(KeyCode::Hiragana),
        0x79 => Some(KeyCode::Henkan),
        0x7B => Some(KeyCode::Muhenkan),
        // Extended keys (E0-prefixed)
        0xE05B => Some(KeyCode::LMeta),
        0xE05C => Some(KeyCode::RMeta),
        0xE05D => Some(KeyCode::Menu),
        0xE037 => Some(KeyCode::PrintScreen),
        0xE11D => Some(KeyCode::Pause),

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

/// Convert KeyCode to Windows scan code for key blocking.
///
/// Uses a hardcoded table (reverse of scancode_to_keycode) to ensure
/// layout-independent mapping. MapVirtualKeyW is NOT used here because
/// it returns layout-dependent scan codes for punctuation keys (e.g.,
/// VK_OEM_4 → wrong scan code on JIS keyboards).
pub fn keycode_to_scancode(keycode: KeyCode) -> Option<u32> {
    let scan = match keycode {
        // Letters
        KeyCode::A => 0x1E,
        KeyCode::B => 0x30,
        KeyCode::C => 0x2E,
        KeyCode::D => 0x20,
        KeyCode::E => 0x12,
        KeyCode::F => 0x21,
        KeyCode::G => 0x22,
        KeyCode::H => 0x23,
        KeyCode::I => 0x17,
        KeyCode::J => 0x24,
        KeyCode::K => 0x25,
        KeyCode::L => 0x26,
        KeyCode::M => 0x32,
        KeyCode::N => 0x31,
        KeyCode::O => 0x18,
        KeyCode::P => 0x19,
        KeyCode::Q => 0x10,
        KeyCode::R => 0x13,
        KeyCode::S => 0x1F,
        KeyCode::T => 0x14,
        KeyCode::U => 0x16,
        KeyCode::V => 0x2F,
        KeyCode::W => 0x11,
        KeyCode::X => 0x2D,
        KeyCode::Y => 0x15,
        KeyCode::Z => 0x2C,
        // Numbers
        KeyCode::Num0 => 0x0B,
        KeyCode::Num1 => 0x02,
        KeyCode::Num2 => 0x03,
        KeyCode::Num3 => 0x04,
        KeyCode::Num4 => 0x05,
        KeyCode::Num5 => 0x06,
        KeyCode::Num6 => 0x07,
        KeyCode::Num7 => 0x08,
        KeyCode::Num8 => 0x09,
        KeyCode::Num9 => 0x0A,
        // Control keys
        KeyCode::Escape => 0x01,
        KeyCode::Enter => 0x1C,
        KeyCode::Space => 0x39,
        KeyCode::Backspace => 0x0E,
        KeyCode::Tab => 0x0F,
        KeyCode::LShift => 0x2A,
        KeyCode::RShift => 0x36,
        KeyCode::LCtrl => 0x1D,
        KeyCode::LAlt => 0x38,
        // Punctuation
        KeyCode::Grave => 0x29,
        KeyCode::Minus => 0x0C,
        KeyCode::Equal => 0x0D,
        KeyCode::LeftBracket => 0x1A,
        KeyCode::RightBracket => 0x1B,
        KeyCode::Backslash => 0x2B,
        KeyCode::Semicolon => 0x27,
        KeyCode::Quote => 0x28,
        KeyCode::Comma => 0x33,
        KeyCode::Period => 0x34,
        KeyCode::Slash => 0x35,
        // Lock keys
        KeyCode::CapsLock => 0x3A,
        KeyCode::NumLock => 0x45,
        KeyCode::ScrollLock => 0x46,
        // Function keys
        KeyCode::F1 => 0x3B,
        KeyCode::F2 => 0x3C,
        KeyCode::F3 => 0x3D,
        KeyCode::F4 => 0x3E,
        KeyCode::F5 => 0x3F,
        KeyCode::F6 => 0x40,
        KeyCode::F7 => 0x41,
        KeyCode::F8 => 0x42,
        KeyCode::F9 => 0x43,
        KeyCode::F10 => 0x44,
        KeyCode::F11 => 0x57,
        KeyCode::F12 => 0x58,
        // Numpad
        KeyCode::Numpad0 => 0x52,
        KeyCode::Numpad1 => 0x4F,
        KeyCode::Numpad2 => 0x50,
        KeyCode::Numpad3 => 0x51,
        KeyCode::Numpad4 => 0x4B,
        KeyCode::Numpad5 => 0x4C,
        KeyCode::Numpad6 => 0x4D,
        KeyCode::Numpad7 => 0x47,
        KeyCode::Numpad8 => 0x48,
        KeyCode::Numpad9 => 0x49,
        KeyCode::NumpadDecimal => 0x53,
        KeyCode::NumpadMultiply => 0x37,
        KeyCode::NumpadSubtract => 0x4A,
        KeyCode::NumpadAdd => 0x4E,
        // JIS-specific keys
        KeyCode::Ro => 0x73,
        KeyCode::Yen => 0x7D,
        KeyCode::Hiragana | KeyCode::KatakanaHiragana => 0x70,
        KeyCode::Henkan => 0x79,
        KeyCode::Muhenkan => 0x7B,
        KeyCode::Zenkaku => 0x29,
        // Extended keys (E0-prefixed)
        KeyCode::RCtrl => 0xE01D,
        KeyCode::RAlt => 0xE038,
        KeyCode::Home => 0xE047,
        KeyCode::Up => 0xE048,
        KeyCode::PageUp => 0xE049,
        KeyCode::Left => 0xE04B,
        KeyCode::Right => 0xE04D,
        KeyCode::End => 0xE04F,
        KeyCode::Down => 0xE050,
        KeyCode::PageDown => 0xE051,
        KeyCode::Insert => 0xE052,
        KeyCode::Delete => 0xE053,
        KeyCode::NumpadDivide => 0xE035,
        KeyCode::LMeta => 0xE05B,
        KeyCode::RMeta => 0xE05C,
        KeyCode::Menu => 0xE05D,
        KeyCode::PrintScreen => 0xE037,
        KeyCode::Pause => 0xE11D,
        _ => return None,
    };
    Some(scan)
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

    #[test]
    fn test_scancode_to_keycode_punctuation() {
        // Punctuation — these were previously falling through to MapVirtualKeyW
        assert_eq!(scancode_to_keycode(0x29), Some(KeyCode::Grave));
        assert_eq!(scancode_to_keycode(0x0C), Some(KeyCode::Minus));
        assert_eq!(scancode_to_keycode(0x0D), Some(KeyCode::Equal));
        assert_eq!(scancode_to_keycode(0x1A), Some(KeyCode::LeftBracket));
        assert_eq!(scancode_to_keycode(0x1B), Some(KeyCode::RightBracket));
        assert_eq!(scancode_to_keycode(0x2B), Some(KeyCode::Backslash));
        assert_eq!(scancode_to_keycode(0x27), Some(KeyCode::Semicolon));
        assert_eq!(scancode_to_keycode(0x28), Some(KeyCode::Quote));
        assert_eq!(scancode_to_keycode(0x33), Some(KeyCode::Comma));
        assert_eq!(scancode_to_keycode(0x34), Some(KeyCode::Period));
        assert_eq!(scancode_to_keycode(0x35), Some(KeyCode::Slash));
    }

    #[test]
    fn test_scancode_to_keycode_numpad() {
        assert_eq!(scancode_to_keycode(0x52), Some(KeyCode::Numpad0));
        assert_eq!(scancode_to_keycode(0x4F), Some(KeyCode::Numpad1));
        assert_eq!(scancode_to_keycode(0x4D), Some(KeyCode::Numpad6));
        assert_eq!(scancode_to_keycode(0x49), Some(KeyCode::Numpad9));
        assert_eq!(scancode_to_keycode(0x37), Some(KeyCode::NumpadMultiply));
        assert_eq!(scancode_to_keycode(0x4A), Some(KeyCode::NumpadSubtract));
        assert_eq!(scancode_to_keycode(0x4E), Some(KeyCode::NumpadAdd));
        assert_eq!(scancode_to_keycode(0x53), Some(KeyCode::NumpadDecimal));
    }

    #[test]
    fn test_scancode_to_keycode_lock_keys() {
        assert_eq!(scancode_to_keycode(0x3A), Some(KeyCode::CapsLock));
        assert_eq!(scancode_to_keycode(0x45), Some(KeyCode::NumLock));
        assert_eq!(scancode_to_keycode(0x46), Some(KeyCode::ScrollLock));
    }

    #[test]
    fn test_scancode_to_keycode_jis() {
        assert_eq!(scancode_to_keycode(0x73), Some(KeyCode::Ro));
        assert_eq!(scancode_to_keycode(0x7D), Some(KeyCode::Yen));
        assert_eq!(scancode_to_keycode(0x70), Some(KeyCode::Hiragana));
        assert_eq!(scancode_to_keycode(0x79), Some(KeyCode::Henkan));
        assert_eq!(scancode_to_keycode(0x7B), Some(KeyCode::Muhenkan));
    }

    #[test]
    fn test_scancode_to_keycode_extended() {
        assert_eq!(scancode_to_keycode(0xE05B), Some(KeyCode::LMeta));
        assert_eq!(scancode_to_keycode(0xE05C), Some(KeyCode::RMeta));
        assert_eq!(scancode_to_keycode(0xE05D), Some(KeyCode::Menu));
        assert_eq!(scancode_to_keycode(0xE037), Some(KeyCode::PrintScreen));
    }

    #[test]
    fn test_scancode_roundtrip() {
        // Every scancode→keycode mapping must roundtrip through keycode→scancode.
        // This ensures the key blocker registers the same scan code the hook sees.
        let test_scancodes: &[u32] = &[
            // Letters
            0x1E, 0x30, 0x2E, 0x20, 0x12, 0x21, 0x22, 0x23, 0x17, 0x24, 0x25, 0x26, 0x32, 0x31,
            0x18, 0x19, 0x10, 0x13, 0x1F, 0x14, 0x16, 0x2F, 0x11, 0x2D, 0x15, 0x2C,
            // Numbers
            0x0B, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, // Control
            0x01, 0x1C, 0x39, 0x0E, 0x0F, 0x2A, 0x36, 0x1D, 0x38,
            // Punctuation (critical for JIS)
            0x29, 0x0C, 0x0D, 0x1A, 0x1B, 0x2B, 0x27, 0x28, 0x33, 0x34, 0x35, // Locks
            0x3A, 0x45, 0x46, // Function keys
            0x3B, 0x3C, 0x3D, 0x3E, 0x3F, 0x40, 0x41, 0x42, 0x43, 0x44, 0x57, 0x58,
            // Numpad
            0x52, 0x4F, 0x50, 0x51, 0x4B, 0x4C, 0x4D, 0x47, 0x48, 0x49, 0x53, 0x37, 0x4A, 0x4E,
            // JIS
            0x73, 0x7D, 0x70, 0x79, 0x7B, // Extended
            0xE01D, 0xE038, 0xE047, 0xE048, 0xE049, 0xE04B, 0xE04D, 0xE04F, 0xE050, 0xE051, 0xE052,
            0xE053, 0xE035, 0xE05B, 0xE05C, 0xE05D, 0xE037,
        ];

        for &sc in test_scancodes {
            let kc = scancode_to_keycode(sc)
                .unwrap_or_else(|| panic!("scancode 0x{:04X} has no keycode", sc));
            let sc2 = keycode_to_scancode(kc)
                .unwrap_or_else(|| panic!("keycode {:?} has no scancode", kc));
            assert_eq!(
                sc, sc2,
                "Roundtrip failed: 0x{:04X} → {:?} → 0x{:04X}",
                sc, kc, sc2
            );
        }
    }
}
