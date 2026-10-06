//! Integration tests for `keyrx profiles` command.

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use tempfile::TempDir;

/// Helper to create a test command with a temporary config directory.
#[allow(deprecated)]
fn profiles_cmd(temp_dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("keyrx_daemon").unwrap();

    // Set KEYRX_CONFIG_DIR to use temp directory (works on all platforms)
    let config_dir = temp_dir.path().join("keyrx");
    std::fs::create_dir_all(&config_dir).unwrap();
    cmd.env("KEYRX_CONFIG_DIR", config_dir);

    cmd
}

#[test]
fn test_profiles_list_empty() {
    let temp_dir = TempDir::new().unwrap();

    profiles_cmd(&temp_dir)
        .arg("profiles")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No profiles found"));
}

#[test]
fn test_profiles_list_empty_json() {
    let temp_dir = TempDir::new().unwrap();

    let output = profiles_cmd(&temp_dir)
        .arg("profiles")
        .arg("list")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["profiles"].as_array().unwrap().len(), 0);
    assert_eq!(json["active"], Value::Null);
}

#[test]
fn test_profiles_create_blank() {
    let temp_dir = TempDir::new().unwrap();

    profiles_cmd(&temp_dir)
        .arg("profiles")
        .arg("create")
        .arg("test-profile")
        .assert()
        .success()
        .stdout(predicate::str::contains("Profile 'test-profile' created"));

    // Verify profile file exists
    let profile_path = temp_dir
        .path()
        .join("keyrx")
        .join("profiles")
        .join("test-profile.rhai");
    assert!(profile_path.exists());
}

#[test]
fn test_profiles_create_blank_json() {
    let temp_dir = TempDir::new().unwrap();

    let output = profiles_cmd(&temp_dir)
        .arg("profiles")
        .arg("create")
        .arg("test-json")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["name"], "test-json");
}

#[test]
fn test_profiles_create_qmk() {
    let temp_dir = TempDir::new().unwrap();

    profiles_cmd(&temp_dir)
        .arg("profiles")
        .arg("create")
        .arg("qmk-test")
        .arg("--template")
        .arg("vim_navigation") // Updated to use valid template
        .assert()
        .success();

    // Verify profile file contains device configuration
    let profile_path = temp_dir
        .path()
        .join("keyrx")
        .join("profiles")
        .join("qmk-test.rhai");
    let content = fs::read_to_string(profile_path).unwrap();
    assert!(content.contains("device_start"));
    assert!(content.contains("device_end"));
}

#[test]
fn test_profiles_create_invalid_name() {
    let temp_dir = TempDir::new().unwrap();

    profiles_cmd(&temp_dir)
        .arg("profiles")
        .arg("create")
        .arg("invalid name!")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Invalid name"));
}

#[test]
fn test_profiles_create_already_exists() {
    let temp_dir = TempDir::new().unwrap();

    // Create profile first time
    profiles_cmd(&temp_dir)
        .arg("profiles")
        .arg("create")
        .arg("duplicate")
        .assert()
        .success();

    // Try to create again
    profiles_cmd(&temp_dir)
        .arg("profiles")
        .arg("create")
        .arg("duplicate")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn test_profiles_list_with_profiles() {
    let temp_dir = TempDir::new().unwrap();

    // Create some profiles
    profiles_cmd(&temp_dir)
        .arg("profiles")
        .arg("create")
        .arg("profile1")
        .assert()
        .success();

    profiles_cmd(&temp_dir)
        .arg("profiles")
        .arg("create")
        .arg("profile2")
        .arg("--template")
        .arg("vim_navigation")
        .assert()
        .success();

    // List profiles
    let output = profiles_cmd(&temp_dir)
        .arg("profiles")
        .arg("list")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["profiles"].as_array().unwrap().len(), 2);
}

#[test]
fn test_profiles_duplicate() {
    let temp_dir = TempDir::new().unwrap();

    // Create original profile
    profiles_cmd(&temp_dir)
        .arg("profiles")
        .arg("create")
        .arg("original")
        .assert()
        .success();

    // Duplicate it
    profiles_cmd(&temp_dir)
        .arg("profiles")
        .arg("duplicate")
        .arg("original")
        .arg("copy")
        .assert()
        .success()
        .stdout(predicate::str::contains("duplicated"));

    // Verify copy exists
    let copy_path = temp_dir
        .path()
        .join("keyrx")
        .join("profiles")
        .join("copy.rhai");
    assert!(copy_path.exists());
}

