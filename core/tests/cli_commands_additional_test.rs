//! Additional CLI command tests for check, state, and devices commands.

use keyrx_core::cli::commands::{CheckCommand, DevicesCommand, StateCommand};
use keyrx_core::cli::OutputFormat;
use keyrx_core::drivers::DeviceInfo;
use keyrx_core::error::KeyRxError;
use std::io::Write;
use std::path::PathBuf;
use tempfile::{tempdir, NamedTempFile};

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

    let cmd = CheckCommand::new(file.path().to_path_buf(), OutputFormat::Human);
    cmd.run().expect("valid script should compile");
}

#[test]
fn check_command_runs_successfully_with_json_output() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "let message = \"hello\";").unwrap();

    let cmd = CheckCommand::new(file.path().to_path_buf(), OutputFormat::Json);
    cmd.run().expect("valid script should compile");
}

#[test]
fn check_command_returns_compile_error_for_invalid_script() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "let = ;").unwrap();

    let cmd = CheckCommand::new(file.path().to_path_buf(), OutputFormat::Human);
    let err = cmd.run().expect_err("invalid script should fail");
    let compile_error = err
        .downcast_ref::<KeyRxError>()
        .expect("expected KeyRxError");
    assert!(matches!(
        compile_error,
        KeyRxError::ScriptCompileError { .. }
    ));
}

#[test]
fn check_command_propagates_io_errors_for_missing_file() {
    let temp_dir = tempdir().unwrap();
    let missing_path = temp_dir.path().join("missing.rhai");
    let cmd = CheckCommand::new(missing_path, OutputFormat::Human);

    let err = cmd.run().expect_err("missing file should produce IO error");
    let io_error = err
        .downcast_ref::<std::io::Error>()
        .expect("expected io::Error");
    assert_eq!(io_error.kind(), std::io::ErrorKind::NotFound);
}

#[test]
fn state_command_new_sets_flags_and_format() {
    let cmd = StateCommand::new(true, false, OutputFormat::Json);
    assert!(cmd.show_layers);
    assert!(!cmd.show_modifiers);
    assert!(matches!(cmd.output.format(), OutputFormat::Json));
}

#[test]
fn state_command_runs_with_human_output() {
    let cmd = StateCommand::new(true, true, OutputFormat::Human);
    cmd.run().expect("state command should succeed");
}

#[test]
fn state_command_runs_with_json_output() {
    let cmd = StateCommand::new(false, true, OutputFormat::Json);
    cmd.run().expect("state command should succeed");
}

fn sample_devices() -> Vec<DeviceInfo> {
    vec![
        DeviceInfo::new(
            PathBuf::from("/dev/input/event2"),
            "Keyboard X".to_string(),
            0xAAAA,
            0xBBBB,
            true,
        ),
        DeviceInfo::new(
            PathBuf::from("/dev/input/event3"),
            "Keyboard Y".to_string(),
            0xCCCC,
            0xDDDD,
            true,
        ),
    ]
}

#[test]
fn devices_command_new_sets_format() {
    let cmd = DevicesCommand::new(OutputFormat::Json);
    assert!(matches!(cmd.output.format(), OutputFormat::Json));
}

#[test]
fn devices_command_runs_with_injected_devices_human() {
    let cmd = DevicesCommand::with_provider(OutputFormat::Human, || Ok(sample_devices()));
    cmd.run().expect("rendering devices should succeed");
}

#[test]
fn devices_command_runs_with_injected_devices_json() {
    let cmd = DevicesCommand::with_provider(OutputFormat::Json, || Ok(sample_devices()));
    cmd.run().expect("rendering devices should succeed");
}

#[test]
fn devices_command_handles_empty_lists_in_json() {
    let cmd = DevicesCommand::with_provider(OutputFormat::Json, || Ok(vec![]));
    cmd.run().expect("empty list should be handled");
}

#[test]
fn devices_command_propagates_errors() {
    let cmd = DevicesCommand::with_provider(OutputFormat::Human, || {
        anyhow::bail!("device enumeration failed")
    });

    let err = cmd.run().expect_err("errors should propagate");
    assert!(err.to_string().contains("device enumeration failed"));
}
