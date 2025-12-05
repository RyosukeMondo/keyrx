//! Phase 1-3 Integration Tests
//!
//! End-to-end tests verifying the full feature chain:
//! - Script testing → test discovery → test runner
//! - Session recording → replay → analysis
//! - Commands work together in realistic workflows

use keyrx_core::cli::commands::{AnalyzeCommand, ReplayCommand, TestCommand};
use keyrx_core::cli::OutputFormat;
use keyrx_core::engine::{
    DecisionType, EventRecordBuilder, EventRecorder, InputEvent, KeyCode, ModifierState,
    OutputAction, ReplaySession, SessionFile, StateSnapshot, TimingConfig,
};
use keyrx_core::scripting::test_discovery::discover_tests;
use keyrx_core::scripting::test_runner::{TestRunner, TestSummary};
use keyrx_core::scripting::RhaiRuntime;
use keyrx_core::traits::{InputSource, ScriptRuntime};
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

// =============================================================================
// Test Script → Run → Verify Chain
// =============================================================================

/// Test that a Rhai test script can be written, discovered, and executed.
#[test]
fn write_test_script_discover_and_run() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let script_path = temp_dir.path().join("test_remapping.rhai");

    // Write a test script with multiple test functions
    let script_content = r#"
        // Helper function (not a test)
        fn helper_add(a, b) {
            a + b
        }

        // Test: Simple arithmetic
        fn test_arithmetic() {
            let result = helper_add(2, 3);
            if result != 5 {
                throw "Expected 5, got " + result;
            }
        }

        // Test: String operations
        fn test_string_concat() {
            let s = "hello" + " world";
            if s != "hello world" {
                throw "String concat failed";
            }
        }

        // Test: Array operations
        fn test_array_push() {
            let arr = [];
            arr.push(1);
            arr.push(2);
            if arr.len() != 2 {
                throw "Array length should be 2";
            }
        }
    "#;
    fs::write(&script_path, script_content).expect("write script");

    // Compile for discovery
    let engine = rhai::Engine::new();
    let ast = engine.compile(script_content).expect("compile script");

    // Discover tests
    let tests = discover_tests(&ast);
    assert_eq!(tests.len(), 3, "Should discover 3 test functions");

    // Verify test names
    let test_names: Vec<&str> = tests.iter().map(|t| t.name.as_str()).collect();
    assert!(test_names.contains(&"test_arithmetic"));
    assert!(test_names.contains(&"test_string_concat"));
    assert!(test_names.contains(&"test_array_push"));
    assert!(
        !test_names.contains(&"helper_add"),
        "Helper should not be discovered"
    );

    // Create runtime and run tests
    let mut runtime = RhaiRuntime::new().expect("create runtime");
    runtime
        .load_file(script_path.to_str().unwrap())
        .expect("load script");

    let runner = TestRunner::new();
    let results = runner.run_tests(&mut runtime, &tests);

    // Verify all tests pass
    let summary = TestSummary::from_results(&results);
    assert_eq!(summary.total, 3);
    assert_eq!(summary.passed, 3);
    assert_eq!(summary.failed, 0);
    assert!(summary.all_passed());
}

/// Test that failing tests are properly detected and reported.
#[test]
fn test_script_with_failures_detected() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let script_path = temp_dir.path().join("failing_tests.rhai");

    let script_content = r#"
        fn test_passing() {
            let x = 1 + 1;
        }

        fn test_failing() {
            throw "This test intentionally fails";
        }

        fn test_another_pass() {
            let arr = [1, 2, 3];
        }
    "#;
    fs::write(&script_path, script_content).expect("write script");

    let engine = rhai::Engine::new();
    let ast = engine.compile(script_content).expect("compile");
    let tests = discover_tests(&ast);

    let mut runtime = RhaiRuntime::new().expect("create runtime");
    runtime
        .load_file(script_path.to_str().unwrap())
        .expect("load script");

    let runner = TestRunner::new();
    let results = runner.run_tests(&mut runtime, &tests);

    let summary = TestSummary::from_results(&results);
    assert_eq!(summary.total, 3);
    assert_eq!(summary.passed, 2);
    assert_eq!(summary.failed, 1);
    assert!(!summary.all_passed());

    // Verify the failing test has proper error message
    let failed = results
        .iter()
        .find(|r| !r.passed)
        .expect("find failed test");
    assert_eq!(failed.name, "test_failing");
    assert!(failed.message.contains("intentionally fails"));
}

