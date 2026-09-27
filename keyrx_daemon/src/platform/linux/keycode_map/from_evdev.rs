use evdev::Key;

use keyrx_core::config::KeyCode;

/// Maps an evdev key code to a keyrx KeyCode.
///
/// # Arguments
/// * `code` - The evdev key code (from linux/input-event-codes.h)
///
/// # Returns
/// * `Some(KeyCode)` if the code maps to a known key
/// * `None` if the code is unknown (passthrough handling)
///
/// # Key Categories
/// - Letters: KEY_A (30) through KEY_Z
/// - Numbers: KEY_1 (2) through KEY_0 (11)
/// - Function keys: KEY_F1 (59) through KEY_F24
/// - Modifiers: KEY_LEFTSHIFT, KEY_RIGHTSHIFT, etc.
/// - Special keys: KEY_ESC, KEY_ENTER, KEY_BACKSPACE, etc.
#[must_use]
pub fn evdev_to_keycode(code: u16) -> Option<KeyCode> {
    // Convert u16 to evdev Key for pattern matching
    let key = Key::new(code);

    match key {
        // Letters A-Z
        Key::KEY_A => Some(KeyCode::A),
        Key::KEY_B => Some(KeyCode::B),
        Key::KEY_C => Some(KeyCode::C),
        Key::KEY_D => Some(KeyCode::D),
        Key::KEY_E => Some(KeyCode::E),
        Key::KEY_F => Some(KeyCode::F),
        Key::KEY_G => Some(KeyCode::G),
        Key::KEY_H => Some(KeyCode::H),
        Key::KEY_I => Some(KeyCode::I),
        Key::KEY_J => Some(KeyCode::J),
        Key::KEY_K => Some(KeyCode::K),
        Key::KEY_L => Some(KeyCode::L),
        Key::KEY_M => Some(KeyCode::M),
        Key::KEY_N => Some(KeyCode::N),
        Key::KEY_O => Some(KeyCode::O),
        Key::KEY_P => Some(KeyCode::P),
        Key::KEY_Q => Some(KeyCode::Q),
        Key::KEY_R => Some(KeyCode::R),
        Key::KEY_S => Some(KeyCode::S),
        Key::KEY_T => Some(KeyCode::T),
        Key::KEY_U => Some(KeyCode::U),
        Key::KEY_V => Some(KeyCode::V),
        Key::KEY_W => Some(KeyCode::W),
        Key::KEY_X => Some(KeyCode::X),
        Key::KEY_Y => Some(KeyCode::Y),
        Key::KEY_Z => Some(KeyCode::Z),

        // Numbers 0-9 (top row)
        // Note: evdev uses KEY_1 (2) through KEY_0 (11), not KEY_0 through KEY_9
        Key::KEY_1 => Some(KeyCode::Num1),
        Key::KEY_2 => Some(KeyCode::Num2),
        Key::KEY_3 => Some(KeyCode::Num3),
        Key::KEY_4 => Some(KeyCode::Num4),
        Key::KEY_5 => Some(KeyCode::Num5),
        Key::KEY_6 => Some(KeyCode::Num6),
        Key::KEY_7 => Some(KeyCode::Num7),
        Key::KEY_8 => Some(KeyCode::Num8),
        Key::KEY_9 => Some(KeyCode::Num9),
        Key::KEY_0 => Some(KeyCode::Num0),

        // Function keys F1-F12
        Key::KEY_F1 => Some(KeyCode::F1),
        Key::KEY_F2 => Some(KeyCode::F2),
        Key::KEY_F3 => Some(KeyCode::F3),
        Key::KEY_F4 => Some(KeyCode::F4),
        Key::KEY_F5 => Some(KeyCode::F5),
        Key::KEY_F6 => Some(KeyCode::F6),
        Key::KEY_F7 => Some(KeyCode::F7),
        Key::KEY_F8 => Some(KeyCode::F8),
        Key::KEY_F9 => Some(KeyCode::F9),
        Key::KEY_F10 => Some(KeyCode::F10),
        Key::KEY_F11 => Some(KeyCode::F11),
        Key::KEY_F12 => Some(KeyCode::F12),

        // Extended function keys F13-F24
        Key::KEY_F13 => Some(KeyCode::F13),
        Key::KEY_F14 => Some(KeyCode::F14),
        Key::KEY_F15 => Some(KeyCode::F15),
        Key::KEY_F16 => Some(KeyCode::F16),
        Key::KEY_F17 => Some(KeyCode::F17),
        Key::KEY_F18 => Some(KeyCode::F18),
        Key::KEY_F19 => Some(KeyCode::F19),
        Key::KEY_F20 => Some(KeyCode::F20),
        Key::KEY_F21 => Some(KeyCode::F21),
        Key::KEY_F22 => Some(KeyCode::F22),
        Key::KEY_F23 => Some(KeyCode::F23),
        Key::KEY_F24 => Some(KeyCode::F24),

        // Modifier keys
        Key::KEY_LEFTSHIFT => Some(KeyCode::LShift),
        Key::KEY_RIGHTSHIFT => Some(KeyCode::RShift),
        Key::KEY_LEFTCTRL => Some(KeyCode::LCtrl),
        Key::KEY_RIGHTCTRL => Some(KeyCode::RCtrl),
        Key::KEY_LEFTALT => Some(KeyCode::LAlt),
        Key::KEY_RIGHTALT => Some(KeyCode::RAlt),
        Key::KEY_LEFTMETA => Some(KeyCode::LMeta),
        Key::KEY_RIGHTMETA => Some(KeyCode::RMeta),

        // Special keys
        Key::KEY_ESC => Some(KeyCode::Escape),
        Key::KEY_ENTER => Some(KeyCode::Enter),
        Key::KEY_BACKSPACE => Some(KeyCode::Backspace),
        Key::KEY_TAB => Some(KeyCode::Tab),
        Key::KEY_SPACE => Some(KeyCode::Space),
        Key::KEY_CAPSLOCK => Some(KeyCode::CapsLock),
        Key::KEY_NUMLOCK => Some(KeyCode::NumLock),
        Key::KEY_SCROLLLOCK => Some(KeyCode::ScrollLock),
        Key::KEY_SYSRQ => Some(KeyCode::PrintScreen),
        Key::KEY_PAUSE => Some(KeyCode::Pause),
        Key::KEY_INSERT => Some(KeyCode::Insert),
        Key::KEY_DELETE => Some(KeyCode::Delete),
        Key::KEY_HOME => Some(KeyCode::Home),
        Key::KEY_END => Some(KeyCode::End),
        Key::KEY_PAGEUP => Some(KeyCode::PageUp),
        Key::KEY_PAGEDOWN => Some(KeyCode::PageDown),

        // Arrow keys
        Key::KEY_LEFT => Some(KeyCode::Left),
        Key::KEY_RIGHT => Some(KeyCode::Right),
        Key::KEY_UP => Some(KeyCode::Up),
        Key::KEY_DOWN => Some(KeyCode::Down),

        // Punctuation and symbols
        Key::KEY_LEFTBRACE => Some(KeyCode::LeftBracket),
        Key::KEY_RIGHTBRACE => Some(KeyCode::RightBracket),
        Key::KEY_BACKSLASH => Some(KeyCode::Backslash),
        Key::KEY_SEMICOLON => Some(KeyCode::Semicolon),
        Key::KEY_APOSTROPHE => Some(KeyCode::Quote),
        Key::KEY_COMMA => Some(KeyCode::Comma),
        Key::KEY_DOT => Some(KeyCode::Period),
        Key::KEY_SLASH => Some(KeyCode::Slash),
        Key::KEY_GRAVE => Some(KeyCode::Grave),
        Key::KEY_MINUS => Some(KeyCode::Minus),
        Key::KEY_EQUAL => Some(KeyCode::Equal),

        // Numpad keys
        Key::KEY_KP0 => Some(KeyCode::Numpad0),
        Key::KEY_KP1 => Some(KeyCode::Numpad1),
        Key::KEY_KP2 => Some(KeyCode::Numpad2),
        Key::KEY_KP3 => Some(KeyCode::Numpad3),
        Key::KEY_KP4 => Some(KeyCode::Numpad4),
        Key::KEY_KP5 => Some(KeyCode::Numpad5),
        Key::KEY_KP6 => Some(KeyCode::Numpad6),
        Key::KEY_KP7 => Some(KeyCode::Numpad7),
        Key::KEY_KP8 => Some(KeyCode::Numpad8),
        Key::KEY_KP9 => Some(KeyCode::Numpad9),
        Key::KEY_KPSLASH => Some(KeyCode::NumpadDivide),
        Key::KEY_KPASTERISK => Some(KeyCode::NumpadMultiply),
        Key::KEY_KPMINUS => Some(KeyCode::NumpadSubtract),
        Key::KEY_KPPLUS => Some(KeyCode::NumpadAdd),
        Key::KEY_KPENTER => Some(KeyCode::NumpadEnter),
        Key::KEY_KPDOT => Some(KeyCode::NumpadDecimal),

        // Media keys
        Key::KEY_MUTE => Some(KeyCode::Mute),
        Key::KEY_VOLUMEDOWN => Some(KeyCode::VolumeDown),
        Key::KEY_VOLUMEUP => Some(KeyCode::VolumeUp),
        Key::KEY_PLAYPAUSE => Some(KeyCode::MediaPlayPause),
        Key::KEY_STOPCD => Some(KeyCode::MediaStop),
        Key::KEY_PREVIOUSSONG => Some(KeyCode::MediaPrevious),
        Key::KEY_NEXTSONG => Some(KeyCode::MediaNext),

        // System keys
        Key::KEY_POWER => Some(KeyCode::Power),
        Key::KEY_SLEEP => Some(KeyCode::Sleep),
        Key::KEY_WAKEUP => Some(KeyCode::Wake),

        // Browser keys
        Key::KEY_BACK => Some(KeyCode::BrowserBack),
        Key::KEY_FORWARD => Some(KeyCode::BrowserForward),
        Key::KEY_REFRESH => Some(KeyCode::BrowserRefresh),
        Key::KEY_STOP => Some(KeyCode::BrowserStop),
        Key::KEY_SEARCH => Some(KeyCode::BrowserSearch),
        Key::KEY_BOOKMARKS => Some(KeyCode::BrowserFavorites),
        Key::KEY_HOMEPAGE => Some(KeyCode::BrowserHome),

        // Application keys
        Key::KEY_MAIL => Some(KeyCode::AppMail),
        Key::KEY_CALC => Some(KeyCode::AppCalculator),
        Key::KEY_COMPUTER => Some(KeyCode::AppMyComputer),

        // Additional keys
        Key::KEY_COMPOSE => Some(KeyCode::Menu),
        Key::KEY_HELP => Some(KeyCode::Help),
        Key::KEY_SELECT => Some(KeyCode::Select),
        Key::KEY_OPEN => Some(KeyCode::Execute), // KEY_OPEN is closest match for Execute
        Key::KEY_UNDO => Some(KeyCode::Undo),
        Key::KEY_REDO => Some(KeyCode::Redo),
        Key::KEY_CUT => Some(KeyCode::Cut),
        Key::KEY_COPY => Some(KeyCode::Copy),
        Key::KEY_PASTE => Some(KeyCode::Paste),
        Key::KEY_FIND => Some(KeyCode::Find),

        // Japanese JIS keyboard keys (日本語キーボード)
        Key::KEY_ZENKAKUHANKAKU => Some(KeyCode::Zenkaku),
        Key::KEY_KATAKANA => Some(KeyCode::Katakana),
        Key::KEY_HIRAGANA => Some(KeyCode::Hiragana),
        Key::KEY_HENKAN => Some(KeyCode::Henkan),
        Key::KEY_MUHENKAN => Some(KeyCode::Muhenkan),
        Key::KEY_YEN => Some(KeyCode::Yen),
        Key::KEY_RO => Some(KeyCode::Ro),
        Key::KEY_KATAKANAHIRAGANA => Some(KeyCode::KatakanaHiragana),

        // Korean keyboard keys (한국어 키보드)
        Key::KEY_HANGEUL => Some(KeyCode::Hangeul),
        Key::KEY_HANJA => Some(KeyCode::Hanja),

        // ISO/European keyboard keys
        Key::KEY_102ND => Some(KeyCode::Iso102nd),

        // Unknown key - return None for passthrough handling
        _ => None,
    }
}
