macro_rules! define_keycodes {
    (
        $(
            $variant:ident => $display:literal, $evdev:expr, $vk:expr, [$($alias:literal),* $(,)?]
        ),* $(,)?
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub enum KeyCode {
            $(
                $variant,
            )*
            Unknown(u16),
        }
        impl KeyCode {
            #[inline]
            pub fn from_name(name: &str) -> Option<Self> {
                Self::from_str(name).ok()
            }
            #[inline]
            pub fn name(&self) -> String {
                format!("{self}")
            }
        }
        impl fmt::Display for KeyCode {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $(
                        Self::$variant => write!(f, $display),
                    )*
                    Self::Unknown(code) => write!(f, "Unknown({code})"),
                }
            }
        }
        impl FromStr for KeyCode {
            type Err = String;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let s_upper = s.to_uppercase();
                match s_upper.as_str() {
                    $(
                        $($alias)|* => Ok(Self::$variant),
                    )*
                    _ => Err(format!("Unknown key: {s}")),
                }
            }
        }
        #[cfg(target_os = "linux")]
        pub fn evdev_to_keycode(code: u16) -> KeyCode {
            match code {
                $(
                    $evdev => KeyCode::$variant,
                )*
                _ => KeyCode::Unknown(code),
            }
        }
        #[cfg(target_os = "linux")]
        pub fn keycode_to_evdev(key: KeyCode) -> u16 {
            match key {
                $(
                    KeyCode::$variant => $evdev,
                )*
                KeyCode::Unknown(code) => code,
            }
        }
        #[allow(unreachable_patterns)]
        pub fn vk_to_keycode(vk: u16) -> KeyCode {
            // Letters (0x41-0x5A)
            if (0x41..=0x5A).contains(&vk) {
                return vk_to_keycode_letters(vk);
            }
            // Numbers (0x30-0x39)
            if (0x30..=0x39).contains(&vk) {
                return vk_to_keycode_numbers(vk);
            }
            // Function keys (0x70-0x7B)
            if (0x70..=0x7B).contains(&vk) {
                return vk_to_keycode_function_keys(vk);
            }
            // Modifiers (0xA0-0xA5, 0x5B-0x5C)
            if (0xA0..=0xA5).contains(&vk) || (0x5B..=0x5C).contains(&vk) {
                return vk_to_keycode_modifiers(vk);
            }
            // Navigation (0x21-0x28)
            if (0x21..=0x28).contains(&vk) {
                return vk_to_keycode_navigation(vk);
            }
            // Numpad (0x60-0x6F)
            if (0x60..=0x6F).contains(&vk) {
                return vk_to_keycode_numpad(vk);
            }
            // Media (0xAD-0xB3)
            if (0xAD..=0xB3).contains(&vk) {
                return vk_to_keycode_media(vk);
            }
            // Remaining keys
            vk_to_keycode_other(vk)
        }
        #[inline]
        fn vk_to_keycode_letters(vk: u16) -> KeyCode {
            match vk {
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
                _ => KeyCode::Unknown(vk),
            }
        }
        #[inline]
        fn vk_to_keycode_numbers(vk: u16) -> KeyCode {
            match vk {
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
                _ => KeyCode::Unknown(vk),
            }
        }
        #[inline]
        fn vk_to_keycode_function_keys(vk: u16) -> KeyCode {
            match vk {
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
                _ => KeyCode::Unknown(vk),
            }
        }
        #[inline]
        fn vk_to_keycode_modifiers(vk: u16) -> KeyCode {
            match vk {
                0xA0 => KeyCode::LeftShift,
                0xA1 => KeyCode::RightShift,
                0xA2 => KeyCode::LeftCtrl,
                0xA3 => KeyCode::RightCtrl,
                0xA4 => KeyCode::LeftAlt,
                0xA5 => KeyCode::RightAlt,
                0x5B => KeyCode::LeftMeta,
                0x5C => KeyCode::RightMeta,
                _ => KeyCode::Unknown(vk),
            }
        }
        #[inline]
        fn vk_to_keycode_navigation(vk: u16) -> KeyCode {
            match vk {
                0x26 => KeyCode::Up,
                0x28 => KeyCode::Down,
                0x25 => KeyCode::Left,
                0x27 => KeyCode::Right,
                0x24 => KeyCode::Home,
                0x23 => KeyCode::End,
                0x21 => KeyCode::PageUp,
                0x22 => KeyCode::PageDown,
                _ => KeyCode::Unknown(vk),
            }
        }
        #[inline]
        fn vk_to_keycode_numpad(vk: u16) -> KeyCode {
            match vk {
                0x60 => KeyCode::Numpad0,
                0x61 => KeyCode::Numpad1,
                0x62 => KeyCode::Numpad2,
                0x63 => KeyCode::Numpad3,
                0x64 => KeyCode::Numpad4,
                0x65 => KeyCode::Numpad5,
                0x66 => KeyCode::Numpad6,
                0x67 => KeyCode::Numpad7,
                0x68 => KeyCode::Numpad8,
                0x69 => KeyCode::Numpad9,
                0x6B => KeyCode::NumpadAdd,
                0x6D => KeyCode::NumpadSubtract,
                0x6A => KeyCode::NumpadMultiply,
                0x6F => KeyCode::NumpadDivide,
                0x6E => KeyCode::NumpadDecimal,
                _ => KeyCode::Unknown(vk),
            }
        }
        #[inline]
        fn vk_to_keycode_media(vk: u16) -> KeyCode {
            match vk {
                0xAF => KeyCode::VolumeUp,
                0xAE => KeyCode::VolumeDown,
                0xAD => KeyCode::VolumeMute,
                0xB3 => KeyCode::MediaPlayPause,
                0xB2 => KeyCode::MediaStop,
                0xB0 => KeyCode::MediaNext,
                0xB1 => KeyCode::MediaPrev,
                _ => KeyCode::Unknown(vk),
            }
        }
        #[inline]
        fn vk_to_keycode_other(vk: u16) -> KeyCode {
            match vk {
                0x2D => KeyCode::Insert,
                0x2E => KeyCode::Delete,
                0x08 => KeyCode::Backspace,
                0x20 => KeyCode::Space,
                0x09 => KeyCode::Tab,
                0x0D => KeyCode::Enter,
                0x14 => KeyCode::CapsLock,
                0x90 => KeyCode::NumLock,
                0x91 => KeyCode::ScrollLock,
                0x1B => KeyCode::Escape,
                0x2C => KeyCode::PrintScreen,
                0x13 => KeyCode::Pause,
                0xC0 => KeyCode::Grave,
                0xBD => KeyCode::Minus,
                0xBB => KeyCode::Equal,
                0xDB => KeyCode::LeftBracket,
                0xDD => KeyCode::RightBracket,
                0xDC => KeyCode::Backslash,
                0xBA => KeyCode::Semicolon,
                0xDE => KeyCode::Apostrophe,
                0xBC => KeyCode::Comma,
                0xBE => KeyCode::Period,
                0xBF => KeyCode::Slash,
                _ => KeyCode::Unknown(vk),
            }
        }
        pub fn keycode_to_vk(key: KeyCode) -> u16 {
            match key {
                // Letters
                KeyCode::A | KeyCode::B | KeyCode::C | KeyCode::D | KeyCode::E |
                KeyCode::F | KeyCode::G | KeyCode::H | KeyCode::I | KeyCode::J |
                KeyCode::K | KeyCode::L | KeyCode::M | KeyCode::N | KeyCode::O |
                KeyCode::P | KeyCode::Q | KeyCode::R | KeyCode::S | KeyCode::T |
                KeyCode::U | KeyCode::V | KeyCode::W | KeyCode::X | KeyCode::Y |
                KeyCode::Z => keycode_to_vk_letters(key),
                // Numbers
                KeyCode::Key0 | KeyCode::Key1 | KeyCode::Key2 | KeyCode::Key3 | KeyCode::Key4 |
                KeyCode::Key5 | KeyCode::Key6 | KeyCode::Key7 | KeyCode::Key8 | KeyCode::Key9 => {
                    keycode_to_vk_numbers(key)
                }
                // Function keys
                KeyCode::F1 | KeyCode::F2 | KeyCode::F3 | KeyCode::F4 | KeyCode::F5 | KeyCode::F6 |
                KeyCode::F7 | KeyCode::F8 | KeyCode::F9 | KeyCode::F10 | KeyCode::F11 | KeyCode::F12 => {
                    keycode_to_vk_function_keys(key)
                }
                // Modifiers
                KeyCode::LeftShift | KeyCode::RightShift |
                KeyCode::LeftCtrl | KeyCode::RightCtrl |
                KeyCode::LeftAlt | KeyCode::RightAlt |
                KeyCode::LeftMeta | KeyCode::RightMeta => keycode_to_vk_modifiers(key),
                // Navigation
                KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right |
                KeyCode::Home | KeyCode::End | KeyCode::PageUp | KeyCode::PageDown => {
                    keycode_to_vk_navigation(key)
                }
                // Numpad
                KeyCode::Numpad0 | KeyCode::Numpad1 | KeyCode::Numpad2 | KeyCode::Numpad3 |
                KeyCode::Numpad4 | KeyCode::Numpad5 | KeyCode::Numpad6 | KeyCode::Numpad7 |
                KeyCode::Numpad8 | KeyCode::Numpad9 | KeyCode::NumpadAdd | KeyCode::NumpadSubtract |
                KeyCode::NumpadMultiply | KeyCode::NumpadDivide | KeyCode::NumpadEnter |
                KeyCode::NumpadDecimal => keycode_to_vk_numpad(key),
                // Media
                KeyCode::VolumeUp | KeyCode::VolumeDown | KeyCode::VolumeMute |
                KeyCode::MediaPlayPause | KeyCode::MediaStop | KeyCode::MediaNext |
                KeyCode::MediaPrev => keycode_to_vk_media(key),
                // Other keys
                _ => keycode_to_vk_other(key),
            }
        }
        #[inline]
        fn keycode_to_vk_letters(key: KeyCode) -> u16 {
            match key {
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
                _ => 0,
            }
        }
        #[inline]
        fn keycode_to_vk_numbers(key: KeyCode) -> u16 {
            match key {
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
                _ => 0,
            }
        }
        #[inline]
        fn keycode_to_vk_function_keys(key: KeyCode) -> u16 {
            match key {
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
                _ => 0,
            }
        }
        #[inline]
        fn keycode_to_vk_modifiers(key: KeyCode) -> u16 {
            match key {
                KeyCode::LeftShift => 0xA0,
                KeyCode::RightShift => 0xA1,
                KeyCode::LeftCtrl => 0xA2,
                KeyCode::RightCtrl => 0xA3,
                KeyCode::LeftAlt => 0xA4,
                KeyCode::RightAlt => 0xA5,
                KeyCode::LeftMeta => 0x5B,
                KeyCode::RightMeta => 0x5C,
                _ => 0,
            }
        }
        #[inline]
        fn keycode_to_vk_navigation(key: KeyCode) -> u16 {
            match key {
                KeyCode::Up => 0x26,
                KeyCode::Down => 0x28,
                KeyCode::Left => 0x25,
                KeyCode::Right => 0x27,
                KeyCode::Home => 0x24,
                KeyCode::End => 0x23,
                KeyCode::PageUp => 0x21,
                KeyCode::PageDown => 0x22,
                _ => 0,
            }
        }
        #[inline]
        fn keycode_to_vk_numpad(key: KeyCode) -> u16 {
            match key {
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
                KeyCode::NumpadEnter => 0x0D,
                KeyCode::NumpadDecimal => 0x6E,
                _ => 0,
            }
        }
        #[inline]
        fn keycode_to_vk_media(key: KeyCode) -> u16 {
            match key {
                KeyCode::VolumeUp => 0xAF,
                KeyCode::VolumeDown => 0xAE,
                KeyCode::VolumeMute => 0xAD,
                KeyCode::MediaPlayPause => 0xB3,
                KeyCode::MediaStop => 0xB2,
                KeyCode::MediaNext => 0xB0,
                KeyCode::MediaPrev => 0xB1,
                _ => 0,
            }
        }
        #[inline]
        fn keycode_to_vk_other(key: KeyCode) -> u16 {
            match key {
                KeyCode::Insert => 0x2D,
                KeyCode::Delete => 0x2E,
                KeyCode::Backspace => 0x08,
                KeyCode::Space => 0x20,
                KeyCode::Tab => 0x09,
                KeyCode::Enter => 0x0D,
                KeyCode::CapsLock => 0x14,
                KeyCode::NumLock => 0x90,
                KeyCode::ScrollLock => 0x91,
                KeyCode::Escape => 0x1B,
                KeyCode::PrintScreen => 0x2C,
                KeyCode::Pause => 0x13,
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
                KeyCode::Unknown(vk) => vk,
                _ => 0,
            }
        }
        pub fn all_keycodes() -> Vec<KeyCode> {
            vec![
                $(
                    KeyCode::$variant,
                )*
            ]
        }
        #[cfg(target_os = "linux")]
        pub fn all_evdev_codes() -> Vec<u16> {
            vec![
                $(
                    $evdev,
                )*
            ]
        }
    };
}
pub(crate) use define_keycodes;
