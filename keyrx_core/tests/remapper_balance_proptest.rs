//! Property: whatever well-formed physical typing arrives, from any mix of
//! devices, the OS-visible output stream is balanced - every output key that
//! went down comes back up, and a probe key typed afterwards still passes
//! through. A press or release swallowed by the tap-hold buffer, the shared
//! modifier/lock state or the output reference counts (`HeldOutputs`) would
//! leave a key stuck down or permanently swallowed. Found by a live run on
//! the Linux daemon where one key stopped typing after a burst.

#![cfg(not(target_arch = "wasm32"))]

use keyrx_core::config::{
    BaseKeyMapping, Condition, DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping,
};
use keyrx_core::runtime::Remapper;
use keyrx_core::simulate::{run, stuck_keys, SimInput};
use proptest::prelude::*;

fn block(pattern: &str, mappings: Vec<KeyMapping>) -> DeviceConfig {
    DeviceConfig {
        identifier: DeviceIdentifier {
            pattern: pattern.to_string(),
        },
        mappings,
    }
}

/// Two keyboards sharing modifier/lock state: dev1 has plain remaps, a
/// tap-hold layer key, a modifier layer, a lock and a modified output; dev2
/// reads dev1's MD_00 layer.
fn remapper() -> Remapper {
    let dev1 = block(
        "dev1",
        vec![
            KeyMapping::simple(KeyCode::A, KeyCode::B),
            KeyMapping::simple(KeyCode::CapsLock, KeyCode::Escape),
            KeyMapping::tap_hold(KeyCode::Space, KeyCode::Space, 0, 200),
            KeyMapping::conditional(
                Condition::ModifierActive(0),
                vec![
                    BaseKeyMapping::Simple {
                        from: KeyCode::H,
                        to: KeyCode::Left,
                    },
                    BaseKeyMapping::Simple {
                        from: KeyCode::J,
                        to: KeyCode::Down,
                    },
                ],
            ),
            KeyMapping::modifier(KeyCode::RAlt, 1),
            KeyMapping::conditional(
                Condition::ModifierActive(1),
                vec![BaseKeyMapping::Simple {
                    from: KeyCode::K,
                    to: KeyCode::Up,
                }],
            ),
            KeyMapping::lock(KeyCode::ScrollLock, 0),
            KeyMapping::conditional(
                Condition::LockActive(0),
                vec![BaseKeyMapping::Simple {
                    from: KeyCode::N,
                    to: KeyCode::M,
                }],
            ),
            KeyMapping::modified_output(KeyCode::F22, KeyCode::C, false, true, false, false),
            // Real-key holds: Tab = Enter on tap / LCtrl on hold, Backspace
            // = LShift on hold only. Tab is remapped in the MD_00 layer, so
            // a layer change while it is held must not strand LCtrl.
            KeyMapping::Base(BaseKeyMapping::TapHoldKey {
                from: KeyCode::Tab,
                tap: Some(KeyCode::Enter),
                hold: KeyCode::LCtrl,
                threshold_ms: 200,
            }),
            KeyMapping::Base(BaseKeyMapping::TapHoldKey {
                from: KeyCode::Backspace,
                tap: None,
                hold: KeyCode::LShift,
                threshold_ms: 150,
            }),
            KeyMapping::conditional(
                Condition::ModifierActive(0),
                vec![BaseKeyMapping::Simple {
                    from: KeyCode::Tab,
                    to: KeyCode::Delete,
                }],
            ),
            // Accessibility: sticky modifiers (with and without a timeout)
            // and a tap-hold that typing cannot decide. They share LShift /
            // LCtrl with the physical keys and the other mappings, so the
            // output reference counts are exercised too.
            KeyMapping::Base(BaseKeyMapping::OneShot {
                from: KeyCode::F20,
                modifier: KeyCode::LShift,
                timeout_ms: 0,
            }),
            KeyMapping::Base(BaseKeyMapping::OneShot {
                from: KeyCode::F21,
                modifier: KeyCode::LCtrl,
                timeout_ms: 300,
            }),
            KeyMapping::Base(BaseKeyMapping::TapHoldKeyTimeoutOnly {
                from: KeyCode::D,
                tap: Some(KeyCode::D),
                hold: KeyCode::LAlt,
                threshold_ms: 200,
            }),
        ],
    );
    let dev2 = block(
        "dev2",
        vec![
            KeyMapping::simple(KeyCode::Z, KeyCode::Y),
            KeyMapping::conditional(
                Condition::ModifierActive(0),
                vec![BaseKeyMapping::Simple {
                    from: KeyCode::X,
                    to: KeyCode::V,
                }],
            ),
        ],
    );
    Remapper::from_blocks(&[dev1, dev2])
}

