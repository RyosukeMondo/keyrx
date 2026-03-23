//! Tests for parser validators module.
//!
//! Covers key parsing, prefix validation, modifier/lock ID parsing,
//! condition parsing, fuzzy suggestions, QMK aliases, and international keys.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use super::validators::*;
use crate::config::{Condition, KeyCode};
use crate::parser::error::ParseError;

// ============================================================================
// parse_virtual_key tests
// ============================================================================

#[test]
fn test_parse_virtual_key_common_keys() {
    assert_eq!(parse_virtual_key("VK_A").unwrap(), KeyCode::A);
    assert_eq!(parse_virtual_key("VK_Enter").unwrap(), KeyCode::Enter);
    assert_eq!(parse_virtual_key("VK_Escape").unwrap(), KeyCode::Escape);
    assert_eq!(parse_virtual_key("VK_Space").unwrap(), KeyCode::Space);
    assert_eq!(parse_virtual_key("VK_Tab").unwrap(), KeyCode::Tab);
}

#[test]
fn test_parse_virtual_key_requires_prefix() {
    let err = parse_virtual_key("A").unwrap_err();
    assert!(
        matches!(err, ParseError::MissingPrefix { ref key, .. } if key == "A"),
        "Expected MissingPrefix for bare 'A', got: {err:?}"
    );

    // With prefix should succeed
    assert_eq!(parse_virtual_key("VK_A").unwrap(), KeyCode::A);
}

// ============================================================================
// parse_physical_key tests
// ============================================================================

#[test]
fn test_parse_physical_key_accepts_bare_and_prefix() {
    // Bare name (no prefix) accepted for physical keys
    assert_eq!(parse_physical_key("A").unwrap(), KeyCode::A);
    // VK_ prefix also accepted
    assert_eq!(parse_physical_key("VK_A").unwrap(), KeyCode::A);
    // Both should produce the same result
    assert_eq!(parse_physical_key("CapsLock").unwrap(), KeyCode::CapsLock);
    assert_eq!(
        parse_physical_key("VK_CapsLock").unwrap(),
        KeyCode::CapsLock
    );
}

// ============================================================================
// parse_modifier_id tests
// ============================================================================

#[test]
fn test_parse_modifier_id_valid_range() {
    assert_eq!(parse_modifier_id("MD_00").unwrap(), 0x00);
    assert_eq!(parse_modifier_id("MD_7F").unwrap(), 0x7F);
    assert_eq!(parse_modifier_id("MD_FE").unwrap(), 0xFE);
}

#[test]
fn test_parse_modifier_id_out_of_range() {
    let err = parse_modifier_id("MD_FF").unwrap_err();
    assert!(
        matches!(err, ParseError::ModifierIdOutOfRange { got: 0xFF, .. }),
        "Expected ModifierIdOutOfRange for MD_FF, got: {err:?}"
    );
}

#[test]
fn test_parse_modifier_id_rejects_physical() {
    let err = parse_modifier_id("MD_LShift").unwrap_err();
    assert!(
        matches!(err, ParseError::PhysicalModifierInMD { ref name } if name == "LShift"),
        "Expected PhysicalModifierInMD for 'LShift', got: {err:?}"
    );

    // Also test other physical modifiers
    assert!(parse_modifier_id("MD_RCtrl").is_err());
    assert!(parse_modifier_id("MD_LAlt").is_err());
    assert!(parse_modifier_id("MD_RMeta").is_err());
}

#[test]
fn test_parse_modifier_id_missing_prefix() {
    let err = parse_modifier_id("00").unwrap_err();
    assert!(
        matches!(err, ParseError::MissingPrefix { .. }),
        "Expected MissingPrefix for bare '00', got: {err:?}"
    );
}

// ============================================================================
// parse_lock_id tests
// ============================================================================

#[test]
fn test_parse_lock_id_valid_range() {
    assert_eq!(parse_lock_id("LK_00").unwrap(), 0x00);
    assert_eq!(parse_lock_id("LK_3F").unwrap(), 0x3F);
    assert_eq!(parse_lock_id("LK_FE").unwrap(), 0xFE);
}

