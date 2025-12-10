#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::print_stdout, clippy::print_stderr, clippy::field_reassign_with_default, clippy::useless_conversion, clippy::assertions_on_constants, clippy::manual_div_ceil, clippy::manual_strip, clippy::len_zero, clippy::redundant_closure, clippy::manual_range_contains, clippy::default_constructed_unit_structs, clippy::clone_on_copy, clippy::io_other_error, clippy::bool_assert_comparison, clippy::approx_constant, clippy::let_unit_value, clippy::while_let_on_iterator, clippy::await_holding_lock, clippy::unnecessary_cast, clippy::drop_non_drop, clippy::needless_range_loop, unused_imports, unused_variables, dead_code, unsafe_code, clippy::collapsible_if, clippy::bool_comparison, unexpected_cfgs)]
//! Session Recording Integration Tests
//!
//! Tests verifying session recording functionality:
//! - Event recording with various decision types
//! - Session file creation and serialization
//! - Recording metadata preservation

use keyrx_core::engine::{
    DecisionType, EventRecordBuilder, EventRecorder, InputEvent, KeyCode, ModifierState,
    OutputAction, SessionFile, StateSnapshot, TimingConfig,
};
use std::fs;
use tempfile::TempDir;

/// Create a default engine state for testing.
fn make_initial_state() -> StateSnapshot {
    let engine = keyrx_core::engine::AdvancedEngine::new(
        keyrx_core::scripting::RhaiRuntime::new().unwrap(),
        TimingConfig::default(),
    );
    engine.snapshot()
}

/// Test that recording events creates a valid session file.
#[test]
fn record_events_creates_session() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let session_path = temp_dir.path().join("test_record.krx");

    // Generate a sequence of events
    let events: Vec<(InputEvent, Vec<OutputAction>, DecisionType)> = vec![
        (
            InputEvent::key_down(KeyCode::A, 0),
            vec![OutputAction::KeyDown(KeyCode::A)],
            DecisionType::PassThrough,
        ),
        (
            InputEvent::key_up(KeyCode::A, 10_000),
            vec![OutputAction::KeyUp(KeyCode::A)],
            DecisionType::PassThrough,
        ),
        (
            InputEvent::key_down(KeyCode::CapsLock, 20_000),
            vec![OutputAction::KeyDown(KeyCode::Escape)],
            DecisionType::Remap,
        ),
        (
            InputEvent::key_up(KeyCode::CapsLock, 30_000),
            vec![OutputAction::KeyUp(KeyCode::Escape)],
            DecisionType::Remap,
        ),
    ];

    // Record events
    let mut recorder = EventRecorder::new(
        &session_path,
        None,
        TimingConfig::default(),
        make_initial_state(),
    )
    .expect("create recorder");

    for (seq, (input, output, decision)) in events.iter().enumerate() {
        let record = EventRecordBuilder::new()
            .seq(seq as u64)
            .timestamp_us(input.timestamp_us)
            .input(input.clone())
            .output(output.clone())
            .decision_type(*decision)
            .active_layers(vec![0])
            .modifiers_state(ModifierState::default())
            .latency_us(25)
            .build();

        recorder.record_event(record);
    }

    let session = recorder.finish().expect("finish recording");
    assert_eq!(session.event_count(), 4);

    // Verify file was created
    assert!(session_path.exists(), "Session file should exist");
}

/// Test recording with script metadata.
#[test]
fn record_with_script_metadata() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let session_path = temp_dir.path().join("with_script.krx");

    let recorder = EventRecorder::new(
        &session_path,
        Some("/path/to/test.rhai".to_string()),
        TimingConfig::default(),
        make_initial_state(),
    )
    .expect("create recorder");

    let session = recorder.finish().expect("finish");

    // Verify script path is preserved
    assert_eq!(session.script_path, Some("/path/to/test.rhai".to_string()));
}