const KEYS: [KeyCode; 21] = [
    KeyCode::A,
    KeyCode::Z,
    KeyCode::X,
    KeyCode::H,
    KeyCode::J,
    KeyCode::K,
    KeyCode::N,
    KeyCode::Space,
    KeyCode::CapsLock,
    KeyCode::RAlt,
    KeyCode::ScrollLock,
    KeyCode::F22,
    KeyCode::LShift,
    KeyCode::LCtrl,
    KeyCode::E,
    KeyCode::Escape,
    KeyCode::Tab,
    KeyCode::Backspace,
    KeyCode::F20,
    KeyCode::F21,
    KeyCode::D,
];

fn identities(id: &str) -> Vec<String> {
    vec![id.to_string()]
}

/// Turns toggle operations (device, key, gap in ms) into a well-formed
/// stream (a key is pressed only when up and released only when down, per
/// device) that ends with every key released.
fn stream(ops: &[(bool, usize, u64)]) -> (Vec<SimInput>, u64) {
    let mut inputs = Vec::new();
    let mut down: Vec<(bool, KeyCode)> = Vec::new();
    let mut t = 0u64;
    for &(second, key, gap_ms) in ops {
        t += gap_ms * 1000;
        let key = KEYS[key % KEYS.len()];
        let press = !down.contains(&(second, key));
        if press {
            down.push((second, key));
        } else {
            down.retain(|&d| d != (second, key));
        }
        inputs.push(input(t, second, press, key));
    }
    for (second, key) in down {
        t += 1000;
        inputs.push(input(t, second, false, key));
    }
    (inputs, t)
}

fn input(at_us: u64, second: bool, press: bool, key: KeyCode) -> SimInput {
    SimInput {
        at_us,
        device: Some(if second { "dev2" } else { "dev1" }.to_string()),
        press,
        key,
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2000))]

    #[test]
    fn typing_leaves_no_output_key_stuck_or_swallowed(
        ops in proptest::collection::vec((any::<bool>(), 0usize..21, 0u64..400), 1..120)
    ) {
        let mut remapper = remapper();
        let (mut inputs, end) = stream(&ops);
        // Probe: after everything, a pass-through key on each device must
        // still type (press then release), and stay balanced.
        let probe_t = end + 2_000_000;
        for (n, second) in [false, true].into_iter().enumerate() {
            let at = probe_t + n as u64 * 10_000;
            inputs.push(input(at, second, true, KeyCode::Q));
            inputs.push(input(at + 1000, second, false, KeyCode::Q));
        }
        let steps = run(&mut remapper, &inputs, probe_t + 3_000_000, identities);

        prop_assert_eq!(stuck_keys(&steps), vec![], "keys left down: {:?}", ops);
        let probes: Vec<_> = steps
            .iter()
            .flat_map(|s| s.outputs.iter())
            .filter(|o| o.key == KeyCode::Q)
            .collect();
        prop_assert_eq!(probes.len(), 4, "probe keys swallowed: {:?}", ops);
        prop_assert_eq!(remapper.active_modifiers(), Vec::<u8>::new(), "modifier stuck: {:?}", ops);
    }
}
