//! E2E tests for the default profile's key remappings
//!
//! Verifies every simple remap and tap-hold tap output in the default profile
//! using pure logic (no Windows hooks needed).
//!
//! Run with: cargo test -p keyrx_core --test default_profile_remap

#![cfg(not(target_arch = "wasm32"))]

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use keyrx_core::config::{
    BaseKeyMapping, Condition, DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping,
};
use keyrx_core::runtime::{
    check_tap_hold_timeouts, process_event, DeviceState, KeyEvent, KeyLookup,
};

/// Build the default profile's DeviceConfig matching default.rhai
fn default_profile_config() -> DeviceConfig {
    let mut mappings: Vec<KeyMapping> = Vec::new();

    // === TapHold Modifiers ===
    mappings.push(KeyMapping::tap_hold(KeyCode::B, KeyCode::Enter, 0, 200));
    mappings.push(KeyMapping::tap_hold(KeyCode::V, KeyCode::Delete, 1, 200));
    mappings.push(KeyMapping::tap_hold(KeyCode::M, KeyCode::Backspace, 2, 200));
    mappings.push(KeyMapping::tap_hold(KeyCode::X, KeyCode::Delete, 3, 200));
    mappings.push(KeyMapping::tap_hold(KeyCode::Num1, KeyCode::Num1, 4, 200));
    mappings.push(KeyMapping::tap_hold(KeyCode::LCtrl, KeyCode::Space, 5, 200));
    mappings.push(KeyMapping::tap_hold(KeyCode::C, KeyCode::Delete, 6, 200));
    mappings.push(KeyMapping::tap_hold(KeyCode::Tab, KeyCode::Space, 7, 200));
    mappings.push(KeyMapping::tap_hold(KeyCode::Q, KeyCode::Minus, 8, 200));
    mappings.push(KeyMapping::tap_hold(KeyCode::A, KeyCode::Tab, 9, 200));
    mappings.push(KeyMapping::tap_hold(KeyCode::N, KeyCode::N, 10, 200));

    // === MD_00 Layer (B hold) ===
    mappings.push(KeyMapping::conditional(
        Condition::ModifierActive(0),
        vec![
            BaseKeyMapping::Simple {
                from: KeyCode::Zenkaku,
                to: KeyCode::Enter,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::Num2,
                to: KeyCode::Left,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::Num3,
                to: KeyCode::Right,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::Num4,
                to: KeyCode::Down,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::Num5,
                to: KeyCode::Up,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::Num8,
                to: KeyCode::Home,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::Num9,
                to: KeyCode::End,
            },
            BaseKeyMapping::ModifiedOutput {
                from: KeyCode::Slash,
                to: KeyCode::Num2,
                shift: true,
                ctrl: false,
                alt: false,
                win: false,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::Minus,
                to: KeyCode::F12,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::W,
                to: KeyCode::Num1,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::E,
                to: KeyCode::Num2,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::R,
                to: KeyCode::Num3,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::T,
                to: KeyCode::Num4,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::Y,
                to: KeyCode::Num5,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::U,
                to: KeyCode::Num6,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::I,
                to: KeyCode::Num7,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::O,
                to: KeyCode::Num8,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::P,
                to: KeyCode::Num9,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::LeftBracket,
                to: KeyCode::Num0,
            },
            // Function keys on home row
            BaseKeyMapping::Simple {
                from: KeyCode::S,
                to: KeyCode::F1,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::D,
                to: KeyCode::F2,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::F,
                to: KeyCode::F3,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::G,
                to: KeyCode::F4,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::H,
                to: KeyCode::F5,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::J,
                to: KeyCode::F6,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::K,
                to: KeyCode::F7,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::L,
                to: KeyCode::F8,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::Semicolon,
                to: KeyCode::F9,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::Quote,
                to: KeyCode::F10,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::Backslash,
                to: KeyCode::F11,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::RightBracket,
                to: KeyCode::F12,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::Space,
                to: KeyCode::F12,
            },
        ],
    ));

    // === Modified Output ===
    mappings.push(KeyMapping::modified_output(
        KeyCode::Num2,
        KeyCode::Num7,
        true,
        false,
        false,
        false,
    )); // 2 -> Shift+7 (')

    // === Simple Remappings ===
    mappings.push(KeyMapping::simple(KeyCode::LeftBracket, KeyCode::S));
    mappings.push(KeyMapping::simple(KeyCode::RightBracket, KeyCode::Minus));
    mappings.push(KeyMapping::simple(KeyCode::Quote, KeyCode::Z));
    mappings.push(KeyMapping::simple(KeyCode::Equal, KeyCode::Slash));

    // Letter row
    mappings.push(KeyMapping::simple(KeyCode::D, KeyCode::Q));
    mappings.push(KeyMapping::simple(KeyCode::E, KeyCode::O));
    mappings.push(KeyMapping::simple(KeyCode::F, KeyCode::J));
    mappings.push(KeyMapping::simple(KeyCode::G, KeyCode::K));
    mappings.push(KeyMapping::simple(KeyCode::H, KeyCode::X));
    mappings.push(KeyMapping::simple(KeyCode::I, KeyCode::H));
    mappings.push(KeyMapping::simple(KeyCode::J, KeyCode::B));
    mappings.push(KeyMapping::simple(KeyCode::K, KeyCode::M));
    mappings.push(KeyMapping::simple(KeyCode::L, KeyCode::W));
    mappings.push(KeyMapping::simple(KeyCode::O, KeyCode::T));
    mappings.push(KeyMapping::simple(KeyCode::P, KeyCode::N));
    mappings.push(KeyMapping::simple(KeyCode::R, KeyCode::E));
    mappings.push(KeyMapping::simple(KeyCode::S, KeyCode::Semicolon));
    mappings.push(KeyMapping::simple(KeyCode::T, KeyCode::U));
    mappings.push(KeyMapping::simple(KeyCode::U, KeyCode::D));
    mappings.push(KeyMapping::simple(KeyCode::W, KeyCode::A));
    mappings.push(KeyMapping::simple(KeyCode::Y, KeyCode::I));

    // Number row
    mappings.push(KeyMapping::simple(KeyCode::Num3, KeyCode::Comma));
    mappings.push(KeyMapping::simple(KeyCode::Num4, KeyCode::Period));
    mappings.push(KeyMapping::simple(KeyCode::Num5, KeyCode::P));
    mappings.push(KeyMapping::simple(KeyCode::Num6, KeyCode::Y));
    mappings.push(KeyMapping::simple(KeyCode::Num7, KeyCode::F));
    mappings.push(KeyMapping::simple(KeyCode::Num8, KeyCode::G));
    mappings.push(KeyMapping::simple(KeyCode::Num9, KeyCode::C));
    mappings.push(KeyMapping::simple(KeyCode::Num0, KeyCode::R));

    // Function keys
    mappings.push(KeyMapping::simple(KeyCode::F1, KeyCode::LMeta));
    mappings.push(KeyMapping::simple(KeyCode::F2, KeyCode::Escape));
    mappings.push(KeyMapping::simple(KeyCode::F3, KeyCode::LCtrl));
    mappings.push(KeyMapping::simple(KeyCode::F4, KeyCode::LAlt));
    mappings.push(KeyMapping::simple(KeyCode::F5, KeyCode::Backspace));
    mappings.push(KeyMapping::simple(KeyCode::F6, KeyCode::Delete));
    mappings.push(KeyMapping::simple(KeyCode::F8, KeyCode::Tab));
    mappings.push(KeyMapping::simple(KeyCode::F9, KeyCode::Tab));
    mappings.push(KeyMapping::simple(KeyCode::F10, KeyCode::Tab));
    mappings.push(KeyMapping::simple(KeyCode::F11, KeyCode::Tab));
    mappings.push(KeyMapping::simple(KeyCode::F12, KeyCode::Tab));

    // Special keys
    mappings.push(KeyMapping::simple(KeyCode::Escape, KeyCode::Num5));
    mappings.push(KeyMapping::simple(KeyCode::Backspace, KeyCode::Delete));
    mappings.push(KeyMapping::simple(KeyCode::Delete, KeyCode::Num4));
    mappings.push(KeyMapping::simple(KeyCode::Enter, KeyCode::Yen));

    // Modifier keys
    mappings.push(KeyMapping::simple(KeyCode::LAlt, KeyCode::LCtrl));

    // Punctuation
    mappings.push(KeyMapping::simple(KeyCode::Minus, KeyCode::L));
    mappings.push(KeyMapping::simple(KeyCode::Semicolon, KeyCode::V));
    mappings.push(KeyMapping::simple(KeyCode::Comma, KeyCode::F9));
    mappings.push(KeyMapping::simple(KeyCode::Period, KeyCode::F10));
    mappings.push(KeyMapping::simple(KeyCode::Slash, KeyCode::F11));
    mappings.push(KeyMapping::simple(KeyCode::Ro, KeyCode::F12));

    // Japanese IME keys
    // Note: Muhenkan/Henkan/Hiragana/Katakana may not have KeyCode variants
    mappings.push(KeyMapping::simple(KeyCode::Zenkaku, KeyCode::Escape));

    DeviceConfig {
        identifier: DeviceIdentifier {
            pattern: String::from("*"),
        },
        mappings,
    }
}

