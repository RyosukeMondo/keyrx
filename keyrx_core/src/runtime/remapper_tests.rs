//! Tests for `Remapper` (split from remapper.rs for the file-size gate).
use super::*;
use crate::config::{BaseKeyMapping, Condition, DeviceIdentifier, KeyCode, KeyMapping};
use alloc::string::ToString;

fn block(pattern: &str, from: KeyCode, to: KeyCode) -> DeviceConfig {
    DeviceConfig {
        identifier: DeviceIdentifier {
            pattern: pattern.to_string(),
        },
        mappings: alloc::vec![KeyMapping::simple(from, to)],
    }
}

fn names(name: &'static str) -> impl FnOnce(&str) -> Vec<String> {
    move |id| alloc::vec![id.to_string(), name.to_string()]
}

fn output(state: &mut Remapper, id: &str, name: &'static str) -> Option<KeyCode> {
    let routed = state.route(Some(id), names(name))?;
    match routed.lookup.find_mapping(KeyCode::A, routed.state)? {
        BaseKeyMapping::Simple { to, .. } => Some(*to),
        _ => None,
    }
}

#[test]
fn devices_use_the_first_matching_block() {
    let mut state = Remapper::from_blocks(&[
        block("*numpad*", KeyCode::A, KeyCode::B),
        block("*", KeyCode::A, KeyCode::C),
    ]);
    assert_eq!(output(&mut state, "path-7", "USB NumPad"), Some(KeyCode::B));
    assert_eq!(
        output(&mut state, "path-3", "USB Keyboard"),
        Some(KeyCode::C)
    );
}

#[test]
fn unmatched_devices_pass_through() {
    let mut state = Remapper::from_blocks(&[block("*numpad*", KeyCode::A, KeyCode::B)]);
    assert!(state.route(Some("path-3"), names("USB Keyboard")).is_none());
    assert!(state.state_of(Some("path-3")).is_none());
    assert_eq!(state.states_mut().count(), 0);
}

/// Custom modifiers/locks are ONE state shared by every routed device
/// (DSL manual §"Cross-Device State Sharing"); only tap-hold pending
/// state and press/release output tracking stay per-device.
#[test]
fn modifier_state_is_shared_across_devices_but_pressed_keys_are_not() {
    let mut state = Remapper::new(&block("*", KeyCode::A, KeyCode::B));
    state
        .route(Some("kbd-1"), names("One"))
        .unwrap()
        .state
        .set_modifier(0);
    let other = state.route(Some("kbd-2"), names("Two")).unwrap();
    assert!(
        other.state.is_modifier_active(0),
        "kbd-2 must see kbd-1's modifier - modifier/lock state is shared"
    );

    // Press/release tracking, in contrast, stays private to each device:
    // releasing A on kbd-2 (which never pressed it) must not disturb
    // kbd-1's tracked press.
    state
        .route(Some("kbd-1"), names("One"))
        .unwrap()
        .state
        .record_press(KeyCode::A, &[KeyCode::LShift, KeyCode::B]);
    state
        .route(Some("kbd-2"), names("Two"))
        .unwrap()
        .state
        .clear_press(KeyCode::A);
    let kbd1 = state.state_of(Some("kbd-1")).unwrap();
    assert_eq!(
        kbd1.get_release_key(KeyCode::A).as_slice(),
        &[KeyCode::LShift, KeyCode::B],
        "kbd-1's press tracking must be untouched by a release on kbd-2"
    );
}

