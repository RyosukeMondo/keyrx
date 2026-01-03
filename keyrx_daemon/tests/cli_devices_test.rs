//! Integration tests for `keyrx_daemon devices` command.

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::TempDir;

/// Helper to create a test command with a temporary registry.
#[allow(deprecated)]
fn devices_cmd(temp_dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("keyrx_daemon").unwrap();

    // Set XDG_CONFIG_HOME to use temp directory
    let config_dir = temp_dir.path();
    cmd.env("XDG_CONFIG_HOME", config_dir);

    cmd
}

#[test]
fn test_devices_list_empty() {
    let temp_dir = TempDir::new().unwrap();

    devices_cmd(&temp_dir)
        .arg("devices")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No devices registered"));
}

#[test]
fn test_devices_list_empty_json() {
    let temp_dir = TempDir::new().unwrap();

    let output = devices_cmd(&temp_dir)
        .arg("devices")
        .arg("list")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["devices"].as_array().unwrap().len(), 0);
}

#[test]
fn test_devices_rename_not_found() {
    let temp_dir = TempDir::new().unwrap();

    devices_cmd(&temp_dir)
        .arg("devices")
        .arg("rename")
        .arg("nonexistent-device")
        .arg("New Name")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_devices_rename_not_found_json() {
    let temp_dir = TempDir::new().unwrap();

    let output = devices_cmd(&temp_dir)
        .arg("devices")
        .arg("rename")
        .arg("nonexistent-device")
        .arg("New Name")
        .arg("--json")
        .assert()
        .failure()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["success"], false);
    assert_eq!(json["code"], 1001);
    assert!(json["error"].as_str().unwrap().contains("not found"));
}

#[test]
fn test_devices_set_scope_invalid() {
    let temp_dir = TempDir::new().unwrap();

    devices_cmd(&temp_dir)
        .arg("devices")
        .arg("set-scope")
        .arg("device1")
        .arg("invalid-scope")
        .assert()
        .failure();
}

#[test]
fn test_devices_forget_not_found() {
    let temp_dir = TempDir::new().unwrap();

    devices_cmd(&temp_dir)
        .arg("devices")
        .arg("forget")
        .arg("nonexistent-device")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_devices_set_layout_not_found() {
    let temp_dir = TempDir::new().unwrap();

    devices_cmd(&temp_dir)
        .arg("devices")
        .arg("set-layout")
        .arg("nonexistent-device")
        .arg("ansi_104")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_devices_help() {
    let temp_dir = TempDir::new().unwrap();

    devices_cmd(&temp_dir)
        .arg("devices")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Device management"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("rename"))
        .stdout(predicate::str::contains("set-scope"))
        .stdout(predicate::str::contains("forget"))
        .stdout(predicate::str::contains("set-layout"));
}
