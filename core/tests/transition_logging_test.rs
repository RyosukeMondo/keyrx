#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::print_stdout, clippy::print_stderr, clippy::field_reassign_with_default, clippy::useless_conversion, clippy::assertions_on_constants, clippy::manual_div_ceil, clippy::manual_strip, clippy::len_zero, clippy::redundant_closure, clippy::manual_range_contains, clippy::default_constructed_unit_structs, clippy::clone_on_copy, clippy::io_other_error, clippy::bool_assert_comparison, clippy::approx_constant, clippy::let_unit_value, clippy::while_let_on_iterator, clippy::await_holding_lock, clippy::unnecessary_cast, clippy::drop_non_drop, clippy::needless_range_loop, unused_imports, unused_variables, dead_code, unsafe_code, clippy::collapsible_if, clippy::bool_comparison, unexpected_cfgs)]
//! Tests for transition logging in AdvancedEngine.

use keyrx_core::engine::{AdvancedEngine, InputEvent, KeyCode, TimingConfig};
use keyrx_core::mocks::MockRuntime;

fn key_down(key: KeyCode, ts: u64) -> InputEvent {
    InputEvent::key_down(key, ts)
}

fn key_up(key: KeyCode, ts: u64) -> InputEvent {
    InputEvent::key_up(key, ts)
}

#[test]
#[cfg(feature = "transition-logging")]
fn test_transition_log_records_key_events() {
    let mut engine = AdvancedEngine::new(MockRuntime::default(), TimingConfig::default());

    // Initially log should be empty
    assert_eq!(engine.transition_log().len(), 0);

    // Process a key press
    engine.process_event(key_down(KeyCode::A, 1000));

    // Log should contain the transition
    assert!(engine.transition_log().len() > 0);

    // Process a key release
    engine.process_event(key_up(KeyCode::A, 2000));

    // Log should contain both transitions
    assert!(engine.transition_log().len() >= 2);
}

#[test]
#[cfg(feature = "transition-logging")]
fn test_transition_log_captures_state_snapshots() {
    let mut engine = AdvancedEngine::new(MockRuntime::default(), TimingConfig::default());

    // Process a key press
    engine.process_event(key_down(KeyCode::A, 1000));

    // Get the last logged entry
    let last_entry = engine.transition_log().last();
    assert!(last_entry.is_some());

    let entry = last_entry.unwrap();

    // Verify the transition name
    assert_eq!(entry.name(), "KeyPressed");

    // Verify timing information is captured
    assert!(entry.wall_time_us > 0);
    // duration_ns is u64, so it's always >= 0

    // Verify state snapshots are captured
    assert!(entry.version_after() >= entry.version_before());
}

#[test]
#[cfg(feature = "transition-logging")]
fn test_transition_log_search_by_name() {
    let mut engine = AdvancedEngine::new(MockRuntime::default(), TimingConfig::default());

    // Process multiple key events
    for i in 0..3 {
        engine.process_event(key_down(KeyCode::A, i * 1000));
        engine.process_event(key_up(KeyCode::A, i * 1000 + 500));
    }

    // Search for key presses
    let presses = engine.transition_log().search_by_name("KeyPressed");
    assert_eq!(presses.len(), 3);

    // Search for key releases
    let releases = engine.transition_log().search_by_name("KeyReleased");
    assert_eq!(releases.len(), 3);
}

#[test]
#[cfg(feature = "transition-logging")]
fn test_transition_log_clear() {
    let mut engine = AdvancedEngine::new(MockRuntime::default(), TimingConfig::default());

    // Add some transitions
    engine.process_event(key_down(KeyCode::A, 1000));

    assert!(engine.transition_log().len() > 0);

    // Clear the log
    engine.transition_log_mut().clear();

    assert_eq!(engine.transition_log().len(), 0);
}

#[test]
#[cfg(feature = "transition-logging")]
fn test_transition_log_statistics() {
    let mut engine = AdvancedEngine::new(MockRuntime::default(), TimingConfig::default());

    // Process multiple events
    for i in 0..5 {
        engine.process_event(key_down(KeyCode::A, i * 1000));
    }

    let (total, unique_names, _total_duration, _avg_duration) =
        engine.transition_log().statistics();

    assert_eq!(total, 5);
    assert!(unique_names >= 1); // At least "KeyPressed"
}

#[test]
#[cfg(feature = "transition-logging")]
fn test_transition_log_export_json() {
    let mut engine = AdvancedEngine::new(MockRuntime::default(), TimingConfig::default());

    // Add a transition
    engine.process_event(key_down(KeyCode::A, 1000));

    // Export to JSON
    let json = engine.transition_log().export_json();
    assert!(json.is_ok());

    let json_str = json.unwrap();
    assert!(json_str.contains("transition"));
    assert!(json_str.contains("state_before"));
    assert!(json_str.contains("state_after"));
}

#[test]
#[cfg(not(feature = "transition-logging"))]
fn test_transition_log_disabled_has_zero_overhead() {
    let mut engine = AdvancedEngine::new(MockRuntime::default(), TimingConfig::default());

    // Process events
    engine.process_event(key_down(KeyCode::A, 1000));

    // When disabled, log should always be empty
    assert_eq!(engine.transition_log().len(), 0);
    assert_eq!(engine.transition_log().capacity(), 0);

    // Export should return empty array
    let json = engine.transition_log().export_json().unwrap();
    assert_eq!(json, "[]");
}