/// Helper: assert simple remap (press X -> press Y, release X -> release Y)
fn assert_simple_remap(from: KeyCode, to: KeyCode, lookup: &KeyLookup, state: &mut DeviceState) {
    let press_out = process_event(KeyEvent::Press(from), lookup, state);
    assert_eq!(
        press_out.len(),
        1,
        "Press {:?} should produce 1 event, got {}",
        from,
        press_out.len()
    );
    assert_eq!(
        press_out[0].keycode(),
        to,
        "Press {:?} should map to {:?}, got {:?}",
        from,
        to,
        press_out[0].keycode()
    );
    assert!(press_out[0].is_press(), "Should be press event");

    let release_out = process_event(KeyEvent::Release(from), lookup, state);
    assert_eq!(
        release_out.len(),
        1,
        "Release {:?} should produce 1 event",
        from
    );
    assert_eq!(
        release_out[0].keycode(),
        to,
        "Release {:?} should map to {:?}",
        from,
        to
    );
    assert!(release_out[0].is_release(), "Should be release event");
}

// ============================================================
// Simple Remap Tests
// ============================================================

#[test]
fn test_letter_remaps() {
    let config = default_profile_config();
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // All letter remaps from the default profile (Dvorak-like)
    let letter_remaps = [
        (KeyCode::D, KeyCode::Q),
        (KeyCode::E, KeyCode::O),
        (KeyCode::F, KeyCode::J),
        (KeyCode::G, KeyCode::K),
        (KeyCode::H, KeyCode::X),
        (KeyCode::I, KeyCode::H),
        (KeyCode::J, KeyCode::B),
        (KeyCode::K, KeyCode::M),
        (KeyCode::L, KeyCode::W),
        (KeyCode::O, KeyCode::T),
        (KeyCode::P, KeyCode::N),
        (KeyCode::R, KeyCode::E),
        (KeyCode::S, KeyCode::Semicolon),
        (KeyCode::T, KeyCode::U),
        (KeyCode::U, KeyCode::D),
        (KeyCode::W, KeyCode::A),
        (KeyCode::Y, KeyCode::I),
    ];

    for (from, to) in &letter_remaps {
        assert_simple_remap(*from, *to, &lookup, &mut state);
    }
}

