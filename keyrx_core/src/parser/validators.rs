//! Key name and prefix validation utilities.
//!
//! Provides functions to parse key names (VK_A, MD_00, etc.) into KeyCode values.

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use super::error::ParseError;
use crate::config::{Condition, KeyCode, MAX_LOCK_ID, MAX_MODIFIER_ID};

/// Physical modifier key names that cannot be used as custom MD_ identifiers.
pub const PHYSICAL_MODIFIERS: &[&str] = &[
    "LShift", "RShift", "LCtrl", "RCtrl", "LAlt", "RAlt", "LMeta", "RMeta",
];

/// Parse a physical key name (without VK_ prefix requirement).
/// Used for input keys where prefix is optional.
pub fn parse_physical_key(s: &str) -> Result<KeyCode, ParseError> {
    // If it has VK_ prefix, strip it
    let name = s.strip_prefix("VK_").unwrap_or(s);
    parse_key_name(name)
}

/// Parse a virtual key name (requires VK_ prefix).
/// Used for output keys where prefix is mandatory.
pub fn parse_virtual_key(s: &str) -> Result<KeyCode, ParseError> {
    if !s.starts_with("VK_") {
        return Err(ParseError::MissingPrefix {
            key: s.to_string(),
            context: "virtual key".to_string(),
        });
    }
    parse_key_name(&s[3..])
}

/// Parse a modifier ID (MD_XX format, hex 00-FE).
pub fn parse_modifier_id(s: &str) -> Result<u8, ParseError> {
    if !s.starts_with("MD_") {
        return Err(ParseError::MissingPrefix {
            key: s.to_string(),
            context: "custom modifier".to_string(),
        });
    }
    let id_part = &s[3..];
    if PHYSICAL_MODIFIERS.contains(&id_part) {
        return Err(ParseError::PhysicalModifierInMD {
            name: id_part.to_string(),
        });
    }
    let id = u16::from_str_radix(id_part, 16).map_err(|_| ParseError::InvalidPrefix {
        expected: "MD_XX (hex, 00-FE)".to_string(),
        got: s.to_string(),
        context: "custom modifier ID".to_string(),
    })?;
    if id > MAX_MODIFIER_ID {
        return Err(ParseError::ModifierIdOutOfRange {
            got: id,
            max: MAX_MODIFIER_ID,
        });
    }
    Ok(id as u8)
}

/// Parse a lock ID (LK_XX format, hex 00-FE).
pub fn parse_lock_id(s: &str) -> Result<u8, ParseError> {
    if !s.starts_with("LK_") {
        return Err(ParseError::MissingPrefix {
            key: s.to_string(),
            context: "custom lock".to_string(),
        });
    }
    let id_part = &s[3..];
    let id = u16::from_str_radix(id_part, 16).map_err(|_| ParseError::InvalidPrefix {
        expected: "LK_XX (hex, 00-FE)".to_string(),
        got: s.to_string(),
        context: "custom lock ID".to_string(),
    })?;
    if id > MAX_LOCK_ID {
        return Err(ParseError::LockIdOutOfRange {
            got: id,
            max: MAX_LOCK_ID,
        });
    }
    Ok(id as u8)
}

/// Parse a condition string (MD_XX or LK_XX) into a Condition.
pub fn parse_condition_string(s: &str) -> Result<Condition, ParseError> {
    if s.starts_with("MD_") {
        let id = parse_modifier_id(s)?;
        Ok(Condition::ModifierActive(id))
    } else if s.starts_with("LK_") {
        let id = parse_lock_id(s)?;
        Ok(Condition::LockActive(id))
    } else {
        Err(ParseError::InvalidPrefix {
            expected: "MD_XX or LK_XX".to_string(),
            got: s.to_string(),
            context: "condition".to_string(),
        })
    }
}

