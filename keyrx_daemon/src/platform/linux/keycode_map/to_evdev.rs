use evdev::Key;

use keyrx_core::config::KeyCode;

/// Maps a keyrx KeyCode to an evdev key code.
///
/// # Arguments
/// * `keycode` - The keyrx KeyCode to convert
///
/// # Returns
/// The corresponding evdev key code (u16)
///
/// # Note
/// This function covers all KeyCode variants exhaustively.
/// The mapping is the inverse of `evdev_to_keycode`.
#[must_use]
#[allow(dead_code)] // Used in tests and will be used for output injection
pub fn keycode_to_evdev(keycode: KeyCode) -> u16 {
    match keycode {
        // Letters A-Z
        KeyCode::A => Key::KEY_A.code(),
        KeyCode::B => Key::KEY_B.code(),
        KeyCode::C => Key::KEY_C.code(),
        KeyCode::D => Key::KEY_D.code(),
        KeyCode::E => Key::KEY_E.code(),
        KeyCode::F => Key::KEY_F.code(),
        KeyCode::G => Key::KEY_G.code(),
        KeyCode::H => Key::KEY_H.code(),
        KeyCode::I => Key::KEY_I.code(),
        KeyCode::J => Key::KEY_J.code(),
        KeyCode::K => Key::KEY_K.code(),
        KeyCode::L => Key::KEY_L.code(),
        KeyCode::M => Key::KEY_M.code(),
        KeyCode::N => Key::KEY_N.code(),
        KeyCode::O => Key::KEY_O.code(),
        KeyCode::P => Key::KEY_P.code(),
        KeyCode::Q => Key::KEY_Q.code(),
        KeyCode::R => Key::KEY_R.code(),
        KeyCode::S => Key::KEY_S.code(),
        KeyCode::T => Key::KEY_T.code(),
        KeyCode::U => Key::KEY_U.code(),
        KeyCode::V => Key::KEY_V.code(),
        KeyCode::W => Key::KEY_W.code(),
        KeyCode::X => Key::KEY_X.code(),
        KeyCode::Y => Key::KEY_Y.code(),
        KeyCode::Z => Key::KEY_Z.code(),

        // Numbers 0-9 (top row)
        KeyCode::Num0 => Key::KEY_0.code(),
        KeyCode::Num1 => Key::KEY_1.code(),
        KeyCode::Num2 => Key::KEY_2.code(),
        KeyCode::Num3 => Key::KEY_3.code(),
        KeyCode::Num4 => Key::KEY_4.code(),
        KeyCode::Num5 => Key::KEY_5.code(),
        KeyCode::Num6 => Key::KEY_6.code(),
        KeyCode::Num7 => Key::KEY_7.code(),
        KeyCode::Num8 => Key::KEY_8.code(),
        KeyCode::Num9 => Key::KEY_9.code(),

        // Function keys F1-F12
        KeyCode::F1 => Key::KEY_F1.code(),
        KeyCode::F2 => Key::KEY_F2.code(),
        KeyCode::F3 => Key::KEY_F3.code(),
        KeyCode::F4 => Key::KEY_F4.code(),
        KeyCode::F5 => Key::KEY_F5.code(),
        KeyCode::F6 => Key::KEY_F6.code(),
        KeyCode::F7 => Key::KEY_F7.code(),
        KeyCode::F8 => Key::KEY_F8.code(),
        KeyCode::F9 => Key::KEY_F9.code(),
        KeyCode::F10 => Key::KEY_F10.code(),
        KeyCode::F11 => Key::KEY_F11.code(),
        KeyCode::F12 => Key::KEY_F12.code(),

        // Extended function keys F13-F24
        KeyCode::F13 => Key::KEY_F13.code(),
        KeyCode::F14 => Key::KEY_F14.code(),
        KeyCode::F15 => Key::KEY_F15.code(),
        KeyCode::F16 => Key::KEY_F16.code(),
        KeyCode::F17 => Key::KEY_F17.code(),
        KeyCode::F18 => Key::KEY_F18.code(),
        KeyCode::F19 => Key::KEY_F19.code(),
        KeyCode::F20 => Key::KEY_F20.code(),
        KeyCode::F21 => Key::KEY_F21.code(),
        KeyCode::F22 => Key::KEY_F22.code(),
        KeyCode::F23 => Key::KEY_F23.code(),
        KeyCode::F24 => Key::KEY_F24.code(),

        // Modifier keys
        KeyCode::LShift => Key::KEY_LEFTSHIFT.code(),
        KeyCode::RShift => Key::KEY_RIGHTSHIFT.code(),
        KeyCode::LCtrl => Key::KEY_LEFTCTRL.code(),
        KeyCode::RCtrl => Key::KEY_RIGHTCTRL.code(),
        KeyCode::LAlt => Key::KEY_LEFTALT.code(),
        KeyCode::RAlt => Key::KEY_RIGHTALT.code(),
        KeyCode::LMeta => Key::KEY_LEFTMETA.code(),
        KeyCode::RMeta => Key::KEY_RIGHTMETA.code(),

        // Special keys
        KeyCode::Escape => Key::KEY_ESC.code(),
        KeyCode::Enter => Key::KEY_ENTER.code(),
        KeyCode::Backspace => Key::KEY_BACKSPACE.code(),
        KeyCode::Tab => Key::KEY_TAB.code(),
        KeyCode::Space => Key::KEY_SPACE.code(),
        KeyCode::CapsLock => Key::KEY_CAPSLOCK.code(),
        KeyCode::NumLock => Key::KEY_NUMLOCK.code(),
        KeyCode::ScrollLock => Key::KEY_SCROLLLOCK.code(),
        KeyCode::PrintScreen => Key::KEY_SYSRQ.code(),
        KeyCode::Pause => Key::KEY_PAUSE.code(),
        KeyCode::Insert => Key::KEY_INSERT.code(),
        KeyCode::Delete => Key::KEY_DELETE.code(),
        KeyCode::Home => Key::KEY_HOME.code(),
        KeyCode::End => Key::KEY_END.code(),
        KeyCode::PageUp => Key::KEY_PAGEUP.code(),
        KeyCode::PageDown => Key::KEY_PAGEDOWN.code(),

        // Arrow keys
        KeyCode::Left => Key::KEY_LEFT.code(),
        KeyCode::Right => Key::KEY_RIGHT.code(),
        KeyCode::Up => Key::KEY_UP.code(),
        KeyCode::Down => Key::KEY_DOWN.code(),

        // Punctuation and symbols
        KeyCode::LeftBracket => Key::KEY_LEFTBRACE.code(),
        KeyCode::RightBracket => Key::KEY_RIGHTBRACE.code(),
        KeyCode::Backslash => Key::KEY_BACKSLASH.code(),
        KeyCode::Semicolon => Key::KEY_SEMICOLON.code(),
        KeyCode::Quote => Key::KEY_APOSTROPHE.code(),
        KeyCode::Comma => Key::KEY_COMMA.code(),
        KeyCode::Period => Key::KEY_DOT.code(),
        KeyCode::Slash => Key::KEY_SLASH.code(),
        KeyCode::Grave => Key::KEY_GRAVE.code(),
        KeyCode::Minus => Key::KEY_MINUS.code(),
        KeyCode::Equal => Key::KEY_EQUAL.code(),

        // Numpad keys
        KeyCode::Numpad0 => Key::KEY_KP0.code(),
        KeyCode::Numpad1 => Key::KEY_KP1.code(),
        KeyCode::Numpad2 => Key::KEY_KP2.code(),
        KeyCode::Numpad3 => Key::KEY_KP3.code(),
        KeyCode::Numpad4 => Key::KEY_KP4.code(),
        KeyCode::Numpad5 => Key::KEY_KP5.code(),
        KeyCode::Numpad6 => Key::KEY_KP6.code(),
        KeyCode::Numpad7 => Key::KEY_KP7.code(),
        KeyCode::Numpad8 => Key::KEY_KP8.code(),
        KeyCode::Numpad9 => Key::KEY_KP9.code(),
        KeyCode::NumpadDivide => Key::KEY_KPSLASH.code(),
        KeyCode::NumpadMultiply => Key::KEY_KPASTERISK.code(),
        KeyCode::NumpadSubtract => Key::KEY_KPMINUS.code(),
        KeyCode::NumpadAdd => Key::KEY_KPPLUS.code(),
        KeyCode::NumpadEnter => Key::KEY_KPENTER.code(),
        KeyCode::NumpadDecimal => Key::KEY_KPDOT.code(),

        // Media keys
        KeyCode::Mute => Key::KEY_MUTE.code(),
        KeyCode::VolumeDown => Key::KEY_VOLUMEDOWN.code(),
        KeyCode::VolumeUp => Key::KEY_VOLUMEUP.code(),
        KeyCode::MediaPlayPause => Key::KEY_PLAYPAUSE.code(),
        KeyCode::MediaStop => Key::KEY_STOPCD.code(),
        KeyCode::MediaPrevious => Key::KEY_PREVIOUSSONG.code(),
        KeyCode::MediaNext => Key::KEY_NEXTSONG.code(),

        // System keys
        KeyCode::Power => Key::KEY_POWER.code(),
        KeyCode::Sleep => Key::KEY_SLEEP.code(),
        KeyCode::Wake => Key::KEY_WAKEUP.code(),

        // Browser keys
        KeyCode::BrowserBack => Key::KEY_BACK.code(),
        KeyCode::BrowserForward => Key::KEY_FORWARD.code(),
        KeyCode::BrowserRefresh => Key::KEY_REFRESH.code(),
        KeyCode::BrowserStop => Key::KEY_STOP.code(),
        KeyCode::BrowserSearch => Key::KEY_SEARCH.code(),
        KeyCode::BrowserFavorites => Key::KEY_BOOKMARKS.code(),
        KeyCode::BrowserHome => Key::KEY_HOMEPAGE.code(),

        // Application keys
        KeyCode::AppMail => Key::KEY_MAIL.code(),
        KeyCode::AppCalculator => Key::KEY_CALC.code(),
        KeyCode::AppMyComputer => Key::KEY_COMPUTER.code(),

        // Additional keys
        KeyCode::Menu => Key::KEY_COMPOSE.code(),
        KeyCode::Help => Key::KEY_HELP.code(),
        KeyCode::Select => Key::KEY_SELECT.code(),
        KeyCode::Execute => Key::KEY_OPEN.code(), // Closest match for Execute
        KeyCode::Undo => Key::KEY_UNDO.code(),
        KeyCode::Redo => Key::KEY_REDO.code(),
        KeyCode::Cut => Key::KEY_CUT.code(),
        KeyCode::Copy => Key::KEY_COPY.code(),
        KeyCode::Paste => Key::KEY_PASTE.code(),
        KeyCode::Find => Key::KEY_FIND.code(),

        // Japanese JIS keyboard keys (日本語キーボード)
        KeyCode::Zenkaku => Key::KEY_ZENKAKUHANKAKU.code(),
        KeyCode::Katakana => Key::KEY_KATAKANA.code(),
        KeyCode::Hiragana => Key::KEY_HIRAGANA.code(),
        KeyCode::Henkan => Key::KEY_HENKAN.code(),
        KeyCode::Muhenkan => Key::KEY_MUHENKAN.code(),
        KeyCode::Yen => Key::KEY_YEN.code(),
        KeyCode::Ro => Key::KEY_RO.code(),
        KeyCode::KatakanaHiragana => Key::KEY_KATAKANAHIRAGANA.code(),

        // Korean keyboard keys (한국어 키보드)
        KeyCode::Hangeul => Key::KEY_HANGEUL.code(),
        KeyCode::Hanja => Key::KEY_HANJA.code(),

        // ISO/European keyboard keys
        KeyCode::Iso102nd => Key::KEY_102ND.code(),
    }
}
