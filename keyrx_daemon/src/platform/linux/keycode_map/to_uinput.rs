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
        KeyCode::Menu => Keyboard::Misc(Misc::Compose),
        KeyCode::Help => Keyboard::Misc(Misc::Help),
        KeyCode::Select => Keyboard::Misc(Misc::Select),
        KeyCode::Execute => Keyboard::Misc(Misc::Open),
        KeyCode::Undo => Keyboard::Misc(Misc::Undo),
        KeyCode::Redo => Keyboard::Misc(Misc::Redo),
        KeyCode::Cut => Keyboard::Misc(Misc::Cut),
        KeyCode::Copy => Keyboard::Misc(Misc::Copy),
        KeyCode::Paste => Keyboard::Misc(Misc::Paste),
        KeyCode::Find => Keyboard::Misc(Misc::Find),

        // Japanese JIS keyboard keys (日本語キーボード)
        // Note: uinput may not have direct support for all Japanese keys,
        // fallback to raw key injection via evdev codes in platform layer
        KeyCode::Zenkaku => Keyboard::Misc(Misc::ZenkakuHankaku),
        KeyCode::Katakana => Keyboard::Misc(Misc::Katakana),
        KeyCode::Hiragana => Keyboard::Misc(Misc::Hiragana),
        KeyCode::Henkan => Keyboard::Misc(Misc::Henkan),
        KeyCode::Muhenkan => Keyboard::Misc(Misc::Muhenkan),
        KeyCode::Yen => Keyboard::Misc(Misc::Yen),
        KeyCode::Ro => Keyboard::Misc(Misc::RO),
        KeyCode::KatakanaHiragana => Keyboard::Misc(Misc::KatakanaHiragana),

        // Korean keyboard keys (한국어 키보드)
        KeyCode::Hangeul => Keyboard::Misc(Misc::Hangeul),
        KeyCode::Hanja => Keyboard::Misc(Misc::Hanja),

        // ISO/European keyboard keys
        KeyCode::Iso102nd => Keyboard::Misc(Misc::ND102),
    }
}
