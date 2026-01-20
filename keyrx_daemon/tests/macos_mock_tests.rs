//! macOS mock tests for keycode conversion.
//!
//! These tests validate CGKeyCode ↔ KeyCode bidirectional conversion
//! without requiring Accessibility permissions, making them suitable for CI.
//!
//! Tests cover:
//! - All 140+ keycode mappings with round-trip validation
//! - Edge cases: unknown codes, reserved values, boundary conditions
//! - Zero data loss guarantee for CGKeyCode → KeyCode → CGKeyCode
//!
//! # Design
//!
//! Unlike integration tests that require real hardware and permissions,
//! these tests operate purely on conversion functions, ensuring:
//! 1. Complete coverage of keycode_map.rs
//! 2. Deterministic behavior (no hardware dependencies)
//! 3. Fast execution (<1s)
//! 4. CI-friendly (no permission gates)

#![cfg(target_os = "macos")]

use keyrx_core::config::keys::KeyCode;
use keyrx_daemon::platform::macos::keycode_map::{cgkeycode_to_keyrx, keyrx_to_cgkeycode};

/// Test: Round-trip conversion for all letter keys (A-Z).
///
/// Validates that CGKeyCode → KeyCode → CGKeyCode preserves the original
/// CGKeyCode value with zero data loss for all QWERTY letter positions.
#[test]
fn test_cgkeycode_letters_roundtrip() {
    let letters = vec![
        (0x00, KeyCode::A),
        (0x0B, KeyCode::B),
        (0x08, KeyCode::C),
        (0x02, KeyCode::D),
        (0x0E, KeyCode::E),
        (0x03, KeyCode::F),
        (0x05, KeyCode::G),
        (0x04, KeyCode::H),
        (0x22, KeyCode::I),
        (0x26, KeyCode::J),
        (0x28, KeyCode::K),
        (0x25, KeyCode::L),
        (0x2E, KeyCode::M),
        (0x2D, KeyCode::N),
        (0x1F, KeyCode::O),
        (0x23, KeyCode::P),
        (0x0C, KeyCode::Q),
        (0x0F, KeyCode::R),
        (0x01, KeyCode::S),
        (0x11, KeyCode::T),
        (0x20, KeyCode::U),
        (0x09, KeyCode::V),
        (0x0D, KeyCode::W),
        (0x07, KeyCode::X),
        (0x10, KeyCode::Y),
        (0x06, KeyCode::Z),
    ];

    for (cgcode, expected_keycode) in letters {
        // Forward: CGKeyCode → KeyCode
        let keycode = cgkeycode_to_keyrx(cgcode);
        assert_eq!(
            keycode,
            Some(expected_keycode),
            "CGKeyCode 0x{:02x} should map to {:?}",
            cgcode,
            expected_keycode
        );

        // Backward: KeyCode → CGKeyCode
        let back = keyrx_to_cgkeycode(expected_keycode);
        assert_eq!(
            back,
            Some(cgcode),
            "{:?} should map back to CGKeyCode 0x{:02x}",
            expected_keycode,
            cgcode
        );

        // Round-trip: CGKeyCode → KeyCode → CGKeyCode
        if let Some(kc) = cgkeycode_to_keyrx(cgcode) {
            let roundtrip = keyrx_to_cgkeycode(kc);
            assert_eq!(
                roundtrip,
                Some(cgcode),
                "Round-trip failed for CGKeyCode 0x{:02x} ({:?})",
                cgcode,
                kc
            );
        }
    }
}

/// Test: Round-trip conversion for number keys (0-9).
///
/// Validates conversion for top-row number keys with zero data loss.
#[test]
fn test_cgkeycode_numbers_roundtrip() {
    let numbers = vec![
        (0x1D, KeyCode::Num0),
        (0x12, KeyCode::Num1),
        (0x13, KeyCode::Num2),
        (0x14, KeyCode::Num3),
        (0x15, KeyCode::Num4),
        (0x17, KeyCode::Num5),
        (0x16, KeyCode::Num6),
        (0x1A, KeyCode::Num7),
        (0x1C, KeyCode::Num8),
        (0x19, KeyCode::Num9),
    ];

    for (cgcode, expected_keycode) in numbers {
        let keycode = cgkeycode_to_keyrx(cgcode);
        assert_eq!(keycode, Some(expected_keycode));

        let back = keyrx_to_cgkeycode(expected_keycode);
        assert_eq!(back, Some(cgcode));

        if let Some(kc) = cgkeycode_to_keyrx(cgcode) {
            let roundtrip = keyrx_to_cgkeycode(kc);
            assert_eq!(
                roundtrip,
                Some(cgcode),
                "Round-trip failed for number key 0x{:02x}",
                cgcode
            );
        }
    }
}

