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
//! Tests for RunCommand.
//!
//! Comprehensive tests for the run command including constructor,
//! mock execution, and integration with Engine.

use keyrx_core::cli::commands::RunCommand;
use keyrx_core::cli::OutputFormat;
use std::path::PathBuf;

/// Test RunCommand::new() constructor with all parameters.
#[test]
fn run_command_new_creates_instance() {
    let script_path = Some(PathBuf::from("test.rhai"));
    let debug = true;
    let use_mock = true;
    let device_path = Some(PathBuf::from("/dev/input/event0"));
    let format = OutputFormat::Json;

    let cmd = RunCommand::new(
        script_path.clone(),
        debug,
        use_mock,
        device_path.clone(),
        format,
    );

    assert_eq!(cmd.script_path, script_path);
    assert_eq!(cmd.debug, debug);
    assert_eq!(cmd.use_mock, use_mock);
    assert_eq!(cmd.device_path, device_path);
    assert!(matches!(cmd.output.format(), OutputFormat::Json));
}

/// Test RunCommand::new() with None values.
#[test]
fn run_command_new_with_none_values() {
    let cmd = RunCommand::new(None, false, false, None, OutputFormat::Human);

    assert!(cmd.script_path.is_none());
    assert!(!cmd.debug);
    assert!(!cmd.use_mock);
    assert!(cmd.device_path.is_none());
    assert!(matches!(cmd.output.format(), OutputFormat::Human));
}

/// Test RunCommand::new() with debug mode enabled.
#[test]
fn run_command_new_debug_mode() {
    let cmd = RunCommand::new(None, true, false, None, OutputFormat::Human);

    assert!(cmd.debug);
}

/// Test RunCommand::new() with mock mode enabled.
#[test]
fn run_command_new_mock_mode() {
    let cmd = RunCommand::new(None, false, true, None, OutputFormat::Human);

    assert!(cmd.use_mock);
}

/// Test RunCommand::new() with custom device path.
#[test]
fn run_command_new_custom_device() {
    let device_path = Some(PathBuf::from("/dev/input/event5"));
    let cmd = RunCommand::new(None, false, false, device_path.clone(), OutputFormat::Human);

    assert_eq!(cmd.device_path, device_path);
}

/// Test RunCommand::new() with JSON output format.
#[test]
fn run_command_new_json_format() {
    let cmd = RunCommand::new(None, false, false, None, OutputFormat::Json);

    assert!(matches!(cmd.output.format(), OutputFormat::Json));
}

/// Test RunCommand::new() with Human output format (default).
#[test]
fn run_command_new_human_format() {
    let cmd = RunCommand::new(None, false, false, None, OutputFormat::Human);

    assert!(matches!(cmd.output.format(), OutputFormat::Human));
}

/// Test RunCommand with all fields set.
#[test]
fn run_command_new_all_fields() {
    let script_path = Some(PathBuf::from("/path/to/script.rhai"));
    let device_path = Some(PathBuf::from("/dev/input/event3"));

    let cmd = RunCommand::new(
        script_path.clone(),
        true,
        true,
        device_path.clone(),
        OutputFormat::Json,
    );

    assert_eq!(cmd.script_path, script_path);
    assert!(cmd.debug);
    assert!(cmd.use_mock);
    assert_eq!(cmd.device_path, device_path);
    assert!(matches!(cmd.output.format(), OutputFormat::Json));
}

/// Test that RunCommand fields are accessible.
#[test]
fn run_command_fields_accessible() {
    let cmd = RunCommand::new(
        Some(PathBuf::from("test.rhai")),
        true,
        false,
        Some(PathBuf::from("/dev/input/event0")),
        OutputFormat::Human,
    );

    // Verify we can access all fields
    let _ = &cmd.script_path;
    let _ = cmd.debug;
    let _ = cmd.use_mock;
    let _ = &cmd.device_path;
    let _ = &cmd.output;
}

/// Test RunCommand with various path formats.
#[test]
fn run_command_path_formats() {
    // Absolute path
    let cmd1 = RunCommand::new(
        Some(PathBuf::from("/absolute/path/script.rhai")),
        false,
        false,
        None,
        OutputFormat::Human,
    );
    assert_eq!(
        cmd1.script_path,
        Some(PathBuf::from("/absolute/path/script.rhai"))
    );

    // Relative path
    let cmd2 = RunCommand::new(
        Some(PathBuf::from("./relative/script.rhai")),
        false,
        false,
        None,
        OutputFormat::Human,
    );
    assert_eq!(
        cmd2.script_path,
        Some(PathBuf::from("./relative/script.rhai"))
    );

    // File in current directory
    let cmd3 = RunCommand::new(
        Some(PathBuf::from("script.rhai")),
        false,
        false,
        None,
        OutputFormat::Human,
    );
    assert_eq!(cmd3.script_path, Some(PathBuf::from("script.rhai")));
}

/// Test RunCommand constructor with combinations of debug and mock.
#[test]
fn run_command_debug_and_mock_combinations() {
    // Both false
    let cmd1 = RunCommand::new(None, false, false, None, OutputFormat::Human);
    assert!(!cmd1.debug && !cmd1.use_mock);

    // Debug only
    let cmd2 = RunCommand::new(None, true, false, None, OutputFormat::Human);
    assert!(cmd2.debug && !cmd2.use_mock);

    // Mock only
    let cmd3 = RunCommand::new(None, false, true, None, OutputFormat::Human);
    assert!(!cmd3.debug && cmd3.use_mock);

    // Both true
    let cmd4 = RunCommand::new(None, true, true, None, OutputFormat::Human);
    assert!(cmd4.debug && cmd4.use_mock);
}

/// Test OutputWriter is properly initialized with Human format.
#[test]
fn run_command_output_writer_human() {
    let cmd = RunCommand::new(None, false, false, None, OutputFormat::Human);
    assert!(matches!(cmd.output.format(), OutputFormat::Human));
}

/// Test OutputWriter is properly initialized with JSON format.
#[test]
fn run_command_output_writer_json() {
    let cmd = RunCommand::new(None, false, false, None, OutputFormat::Json);
    assert!(matches!(cmd.output.format(), OutputFormat::Json));
}
