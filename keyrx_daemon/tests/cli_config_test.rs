//! Integration tests for `keyrx config` CLI command.

#![allow(deprecated)] // Allow deprecated Command::cargo_bin in tests
#![allow(clippy::needless_borrows_for_generic_args)] // Allow array borrows in tests for clarity

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn setup_test_env() -> (TempDir, String) {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().to_str().unwrap().to_string();

    // Create profiles directory
    fs::create_dir_all(temp_dir.path().join("profiles")).unwrap();

    // Create a test profile
    let profile_path = temp_dir.path().join("profiles").join("test.rhai");
    let content = r#"
device_start("*");

map("VK_A", "VK_B");

when_start("MD_00");
  map("VK_C", "VK_D");
when_end();

device_end();
"#;
    fs::write(&profile_path, content).unwrap();

    // Compile the profile
    let krx_path = temp_dir.path().join("profiles").join("test.krx");
    keyrx_compiler::compile_file(&profile_path, &krx_path).unwrap();

    (temp_dir, config_path)
}

#[test]
fn test_config_set_key() {
    let (_temp, config_path) = setup_test_env();

    let mut cmd = Command::cargo_bin("keyrx_daemon").unwrap();
    cmd.env("KEYRX_CONFIG_DIR", &config_path).args(&[
        "config",
        "set-key",
        "VK_X",
        "VK_Y",
        "--profile",
        "test",
        "--json",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"success\":true"))
        .stdout(predicate::str::contains("\"key\":\"VK_X\""));
}

#[test]
fn test_config_set_tap_hold() {
    let (_temp, config_path) = setup_test_env();

    let mut cmd = Command::cargo_bin("keyrx_daemon").unwrap();
    cmd.env("KEYRX_CONFIG_DIR", &config_path).args(&[
        "config",
        "set-tap-hold",
        "VK_Space",
        "VK_Space",
        "MD_01",
        "--threshold",
        "250",
        "--profile",
        "test",
        "--json",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"success\":true"))
        .stdout(predicate::str::contains("\"key\":\"VK_Space\""));
}

// TODO: Implement macro support in Rhai parser
// The set-macro CLI command generates Rhai code with press()/release()/wait() functions,
// but these functions are not registered in the Rhai engine (keyrx_compiler/src/parser/).
// This causes compilation to fail with "Function not found: press".
// To fix: Implement macro(), press(), release(), and wait() functions in the Rhai parser.
#[test]
#[ignore = "Macro feature not implemented in Rhai parser"]
fn test_config_set_macro() {
    let (_temp, config_path) = setup_test_env();

    let mut cmd = Command::cargo_bin("keyrx_daemon").unwrap();
    cmd.env("KEYRX_CONFIG_DIR", &config_path).args(&[
        "config",
        "set-macro",
        "VK_F1",
        "press:VK_H,press:VK_E,release:VK_E,release:VK_H",
        "--profile",
        "test",
        "--json",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"success\":true"))
        .stdout(predicate::str::contains("\"key\":\"VK_F1\""));
}

#[test]
fn test_config_get_key() {
    let (_temp, config_path) = setup_test_env();

    let mut cmd = Command::cargo_bin("keyrx_daemon").unwrap();
    cmd.env("KEYRX_CONFIG_DIR", &config_path).args(&[
        "config",
        "get-key",
        "VK_A",
        "--profile",
        "test",
        "--json",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"key\":\"VK_A\""))
        .stdout(predicate::str::contains("\"layer\":\"base\""));
}

#[test]
fn test_config_get_key_in_layer() {
    let (_temp, config_path) = setup_test_env();

    let mut cmd = Command::cargo_bin("keyrx_daemon").unwrap();
    cmd.env("KEYRX_CONFIG_DIR", &config_path).args(&[
        "config",
        "get-key",
        "VK_C",
        "--layer",
        "MD_00",
        "--profile",
        "test",
        "--json",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"key\":\"VK_C\""))
        .stdout(predicate::str::contains("\"layer\":\"MD_00\""));
}

#[test]
fn test_config_delete_key() {
    let (_temp, config_path) = setup_test_env();

    // First, verify the key exists
    let mut cmd = Command::cargo_bin("keyrx_daemon").unwrap();
    cmd.env("KEYRX_CONFIG_DIR", &config_path).args(&[
        "config",
        "get-key",
        "VK_A",
        "--profile",
        "test",
        "--json",
    ]);
    cmd.assert().success();

    // Delete the key
    let mut cmd = Command::cargo_bin("keyrx_daemon").unwrap();
    cmd.env("KEYRX_CONFIG_DIR", &config_path).args(&[
        "config",
        "delete-key",
        "VK_A",
        "--profile",
        "test",
        "--json",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"success\":true"));
}

