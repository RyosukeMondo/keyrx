//! Core type definitions for input/output events.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Physical key code representing keyboard keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyCode {
    // Letters A-Z
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,

    // Numbers 0-9 (top row)
    Key0, Key1, Key2, Key3, Key4,
    Key5, Key6, Key7, Key8, Key9,

    // Function keys F1-F12
    F1, F2, F3, F4, F5, F6,
    F7, F8, F9, F10, F11, F12,

    // Modifier keys
    LeftShift, RightShift,
    LeftCtrl, RightCtrl,
    LeftAlt, RightAlt,
    LeftMeta, RightMeta,

    // Navigation keys
    Up, Down, Left, Right,
    Home, End, PageUp, PageDown,

    // Editing keys
    Insert, Delete, Backspace,

    // Whitespace keys
    Space, Tab, Enter,

    // Lock keys
    CapsLock, NumLock, ScrollLock,

    // Escape and Print Screen area
    Escape, PrintScreen, Pause,

    // Punctuation and symbols
    Grave,          // ` ~
    Minus,          // - _
    Equal,          // = +
    LeftBracket,    // [ {
    RightBracket,   // ] }
    Backslash,      // \ |
    Semicolon,      // ; :
    Apostrophe,     // ' "
    Comma,          // , <
    Period,         // . >
    Slash,          // / ?

    // Numpad keys
    Numpad0, Numpad1, Numpad2, Numpad3, Numpad4,
    Numpad5, Numpad6, Numpad7, Numpad8, Numpad9,
    NumpadAdd, NumpadSubtract, NumpadMultiply, NumpadDivide,
    NumpadEnter, NumpadDecimal,

    // Media keys
    VolumeUp, VolumeDown, VolumeMute,
    MediaPlayPause, MediaStop, MediaNext, MediaPrev,

    // Unknown key with raw scan code
    Unknown(u16),
}

impl KeyCode {
    /// Parse a key code from a string name.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::from_str(name).ok()
    }

    /// Get the string name of this key code.
    pub fn name(&self) -> String {
        format!("{self}")
    }
}

