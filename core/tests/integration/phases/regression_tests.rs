#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::print_stdout, clippy::print_stderr, clippy::field_reassign_with_default, clippy::useless_conversion, clippy::assertions_on_constants, clippy::manual_div_ceil, clippy::manual_strip, clippy::len_zero, clippy::redundant_closure, clippy::manual_range_contains, clippy::default_constructed_unit_structs, clippy::clone_on_copy, clippy::io_other_error, clippy::bool_assert_comparison, clippy::approx_constant, clippy::let_unit_value, clippy::while_let_on_iterator, clippy::await_holding_lock, clippy::unnecessary_cast, clippy::drop_non_drop, clippy::needless_range_loop, unused_imports, unused_variables, dead_code, unsafe_code, clippy::collapsible_if, clippy::bool_comparison, unexpected_cfgs)]
//! Regression and Edge Case Tests
//!
//! Tests for error handling, edge cases, and regression prevention.
//! Ensures robust behavior across the feature chain.

use keyrx_core::cli::commands::AnalyzeCommand;
use keyrx_core::cli::OutputFormat;
use keyrx_core::engine::{EventRecorder, ReplaySession, StateSnapshot, TimingConfig};
use keyrx_core::traits::InputSource;
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

/// Test error handling for invalid session files.
#[test]
fn error_handling_invalid_session() {
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
    assert!(events.is_empty(), "Empty session should yield no events");
}
