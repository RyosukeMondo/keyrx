//! Engine unit tests.
//!
//! Tests for core engine behavior including event processing
//! with various RemapActions.

use keyrx_core::engine::{Engine, InputEvent, KeyCode, OutputAction};
use keyrx_core::mocks::{MockInput, MockRuntime, MockState};

/// Test that Engine can be created with mock dependencies.
#[test]
fn engine_creation_with_mocks() {
    let input = MockInput::new();
    let runtime = MockRuntime::new();
    let state = MockState::new();

    let engine = Engine::new(
        input,
        runtime,
        state,
        keyrx_core::metrics::default_noop_collector(),
    );
    assert!(!engine.is_running());
}

/// Test process_event with Remap action (key-down).
#[test]
fn process_event_remap_key_down() {
    let input = MockInput::new();
    let mut runtime = MockRuntime::new();
    let state = MockState::new();

    // Configure A -> B remap
    runtime.registry_mut().remap(KeyCode::A, KeyCode::B);

    let engine = Engine::new(
        input,
        runtime,
        state,
        keyrx_core::metrics::default_noop_collector(),
    );

    let event = InputEvent::key_down(KeyCode::A, 0);
    let output = engine.process_event(&event);

    assert_eq!(output, OutputAction::KeyDown(KeyCode::B));
}

/// Test process_event with Remap action (key-up).
#[test]
fn process_event_remap_key_up() {
    let input = MockInput::new();
    let mut runtime = MockRuntime::new();
    let state = MockState::new();

    // Configure A -> B remap
    runtime.registry_mut().remap(KeyCode::A, KeyCode::B);

    let engine = Engine::new(
        input,
        runtime,
        state,
        keyrx_core::metrics::default_noop_collector(),
    );

    let event = InputEvent::key_up(KeyCode::A, 0);
    let output = engine.process_event(&event);

    assert_eq!(output, OutputAction::KeyUp(KeyCode::B));
}

/// Test process_event with Block action (key-down).
#[test]
fn process_event_block_key_down() {
    let input = MockInput::new();
    let mut runtime = MockRuntime::new();
    let state = MockState::new();

    // Configure CapsLock to be blocked
    runtime.registry_mut().block(KeyCode::CapsLock);

    let engine = Engine::new(
        input,
        runtime,
        state,
        keyrx_core::metrics::default_noop_collector(),
    );

    let event = InputEvent::key_down(KeyCode::CapsLock, 0);
    let output = engine.process_event(&event);

    assert_eq!(output, OutputAction::Block);
}

/// Test process_event with Block action (key-up).
#[test]
fn process_event_block_key_up() {
    let input = MockInput::new();
    let mut runtime = MockRuntime::new();
    let state = MockState::new();

    // Configure CapsLock to be blocked
    runtime.registry_mut().block(KeyCode::CapsLock);

    let engine = Engine::new(
        input,
        runtime,
        state,
        keyrx_core::metrics::default_noop_collector(),
    );

    let event = InputEvent::key_up(KeyCode::CapsLock, 0);
    let output = engine.process_event(&event);

    assert_eq!(output, OutputAction::Block);
}

/// Test process_event with Pass action (key-down).
#[test]
fn process_event_pass_key_down() {
    let input = MockInput::new();
    let mut runtime = MockRuntime::new();
    let state = MockState::new();

    // Explicitly configure Enter to pass (this is also the default)
    runtime.registry_mut().pass(KeyCode::Enter);

    let engine = Engine::new(
        input,
        runtime,
        state,
        keyrx_core::metrics::default_noop_collector(),
    );

    let event = InputEvent::key_down(KeyCode::Enter, 0);
    let output = engine.process_event(&event);

    assert_eq!(output, OutputAction::PassThrough);
}

/// Test process_event with Pass action (key-up).
#[test]
fn process_event_pass_key_up() {
    let input = MockInput::new();
    let mut runtime = MockRuntime::new();
    let state = MockState::new();

    // Explicitly configure Enter to pass
    runtime.registry_mut().pass(KeyCode::Enter);

    let engine = Engine::new(
        input,
        runtime,
        state,
        keyrx_core::metrics::default_noop_collector(),
    );

    let event = InputEvent::key_up(KeyCode::Enter, 0);
    let output = engine.process_event(&event);

    assert_eq!(output, OutputAction::PassThrough);
}