/// Test the TestCommand CLI interface end-to-end.
#[test]
fn test_command_end_to_end() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let script_path = temp_dir.path().join("cli_test.rhai");

    let script_content = r#"
        fn test_simple() {
            let x = 42;
        }
    "#;
    fs::write(&script_path, script_content).expect("write script");

    let cmd = TestCommand::new(script_path, OutputFormat::Human);
    let exit_code = cmd.run().expect("run test command");

    assert_eq!(exit_code, 0, "Exit code should be 0 for passing tests");
}

// =============================================================================
// Record → Replay → Deterministic Match Chain
// =============================================================================

/// Test that recording events and replaying them produces identical outputs.
#[tokio::test]
async fn record_replay_deterministic_match() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let session_path = temp_dir.path().join("deterministic.krx");

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
        (
            InputEvent::key_down(KeyCode::CapsLock, 40_000),
            vec![OutputAction::KeyDown(KeyCode::Escape)], // Simulated remap
        ),
        (
            InputEvent::key_up(KeyCode::CapsLock, 50_000),
            vec![OutputAction::KeyUp(KeyCode::Escape)],
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
        let decision = if input.key == KeyCode::CapsLock {
            DecisionType::Remap
        } else {
            DecisionType::PassThrough
        };

        let record = EventRecordBuilder::new()
            .seq(seq as u64)
            .timestamp_us(input.timestamp_us)
            .input(input.clone())
            .output(output.clone())
            .decision_type(decision)
            .active_layers(vec![0])
            .modifiers_state(ModifierState::default())
            .latency_us(25)
            .build();

        recorder.record_event(record);
    }

    let session = recorder.finish().expect("finish recording");
    assert_eq!(session.event_count(), 6);

    // Load and replay
    let mut replay = ReplaySession::from_file(&session_path).expect("load replay");
    replay.set_speed(0.0); // Instant replay

    replay.start().await.expect("start replay");

    let replayed = replay.poll_events().await.expect("poll events");
    assert_eq!(replayed.len(), 6, "Should replay all 6 events");

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

    // Verify outputs from session match what we recorded
    for (seq, (_, expected_output)) in events.iter().enumerate() {
        let recorded = session
            .events
            .iter()
            .find(|e| e.seq == seq as u64)
            .expect("find event");
        assert_eq!(
            &recorded.output, expected_output,
            "Event {} output mismatch",
            seq
        );
    }
}

