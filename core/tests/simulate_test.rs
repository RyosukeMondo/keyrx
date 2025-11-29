//! Simulate command unit tests.
//!
//! Tests for input parsing, script execution, and output format verification.

use keyrx_core::cli::commands::SimulateCommand;
use keyrx_core::cli::OutputFormat;
use keyrx_core::engine::KeyCode;
use std::io::Write;
use tempfile::NamedTempFile;

/// Test parsing single key input.
#[test]
fn parse_single_key() {
    let cmd = SimulateCommand::new("A".to_string(), None, OutputFormat::Human);
    let events = cmd.parse_input().unwrap();

    // Single key generates key-down and key-up
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].key, KeyCode::A);
    assert!(events[0].pressed);
    assert_eq!(events[1].key, KeyCode::A);
    assert!(!events[1].pressed);
}

/// Test parsing multiple keys.
#[test]
fn parse_multiple_keys() {
    let cmd = SimulateCommand::new("A,B,C".to_string(), None, OutputFormat::Human);
    let events = cmd.parse_input().unwrap();

    // 3 keys = 6 events (3 key-down + 3 key-up)
    assert_eq!(events.len(), 6);

    // Verify key order: A down, A up, B down, B up, C down, C up
    assert_eq!(events[0].key, KeyCode::A);
    assert!(events[0].pressed);
    assert_eq!(events[1].key, KeyCode::A);
    assert!(!events[1].pressed);

    assert_eq!(events[2].key, KeyCode::B);
    assert!(events[2].pressed);
    assert_eq!(events[3].key, KeyCode::B);
    assert!(!events[3].pressed);

    assert_eq!(events[4].key, KeyCode::C);
    assert!(events[4].pressed);
    assert_eq!(events[5].key, KeyCode::C);
    assert!(!events[5].pressed);
}

/// Test parsing handles whitespace around keys.
#[test]
fn parse_keys_with_whitespace() {
    let cmd = SimulateCommand::new(" A , B , C ".to_string(), None, OutputFormat::Human);
    let events = cmd.parse_input().unwrap();

    assert_eq!(events.len(), 6);
    assert_eq!(events[0].key, KeyCode::A);
    assert_eq!(events[2].key, KeyCode::B);
    assert_eq!(events[4].key, KeyCode::C);
}

/// Test parsing special keys.
#[test]
fn parse_special_keys() {
    let cmd = SimulateCommand::new("Enter,Escape,CapsLock".to_string(), None, OutputFormat::Human);
    let events = cmd.parse_input().unwrap();

    assert_eq!(events.len(), 6);
    assert_eq!(events[0].key, KeyCode::Enter);
    assert_eq!(events[2].key, KeyCode::Escape);
    assert_eq!(events[4].key, KeyCode::CapsLock);
}

/// Test parsing function keys.
#[test]
fn parse_function_keys() {
    let cmd = SimulateCommand::new("F1,F12".to_string(), None, OutputFormat::Human);
    let events = cmd.parse_input().unwrap();

    assert_eq!(events.len(), 4);
    assert_eq!(events[0].key, KeyCode::F1);
    assert_eq!(events[2].key, KeyCode::F12);
}

/// Test parsing modifier keys.
#[test]
fn parse_modifier_keys() {
    let cmd =
        SimulateCommand::new("LeftCtrl,LeftAlt,LeftShift".to_string(), None, OutputFormat::Human);
    let events = cmd.parse_input().unwrap();

    assert_eq!(events.len(), 6);
    assert_eq!(events[0].key, KeyCode::LeftCtrl);
    assert_eq!(events[2].key, KeyCode::LeftAlt);
    assert_eq!(events[4].key, KeyCode::LeftShift);
}

/// Test parsing unknown key returns error.
#[test]
fn parse_unknown_key_returns_error() {
    let cmd = SimulateCommand::new("InvalidKey".to_string(), None, OutputFormat::Human);
    let result = cmd.parse_input();

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Unknown key"));
    assert!(err.contains("InvalidKey"));
}

