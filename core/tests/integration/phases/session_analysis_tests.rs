#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::print_stdout, clippy::print_stderr, clippy::field_reassign_with_default, clippy::useless_conversion, clippy::assertions_on_constants, clippy::manual_div_ceil, clippy::manual_strip, clippy::len_zero, clippy::redundant_closure, clippy::manual_range_contains, clippy::default_constructed_unit_structs, clippy::clone_on_copy, clippy::io_other_error, clippy::bool_assert_comparison, clippy::approx_constant, clippy::let_unit_value, clippy::while_let_on_iterator, clippy::await_holding_lock, clippy::unnecessary_cast, clippy::drop_non_drop, clippy::needless_range_loop, unused_imports, unused_variables, dead_code, unsafe_code, clippy::collapsible_if, clippy::bool_comparison, unexpected_cfgs)]
//! Session Analysis Tests
//!
//! Tests for the analyze command and session analysis functionality.
//! Covers timing statistics, decision breakdowns, and output formatting.

use keyrx_core::cli::commands::AnalyzeCommand;
use keyrx_core::cli::OutputFormat;
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

/// Test empty session analysis.
#[test]
fn analyze_empty_session() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let session_path = temp_dir.path().join("empty_analyze.krx");

    // Create empty session
    let recorder = EventRecorder::new(
        &session_path,
        None,
        TimingConfig::default(),
        make_initial_state(),
    )
    .expect("create recorder");

    recorder.finish().expect("finish empty");

    // Analyze empty session
    let analyze_cmd = AnalyzeCommand::new(session_path, OutputFormat::Human);
    let result = analyze_cmd.run().into_result().expect("analyze empty");
    assert_eq!(result.event_count, 0);
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