impl fmt::Display for KeyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Letters
            Self::A => write!(f, "A"),
            Self::B => write!(f, "B"),
            Self::C => write!(f, "C"),
            Self::D => write!(f, "D"),
            Self::E => write!(f, "E"),
            Self::F => write!(f, "F"),
            Self::G => write!(f, "G"),
            Self::H => write!(f, "H"),
            Self::I => write!(f, "I"),
            Self::J => write!(f, "J"),
            Self::K => write!(f, "K"),
            Self::L => write!(f, "L"),
            Self::M => write!(f, "M"),
            Self::N => write!(f, "N"),
            Self::O => write!(f, "O"),
            Self::P => write!(f, "P"),
            Self::Q => write!(f, "Q"),
            Self::R => write!(f, "R"),
            Self::S => write!(f, "S"),
            Self::T => write!(f, "T"),
            Self::U => write!(f, "U"),
            Self::V => write!(f, "V"),
            Self::W => write!(f, "W"),
            Self::X => write!(f, "X"),
            Self::Y => write!(f, "Y"),
            Self::Z => write!(f, "Z"),
            // Numbers
            Self::Key0 => write!(f, "0"),
            Self::Key1 => write!(f, "1"),
            Self::Key2 => write!(f, "2"),
            Self::Key3 => write!(f, "3"),
            Self::Key4 => write!(f, "4"),
            Self::Key5 => write!(f, "5"),
            Self::Key6 => write!(f, "6"),
            Self::Key7 => write!(f, "7"),
            Self::Key8 => write!(f, "8"),
            Self::Key9 => write!(f, "9"),
            // Function keys
            Self::F1 => write!(f, "F1"),
            Self::F2 => write!(f, "F2"),
            Self::F3 => write!(f, "F3"),
            Self::F4 => write!(f, "F4"),
            Self::F5 => write!(f, "F5"),
            Self::F6 => write!(f, "F6"),
            Self::F7 => write!(f, "F7"),
            Self::F8 => write!(f, "F8"),
            Self::F9 => write!(f, "F9"),
            Self::F10 => write!(f, "F10"),
            Self::F11 => write!(f, "F11"),
            Self::F12 => write!(f, "F12"),
            // Modifiers
            Self::LeftShift => write!(f, "LeftShift"),
            Self::RightShift => write!(f, "RightShift"),
            Self::LeftCtrl => write!(f, "LeftCtrl"),
            Self::RightCtrl => write!(f, "RightCtrl"),
            Self::LeftAlt => write!(f, "LeftAlt"),
            Self::RightAlt => write!(f, "RightAlt"),
            Self::LeftMeta => write!(f, "LeftMeta"),
            Self::RightMeta => write!(f, "RightMeta"),
            // Navigation
            Self::Up => write!(f, "Up"),
            Self::Down => write!(f, "Down"),
            Self::Left => write!(f, "Left"),
            Self::Right => write!(f, "Right"),
            Self::Home => write!(f, "Home"),
            Self::End => write!(f, "End"),
            Self::PageUp => write!(f, "PageUp"),
            Self::PageDown => write!(f, "PageDown"),
            // Editing
            Self::Insert => write!(f, "Insert"),
            Self::Delete => write!(f, "Delete"),
            Self::Backspace => write!(f, "Backspace"),
            // Whitespace
            Self::Space => write!(f, "Space"),
            Self::Tab => write!(f, "Tab"),
            Self::Enter => write!(f, "Enter"),
            // Locks
            Self::CapsLock => write!(f, "CapsLock"),
            Self::NumLock => write!(f, "NumLock"),
            Self::ScrollLock => write!(f, "ScrollLock"),
            // Escape area
            Self::Escape => write!(f, "Escape"),
            Self::PrintScreen => write!(f, "PrintScreen"),
            Self::Pause => write!(f, "Pause"),
            // Punctuation
            Self::Grave => write!(f, "Grave"),
            Self::Minus => write!(f, "Minus"),
            Self::Equal => write!(f, "Equal"),
            Self::LeftBracket => write!(f, "LeftBracket"),
            Self::RightBracket => write!(f, "RightBracket"),
            Self::Backslash => write!(f, "Backslash"),
            Self::Semicolon => write!(f, "Semicolon"),
            Self::Apostrophe => write!(f, "Apostrophe"),
            Self::Comma => write!(f, "Comma"),
            Self::Period => write!(f, "Period"),
            Self::Slash => write!(f, "Slash"),
            // Numpad
            Self::Numpad0 => write!(f, "Numpad0"),
            Self::Numpad1 => write!(f, "Numpad1"),
            Self::Numpad2 => write!(f, "Numpad2"),
            Self::Numpad3 => write!(f, "Numpad3"),
            Self::Numpad4 => write!(f, "Numpad4"),
            Self::Numpad5 => write!(f, "Numpad5"),
            Self::Numpad6 => write!(f, "Numpad6"),
            Self::Numpad7 => write!(f, "Numpad7"),
            Self::Numpad8 => write!(f, "Numpad8"),
            Self::Numpad9 => write!(f, "Numpad9"),
            Self::NumpadAdd => write!(f, "NumpadAdd"),
            Self::NumpadSubtract => write!(f, "NumpadSubtract"),
            Self::NumpadMultiply => write!(f, "NumpadMultiply"),
            Self::NumpadDivide => write!(f, "NumpadDivide"),
            Self::NumpadEnter => write!(f, "NumpadEnter"),
            Self::NumpadDecimal => write!(f, "NumpadDecimal"),
            // Media
            Self::VolumeUp => write!(f, "VolumeUp"),
            Self::VolumeDown => write!(f, "VolumeDown"),
            Self::VolumeMute => write!(f, "VolumeMute"),
            Self::MediaPlayPause => write!(f, "MediaPlayPause"),
            Self::MediaStop => write!(f, "MediaStop"),
            Self::MediaNext => write!(f, "MediaNext"),
            Self::MediaPrev => write!(f, "MediaPrev"),
            // Unknown
            Self::Unknown(code) => write!(f, "Unknown({code})"),
        }
    }
}