/// Test that unmapped keys default to PassThrough.
#[test]
fn process_event_unmapped_key_passes_through() {
    let input = MockInput::new();
    let runtime = MockRuntime::new();
    let state = MockState::new();

    // No remaps configured
    let engine = Engine::new(
        input,
        runtime,
        state,
        keyrx_core::metrics::default_noop_collector(),
    );

    let event = InputEvent::key_down(KeyCode::Z, 0);
    let output = engine.process_event(&event);

    assert_eq!(output, OutputAction::PassThrough);
}

/// Test multiple remaps in sequence.
#[test]
fn process_event_multiple_remaps() {
    let input = MockInput::new();
    let mut runtime = MockRuntime::new();
    let state = MockState::new();

    // Configure multiple remaps
    runtime.registry_mut().remap(KeyCode::A, KeyCode::B);
    runtime
        .registry_mut()
        .remap(KeyCode::CapsLock, KeyCode::Escape);
    runtime.registry_mut().block(KeyCode::Insert);

    let engine = Engine::new(
        input,
        runtime,
        state,
        keyrx_core::metrics::default_noop_collector(),
    );

    // Test A -> B
    let event_a = InputEvent::key_down(KeyCode::A, 0);
    assert_eq!(
        engine.process_event(&event_a),
        OutputAction::KeyDown(KeyCode::B)
    );

    // Test CapsLock -> Escape
    let event_caps = InputEvent::key_down(KeyCode::CapsLock, 0);
    assert_eq!(
        engine.process_event(&event_caps),
        OutputAction::KeyDown(KeyCode::Escape)
    );

    // Test Insert blocked
    let event_insert = InputEvent::key_down(KeyCode::Insert, 0);
    assert_eq!(engine.process_event(&event_insert), OutputAction::Block);

    // Test unmapped key passes
    let event_enter = InputEvent::key_down(KeyCode::Enter, 0);
    assert_eq!(
        engine.process_event(&event_enter),
        OutputAction::PassThrough
    );
}

/// Test engine start and stop lifecycle.
#[tokio::test]
async fn engine_start_stop_lifecycle() {
    let input = MockInput::new();
    let runtime = MockRuntime::new();
    let state = MockState::new();

    let mut engine = Engine::new(
        input,
        runtime,
        state,
        keyrx_core::metrics::default_noop_collector(),
    );

    assert!(!engine.is_running());

    engine.start().await.unwrap();
    assert!(engine.is_running());

    engine.stop().await.unwrap();
    assert!(!engine.is_running());
}

/// Test state accessor.
#[test]
fn engine_state_accessor() {
    let input = MockInput::new();
    let runtime = MockRuntime::new();
    let state = MockState::new();

    let engine = Engine::new(
        input,
        runtime,
        state,
        keyrx_core::metrics::default_noop_collector(),
    );

    // Just verify we can access state
    let _state = engine.state();
}

/// Test script accessor.
#[test]
fn engine_script_accessor() {
    let input = MockInput::new();
    let runtime = MockRuntime::new();
    let state = MockState::new();

    let engine = Engine::new(
        input,
        runtime,
        state,
        keyrx_core::metrics::default_noop_collector(),
    );

    // Just verify we can access script runtime
    let _script = engine.script();
}

/// Test key-down followed by key-up for same remapped key.
#[test]
fn process_event_remap_down_then_up() {
    let input = MockInput::new();
    let mut runtime = MockRuntime::new();
    let state = MockState::new();

    runtime.registry_mut().remap(KeyCode::A, KeyCode::B);

    let engine = Engine::new(
        input,
        runtime,
        state,
        keyrx_core::metrics::default_noop_collector(),
    );

    // Key down
    let down_event = InputEvent::key_down(KeyCode::A, 100);
    let down_output = engine.process_event(&down_event);
    assert_eq!(down_output, OutputAction::KeyDown(KeyCode::B));

    // Key up
    let up_event = InputEvent::key_up(KeyCode::A, 200);
    let up_output = engine.process_event(&up_event);
    assert_eq!(up_output, OutputAction::KeyUp(KeyCode::B));
}

/// Test that modifier keys can be remapped.
#[test]
fn process_event_remap_modifier_keys() {
    let input = MockInput::new();
    let mut runtime = MockRuntime::new();
    let state = MockState::new();

    // Remap left ctrl to left alt
    runtime
        .registry_mut()
        .remap(KeyCode::LeftCtrl, KeyCode::LeftAlt);

    let engine = Engine::new(
        input,
        runtime,
        state,
        keyrx_core::metrics::default_noop_collector(),
    );

    let event = InputEvent::key_down(KeyCode::LeftCtrl, 0);
    let output = engine.process_event(&event);

    assert_eq!(output, OutputAction::KeyDown(KeyCode::LeftAlt));
}