#[test]
fn test_parse_lock_id_out_of_range() {
    let err = parse_lock_id("LK_FF").unwrap_err();
    assert!(
        matches!(err, ParseError::LockIdOutOfRange { got: 0xFF, .. }),
        "Expected LockIdOutOfRange for LK_FF, got: {err:?}"
    );
}

// ============================================================================
// parse_condition_string tests
// ============================================================================

#[test]
fn test_parse_condition_string_modifier_and_lock() {
    let cond = parse_condition_string("MD_00").unwrap();
    assert!(
        matches!(cond, Condition::ModifierActive(0)),
        "Expected ModifierActive(0), got: {cond:?}"
    );

    let cond = parse_condition_string("LK_01").unwrap();
    assert!(
        matches!(cond, Condition::LockActive(1)),
        "Expected LockActive(1), got: {cond:?}"
    );
}

#[test]
fn test_parse_condition_string_invalid_prefix() {
    let err = parse_condition_string("VK_A").unwrap_err();
    assert!(
        matches!(err, ParseError::InvalidPrefix { .. }),
        "Expected InvalidPrefix for 'VK_A' as condition, got: {err:?}"
    );
}

// ============================================================================
// Unknown key suggestions
// ============================================================================

#[test]
fn test_unknown_key_suggests_alternatives() {
    let err = parse_key_name("Shft").unwrap_err();
    match err {
        ParseError::UnknownKey { suggestions, .. } => {
            // Should suggest LShift and/or RShift (Levenshtein distance ≤ 3)
            let has_shift = suggestions.iter().any(|s| s.contains("Shift"));
            assert!(
                has_shift,
                "Expected shift suggestions for 'Shft', got: {suggestions:?}"
            );
        }
        other => panic!("Expected UnknownKey, got: {other:?}"),
    }
}

#[test]
fn test_completely_unknown_key() {
    let err = parse_key_name("XYZZY").unwrap_err();
    assert!(
        matches!(err, ParseError::UnknownKey { .. }),
        "Expected UnknownKey for 'XYZZY', got: {err:?}"
    );
}

// ============================================================================
// QMK aliases
// ============================================================================

#[test]
fn test_qmk_aliases() {
    assert_eq!(parse_key_name("ENT").unwrap(), KeyCode::Enter);
    assert_eq!(parse_key_name("BSPC").unwrap(), KeyCode::Backspace);
    assert_eq!(parse_key_name("SPC").unwrap(), KeyCode::Space);
    assert_eq!(parse_key_name("CAPS").unwrap(), KeyCode::CapsLock);
    assert_eq!(parse_key_name("LSFT").unwrap(), KeyCode::LShift);
    assert_eq!(parse_key_name("RCTL").unwrap(), KeyCode::RCtrl);
    assert_eq!(parse_key_name("ESC").unwrap(), KeyCode::Escape);
    assert_eq!(parse_key_name("LGUI").unwrap(), KeyCode::LMeta);
    assert_eq!(parse_key_name("RGUI").unwrap(), KeyCode::RMeta);
}

// ============================================================================
// International keys
// ============================================================================

#[test]
fn test_international_keys() {
    // Japanese JIS
    // Zenkaku/Hankaku resolve to Grave (scan code 0x29)
    assert_eq!(parse_key_name("Zenkaku").unwrap(), KeyCode::Grave);
    assert_eq!(parse_key_name("Hankaku").unwrap(), KeyCode::Grave);
    assert_eq!(parse_key_name("ZenkakuHankaku").unwrap(), KeyCode::Grave);
    // Katakana resolves to KatakanaHiragana (scan code 0x70)
    assert_eq!(
        parse_key_name("Katakana").unwrap(),
        KeyCode::KatakanaHiragana
    );
    assert_eq!(parse_key_name("Hiragana").unwrap(), KeyCode::Hiragana);
    assert_eq!(parse_key_name("Henkan").unwrap(), KeyCode::Henkan);
    assert_eq!(parse_key_name("Muhenkan").unwrap(), KeyCode::Muhenkan);

    // Korean
    assert_eq!(parse_key_name("Hangeul").unwrap(), KeyCode::Hangeul);
    assert_eq!(parse_key_name("Hangul").unwrap(), KeyCode::Hangeul);
    assert_eq!(parse_key_name("Hanja").unwrap(), KeyCode::Hanja);

    // Unicode aliases
    assert_eq!(parse_key_name("全角").unwrap(), KeyCode::Grave);
    assert_eq!(parse_key_name("半角").unwrap(), KeyCode::Grave);
    assert_eq!(
        parse_key_name("カタカナ").unwrap(),
        KeyCode::KatakanaHiragana
    );
    assert_eq!(parse_key_name("한글").unwrap(), KeyCode::Hangeul);
    assert_eq!(parse_key_name("한자").unwrap(), KeyCode::Hanja);
}

