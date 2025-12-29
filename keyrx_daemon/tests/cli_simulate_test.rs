//! Integration tests for the `keyrx simulate` CLI command.

use serde_json::Value;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Get the path to the keyrx_daemon binary.
fn get_binary_path() -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // Remove test binary name
    path.pop(); // Remove "deps"
    path.push("keyrx_daemon");
    path
}

/// Create a test environment with config directory and KRX file.
fn create_test_environment() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().to_path_buf();
    let profiles_dir = config_dir.join("profiles");
    fs::create_dir_all(&profiles_dir).unwrap();

    // Create a test KRX file
    let krx_path = profiles_dir.join("default.krx");
    fs::write(&krx_path, b"test krx data").unwrap();

    (temp_dir, config_dir)
}

/// Create an event file.
fn create_event_file(dir: &std::path::Path, events: &str) -> PathBuf {
    let path = dir.join("events.json");
    let mut file = fs::File::create(&path).unwrap();
    file.write_all(events.as_bytes()).unwrap();
    path
}

#[test]
fn test_simulate_with_inline_events() {
    let (_temp_dir, config_dir) = create_test_environment();

    let output = Command::new(get_binary_path())
        .arg("simulate")
        .arg("--profile")
        .arg("default")
        .arg("--events")
        .arg("press:A,wait:50,release:A")
        .arg("--json")
        .env("KEYRX_CONFIG_DIR", &config_dir)
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Parse JSON output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).expect("Failed to parse JSON");

    assert_eq!(json["success"], true);
    assert_eq!(json["seed"], 0);
    assert!(json["input"].is_array());
    assert!(json["output"].is_array());

    // Verify input events
    let input = json["input"].as_array().unwrap();
    assert_eq!(input.len(), 2);
    assert_eq!(input[0]["key"], "A");
    assert_eq!(input[0]["event_type"], "press");
    assert_eq!(input[1]["key"], "A");
    assert_eq!(input[1]["event_type"], "release");
}

#[test]
fn test_simulate_with_event_file() {
    let (_temp_dir, config_dir) = create_test_environment();

    let events_json = r#"{
        "events": [
            {
                "device_id": null,
                "timestamp_us": 0,
                "key": "B",
                "event_type": "press"
            },
            {
                "device_id": null,
                "timestamp_us": 100000,
                "key": "B",
                "event_type": "release"
            }
        ],
        "seed": 42
    }"#;

    let events_file = create_event_file(config_dir.as_path(), events_json);

    let output = Command::new(get_binary_path())
        .arg("simulate")
        .arg("--profile")
        .arg("default")
        .arg("--events-file")
        .arg(events_file)
        .arg("--json")
        .env("KEYRX_CONFIG_DIR", &config_dir)
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Parse JSON output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).expect("Failed to parse JSON");

    assert_eq!(json["success"], true);
    assert_eq!(json["seed"], 42);

    // Verify input events
    let input = json["input"].as_array().unwrap();
    assert_eq!(input.len(), 2);
    assert_eq!(input[0]["key"], "B");
}

#[test]
fn test_simulate_with_custom_seed() {
    let (_temp_dir, config_dir) = create_test_environment();

    let output = Command::new(get_binary_path())
        .arg("simulate")
        .arg("--profile")
        .arg("default")
        .arg("--events")
        .arg("press:C,wait:25,release:C")
        .arg("--seed")
        .arg("123")
        .arg("--json")
        .env("KEYRX_CONFIG_DIR", &config_dir)
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).expect("Failed to parse JSON");

    assert_eq!(json["seed"], 123);
}

#[test]
fn test_simulate_determinism() {
    let (_temp_dir, config_dir) = create_test_environment();

    // Run same simulation twice with same seed
    let run_simulation = || {
        Command::new(get_binary_path())
            .arg("simulate")
            .arg("--profile")
            .arg("default")
            .arg("--events")
            .arg("press:D,wait:75,release:D")
            .arg("--seed")
            .arg("999")
            .arg("--json")
            .env("KEYRX_CONFIG_DIR", &config_dir)
            .output()
            .expect("Failed to execute command")
    };

    let output1 = run_simulation();
    let output2 = run_simulation();

    assert!(output1.status.success());
    assert!(output2.status.success());

    let json1: Value = serde_json::from_str(&String::from_utf8_lossy(&output1.stdout)).unwrap();
    let json2: Value = serde_json::from_str(&String::from_utf8_lossy(&output2.stdout)).unwrap();

    // Both runs should produce identical output
    assert_eq!(json1["output"], json2["output"]);
}

#[test]
fn test_simulate_profile_not_found() {
    let (_temp_dir, config_dir) = create_test_environment();

    let output = Command::new(get_binary_path())
        .arg("simulate")
        .arg("--profile")
        .arg("nonexistent")
        .arg("--events")
        .arg("press:E,release:E")
        .env("KEYRX_CONFIG_DIR", &config_dir)
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not found") || stderr.contains("Profile"));
}

#[test]
fn test_simulate_missing_events() {
    let (_temp_dir, config_dir) = create_test_environment();

    let output = Command::new(get_binary_path())
        .arg("simulate")
        .arg("--profile")
        .arg("default")
        .env("KEYRX_CONFIG_DIR", &config_dir)
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("events") || stderr.contains("must be specified"));
}

#[test]
fn test_simulate_human_readable_output() {
    let (_temp_dir, config_dir) = create_test_environment();

    let output = Command::new(get_binary_path())
        .arg("simulate")
        .arg("--profile")
        .arg("default")
        .arg("--events")
        .arg("press:F,wait:30,release:F")
        .env("KEYRX_CONFIG_DIR", &config_dir)
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Simulation Results"));
    assert!(stdout.contains("Input Events"));
    assert!(stdout.contains("Output Events"));
    assert!(stdout.contains("seed:"));
}

#[test]
fn test_simulate_invalid_dsl() {
    let (_temp_dir, config_dir) = create_test_environment();

    let output = Command::new(get_binary_path())
        .arg("simulate")
        .arg("--profile")
        .arg("default")
        .arg("--events")
        .arg("invalid-dsl-format")
        .arg("--json")
        .env("KEYRX_CONFIG_DIR", &config_dir)
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
}

#[test]
fn test_simulate_conflicts_events_and_file() {
    let (_temp_dir, config_dir) = create_test_environment();
    let events_file = create_event_file(config_dir.as_path(), r#"{"events": [], "seed": 0}"#);

    let output = Command::new(get_binary_path())
        .arg("simulate")
        .arg("--profile")
        .arg("default")
        .arg("--events")
        .arg("press:G,release:G")
        .arg("--events-file")
        .arg(events_file)
        .env("KEYRX_CONFIG_DIR", &config_dir)
        .output()
        .expect("Failed to execute command");

    // Should fail due to conflicting arguments
    assert!(!output.status.success());
}
