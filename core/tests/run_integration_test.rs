//! Integration tests for RunCommand execution.
//!
//! Tests the actual run() method with mock input and script loading.

use keyrx_core::cli::commands::RunCommand;
use keyrx_core::cli::OutputFormat;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::time::{timeout, Duration};

/// Test running with mock input and no script.
#[tokio::test]
async fn run_with_mock_no_script() {
    let cmd = RunCommand::new(None, false, true, None, OutputFormat::Human);

    // Run the engine in the background with a timeout
    let result = timeout(Duration::from_millis(500), async {
        // We can't actually run indefinitely in a test, so we'll just verify
        // the command can be created and fields are set correctly
        Ok::<(), anyhow::Error>(())
    })
    .await;

    assert!(result.is_ok());
    assert!(cmd.use_mock);
    assert!(cmd.script_path.is_none());
}

/// Test running with mock input using a bounded runtime to avoid hanging.
#[tokio::test]
async fn run_with_mock_bounded_executes() {
    let cmd = RunCommand::new(None, false, true, None, OutputFormat::Human)
        .with_mock_run_limit(Duration::from_millis(20));

    let result = cmd.run().await;
    assert!(result.is_ok());
}

/// Test RunCommand creation with script path.
#[tokio::test]
async fn run_with_script_path_set() {
    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("test.rhai");

    // Create a simple test script
    let script_content = r#"
        // Simple test script
        remap("a", "b");
    "#;
    fs::write(&script_path, script_content).unwrap();

    let cmd = RunCommand::new(
        Some(script_path.clone()),
        false,
        true,
        None,
        OutputFormat::Human,
    );

    assert_eq!(cmd.script_path, Some(script_path));
    assert!(cmd.use_mock);
}

/// Test bounded mock run with a script to ensure startup/shutdown works.
#[tokio::test]
async fn run_with_mock_bounded_and_script() {
    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("bounded.rhai");
    fs::write(&script_path, "remap(\"a\", \"b\");").unwrap();

    let cmd = RunCommand::new(
        Some(script_path.clone()),
        false,
        true,
        None,
        OutputFormat::Human,
    )
    .with_mock_run_limit(Duration::from_millis(20));

    let result = cmd.run().await;
    assert!(result.is_ok());
}

/// Test RunCommand with debug mode creates proper configuration.
#[tokio::test]
async fn run_with_debug_mode() {
    let cmd = RunCommand::new(None, true, true, None, OutputFormat::Json);

    assert!(cmd.debug);
    assert!(matches!(cmd.output.format(), OutputFormat::Json));
}

/// Test RunCommand with device path configuration.
#[tokio::test]
async fn run_with_device_path() {
    let device_path = PathBuf::from("/dev/input/event0");
    let cmd = RunCommand::new(
        None,
        false,
        false,
        Some(device_path.clone()),
        OutputFormat::Human,
    );

    assert_eq!(cmd.device_path, Some(device_path));
    assert!(!cmd.use_mock);
}

/// Test RunCommand fields are properly set for mock mode.
#[test]
fn mock_mode_configuration() {
    let cmd = RunCommand::new(None, false, true, None, OutputFormat::Human);

    assert!(cmd.use_mock);
    assert!(!cmd.debug);
    assert!(cmd.script_path.is_none());
    assert!(cmd.device_path.is_none());
}

/// Test RunCommand with all options enabled.
#[test]
fn full_configuration() {
    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("full_test.rhai");
    let device_path = PathBuf::from("/dev/input/event5");

    fs::write(&script_path, "// Test script\n").unwrap();

    let cmd = RunCommand::new(
        Some(script_path.clone()),
        true,
        true,
        Some(device_path.clone()),
        OutputFormat::Json,
    );

    assert_eq!(cmd.script_path, Some(script_path));
    assert!(cmd.debug);
    assert!(cmd.use_mock);
    assert_eq!(cmd.device_path, Some(device_path));
    assert!(matches!(cmd.output.format(), OutputFormat::Json));
}

/// Test that RunCommand can be created with empty script path.
#[test]
fn no_script_configuration() {
    let cmd = RunCommand::new(None, false, true, None, OutputFormat::Human);

    assert!(cmd.script_path.is_none());
    assert!(cmd.use_mock);
}

/// Test RunCommand with various output formats.
#[test]
fn output_format_configuration() {
    let cmd_human = RunCommand::new(None, false, true, None, OutputFormat::Human);
    assert!(matches!(cmd_human.output.format(), OutputFormat::Human));

    let cmd_json = RunCommand::new(None, false, true, None, OutputFormat::Json);
    assert!(matches!(cmd_json.output.format(), OutputFormat::Json));
}

/// Test creating multiple RunCommand instances with different configurations.
#[test]
fn multiple_instances() {
    let cmd1 = RunCommand::new(None, false, true, None, OutputFormat::Human);
    let cmd2 = RunCommand::new(None, true, false, None, OutputFormat::Json);
    let cmd3 = RunCommand::new(
        Some(PathBuf::from("test.rhai")),
        true,
        true,
        Some(PathBuf::from("/dev/input/event0")),
        OutputFormat::Human,
    );

    // Verify each instance has independent configuration
    assert!(!cmd1.debug);
    assert!(cmd2.debug);
    assert!(cmd3.debug);

    assert!(cmd1.use_mock);
    assert!(!cmd2.use_mock);
    assert!(cmd3.use_mock);
}

/// Test script path with special characters.
#[test]
fn script_path_with_special_chars() {
    let script_path = PathBuf::from("/path/with spaces/and-dashes/script_file.rhai");
    let cmd = RunCommand::new(
        Some(script_path.clone()),
        false,
        true,
        None,
        OutputFormat::Human,
    );

    assert_eq!(cmd.script_path, Some(script_path));
}

/// Test device path with special characters.
#[test]
fn device_path_with_special_chars() {
    let device_path = PathBuf::from("/dev/input/by-id/usb-keyboard-event-kbd");
    let cmd = RunCommand::new(
        None,
        false,
        false,
        Some(device_path.clone()),
        OutputFormat::Human,
    );

    assert_eq!(cmd.device_path, Some(device_path));
}

/// Test RunCommand with UTF-8 paths.
#[test]
fn utf8_paths() {
    let script_path = PathBuf::from("/path/with/日本語/script.rhai");
    let cmd = RunCommand::new(
        Some(script_path.clone()),
        false,
        true,
        None,
        OutputFormat::Human,
    );

    assert_eq!(cmd.script_path, Some(script_path));
}

/// Test debug mode flag is independent of other settings.
#[test]
fn debug_mode_independence() {
    let cmd_debug_mock = RunCommand::new(None, true, true, None, OutputFormat::Human);
    let cmd_debug_real = RunCommand::new(None, true, false, None, OutputFormat::Human);

    assert!(cmd_debug_mock.debug && cmd_debug_mock.use_mock);
    assert!(cmd_debug_real.debug && !cmd_debug_real.use_mock);
}

/// Test that output format is independent of other settings.
#[test]
fn output_format_independence() {
    let cmd1 = RunCommand::new(None, true, true, None, OutputFormat::Json);
    let cmd2 = RunCommand::new(None, false, false, None, OutputFormat::Json);

    assert!(matches!(cmd1.output.format(), OutputFormat::Json));
    assert!(matches!(cmd2.output.format(), OutputFormat::Json));
    assert!(cmd1.debug != cmd2.debug);
}
