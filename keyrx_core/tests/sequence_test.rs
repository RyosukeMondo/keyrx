//! `sequence(key, [keys])`: one press types the keys in order.

#![cfg(not(target_arch = "wasm32"))]

use keyrx_core::config::{BaseKeyMapping, DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping};
use keyrx_core::runtime::Remapper;
use keyrx_core::simulate::{run, SimInput};

#[test]
fn press_types_every_key_and_release_emits_nothing_more() {
    let mut remapper = Remapper::new(&DeviceConfig {
        identifier: DeviceIdentifier {
            pattern: "*".to_string(),
        },
        mappings: vec![KeyMapping::Base(BaseKeyMapping::Sequence {
            from: KeyCode::F21,
            keys: vec![KeyCode::F22, KeyCode::A, KeyCode::F22],
        })],
    });
    let inputs: Vec<SimInput> = [(0, true), (80_000, false)]
        .iter()
        .map(|&(at_us, press)| SimInput {
            at_us,
            device: None,
            press,
            key: KeyCode::F21,
        })
        .collect();
    let steps = run(&mut remapper, &inputs, 1_000_000, |_| Vec::new());
    let out: Vec<Vec<(KeyCode, bool)>> = steps
        .iter()
        .map(|s| s.outputs.iter().map(|o| (o.key, o.press)).collect())
        .collect();
    assert_eq!(
        out[0],
        vec![
            (KeyCode::F22, true),
            (KeyCode::F22, false),
            (KeyCode::A, true),
            (KeyCode::A, false),
            (KeyCode::F22, true),
            (KeyCode::F22, false),
        ]
    );
    // Releasing the trigger must not replay key-ups for keys that are up.
    assert!(out[1].is_empty(), "spurious releases: {:?}", out[1]);
}