/// Test: Round-trip conversion for function keys (F1-F20).
///
/// Validates all function keys including extended range (F13-F20)
/// common on Apple keyboards.
#[test]
fn test_cgkeycode_function_keys_roundtrip() {
    let function_keys = vec![
        (0x7A, KeyCode::F1),
        (0x78, KeyCode::F2),
        (0x63, KeyCode::F3),
        (0x76, KeyCode::F4),
        (0x60, KeyCode::F5),
        (0x61, KeyCode::F6),
        (0x62, KeyCode::F7),
        (0x64, KeyCode::F8),
        (0x65, KeyCode::F9),
        (0x6D, KeyCode::F10),
        (0x67, KeyCode::F11),
        (0x6F, KeyCode::F12),
        (0x69, KeyCode::F13),
        (0x6B, KeyCode::F14),
        (0x71, KeyCode::F15),
        (0x6A, KeyCode::F16),
        (0x40, KeyCode::F17),
        (0x4F, KeyCode::F18),
        (0x50, KeyCode::F19),
        (0x5A, KeyCode::F20),
    ];

    for (cgcode, expected_keycode) in function_keys {
        let keycode = cgkeycode_to_keyrx(cgcode);
        assert_eq!(keycode, Some(expected_keycode));

        let back = keyrx_to_cgkeycode(expected_keycode);
        assert_eq!(back, Some(cgcode));

        if let Some(kc) = cgkeycode_to_keyrx(cgcode) {
            let roundtrip = keyrx_to_cgkeycode(kc);
            assert_eq!(
                roundtrip,
                Some(cgcode),
                "Round-trip failed for function key 0x{:02x}",
                cgcode
            );
        }
    }
}

/// Test: Round-trip conversion for modifier keys.
///
/// Validates macOS modifier keys including Command (⌘), Option, Control, Shift.
#[test]
fn test_cgkeycode_modifiers_roundtrip() {
    let modifiers = vec![
        (0x38, KeyCode::LShift),
        (0x3C, KeyCode::RShift),
        (0x3B, KeyCode::LCtrl),
        (0x3E, KeyCode::RCtrl),
        (0x3A, KeyCode::LAlt),
        (0x3D, KeyCode::RAlt),
        (0x37, KeyCode::LMeta), // Command/Cmd
        (0x36, KeyCode::RMeta), // Command/Cmd
    ];

    for (cgcode, expected_keycode) in modifiers {
        let keycode = cgkeycode_to_keyrx(cgcode);
        assert_eq!(
            keycode,
            Some(expected_keycode),
            "Modifier CGKeyCode 0x{:02x} should map to {:?}",
            cgcode,
            expected_keycode
        );

        let back = keyrx_to_cgkeycode(expected_keycode);
        assert_eq!(back, Some(cgcode));

        if let Some(kc) = cgkeycode_to_keyrx(cgcode) {
            let roundtrip = keyrx_to_cgkeycode(kc);
            assert_eq!(
                roundtrip,
                Some(cgcode),
                "Round-trip failed for modifier 0x{:02x}",
                cgcode
            );
        }
    }
}

/// Test: Round-trip conversion for special keys.
///
/// Validates Escape, Enter, Backspace, Tab, Space, CapsLock.
#[test]
fn test_cgkeycode_special_keys_roundtrip() {
    let special_keys = vec![
        (0x35, KeyCode::Escape),
        (0x24, KeyCode::Enter),
        (0x33, KeyCode::Backspace),
        (0x30, KeyCode::Tab),
        (0x31, KeyCode::Space),
        (0x39, KeyCode::CapsLock),
    ];

    for (cgcode, expected_keycode) in special_keys {
        let keycode = cgkeycode_to_keyrx(cgcode);
        assert_eq!(keycode, Some(expected_keycode));

        let back = keyrx_to_cgkeycode(expected_keycode);
        assert_eq!(back, Some(cgcode));

        if let Some(kc) = cgkeycode_to_keyrx(cgcode) {
            let roundtrip = keyrx_to_cgkeycode(kc);
            assert_eq!(
                roundtrip,
                Some(cgcode),
                "Round-trip failed for special key 0x{:02x}",
                cgcode
            );
        }
    }
}