/// A foot pedal (device A) used as a layer key for a separate keyboard
/// (device B): holding the pedal's F24 sets MD_00, which a `when`
/// mapping on B's block reacts to - the scenario the DSL manual's
/// cross-device section documents.
#[test]
fn a_layer_key_on_one_device_affects_mappings_on_another() {
    let pedal = DeviceConfig {
        identifier: DeviceIdentifier {
            pattern: "*pedal*".to_string(),
        },
        mappings: alloc::vec![KeyMapping::modifier(KeyCode::F24, 0)],
    };
    let keyboard = DeviceConfig {
        identifier: DeviceIdentifier {
            pattern: "*".to_string(),
        },
        mappings: alloc::vec![KeyMapping::conditional(
            Condition::ModifierActive(0),
            alloc::vec![BaseKeyMapping::Simple {
                from: KeyCode::A,
                to: KeyCode::B,
            }],
        )],
    };
    let mut remap = Remapper::from_blocks(&[pedal, keyboard]);

    // Before the pedal is held, A on the keyboard passes through
    // unmapped (no matching mapping for A on the base layer).
    let before = remap.process(
        KeyEvent::press(KeyCode::A),
        |id| alloc::vec![id.to_string()],
        None,
    );
    assert!(!before.triggered);

    // Hold the pedal: sets the shared MD_00.
    let hold = remap.process(
        KeyEvent::press(KeyCode::F24).with_device_id("pedal-1".to_string()),
        |id| alloc::vec![id.to_string()],
        None,
    );
    assert!(hold.triggered);

    // Now A on the keyboard (a different device) is remapped to B.
    let after = remap.process(
        KeyEvent::press(KeyCode::A).with_device_id("keyboard-1".to_string()),
        |id| alloc::vec![id.to_string()],
        None,
    );
    assert!(after.triggered);
    assert_eq!(after.outputs.len(), 1);
    assert_eq!(after.outputs[0].keycode(), KeyCode::B);
}

#[test]
fn identities_are_resolved_once_per_device() {
    let mut state = Remapper::new(&block("*", KeyCode::A, KeyCode::B));
    let mut calls = 0;
    for _ in 0..3 {
        state.route(Some("kbd"), |id| {
            calls += 1;
            alloc::vec![id.to_string()]
        });
    }
    assert_eq!(calls, 1);
}

#[test]
fn events_without_device_use_a_wildcard_block() {
    let mut wildcard = Remapper::new(&block("*", KeyCode::A, KeyCode::B));
    assert!(wildcard.route(None, |_| unreachable!()).is_some());
    let mut specific = Remapper::new(&block("*numpad*", KeyCode::A, KeyCode::B));
    assert!(specific.route(None, |_| unreachable!()).is_none());
}

#[test]
fn active_layer_is_the_first_active_layer_modifier() {
    let config = DeviceConfig {
        identifier: DeviceIdentifier {
            pattern: "*".to_string(),
        },
        mappings: alloc::vec![
            KeyMapping::Conditional {
                condition: Condition::ModifierActive(0x0A),
                mappings: alloc::vec![BaseKeyMapping::Simple {
                    from: KeyCode::H,
                    to: KeyCode::Left,
                }],
            },
            KeyMapping::Conditional {
                condition: Condition::ModifierActive(0x02),
                mappings: alloc::vec![BaseKeyMapping::Simple {
                    from: KeyCode::J,
                    to: KeyCode::Down,
                }],
            },
        ],
    };
    let mut remap = Remapper::new(&config);
    let routed = remap
        .route(Some("kbd"), |id| alloc::vec![id.to_string()])
        .unwrap();
    assert_eq!(routed.layers, &[0x0A, 0x02]);
    assert_eq!(active_layer(routed.state, routed.layers), None);
    routed.state.set_modifier(0x02);
    assert_eq!(active_layer(routed.state, routed.layers), Some(0x02));
    routed.state.set_modifier(0x0A);
    assert_eq!(active_layer(routed.state, routed.layers), Some(0x0A));
}

