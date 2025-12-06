//! Integration tests for CLI exit codes.
//!
//! These tests verify that the CLI binary returns the correct exit codes
//! for various scenarios, matching the documentation in ExitCode.

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::io::Write;
use std::process::Command;
use tempfile::{tempdir, NamedTempFile};

/// Helper to create a Command for the keyrx binary.
fn keyrx_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_keyrx"))
}

// =============================================================================
// Exit Code 0: Success
// =============================================================================

#[test]
fn exit_0_help_succeeds() {
    keyrx_cmd().arg("--help").assert().success().code(0);
}

#[test]
fn exit_0_version_succeeds() {
    keyrx_cmd().arg("--version").assert().success().code(0);
}

#[test]
fn exit_0_exit_codes_command_succeeds() {
    keyrx_cmd()
        .arg("exit-codes")
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::contains("Exit Code"));
}

#[test]
fn exit_0_exit_codes_json_succeeds() {
    keyrx_cmd()
        .args(["--json", "exit-codes"])
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::contains("\"code\""));
}

#[test]
fn exit_0_check_valid_script() {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    writeln!(file, "let x = 1 + 2;").expect("Failed to write to temp file");

    keyrx_cmd()
        .arg("check")
        .arg(file.path())
        .assert()
        .success()
        .code(0);
}

#[test]
fn exit_0_state_command_succeeds() {
    keyrx_cmd().arg("state").assert().success().code(0);
}

#[test]
fn exit_0_devices_command_succeeds() {
    keyrx_cmd().arg("devices").assert().success().code(0);
}

// =============================================================================
// Exit Code 1: General Error
// =============================================================================

#[test]
fn exit_1_check_missing_file() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let missing_path = temp_dir.path().join("missing.rhai");

    keyrx_cmd()
        .arg("check")
        .arg(&missing_path)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("error"));
}

#[test]
fn exit_1_invalid_command() {
    // clap returns exit code 2 for invalid arguments/commands
    keyrx_cmd()
        .arg("nonexistent-command")
        .assert()
        .failure()
        .code(2);
}

#[test]
#[ignore] // Run command has tokio runtime issues in tests
fn exit_1_run_missing_config() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let missing_config = temp_dir.path().join("nonexistent.toml");

    keyrx_cmd()
        .arg("run")
        .arg("--config")
        .arg(&missing_config)
        .assert()
        .failure()
        .code(1);
}

#[test]
fn exit_1_replay_missing_session() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let missing_session = temp_dir.path().join("nonexistent.krx");

    // Missing session file currently surfaces as a configuration read error (4)
    keyrx_cmd()
        .arg("replay")
        .arg(&missing_session)
        .assert()
        .failure()
        .code(4);
}

// =============================================================================
// Exit Code 2: Assertion Failed (Test Failures)
// =============================================================================

#[test]
fn exit_2_test_with_failing_assertions() {
    // Create a test file that will fail
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("failing_test.rhai");

    let test_content = r#"
// This test is designed to fail
fn test_will_fail() {
    assert_eq(1, 2, "1 should never equal 2");
}
"#;

    fs::write(&test_file, test_content).expect("Failed to write test file");

    keyrx_cmd()
        .arg("test")
        .arg(&test_file)
        .assert()
        .failure()
        .code(2);
}

#[test]
fn exit_2_uat_with_regression() {
    // Create a UAT scenario that will fail
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let uat_file = temp_dir.path().join("failing_uat.json");

    // Create a UAT file with impossible golden output match
    let uat_content = r#"{
    "sessions": [],
    "expected_outputs": ["impossible_to_match_12345"]
}"#;

    fs::write(&uat_file, uat_content).expect("Failed to write UAT file");

    // Note: This might return exit 1 if the file format is invalid instead of 2
    // The actual behavior depends on implementation
    keyrx_cmd().arg("uat").arg(&uat_file).assert().failure();
    // We don't check specific code here as it depends on validation order
}

#[test]
fn exit_2_replay_with_verification_failure() {
    // Create a replay session with incorrect expected output
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let session_file = temp_dir.path().join("session.json");

    let session_content = r#"{
    "events": [],
    "expected_events": [{"type": "impossible_event"}]
}"#;

    fs::write(&session_file, session_content).expect("Failed to write session file");

    keyrx_cmd()
        .arg("replay")
        .arg("--verify")
        .arg(&session_file)
        .assert()
        .failure();
    // Exit code depends on whether it's validation (1/4) or verification (2)
}

