#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::field_reassign_with_default,
    clippy::useless_conversion,
    clippy::assertions_on_constants,
    clippy::manual_div_ceil,
    clippy::manual_strip,
    clippy::len_zero,
    clippy::redundant_closure,
    clippy::manual_range_contains,
    clippy::default_constructed_unit_structs,
    clippy::clone_on_copy,
    clippy::io_other_error,
    clippy::bool_assert_comparison,
    clippy::approx_constant,
    clippy::let_unit_value,
    clippy::while_let_on_iterator,
    clippy::await_holding_lock,
    clippy::unnecessary_cast,
    clippy::drop_non_drop,
    clippy::needless_range_loop,
    unused_imports,
    unused_variables,
    dead_code,
    unsafe_code,
    clippy::collapsible_if,
    clippy::bool_comparison,
    unexpected_cfgs
)]
//! Integration tests for combo behavior in the advanced engine.

use keyrx_core::engine::{
    AdvancedEngine, InputEvent, KeyCode, LayerAction, OutputAction, TimingConfig,
};
use keyrx_core::mocks::MockRuntime;

fn key_down(key: KeyCode, ts: u64) -> InputEvent {
    InputEvent::key_down(key, ts)
}

fn key_up(key: KeyCode, ts: u64) -> InputEvent {
    InputEvent::key_up(key, ts)
}

fn engine_with_default_config() -> AdvancedEngine<MockRuntime> {
    AdvancedEngine::new(MockRuntime::default(), TimingConfig::default())
}

#[test]
fn two_key_combo_triggers_action() {
    let mut engine = engine_with_default_config();
    engine.combos_mut().register(
        &[KeyCode::A, KeyCode::B],
        LayerAction::Remap(KeyCode::Escape),
    );

    let first = engine.process_event(key_down(KeyCode::A, 0));
    assert_eq!(first, vec![OutputAction::Block]);

    let second = engine.process_event(key_down(KeyCode::B, 10_000));
    assert_eq!(
        second,
        vec![
            OutputAction::KeyDown(KeyCode::Escape),
            OutputAction::KeyUp(KeyCode::Escape),
            OutputAction::Block
        ]
    );
    assert!(engine.pending().is_empty());
}

#[test]
fn three_key_combo_triggers_on_third_press() {
    let mut engine = engine_with_default_config();
    engine.combos_mut().register(
        &[KeyCode::Q, KeyCode::W, KeyCode::E],
        LayerAction::Remap(KeyCode::Tab),
    );

    let first = engine.process_event(key_down(KeyCode::Q, 0));
    assert_eq!(first, vec![OutputAction::Block]);

    let second = engine.process_event(key_down(KeyCode::W, 10_000));
    assert_eq!(second, vec![OutputAction::Block]);

    let third = engine.process_event(key_down(KeyCode::E, 20_000));
    assert_eq!(
        third,
        vec![
            OutputAction::KeyDown(KeyCode::Tab),
            OutputAction::KeyUp(KeyCode::Tab),
            OutputAction::Block
        ]
    );
    assert!(engine.pending().is_empty());
}

#[test]
fn combo_matches_regardless_of_press_order() {
    let mut engine = engine_with_default_config();
    engine.combos_mut().register(
        &[KeyCode::Z, KeyCode::X, KeyCode::C],
        LayerAction::Remap(KeyCode::Enter),
    );

    let first = engine.process_event(key_down(KeyCode::C, 0));
    assert_eq!(first, vec![OutputAction::Block]);

    let second = engine.process_event(key_down(KeyCode::Z, 5_000));
    assert_eq!(second, vec![OutputAction::Block]);

    let third = engine.process_event(key_down(KeyCode::X, 10_000));
    assert_eq!(
        third,
        vec![
            OutputAction::KeyDown(KeyCode::Enter),
            OutputAction::KeyUp(KeyCode::Enter),
            OutputAction::Block
        ]
    );
    assert!(engine.pending().is_empty());
}

#[test]
fn partial_combo_times_out_and_passes_through_keys() {
    let mut engine = engine_with_default_config();
    engine.combos_mut().register(
        &[KeyCode::A, KeyCode::B],
        LayerAction::Remap(KeyCode::Escape),
    );

    let first = engine.process_event(key_down(KeyCode::A, 0));
    assert_eq!(first, vec![OutputAction::Block]);

    let timeout = engine.tick(60_000);
    assert_eq!(timeout, vec![OutputAction::KeyDown(KeyCode::A)]);
    assert!(engine.pending().is_empty());

    let release = engine.process_event(key_up(KeyCode::A, 70_000));
    assert_eq!(release, vec![OutputAction::KeyUp(KeyCode::A)]);
}
