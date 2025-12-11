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
//! Integration tests for session recording and replay.
//!
//! Tests deterministic recording of events, replay with timing verification,
//! and proper handling of corrupted/empty session files.

use keyrx_core::engine::{
    DecisionType, EventRecordBuilder, EventRecorder, InputEvent, KeyCode, ModifierState,
    OutputAction, ReplaySession, ReplayState, SessionFile, StateSnapshot, TimingConfig,
    SESSION_FILE_VERSION,
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

/// Generate a sequence of test events simulating key presses.
fn generate_test_events(count: usize) -> Vec<(InputEvent, Vec<OutputAction>)> {
    let keys = [
        KeyCode::A,
        KeyCode::B,
        KeyCode::C,
        KeyCode::D,
        KeyCode::E,
        KeyCode::F,
        KeyCode::G,
        KeyCode::H,
        KeyCode::I,
        KeyCode::J,
    ];

    let mut events = Vec::with_capacity(count);
    for i in 0..count {
        let key = keys[i % keys.len()];
        let timestamp = (i as u64) * 10_000; // 10ms between events

        // Alternate between down and up events
        let (input, output) = if i % 2 == 0 {
            (
                InputEvent::key_down(key, timestamp),
                vec![OutputAction::KeyDown(key)],
            )
        } else {
            (
                InputEvent::key_up(key, timestamp),
                vec![OutputAction::KeyUp(key)],
            )
        };

        events.push((input, output));
    }
    events
}

/// Record events, replay them, and verify outputs match exactly.
#[tokio::test]
async fn record_100_events_and_replay_matches() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let session_path = temp_dir.path().join("test_session.krx");

    // Generate 100 test events
    let events = generate_test_events(100);

    // Record the events
    let mut recorder = EventRecorder::new(
        &session_path,
        Some("/test/script.rhai".to_string()),
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
            .latency_us(50)
            .build();

        recorder.record_event(record);
    }

    assert_eq!(recorder.event_count(), 100);

    // Finish recording (writes to file)
    let session = recorder.finish().expect("finish recording");
    assert_eq!(session.event_count(), 100);

    // Verify file was written
    assert!(session_path.exists());

    // Load the recorded session for replay
    let mut replay = ReplaySession::from_file(&session_path).expect("load replay");

    assert_eq!(replay.total_events(), 100);
    assert_eq!(replay.events_remaining(), 100);
    assert_eq!(replay.state(), ReplayState::Idle);

    // Set instant mode (no delays) for fast testing
    replay.set_speed(0.0);

    // Start replay
    replay.start().await.expect("start replay");
    assert_eq!(replay.state(), ReplayState::Playing);

    // Poll all events (instant mode returns all at once)
    let replayed_events = replay.poll_events().await.expect("poll events");
    assert_eq!(replayed_events.len(), 100);

    // Verify each replayed event matches the original
    for (i, replayed) in replayed_events.iter().enumerate() {
        let (original, _) = &events[i];
        assert_eq!(
            replayed.key, original.key,
            "Event {} key mismatch: expected {:?}, got {:?}",
            i, original.key, replayed.key
        );
        assert_eq!(
            replayed.pressed, original.pressed,
            "Event {} pressed mismatch: expected {}, got {}",
            i, original.pressed, replayed.pressed
        );
    }

    // Second poll should complete replay
    let remaining = replay.poll_events().await.expect("poll remaining");
    assert!(remaining.is_empty());
    assert!(replay.is_completed());

    // Verify recorded outputs match through session accessor
    let session_ref = replay.session().expect("legacy session available");
    for (seq, (_, expected_output)) in events.iter().enumerate() {
        let recorded_output = session_ref
            .events
            .iter()
            .find(|e| e.seq == seq as u64)
            .map(|e| &e.output)
            .expect("find event by seq");

        assert_eq!(
            recorded_output, expected_output,
            "Event {} output mismatch",
            seq
        );
    }
}

/// Test that malformed JSON is properly rejected.
#[test]
fn corrupted_file_returns_error_invalid_json() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let session_path = temp_dir.path().join("corrupted.krx");

    // Write invalid JSON
    std::fs::write(&session_path, "{ this is not valid json }").expect("write corrupted file");

    let result = ReplaySession::from_file(&session_path);
    assert!(result.is_err(), "Should fail to parse corrupted JSON");

    let error = result.unwrap_err();
    let error_str = error.to_string();
    assert!(
        error_str.contains("parse") || error_str.contains("Failed"),
        "Error should mention parsing: {}",
        error_str
    );
}

/// Test that truncated JSON files are rejected.
#[test]
fn corrupted_file_returns_error_truncated() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let session_path = temp_dir.path().join("truncated.krx");

    // Write truncated JSON (missing closing brace)
    let truncated = r#"{"version": 1, "created_at": "2024-01-15""#;
    std::fs::write(&session_path, truncated).expect("write truncated file");

    let result = ReplaySession::from_file(&session_path);
    assert!(result.is_err(), "Should fail to parse truncated JSON");
}