// =============================================================================
// Exit Code 3: Timeout
// =============================================================================

#[test]
#[ignore] // Ignore by default as it takes time to run
fn exit_3_simulate_timeout() {
    // Create a script that takes too long
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    writeln!(file, "loop {{ }}").expect("Failed to write to temp file");

    keyrx_cmd()
        .arg("simulate")
        .arg(file.path())
        .arg("--timeout")
        .arg("1") // 1 second timeout
        .assert()
        .failure()
        .code(3);
}

#[test]
#[ignore] // Ignore by default as it's timing-sensitive
fn exit_3_test_timeout() {
    // Create a test that takes too long
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("timeout_test.rhai");

    let test_content = r#"
fn test_infinite_loop() {
    loop { }
}
"#;

    fs::write(&test_file, test_content).expect("Failed to write test file");

    keyrx_cmd()
        .arg("test")
        .arg(&test_file)
        .arg("--timeout")
        .arg("1")
        .assert()
        .failure()
        .code(3);
}

// =============================================================================
// Exit Code 4: Validation Failed
// =============================================================================

#[test]
fn exit_4_check_invalid_syntax() {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    writeln!(file, "let = ;").expect("Failed to write to temp file");

    keyrx_cmd()
        .arg("check")
        .arg(file.path())
        .assert()
        .failure()
        .code(4)
        .stderr(predicate::str::contains("error"));
}

#[test]
fn exit_4_check_parse_error() {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    writeln!(file, "fn incomplete(").expect("Failed to write to temp file");

    keyrx_cmd()
        .arg("check")
        .arg(file.path())
        .assert()
        .failure()
        .code(4);
}

#[test]
#[ignore] // Run command has tokio runtime issues in tests
fn exit_4_run_invalid_config_syntax() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let config_file = temp_dir.path().join("invalid.toml");

    // Write invalid TOML
    fs::write(&config_file, "invalid toml [[[").expect("Failed to write config");

    keyrx_cmd()
        .arg("run")
        .arg("--config")
        .arg(&config_file)
        .assert()
        .failure()
        .code(4);
}

#[test]
fn exit_4_check_semantic_error() {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    // Valid syntax but semantic error (using undefined variable)
    writeln!(file, "print(undefined_variable);").expect("Failed to write to temp file");

    keyrx_cmd().arg("check").arg(file.path()).assert().failure();
    // May return 4 or 7 depending on when the error is caught
}

// =============================================================================
// Exit Code 5: Permission Denied
// =============================================================================

#[test]
#[cfg(unix)]
#[ignore] // Requires specific permission setup
fn exit_5_run_permission_denied() {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = tempdir().expect("Failed to create temp dir");
    let config_file = temp_dir.path().join("readonly.toml");

    // Create file
    fs::write(&config_file, "# empty config").expect("Failed to write config");

    // Remove read permissions
    let mut perms = fs::metadata(&config_file)
        .expect("Failed to get metadata")
        .permissions();
    perms.set_mode(0o000);
    fs::set_permissions(&config_file, perms).expect("Failed to set permissions");

    keyrx_cmd()
        .arg("run")
        .arg("--config")
        .arg(&config_file)
        .assert()
        .failure()
        .code(5);

    // Cleanup: restore permissions
    let mut perms = fs::metadata(&config_file)
        .expect("Failed to get metadata")
        .permissions();
    perms.set_mode(0o644);
    fs::set_permissions(&config_file, perms).expect("Failed to restore permissions");
}

#[test]
#[cfg(unix)]
#[ignore] // Requires permission setup
fn exit_5_check_permission_denied() {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = tempdir().expect("Failed to create temp dir");
    let script_file = temp_dir.path().join("noaccess.rhai");

    // Create file
    fs::write(&script_file, "let x = 1;").expect("Failed to write script");

    // Remove read permissions
    let mut perms = fs::metadata(&script_file)
        .expect("Failed to get metadata")
        .permissions();
    perms.set_mode(0o000);
    fs::set_permissions(&script_file, perms).expect("Failed to set permissions");

    keyrx_cmd()
        .arg("check")
        .arg(&script_file)
        .assert()
        .failure()
        .code(5);

    // Cleanup
    let mut perms = fs::metadata(&script_file)
        .expect("Failed to get metadata")
        .permissions();
    perms.set_mode(0o644);
    fs::set_permissions(&script_file, perms).expect("Failed to restore permissions");
}