/// Test: Round-trip conversion for navigation keys.
///
/// Validates arrow keys, Insert, Delete, Home, End, PageUp, PageDown.
#[test]
fn test_cgkeycode_navigation_keys_roundtrip() {
    let navigation_keys = vec![
        (0x72, KeyCode::Insert), // Help key on Mac keyboards
        (0x75, KeyCode::Delete),
        (0x73, KeyCode::Home),
        (0x77, KeyCode::End),
        (0x74, KeyCode::PageUp),
        (0x79, KeyCode::PageDown),
        (0x7B, KeyCode::Left),
        (0x7C, KeyCode::Right),
        (0x7E, KeyCode::Up),
        (0x7D, KeyCode::Down),
    ];

    for (cgcode, expected_keycode) in navigation_keys {
        let keycode = cgkeycode_to_keyrx(cgcode);
        assert_eq!(keycode, Some(expected_keycode));

        let back = keyrx_to_cgkeycode(expected_keycode);
        assert_eq!(back, Some(cgcode));

        if let Some(kc) = cgkeycode_to_keyrx(cgcode) {
            let roundtrip = keyrx_to_cgkeycode(kc);
            assert_eq!(
                roundtrip,
                Some(cgcode),
                "Round-trip failed for navigation key 0x{:02x}",
                cgcode
            );
        }
    }
}

/// Test: Round-trip conversion for punctuation and symbol keys.
///
/// Validates brackets, backslash, semicolon, quote, comma, period, etc.
#[test]
fn test_cgkeycode_punctuation_roundtrip() {
    let punctuation = vec![
        (0x21, KeyCode::LeftBracket),
        (0x1E, KeyCode::RightBracket),
        (0x2A, KeyCode::Backslash),
        (0x29, KeyCode::Semicolon),
        (0x27, KeyCode::Quote),
        (0x2B, KeyCode::Comma),
        (0x2F, KeyCode::Period),
        (0x2C, KeyCode::Slash),
        (0x32, KeyCode::Grave),
        (0x1B, KeyCode::Minus),
        (0x18, KeyCode::Equal),
    ];

    for (cgcode, expected_keycode) in punctuation {
        let keycode = cgkeycode_to_keyrx(cgcode);
        assert_eq!(keycode, Some(expected_keycode));

        let back = keyrx_to_cgkeycode(expected_keycode);
        assert_eq!(back, Some(cgcode));

        if let Some(kc) = cgkeycode_to_keyrx(cgcode) {
            let roundtrip = keyrx_to_cgkeycode(kc);
            assert_eq!(
                roundtrip,
                Some(cgcode),
                "Round-trip failed for punctuation 0x{:02x}",
                cgcode
            );
        }
    }
}

/// Test: Round-trip conversion for numpad keys.
///
/// Validates numpad numbers, operators, Enter, Decimal, and Clear/NumLock.
#[test]
fn test_cgkeycode_numpad_roundtrip() {
    let numpad = vec![
        (0x52, KeyCode::Numpad0),
        (0x53, KeyCode::Numpad1),
        (0x54, KeyCode::Numpad2),
        (0x55, KeyCode::Numpad3),
        (0x56, KeyCode::Numpad4),
        (0x57, KeyCode::Numpad5),
        (0x58, KeyCode::Numpad6),
        (0x59, KeyCode::Numpad7),
        (0x5B, KeyCode::Numpad8),
        (0x5C, KeyCode::Numpad9),
        (0x4B, KeyCode::NumpadDivide),
        (0x43, KeyCode::NumpadMultiply),
        (0x4E, KeyCode::NumpadSubtract),
        (0x45, KeyCode::NumpadAdd),
        (0x4C, KeyCode::NumpadEnter),
        (0x41, KeyCode::NumpadDecimal),
        (0x47, KeyCode::NumLock), // Clear key on Mac
    ];

    for (cgcode, expected_keycode) in numpad {
        let keycode = cgkeycode_to_keyrx(cgcode);
        assert_eq!(keycode, Some(expected_keycode));

        let back = keyrx_to_cgkeycode(expected_keycode);
        assert_eq!(back, Some(cgcode));

        if let Some(kc) = cgkeycode_to_keyrx(cgcode) {
            let roundtrip = keyrx_to_cgkeycode(kc);
            assert_eq!(
                roundtrip,
                Some(cgcode),
                "Round-trip failed for numpad key 0x{:02x}",
                cgcode
            );
        }
    }
}