/// Test replay command with verification flag.
#[tokio::test]
async fn replay_with_verification() {
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

// =============================================================================
// Analyze Command Output Verification
// =============================================================================

/// Test that analyze command produces correct timing statistics.
#[test]
fn analyze_outputs_timing_diagram() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let session_path = temp_dir.path().join("analyze_test.krx");

    // Create session with diverse decision types
    let mut session = SessionFile::new(
        "2024-01-15T10:30:00Z".to_string(),
        Some("/test/script.rhai".to_string()),
        TimingConfig::default(),
        make_initial_state(),
    );

    let test_events = [
        (KeyCode::A, DecisionType::PassThrough, 30),
        (KeyCode::CapsLock, DecisionType::Remap, 45),
        (KeyCode::Insert, DecisionType::Block, 20),
        (KeyCode::Space, DecisionType::Tap, 60),
        (KeyCode::LeftShift, DecisionType::Hold, 80),
    ];

    for (i, (key, decision, latency)) in test_events.iter().enumerate() {
        let input = InputEvent::key_down(*key, (i * 10_000) as u64);
        session.add_event(
            EventRecordBuilder::new()
                .seq(i as u64)
                .timestamp_us((i * 10_000) as u64)
                .input(input)
                .output(vec![OutputAction::KeyDown(*key)])
                .decision_type(*decision)
                .active_layers(vec![0])
                .modifiers_state(ModifierState::default())
                .latency_us(*latency)
                .build(),
        );
    }

    // Write session to file
    let content = session.to_json().expect("serialize");
    fs::write(&session_path, content).expect("write file");

    // Run analyze command
    let cmd = AnalyzeCommand::new(session_path, OutputFormat::Human).with_diagram(true);

    let result = cmd.run().into_result().expect("analyze should succeed");

    // Verify statistics
    assert_eq!(result.event_count, 5);
    assert_eq!(result.duration_us, 40_000); // Last event timestamp
    assert_eq!(result.min_latency_us, 20);
    assert_eq!(result.max_latency_us, 80);
    assert_eq!(result.avg_latency_us, 47); // (30+45+20+60+80)/5 = 47

    // Verify decision breakdown
    assert_eq!(result.decision_breakdown.pass_through, 1);
    assert_eq!(result.decision_breakdown.remap, 1);
    assert_eq!(result.decision_breakdown.block, 1);
    assert_eq!(result.decision_breakdown.tap, 1);
    assert_eq!(result.decision_breakdown.hold, 1);
}

/// Test analyze command JSON output format.
#[test]
fn analyze_json_output() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let session_path = temp_dir.path().join("json_analyze.krx");

    let mut session = SessionFile::new(
        "2024-01-15T10:30:00Z".to_string(),
        None,
        TimingConfig::default(),
        make_initial_state(),
    );

    // Add a few events
    for i in 0..3u64 {
        let input = InputEvent::key_down(KeyCode::A, i * 5_000);
        session.add_event(
            EventRecordBuilder::new()
                .seq(i)
                .timestamp_us(i * 5_000)
                .input(input)
                .output(vec![OutputAction::PassThrough])
                .decision_type(DecisionType::PassThrough)
                .active_layers(vec![0])
                .modifiers_state(ModifierState::default())
                .latency_us(15)
                .build(),
        );
    }

    let content = session.to_json().expect("serialize");
    fs::write(&session_path, content).expect("write file");

    let cmd = AnalyzeCommand::new(session_path, OutputFormat::Json);
    let result = cmd.run().into_result().expect("analyze json");

    assert_eq!(result.event_count, 3);
    assert_eq!(result.decision_breakdown.pass_through, 3);
}

// =============================================================================
// Full Feature Chain Integration
// =============================================================================