/// Test that files with missing required fields are rejected.
#[test]
fn corrupted_file_returns_error_missing_fields() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let session_path = temp_dir.path().join("incomplete.krx");

    // Write JSON missing required fields
    let incomplete = r#"{"version": 1}"#;
    std::fs::write(&session_path, incomplete).expect("write incomplete file");

    let result = ReplaySession::from_file(&session_path);
    assert!(result.is_err(), "Should fail with missing fields");
}

/// Test that empty session files are handled gracefully.
#[tokio::test]
async fn empty_session_handles_gracefully() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let session_path = temp_dir.path().join("empty_session.krx");

    // Create a session with no events
    let recorder = EventRecorder::new(
        &session_path,
        None,
        TimingConfig::default(),
        make_initial_state(),
    )
    .expect("create recorder");

    // Don't add any events - finish immediately
    let session = recorder.finish().expect("finish empty recording");
    assert_eq!(session.event_count(), 0);

    // Load and replay empty session
    let mut replay = ReplaySession::from_file(&session_path).expect("load empty replay");

    assert_eq!(replay.total_events(), 0);
    assert_eq!(replay.events_remaining(), 0);

    replay.start().await.expect("start empty replay");

    // Polling empty session should complete immediately
    let events = replay.poll_events().await.expect("poll empty");
    assert!(events.is_empty());
    assert!(replay.is_completed());
}

/// Test that nonexistent files return appropriate errors.
#[test]
fn nonexistent_file_returns_error() {
    let result = ReplaySession::from_file("/nonexistent/path/to/session.krx");
    assert!(result.is_err(), "Should fail for nonexistent file");

    let error = result.unwrap_err();
    let error_str = error.to_string();
    assert!(
        error_str.contains("read") || error_str.contains("Failed"),
        "Error should mention reading: {}",
        error_str
    );
}

/// Test session file statistics are calculated correctly.
#[test]
fn session_statistics_are_accurate() {
    let mut session = SessionFile::new(
        "2024-01-15T10:30:00Z".to_string(),
        Some("/test/script.rhai".to_string()),
        TimingConfig::default(),
        make_initial_state(),
    );

    // Add events with known timestamps and latencies
    let latencies = [10, 20, 30, 40, 50];
    for (i, &latency) in latencies.iter().enumerate() {
        let input = InputEvent::key_down(KeyCode::A, (i as u64) * 100_000);
        session.add_event(
            EventRecordBuilder::new()
                .seq(i as u64)
                .timestamp_us((i as u64) * 100_000) // 0, 100ms, 200ms, 300ms, 400ms
                .input(input)
                .output(vec![OutputAction::PassThrough])
                .decision_type(DecisionType::PassThrough)
                .active_layers(vec![0])
                .modifiers_state(ModifierState::default())
                .latency_us(latency)
                .build(),
        );
    }

    assert_eq!(session.event_count(), 5);
    assert_eq!(session.duration_us(), 400_000); // Last event timestamp
    assert_eq!(session.avg_latency_us(), 30); // (10+20+30+40+50)/5
}

/// Test session file roundtrip through JSON.
#[test]
fn session_file_json_roundtrip() {
    let mut session = SessionFile::new(
        "2024-01-15T10:30:00Z".to_string(),
        Some("/test/script.rhai".to_string()),
        TimingConfig::default(),
        make_initial_state(),
    );

    // Add diverse events with different decision types
    let test_cases = [
        (KeyCode::A, KeyCode::B, DecisionType::Remap),
        (KeyCode::CapsLock, KeyCode::Escape, DecisionType::Tap),
        (KeyCode::Space, KeyCode::Space, DecisionType::PassThrough),
        (KeyCode::Insert, KeyCode::Insert, DecisionType::Block),
    ];

    for (i, (from_key, to_key, decision)) in test_cases.iter().enumerate() {
        let input = InputEvent::key_down(*from_key, (i as u64) * 1000);
        let output = match decision {
            DecisionType::Block => vec![OutputAction::Block],
            DecisionType::PassThrough => vec![OutputAction::PassThrough],
            DecisionType::Remap | DecisionType::Tap => vec![OutputAction::KeyDown(*to_key)],
            _ => vec![OutputAction::PassThrough],
        };

        session.add_event(
            EventRecordBuilder::new()
                .seq(i as u64)
                .timestamp_us((i as u64) * 1000)
                .input(input)
                .output(output)
                .decision_type(*decision)
                .active_layers(vec![0])
                .modifiers_state(ModifierState::default())
                .latency_us(25)
                .build(),
        );
    }

    // Roundtrip through JSON
    let json = session.to_json().expect("serialize to JSON");
    let loaded = SessionFile::from_json(&json).expect("deserialize from JSON");

    assert_eq!(loaded.version, SESSION_FILE_VERSION);
    assert_eq!(loaded.created_at, session.created_at);
    assert_eq!(loaded.script_path, session.script_path);
    assert_eq!(loaded.event_count(), session.event_count());

    // Verify each event
    for (original, loaded_event) in session.events.iter().zip(loaded.events.iter()) {
        assert_eq!(original.seq, loaded_event.seq);
        assert_eq!(original.timestamp_us, loaded_event.timestamp_us);
        assert_eq!(original.input.key, loaded_event.input.key);
        assert_eq!(original.decision_type, loaded_event.decision_type);
        assert_eq!(original.output, loaded_event.output);
    }
}

