//! Tests for the advanced engine.

use super::*;
use crate::engine::{DecisionQueue, HoldAction, InputEvent, Layer, LayerAction};
use crate::mocks::MockRuntime;

fn test_engine() -> AdvancedEngine<MockRuntime> {
    AdvancedEngine::new(MockRuntime::default(), TimingConfig::default())
}

fn key_down(key: KeyCode, ts: u64) -> InputEvent {
    InputEvent::key_down(key, ts)
}

fn key_up(key: KeyCode, ts: u64) -> InputEvent {
    InputEvent::key_up(key, ts)
}

#[test]
fn tap_hold_tap_path_emits_tap() {
    let mut engine = test_engine();
    let mut base = Layer::base();
    base.set_mapping(
        KeyCode::CapsLock,
        LayerAction::TapHold {
            tap: KeyCode::Escape,
            hold: HoldAction::Key(KeyCode::LeftCtrl),
        },
    );
    engine.layers_mut().define_layer(base);

    let output = engine.process_event(key_down(KeyCode::CapsLock, 0));
    assert_eq!(output, vec![OutputAction::Block]);

    let output = engine.process_event(key_up(KeyCode::CapsLock, 50_000));
    assert_eq!(
        output,
        vec![
            OutputAction::KeyDown(KeyCode::Escape),
            OutputAction::KeyUp(KeyCode::Escape),
            OutputAction::Block
        ]
    );
}

#[test]
fn tap_hold_hold_path_emits_hold_action_on_timeout() {
    let mut engine = test_engine();
    let mut base = Layer::base();
    base.set_mapping(
        KeyCode::CapsLock,
        LayerAction::TapHold {
            tap: KeyCode::Escape,
            hold: HoldAction::Key(KeyCode::LeftCtrl),
        },
    );
    engine.layers_mut().define_layer(base);

    let output = engine.process_event(key_down(KeyCode::CapsLock, 0));
    assert_eq!(output, vec![OutputAction::Block]);

    let tick_out = engine.tick(250_000);
    assert_eq!(tick_out, vec![OutputAction::KeyDown(KeyCode::LeftCtrl)]);
}

#[test]
fn combo_trigger_blocks_keys_and_emits_action() {
    let mut engine = test_engine();
    engine.combos_mut().register(
        &[KeyCode::A, KeyCode::B],
        LayerAction::Remap(KeyCode::Escape),
    );

    let out1 = engine.process_event(key_down(KeyCode::A, 0));
    assert!(out1.contains(&OutputAction::Block));

    let out2 = engine.process_event(key_down(KeyCode::B, 10_000));
    assert_eq!(
        out2,
        vec![
            OutputAction::KeyDown(KeyCode::Escape),
            OutputAction::KeyUp(KeyCode::Escape),
            OutputAction::Block
        ]
    );
}

#[test]
fn safe_mode_toggle_passes_through_events() {
    let mut engine = test_engine();

    // Press chord Ctrl+Alt+Shift+Escape to toggle.
    let _ = engine.process_event(key_down(KeyCode::LeftCtrl, 0));
    let _ = engine.process_event(key_down(KeyCode::LeftAlt, 0));
    let _ = engine.process_event(key_down(KeyCode::LeftShift, 0));
    let out = engine.process_event(key_down(KeyCode::Escape, 0));
    assert_eq!(out, vec![OutputAction::PassThrough]);

    // Subsequent events should pass through while safe mode is active.
    let out2 = engine.process_event(key_down(KeyCode::A, 10_000));
    assert_eq!(out2, vec![OutputAction::PassThrough]);
}

#[test]
fn snapshot_exposes_serializable_state() {
    let mut engine = test_engine();
    let mut base = Layer::base();
    base.set_mapping(
        KeyCode::CapsLock,
        LayerAction::TapHold {
            tap: KeyCode::Escape,
            hold: HoldAction::Key(KeyCode::LeftCtrl),
        },
    );
    engine.layers_mut().define_layer(base);

    let _ = engine.process_event(key_down(KeyCode::CapsLock, 100));

    let snapshot = engine.snapshot();
    assert!(snapshot
        .pressed_keys
        .iter()
        .any(|pk| pk.key == KeyCode::CapsLock && pk.pressed_at == 100));
    assert!(snapshot.is_layer_active(0));
    assert_eq!(snapshot.pending_count, 1);

    serde_json::to_string(&snapshot).expect("engine state serializes");
}

#[test]
fn combo_timeout_emits_single_release_path() {
    let mut engine = test_engine();
    engine.combos_mut().register(
        &[KeyCode::A, KeyCode::B],
        LayerAction::Remap(KeyCode::Escape),
    );

    let first = engine.process_event(key_down(KeyCode::A, 0));
    assert_eq!(first, vec![OutputAction::Block]);

    // Timeout should synthesize only a KeyDown for the still-pressed key.
    let timeout = engine.tick(60_000);
    assert_eq!(timeout, vec![OutputAction::KeyDown(KeyCode::A)]);

    // The real release should produce a single KeyUp, not a double release.
    let release = engine.process_event(key_up(KeyCode::A, 70_000));
    assert_eq!(release, vec![OutputAction::KeyUp(KeyCode::A)]);
}

#[test]
fn combo_queue_saturation_does_not_block_events() {
    let mut engine = test_engine();
    engine.combos_mut().register(
        &[KeyCode::A, KeyCode::B],
        LayerAction::Remap(KeyCode::Escape),
    );

    for _ in 0..DecisionQueue::MAX_PENDING {
        let _ = engine
            .state
            .pending_mut()
            .add_combo(&[KeyCode::A, KeyCode::B], 0, LayerAction::Block);
    }

    // With a full queue, new combo-related keys should pass through instead of being blocked.
    let output = engine.process_event(key_down(KeyCode::A, 0));
    assert_eq!(output, vec![OutputAction::KeyDown(KeyCode::A)]);
}

#[test]
fn combo_tap_hold_uses_event_timestamp() {
    let mut engine = test_engine();
    engine.combos_mut().register(
        &[KeyCode::A, KeyCode::B],
        LayerAction::TapHold {
            tap: KeyCode::Escape,
            hold: HoldAction::Key(KeyCode::LeftCtrl),
        },
    );

    let first = engine.process_event(key_down(KeyCode::A, 0));
    assert_eq!(first, vec![OutputAction::Block]);

    // Second key should enqueue tap-hold with the event's timestamp.
    let second = engine.process_event(key_down(KeyCode::B, 10_000));
    assert_eq!(second, vec![OutputAction::Block]);

    let pending = engine.state.pending().snapshot();
    let (key, pressed_at) = match pending.first() {
        Some(crate::engine::decision::pending::PendingDecisionState::TapHold {
            key,
            pressed_at,
            ..
        }) => (*key, *pressed_at),
        other => {
            unreachable!("expected TapHold pending, got {:?}", other)
        }
    };
    assert_eq!(key, KeyCode::B);
    assert_eq!(pressed_at, 10_000);

    // Resolve as hold via timeout to ensure the pending entry completes.
    let timeout = engine.tick(260_000);
    assert_eq!(timeout, vec![OutputAction::KeyDown(KeyCode::LeftCtrl)]);

    let release = engine.process_event(key_up(KeyCode::B, 300_000));
    assert_eq!(release, vec![OutputAction::Block]);
}