#[test]
fn test_profiles_duplicate_not_found() {
    let temp_dir = TempDir::new().unwrap();

    profiles_cmd(&temp_dir)
        .arg("profiles")
        .arg("duplicate")
        .arg("nonexistent")
        .arg("copy")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_profiles_delete_with_confirm() {
    let temp_dir = TempDir::new().unwrap();

    // Create profile
    profiles_cmd(&temp_dir)
        .arg("profiles")
        .arg("create")
        .arg("to-delete")
        .assert()
        .success();

    // Delete it with --confirm flag
    profiles_cmd(&temp_dir)
        .arg("profiles")
        .arg("delete")
        .arg("to-delete")
        .arg("--confirm")
        .assert()
        .success()
        .stdout(predicate::str::contains("deleted"));

    // Verify profile is gone
    let profile_path = temp_dir
        .path()
        .join("keyrx")
        .join("profiles")
        .join("to-delete.rhai");
    assert!(!profile_path.exists());
}

#[test]
fn test_profiles_delete_not_found() {
    let temp_dir = TempDir::new().unwrap();

    profiles_cmd(&temp_dir)
        .arg("profiles")
        .arg("delete")
        .arg("nonexistent")
        .arg("--confirm")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_profiles_export() {
    let temp_dir = TempDir::new().unwrap();

    // Create profile
    profiles_cmd(&temp_dir)
        .arg("profiles")
        .arg("create")
        .arg("to-export")
        .assert()
        .success();

    // Export it
    let export_path = temp_dir.path().join("exported.rhai");
    profiles_cmd(&temp_dir)
        .arg("profiles")
        .arg("export")
        .arg("to-export")
        .arg(export_path.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("exported"));

    // Verify export file exists
    assert!(export_path.exists());
}

#[test]
fn test_profiles_export_not_found() {
    let temp_dir = TempDir::new().unwrap();

    let export_path = temp_dir.path().join("exported.rhai");
    profiles_cmd(&temp_dir)
        .arg("profiles")
        .arg("export")
        .arg("nonexistent")
        .arg(export_path.to_str().unwrap())
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_profiles_import() {
    let temp_dir = TempDir::new().unwrap();

    // Create a source file
    let source_path = temp_dir.path().join("source.rhai");
    fs::write(
        &source_path,
        "device_start(\"*\");\nmap(\"A\", \"VK_B\");\ndevice_end();\n",
    )
    .unwrap();

    // Import it
    profiles_cmd(&temp_dir)
        .arg("profiles")
        .arg("import")
        .arg(source_path.to_str().unwrap())
        .arg("imported")
        .assert()
        .success()
        .stdout(predicate::str::contains("imported"));

    // Verify imported profile exists
    let imported_path = temp_dir
        .path()
        .join("keyrx")
        .join("profiles")
        .join("imported.rhai");
    assert!(imported_path.exists());
}

#[test]
fn test_profiles_import_krx_converts_and_defaults_the_name() {
    let temp_dir = TempDir::new().unwrap();
    let krx = concat!(env!("CARGO_MANIFEST_DIR"), "/../uat_tests/test5_vim.krx");

    profiles_cmd(&temp_dir)
        .args(["profiles", "import", krx])
        .assert()
        .success()
        .stdout(predicate::str::contains("test5_vim"))
        .stdout(predicate::str::contains("Converted"));

    let profiles = temp_dir.path().join("keyrx").join("profiles");
    let source = fs::read_to_string(profiles.join("test5_vim.rhai")).unwrap();
    assert!(source.contains("when_start(\"MD_00\");"), "{source}");
    assert!(profiles.join("test5_vim.krx").exists());
}

#[test]
fn test_profiles_import_rejects_a_corrupt_krx_and_stores_nothing() {
    let temp_dir = TempDir::new().unwrap();
    let bad = temp_dir.path().join("bad.krx");
    fs::write(
        &bad,
        b"definitely not a krx file, but long enough to have a header....",
    )
    .unwrap();

    profiles_cmd(&temp_dir)
        .args(["profiles", "import"])
        .arg(&bad)
        .assert()
        .failure()
        .stderr(predicate::str::contains("not a valid .krx"));

    let profiles = temp_dir.path().join("keyrx").join("profiles");
    assert_eq!(fs::read_dir(profiles).map(|d| d.count()).unwrap_or(0), 0);
}

#[test]
fn test_profiles_activate_not_found() {
    let temp_dir = TempDir::new().unwrap();

    profiles_cmd(&temp_dir)
        .arg("profiles")
        .arg("activate")
        .arg("nonexistent")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_profiles_help() {
    let temp_dir = TempDir::new().unwrap();

    profiles_cmd(&temp_dir)
        .arg("profiles")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Manage configuration profiles"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("activate"))
        .stdout(predicate::str::contains("delete"))
        .stdout(predicate::str::contains("duplicate"))
        .stdout(predicate::str::contains("export"))
        .stdout(predicate::str::contains("import"));
}

/// `profiles import --device` installs a `device_start("*")` sample scoped to
/// one keyboard (the daily layout must never grab every keyboard on the box).
#[test]
fn test_profiles_import_device_scopes_a_wildcard_layout() {
    let temp_dir = TempDir::new().unwrap();
    let sample = temp_dir.path().join("sample.rhai");
    fs::write(
        &sample,
        "device_start(\"*\");\nmap(\"A\", \"VK_B\");\ndevice_end();\n",
    )
    .unwrap();

    profiles_cmd(&temp_dir)
        .args(["profiles", "import"])
        .arg(&sample)
        .args(["scoped", "--device", "USB Keyboard"])
        .assert()
        .success();

    let stored = fs::read_to_string(temp_dir.path().join("keyrx/profiles/scoped.rhai")).unwrap();
    assert!(
        stored.contains("device_start(\"USB Keyboard\")"),
        "{stored}"
    );
    assert!(!stored.contains("device_start(\"*\")"), "{stored}");
}

#[test]
fn test_profiles_import_device_refuses_a_layout_it_cannot_scope() {
    let temp_dir = TempDir::new().unwrap();
    let sample = temp_dir.path().join("pinned.rhai");
    fs::write(
        &sample,
        "device_start(\"Other\");\nmap(\"A\", \"VK_B\");\ndevice_end();\n",
    )
    .unwrap();

    profiles_cmd(&temp_dir)
        .args(["profiles", "import"])
        .arg(&sample)
        .args(["pinned", "--device", "USB Keyboard"])
        .assert()
        .failure();
    assert!(!temp_dir.path().join("keyrx/profiles/pinned.rhai").exists());
}
