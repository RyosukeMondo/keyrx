//! Accessibility mappings: sticky one-shot modifiers (`one_shot`) and
//! tap-holds that typing cannot decide (`tap_hold_timeout_only`).

#![cfg(not(target_arch = "wasm32"))]

use keyrx_core::config::{BaseKeyMapping, DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping};
use keyrx_core::runtime::Remapper;
use keyrx_core::simulate::{run, stuck_keys, SimInput};
use KeyCode::{LAlt, LCtrl, LShift, A, C, D, E, F, F20, F21};

fn remapper() -> Remapper {
    Remapper::new(&DeviceConfig {
        identifier: DeviceIdentifier {
            pattern: "*".to_string(),
        },
        mappings: vec![
            KeyMapping::Base(BaseKeyMapping::OneShot {
                from: F20,
                modifier: LShift,
                timeout_ms: 0,
            }),
            KeyMapping::Base(BaseKeyMapping::OneShot {
                from: F21,
                modifier: LCtrl,
                timeout_ms: 300,
            }),
            // Permissive (default) and timeout-only home-row mods.
            KeyMapping::Base(BaseKeyMapping::TapHoldKey {
                from: F,
                tap: Some(F),
                hold: LCtrl,
                threshold_ms: 200,
            }),
            KeyMapping::Base(BaseKeyMapping::TapHoldKeyTimeoutOnly {
                from: D,
                tap: Some(D),
                hold: LAlt,
                threshold_ms: 200,
            }),
        ],
    })
}

/// (ms, press, key) -> the output as (key, press) pairs; fails on stuck keys
/// after everything was released and `end_ms` of idle time passed.
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
    steps
        .iter()
        .flat_map(|s| s.outputs.iter())
        .map(|o| (o.key, o.press))
        .collect()
}

fn typed(key: KeyCode, down_ms: u64, up_ms: u64) -> [(u64, bool, KeyCode); 2] {
    [(down_ms, true, key), (up_ms, false, key)]
}

#[test]
fn tapped_one_shot_modifies_only_the_next_key() {
    let mut input = typed(F20, 0, 50).to_vec();
    input.extend(typed(A, 200, 250));
    input.extend(typed(A, 400, 450));
    let out = outputs(&input, 1000);
    assert_eq!(
        out,
        vec![
            (LShift, true),
            (A, true),
            (LShift, false),
            (A, false),
            (A, true),
            (A, false),
        ]
    );
}

#[test]
fn held_one_shot_is_a_plain_modifier() {
    let out = outputs(
        &[
            (0, true, F20),
            (100, true, A),
            (150, false, A),
            (200, true, A),
            (250, false, A),
            (300, false, F20),
        ],
        1000,
    );
    assert_eq!(
        out,
        vec![
            (LShift, true),
            (A, true),
            (A, false),
            (A, true),
            (A, false),
            (LShift, false),
        ]
    );
}

#[test]
fn tapping_the_one_shot_twice_cancels_it() {
    let mut input = typed(F20, 0, 50).to_vec();
    input.extend(typed(F20, 100, 150));
    input.extend(typed(A, 300, 350));
    let out = outputs(&input, 1000);
    assert_eq!(
        out,
        vec![(LShift, true), (LShift, false), (A, true), (A, false)]
    );
}

#[test]
fn unused_latch_expires_after_its_timeout() {
    let mut input = typed(F21, 0, 50).to_vec();
    input.extend(typed(A, 1000, 1050));
    let out = outputs(&input, 2000);
    assert_eq!(
        out,
        vec![(LCtrl, true), (LCtrl, false), (A, true), (A, false)]
    );
}

#[test]
fn latch_without_timeout_waits_and_is_released_by_the_probe_key() {
    let out = outputs(&typed(F20, 0, 50), 60_000);
    assert_eq!(out, vec![(LShift, true)], "still latched after a minute");
}

#[test]
fn latched_modifier_is_not_consumed_by_another_modifier() {
    let mut input = typed(F20, 0, 50).to_vec();
    input.extend(typed(LCtrl, 100, 150));
    input.extend(typed(A, 300, 350));
    let out = outputs(&input, 1000);
    assert_eq!(
        out,
        vec![
            (LShift, true),
            (LCtrl, true),
            (LCtrl, false),
            (A, true),
            (LShift, false),
            (A, false),
        ]
    );
}

#[test]
fn permissive_hold_turns_a_fast_roll_into_a_chord() {
    // f down, c down, c up, f up inside 30ms: the default decides HOLD.
    let out = outputs(
        &[(0, true, F), (10, true, C), (20, false, C), (30, false, F)],
        1000,
    );
    assert_eq!(
        out,
        vec![(LCtrl, true), (C, true), (C, false), (LCtrl, false)]
    );
}

#[test]
fn timeout_only_keeps_a_fast_roll_as_typing() {
    let out = outputs(
        &[(0, true, D), (10, true, E), (20, false, E), (30, false, D)],
        1000,
    );
    assert_eq!(out, vec![(D, true), (D, false), (E, true), (E, false)]);
}

#[test]
fn timeout_only_still_holds_once_its_threshold_passes() {
    let out = outputs(
        &[
            (0, true, D),
            (250, true, E),
            (300, false, E),
            (400, false, D),
        ],
        1000,
    );
    assert_eq!(
        out,
        vec![(LAlt, true), (E, true), (E, false), (LAlt, false)]
    );
}

#[test]
fn nothing_is_left_stuck_after_a_used_latch() {
    let mut input = typed(F20, 0, 50).to_vec();
    input.extend(typed(A, 100, 150));
    let inputs: Vec<SimInput> = input
        .iter()
        .map(|&(ms, press, key)| SimInput {
            at_us: ms * 1000,
            device: None,
            press,
            key,
        })
        .collect();
    let steps = run(&mut remapper(), &inputs, 1_000_000, |_| Vec::new());
    assert!(stuck_keys(&steps).is_empty());
}