/// Test recording diverse decision types.
#[test]
fn record_diverse_decision_types() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let session_path = temp_dir.path().join("diverse.krx");

    let mut recorder = EventRecorder::new(
        &session_path,
        None,
        TimingConfig::default(),
        make_initial_state(),
    )
    .expect("create recorder");

    let test_events = [
        (KeyCode::A, DecisionType::PassThrough),
        (KeyCode::CapsLock, DecisionType::Remap),
        (KeyCode::Insert, DecisionType::Block),
        (KeyCode::Space, DecisionType::Tap),
        (KeyCode::LeftShift, DecisionType::Hold),
    ];

    for (i, (key, decision)) in test_events.iter().enumerate() {
        let input = InputEvent::key_down(*key, (i * 10_000) as u64);
        recorder.record_event(
            EventRecordBuilder::new()
                .seq(i as u64)
                .timestamp_us((i * 10_000) as u64)
                .input(input)
                .output(vec![OutputAction::KeyDown(*key)])
                .decision_type(*decision)
                .active_layers(vec![0])
                .modifiers_state(ModifierState::default())
                .latency_us(30)
                .build(),
        );
    }

    let session = recorder.finish().expect("finish");
    assert_eq!(session.event_count(), 5);

    // Verify all decision types are recorded
    let decisions: Vec<_> = session.events.iter().map(|e| e.decision_type).collect();
    assert!(decisions.contains(&DecisionType::PassThrough));
    assert!(decisions.contains(&DecisionType::Remap));
    assert!(decisions.contains(&DecisionType::Block));
    assert!(decisions.contains(&DecisionType::Tap));
    assert!(decisions.contains(&DecisionType::Hold));
}

/// Test empty session handling.
#[test]
fn record_empty_session() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let session_path = temp_dir.path().join("empty.krx");

    let recorder = EventRecorder::new(
        &session_path,
        None,
        TimingConfig::default(),
        make_initial_state(),
    )
    .expect("create recorder");

    let session = recorder.finish().expect("finish empty");
    assert_eq!(session.event_count(), 0);
    assert!(session_path.exists());
}

/// Test session file version preservation.
#[test]
fn session_version_preservation() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let session_path = temp_dir.path().join("version.krx");

    let mut session = SessionFile::new(
        "2024-01-15T10:30:00Z".to_string(),
        None,
        TimingConfig::default(),
        make_initial_state(),
    );

    session.add_event(
        EventRecordBuilder::new()
            .seq(0)
            .timestamp_us(0)
            .input(InputEvent::key_down(KeyCode::A, 0))
            .output(vec![OutputAction::PassThrough])
            .decision_type(DecisionType::PassThrough)
            .active_layers(vec![0])
            .modifiers_state(ModifierState::default())
            .latency_us(10)
            .build(),
    );

    let json = session.to_json().expect("serialize");
    fs::write(&session_path, &json).expect("write");

    // Verify version is preserved when loaded
    let loaded = SessionFile::from_json(&json).expect("parse");
    assert_eq!(loaded.version, session.version);
}

/// Test recording with various latencies.
#[test]
fn record_with_latency_variations() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let session_path = temp_dir.path().join("latency.krx");

    let mut recorder = EventRecorder::new(
        &session_path,
        None,
        TimingConfig::default(),
        make_initial_state(),
    )
    .expect("create recorder");

    let latencies = [10, 25, 50, 100, 200];

    for (i, latency) in latencies.iter().enumerate() {
        let input = InputEvent::key_down(KeyCode::A, (i * 10_000) as u64);
        recorder.record_event(
            EventRecordBuilder::new()
                .seq(i as u64)
                .timestamp_us((i * 10_000) as u64)
                .input(input)
                .output(vec![OutputAction::PassThrough])
                .decision_type(DecisionType::PassThrough)
                .active_layers(vec![0])
                .modifiers_state(ModifierState::default())
                .latency_us(*latency)
                .build(),
        );
    }

    let session = recorder.finish().expect("finish");

    // Verify latencies are recorded
    for (i, expected_latency) in latencies.iter().enumerate() {
        assert_eq!(session.events[i].latency_us, *expected_latency);
    }
}
