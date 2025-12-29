//! Integration tests for `keyrx_daemon status` command.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// Helper to create a test command with a custom socket path.
fn status_cmd() -> Command {
    Command::cargo_bin("keyrx_daemon").unwrap()
}

#[test]
fn test_status_daemon_not_running() {
    let temp_dir = TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("test-daemon.sock");

    // Try to query status when daemon is not running (socket doesn't exist)
    status_cmd()
        .arg("status")
        .arg("--socket")
        .arg(&socket_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Daemon socket not found"))
        .stderr(predicate::str::contains("error code 3005"));
}

#[test]
fn test_status_daemon_not_running_json() {
    let temp_dir = TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("test-daemon.sock");

    // Try to query status with JSON output when daemon is not running
    let output = status_cmd()
        .arg("status")
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
fn test_status_default_socket_not_running() {
    // Test with default socket path (daemon not running)
    // This should fail gracefully with appropriate error message
    status_cmd().arg("status").assert().failure().stderr(
        predicate::str::contains("Daemon socket not found")
            .or(predicate::str::contains("error code 3005")),
    );
}

#[test]
fn test_status_help() {
    status_cmd()
        .arg("status")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Query daemon status"))
        .stdout(predicate::str::contains("--json"))
        .stdout(predicate::str::contains("--socket"));
}

#[test]
fn test_status_json_flag_parsing() {
    let temp_dir = TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("test-daemon.sock");

    // Verify that JSON flag is parsed correctly (even if daemon isn't running)
    // The command should fail with socket error, not argument parsing error
    status_cmd()
        .arg("status")
        .arg("--json")
        .arg("--socket")
        .arg(&socket_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Daemon socket not found"));
}

// Note: Testing with a real running daemon requires:
// 1. Starting a daemon in the background
// 2. Waiting for it to initialize
// 3. Querying its status
// 4. Cleaning up the daemon
//
// This is more complex and would be better suited for end-to-end tests.
// For now, we verify the command interface and error handling.
