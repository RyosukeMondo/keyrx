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
//! Additional CLI command tests for check, state, and devices commands.

use keyrx_core::cli::commands::{CheckCommand, DeviceAction, DevicesCommand, StateCommand};
use keyrx_core::cli::{Command, CommandContext, OutputFormat, Verbosity};
use std::io::Write;
use std::path::PathBuf;
use tempfile::{tempdir, NamedTempFile};

fn create_ctx(format: OutputFormat) -> CommandContext {
    CommandContext::new(format, Verbosity::Normal)
}

#[test]
fn check_command_new_sets_fields() {
    let script_path = PathBuf::from("script.rhai");
    let cmd = CheckCommand::new(script_path.clone(), OutputFormat::Json);

    assert_eq!(cmd.script_path, script_path);
    assert!(matches!(cmd.output.format(), OutputFormat::Json));
}

#[test]
fn check_command_runs_successfully_with_human_output() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "let x = 1 + 2;").unwrap();

    let mut cmd = CheckCommand::new(file.path().to_path_buf(), OutputFormat::Human);
    let result = cmd.execute(&create_ctx(OutputFormat::Human));
    assert!(result.is_success(), "valid script should compile");
}

#[test]
fn check_command_runs_successfully_with_json_output() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "let message = \"hello\";").unwrap();

    let mut cmd = CheckCommand::new(file.path().to_path_buf(), OutputFormat::Json);
    let result = cmd.execute(&create_ctx(OutputFormat::Json));
    assert!(result.is_success(), "valid script should compile");
}

#[test]
fn check_command_returns_error_exit_code_for_invalid_script() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "let = ;").unwrap();

    let mut cmd = CheckCommand::new(file.path().to_path_buf(), OutputFormat::Human);
    let result = cmd.execute(&create_ctx(OutputFormat::Human));
    // Command execution succeeds but returns a failure exit code
    assert!(result.is_failure(), "invalid script should return failure");
}

#[test]
fn check_command_propagates_io_errors_for_missing_file() {
    let temp_dir = tempdir().unwrap();
    let missing_path = temp_dir.path().join("missing.rhai");
    let mut cmd = CheckCommand::new(missing_path, OutputFormat::Human);

    let result = cmd.execute(&create_ctx(OutputFormat::Human));
    // Missing file should result in failure
    assert!(result.is_failure(), "missing file should produce error");
    assert!(!result.messages().is_empty(), "should have error messages");
}

#[test]
fn state_command_new_sets_flags_and_format() {
    let cmd = StateCommand::new(true, false, true, None, OutputFormat::Json);
    assert!(cmd.show_layers);
    assert!(!cmd.show_modifiers);
    assert!(cmd.show_pending);
    assert!(matches!(cmd.output.format(), OutputFormat::Json));
}

#[test]
fn state_command_runs_with_human_output() {
    let cmd = StateCommand::new(true, true, false, None, OutputFormat::Human);
    let result = cmd.run();
    assert!(result.is_success(), "state command should succeed");
}

#[test]
fn state_command_runs_with_json_output() {
    let cmd = StateCommand::new(false, true, true, None, OutputFormat::Json);
    let result = cmd.run();
    assert!(result.is_success(), "state command should succeed");
}

#[test]
fn state_command_collects_default_snapshot() {
    let cmd = StateCommand::new(true, true, true, None, OutputFormat::Human);
    let state = cmd
        .collect_state()
        .expect("state snapshot should be available");

    // Base layer is always present and active; pending decisions are empty by default.
    assert!(
        state.active_layers.contains(&state.base_layer),
        "base layer should be active"
    );
    assert_eq!(state.pending_count, 0, "no pending decisions by default");
}

#[test]
fn devices_command_new_sets_format() {
    let temp_dir = tempdir().unwrap();
    let cmd = DevicesCommand::new(OutputFormat::Json, DeviceAction::List)
        .with_bindings_path(temp_dir.path().join("bindings.json"));
    assert!(matches!(cmd.output.format(), OutputFormat::Json));
}

#[test]
fn devices_command_lists_empty_bindings() {
    let temp_dir = tempdir().unwrap();
    let cmd = DevicesCommand::new(OutputFormat::Human, DeviceAction::List)
        .with_bindings_path(temp_dir.path().join("bindings.json"));
    let result = cmd.run();
    assert!(
        result.is_success(),
        "listing with no bindings should succeed"
    );
}

#[test]
fn devices_command_handles_json_output() {
    let temp_dir = tempdir().unwrap();
    let cmd = DevicesCommand::new(OutputFormat::Json, DeviceAction::List)
        .with_bindings_path(temp_dir.path().join("bindings.json"));
    let result = cmd.run();
    assert!(
        result.is_success(),
        "json listing with no bindings should succeed"
    );
}

#[test]
fn devices_command_reports_load_errors() {
    let temp_dir = tempdir().unwrap();
    // Point bindings path to a directory to force a load error.
    let cmd = DevicesCommand::new(OutputFormat::Human, DeviceAction::List)
        .with_bindings_path(temp_dir.path().to_path_buf());

    let result = cmd.run();
    assert!(result.is_failure(), "should surface binding load errors");
    // Depending on the OS and filesystem, reading a directory as a file might fail with different messages.
    // We just check that some error message is present.
    assert!(
        !result.messages().is_empty(),
        "error message should be present"
    );
}
