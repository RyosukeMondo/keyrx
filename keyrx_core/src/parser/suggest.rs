//! "Did you mean" suggestions for unknown key names.

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

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
        // Japanese JIS keyboard keys (Zenkaku/Hankaku → Grave, カタカナ → KatakanaHiragana)
        "Zenkaku",
        "全角",
        "半角",
        "Hankaku",
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

/// Largest edit distance at which `name` may be suggested for a typo of
/// `valid`: about one edit per three characters. Names under 3 characters
/// only match ignoring case, so `Q` or `F` never suggest unrelated keys.
fn max_suggestion_distance(name_len: usize) -> usize {
    if name_len < 3 {
        0
    } else {
        (name_len / 3).clamp(1, 3)
    }
}

/// True when `typed` is a prefix of `valid` or its letters appear in order
/// with at most three extra letters (`shft` -> `lshift`, `ente` -> `enter`).
fn is_abbreviation(typed: &str, valid: &str) -> bool {
    if valid.chars().count() > typed.chars().count() + 3 {
        return false;
    }
    let mut rest = valid.chars();
    typed.chars().all(|c| rest.any(|v| v == c))
}

/// Find "did you mean" suggestions for an unknown key name: a valid name
/// that abbreviates to what was typed (`Ente` -> `Enter`) or is within a small,
/// length-scaled edit distance (`Escpae` -> `Escape`). Nothing is suggested
/// when no name is that close - an unrelated suggestion is worse than none.
pub(crate) fn find_suggestions(name: &str) -> Vec<String> {
    let typed = name.to_lowercase();
    let typed_len = typed.chars().count();
    let max_distance = max_suggestion_distance(typed_len);
    let mut matches: Vec<(usize, &str)> = get_all_key_names()
        .iter()
        .filter_map(|&valid| {
            let lower = valid.to_lowercase();
            let distance = levenshtein_distance(&typed, &lower);
            if distance <= max_distance {
                Some((distance, valid))
            } else if typed_len >= 3 && is_abbreviation(&typed, &lower) {
                // An abbreviation ranks after near-typos but is still helpful.
                Some((max_distance + 1, valid))
            } else {
                None
            }
        })
        .collect();

    matches.sort_by_key(|(distance, _)| *distance);
    matches.truncate(3);

    matches
        .into_iter()
        .map(|(_, name)| name.to_string())
        .collect()
}