#[test]
fn test_number_row_remaps() {
    let config = default_profile_config();
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    let number_remaps = [
        (KeyCode::Num3, KeyCode::Comma),
        (KeyCode::Num4, KeyCode::Period),
        (KeyCode::Num5, KeyCode::P),
        (KeyCode::Num6, KeyCode::Y),
        (KeyCode::Num7, KeyCode::F),
        (KeyCode::Num8, KeyCode::G),
        (KeyCode::Num9, KeyCode::C),
        (KeyCode::Num0, KeyCode::R),
    ];

    for (from, to) in &number_remaps {
        assert_simple_remap(*from, *to, &lookup, &mut state);
    }
}

#[test]
fn test_function_key_remaps() {
    let config = default_profile_config();
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    let fkey_remaps = [
        (KeyCode::F1, KeyCode::LMeta),
        (KeyCode::F2, KeyCode::Escape),
        (KeyCode::F3, KeyCode::LCtrl),
        (KeyCode::F4, KeyCode::LAlt),
        (KeyCode::F5, KeyCode::Backspace),
        (KeyCode::F6, KeyCode::Delete),
        (KeyCode::F8, KeyCode::Tab),
        (KeyCode::F9, KeyCode::Tab),
        (KeyCode::F10, KeyCode::Tab),
        (KeyCode::F11, KeyCode::Tab),
        (KeyCode::F12, KeyCode::Tab),
    ];

    for (from, to) in &fkey_remaps {
        assert_simple_remap(*from, *to, &lookup, &mut state);
    }
}

