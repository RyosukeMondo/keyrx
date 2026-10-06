//! Exhaustive coverage test guarding the `keycode_map` split.
//!
//! `keycode_to_uinput_key` and `keycode_to_evdev` are exhaustive matches over
//! `KeyCode` (no wildcard arm), so the compiler already guarantees every
//! variant is handled by both. What it cannot guarantee is that the evdev
//! *round trip* (`KeyCode` -> evdev code -> `KeyCode`) still lands on the
//! same variant, or that two variants were not accidentally mapped to the
//! same evdev code. This test lists every `KeyCode` variant explicitly (kept
//! in sync with `keyrx_core::config::keys::KeyCode`) and checks both
//! properties, so that splitting the mapping tables across files cannot
//! silently drop or duplicate an entry.

use std::collections::HashSet;

use keyrx_core::config::KeyCode;

use super::{evdev_to_keycode, keycode_to_evdev, keycode_to_uinput_key};

/// Every `KeyCode` variant, in the same order and grouping as the enum
/// definition in `keyrx_core::config::keys`.
const ALL_KEYCODES: &[KeyCode] = &[
    // Letters A-Z
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
    // Numbers 0-9
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
    // Function keys F1-F12
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
    // Physical modifier keys
    KeyCode::LShift,
    KeyCode::RShift,
    KeyCode::LCtrl,
    KeyCode::RCtrl,
    KeyCode::LAlt,
    KeyCode::RAlt,
    KeyCode::LMeta,
    KeyCode::RMeta,
    // Special keys
    KeyCode::Escape,
    KeyCode::Enter,
    KeyCode::Backspace,
    KeyCode::Tab,
    KeyCode::Space,
    KeyCode::CapsLock,
    KeyCode::NumLock,
    KeyCode::ScrollLock,
    KeyCode::PrintScreen,
    KeyCode::Pause,
    KeyCode::Insert,
    KeyCode::Delete,
    KeyCode::Home,
    KeyCode::End,
    KeyCode::PageUp,
    KeyCode::PageDown,
    // Arrow keys
    KeyCode::Left,
    KeyCode::Right,
    KeyCode::Up,
    KeyCode::Down,
    // Additional special keys
    KeyCode::LeftBracket,
    KeyCode::RightBracket,
    KeyCode::Backslash,
    KeyCode::Semicolon,
    KeyCode::Quote,
    KeyCode::Comma,
    KeyCode::Period,
    KeyCode::Slash,
    KeyCode::Grave,
    KeyCode::Minus,
    KeyCode::Equal,
    // Numpad keys
    KeyCode::Numpad0,
    KeyCode::Numpad1,
    KeyCode::Numpad2,
    KeyCode::Numpad3,
    KeyCode::Numpad4,
    KeyCode::Numpad5,
    KeyCode::Numpad6,
    KeyCode::Numpad7,
    KeyCode::Numpad8,
    KeyCode::Numpad9,
    KeyCode::NumpadDivide,
    KeyCode::NumpadMultiply,
    KeyCode::NumpadSubtract,
    KeyCode::NumpadAdd,
    KeyCode::NumpadEnter,
    KeyCode::NumpadDecimal,
    // Extended function keys F13-F24
    KeyCode::F13,
    KeyCode::F14,
    KeyCode::F15,
    KeyCode::F16,
    KeyCode::F17,
    KeyCode::F18,
    KeyCode::F19,
    KeyCode::F20,
    KeyCode::F21,
    KeyCode::F22,
    KeyCode::F23,
    KeyCode::F24,
    // Media keys
    KeyCode::Mute,
    KeyCode::VolumeDown,
    KeyCode::VolumeUp,
    KeyCode::MediaPlayPause,
    KeyCode::MediaStop,
    KeyCode::MediaPrevious,
    KeyCode::MediaNext,
    // System keys
    KeyCode::Power,
    KeyCode::Sleep,
    KeyCode::Wake,
    // Browser keys
    KeyCode::BrowserBack,
    KeyCode::BrowserForward,
    KeyCode::BrowserRefresh,
    KeyCode::BrowserStop,
    KeyCode::BrowserSearch,
    KeyCode::BrowserFavorites,
    KeyCode::BrowserHome,
    // Application keys
    KeyCode::AppMail,
    KeyCode::AppCalculator,
    KeyCode::AppMyComputer,
    // Additional keys
    KeyCode::Menu,
    KeyCode::Help,
    KeyCode::Select,
    KeyCode::Execute,
    KeyCode::Undo,
    KeyCode::Redo,
    KeyCode::Cut,
    KeyCode::Copy,
    KeyCode::Paste,
    KeyCode::Find,
    // Japanese JIS keyboard keys
    KeyCode::Zenkaku,
    KeyCode::Katakana,
    KeyCode::Hiragana,
    KeyCode::Henkan,
    KeyCode::Muhenkan,
    KeyCode::Yen,
    KeyCode::Ro,
    KeyCode::KatakanaHiragana,
    // Korean keyboard keys
    KeyCode::Hangeul,
    KeyCode::Hanja,
    // ISO/European keyboard keys
    KeyCode::Iso102nd,
];

/// `keyrx_core::config::keys::KeyCode` currently has 156 variants. If this
/// fails, a variant was added or removed there without updating
/// `ALL_KEYCODES` above (and, most likely, without updating this module's
/// siblings either).
#[test]
fn test_all_keycodes_list_is_complete() {
    assert_eq!(
        ALL_KEYCODES.len(),
        156,
        "ALL_KEYCODES no longer matches the KeyCode variant count; update this list"
    );
}

/// Every `KeyCode` must survive a `keycode_to_evdev` -> `evdev_to_keycode`
/// round trip unchanged, and no two variants may collide on the same evdev
/// code (which would make the round trip ambiguous).
#[test]
fn test_all_keycodes_evdev_roundtrip() {
    let mut seen_evdev_codes = HashSet::new();

    for &keycode in ALL_KEYCODES {
        let evdev_code = keycode_to_evdev(keycode);
        assert!(
            seen_evdev_codes.insert(evdev_code),
            "evdev code {evdev_code} is mapped from more than one KeyCode (duplicate at {keycode:?})"
        );

        let round_tripped = evdev_to_keycode(evdev_code);
        assert_eq!(
            round_tripped,
            Some(keycode),
            "round-trip failed for {keycode:?} (evdev code {evdev_code})"
        );
    }
}

/// `keycode_to_uinput_key` is an exhaustive match with no wildcard arm, so
/// the compiler already guarantees every variant is handled; this simply
/// documents and exercises that guarantee for every variant at once.
#[test]
fn test_all_keycodes_uinput_mapping_is_total() {
    for &keycode in ALL_KEYCODES {
        let _ = keycode_to_uinput_key(keycode);
    }
}

/// The code the daemon actually writes for a `KeyCode` (via the `uinput`
/// crate's enum) must be the code the daemon reads it back from
/// (`keycode_to_evdev`): two tables that disagree type a different key than
/// the one the layout names.
#[test]
fn test_uinput_codes_equal_evdev_codes() {
    use uinput::event::Code as _;
    let mismatches: Vec<String> = ALL_KEYCODES
        .iter()
        .filter_map(|&k| {
            let out = i32::from(keycode_to_evdev(k));
            let wrote = keycode_to_uinput_key(k).code();
            (out != wrote).then(|| format!("{k:?}: evdev {out} but uinput writes {wrote}"))
        })
        .collect();
    assert!(mismatches.is_empty(), "{mismatches:#?}");
}
