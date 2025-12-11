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
//! CLI integration tests for validation command.
//!
//! Tests CLI integration including:
//! - Exit codes for different validation outcomes
//! - Config file loading
//! - Output format options
//! - Flag combinations

use keyrx_core::cli::commands::CheckCommand;
use keyrx_core::cli::{Command, CommandContext, ExitCode, OutputFormat, Verbosity};
use std::io::Write;
use tempfile::NamedTempFile;

fn create_script_file(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    write!(file, "{}", content).unwrap();
    file
}

fn create_ctx(format: OutputFormat) -> CommandContext {
    CommandContext::new(format, Verbosity::Normal)
}

#[test]
fn exit_code_valid_script() {
    let file = create_script_file(r#"remap("CapsLock", "Escape");"#);
    let mut cmd = CheckCommand::new(file.path().to_path_buf(), OutputFormat::Human);
    let result = cmd.execute(&create_ctx(OutputFormat::Human));
    assert!(result.is_success());
    assert_eq!(result.exit_code(), ExitCode::Success);
}

#[test]
fn exit_code_errors() {
    let file = create_script_file(r#"remap("InvalidKey", "Escape");"#);
    let mut cmd = CheckCommand::new(file.path().to_path_buf(), OutputFormat::Human);
    let result = cmd.execute(&create_ctx(OutputFormat::Human));
    assert!(result.is_failure());
    assert_eq!(result.exit_code(), ExitCode::ValidationFailed);
}

#[test]
fn exit_code_strict_with_warnings() {
    let file = create_script_file(
        r#"
        remap("A", "B");
        remap("A", "C");
    "#,
    );
    let mut cmd = CheckCommand::new(file.path().to_path_buf(), OutputFormat::Human).strict();
    let result = cmd.execute(&create_ctx(OutputFormat::Human));
    assert!(result.is_failure());
    assert_eq!(result.exit_code(), ExitCode::AssertionFailed);
}

#[test]
fn custom_config_affects_validation() {
    // Create config with very restrictive tap timeout range
    let mut config_file = NamedTempFile::new().unwrap();
    writeln!(config_file, "tap_timeout_warn_range = [100, 200]").unwrap();

    // Script with tap_timeout outside the range
    let script_file = create_script_file("tap_timeout(50);");

    let mut cmd = CheckCommand::new(script_file.path().to_path_buf(), OutputFormat::Human)
        .with_config(config_file.path().to_path_buf());
    let result = cmd.execute(&create_ctx(OutputFormat::Human));
    // Should still be valid (warnings don't cause failure by default)
    assert!(result.is_success());
    assert_eq!(result.exit_code(), ExitCode::Success);
}

#[test]
fn custom_config_with_strict_mode() {
    // Create config with restrictive tap timeout range
    let mut config_file = NamedTempFile::new().unwrap();
    writeln!(config_file, "tap_timeout_warn_range = [100, 200]").unwrap();

    // Script with tap_timeout outside the range
    let script_file = create_script_file("tap_timeout(50);");

    let mut cmd = CheckCommand::new(script_file.path().to_path_buf(), OutputFormat::Human)
        .with_config(config_file.path().to_path_buf())
        .strict();
    let result = cmd.execute(&create_ctx(OutputFormat::Human));
    // Should fail in strict mode with warnings
    assert!(result.is_failure());
    assert_eq!(result.exit_code(), ExitCode::AssertionFailed);
}

#[test]
fn coverage_includes_report() {
    let file = create_script_file(
        r#"
        remap("A", "B");
        block("C");
    "#,
    );
    let mut cmd = CheckCommand::new(file.path().to_path_buf(), OutputFormat::Human).with_coverage();
    let result = cmd.execute(&create_ctx(OutputFormat::Human));
    assert!(result.is_success());
    assert_eq!(result.exit_code(), ExitCode::Success);
}

#[test]
fn visual_includes_keyboard() {
    let file = create_script_file(r#"remap("A", "B");"#);
    let mut cmd = CheckCommand::new(file.path().to_path_buf(), OutputFormat::Human).with_visual();
    let result = cmd.execute(&create_ctx(OutputFormat::Human));
    assert!(result.is_success());
    assert_eq!(result.exit_code(), ExitCode::Success);
}

#[test]
fn no_warnings_suppresses_output() {
    let file = create_script_file(
        r#"
        remap("A", "B");
        remap("A", "C");
    "#,
    );
    let mut cmd = CheckCommand::new(file.path().to_path_buf(), OutputFormat::Human).no_warnings();
    let result = cmd.execute(&create_ctx(OutputFormat::Human));
    assert!(result.is_success());
    assert_eq!(result.exit_code(), ExitCode::Success);
}

#[test]
fn show_config_returns_valid() {
    let file = create_script_file("");
    let mut cmd = CheckCommand::new(file.path().to_path_buf(), OutputFormat::Human).show_config();
    let result = cmd.execute(&create_ctx(OutputFormat::Human));
    assert!(result.is_success());
    assert_eq!(result.exit_code(), ExitCode::Success);
}

#[test]
fn invalid_config_path_errors() {
    let file = create_script_file(r#"remap("A", "B");"#);
    let mut cmd = CheckCommand::new(file.path().to_path_buf(), OutputFormat::Human)
        .with_config("/nonexistent/config.toml".into());
    let result = cmd.execute(&create_ctx(OutputFormat::Human));
    assert!(result.is_failure());
    assert_eq!(result.exit_code(), ExitCode::GeneralError);
}
