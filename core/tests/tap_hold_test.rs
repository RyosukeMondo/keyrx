//! Integration tests for tap-hold behavior in the advanced engine.

use keyrx_core::engine::{
    AdvancedEngine, HoldAction, InputEvent, KeyCode, Layer, LayerAction, OutputAction, TimingConfig,
};
use keyrx_core::mocks::MockRuntime;

fn key_down(key: KeyCode, ts: u64) -> InputEvent {
    InputEvent::key_down(key, ts)
}

fn key_up(key: KeyCode, ts: u64) -> InputEvent {
    InputEvent::key_up(key, ts)
}

fn engine_with_config(config: TimingConfig) -> AdvancedEngine<MockRuntime> {
    let mut engine = AdvancedEngine::new(MockRuntime::default(), config);
    let mut base = Layer::base();
    base.set_mapping(
        KeyCode::CapsLock,
        LayerAction::TapHold {
            tap: KeyCode::Escape,
            hold: HoldAction::Key(KeyCode::LeftCtrl),
        },
    );
    engine.layers_mut().define_layer(base);
    engine
}

#[test]
fn tap_path_resolves_on_quick_release() {
    let mut engine = engine_with_config(TimingConfig::default());

    let down = engine.process_event(key_down(KeyCode::CapsLock, 0));
    assert_eq!(down, vec![OutputAction::Block]);

    let up = engine.process_event(key_up(KeyCode::CapsLock, 50_000));
    assert_eq!(
        up,
        vec![
            OutputAction::KeyDown(KeyCode::Escape),
            OutputAction::KeyUp(KeyCode::Escape),
            OutputAction::Block
        ]
    );
}

#[test]
fn hold_path_resolves_after_timeout() {
    let mut engine = engine_with_config(TimingConfig::default());

    let down = engine.process_event(key_down(KeyCode::CapsLock, 0));
    assert_eq!(down, vec![OutputAction::Block]);

    let tick = engine.tick(250_000);
    assert_eq!(tick, vec![OutputAction::KeyDown(KeyCode::LeftCtrl)]);

    let up = engine.process_event(key_up(KeyCode::CapsLock, 300_000));
    assert_eq!(up, vec![OutputAction::Block]);
}

#[test]
fn permissive_hold_triggers_hold_on_interrupt() {
    let mut engine = engine_with_config(TimingConfig::default());

    let down = engine.process_event(key_down(KeyCode::CapsLock, 0));
    assert_eq!(down, vec![OutputAction::Block]);

    let interrupt_down = engine.process_event(key_down(KeyCode::A, 20_000));
    assert_eq!(interrupt_down, vec![OutputAction::KeyDown(KeyCode::A)]);

    let interrupt_up = engine.process_event(key_up(KeyCode::A, 30_000));
    assert_eq!(interrupt_up, vec![OutputAction::KeyUp(KeyCode::A)]);

    let up = engine.process_event(key_up(KeyCode::CapsLock, 40_000));
    assert_eq!(
        up,
        vec![
            OutputAction::KeyDown(KeyCode::LeftCtrl),
            OutputAction::Block
        ]
    );
}

#[test]
fn eager_tap_emits_immediately_and_upgrades_to_hold() {
    let mut config = TimingConfig::default();
    config.eager_tap = true;
    let mut engine = engine_with_config(config);

    let down = engine.process_event(key_down(KeyCode::CapsLock, 0));
    assert_eq!(
        down,
        vec![
            OutputAction::KeyDown(KeyCode::Escape),
            OutputAction::KeyUp(KeyCode::Escape),
            OutputAction::Block
        ]
    );

    let tick = engine.tick(250_000);
    assert_eq!(tick, vec![OutputAction::KeyDown(KeyCode::LeftCtrl)]);

    let up = engine.process_event(key_up(KeyCode::CapsLock, 300_000));
    assert_eq!(up, vec![OutputAction::Block]);
}

#[test]
fn retro_tap_emits_tap_after_hold_on_release() {
    let mut config = TimingConfig::default();
    config.retro_tap = true;
    let mut engine = engine_with_config(config);

    let down = engine.process_event(key_down(KeyCode::CapsLock, 0));
    assert_eq!(down, vec![OutputAction::Block]);

    let tick = engine.tick(250_000);
    assert_eq!(tick, vec![OutputAction::KeyDown(KeyCode::LeftCtrl)]);

    let up = engine.process_event(key_up(KeyCode::CapsLock, 300_000));
    assert_eq!(
        up,
        vec![
            OutputAction::KeyDown(KeyCode::Escape),
            OutputAction::KeyUp(KeyCode::Escape),
            OutputAction::Block
        ]
    );
}