/// Test the complete workflow: write script → test → record → replay → analyze.
#[tokio::test]
async fn full_feature_chain_integration() {
    let temp_dir = TempDir::new().expect("create temp dir");

    // Step 1: Write and test a Rhai script
    let script_path = temp_dir.path().join("integration.rhai");
    let script_content = r#"
        fn test_remap_definition() {
            // Test that we can define remaps
            let x = "CapsLock";
            let y = "Escape";
            if x == y {
                throw "Keys should be different";
            }
        }
    "#;
    fs::write(&script_path, script_content).expect("write script");

    let cmd = TestCommand::new(script_path.clone(), OutputFormat::Human);
    let test_exit = cmd.run().expect("run tests");
    assert_eq!(test_exit, 0, "Script tests should pass");

    // Step 2: Create a recording session
    let session_path = temp_dir.path().join("integration.krx");
    let mut recorder = EventRecorder::new(
        &session_path,
        Some(script_path.to_str().unwrap().to_string()),
        TimingConfig::default(),
        make_initial_state(),
    )
    .expect("create recorder");

    // Simulate a user session
    let user_events = [
        (KeyCode::CapsLock, true, 0u64),
        (KeyCode::CapsLock, false, 50_000),
        (KeyCode::A, true, 100_000),
        (KeyCode::A, false, 150_000),
    ];

    for (seq, (key, pressed, ts)) in user_events.iter().enumerate() {
        let input = if *pressed {
            InputEvent::key_down(*key, *ts)
        } else {
            InputEvent::key_up(*key, *ts)
        };

        let output = if *key == KeyCode::CapsLock {
            if *pressed {
                vec![OutputAction::KeyDown(KeyCode::Escape)]
            } else {
                vec![OutputAction::KeyUp(KeyCode::Escape)]
            }
        } else if *pressed {
            vec![OutputAction::KeyDown(*key)]
        } else {
            vec![OutputAction::KeyUp(*key)]
        };

        let decision = if *key == KeyCode::CapsLock {
            DecisionType::Remap
        } else {
            DecisionType::PassThrough
        };

        recorder.record_event(
            EventRecordBuilder::new()
                .seq(seq as u64)
                .timestamp_us(*ts)
                .input(input)
                .output(output)
                .decision_type(decision)
                .active_layers(vec![0])
                .modifiers_state(ModifierState::default())
                .latency_us(30)
                .build(),
        );
    }

    let session = recorder.finish().expect("finish recording");
    assert_eq!(session.event_count(), 4);

    // Step 3: Replay and verify
    let replay_cmd =
        ReplayCommand::new(session_path.clone(), OutputFormat::Human).with_verify(true);

    let replay_result = replay_cmd.run().await.expect("replay");
    assert_eq!(replay_result.total_events, 4);

    // Step 4: Analyze the session
    let analyze_cmd = AnalyzeCommand::new(session_path, OutputFormat::Human).with_diagram(true);

    let analysis = analyze_cmd.run().into_result().expect("analyze");
    assert_eq!(analysis.event_count, 4);
    assert_eq!(analysis.decision_breakdown.remap, 2); // CapsLock down + up
    assert_eq!(analysis.decision_breakdown.pass_through, 2); // A down + up
    assert_eq!(analysis.avg_latency_us, 30);
}

/// Test error handling across the feature chain.
#[test]
fn error_handling_chain() {
    // Test missing file
    let cmd = TestCommand::new("/nonexistent/script.rhai".into(), OutputFormat::Human);
    let result = cmd.run();
    assert!(result.is_err(), "Should fail for missing file");

    // Test invalid JSON session
    let temp_dir = TempDir::new().expect("create temp dir");
    let bad_session = temp_dir.path().join("bad.krx");
    fs::write(&bad_session, "{ invalid json }").expect("write bad json");

    let analyze_cmd = AnalyzeCommand::new(bad_session, OutputFormat::Human);
    let result = analyze_cmd.run();
    assert!(result.is_failure(), "Should fail for invalid JSON");
}

/// Test empty session handling.
#[tokio::test]
async fn empty_session_chain() {
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
    assert!(events.is_empty());

    // Analyze empty session
    let analyze_cmd = AnalyzeCommand::new(session_path, OutputFormat::Human);
    let result = analyze_cmd.run().into_result().expect("analyze empty");
    assert_eq!(result.event_count, 0);
}

/// Test test filtering functionality.
#[test]
fn test_filtering_integration() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let script_path = temp_dir.path().join("filtered.rhai");

    let script_content = r#"
        fn test_capslock_basic() { }
        fn test_capslock_advanced() { }
        fn test_layer_push() { }
        fn test_layer_pop() { }
        fn test_modifier_shift() { }
    "#;
    fs::write(&script_path, script_content).expect("write");

    // Test with capslock filter
    let cmd = TestCommand::new(script_path.clone(), OutputFormat::Human)
        .with_filter(Some("test_capslock*".to_string()));
    let result = cmd.run().expect("run filtered");
    assert_eq!(result, 0);

    // Test with layer filter
    let cmd2 = TestCommand::new(script_path, OutputFormat::Human)
        .with_filter(Some("test_layer*".to_string()));
    let result2 = cmd2.run().expect("run filtered 2");
    assert_eq!(result2, 0);
}

/// Test session file version compatibility.
#[test]
fn session_version_compatibility() {
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

    // Verify version is preserved
    let loaded = SessionFile::from_json(&json).expect("parse");
    assert_eq!(loaded.version, session.version);
}