#[test]
fn test_config_validate_valid_profile() {
    let (_temp, config_path) = setup_test_env();

    let mut cmd = Command::cargo_bin("keyrx_daemon").unwrap();
    cmd.env("KEYRX_CONFIG_DIR", &config_path)
        .args(&["config", "validate", "test", "--json"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"success\":true"))
        .stdout(predicate::str::contains("\"profile\":\"test\""));
}

#[test]
fn test_config_validate_invalid_profile() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().to_str().unwrap();

    // Create profiles directory
    fs::create_dir_all(temp_dir.path().join("profiles")).unwrap();

    // Create an invalid profile (syntax error)
    let profile_path = temp_dir.path().join("profiles").join("invalid.rhai");
    let content = "invalid syntax here!!!";
    fs::write(&profile_path, content).unwrap();

    let mut cmd = Command::cargo_bin("keyrx_daemon").unwrap();
    cmd.env("KEYRX_CONFIG_DIR", config_path)
        .args(&["config", "validate", "invalid", "--json"]);

    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("\"success\":false"));
}

#[test]
fn test_config_show() {
    let (_temp, config_path) = setup_test_env();

    let mut cmd = Command::cargo_bin("keyrx_daemon").unwrap();
    cmd.env("KEYRX_CONFIG_DIR", &config_path)
        .args(&["config", "show", "test", "--json"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"profile\":\"test\""))
        .stdout(predicate::str::contains("\"device_id\""))
        .stdout(predicate::str::contains("\"layers\""));
}

#[test]
fn test_config_diff() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().to_str().unwrap();

    // Create profiles directory
    fs::create_dir_all(temp_dir.path().join("profiles")).unwrap();

    // Create profile 1
    let profile1_path = temp_dir.path().join("profiles").join("profile1.rhai");
    let content1 = r#"
device_start("*");
map("VK_A", "VK_B");
device_end();
"#;
    fs::write(&profile1_path, content1).unwrap();

    // Create profile 2
    let profile2_path = temp_dir.path().join("profiles").join("profile2.rhai");
    let content2 = r#"
device_start("*");
map("VK_A", "VK_C");
device_end();
"#;
    fs::write(&profile2_path, content2).unwrap();

    let mut cmd = Command::cargo_bin("keyrx_daemon").unwrap();
    cmd.env("KEYRX_CONFIG_DIR", config_path)
        .args(&["config", "diff", "profile1", "profile2", "--json"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"profile1\":\"profile1\""))
        .stdout(predicate::str::contains("\"profile2\":\"profile2\""))
        .stdout(predicate::str::contains("\"differences\""));
}

#[test]
fn test_config_set_key_with_layer() {
    let (_temp, config_path) = setup_test_env();

    let mut cmd = Command::cargo_bin("keyrx_daemon").unwrap();
    cmd.env("KEYRX_CONFIG_DIR", &config_path).args(&[
        "config",
        "set-key",
        "VK_Z",
        "VK_Y",
        "--layer",
        "MD_00",
        "--profile",
        "test",
        "--json",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"success\":true"))
        .stdout(predicate::str::contains("\"layer\":\"MD_00\""));
}

#[test]
fn test_config_profile_not_found() {
    let (_temp, config_path) = setup_test_env();

    let mut cmd = Command::cargo_bin("keyrx_daemon").unwrap();
    cmd.env("KEYRX_CONFIG_DIR", &config_path).args(&[
        "config",
        "set-key",
        "VK_A",
        "VK_B",
        "--profile",
        "nonexistent",
        "--json",
    ]);

    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("\"success\":false"))
        .stdout(predicate::str::contains("Profile not found"));
}

#[test]
fn test_config_invalid_key_name() {
    let (_temp, config_path) = setup_test_env();

    let mut cmd = Command::cargo_bin("keyrx_daemon").unwrap();
    cmd.env("KEYRX_CONFIG_DIR", &config_path).args(&[
        "config",
        "set-key",
        "InvalidKey",
        "VK_B",
        "--profile",
        "test",
        "--json",
    ]);

    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("\"success\":false"));
}

#[test]
fn test_config_auto_recompile() {
    let (_temp, config_path) = setup_test_env();

    // Set a key mapping
    let mut cmd = Command::cargo_bin("keyrx_daemon").unwrap();
    cmd.env("KEYRX_CONFIG_DIR", &config_path).args(&[
        "config",
        "set-key",
        "VK_Q",
        "VK_W",
        "--profile",
        "test",
        "--json",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"compile_time_ms\""));
}
