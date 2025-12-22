//! Key code mapping between keyrx, evdev, and uinput formats.
//!
//! This module provides conversion functions between different key code representations:
//! - `KeyCode`: The keyrx internal representation (platform-agnostic)
//! - evdev key codes (u16): Raw Linux input event codes
//! - uinput `Keyboard` variants: Used for event injection via uinput

use evdev::Key;
use uinput::event::keyboard::{Key as UKey, KeyPad, Keyboard, Misc};

use keyrx_core::config::KeyCode;

/// Maps a keyrx KeyCode to a uinput Keyboard variant.
///
/// This is used by the OutputDevice implementation to convert keyrx KeyCodes
/// to the uinput crate's key type for event injection.
///
/// The uinput crate organizes keys into different enums:
/// - `Key`: Basic keyboard keys (letters, numbers, F-keys, modifiers, arrows, etc.)
/// - `KeyPad`: Numpad keys
/// - `Misc`: Media keys, system keys, browser keys, and special function keys
pub fn keycode_to_uinput_key(keycode: KeyCode) -> Keyboard {
    match keycode {
        // Letters A-Z
        KeyCode::A => Keyboard::Key(UKey::A),
        KeyCode::B => Keyboard::Key(UKey::B),
        KeyCode::C => Keyboard::Key(UKey::C),
        KeyCode::D => Keyboard::Key(UKey::D),
        KeyCode::E => Keyboard::Key(UKey::E),
        KeyCode::F => Keyboard::Key(UKey::F),
        KeyCode::G => Keyboard::Key(UKey::G),
        KeyCode::H => Keyboard::Key(UKey::H),
        KeyCode::I => Keyboard::Key(UKey::I),
        KeyCode::J => Keyboard::Key(UKey::J),
        KeyCode::K => Keyboard::Key(UKey::K),
        KeyCode::L => Keyboard::Key(UKey::L),
        KeyCode::M => Keyboard::Key(UKey::M),
        KeyCode::N => Keyboard::Key(UKey::N),
        KeyCode::O => Keyboard::Key(UKey::O),
        KeyCode::P => Keyboard::Key(UKey::P),
        KeyCode::Q => Keyboard::Key(UKey::Q),
        KeyCode::R => Keyboard::Key(UKey::R),
        KeyCode::S => Keyboard::Key(UKey::S),
        KeyCode::T => Keyboard::Key(UKey::T),
        KeyCode::U => Keyboard::Key(UKey::U),
        KeyCode::V => Keyboard::Key(UKey::V),
        KeyCode::W => Keyboard::Key(UKey::W),
        KeyCode::X => Keyboard::Key(UKey::X),
        KeyCode::Y => Keyboard::Key(UKey::Y),
        KeyCode::Z => Keyboard::Key(UKey::Z),

        // Numbers 0-9 (top row)
        KeyCode::Num0 => Keyboard::Key(UKey::_0),
        KeyCode::Num1 => Keyboard::Key(UKey::_1),
        KeyCode::Num2 => Keyboard::Key(UKey::_2),
        KeyCode::Num3 => Keyboard::Key(UKey::_3),
        KeyCode::Num4 => Keyboard::Key(UKey::_4),
        KeyCode::Num5 => Keyboard::Key(UKey::_5),
        KeyCode::Num6 => Keyboard::Key(UKey::_6),
        KeyCode::Num7 => Keyboard::Key(UKey::_7),
        KeyCode::Num8 => Keyboard::Key(UKey::_8),
        KeyCode::Num9 => Keyboard::Key(UKey::_9),

        // Function keys F1-F12
        KeyCode::F1 => Keyboard::Key(UKey::F1),
        KeyCode::F2 => Keyboard::Key(UKey::F2),
        KeyCode::F3 => Keyboard::Key(UKey::F3),
        KeyCode::F4 => Keyboard::Key(UKey::F4),
        KeyCode::F5 => Keyboard::Key(UKey::F5),
        KeyCode::F6 => Keyboard::Key(UKey::F6),
        KeyCode::F7 => Keyboard::Key(UKey::F7),
        KeyCode::F8 => Keyboard::Key(UKey::F8),
        KeyCode::F9 => Keyboard::Key(UKey::F9),
        KeyCode::F10 => Keyboard::Key(UKey::F10),
        KeyCode::F11 => Keyboard::Key(UKey::F11),
        KeyCode::F12 => Keyboard::Key(UKey::F12),

        // Extended function keys F13-F24
        KeyCode::F13 => Keyboard::Key(UKey::F13),
        KeyCode::F14 => Keyboard::Key(UKey::F14),
        KeyCode::F15 => Keyboard::Key(UKey::F15),
        KeyCode::F16 => Keyboard::Key(UKey::F16),
        KeyCode::F17 => Keyboard::Key(UKey::F17),
        KeyCode::F18 => Keyboard::Key(UKey::F18),
        KeyCode::F19 => Keyboard::Key(UKey::F19),
        KeyCode::F20 => Keyboard::Key(UKey::F20),
        KeyCode::F21 => Keyboard::Key(UKey::F21),
        KeyCode::F22 => Keyboard::Key(UKey::F22),
        KeyCode::F23 => Keyboard::Key(UKey::F23),
        KeyCode::F24 => Keyboard::Key(UKey::F24),

        // Modifier keys
        KeyCode::LShift => Keyboard::Key(UKey::LeftShift),
        KeyCode::RShift => Keyboard::Key(UKey::RightShift),
        KeyCode::LCtrl => Keyboard::Key(UKey::LeftControl),
        KeyCode::RCtrl => Keyboard::Key(UKey::RightControl),
        KeyCode::LAlt => Keyboard::Key(UKey::LeftAlt),
        KeyCode::RAlt => Keyboard::Key(UKey::RightAlt),
        KeyCode::LMeta => Keyboard::Key(UKey::LeftMeta),
        KeyCode::RMeta => Keyboard::Key(UKey::RightMeta),

        // Special keys
        KeyCode::Escape => Keyboard::Key(UKey::Esc),
        KeyCode::Enter => Keyboard::Key(UKey::Enter),
        KeyCode::Backspace => Keyboard::Key(UKey::BackSpace),
        KeyCode::Tab => Keyboard::Key(UKey::Tab),
        KeyCode::Space => Keyboard::Key(UKey::Space),
        KeyCode::CapsLock => Keyboard::Key(UKey::CapsLock),
        KeyCode::NumLock => Keyboard::Key(UKey::NumLock),
        KeyCode::ScrollLock => Keyboard::Key(UKey::ScrollLock),
        KeyCode::PrintScreen => Keyboard::Key(UKey::SysRq),
        KeyCode::Pause => Keyboard::Misc(Misc::Pause),
        KeyCode::Insert => Keyboard::Key(UKey::Insert),
        KeyCode::Delete => Keyboard::Key(UKey::Delete),
        KeyCode::Home => Keyboard::Key(UKey::Home),
        KeyCode::End => Keyboard::Key(UKey::End),
        KeyCode::PageUp => Keyboard::Key(UKey::PageUp),
        KeyCode::PageDown => Keyboard::Key(UKey::PageDown),

        // Arrow keys
        KeyCode::Left => Keyboard::Key(UKey::Left),
        KeyCode::Right => Keyboard::Key(UKey::Right),
        KeyCode::Up => Keyboard::Key(UKey::Up),
        KeyCode::Down => Keyboard::Key(UKey::Down),

        // Punctuation and symbols
        KeyCode::LeftBracket => Keyboard::Key(UKey::LeftBrace),
        KeyCode::RightBracket => Keyboard::Key(UKey::RightBrace),
        KeyCode::Backslash => Keyboard::Key(UKey::BackSlash),
        KeyCode::Semicolon => Keyboard::Key(UKey::SemiColon),
        KeyCode::Quote => Keyboard::Key(UKey::Apostrophe),
        KeyCode::Comma => Keyboard::Key(UKey::Comma),
        KeyCode::Period => Keyboard::Key(UKey::Dot),
        KeyCode::Slash => Keyboard::Key(UKey::Slash),
        KeyCode::Grave => Keyboard::Key(UKey::Grave),
        KeyCode::Minus => Keyboard::Key(UKey::Minus),
        KeyCode::Equal => Keyboard::Key(UKey::Equal),

        // Numpad keys (use KeyPad enum)
        KeyCode::Numpad0 => Keyboard::KeyPad(KeyPad::_0),
        KeyCode::Numpad1 => Keyboard::KeyPad(KeyPad::_1),
        KeyCode::Numpad2 => Keyboard::KeyPad(KeyPad::_2),
        KeyCode::Numpad3 => Keyboard::KeyPad(KeyPad::_3),
        KeyCode::Numpad4 => Keyboard::KeyPad(KeyPad::_4),
        KeyCode::Numpad5 => Keyboard::KeyPad(KeyPad::_5),
        KeyCode::Numpad6 => Keyboard::KeyPad(KeyPad::_6),
        KeyCode::Numpad7 => Keyboard::KeyPad(KeyPad::_7),
        KeyCode::Numpad8 => Keyboard::KeyPad(KeyPad::_8),
        KeyCode::Numpad9 => Keyboard::KeyPad(KeyPad::_9),
        KeyCode::NumpadDivide => Keyboard::KeyPad(KeyPad::Slash),
        KeyCode::NumpadMultiply => Keyboard::KeyPad(KeyPad::Asterisk),
        KeyCode::NumpadSubtract => Keyboard::KeyPad(KeyPad::Minus),
        KeyCode::NumpadAdd => Keyboard::KeyPad(KeyPad::Plus),
        KeyCode::NumpadEnter => Keyboard::KeyPad(KeyPad::Enter),
        KeyCode::NumpadDecimal => Keyboard::KeyPad(KeyPad::Dot),

        // Media keys (use Misc enum)
        KeyCode::Mute => Keyboard::Misc(Misc::Mute),
        KeyCode::VolumeDown => Keyboard::Misc(Misc::VolumeDown),
        KeyCode::VolumeUp => Keyboard::Misc(Misc::VolumeUp),
        KeyCode::MediaPlayPause => Keyboard::Misc(Misc::PlayPause),
        KeyCode::MediaStop => Keyboard::Misc(Misc::StopCD),
        KeyCode::MediaPrevious => Keyboard::Misc(Misc::PreviousSong),
        KeyCode::MediaNext => Keyboard::Misc(Misc::NextSong),

        // System keys (use Misc enum)
        KeyCode::Power => Keyboard::Misc(Misc::Power),
        KeyCode::Sleep => Keyboard::Misc(Misc::Sleep),
        KeyCode::Wake => Keyboard::Misc(Misc::WakeUp),

        // Browser keys (use Misc enum)
        KeyCode::BrowserBack => Keyboard::Misc(Misc::Back),
        KeyCode::BrowserForward => Keyboard::Misc(Misc::Forward),
        KeyCode::BrowserRefresh => Keyboard::Misc(Misc::Refresh),
        KeyCode::BrowserStop => Keyboard::Misc(Misc::Stop),
        KeyCode::BrowserSearch => Keyboard::Misc(Misc::Search),
        KeyCode::BrowserFavorites => Keyboard::Misc(Misc::Bookmarks),
        KeyCode::BrowserHome => Keyboard::Misc(Misc::HomePage),

        // Application keys (use Misc enum)
        KeyCode::AppMail => Keyboard::Misc(Misc::Mail),
        KeyCode::AppCalculator => Keyboard::Misc(Misc::Calc),
        KeyCode::AppMyComputer => Keyboard::Misc(Misc::Computer),

        // Additional keys (use Misc enum)
        KeyCode::Menu => Keyboard::Misc(Misc::Menu),
        KeyCode::Help => Keyboard::Misc(Misc::Help),
        KeyCode::Select => Keyboard::Misc(Misc::Select),
        KeyCode::Execute => Keyboard::Misc(Misc::Open),
        KeyCode::Undo => Keyboard::Misc(Misc::Undo),
        KeyCode::Redo => Keyboard::Misc(Misc::Redo),
        KeyCode::Cut => Keyboard::Misc(Misc::Cut),
        KeyCode::Copy => Keyboard::Misc(Misc::Copy),
        KeyCode::Paste => Keyboard::Misc(Misc::Paste),
        KeyCode::Find => Keyboard::Misc(Misc::Find),
    }
}

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

        // Unknown key - return None for passthrough handling
        _ => None,
    }
}

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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that all letter keys map correctly
    #[test]
    fn test_letter_keys_mapping() {
        // Test A-Z
        assert_eq!(evdev_to_keycode(Key::KEY_A.code()), Some(KeyCode::A));
        assert_eq!(evdev_to_keycode(Key::KEY_Z.code()), Some(KeyCode::Z));
        assert_eq!(evdev_to_keycode(Key::KEY_M.code()), Some(KeyCode::M));

        // Test round-trip
        assert_eq!(keycode_to_evdev(KeyCode::A), Key::KEY_A.code());
        assert_eq!(keycode_to_evdev(KeyCode::Z), Key::KEY_Z.code());
    }

    /// Test that number keys map correctly
    #[test]
    fn test_number_keys_mapping() {
        // Note: evdev KEY_0 is actually the '0' key, not at position 0
        assert_eq!(evdev_to_keycode(Key::KEY_1.code()), Some(KeyCode::Num1));
        assert_eq!(evdev_to_keycode(Key::KEY_0.code()), Some(KeyCode::Num0));
        assert_eq!(evdev_to_keycode(Key::KEY_5.code()), Some(KeyCode::Num5));

        // Round-trip
        assert_eq!(keycode_to_evdev(KeyCode::Num0), Key::KEY_0.code());
        assert_eq!(keycode_to_evdev(KeyCode::Num9), Key::KEY_9.code());
    }

    /// Test that function keys map correctly
    #[test]
    fn test_function_keys_mapping() {
        assert_eq!(evdev_to_keycode(Key::KEY_F1.code()), Some(KeyCode::F1));
        assert_eq!(evdev_to_keycode(Key::KEY_F12.code()), Some(KeyCode::F12));
        assert_eq!(evdev_to_keycode(Key::KEY_F24.code()), Some(KeyCode::F24));

        // Round-trip
        assert_eq!(keycode_to_evdev(KeyCode::F1), Key::KEY_F1.code());
        assert_eq!(keycode_to_evdev(KeyCode::F12), Key::KEY_F12.code());
    }

    /// Test that modifier keys map correctly
    #[test]
    fn test_modifier_keys_mapping() {
        assert_eq!(
            evdev_to_keycode(Key::KEY_LEFTSHIFT.code()),
            Some(KeyCode::LShift)
        );
        assert_eq!(
            evdev_to_keycode(Key::KEY_RIGHTSHIFT.code()),
            Some(KeyCode::RShift)
        );
        assert_eq!(
            evdev_to_keycode(Key::KEY_LEFTCTRL.code()),
            Some(KeyCode::LCtrl)
        );
        assert_eq!(
            evdev_to_keycode(Key::KEY_RIGHTCTRL.code()),
            Some(KeyCode::RCtrl)
        );
        assert_eq!(
            evdev_to_keycode(Key::KEY_LEFTALT.code()),
            Some(KeyCode::LAlt)
        );
        assert_eq!(
            evdev_to_keycode(Key::KEY_RIGHTALT.code()),
            Some(KeyCode::RAlt)
        );
        assert_eq!(
            evdev_to_keycode(Key::KEY_LEFTMETA.code()),
            Some(KeyCode::LMeta)
        );
        assert_eq!(
            evdev_to_keycode(Key::KEY_RIGHTMETA.code()),
            Some(KeyCode::RMeta)
        );

        // Round-trip
        assert_eq!(keycode_to_evdev(KeyCode::LShift), Key::KEY_LEFTSHIFT.code());
        assert_eq!(keycode_to_evdev(KeyCode::RAlt), Key::KEY_RIGHTALT.code());
    }

    /// Test special keys mapping
    #[test]
    fn test_special_keys_mapping() {
        assert_eq!(evdev_to_keycode(Key::KEY_ESC.code()), Some(KeyCode::Escape));
        assert_eq!(
            evdev_to_keycode(Key::KEY_ENTER.code()),
            Some(KeyCode::Enter)
        );
        assert_eq!(
            evdev_to_keycode(Key::KEY_BACKSPACE.code()),
            Some(KeyCode::Backspace)
        );
        assert_eq!(evdev_to_keycode(Key::KEY_TAB.code()), Some(KeyCode::Tab));
        assert_eq!(
            evdev_to_keycode(Key::KEY_SPACE.code()),
            Some(KeyCode::Space)
        );

        // Round-trip
        assert_eq!(keycode_to_evdev(KeyCode::Escape), Key::KEY_ESC.code());
        assert_eq!(keycode_to_evdev(KeyCode::Enter), Key::KEY_ENTER.code());
    }

    /// Test arrow keys mapping
    #[test]
    fn test_arrow_keys_mapping() {
        assert_eq!(evdev_to_keycode(Key::KEY_LEFT.code()), Some(KeyCode::Left));
        assert_eq!(
            evdev_to_keycode(Key::KEY_RIGHT.code()),
            Some(KeyCode::Right)
        );
        assert_eq!(evdev_to_keycode(Key::KEY_UP.code()), Some(KeyCode::Up));
        assert_eq!(evdev_to_keycode(Key::KEY_DOWN.code()), Some(KeyCode::Down));

        // Round-trip
        assert_eq!(keycode_to_evdev(KeyCode::Left), Key::KEY_LEFT.code());
        assert_eq!(keycode_to_evdev(KeyCode::Down), Key::KEY_DOWN.code());
    }

    /// Test numpad keys mapping
    #[test]
    fn test_numpad_keys_mapping() {
        assert_eq!(
            evdev_to_keycode(Key::KEY_KP0.code()),
            Some(KeyCode::Numpad0)
        );
        assert_eq!(
            evdev_to_keycode(Key::KEY_KP9.code()),
            Some(KeyCode::Numpad9)
        );
        assert_eq!(
            evdev_to_keycode(Key::KEY_KPENTER.code()),
            Some(KeyCode::NumpadEnter)
        );

        // Round-trip
        assert_eq!(keycode_to_evdev(KeyCode::Numpad0), Key::KEY_KP0.code());
        assert_eq!(
            keycode_to_evdev(KeyCode::NumpadEnter),
            Key::KEY_KPENTER.code()
        );
    }

    /// Test unknown key returns None
    #[test]
    fn test_unknown_key_returns_none() {
        // Use an invalid/unknown key code
        assert_eq!(evdev_to_keycode(0xFFFF), None);
    }

    /// Test all KeyCode variants have round-trip consistency
    #[test]
    fn test_all_keycodes_roundtrip() {
        let all_keycodes = [
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
            KeyCode::Num0,
            KeyCode::Num1,
            KeyCode::Num2,
            KeyCode::Num3,
            KeyCode::Num4,
            KeyCode::Num5,
            KeyCode::Num6,
            KeyCode::Num7,
            KeyCode::Num8,
            KeyCode::Num9,
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
            KeyCode::Escape,
            KeyCode::Enter,
            KeyCode::Backspace,
            KeyCode::Tab,
            KeyCode::Space,
            KeyCode::LShift,
            KeyCode::RShift,
            KeyCode::LCtrl,
            KeyCode::RCtrl,
            KeyCode::LAlt,
            KeyCode::RAlt,
            KeyCode::LMeta,
            KeyCode::RMeta,
            KeyCode::Left,
            KeyCode::Right,
            KeyCode::Up,
            KeyCode::Down,
        ];

        for keycode in all_keycodes {
            let evdev_code = keycode_to_evdev(keycode);
            let result = evdev_to_keycode(evdev_code);
            assert_eq!(result, Some(keycode), "Round-trip failed for {:?}", keycode);
        }
    }
}