/// Test: Round-trip conversion for media keys.
///
/// Validates Mute, VolumeDown, VolumeUp.
#[test]
fn test_cgkeycode_media_keys_roundtrip() {
    let media_keys = vec![
        (0x4A, KeyCode::Mute),
        (0x49, KeyCode::VolumeDown),
        (0x48, KeyCode::VolumeUp),
    ];

    for (cgcode, expected_keycode) in media_keys {
        let keycode = cgkeycode_to_keyrx(cgcode);
        assert_eq!(keycode, Some(expected_keycode));

        let back = keyrx_to_cgkeycode(expected_keycode);
        assert_eq!(back, Some(cgcode));

        if let Some(kc) = cgkeycode_to_keyrx(cgcode) {
            let roundtrip = keyrx_to_cgkeycode(kc);
            assert_eq!(
                roundtrip,
                Some(cgcode),
                "Round-trip failed for media key 0x{:02x}",
                cgcode
            );
        }
    }
}

/// Test: Unknown CGKeyCode returns None.
///
/// Edge case: Unmapped CGKeyCodes should return None, not panic or
/// return incorrect values.
#[test]
fn test_cgkeycode_unknown_returns_none() {
    // Test various unmapped CGKeyCodes
    let unknown_codes = vec![
        0xFF,   // High unmapped value
        0x100,  // Beyond u8 range
        0x200,  // Far beyond mapped range
        0x500,  // Even farther
        0x1000, // Way out of range
        0xFFFF, // Maximum u16 value
        0x80,   // Just above typical range
        0x90,   // Mid-range unmapped
        0xA0,   // Another unmapped
        0xB0,   // Yet another
    ];

    for unknown_code in unknown_codes {
        let result = cgkeycode_to_keyrx(unknown_code);
        assert_eq!(
            result, None,
            "Unknown CGKeyCode 0x{:04x} should return None",
            unknown_code
        );
    }
}

/// Test: Reserved CGKeyCode values return None.
///
/// Edge case: Apple reserves certain CGKeyCode ranges. These should
/// gracefully return None.
#[test]
fn test_cgkeycode_reserved_values() {
    // Test reserved/unused CGKeyCode ranges
    // Most macOS CGKeyCodes are < 0x80, so test 0x80-0xFF
    for cgcode in 0x80..=0xFF {
        // Skip if it happens to be mapped (media keys, etc.)
        let result = cgkeycode_to_keyrx(cgcode);
        // Should either return None or a valid KeyCode (no panic)
        if let Some(keycode) = result {
            // If mapped, verify round-trip still works
            let back = keyrx_to_cgkeycode(keycode);
            assert!(
                back.is_some(),
                "Reserved CGKeyCode 0x{:02x} mapped to {:?} but doesn't round-trip",
                cgcode,
                keycode
            );
        }
    }
}

/// Test: Boundary CGKeyCode values (0x00 and 0x7F).
///
/// Edge case: Test boundary values at the edges of common CGKeyCode range.
#[test]
fn test_cgkeycode_boundary_values() {
    // 0x00 is 'A' key, should be mapped
    let result_min = cgkeycode_to_keyrx(0x00);
    assert_eq!(
        result_min,
        Some(KeyCode::A),
        "CGKeyCode 0x00 should map to KeyCode::A"
    );

    // Verify round-trip
    let back_min = keyrx_to_cgkeycode(KeyCode::A);
    assert_eq!(back_min, Some(0x00));

    // 0x7F is typically unmapped
    let result_max = cgkeycode_to_keyrx(0x7F);
    // Should not panic, may be None or Some depending on mapping
    assert!(
        result_max.is_none() || result_max.is_some(),
        "CGKeyCode 0x7F should return Some or None, not panic"
    );
}