/// Test parsing empty input returns empty vec.
#[test]
fn parse_empty_input() {
    let cmd = SimulateCommand::new("".to_string(), None, OutputFormat::Human);
    let events = cmd.parse_input().unwrap();

    assert!(events.is_empty());
}

/// Test parsing input with only commas returns empty vec.
#[test]
fn parse_only_commas() {
    let cmd = SimulateCommand::new(",,,".to_string(), None, OutputFormat::Human);
    let events = cmd.parse_input().unwrap();

    assert!(events.is_empty());
}

/// Test event timestamps increment correctly.
#[test]
fn parse_timestamps_increment() {
    let cmd = SimulateCommand::new("A,B".to_string(), None, OutputFormat::Human);
    let events = cmd.parse_input().unwrap();

    // Timestamps should increment by 1000 (1ms) between events
    assert_eq!(events[0].timestamp, 0);
    assert_eq!(events[1].timestamp, 1000);
    assert_eq!(events[2].timestamp, 2000);
    assert_eq!(events[3].timestamp, 3000);
}

/// Test simulate without script passes all keys through.
#[tokio::test]
async fn simulate_without_script_passes_through() {
    let cmd = SimulateCommand::new("A,B".to_string(), None, OutputFormat::Human);
    let output = cmd.execute().await.unwrap();

    // All keys should pass through
    assert_eq!(output.total, 4); // 2 keys * 2 events each
    assert_eq!(output.passed, 4);
    assert_eq!(output.remapped, 0);
    assert_eq!(output.blocked, 0);
}

/// Test simulate with remap script.
#[tokio::test]
async fn simulate_with_remap_script() {
    // Create temp script file
    let mut script_file = NamedTempFile::new().unwrap();
    writeln!(script_file, "remap(\"A\", \"B\");").unwrap();

    let cmd = SimulateCommand::new(
        "A".to_string(),
        Some(script_file.path().to_path_buf()),
        OutputFormat::Human,
    );
    let output = cmd.execute().await.unwrap();

    // A should be remapped to B
    assert_eq!(output.total, 2);
    assert_eq!(output.remapped, 2); // Both key-down and key-up
    assert_eq!(output.results[0].input, "A");
    assert_eq!(output.results[0].output, "B");
}

/// Test simulate with block script.
#[tokio::test]
async fn simulate_with_block_script() {
    let mut script_file = NamedTempFile::new().unwrap();
    writeln!(script_file, "block(\"CapsLock\");").unwrap();

    let cmd = SimulateCommand::new(
        "CapsLock".to_string(),
        Some(script_file.path().to_path_buf()),
        OutputFormat::Human,
    );
    let output = cmd.execute().await.unwrap();

    assert_eq!(output.total, 2);
    assert_eq!(output.blocked, 2);
    assert_eq!(output.results[0].output, "BLOCKED");
}

/// Test simulate with mixed script (remap, block, pass).
#[tokio::test]
async fn simulate_with_mixed_script() {
    let mut script_file = NamedTempFile::new().unwrap();
    writeln!(script_file, "remap(\"A\", \"B\");").unwrap();
    writeln!(script_file, "block(\"CapsLock\");").unwrap();
    writeln!(script_file, "pass(\"Enter\");").unwrap();

    let cmd = SimulateCommand::new(
        "A,CapsLock,Enter".to_string(),
        Some(script_file.path().to_path_buf()),
        OutputFormat::Human,
    );
    let output = cmd.execute().await.unwrap();

    assert_eq!(output.total, 6);
    assert_eq!(output.remapped, 2); // A -> B (down + up)
    assert_eq!(output.blocked, 2); // CapsLock (down + up)
    assert_eq!(output.passed, 2); // Enter (down + up)
}