#[test]
fn test_special_key_remaps() {
    let config = default_profile_config();
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    assert_simple_remap(KeyCode::Escape, KeyCode::Num5, &lookup, &mut state);
    assert_simple_remap(KeyCode::Backspace, KeyCode::Delete, &lookup, &mut state);
    assert_simple_remap(KeyCode::Delete, KeyCode::Num4, &lookup, &mut state);
    assert_simple_remap(KeyCode::Enter, KeyCode::Yen, &lookup, &mut state);
    assert_simple_remap(KeyCode::LAlt, KeyCode::LCtrl, &lookup, &mut state);
}

#[test]
fn test_punctuation_remaps() {
    let config = default_profile_config();
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    assert_simple_remap(KeyCode::LeftBracket, KeyCode::S, &lookup, &mut state);
    assert_simple_remap(KeyCode::RightBracket, KeyCode::Minus, &lookup, &mut state);
    assert_simple_remap(KeyCode::Quote, KeyCode::Z, &lookup, &mut state);
    assert_simple_remap(KeyCode::Equal, KeyCode::Slash, &lookup, &mut state);
    assert_simple_remap(KeyCode::Minus, KeyCode::L, &lookup, &mut state);
    assert_simple_remap(KeyCode::Semicolon, KeyCode::V, &lookup, &mut state);
    assert_simple_remap(KeyCode::Comma, KeyCode::F9, &lookup, &mut state);
    assert_simple_remap(KeyCode::Period, KeyCode::F10, &lookup, &mut state);
    assert_simple_remap(KeyCode::Slash, KeyCode::F11, &lookup, &mut state);
    assert_simple_remap(KeyCode::Ro, KeyCode::F12, &lookup, &mut state);
}

// ============================================================
// TapHold Tap Tests (quick press+release = tap output)
// ============================================================

#[test]
fn test_taphold_b_tap_outputs_enter() {
    let config = default_profile_config();
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Quick tap: press then immediately release (no timeout)
    let press_out = process_event(KeyEvent::Press(KeyCode::B), &lookup, &mut state);
    // TapHold press is held pending - no output yet
    assert_eq!(press_out.len(), 0, "TapHold press should be pending");

    let release_out = process_event(KeyEvent::Release(KeyCode::B), &lookup, &mut state);
    // Quick release = tap output (Enter)
    assert!(
        release_out.iter().any(|e| e.keycode() == KeyCode::Enter),
        "TapHold B tap should output Enter, got: {:?}",
        release_out
    );
}

#[test]
fn test_taphold_v_tap_outputs_delete() {
    let config = default_profile_config();
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    let _ = process_event(KeyEvent::Press(KeyCode::V), &lookup, &mut state);
    let release_out = process_event(KeyEvent::Release(KeyCode::V), &lookup, &mut state);
    assert!(
        release_out.iter().any(|e| e.keycode() == KeyCode::Delete),
        "TapHold V tap should output Delete, got: {:?}",
        release_out
    );
}

#[test]
fn test_taphold_m_tap_outputs_backspace() {
    let config = default_profile_config();
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    let _ = process_event(KeyEvent::Press(KeyCode::M), &lookup, &mut state);
    let release_out = process_event(KeyEvent::Release(KeyCode::M), &lookup, &mut state);
    assert!(
        release_out
            .iter()
            .any(|e| e.keycode() == KeyCode::Backspace),
        "TapHold M tap should output Backspace, got: {:?}",
        release_out
    );
}

#[test]
fn test_taphold_q_tap_outputs_minus() {
    let config = default_profile_config();
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    let _ = process_event(KeyEvent::Press(KeyCode::Q), &lookup, &mut state);
    let release_out = process_event(KeyEvent::Release(KeyCode::Q), &lookup, &mut state);
    assert!(
        release_out.iter().any(|e| e.keycode() == KeyCode::Minus),
        "TapHold Q tap should output Minus, got: {:?}",
        release_out
    );
}