impl FromStr for KeyCode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s_upper = s.to_uppercase();
        match s_upper.as_str() {
            // Letters
            "A" => Ok(Self::A),
            "B" => Ok(Self::B),
            "C" => Ok(Self::C),
            "D" => Ok(Self::D),
            "E" => Ok(Self::E),
            "F" => Ok(Self::F),
            "G" => Ok(Self::G),
            "H" => Ok(Self::H),
            "I" => Ok(Self::I),
            "J" => Ok(Self::J),
            "K" => Ok(Self::K),
            "L" => Ok(Self::L),
            "M" => Ok(Self::M),
            "N" => Ok(Self::N),
            "O" => Ok(Self::O),
            "P" => Ok(Self::P),
            "Q" => Ok(Self::Q),
            "R" => Ok(Self::R),
            "S" => Ok(Self::S),
            "T" => Ok(Self::T),
            "U" => Ok(Self::U),
            "V" => Ok(Self::V),
            "W" => Ok(Self::W),
            "X" => Ok(Self::X),
            "Y" => Ok(Self::Y),
            "Z" => Ok(Self::Z),
            // Numbers
            "0" | "KEY0" => Ok(Self::Key0),
            "1" | "KEY1" => Ok(Self::Key1),
            "2" | "KEY2" => Ok(Self::Key2),
            "3" | "KEY3" => Ok(Self::Key3),
            "4" | "KEY4" => Ok(Self::Key4),
            "5" | "KEY5" => Ok(Self::Key5),
            "6" | "KEY6" => Ok(Self::Key6),
            "7" | "KEY7" => Ok(Self::Key7),
            "8" | "KEY8" => Ok(Self::Key8),
            "9" | "KEY9" => Ok(Self::Key9),
            // Function keys
            "F1" => Ok(Self::F1),
            "F2" => Ok(Self::F2),
            "F3" => Ok(Self::F3),
            "F4" => Ok(Self::F4),
            "F5" => Ok(Self::F5),
            "F6" => Ok(Self::F6),
            "F7" => Ok(Self::F7),
            "F8" => Ok(Self::F8),
            "F9" => Ok(Self::F9),
            "F10" => Ok(Self::F10),
            "F11" => Ok(Self::F11),
            "F12" => Ok(Self::F12),
            // Modifiers
            "LEFTSHIFT" | "LSHIFT" | "SHIFT" => Ok(Self::LeftShift),
            "RIGHTSHIFT" | "RSHIFT" => Ok(Self::RightShift),
            "LEFTCTRL" | "LCTRL" | "CTRL" | "CONTROL" => Ok(Self::LeftCtrl),
            "RIGHTCTRL" | "RCTRL" => Ok(Self::RightCtrl),
            "LEFTALT" | "LALT" | "ALT" => Ok(Self::LeftAlt),
            "RIGHTALT" | "RALT" | "ALTGR" => Ok(Self::RightAlt),
            "LEFTMETA" | "LMETA" | "META" | "WIN" | "SUPER" | "CMD" => Ok(Self::LeftMeta),
            "RIGHTMETA" | "RMETA" | "RWIN" | "RSUPER" | "RCMD" => Ok(Self::RightMeta),
            // Navigation
            "UP" | "UPARROW" => Ok(Self::Up),
            "DOWN" | "DOWNARROW" => Ok(Self::Down),
            "LEFT" | "LEFTARROW" => Ok(Self::Left),
            "RIGHT" | "RIGHTARROW" => Ok(Self::Right),
            "HOME" => Ok(Self::Home),
            "END" => Ok(Self::End),
            "PAGEUP" | "PGUP" => Ok(Self::PageUp),
            "PAGEDOWN" | "PGDN" => Ok(Self::PageDown),
            // Editing
            "INSERT" | "INS" => Ok(Self::Insert),
            "DELETE" | "DEL" => Ok(Self::Delete),
            "BACKSPACE" | "BACK" | "BS" => Ok(Self::Backspace),
            // Whitespace
            "SPACE" | "SPACEBAR" => Ok(Self::Space),
            "TAB" => Ok(Self::Tab),
            "ENTER" | "RETURN" => Ok(Self::Enter),
            // Locks
            "CAPSLOCK" | "CAPS" => Ok(Self::CapsLock),
            "NUMLOCK" | "NUM" => Ok(Self::NumLock),
            "SCROLLLOCK" | "SCROLL" => Ok(Self::ScrollLock),
            // Escape area
            "ESCAPE" | "ESC" => Ok(Self::Escape),
            "PRINTSCREEN" | "PRINT" | "PRTSC" => Ok(Self::PrintScreen),
            "PAUSE" | "BREAK" => Ok(Self::Pause),
            // Punctuation
            "GRAVE" | "BACKTICK" | "TILDE" => Ok(Self::Grave),
            "MINUS" | "DASH" => Ok(Self::Minus),
            "EQUAL" | "EQUALS" => Ok(Self::Equal),
            "LEFTBRACKET" | "LBRACKET" => Ok(Self::LeftBracket),
            "RIGHTBRACKET" | "RBRACKET" => Ok(Self::RightBracket),
            "BACKSLASH" => Ok(Self::Backslash),
            "SEMICOLON" => Ok(Self::Semicolon),
            "APOSTROPHE" | "QUOTE" => Ok(Self::Apostrophe),
            "COMMA" => Ok(Self::Comma),
            "PERIOD" | "DOT" => Ok(Self::Period),
            "SLASH" => Ok(Self::Slash),
            // Numpad
            "NUMPAD0" | "KP0" => Ok(Self::Numpad0),
            "NUMPAD1" | "KP1" => Ok(Self::Numpad1),
            "NUMPAD2" | "KP2" => Ok(Self::Numpad2),
            "NUMPAD3" | "KP3" => Ok(Self::Numpad3),
            "NUMPAD4" | "KP4" => Ok(Self::Numpad4),
            "NUMPAD5" | "KP5" => Ok(Self::Numpad5),
            "NUMPAD6" | "KP6" => Ok(Self::Numpad6),
            "NUMPAD7" | "KP7" => Ok(Self::Numpad7),
            "NUMPAD8" | "KP8" => Ok(Self::Numpad8),
            "NUMPAD9" | "KP9" => Ok(Self::Numpad9),
            "NUMPADADD" | "KPADD" | "KPPLUS" => Ok(Self::NumpadAdd),
            "NUMPADSUBTRACT" | "KPSUB" | "KPMINUS" => Ok(Self::NumpadSubtract),
            "NUMPADMULTIPLY" | "KPMUL" | "KPASTERISK" => Ok(Self::NumpadMultiply),
            "NUMPADDIVIDE" | "KPDIV" | "KPSLASH" => Ok(Self::NumpadDivide),
            "NUMPADENTER" | "KPENTER" => Ok(Self::NumpadEnter),
            "NUMPADDECIMAL" | "KPDECIMAL" | "KPDOT" => Ok(Self::NumpadDecimal),
            // Media
            "VOLUMEUP" | "VOLUP" => Ok(Self::VolumeUp),
            "VOLUMEDOWN" | "VOLDOWN" => Ok(Self::VolumeDown),
            "VOLUMEMUTE" | "MUTE" => Ok(Self::VolumeMute),
            "MEDIAPLAYPAUSE" | "PLAYPAUSE" | "PLAY" => Ok(Self::MediaPlayPause),
            "MEDIASTOP" | "STOP" => Ok(Self::MediaStop),
            "MEDIANEXT" | "NEXT" | "NEXTTRACK" => Ok(Self::MediaNext),
            "MEDIAPREV" | "PREV" | "PREVTRACK" => Ok(Self::MediaPrev),
            _ => Err(format!("Unknown key: {s}")),
        }
    }
}

