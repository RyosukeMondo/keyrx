//! Windows input driver using WH_KEYBOARD_LL.

use crate::engine::{InputEvent, KeyCode, OutputAction};
use crate::traits::InputSource;
use anyhow::Result;
use async_trait::async_trait;
use tracing::{debug, warn};

/// Windows input source using low-level keyboard hook.
pub struct WindowsInput {
    running: bool,
    /// Placeholder for hook handle (full integration is post-MVP).
    _hook_handle: Option<()>,
}

impl WindowsInput {
    /// Create a new Windows input source.
    pub fn new() -> Result<Self> {
        Ok(Self {
            running: false,
            _hook_handle: None,
        })
    }
}

impl Default for WindowsInput {
    fn default() -> Self {
        Self {
            running: false,
            _hook_handle: None,
        }
    }
}

#[async_trait]
impl InputSource for WindowsInput {
    async fn poll_events(&mut self) -> Result<Vec<InputEvent>> {
        if !self.running {
            warn!("poll_events called while not running");
            return Ok(vec![]);
        }

        // Stub: Return empty vec. Full WH_KEYBOARD_LL integration is post-MVP.
        // In the future, this would read events from the message queue.
        Ok(vec![])
    }

    async fn send_output(&mut self, action: OutputAction) -> Result<()> {
        if !self.running {
            warn!("send_output called while not running");
            return Ok(());
        }

        // Stub: Log the action. Full SendInput integration is post-MVP.
        debug!("Would send output: {:?}", action);
        Ok(())
    }

    async fn start(&mut self) -> Result<()> {
        if self.running {
            warn!("WindowsInput already running");
            return Ok(());
        }

        // Stub: Log hook registration attempt (always succeed for stub).
        // Full SetWindowsHookExW integration is post-MVP.
        debug!("Attempting to register WH_KEYBOARD_LL hook (stub: succeeding)");

        self.running = true;
        debug!("WindowsInput started successfully");

        // Stub: Full keyboard hook setup is post-MVP.
        // In the future, this would:
        // 1. Call SetWindowsHookExW with WH_KEYBOARD_LL
        // 2. Store the hook handle for later unhooking
        // 3. Start message pump thread for hook callbacks

        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        if !self.running {
            debug!("WindowsInput already stopped");
            return Ok(());
        }

        debug!("Unhooking WH_KEYBOARD_LL (stub)");
        self.running = false;
        debug!("WindowsInput stopped");

        // Stub: Full cleanup is post-MVP.
        // In the future, this would:
        // 1. Call UnhookWindowsHookEx with stored handle
        // 2. Stop the message pump thread
        // 3. Clean up any pending events

        Ok(())
    }
}

/// Convert Windows virtual key code to KeyRx KeyCode.
/// This is a stub mapping - full implementation post-MVP.
#[allow(dead_code)]
fn vk_to_keycode(vk: u16) -> KeyCode {
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
/// This is a stub mapping - full implementation post-MVP.
#[allow(dead_code)]
fn keycode_to_vk(key: KeyCode) -> u16 {
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
        // Unknown - return 0
        KeyCode::Unknown(vk) => vk,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_input_default() {
        let input = WindowsInput::default();
        assert!(!input.running);
    }

    #[test]
    fn windows_input_new() {
        let input = WindowsInput::new().unwrap();
        assert!(!input.running);
    }

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
}