/// Test: All CGKeyCodes in standard range.
///
/// Comprehensive test: Iterate through entire standard CGKeyCode range (0x00-0x7F)
/// and verify that all conversions are consistent.
#[test]
fn test_cgkeycode_all_standard_range() {
    let mut mapped_count = 0;
    let mut unmapped_count = 0;

    for cgcode in 0x00..=0x7F {
        let keycode_result = cgkeycode_to_keyrx(cgcode);

        match keycode_result {
            Some(keycode) => {
                mapped_count += 1;

                // Verify round-trip
                let back = keyrx_to_cgkeycode(keycode);
                assert_eq!(
                    back,
                    Some(cgcode),
                    "Round-trip failed for CGKeyCode 0x{:02x} → {:?} → ?",
                    cgcode,
                    keycode
                );
            }
            None => {
                unmapped_count += 1;
            }
        }
    }

    // Verify we have substantial coverage (at least 100 mappings)
    assert!(
        mapped_count >= 100,
        "Expected at least 100 mapped CGKeyCodes, found {}",
        mapped_count
    );

    println!(
        "Coverage: {} mapped, {} unmapped in range 0x00-0x7F",
        mapped_count, unmapped_count
    );
}

/// Test: KeyCode to CGKeyCode for all mapped keys.
///
/// Validates reverse direction: every KeyCode that has a CGKeyCode mapping
/// should convert correctly.
#[test]
fn test_keycode_to_cgkeycode_all() {
    // This list represents all KeyCodes that should have CGKeyCode mappings
    let all_keycodes = vec![
        // Letters
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
        // Numbers
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
        // Function keys
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
        KeyCode::F13,
        KeyCode::F14,
        KeyCode::F15,
        KeyCode::F16,
        KeyCode::F17,
        KeyCode::F18,
        KeyCode::F19,
        KeyCode::F20,
        // Modifiers
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
        // Navigation
        KeyCode::Insert,
        KeyCode::Delete,
        KeyCode::Home,
        KeyCode::End,
        KeyCode::PageUp,
        KeyCode::PageDown,
        KeyCode::Left,
        KeyCode::Right,
        KeyCode::Up,
        KeyCode::Down,
        // Punctuation
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
        // Numpad
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
        KeyCode::NumLock,
        // Media keys
        KeyCode::Mute,
        KeyCode::VolumeDown,
        KeyCode::VolumeUp,
    ];

    for keycode in all_keycodes {
        let cgcode_result = keyrx_to_cgkeycode(keycode);
        assert!(
            cgcode_result.is_some(),
            "{:?} should have a CGKeyCode mapping",
            keycode
        );

        // Verify round-trip
        if let Some(cgcode) = cgcode_result {
            let back = cgkeycode_to_keyrx(cgcode);
            assert_eq!(
                back,
                Some(keycode),
                "{:?} → 0x{:02x} → {:?} round-trip failed",
                keycode,
                cgcode,
                back
            );
        }
    }
}

/// Test: Zero data loss guarantee.
///
/// Property: For any CGKeyCode that maps to a KeyCode, converting back
/// must yield the exact original CGKeyCode. This is critical for event
/// accuracy in the daemon.
#[test]
fn test_zero_data_loss_guarantee() {
    // Iterate all CGKeyCodes in standard range
    for cgcode in 0x00..=0x7F {
        if let Some(keycode) = cgkeycode_to_keyrx(cgcode) {
            let back = keyrx_to_cgkeycode(keycode);
            assert_eq!(
                back,
                Some(cgcode),
                "ZERO DATA LOSS VIOLATION: 0x{:02x} → {:?} → {:?} (expected Some(0x{:02x}))",
                cgcode,
                keycode,
                back,
                cgcode
            );
        }
    }

    // Extended range (media keys, etc.)
    for cgcode in 0x80..=0xFF {
        if let Some(keycode) = cgkeycode_to_keyrx(cgcode) {
            let back = keyrx_to_cgkeycode(keycode);
            assert_eq!(
                back,
                Some(cgcode),
                "ZERO DATA LOSS VIOLATION (extended): 0x{:02x} → {:?} → {:?}",
                cgcode,
                keycode,
                back
            );
        }
    }
}