#[test]
fn test_taphold_a_tap_outputs_tab() {
    let config = default_profile_config();
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    let _ = process_event(KeyEvent::Press(KeyCode::A), &lookup, &mut state);
    let release_out = process_event(KeyEvent::Release(KeyCode::A), &lookup, &mut state);
    assert!(
        release_out.iter().any(|e| e.keycode() == KeyCode::Tab),
        "TapHold A tap should output Tab, got: {:?}",
        release_out
    );
}

#[test]
fn test_taphold_lctrl_tap_outputs_space() {
    let config = default_profile_config();
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    let _ = process_event(KeyEvent::Press(KeyCode::LCtrl), &lookup, &mut state);
    let release_out = process_event(KeyEvent::Release(KeyCode::LCtrl), &lookup, &mut state);
    assert!(
        release_out.iter().any(|e| e.keycode() == KeyCode::Space),
        "TapHold LCtrl tap should output Space, got: {:?}",
        release_out
    );
}

#[test]
fn test_taphold_tab_tap_outputs_space() {
    let config = default_profile_config();
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    let _ = process_event(KeyEvent::Press(KeyCode::Tab), &lookup, &mut state);
    let release_out = process_event(KeyEvent::Release(KeyCode::Tab), &lookup, &mut state);
    assert!(
        release_out.iter().any(|e| e.keycode() == KeyCode::Space),
        "TapHold Tab tap should output Space, got: {:?}",
        release_out
    );
}

#[test]
fn test_taphold_n_tap_outputs_n() {
    let config = default_profile_config();
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    let _ = process_event(KeyEvent::Press(KeyCode::N), &lookup, &mut state);
    let release_out = process_event(KeyEvent::Release(KeyCode::N), &lookup, &mut state);
    assert!(
        release_out.iter().any(|e| e.keycode() == KeyCode::N),
        "TapHold N tap should output N, got: {:?}",
        release_out
    );
}

// ============================================================
// TapHold Hold Tests (hold past threshold = activate modifier)
// ============================================================

#[test]
fn test_taphold_b_hold_activates_md00() {
    let config = default_profile_config();
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Press B with timestamp 0
    let press = KeyEvent::press(KeyCode::B).with_timestamp(0);
    let _ = process_event(press, &lookup, &mut state);

    // Simulate timeout (200ms = 200_000 us)
    let _timeout_out = check_tap_hold_timeouts(300_000, &mut state);
    // After timeout, MD_00 should be active
    assert!(
        state.is_modifier_active(0),
        "After B hold timeout, MD_00 should be active"
    );

    // Release B
    let _release_out = process_event(KeyEvent::Release(KeyCode::B), &lookup, &mut state);
    assert!(
        !state.is_modifier_active(0),
        "After B release, MD_00 should be inactive"
    );
}

// ============================================================
// MD_00 Layer Tests (B hold + key)
// ============================================================

#[test]
fn test_md00_navigation_keys() {
    let config = default_profile_config();
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Activate MD_00 via hold
    let press = KeyEvent::press(KeyCode::B).with_timestamp(0);
    let _ = process_event(press, &lookup, &mut state);
    let _ = check_tap_hold_timeouts(300_000, &mut state);
    assert!(state.is_modifier_active(0));

    // Now test navigation keys while MD_00 is active
    let nav_remaps = [
        (KeyCode::Num2, KeyCode::Left),
        (KeyCode::Num3, KeyCode::Right),
        (KeyCode::Num4, KeyCode::Down),
        (KeyCode::Num5, KeyCode::Up),
        (KeyCode::Num8, KeyCode::Home),
        (KeyCode::Num9, KeyCode::End),
    ];

    for (from, to) in &nav_remaps {
        let out = process_event(KeyEvent::Press(*from), &lookup, &mut state);
        assert_eq!(out.len(), 1, "MD_00 + {:?} should produce 1 event", from);
        assert_eq!(
            out[0].keycode(),
            *to,
            "MD_00 + {:?} should map to {:?}, got {:?}",
            from,
            to,
            out[0].keycode()
        );
        let _ = process_event(KeyEvent::Release(*from), &lookup, &mut state);
    }

    // Release B
    let _ = process_event(KeyEvent::Release(KeyCode::B), &lookup, &mut state);
}