/// Get all valid key names for fuzzy matching suggestions.
fn get_all_key_names() -> Vec<&'static str> {
    vec![
        // Letters
        "A",
        "B",
        "C",
        "D",
        "E",
        "F",
        "G",
        "H",
        "I",
        "J",
        "K",
        "L",
        "M",
        "N",
        "O",
        "P",
        "Q",
        "R",
        "S",
        "T",
        "U",
        "V",
        "W",
        "X",
        "Y",
        "Z",
        // Numbers
        "Num0",
        "Num1",
        "Num2",
        "Num3",
        "Num4",
        "Num5",
        "Num6",
        "Num7",
        "Num8",
        "Num9",
        "0",
        "1",
        "2",
        "3",
        "4",
        "5",
        "6",
        "7",
        "8",
        "9",
        // Function keys
        "F1",
        "F2",
        "F3",
        "F4",
        "F5",
        "F6",
        "F7",
        "F8",
        "F9",
        "F10",
        "F11",
        "F12",
        "F13",
        "F14",
        "F15",
        "F16",
        "F17",
        "F18",
        "F19",
        "F20",
        "F21",
        "F22",
        "F23",
        "F24",
        // Modifiers
        "LShift",
        "RShift",
        "LCtrl",
        "RCtrl",
        "LAlt",
        "RAlt",
        "LMeta",
        "RMeta",
        // Special keys
        "Escape",
        "Esc",
        "Enter",
        "Return",
        "Backspace",
        "Tab",
        "Space",
        "CapsLock",
        "NumLock",
        "ScrollLock",
        "PrintScreen",
        "Pause",
        "Insert",
        "Ins",
        "Delete",
        "Del",
        "Home",
        "End",
        "PageUp",
        "PageDown",
        // Arrow keys
        "Left",
        "Right",
        "Up",
        "Down",
        // Symbols
        "LeftBracket",
        "RightBracket",
        "Backslash",
        "Semicolon",
        "Quote",
        "Comma",
        "Period",
        "Slash",
        "Grave",
        "Minus",
        "Equal",
        // Numpad
        "Numpad0",
        "Numpad1",
        "Numpad2",
        "Numpad3",
        "Numpad4",
        "Numpad5",
        "Numpad6",
        "Numpad7",
        "Numpad8",
        "Numpad9",
        "NumpadDivide",
        "NumpadMultiply",
        "NumpadSubtract",
        "NumpadAdd",
        "NumpadEnter",
        "NumpadDecimal",
        // Media keys
        "Mute",
        "VolumeDown",
        "VolumeUp",
        "MediaPlayPause",
        "MediaStop",
        "MediaPrevious",
        "MediaNext",
        // System keys
        "Power",
        "Sleep",
        "Wake",
        // Browser keys
        "BrowserBack",
        "BrowserForward",
        "BrowserRefresh",
        "BrowserStop",
        "BrowserSearch",
        "BrowserFavorites",
        "BrowserHome",
        // Application keys
        "AppMail",
        "AppCalculator",
        "AppMyComputer",
        // Additional keys
        "Menu",
        "Help",
        "Select",
        "Execute",
        "Undo",
        "Redo",
        "Cut",
        "Copy",
        "Paste",
        "Find",
        // Japanese JIS keyboard keys
        "Zenkaku",
        "全角",
        "半角",
        "Katakana",
        "カタカナ",
        "Hiragana",
        "ひらがな",
        "Henkan",
        "変換",
        "Muhenkan",
        "無変換",
        "Yen",
        "円",
        "Ro",
        "ろ",
        "KatakanaHiragana",
        // Korean keyboard keys
        "Hangeul",
        "Hangul",
        "한글",
        "Hanja",
        "한자",
        // ISO keyboard keys
        "Iso102nd",
    ]
}

/// Calculate Levenshtein distance for fuzzy matching.
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();
    let a_chars: Vec<char> = a_lower.chars().collect();
    let b_chars: Vec<char> = b_lower.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut prev_row: Vec<usize> = (0..=b_len).collect();
    let mut curr_row = vec![0; b_len + 1];

    for i in 1..=a_len {
        curr_row[0] = i;
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            curr_row[j] = (curr_row[j - 1] + 1)
                .min(prev_row[j] + 1)
                .min(prev_row[j - 1] + cost);
        }
        prev_row.clone_from_slice(&curr_row);
    }

    curr_row[b_len]
}

/// Find fuzzy suggestions for an unknown key name.
fn find_suggestions(name: &str) -> Vec<String> {
    let all_names = get_all_key_names();
    let mut matches: Vec<(usize, &str)> = all_names
        .iter()
        .map(|&valid_name| {
            let distance = levenshtein_distance(name, valid_name);
            (distance, valid_name)
        })
        .filter(|(distance, _)| *distance <= 3) // Only suggest if within 3 edits
        .collect();

    matches.sort_by_key(|(distance, _)| *distance);
    matches.truncate(3); // Only show top 3 suggestions

    matches
        .into_iter()
        .map(|(_, name)| name.to_string())
        .collect()
}