#[test]
fn tick_fires_tap_hold_timeouts_for_every_routed_device() {
    let config = DeviceConfig {
        identifier: DeviceIdentifier {
            pattern: "*".to_string(),
        },
        mappings: alloc::vec![KeyMapping::tap_hold(
            KeyCode::CapsLock,
            KeyCode::Escape,
            0,
            200
        )],
    };
    let mut remap = Remapper::new(&config);
    let remapped = remap.process(
        KeyEvent::press(KeyCode::CapsLock).with_timestamp(0),
        |id| alloc::vec![id.to_string()],
        None,
    );
    assert!(
        remapped.outputs.is_empty(),
        "press alone must not resolve tap/hold yet"
    );

    // Past the 200ms threshold: ticking must fire the hold (activate MD_00).
    let events = remap.tick(250_000);
    assert!(
        events.is_empty(),
        "hold-only activation has no KeyEvent output"
    );
    assert!(remap.active_modifiers().contains(&0));
}

#[test]
fn process_reports_pass_through_for_unmatched_devices() {
    let mut remap = Remapper::from_blocks(&[block("*numpad*", KeyCode::A, KeyCode::B)]);
    let remapped = remap.process(
        KeyEvent::press(KeyCode::A).with_device_id("kbd".to_string()),
        |id| alloc::vec![id.to_string()],
        None,
    );
    assert!(!remapped.triggered);
    assert_eq!(remapped.outputs.len(), 1);
    assert_eq!(remapped.outputs[0].keycode(), KeyCode::A);
}

/// Ported from the now-deleted `keyrx_daemon` EventProcessor test suite
/// (`multi_device_integration_test::test_multi_device_independent_state` /
/// `test_device_state_isolation`). Those tests gave false confidence: they
/// fed each "device" to its OWN `EventProcessor`/config, which proves
/// nothing about routing - two independent engines are trivially
/// independent. The real behaviour to verify is a SINGLE production
/// `Remapper` (as used by the daemon's one event loop) routing
/// INTERLEAVED events from two different devices to two different
/// `device_start` blocks, with each device's own modifier/lock/press
/// state kept separate (see `modifier_state_is_shared_across_devices_...`
/// for the one exception: custom modifiers/locks) while their output
/// streams stay correctly attributed.
#[test]
fn two_devices_routed_by_one_remapper_stay_independent_when_interleaved() {
    let numpad = DeviceConfig {
        identifier: DeviceIdentifier {
            pattern: "*numpad*".to_string(),
        },
        mappings: alloc::vec![
            KeyMapping::simple(KeyCode::Numpad1, KeyCode::F13),
            KeyMapping::simple(KeyCode::Numpad2, KeyCode::F14),
        ],
    };
    let main_keyboard = DeviceConfig {
        identifier: DeviceIdentifier {
            pattern: "*".to_string(),
        },
        mappings: alloc::vec![
            KeyMapping::modifier(KeyCode::CapsLock, 0),
            KeyMapping::conditional(
                Condition::ModifierActive(0),
                alloc::vec![BaseKeyMapping::Simple {
                    from: KeyCode::H,
                    to: KeyCode::Left,
                }],
            ),
        ],
    };
    let mut remap = Remapper::from_blocks(&[numpad, main_keyboard]);
    let ids = |id: &str| alloc::vec![id.to_string()];

    // Interleave: numpad press, main-keyboard CapsLock+H, numpad press,
    // main-keyboard H (layer now off) - a single event loop's real
    // traffic pattern, all through the same `Remapper`.
    let numpad1 = remap.process(
        KeyEvent::press(KeyCode::Numpad1).with_device_id("numpad-1".to_string()),
        ids,
        None,
    );
    assert_eq!(numpad1.outputs.len(), 1);
    assert_eq!(numpad1.outputs[0].keycode(), KeyCode::F13);
    remap.process(
        KeyEvent::release(KeyCode::Numpad1).with_device_id("numpad-1".to_string()),
        ids,
        None,
    );

    let caps_on = remap.process(
        KeyEvent::press(KeyCode::CapsLock).with_device_id("main-1".to_string()),
        ids,
        None,
    );
    assert!(
        caps_on.outputs.is_empty(),
        "modifier press produces no output"
    );

    let numpad2 = remap.process(
        KeyEvent::press(KeyCode::Numpad2).with_device_id("numpad-1".to_string()),
        ids,
        None,
    );
    assert_eq!(
        numpad2.outputs.len(),
        1,
        "numpad state must be untouched by the main keyboard's CapsLock"
    );
    assert_eq!(numpad2.outputs[0].keycode(), KeyCode::F14);
    remap.process(
        KeyEvent::release(KeyCode::Numpad2).with_device_id("numpad-1".to_string()),
        ids,
        None,
    );

    let h_with_layer = remap.process(
        KeyEvent::press(KeyCode::H).with_device_id("main-1".to_string()),
        ids,
        None,
    );
    assert_eq!(h_with_layer.outputs.len(), 1);
    assert_eq!(h_with_layer.outputs[0].keycode(), KeyCode::Left);
    remap.process(
        KeyEvent::release(KeyCode::H).with_device_id("main-1".to_string()),
        ids,
        None,
    );

    let caps_off = remap.process(
        KeyEvent::release(KeyCode::CapsLock).with_device_id("main-1".to_string()),
        ids,
        None,
    );
    assert!(caps_off.outputs.is_empty());

    // A numpad key never has H mapped at all: unaffected by the main
    // keyboard's layer having been active a moment ago.
    let numpad_h = remap.process(
        KeyEvent::press(KeyCode::H).with_device_id("numpad-1".to_string()),
        ids,
        None,
    );
    assert!(
        !numpad_h.triggered,
        "numpad block has no H mapping - passes through"
    );
    assert_eq!(numpad_h.outputs[0].keycode(), KeyCode::H);
    remap.process(
        KeyEvent::release(KeyCode::H).with_device_id("numpad-1".to_string()),
        ids,
        None,
    );

    // And back on the main keyboard, the layer is off again.
    let h_without_layer = remap.process(
        KeyEvent::press(KeyCode::H).with_device_id("main-1".to_string()),
        ids,
        None,
    );
    assert!(!h_without_layer.triggered);
    assert_eq!(h_without_layer.outputs[0].keycode(), KeyCode::H);
}

