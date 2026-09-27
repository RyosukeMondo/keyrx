//! Integration tests for `keyrx_daemon metrics` command.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// Helper to create a test command.
#[allow(deprecated)]
fn metrics_cmd() -> Command {
    #[allow(deprecated)]
    Command::cargo_bin("keyrx_daemon").unwrap()
}

#[test]
fn test_metrics_latency_daemon_not_running() {
    let temp_dir = TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("test-daemon.sock");

    // Try to query latency metrics when daemon is not running
    metrics_cmd()
        .arg("metrics")
        .arg("latency")
        .arg("--socket")
        .arg(&socket_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Daemon socket not found"))
        .stderr(predicate::str::contains("error code 3005"));
}

#[test]
fn test_metrics_events_daemon_not_running() {
    let temp_dir = TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("test-daemon.sock");

    // Try to query events when daemon is not running
    metrics_cmd()
        .arg("metrics")
        .arg("events")
        .arg("--socket")
        .arg(&socket_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Daemon socket not found"))
        .stderr(predicate::str::contains("error code 3005"));
}

#[test]
fn test_metrics_latency_json_daemon_not_running() {
    let temp_dir = TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("test-daemon.sock");

    // Try to query latency metrics with JSON output when daemon is not running
    let output = metrics_cmd()
        .arg("metrics")
        .arg("latency")
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
fn test_metrics_events_json_daemon_not_running() {
    let temp_dir = TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("test-daemon.sock");

    // Try to query events with JSON output when daemon is not running
    let output = metrics_cmd()
        .arg("metrics")
        .arg("events")
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
fn test_metrics_events_custom_count() {
    let temp_dir = TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("test-daemon.sock");

    // Verify that custom count is parsed correctly (even if daemon isn't running)
    metrics_cmd()
        .arg("metrics")
        .arg("events")
        .arg("--count")
        .arg("50")
        .arg("--socket")
        .arg(&socket_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Daemon socket not found"));
}

#[test]
fn test_metrics_events_follow_without_daemon_fails_fast() {
    let temp_dir = TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("test-daemon.sock");

    // Follow mode needs a daemon; it must not hang polling a missing socket.
    metrics_cmd()
        .arg("metrics")
        .arg("events")
        .arg("--follow")
        .arg("--socket")
        .arg(&socket_path)
        .timeout(std::time::Duration::from_secs(10))
        .assert()
        .failure()
        .stderr(predicate::str::contains("Daemon socket not found"));
}

#[test]
fn test_metrics_help() {
    metrics_cmd()
        .arg("metrics")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Query daemon performance metrics"))
        .stdout(predicate::str::contains("latency"))
        .stdout(predicate::str::contains("events"))
        .stdout(predicate::str::contains("--json"))
        .stdout(predicate::str::contains("--socket"));
}

#[test]
fn test_metrics_latency_help() {
    metrics_cmd()
        .arg("metrics")
        .arg("latency")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("latency"));
}

#[test]
fn test_metrics_events_help() {
    metrics_cmd()
        .arg("metrics")
        .arg("events")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("events"))
        .stdout(predicate::str::contains("--count"))
        .stdout(predicate::str::contains("--follow"));
}

#[test]
fn test_metrics_default_socket_not_running() {
    // Test with default socket path (daemon not running)
    // This should fail gracefully with appropriate error message
    metrics_cmd()
        .arg("metrics")
        .arg("latency")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Daemon socket not found")
                .or(predicate::str::contains("error code 3005")),
        );
}

#[test]
fn test_metrics_events_default_count() {
    let temp_dir = TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("test-daemon.sock");

    // Test with default count (100)
    // Should fail because daemon isn't running, but parsing should succeed
    metrics_cmd()
        .arg("metrics")
        .arg("events")
        .arg("--socket")
        .arg(&socket_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Daemon socket not found"));
}

// Note: Testing with a real running daemon requires:
// 1. Starting a daemon in the background
// 2. Waiting for it to initialize
// 3. Querying its metrics
// 4. Cleaning up the daemon
//
// This is more complex and would be better suited for end-to-end tests.
// For now, we verify the command interface and error handling.
