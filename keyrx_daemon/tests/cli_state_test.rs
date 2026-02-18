//! Integration tests for `keyrx_daemon state` command.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// Helper to create a test command with a custom socket path.
#[allow(deprecated)]
fn state_cmd() -> Command {
    Command::cargo_bin("keyrx_daemon").unwrap()
}

#[test]
fn test_state_inspect_daemon_not_running() {
    let temp_dir = TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("test-daemon.sock");

    // Try to query state when daemon is not running (socket doesn't exist)
    state_cmd()
        .arg("state")
        .arg("inspect")
        .arg("--socket")
        .arg(&socket_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Daemon socket not found"))
        .stderr(predicate::str::contains("error code 3005"));
}

#[test]
fn test_state_inspect_daemon_not_running_json() {
    let temp_dir = TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("test-daemon.sock");

    // Try to query state with JSON output when daemon is not running
    let output = state_cmd()
        .arg("state")
        .arg("inspect")
        .arg("--socket")
        .arg(&socket_path)
        .arg("--json")
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    // Should still fail with error message
    let stderr = String::from_utf8_lossy(&output);
    assert!(stderr.contains("Daemon socket not found") || stderr.contains("error code 3005"));
}

#[test]
fn test_state_inspect_default_socket_not_running() {
    // Test with default socket path (daemon not running)
    // This should fail gracefully with appropriate error message
    state_cmd()
        .arg("state")
        .arg("inspect")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Daemon socket not found")
                .or(predicate::str::contains("error code 3005")),
        );
}

#[test]
fn test_state_help() {
    state_cmd()
        .arg("state")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Inspect runtime state"))
        .stdout(predicate::str::contains("inspect"));
}

#[test]
fn test_state_inspect_help() {
    state_cmd()
        .arg("state")
        .arg("inspect")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Inspect the current modifier/lock state",
        ))
        .stdout(predicate::str::contains("--json"))
        .stdout(predicate::str::contains("--socket"));
}

#[test]
fn test_state_inspect_json_flag_parsing() {
    let temp_dir = TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("test-daemon.sock");

    // Verify that JSON flag is parsed correctly (even if daemon isn't running)
    // The command should fail with socket error, not argument parsing error
    state_cmd()
        .arg("state")
        .arg("inspect")
        .arg("--json")
        .arg("--socket")
        .arg(&socket_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Daemon socket not found"));
}

#[test]
fn test_state_requires_subcommand() {
    // Calling "state" without a subcommand should show help and fail
    state_cmd()
        .arg("state")
        .assert()
        .failure()
        .stderr(predicate::str::contains("state <COMMAND>"));
}

// Note: Testing with a real running daemon requires:
// 1. Starting a daemon in the background
// 2. Waiting for it to initialize
// 3. Querying its state
// 4. Verifying the 255-bit state array
// 5. Cleaning up the daemon
//
// This is more complex and would be better suited for end-to-end tests.
// For now, we verify the command interface and error handling.