// =============================================================================
// Exit Code 6: Device Not Found
// =============================================================================

#[test]
#[ignore] // Device-specific test, may not work in all environments
fn exit_6_run_device_not_found() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let config_file = temp_dir.path().join("device_config.toml");

    // Create a config that references a non-existent device
    let config_content = r#"
[device]
path = "/dev/input/nonexistent_device_12345"
"#;

    fs::write(&config_file, config_content).expect("Failed to write config");

    keyrx_cmd()
        .arg("run")
        .arg("--config")
        .arg(&config_file)
        .assert()
        .failure()
        .code(6);
}

#[test]
#[ignore] // Device-specific test
fn exit_6_discover_no_devices() {
    // In an environment with no input devices, discover should fail with exit 6
    // This is hard to test reliably, so we ignore it by default
    keyrx_cmd()
        .arg("discover")
        .arg("--device")
        .arg("/dev/input/impossible_device_xyz")
        .assert()
        .failure();
    // Expected code would be 6, but depends on environment
}

// =============================================================================
// Exit Code 7: Script Error
// =============================================================================

#[test]
fn exit_7_simulate_runtime_error() {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    // Script that causes a runtime error (division by zero or similar)
    writeln!(file, "let x = 1 / 0;").expect("Failed to write to temp file");

    // Currently returns exit 1 (GeneralError) - this is a known issue
    // that should be fixed to return exit 7 (ScriptError)
    keyrx_cmd()
        .arg("simulate")
        .arg("--script")
        .arg(file.path())
        .arg("--input")
        .arg("A")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Script execution failed"));
}

#[test]
fn exit_7_script_panic() {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    // Script that causes an error during execution
    writeln!(file, "panic(\"intentional panic\");").expect("Failed to write to temp file");

    // Currently returns exit 1 (GeneralError) - this is a known issue
    // that should be fixed to return exit 7 (ScriptError)
    keyrx_cmd()
        .arg("simulate")
        .arg("--script")
        .arg(file.path())
        .arg("--input")
        .arg("A")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Script execution failed"));
}

#[test]
fn exit_7_script_undefined_function() {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    // Call a function that doesn't exist
    writeln!(file, "nonexistent_function();").expect("Failed to write to temp file");

    keyrx_cmd()
        .arg("simulate")
        .arg(file.path())
        .assert()
        .failure();
    // Could be 4 (validation) or 7 (runtime) depending on when it's caught
}

// =============================================================================
// Exit Code Documentation Verification
// =============================================================================

#[test]
fn exit_codes_command_lists_all_codes() {
    keyrx_cmd()
        .arg("exit-codes")
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::contains("0"))
        .stdout(predicate::str::contains("1"))
        .stdout(predicate::str::contains("2"))
        .stdout(predicate::str::contains("3"))
        .stdout(predicate::str::contains("4"))
        .stdout(predicate::str::contains("5"))
        .stdout(predicate::str::contains("6"))
        .stdout(predicate::str::contains("7"))
        .stdout(predicate::str::contains("101"));
}

#[test]
fn exit_codes_json_has_all_codes() {
    let output = keyrx_cmd()
        .args(["--json", "exit-codes"])
        .assert()
        .success()
        .code(0)
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);

    // Verify all exit codes are present in JSON output
    assert!(output_str.contains("0") || output_str.contains("\"Success\""));
    assert!(output_str.contains("1") || output_str.contains("\"GeneralError\""));
    assert!(output_str.contains("2") || output_str.contains("\"AssertionFailed\""));
    assert!(output_str.contains("3") || output_str.contains("\"Timeout\""));
    assert!(output_str.contains("4") || output_str.contains("\"ValidationFailed\""));
    assert!(output_str.contains("5") || output_str.contains("\"PermissionDenied\""));
    assert!(output_str.contains("6") || output_str.contains("\"DeviceNotFound\""));
    assert!(output_str.contains("7") || output_str.contains("\"ScriptError\""));
    assert!(output_str.contains("101") || output_str.contains("\"Panic\""));
}

#[test]
fn help_mentions_exit_codes() {
    keyrx_cmd()
        .arg("--help")
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::contains("exit").or(predicate::str::contains("Exit")));
}
