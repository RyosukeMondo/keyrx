//! Tap-hold whose HOLD presses a REAL key (`tap_hold("CapsLock", "VK_Escape",
//! "VK_LCtrl", 200)`): Caps = Esc on tap, Ctrl on hold; home-row mods.

#![cfg(not(target_arch = "wasm32"))]

use keyrx_core::config::{BaseKeyMapping, DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping};
use keyrx_core::runtime::Remapper;
use keyrx_core::simulate::{run, stuck_keys, SimInput};

fn remapper() -> Remapper {
    Remapper::new(&DeviceConfig {
        identifier: DeviceIdentifier {
            pattern: "*".to_string(),
        },
        mappings: vec![
            KeyMapping::Base(BaseKeyMapping::TapHoldKey {
                from: KeyCode::CapsLock,
                tap: Some(KeyCode::Escape),
                hold: KeyCode::LCtrl,
                threshold_ms: 200,
            }),
            // Home-row mod: A = a on tap, LShift on hold.
            KeyMapping::Base(BaseKeyMapping::TapHoldKey {
                from: KeyCode::A,
                tap: Some(KeyCode::A),
                hold: KeyCode::LShift,
                threshold_ms: 200,
            }),
            KeyMapping::Base(BaseKeyMapping::TapHoldKey {
                from: KeyCode::Tab,
                tap: None,
                hold: KeyCode::LAlt,
                threshold_ms: 200,
            }),
        ],
    })
}

/// (ms, press, key) -> the output as (key, press) pairs.
fn outputs(events: &[(u64, bool, KeyCode)], end_ms: u64) -> Vec<(KeyCode, bool)> {
    let inputs: Vec<SimInput> = events
        .iter()
        .map(|&(ms, press, key)| SimInput {
            at_us: ms * 1000,
            device: None,
            press,
            key,
        })
        .collect();
    let steps = run(&mut remapper(), &inputs, end_ms * 1000, |_| Vec::new());
    assert!(
        stuck_keys(&steps).is_empty(),
        "stuck: {:?}",
        stuck_keys(&steps)
    );
    steps
        .iter()
        .flat_map(|s| s.outputs.iter())
        .map(|o| (o.key, o.press))
        .collect()
}

#[test]
fn tap_types_the_tap_key() {
    let out = outputs(
        &[(0, true, KeyCode::CapsLock), (80, false, KeyCode::CapsLock)],
        500,
    );
    assert_eq!(out, vec![(KeyCode::Escape, true), (KeyCode::Escape, false)]);
}

#[test]
fn hold_presses_the_real_modifier_until_release() {
    let out = outputs(
        &[
            (0, true, KeyCode::CapsLock),
            (600, false, KeyCode::CapsLock),
        ],
        1000,
    );
    assert_eq!(out, vec![(KeyCode::LCtrl, true), (KeyCode::LCtrl, false)]);
}

#[test]
fn modifier_is_down_around_a_key_typed_while_held() {
    // Caps held past the threshold, then C typed: Ctrl+C.
    let out = outputs(
        &[
            (0, true, KeyCode::CapsLock),
            (300, true, KeyCode::C),
            (350, false, KeyCode::C),
            (400, false, KeyCode::CapsLock),
        ],
        1000,
    );
    assert_eq!(
        out,
        vec![
            (KeyCode::LCtrl, true),
            (KeyCode::C, true),
            (KeyCode::C, false),
            (KeyCode::LCtrl, false),
        ]
    );
}

#[test]
fn quick_complete_tap_inside_resolves_hold_permissive() {
    // Home-row mod: A down, S down+up, A up - all inside the threshold.
    let out = outputs(
        &[
            (0, true, KeyCode::A),
            (50, true, KeyCode::S),
            (80, false, KeyCode::S),
            (100, false, KeyCode::A),
        ],
        1000,
    );
    assert_eq!(
        out,
        vec![
            (KeyCode::LShift, true),
            (KeyCode::S, true),
            (KeyCode::S, false),
            (KeyCode::LShift, false),
        ]
    );
}

#[test]
fn rollover_is_typed_not_held() {
    // A down, S down, A up, S up quickly: ordinary typing "as".
    let out = outputs(
        &[
            (0, true, KeyCode::A),
            (30, true, KeyCode::S),
            (60, false, KeyCode::A),
            (90, false, KeyCode::S),
        ],
        1000,
    );
    assert_eq!(
        out,
        vec![
            (KeyCode::A, true),
            (KeyCode::A, false),
            (KeyCode::S, true),
            (KeyCode::S, false),
        ]
    );
}

#[test]
fn hold_only_suppresses_the_tap_and_holds_the_real_key() {
    let tap = outputs(&[(0, true, KeyCode::Tab), (50, false, KeyCode::Tab)], 500);
    assert_eq!(tap, vec![]);
    let hold = outputs(&[(0, true, KeyCode::Tab), (500, false, KeyCode::Tab)], 1000);
    assert_eq!(hold, vec![(KeyCode::LAlt, true), (KeyCode::LAlt, false)]);
}

#[test]
fn a_physical_ctrl_held_too_is_not_released_early() {
    // Physical LCtrl down, then Caps held (also LCtrl) and released: the OS
    // must keep seeing LCtrl down until the physical key is released.
    let out = outputs(
        &[
            (0, true, KeyCode::LCtrl),
            (10, true, KeyCode::CapsLock),
            (400, false, KeyCode::CapsLock),
            (500, false, KeyCode::LCtrl),
        ],
        1000,
    );
    assert_eq!(out, vec![(KeyCode::LCtrl, true), (KeyCode::LCtrl, false)]);
}