#[test]
fn test_md00_number_keys() {
    let config = default_profile_config();
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Activate MD_00
    let press = KeyEvent::press(KeyCode::B).with_timestamp(0);
    let _ = process_event(press, &lookup, &mut state);
    let _ = check_tap_hold_timeouts(300_000, &mut state);

    // Numbers on letter row: W->1, E->2, R->3, T->4, Y->5, U->6, I->7, O->8, P->9, [->0
    let number_remaps = [
        (KeyCode::W, KeyCode::Num1),
        (KeyCode::E, KeyCode::Num2),
        (KeyCode::R, KeyCode::Num3),
        (KeyCode::T, KeyCode::Num4),
        (KeyCode::Y, KeyCode::Num5),
        (KeyCode::U, KeyCode::Num6),
        (KeyCode::I, KeyCode::Num7),
        (KeyCode::O, KeyCode::Num8),
        (KeyCode::P, KeyCode::Num9),
        (KeyCode::LeftBracket, KeyCode::Num0),
    ];

    for (from, to) in &number_remaps {
        let out = process_event(KeyEvent::Press(*from), &lookup, &mut state);
        assert_eq!(out.len(), 1, "MD_00 + {:?} should produce 1 event", from);
        assert_eq!(
            out[0].keycode(),
            *to,
            "MD_00 + {:?} should map to {:?}",
            from,
            to
        );
        let _ = process_event(KeyEvent::Release(*from), &lookup, &mut state);
    }

    let _ = process_event(KeyEvent::Release(KeyCode::B), &lookup, &mut state);
}

#[test]
fn test_md00_function_keys() {
    let config = default_profile_config();
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Activate MD_00
    let press = KeyEvent::press(KeyCode::B).with_timestamp(0);
    let _ = process_event(press, &lookup, &mut state);
    let _ = check_tap_hold_timeouts(300_000, &mut state);

    let fkey_remaps = [
        (KeyCode::S, KeyCode::F1),
        (KeyCode::D, KeyCode::F2),
        (KeyCode::F, KeyCode::F3),
        (KeyCode::G, KeyCode::F4),
        (KeyCode::H, KeyCode::F5),
        (KeyCode::J, KeyCode::F6),
        (KeyCode::K, KeyCode::F7),
        (KeyCode::L, KeyCode::F8),
        (KeyCode::Semicolon, KeyCode::F9),
        (KeyCode::Quote, KeyCode::F10),
        (KeyCode::Backslash, KeyCode::F11),
        (KeyCode::RightBracket, KeyCode::F12),
        (KeyCode::Space, KeyCode::F12),
    ];

    for (from, to) in &fkey_remaps {
        let out = process_event(KeyEvent::Press(*from), &lookup, &mut state);
        assert_eq!(out.len(), 1, "MD_00 + {:?} should produce 1 event", from);
        assert_eq!(
            out[0].keycode(),
            *to,
            "MD_00 + {:?} should map to {:?}",
            from,
            to
        );
        let _ = process_event(KeyEvent::Release(*from), &lookup, &mut state);
    }

    let _ = process_event(KeyEvent::Release(KeyCode::B), &lookup, &mut state);
}

// ============================================================
// Modified Output Test
// ============================================================

#[test]
fn test_num2_outputs_shift_7() {
    let config = default_profile_config();
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Num2 -> Shift+7 (apostrophe on JIS)
    let out = process_event(KeyEvent::Press(KeyCode::Num2), &lookup, &mut state);
    // Modified output produces: [LShift press, Num7 press]
    assert!(
        out.iter().any(|e| e.keycode() == KeyCode::Num7),
        "Num2 should produce Num7 in output, got: {:?}",
        out
    );
    assert!(
        out.iter().any(|e| e.keycode() == KeyCode::LShift),
        "Num2 should produce LShift in output, got: {:?}",
        out
    );
}

// ============================================================
// Unmapped key passthrough
// ============================================================

#[test]
fn test_z_passthrough() {
    let config = default_profile_config();
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Z is not remapped in the default profile
    assert_simple_remap(KeyCode::Z, KeyCode::Z, &lookup, &mut state);
}

// ============================================================
// IME key remaps
// ============================================================

#[test]
fn test_zenkaku_remaps_to_escape() {
    let config = default_profile_config();
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    assert_simple_remap(KeyCode::Zenkaku, KeyCode::Escape, &lookup, &mut state);
}