/// Parse a key name (without prefix) into a KeyCode.
pub fn parse_key_name(name: &str) -> Result<KeyCode, ParseError> {
    let keycode = match name {
        // Letters
        "A" => KeyCode::A,
        "B" => KeyCode::B,
        "C" => KeyCode::C,
        "D" => KeyCode::D,
        "E" => KeyCode::E,
        "F" => KeyCode::F,
        "G" => KeyCode::G,
        "H" => KeyCode::H,
        "I" => KeyCode::I,
        "J" => KeyCode::J,
        "K" => KeyCode::K,
        "L" => KeyCode::L,
        "M" => KeyCode::M,
        "N" => KeyCode::N,
        "O" => KeyCode::O,
        "P" => KeyCode::P,
        "Q" => KeyCode::Q,
        "R" => KeyCode::R,
        "S" => KeyCode::S,
        "T" => KeyCode::T,
        "U" => KeyCode::U,
        "V" => KeyCode::V,
        "W" => KeyCode::W,
        "X" => KeyCode::X,
        "Y" => KeyCode::Y,
        "Z" => KeyCode::Z,
        // Numbers
        "Num0" | "0" => KeyCode::Num0,
        "Num1" | "1" => KeyCode::Num1,
        "Num2" | "2" => KeyCode::Num2,
        "Num3" | "3" => KeyCode::Num3,
        "Num4" | "4" => KeyCode::Num4,
        "Num5" | "5" => KeyCode::Num5,
        "Num6" | "6" => KeyCode::Num6,
        "Num7" | "7" => KeyCode::Num7,
        "Num8" | "8" => KeyCode::Num8,
        "Num9" | "9" => KeyCode::Num9,
        // Function keys
        "F1" => KeyCode::F1,
        "F2" => KeyCode::F2,
        "F3" => KeyCode::F3,
        "F4" => KeyCode::F4,
        "F5" => KeyCode::F5,
        "F6" => KeyCode::F6,
        "F7" => KeyCode::F7,
        "F8" => KeyCode::F8,
        "F9" => KeyCode::F9,
        "F10" => KeyCode::F10,
        "F11" => KeyCode::F11,
        "F12" => KeyCode::F12,
        "F13" => KeyCode::F13,
        "F14" => KeyCode::F14,
        "F15" => KeyCode::F15,
        "F16" => KeyCode::F16,
        "F17" => KeyCode::F17,
        "F18" => KeyCode::F18,
        "F19" => KeyCode::F19,
        "F20" => KeyCode::F20,
        "F21" => KeyCode::F21,
        "F22" => KeyCode::F22,
        "F23" => KeyCode::F23,
        "F24" => KeyCode::F24,
        // Physical modifier keys
        "LShift" | "LSFT" => KeyCode::LShift,
        "RShift" | "RSFT" => KeyCode::RShift,
        "LCtrl" | "LCTL" => KeyCode::LCtrl,
        "RCtrl" | "RCTL" => KeyCode::RCtrl,
        "LAlt" | "LALT" => KeyCode::LAlt,
        "RAlt" | "RALT" => KeyCode::RAlt,
        "LMeta" | "LGUI" => KeyCode::LMeta,
        "RMeta" | "RGUI" => KeyCode::RMeta,
        // Special keys
        "Escape" | "Esc" | "ESC" => KeyCode::Escape,
        "Enter" | "Return" | "ENT" => KeyCode::Enter,
        "Backspace" | "BSPC" => KeyCode::Backspace,
        "Tab" | "TAB" => KeyCode::Tab,
        "Space" | "SPC" => KeyCode::Space,
        "CapsLock" | "CAPS" => KeyCode::CapsLock,
        "NumLock" | "NLCK" => KeyCode::NumLock,
        "ScrollLock" | "SCRL" => KeyCode::ScrollLock,
        "PrintScreen" | "PSCR" => KeyCode::PrintScreen,
        "Pause" | "PAUS" => KeyCode::Pause,
        "Insert" | "Ins" | "INS" => KeyCode::Insert,
        "Delete" | "Del" | "DEL" => KeyCode::Delete,
        "Home" | "HOME" => KeyCode::Home,
        "End" | "END" => KeyCode::End,
        "PageUp" | "PGUP" => KeyCode::PageUp,
        "PageDown" | "PGDN" => KeyCode::PageDown,
        // Arrow keys (with QMK aliases)
        "Left" | "LEFT" => KeyCode::Left,
        "Right" | "RGHT" => KeyCode::Right,
        "Up" | "UP" => KeyCode::Up,
        "Down" | "DOWN" => KeyCode::Down,
        // Symbols (with QMK aliases)
        "LeftBracket" | "LBRC" => KeyCode::LeftBracket,
        "RightBracket" | "RBRC" => KeyCode::RightBracket,
        "Backslash" | "BSLS" => KeyCode::Backslash,
        "Semicolon" | "SCLN" => KeyCode::Semicolon,
        "Quote" | "QUOT" => KeyCode::Quote,
        "Comma" | "COMM" => KeyCode::Comma,
        "Period" | "DOT" => KeyCode::Period,
        "Slash" | "SLSH" => KeyCode::Slash,
        "Grave" | "GRV" => KeyCode::Grave,
        "Minus" | "MINS" => KeyCode::Minus,
        "Equal" | "EQL" => KeyCode::Equal,
        // Numpad keys
        "Numpad0" => KeyCode::Numpad0,
        "Numpad1" => KeyCode::Numpad1,
        "Numpad2" => KeyCode::Numpad2,
        "Numpad3" => KeyCode::Numpad3,
        "Numpad4" => KeyCode::Numpad4,
        "Numpad5" => KeyCode::Numpad5,
        "Numpad6" => KeyCode::Numpad6,
        "Numpad7" => KeyCode::Numpad7,
        "Numpad8" => KeyCode::Numpad8,
        "Numpad9" => KeyCode::Numpad9,
        "NumpadDivide" => KeyCode::NumpadDivide,
        "NumpadMultiply" => KeyCode::NumpadMultiply,
        "NumpadSubtract" => KeyCode::NumpadSubtract,
        "NumpadAdd" => KeyCode::NumpadAdd,
        "NumpadEnter" => KeyCode::NumpadEnter,
        "NumpadDecimal" => KeyCode::NumpadDecimal,
        // Media keys
        "Mute" => KeyCode::Mute,
        "VolumeDown" => KeyCode::VolumeDown,
        "VolumeUp" => KeyCode::VolumeUp,
        "MediaPlayPause" => KeyCode::MediaPlayPause,
        "MediaStop" => KeyCode::MediaStop,
        "MediaPrevious" => KeyCode::MediaPrevious,
        "MediaNext" => KeyCode::MediaNext,
        // System keys
        "Power" => KeyCode::Power,
        "Sleep" => KeyCode::Sleep,
        "Wake" => KeyCode::Wake,
        // Browser keys
        "BrowserBack" => KeyCode::BrowserBack,
        "BrowserForward" => KeyCode::BrowserForward,
        "BrowserRefresh" => KeyCode::BrowserRefresh,
        "BrowserStop" => KeyCode::BrowserStop,
        "BrowserSearch" => KeyCode::BrowserSearch,
        "BrowserFavorites" => KeyCode::BrowserFavorites,
        "BrowserHome" => KeyCode::BrowserHome,
        // Application keys
        "AppMail" => KeyCode::AppMail,
        "AppCalculator" => KeyCode::AppCalculator,
        "AppMyComputer" => KeyCode::AppMyComputer,
        // Additional keys
        "Menu" => KeyCode::Menu,
        "Help" => KeyCode::Help,
        "Select" => KeyCode::Select,
        "Execute" => KeyCode::Execute,
        "Undo" => KeyCode::Undo,
        "Redo" => KeyCode::Redo,
        "Cut" => KeyCode::Cut,
        "Copy" => KeyCode::Copy,
        "Paste" => KeyCode::Paste,
        "Find" => KeyCode::Find,
        // Japanese JIS keyboard keys
        "Zenkaku" | "全角" | "半角" | "ZenkakuHankaku" => KeyCode::Zenkaku,
        "Katakana" | "カタカナ" => KeyCode::Katakana,
        "Hiragana" | "ひらがな" => KeyCode::Hiragana,
        "Henkan" | "変換" | "Convert" => KeyCode::Henkan,
        "Muhenkan" | "無変換" | "NonConvert" => KeyCode::Muhenkan,
        "Yen" | "円" | "¥" => KeyCode::Yen,
        "Ro" | "ろ" => KeyCode::Ro,
        "KatakanaHiragana" | "カタカナひらがな" => KeyCode::KatakanaHiragana,
        // Korean keyboard keys
        "Hangeul" | "Hangul" | "한글" => KeyCode::Hangeul,
        "Hanja" | "한자" => KeyCode::Hanja,
        // ISO/European keyboard keys
        "Iso102nd" | "102nd" => KeyCode::Iso102nd,
        _ => {
            // Generate suggestions for unknown key name
            let suggestions = find_suggestions(name);
            return Err(ParseError::UnknownKey {
                name: name.to_string(),
                suggestions,
            });
        }
    };
    Ok(keycode)
}