// ============================================================================
// IME condition parsing tests
// ============================================================================

#[test]
fn test_parse_condition_string_ime() {
    let cond = parse_condition_string("IME").unwrap();
    assert!(matches!(cond, Condition::ImeActive));
}

#[test]
fn test_parse_condition_string_language() {
    let cond = parse_condition_string("LANG_JA").unwrap();
    assert!(matches!(cond, Condition::InputLanguage(ref lang) if lang == "ja"));

    let cond2 = parse_condition_string("LANG_KO").unwrap();
    assert!(matches!(cond2, Condition::InputLanguage(ref lang) if lang == "ko"));

    let cond3 = parse_condition_string("LANG_ZH").unwrap();
    assert!(matches!(cond3, Condition::InputLanguage(ref lang) if lang == "zh"));
}

#[test]
fn test_parse_condition_string_language_lowercase() {
    // LANG_JA and LANG_ja should both produce "ja"
    let cond = parse_condition_string("LANG_ja").unwrap();
    assert!(matches!(cond, Condition::InputLanguage(ref lang) if lang == "ja"));
}

#[test]
fn test_parse_condition_string_language_empty() {
    let err = parse_condition_string("LANG_").unwrap_err();
    assert!(matches!(err, ParseError::InvalidPrefix { .. }));
}

#[test]
fn test_parse_condition_string_updated_error_message() {
    let err = parse_condition_string("INVALID").unwrap_err();
    if let ParseError::InvalidPrefix { expected, .. } = &err {
        assert!(expected.contains("IME"));
        assert!(expected.contains("LANG_XX"));
    } else {
        panic!("Expected InvalidPrefix error");
    }
}

// ============================================================================
// Sequence builder tests
// ============================================================================

use crate::parser::builders::{build_sequence, MAX_SEQUENCE_LENGTH};

#[test]
fn test_build_sequence_valid() {
    let keys = vec![String::from("VK_Y"), String::from("VK_A")];
    let result = build_sequence("VK_Semicolon", &keys);
    assert!(result.is_ok(), "Valid sequence should succeed: {:?}", result);
}

#[test]
fn test_build_sequence_single_key() {
    let keys = vec![String::from("VK_K")];
    let result = build_sequence("VK_C", &keys);
    assert!(result.is_ok());
}

#[test]
fn test_build_sequence_empty_rejected() {
    let keys: Vec<String> = vec![];
    let result = build_sequence("VK_A", &keys);
    assert!(result.is_err(), "Empty sequence should be rejected");
}

#[test]
fn test_build_sequence_too_long_rejected() {
    let keys: Vec<String> = (0..=MAX_SEQUENCE_LENGTH)
        .map(|_| String::from("VK_A"))
        .collect();
    let result = build_sequence("VK_A", &keys);
    assert!(result.is_err(), "Sequence exceeding max length should be rejected");
}

#[test]
fn test_build_sequence_max_length_accepted() {
    let keys: Vec<String> = (0..MAX_SEQUENCE_LENGTH)
        .map(|_| String::from("VK_A"))
        .collect();
    let result = build_sequence("VK_A", &keys);
    assert!(result.is_ok(), "Sequence at max length should be accepted");
}

#[test]
fn test_build_sequence_invalid_from_key() {
    let keys = vec![String::from("VK_A")];
    let result = build_sequence("INVALID_KEY", &keys);
    assert!(result.is_err(), "Invalid from key should be rejected");
}

#[test]
fn test_build_sequence_invalid_output_key() {
    let keys = vec![String::from("VK_A"), String::from("INVALID")];
    let result = build_sequence("VK_A", &keys);
    assert!(result.is_err(), "Invalid output key should be rejected");
}