/// Regression: a key typed in the base layer and still down when a layer key
/// activates was released through the NEW layer's mapping (H -> Left), so
/// the OS never saw H come up and it auto-repeated until pressed again.
#[test]
fn a_passthrough_key_is_released_as_pressed_after_a_layer_change() {
    let config = DeviceConfig {
        identifier: DeviceIdentifier {
            pattern: "*".to_string(),
        },
        mappings: alloc::vec![
            KeyMapping::modifier(KeyCode::RAlt, 0),
            KeyMapping::conditional(
                Condition::ModifierActive(0),
                alloc::vec![BaseKeyMapping::Simple {
                    from: KeyCode::H,
                    to: KeyCode::Left,
                }],
            ),
        ],
    };
    let mut state = Remapper::new(&config);
    let mut send = |event: KeyEvent| -> Vec<(KeyCode, bool)> {
        state
            .process(event, |id| alloc::vec![id.to_string()], None)
            .outputs
            .iter()
            .map(|e| (e.keycode(), e.is_press()))
            .collect()
    };

    assert_eq!(send(KeyEvent::press(KeyCode::H)), [(KeyCode::H, true)]);
    assert!(send(KeyEvent::press(KeyCode::RAlt)).is_empty()); // layer on
    assert_eq!(send(KeyEvent::release(KeyCode::H)), [(KeyCode::H, false)]);
    assert!(send(KeyEvent::release(KeyCode::RAlt)).is_empty());
    // A fresh press inside the layer still uses the layer.
    send(KeyEvent::press(KeyCode::RAlt));
    assert_eq!(send(KeyEvent::press(KeyCode::H)), [(KeyCode::Left, true)]);
}