/// Test that function keys can be remapped.
#[test]
fn process_event_remap_function_keys() {
    let input = MockInput::new();
    let mut runtime = MockRuntime::new();
    let state = MockState::new();

    // Remap F1 to F2
    runtime.registry_mut().remap(KeyCode::F1, KeyCode::F2);

    let engine = Engine::new(
        input,
        runtime,
        state,
        keyrx_core::metrics::default_noop_collector(),
    );

    let event = InputEvent::key_down(KeyCode::F1, 0);
    let output = engine.process_event(&event);

    assert_eq!(output, OutputAction::KeyDown(KeyCode::F2));
}

/// Test that synthetic events are passed through without processing.
///
/// This prevents infinite loops when our injected keys are recaptured
/// by the input hook.
#[test]
fn process_event_synthetic_event_passes_through() {
    let input = MockInput::new();
    let mut runtime = MockRuntime::new();
    let state = MockState::new();

    // Configure A -> B remap (should be bypassed for synthetic events)
    runtime.registry_mut().remap(KeyCode::A, KeyCode::B);

    let engine = Engine::new(
        input,
        runtime,
        state,
        keyrx_core::metrics::default_noop_collector(),
    );

    // Create a synthetic event (simulating an injected key)
    let event = InputEvent::with_metadata(
        KeyCode::A,
        true,  // pressed
        0,     // timestamp
        None,  // device_id
        false, // is_repeat
        true,  // is_synthetic - this is the key flag
        0,     // scan_code
        None,  // serial_number
    );
    let output = engine.process_event(&event);

    // Synthetic events should pass through unchanged, not be remapped
    assert_eq!(output, OutputAction::PassThrough);
}

/// Test that synthetic key-up events also pass through.
#[test]
fn process_event_synthetic_key_up_passes_through() {
    let input = MockInput::new();
    let mut runtime = MockRuntime::new();
    let state = MockState::new();

    // Configure A -> B remap
    runtime.registry_mut().remap(KeyCode::A, KeyCode::B);

    let engine = Engine::new(
        input,
        runtime,
        state,
        keyrx_core::metrics::default_noop_collector(),
    );

    // Create a synthetic key-up event
    let event = InputEvent::with_metadata(
        KeyCode::A,
        false, // pressed (key up)
        100,   // timestamp
        None,
        false,
        true, // is_synthetic
        0,
        None,
    );
    let output = engine.process_event(&event);

    assert_eq!(output, OutputAction::PassThrough);
}

/// Test that synthetic events bypass blocking rules.
#[test]
fn process_event_synthetic_bypasses_block() {
    let input = MockInput::new();
    let mut runtime = MockRuntime::new();
    let state = MockState::new();

    // Configure CapsLock to be blocked
    runtime.registry_mut().block(KeyCode::CapsLock);

    let engine = Engine::new(
        input,
        runtime,
        state,
        keyrx_core::metrics::default_noop_collector(),
    );

    // Create a synthetic CapsLock event
    let event = InputEvent::with_metadata(
        KeyCode::CapsLock,
        true,
        0,
        None,
        false,
        true, // is_synthetic
        0,
        None,
    );
    let output = engine.process_event(&event);

    // Should pass through even though CapsLock is configured to be blocked
    assert_eq!(output, OutputAction::PassThrough);
}

/// Test that non-synthetic events are still processed normally.
#[test]
fn process_event_non_synthetic_still_remapped() {
    let input = MockInput::new();
    let mut runtime = MockRuntime::new();
    let state = MockState::new();

    // Configure A -> B remap
    runtime.registry_mut().remap(KeyCode::A, KeyCode::B);

    let engine = Engine::new(
        input,
        runtime,
        state,
        keyrx_core::metrics::default_noop_collector(),
    );

    // Create a non-synthetic event (is_synthetic = false)
    let event = InputEvent::with_metadata(
        KeyCode::A,
        true,
        0,
        Some("device".to_string()),
        false,
        false, // is_synthetic = false
        30,
        None,
    );
    let output = engine.process_event(&event);

    // Non-synthetic events should still be remapped
    assert_eq!(output, OutputAction::KeyDown(KeyCode::B));
}
