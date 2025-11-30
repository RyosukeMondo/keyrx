//! Integration tests for layer behavior in the advanced engine.

use keyrx_core::engine::{
    AdvancedEngine, HoldAction, InputEvent, KeyCode, Layer, LayerAction, OutputAction, TimingConfig,
};
use keyrx_core::mocks::{MockInput, MockRuntime};

fn key_down(key: KeyCode, ts: u64) -> InputEvent {
    InputEvent::key_down(key, ts)
}

fn key_up(key: KeyCode, ts: u64) -> InputEvent {
    InputEvent::key_up(key, ts)
}

fn engine() -> AdvancedEngine<MockInput, MockRuntime> {
    AdvancedEngine::new(
        MockInput::new(),
        MockRuntime::default(),
        TimingConfig::default(),
    )
}

#[test]
fn layer_push_and_pop_change_active_mapping() {
    let mut engine = engine();

    let mut base = Layer::base();
    base.set_mapping(KeyCode::F1, LayerAction::LayerPush(1));
    base.set_mapping(KeyCode::F2, LayerAction::LayerPop);
    engine.layers_mut().define_layer(base);

    let mut nav = Layer::with_id(1, "nav");
    nav.transparent = true;
    nav.set_mapping(KeyCode::A, LayerAction::Remap(KeyCode::Left));
    engine.layers_mut().define_layer(nav);

    let push = engine.process_event(key_down(KeyCode::F1, 0));
    assert_eq!(push, vec![OutputAction::Block]);
    assert!(engine.layers().is_active(1));

    let remap_down = engine.process_event(key_down(KeyCode::A, 10_000));
    assert_eq!(remap_down, vec![OutputAction::KeyDown(KeyCode::Left)]);

    let remap_up = engine.process_event(key_up(KeyCode::A, 20_000));
    assert_eq!(remap_up, vec![OutputAction::KeyUp(KeyCode::Left)]);

    let pop = engine.process_event(key_down(KeyCode::F2, 30_000));
    assert_eq!(pop, vec![OutputAction::Block]);
    assert!(!engine.layers().is_active(1));

    let passthrough = engine.process_event(key_down(KeyCode::A, 40_000));
    assert_eq!(passthrough, vec![OutputAction::KeyDown(KeyCode::A)]);
}

#[test]
fn layer_toggle_toggles_active_mappings() {
    let mut engine = engine();

    let mut base = Layer::base();
    base.set_mapping(KeyCode::F1, LayerAction::LayerToggle(1));
    engine.layers_mut().define_layer(base);

    let mut overlay = Layer::with_id(1, "overlay");
    overlay.transparent = true;
    overlay.set_mapping(KeyCode::B, LayerAction::Block);
    engine.layers_mut().define_layer(overlay);

    let enable = engine.process_event(key_down(KeyCode::F1, 0));
    assert_eq!(enable, vec![OutputAction::Block]);
    assert!(engine.layers().is_active(1));

    let blocked = engine.process_event(key_down(KeyCode::B, 10_000));
    assert_eq!(blocked, vec![OutputAction::Block]);

    let disable = engine.process_event(key_down(KeyCode::F1, 20_000));
    assert_eq!(disable, vec![OutputAction::Block]);
    assert!(!engine.layers().is_active(1));

    let passthrough = engine.process_event(key_down(KeyCode::B, 30_000));
    assert_eq!(passthrough, vec![OutputAction::KeyDown(KeyCode::B)]);
}

#[test]
fn transparent_layer_falls_through_to_lower_layers() {
    let mut engine = engine();

    let mut base = Layer::base();
    base.set_mapping(KeyCode::C, LayerAction::Remap(KeyCode::D));
    engine.layers_mut().define_layer(base);

    let mut overlay = Layer::with_id(1, "overlay");
    overlay.transparent = true;
    overlay.set_mapping(KeyCode::X, LayerAction::Block);
    engine.layers_mut().define_layer(overlay);
    engine.layers_mut().push(1);

    let overlay_action = engine.process_event(key_down(KeyCode::X, 0));
    assert_eq!(overlay_action, vec![OutputAction::Block]);

    let fallthrough_down = engine.process_event(key_down(KeyCode::C, 10_000));
    assert_eq!(fallthrough_down, vec![OutputAction::KeyDown(KeyCode::D)]);

    let fallthrough_up = engine.process_event(key_up(KeyCode::C, 20_000));
    assert_eq!(fallthrough_up, vec![OutputAction::KeyUp(KeyCode::D)]);
}

#[test]
fn tap_hold_mapping_honors_layer_activation() {
    let mut engine = engine();

    let mut base = Layer::base();
    base.set_mapping(KeyCode::F1, LayerAction::LayerPush(1));
    base.set_mapping(KeyCode::F2, LayerAction::LayerPop);
    engine.layers_mut().define_layer(base);

    let mut macro_layer = Layer::with_id(1, "macro");
    macro_layer.transparent = true;
    macro_layer.set_mapping(
        KeyCode::CapsLock,
        LayerAction::TapHold {
            tap: KeyCode::Escape,
            hold: HoldAction::Key(KeyCode::LeftCtrl),
        },
    );
    engine.layers_mut().define_layer(macro_layer);

    let push = engine.process_event(key_down(KeyCode::F1, 0));
    assert_eq!(push, vec![OutputAction::Block]);

    let tap_start = engine.process_event(key_down(KeyCode::CapsLock, 10_000));
    assert_eq!(tap_start, vec![OutputAction::Block]);

    let tap_resolve = engine.process_event(key_up(KeyCode::CapsLock, 20_000));
    assert_eq!(
        tap_resolve,
        vec![
            OutputAction::KeyDown(KeyCode::Escape),
            OutputAction::KeyUp(KeyCode::Escape),
            OutputAction::Block
        ]
    );

    let pop = engine.process_event(key_down(KeyCode::F2, 30_000));
    assert_eq!(pop, vec![OutputAction::Block]);

    let passthrough = engine.process_event(key_down(KeyCode::CapsLock, 40_000));
    assert_eq!(passthrough, vec![OutputAction::KeyDown(KeyCode::CapsLock)]);
}