/// Test simulate with on_init hook.
#[tokio::test]
async fn simulate_with_on_init_hook() {
    let mut script_file = NamedTempFile::new().unwrap();
    writeln!(script_file, "fn on_init() {{").unwrap();
    writeln!(script_file, "    remap(\"A\", \"Z\");").unwrap();
    writeln!(script_file, "}}").unwrap();

    let cmd = SimulateCommand::new(
        "A".to_string(),
        Some(script_file.path().to_path_buf()),
        OutputFormat::Human,
    );
    let output = cmd.execute().await.unwrap();

    // on_init should have set up remap
    assert_eq!(output.remapped, 2);
    assert_eq!(output.results[0].output, "Z");
}

/// Test simulate empty input returns error.
#[tokio::test]
async fn simulate_empty_input_returns_error() {
    let cmd = SimulateCommand::new("".to_string(), None, OutputFormat::Human);
    let result = cmd.execute().await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("No valid input keys"));
}

/// Test simulation output structure is correct.
#[tokio::test]
async fn simulation_output_structure() {
    let mut script_file = NamedTempFile::new().unwrap();
    writeln!(script_file, "remap(\"A\", \"B\");").unwrap();

    let cmd = SimulateCommand::new(
        "A".to_string(),
        Some(script_file.path().to_path_buf()),
        OutputFormat::Human,
    );
    let output = cmd.execute().await.unwrap();

    // Check results structure
    assert_eq!(output.results.len(), 2);

    // Key-down event
    assert_eq!(output.results[0].input, "A");
    assert_eq!(output.results[0].output, "B");
    assert!(output.results[0].pressed);

    // Key-up event
    assert_eq!(output.results[1].input, "A");
    assert_eq!(output.results[1].output, "B");
    assert!(!output.results[1].pressed);
}

/// Test JSON output format produces valid JSON.
#[tokio::test]
async fn json_output_format() {
    let cmd = SimulateCommand::new("A".to_string(), None, OutputFormat::Json);
    let output = cmd.execute().await.unwrap();

    // Verify output serializes to JSON correctly
    let json = serde_json::to_string(&output).unwrap();
    assert!(json.contains("\"results\""));
    assert!(json.contains("\"total\""));
    assert!(json.contains("\"remapped\""));
    assert!(json.contains("\"blocked\""));
    assert!(json.contains("\"passed\""));

    // Verify it can be parsed back
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["total"], 2);
}

/// Test JSON output with script produces correct structure.
#[tokio::test]
async fn json_output_with_script() {
    let mut script_file = NamedTempFile::new().unwrap();
    writeln!(script_file, "remap(\"A\", \"B\");").unwrap();
    writeln!(script_file, "block(\"CapsLock\");").unwrap();

    let cmd = SimulateCommand::new(
        "A,CapsLock,Enter".to_string(),
        Some(script_file.path().to_path_buf()),
        OutputFormat::Json,
    );
    let output = cmd.execute().await.unwrap();

    let json = serde_json::to_string(&output).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["total"], 6);
    assert_eq!(parsed["remapped"], 2);
    assert_eq!(parsed["blocked"], 2);
    assert_eq!(parsed["passed"], 2);

    // Verify results array structure
    let results = parsed["results"].as_array().unwrap();
    assert_eq!(results.len(), 6);
    assert_eq!(results[0]["input"], "A");
    assert_eq!(results[0]["output"], "B");
    assert_eq!(results[0]["pressed"], true);
}

/// Test invalid script path returns error.
#[tokio::test]
async fn invalid_script_path_returns_error() {
    let cmd = SimulateCommand::new(
        "A".to_string(),
        Some("/nonexistent/path/script.rhai".into()),
        OutputFormat::Human,
    );
    let result = cmd.execute().await;

    assert!(result.is_err());
}

/// Test script syntax error returns error.
#[tokio::test]
async fn script_syntax_error_returns_error() {
    let mut script_file = NamedTempFile::new().unwrap();
    writeln!(script_file, "remap(\"A\"").unwrap(); // Missing closing paren and arg

    let cmd = SimulateCommand::new(
        "A".to_string(),
        Some(script_file.path().to_path_buf()),
        OutputFormat::Human,
    );
    let result = cmd.execute().await;

    assert!(result.is_err());
}