/// Test replay with timed events (non-instant mode).
#[tokio::test]
async fn replay_respects_timing() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let session_path = temp_dir.path().join("timed_session.krx");

    // Create session with events spaced 50ms apart
    let mut recorder = EventRecorder::new(
        &session_path,
        None,
        TimingConfig::default(),
        make_initial_state(),
    )
    .expect("create recorder");

    for i in 0..3u64 {
        let input = InputEvent::key_down(KeyCode::A, i * 50_000); // 0, 50ms, 100ms
        recorder.record_event(
            EventRecordBuilder::new()
                .seq(i)
                .timestamp_us(i * 50_000)
                .input(input)
                .output(vec![OutputAction::KeyDown(KeyCode::A)])
                .decision_type(DecisionType::PassThrough)
                .active_layers(vec![0])
                .modifiers_state(ModifierState::default())
                .latency_us(10)
                .build(),
        );
    }

    recorder.finish().expect("finish");

    // Load for replay at 10x speed (should complete in ~10ms instead of 100ms)
    let mut replay = ReplaySession::from_file(&session_path).expect("load");
    replay.set_speed(10.0);

    replay.start().await.expect("start");

    // First event (timestamp 0) should be immediately available
    let events = replay.poll_events().await.expect("poll");
    assert!(
        !events.is_empty(),
        "First event should be available immediately"
    );

    // Wait a bit and check that more events become available
    tokio::time::sleep(tokio::time::Duration::from_millis(15)).await;

    let mut total_events = events.len();
    while !replay.is_completed() {
        let more = replay.poll_events().await.expect("poll more");
        total_events += more.len();
        if more.is_empty() && !replay.is_completed() {
            tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
        }
    }

    assert_eq!(total_events, 3, "All events should be replayed");
}

/// Test recorder abort doesn't write file.
#[test]
fn recorder_abort_does_not_create_file() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let session_path = temp_dir.path().join("aborted.krx");

    let mut recorder = EventRecorder::new(
        &session_path,
        None,
        TimingConfig::default(),
        make_initial_state(),
    )
    .expect("create recorder");

    // Add some events
    for i in 0..10 {
        let input = InputEvent::key_down(KeyCode::A, i * 1000);
        recorder.record_event(
            EventRecordBuilder::new()
                .seq(i)
                .timestamp_us(i * 1000)
                .input(input)
                .output(vec![OutputAction::KeyDown(KeyCode::A)])
                .decision_type(DecisionType::PassThrough)
                .active_layers(vec![0])
                .modifiers_state(ModifierState::default())
                .latency_us(10)
                .build(),
        );
    }

    // Abort instead of finish
    recorder.abort();

    // File should not exist
    assert!(
        !session_path.exists(),
        "Aborted session should not create file"
    );
}

/// Test replay stop clears state.
#[tokio::test]
async fn replay_stop_clears_events() {
    let session = SessionFile::new(
        "2024-01-15T10:30:00Z".to_string(),
        None,
        TimingConfig::default(),
        make_initial_state(),
    );

    // Add some events to a session in memory
    let mut session = session;
    for i in 0..5 {
        let input = InputEvent::key_down(KeyCode::A, i * 10_000);
        session.add_event(
            EventRecordBuilder::new()
                .seq(i)
                .timestamp_us(i * 10_000)
                .input(input)
                .output(vec![OutputAction::KeyDown(KeyCode::A)])
                .decision_type(DecisionType::PassThrough)
                .active_layers(vec![0])
                .modifiers_state(ModifierState::default())
                .latency_us(10)
                .build(),
        );
    }

    let mut replay = ReplaySession::new(session);
    assert_eq!(replay.events_remaining(), 5);

    replay.start().await.expect("start");
    assert_eq!(replay.state(), ReplayState::Playing);

    // Stop in the middle
    replay.stop().await.expect("stop");

    assert_eq!(replay.state(), ReplayState::Idle);
    assert_eq!(
        replay.events_remaining(),
        0,
        "Stop should clear remaining events"
    );
}

/// Test that session version is preserved correctly.
#[test]
fn session_version_preserved() {
    let session = SessionFile::new(
        "2024-01-15T10:30:00Z".to_string(),
        None,
        TimingConfig::default(),
        make_initial_state(),
    );

    assert_eq!(session.version, SESSION_FILE_VERSION);

    let json = session.to_json().expect("serialize");
    let loaded = SessionFile::from_json(&json).expect("deserialize");
    assert_eq!(loaded.version, SESSION_FILE_VERSION);
}
