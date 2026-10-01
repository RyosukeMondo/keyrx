//! Runs the shipped `examples/08-one-handed.rhai` through the real engine.

use super::*;
use keyrx_core::runtime::Remapper;
use keyrx_core::simulate::{run, stuck_keys, SimInput};
use KeyCode::{CapsLock, LShift, Num0, Num1, Semicolon, Space, Tab, A, P, Q};

fn typed(events: &[(u64, bool, KeyCode)], end_ms: u64) -> Vec<(KeyCode, bool)> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../examples/08-one-handed.rhai");
    let config = Parser::new().parse_script(&path).expect("example compiles");
    let mut remapper = Remapper::from_blocks(&config.devices);
    let inputs: Vec<SimInput> = events
        .iter()
        .map(|&(ms, press, key)| SimInput {
            at_us: ms * 1000,
            device: None,
            press,
            key,
        })
        .collect();
    let steps = run(&mut remapper, &inputs, end_ms * 1000, |_| Vec::new());
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
fn holding_space_mirrors_the_keyboard() {
    let out = typed(
        &[
            (0, true, Space),
            (50, true, A),
            (80, false, A),
            (120, false, Space),
        ],
        1000,
    );
    assert_eq!(out, vec![(Semicolon, true), (Semicolon, false)]);
}

#[test]
fn tapping_space_types_a_space() {
    let out = typed(&[(0, true, Space), (60, false, Space)], 1000);
    assert_eq!(out, vec![(Space, true), (Space, false)]);
}

#[test]
fn capslock_latches_the_mirror_layer_on_and_off() {
    let out = typed(
        &[
            (0, true, CapsLock),
            (40, false, CapsLock),
            (100, true, Q),
            (140, false, Q),
            (200, true, CapsLock),
            (240, false, CapsLock),
            (300, true, Q),
            (340, false, Q),
        ],
        1000,
    );
    assert_eq!(out, vec![(P, true), (P, false), (Q, true), (Q, false)]);
}

#[test]
fn tab_is_sticky_shift_for_one_key() {
    let out = typed(
        &[
            (0, true, Tab),
            (40, false, Tab),
            (200, true, A),
            (240, false, A),
        ],
        1000,
    );
    assert_eq!(
        out,
        vec![(LShift, true), (A, true), (LShift, false), (A, false)]
    );
}

#[test]
fn number_row_is_mirrored_too() {
    let out = typed(
        &[
            (0, true, Space),
            (50, true, Num1),
            (80, false, Num1),
            (120, false, Space),
        ],
        1000,
    );
    assert_eq!(out, vec![(Num0, true), (Num0, false)]);
}
