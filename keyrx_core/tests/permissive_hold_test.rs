//! Permissive hold on the real engine (`Remapper` via `simulate::run`, the
//! code the daemon runs): typing rollover over a tap-hold key must type, a
//! complete tap inside it must use its layer.

use keyrx_core::config::{
    BaseKeyMapping, Condition, DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping,
};
use keyrx_core::runtime::Remapper;
use keyrx_core::simulate::{run, SimInput};

/// `B`: tap = Enter, hold = MD_00; in MD_00, E → Num2.
fn config() -> DeviceConfig {
    DeviceConfig {
        identifier: DeviceIdentifier {
            pattern: "*".into(),
        },
        mappings: vec![
            KeyMapping::tap_hold(KeyCode::B, KeyCode::Enter, 0, 200),
            KeyMapping::tap_hold(KeyCode::A, KeyCode::Tab, 1, 200),
            KeyMapping::conditional(
                Condition::ModifierActive(0),
                vec![BaseKeyMapping::Simple {
                    from: KeyCode::E,
                    to: KeyCode::Num2,
                }],
            ),
        ],
    }
}

/// Runs `(ms, press, key)` inputs and returns the outputs as `(key, press)`.
fn typed(inputs: &[(u64, bool, KeyCode)]) -> Vec<(KeyCode, bool)> {
    let mut remapper = Remapper::new(&config());
    let inputs: Vec<SimInput> = inputs
        .iter()
        .map(|&(ms, press, key)| SimInput {
            at_us: ms * 1000,
            device: None,
            press,
            key,
        })
        .collect();
    let end = inputs.last().map_or(0, |i| i.at_us) + 1_000_000;
    run(&mut remapper, &inputs, end, |id| vec![id.into()])
        .iter()
        .flat_map(|step| step.outputs.iter().map(|o| (o.key, o.press)))
        .collect()
}

use KeyCode::{Enter, Num2, Tab, A, B, E};

#[test]
fn rollover_out_of_a_tap_hold_key_types_both_keys() {
    let out = typed(&[(0, true, B), (30, true, E), (60, false, B), (90, false, E)]);
    assert_eq!(
        out,
        vec![(Enter, true), (Enter, false), (E, true), (E, false)]
    );
}

#[test]
fn a_complete_tap_inside_a_tap_hold_key_uses_its_layer() {
    let out = typed(&[(0, true, B), (30, true, E), (60, false, E), (90, false, B)]);
    assert_eq!(out, vec![(Num2, true), (Num2, false)]);
}

#[test]
fn holding_past_the_threshold_uses_the_layer() {
    let out = typed(&[
        (0, true, B),
        (300, true, E),
        (320, false, E),
        (340, false, B),
    ]);
    assert_eq!(out, vec![(Num2, true), (Num2, false)]);
}

#[test]
fn a_key_still_down_when_the_threshold_passes_uses_the_layer() {
    let out = typed(&[
        (0, true, B),
        (20, true, E),
        (270, false, E),
        (280, false, B),
    ]);
    assert_eq!(out, vec![(Num2, true), (Num2, false)]);
}

#[test]
fn rolled_tap_hold_keys_both_tap_in_order() {
    let out = typed(&[(0, true, A), (20, true, B), (40, false, A), (60, false, B)]);
    assert_eq!(
        out,
        vec![(Tab, true), (Tab, false), (Enter, true), (Enter, false)]
    );
}

#[test]
fn a_lone_tap_types_the_tap_key() {
    assert_eq!(
        typed(&[(0, true, B), (50, false, B)]),
        vec![(Enter, true), (Enter, false)]
    );
}
