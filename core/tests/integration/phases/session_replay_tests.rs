#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::print_stdout, clippy::print_stderr, clippy::field_reassign_with_default, clippy::useless_conversion, clippy::assertions_on_constants, clippy::manual_div_ceil, clippy::manual_strip, clippy::len_zero, clippy::redundant_closure, clippy::manual_range_contains, clippy::default_constructed_unit_structs, clippy::clone_on_copy, clippy::io_other_error, clippy::bool_assert_comparison, clippy::approx_constant, clippy::let_unit_value, clippy::while_let_on_iterator, clippy::await_holding_lock, clippy::unnecessary_cast, clippy::drop_non_drop, clippy::needless_range_loop, unused_imports, unused_variables, dead_code, unsafe_code, clippy::collapsible_if, clippy::bool_comparison, unexpected_cfgs)]
//! Session Replay Integration Tests
//!
//! Tests verifying session replay functionality:
//! - Replaying recorded events
//! - Deterministic replay verification
//! - Replay speed control

use keyrx_core::cli::commands::ReplayCommand;
use keyrx_core::cli::OutputFormat;
use keyrx_core::engine::{
    DecisionType, EventRecordBuilder, EventRecorder, InputEvent, KeyCode, ModifierState,
    OutputAction, ReplaySession, StateSnapshot, TimingConfig,
};
use keyrx_core::traits::InputSource;
use tempfile::TempDir;

/// Create a default engine state for testing.
fn make_initial_state() -> StateSnapshot {
    let engine = keyrx_core::engine::AdvancedEngine::new(
        keyrx_core::scripting::RhaiRuntime::new().unwrap(),
        TimingConfig::default(),
    );
    engine.snapshot()
}

/// Test that recording events and replaying them produces identical outputs.
#[tokio::test]
async fn replay_produces_identical_events() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let session_path = temp_dir.path().join("identical.krx");

    // Generate a sequence of events
    let events: Vec<(InputEvent, Vec<OutputAction>)> = vec![
        (
            InputEvent::key_down(KeyCode::A, 0),
            vec![OutputAction::KeyDown(KeyCode::A)],
        ),
        (
            InputEvent::key_up(KeyCode::A, 10_000),
            vec![OutputAction::KeyUp(KeyCode::A)],
        ),
        (
            InputEvent::key_down(KeyCode::B, 20_000),
            vec![OutputAction::KeyDown(KeyCode::B)],
        ),
        (
            InputEvent::key_up(KeyCode::B, 30_000),
            vec![OutputAction::KeyUp(KeyCode::B)],
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

    for (seq, (input, output)) in events.iter().enumerate() {
        let record = EventRecordBuilder::new()
            .seq(seq as u64)
            .timestamp_us(input.timestamp_us)
            .input(input.clone())
            .output(output.clone())
            .decision_type(DecisionType::PassThrough)
            .active_layers(vec![0])
            .modifiers_state(ModifierState::default())
            .latency_us(25)
            .build();

        recorder.record_event(record);
    }

    let session = recorder.finish().expect("finish recording");
    assert_eq!(session.event_count(), 4);

    // Load and replay
    let mut replay = ReplaySession::from_file(&session_path).expect("load replay");
    replay.set_speed(0.0); // Instant replay

    replay.start().await.expect("start replay");

    let replayed = replay.poll_events().await.expect("poll events");
    assert_eq!(replayed.len(), 4, "Should replay all 4 events");

    // Verify each replayed event matches original
    for (i, replayed_event) in replayed.iter().enumerate() {
        let (original, _) = &events[i];
        assert_eq!(replayed_event.key, original.key, "Event {} key mismatch", i);
        assert_eq!(
            replayed_event.pressed, original.pressed,
            "Event {} pressed mismatch",
            i
        );
        assert_eq!(
            replayed_event.timestamp_us, original.timestamp_us,
            "Event {} timestamp mismatch",
            i
        );
    }
}

/// Test replay with remapped events.
#[tokio::test]
async fn replay_with_remapped_events() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let session_path = temp_dir.path().join("remapped.krx");

    // Record CapsLock → Escape remap
    let mut recorder = EventRecorder::new(
        &session_path,
        None,
        TimingConfig::default(),
        make_initial_state(),
    )
    .expect("create recorder");

    recorder.record_event(
        EventRecordBuilder::new()
            .seq(0)
            .timestamp_us(0)
            .input(InputEvent::key_down(KeyCode::CapsLock, 0))
            .output(vec![OutputAction::KeyDown(KeyCode::Escape)])
            .decision_type(DecisionType::Remap)
            .active_layers(vec![0])
            .modifiers_state(ModifierState::default())
            .latency_us(25)
            .build(),
    );

    recorder.record_event(
        EventRecordBuilder::new()
            .seq(1)
            .timestamp_us(50_000)
            .input(InputEvent::key_up(KeyCode::CapsLock, 50_000))
            .output(vec![OutputAction::KeyUp(KeyCode::Escape)])
            .decision_type(DecisionType::Remap)
            .active_layers(vec![0])
            .modifiers_state(ModifierState::default())
            .latency_us(25)
            .build(),
    );

    let session = recorder.finish().expect("finish");

    // Replay and verify
    let mut replay = ReplaySession::from_file(&session_path).expect("load");
    replay.set_speed(0.0);
    replay.start().await.expect("start");

    let replayed = replay.poll_events().await.expect("poll");
    assert_eq!(replayed.len(), 2);

    // Verify output matches the remap (Escape, not CapsLock)
    assert_eq!(
        session.events[0].output[0],
        OutputAction::KeyDown(KeyCode::Escape)
    );
    assert_eq!(
        session.events[1].output[0],
        OutputAction::KeyUp(KeyCode::Escape)
    );
}