/// Action to take when a key is pressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RemapAction {
    /// Remap this key to another key.
    Remap(KeyCode),
    /// Block this key (consume it, don't pass through).
    Block,
    /// Pass this key through unchanged.
    Pass,
}

impl Default for RemapAction {
    fn default() -> Self {
        Self::Pass
    }
}

/// Input event from keyboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputEvent {
    /// Key that was pressed/released.
    pub key: KeyCode,
    /// True if key down, false if key up.
    pub pressed: bool,
    /// Timestamp in microseconds.
    pub timestamp: u64,
}

/// Output action to send to OS.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputAction {
    /// Press a key.
    KeyDown(KeyCode),
    /// Release a key.
    KeyUp(KeyCode),
    /// Press and release a key.
    KeyTap(KeyCode),
    /// Block the original input (consume it).
    Block,
    /// Pass through the original input unchanged.
    PassThrough,
}

impl InputEvent {
    /// Create a new key down event.
    pub fn key_down(key: KeyCode, timestamp: u64) -> Self {
        Self {
            key,
            pressed: true,
            timestamp,
        }
    }

    /// Create a new key up event.
    pub fn key_up(key: KeyCode, timestamp: u64) -> Self {
        Self {
            key,
            pressed: false,
            timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn keycode_can_be_hashmap_key() {
        let mut map: HashMap<KeyCode, RemapAction> = HashMap::new();
        map.insert(KeyCode::A, RemapAction::Remap(KeyCode::B));
        map.insert(KeyCode::CapsLock, RemapAction::Block);

        assert_eq!(map.get(&KeyCode::A), Some(&RemapAction::Remap(KeyCode::B)));
        assert_eq!(map.get(&KeyCode::CapsLock), Some(&RemapAction::Block));
        assert_eq!(map.get(&KeyCode::Z), None);
    }

    #[test]
    fn keycode_from_str() {
        assert_eq!(KeyCode::from_str("A"), Ok(KeyCode::A));
        assert_eq!(KeyCode::from_str("a"), Ok(KeyCode::A));
        assert_eq!(KeyCode::from_str("CapsLock"), Ok(KeyCode::CapsLock));
        assert_eq!(KeyCode::from_str("caps"), Ok(KeyCode::CapsLock));
        assert_eq!(KeyCode::from_str("Escape"), Ok(KeyCode::Escape));
        assert_eq!(KeyCode::from_str("esc"), Ok(KeyCode::Escape));
        assert_eq!(KeyCode::from_str("F1"), Ok(KeyCode::F1));
        assert_eq!(KeyCode::from_str("1"), Ok(KeyCode::Key1));
        assert!(KeyCode::from_str("InvalidKey").is_err());
    }

    #[test]
    fn keycode_display() {
        assert_eq!(KeyCode::A.to_string(), "A");
        assert_eq!(KeyCode::CapsLock.to_string(), "CapsLock");
        assert_eq!(KeyCode::F1.to_string(), "F1");
        assert_eq!(KeyCode::Unknown(999).to_string(), "Unknown(999)");
    }

    #[test]
    fn remap_action_default_is_pass() {
        assert_eq!(RemapAction::default(), RemapAction::Pass);
    }

    #[test]
    fn keycode_traits() {
        // Test Copy
        let key = KeyCode::A;
        let key_copy = key;
        assert_eq!(key, key_copy);

        // Test Clone
        let key_clone = key.clone();
        assert_eq!(key, key_clone);

        // Test Eq
        assert_eq!(KeyCode::A, KeyCode::A);
        assert_ne!(KeyCode::A, KeyCode::B);

        // Test Hash via HashMap
        let mut map = HashMap::new();
        map.insert(KeyCode::A, 1);
        assert_eq!(map.get(&KeyCode::A), Some(&1));
    }
}
