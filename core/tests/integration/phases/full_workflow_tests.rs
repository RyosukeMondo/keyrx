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
//! Full Workflow Integration Tests
//!
//! End-to-end tests verifying the complete feature chain:
//! - Script testing → test discovery → test runner
//! - Session recording → replay → analysis
//! - Commands work together in realistic workflows

use keyrx_core::cli::commands::{AnalyzeCommand, ReplayCommand, TestCommand};
use keyrx_core::cli::OutputFormat;
use keyrx_core::engine::{
    DecisionType, EventRecordBuilder, EventRecorder, InputEvent, KeyCode, ModifierState,
    OutputAction, StateSnapshot, TimingConfig,
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

/// Test test filtering functionality across the workflow.
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