/// Test replay command with verification flag.
#[tokio::test]
async fn replay_command_with_verification() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let session_path = temp_dir.path().join("verify_test.krx");

    // Create a simple session with pass-through events
    let mut recorder = EventRecorder::new(
        &session_path,
        None,
        TimingConfig::default(),
        make_initial_state(),
    )
    .expect("create recorder");

    for i in 0..5u64 {
        let input = InputEvent::key_down(KeyCode::A, i * 10_000);
        recorder.record_event(
            EventRecordBuilder::new()
                .seq(i)
                .timestamp_us(i * 10_000)
                .input(input)
                .output(vec![OutputAction::KeyDown(KeyCode::A)])
                .decision_type(DecisionType::PassThrough)
                .active_layers(vec![0])
                .modifiers_state(ModifierState::default())
                .latency_us(20)
                .build(),
        );
    }

    recorder.finish().expect("finish");

    // Run replay command with verification
    let cmd = ReplayCommand::new(session_path, OutputFormat::Human).with_verify(true);

    let result = cmd.run().await.expect("replay should succeed");

    assert_eq!(result.total_events, 5);
    assert!(
        result.all_matched(),
        "All events should match for pass-through"
    );
}

/// Test empty session replay.
#[tokio::test]
async fn replay_empty_session() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let session_path = temp_dir.path().join("empty.krx");

    // Create empty session
    let recorder = EventRecorder::new(
        &session_path,
        None,
        TimingConfig::default(),
        make_initial_state(),
    )
    .expect("create recorder");

    recorder.finish().expect("finish empty");

    // Replay empty session
    let mut replay = ReplaySession::from_file(&session_path).expect("load");
    replay.start().await.expect("start");
    let events = replay.poll_events().await.expect("poll");
    assert!(events.is_empty(), "Empty session should have no events");
}

/// Test replay with instant speed (0.0).
#[tokio::test]
async fn replay_instant_speed() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let session_path = temp_dir.path().join("instant.krx");

    let mut recorder = EventRecorder::new(
        &session_path,
        None,
        TimingConfig::default(),
        make_initial_state(),
    )
    .expect("create recorder");

    // Events with large time gaps
    for i in 0..3u64 {
        let input = InputEvent::key_down(KeyCode::A, i * 1_000_000); // 1 second gaps
        recorder.record_event(
            EventRecordBuilder::new()
                .seq(i)
                .timestamp_us(i * 1_000_000)
                .input(input)
                .output(vec![OutputAction::PassThrough])
                .decision_type(DecisionType::PassThrough)
                .active_layers(vec![0])
                .modifiers_state(ModifierState::default())
                .latency_us(15)
                .build(),
        );
    }

    recorder.finish().expect("finish");

    // Replay at instant speed should complete quickly
    let mut replay = ReplaySession::from_file(&session_path).expect("load");
    replay.set_speed(0.0);

    let start = std::time::Instant::now();
    replay.start().await.expect("start");
    let events = replay.poll_events().await.expect("poll");
    let elapsed = start.elapsed();

    assert_eq!(events.len(), 3);
    // Should complete in well under 100ms (much less than the 2 seconds of recorded events)
    assert!(
        elapsed.as_millis() < 100,
        "Instant replay should be fast, took {:?}",
        elapsed
    );
}

/// Test verification of outputs during replay.
#[tokio::test]
async fn replay_verifies_output_matches() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let session_path = temp_dir.path().join("verify_output.krx");

    let events = [
        (KeyCode::A, DecisionType::PassThrough),
        (KeyCode::CapsLock, DecisionType::Remap),
        (KeyCode::Insert, DecisionType::Block),
    ];

    let mut recorder = EventRecorder::new(
        &session_path,
        None,
        TimingConfig::default(),
        make_initial_state(),
    )
    .expect("create recorder");

    for (seq, (key, decision)) in events.iter().enumerate() {
        let input = InputEvent::key_down(*key, (seq * 10_000) as u64);
        let output = match decision {
            DecisionType::Remap => vec![OutputAction::KeyDown(KeyCode::Escape)],
            DecisionType::Block => vec![],
            _ => vec![OutputAction::KeyDown(*key)],
        };

        recorder.record_event(
            EventRecordBuilder::new()
                .seq(seq as u64)
                .timestamp_us((seq * 10_000) as u64)
                .input(input)
                .output(output.clone())
                .decision_type(*decision)
                .active_layers(vec![0])
                .modifiers_state(ModifierState::default())
                .latency_us(25)
                .build(),
        );
    }

    let session = recorder.finish().expect("finish");

    // Verify outputs match what we recorded
    for (seq, (_, _)) in events.iter().enumerate() {
        let recorded = &session.events[seq];
        // Each event should have the output we specified
        match recorded.decision_type {
            DecisionType::Remap => {
                assert_eq!(recorded.output[0], OutputAction::KeyDown(KeyCode::Escape))
            }
            DecisionType::Block => assert!(recorded.output.is_empty()),
            DecisionType::PassThrough => {
                assert_eq!(
                    recorded.output[0],
                    OutputAction::KeyDown(recorded.input.key)
                )
            }
            _ => {}
        }
    }
}
