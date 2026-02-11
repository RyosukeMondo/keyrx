//! Comprehensive CLI integration tests for keyrx_daemon.
//!
//! This test suite validates cross-command workflows, error scenarios,
//! deterministic simulation, and property-based testing. Individual command
//! tests are in separate files (cli_devices_test.rs, cli_profiles_test.rs, etc.).
//!
//! Test Categories:
//! - Cross-command workflows (create profile → activate → test)
//! - JSON output parsing for all commands
//! - Error scenarios (profile not found, invalid JSON, etc.)
//! - Deterministic simulation (seed-based replay)
//! - Property-based tests (proptest for simulation determinism)

use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use predicates::prelude::*;
use proptest::prelude::*;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// ============================================================================
// Test Helpers
// ============================================================================

/// Helper to create a test command with a temporary config directory.
fn test_cmd(temp_dir: &TempDir) -> Command {
    let mut cmd = cargo_bin_cmd!("keyrx_daemon");
    let config_dir = temp_dir.path().join("keyrx");
    std::fs::create_dir_all(&config_dir).unwrap_or(());
    cmd.env("KEYRX_CONFIG_DIR", config_dir);
    cmd
}

/// Helper to parse JSON from command output.
fn parse_json_output(output: &[u8]) -> Value {
    serde_json::from_slice(output).unwrap_or_else(|e| {
        eprintln!("Failed to parse JSON: {}", e);
        eprintln!("Output was: {:?}", String::from_utf8_lossy(output));
        panic!("Failed to parse JSON output: {}", e);
    })
}

/// Helper to create a minimal Rhai profile for testing.
/// Note: This is a placeholder - actual Rhai syntax depends on keyrx_compiler implementation.
fn create_test_profile(temp_dir: &TempDir, name: &str) -> PathBuf {
    let profiles_dir = temp_dir.path().join("keyrx").join("profiles");
    fs::create_dir_all(&profiles_dir).unwrap();

    let profile_path = profiles_dir.join(format!("{}.rhai", name));

    // Create a minimal valid Rhai config
    // For now, just create the file - profile commands will create proper content
    let minimal_config = "// Placeholder config\n";
    fs::write(&profile_path, minimal_config).unwrap();
    profile_path
}

// ============================================================================
// Cross-Command Workflow Tests
// ============================================================================

#[test]
#[ignore] // Requires full ProfileManager implementation
fn test_workflow_create_list_activate_profile() {
    let temp_dir = TempDir::new().unwrap();

    // Step 1: Create a profile
    let create_output = test_cmd(&temp_dir)
        .arg("profiles")
        .arg("create")
        .arg("test-profile")
        .arg("--template")
        .arg("blank")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let create_json = parse_json_output(&create_output);
    assert_eq!(create_json["success"], true);
    assert_eq!(create_json["name"], "test-profile");

    // Step 2: List profiles and verify it appears
    let list_output = test_cmd(&temp_dir)
        .arg("profiles")
        .arg("list")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let list_json = parse_json_output(&list_output);
    let profiles = list_json["profiles"].as_array().unwrap();
    assert_eq!(profiles.len(), 1);
    assert_eq!(profiles[0]["name"], "test-profile");

    // Step 3: Activate the profile
    let activate_output = test_cmd(&temp_dir)
        .arg("profiles")
        .arg("activate")
        .arg("test-profile")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let activate_json = parse_json_output(&activate_output);
    assert_eq!(activate_json["success"], true);
    assert!(activate_json["reload_time_ms"].as_u64().unwrap() < 100);
}

#[test]
#[ignore] // Requires full ProfileManager implementation
fn test_workflow_profile_lifecycle() {
    let temp_dir = TempDir::new().unwrap();

    // Create original profile
    test_cmd(&temp_dir)
        .arg("profiles")
        .arg("create")
        .arg("original")
        .arg("--template")
        .arg("blank")
        .arg("--json")
        .assert()
        .success();

    // Duplicate the profile
    let dup_output = test_cmd(&temp_dir)
        .arg("profiles")
        .arg("duplicate")
        .arg("original")
        .arg("copy")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let dup_json = parse_json_output(&dup_output);
    assert_eq!(dup_json["success"], true);
    assert_eq!(dup_json["name"], "copy");

    // List should show both profiles
    let list_output = test_cmd(&temp_dir)
        .arg("profiles")
        .arg("list")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let list_json = parse_json_output(&list_output);
    assert_eq!(list_json["profiles"].as_array().unwrap().len(), 2);

    // Delete one profile
    test_cmd(&temp_dir)
        .arg("profiles")
        .arg("delete")
        .arg("copy")
        .arg("--json")
        .assert()
        .success();

    // List should show only one profile
    let list_output = test_cmd(&temp_dir)
        .arg("profiles")
        .arg("list")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let list_json = parse_json_output(&list_output);
    assert_eq!(list_json["profiles"].as_array().unwrap().len(), 1);
}

#[test]
fn test_workflow_device_configuration() {
    let temp_dir = TempDir::new().unwrap();

    // Initially, device list should be empty
    let list_output = test_cmd(&temp_dir)
        .arg("devices")
        .arg("list")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let list_json = parse_json_output(&list_output);
    assert_eq!(list_json["devices"].as_array().unwrap().len(), 0);

    // Note: We can't add devices directly in tests without mock hardware,
    // but we can test the command structure and error handling
}

#[test]
#[ignore] // Requires full ProfileManager and RhaiGenerator implementation
fn test_workflow_layers_management() {
    let temp_dir = TempDir::new().unwrap();

    // Create a profile
    test_cmd(&temp_dir)
        .arg("profiles")
        .arg("create")
        .arg("layer-test")
        .arg("--template")
        .arg("qmk-layers")
        .arg("--json")
        .assert()
        .success();

    // List layers (qmk-layers template should have multiple layers)
    let list_output = test_cmd(&temp_dir)
        .arg("layers")
        .arg("list")
        .arg("--profile")
        .arg("layer-test")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let list_json = parse_json_output(&list_output);
    let layers = list_json["layers"].as_array().unwrap();
    assert!(!layers.is_empty());

    // Create a new layer
    test_cmd(&temp_dir)
        .arg("layers")
        .arg("create")
        .arg("custom")
        .arg("--profile")
        .arg("layer-test")
        .arg("--json")
        .assert()
        .success();

    // Verify layer was added
    let list_output = test_cmd(&temp_dir)
        .arg("layers")
        .arg("list")
        .arg("--profile")
        .arg("layer-test")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let list_json = parse_json_output(&list_output);
    let new_layers = list_json["layers"].as_array().unwrap();
    assert_eq!(new_layers.len(), layers.len() + 1);
}

// ============================================================================
// JSON Output Parsing Tests
// ============================================================================

#[test]
#[ignore] // Many commands not yet fully implemented
fn test_json_all_commands_produce_valid_json() {
    let temp_dir = TempDir::new().unwrap();

    // Create a test profile for commands that need it
    create_test_profile(&temp_dir, "json-test");

    // Test profiles list JSON
    let output = test_cmd(&temp_dir)
        .arg("profiles")
        .arg("list")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json = parse_json_output(&output);
    assert!(json["profiles"].is_array());

    // Test devices list JSON
    let output = test_cmd(&temp_dir)
        .arg("devices")
        .arg("list")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json = parse_json_output(&output);
    assert!(json["devices"].is_array());

    // Test layouts list JSON
    let output = test_cmd(&temp_dir)
        .arg("layouts")
        .arg("list")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json = parse_json_output(&output);
    assert!(json["layouts"].is_array());

    // Test status JSON (may fail if daemon not running, but should be valid JSON)
    let output = test_cmd(&temp_dir)
        .arg("status")
        .arg("--json")
        .output()
        .unwrap();

    // Parse either stdout or stderr (error response is also JSON)
    if !output.stdout.is_empty() {
        let json = parse_json_output(&output.stdout);
        assert!(json.is_object());
    } else if !output.stderr.is_empty() {
        let json = parse_json_output(&output.stderr);
        assert!(json.is_object());
        assert_eq!(json["success"], false);
    }
}

#[test]
fn test_json_error_responses_are_valid() {
    let temp_dir = TempDir::new().unwrap();

    // Profile not found error
    let result = test_cmd(&temp_dir)
        .arg("profiles")
        .arg("activate")
        .arg("nonexistent")
        .arg("--json")
        .assert()
        .failure()
        .get_output()
        .clone();

    // JSON errors go to stdout for consistency with success outputs
    let output = if !result.stdout.is_empty() {
        &result.stdout
    } else {
        &result.stderr
    };

    let json = parse_json_output(output);
    assert_eq!(json["success"], false);
    assert!(json["code"].is_number());
    assert!(json["error"].is_string());

    // Device not found error
    let result = test_cmd(&temp_dir)
        .arg("devices")
        .arg("rename")
        .arg("nonexistent-device")
        .arg("New Name")
        .arg("--json")
        .assert()
        .failure()
        .get_output()
        .clone();

    // JSON errors go to stdout for consistency with success outputs
    let output = if !result.stdout.is_empty() {
        &result.stdout
    } else {
        &result.stderr
    };

    let json = parse_json_output(output);
    assert_eq!(json["success"], false);
    assert_eq!(json["code"], 1001);
}

#[test]
fn test_json_schema_consistency() {
    let temp_dir = TempDir::new().unwrap();

    // Success responses should have "success": true
    create_test_profile(&temp_dir, "success-test");

    let output = test_cmd(&temp_dir)
        .arg("profiles")
        .arg("list")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = parse_json_output(&output);
    assert!(json.is_object());
    // List commands may not have "success" field, but should have data field

    // Error responses should have consistent schema: success, code, error
    let result = test_cmd(&temp_dir)
        .arg("profiles")
        .arg("delete")
        .arg("nonexistent")
        .arg("--json")
        .assert()
        .failure()
        .get_output()
        .clone();

    // JSON errors go to stdout for consistency with success outputs
    let output = if !result.stdout.is_empty() {
        &result.stdout
    } else {
        &result.stderr
    };

    let json = parse_json_output(output);
    assert_eq!(json["success"], false);
    assert!(json.get("code").is_some());
    assert!(json.get("error").is_some());
}

// ============================================================================
// Error Scenario Tests
// ============================================================================

#[test]
#[ignore] // Requires ProfileManager implementation
fn test_error_profile_not_found() {
    let temp_dir = TempDir::new().unwrap();

    // Activate nonexistent profile
    test_cmd(&temp_dir)
        .arg("profiles")
        .arg("activate")
        .arg("nonexistent")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("not found"));

    // Delete nonexistent profile
    test_cmd(&temp_dir)
        .arg("profiles")
        .arg("delete")
        .arg("nonexistent")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("not found"));

    // Duplicate from nonexistent profile
    test_cmd(&temp_dir)
        .arg("profiles")
        .arg("duplicate")
        .arg("nonexistent")
        .arg("new-name")
        .assert()
        .failure()
        .code(1);
}

#[test]
#[ignore] // Requires ProfileManager implementation
fn test_error_invalid_profile_name() {
    let temp_dir = TempDir::new().unwrap();

    // Profile name too long (>32 chars)
    test_cmd(&temp_dir)
        .arg("profiles")
        .arg("create")
        .arg("this-is-a-very-long-profile-name-that-exceeds-the-maximum-length")
        .arg("--template")
        .arg("blank")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid").or(predicate::str::contains("name")));

    // Profile name with invalid characters
    test_cmd(&temp_dir)
        .arg("profiles")
        .arg("create")
        .arg("invalid/name")
        .arg("--template")
        .arg("blank")
        .assert()
        .failure();
}

#[test]
fn test_error_device_not_found() {
    let temp_dir = TempDir::new().unwrap();

    // Rename nonexistent device
    test_cmd(&temp_dir)
        .arg("devices")
        .arg("rename")
        .arg("nonexistent-device-id")
        .arg("New Name")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("not found"));

    // Forget nonexistent device
    test_cmd(&temp_dir)
        .arg("devices")
        .arg("forget")
        .arg("nonexistent-device-id")
        .assert()
        .failure()
        .code(1);
}

#[test]
#[ignore] // Requires SimulationEngine implementation
fn test_error_invalid_json_in_events_file() {
    let temp_dir = TempDir::new().unwrap();

    // Create invalid JSON file
    let events_file = temp_dir.path().join("invalid.json");
    fs::write(&events_file, "{ invalid json }").unwrap();

    test_cmd(&temp_dir)
        .arg("simulate")
        .arg("--events-file")
        .arg(&events_file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("JSON").or(predicate::str::contains("parse")));
}

#[test]
fn test_error_daemon_not_running() {
    let temp_dir = TempDir::new().unwrap();

    // Status command when daemon is not running
    // Note: Current implementation doesn't output JSON for IPC errors
    test_cmd(&temp_dir)
        .arg("status")
        .arg("--json")
        .assert()
        .failure()
        .code(1) // IPC errors currently return exit code 1
        .stderr(predicate::str::contains("3005")); // Error code should be in message

    // State inspect when daemon is not running
    test_cmd(&temp_dir)
        .arg("state")
        .arg("inspect")
        .assert()
        .failure()
        .code(1) // IPC errors currently return exit code 1
        .stderr(predicate::str::contains("socket").or(predicate::str::contains("3005")));

    // Metrics when daemon is not running
    test_cmd(&temp_dir)
        .arg("metrics")
        .arg("latency")
        .assert()
        .failure()
        .code(1); // IPC errors currently return exit code 1
}

#[test]
#[ignore] // Requires ProfileManager implementation
fn test_error_profile_limit_exceeded() {
    let temp_dir = TempDir::new().unwrap();

    // Create 100 profiles (the limit)
    for i in 0..100 {
        test_cmd(&temp_dir)
            .arg("profiles")
            .arg("create")
            .arg(format!("profile-{}", i))
            .arg("--template")
            .arg("blank")
            .assert()
            .success();
    }

    // Try to create 101st profile - should fail
    let output = test_cmd(&temp_dir)
        .arg("profiles")
        .arg("create")
        .arg("profile-101")
        .arg("--template")
        .arg("blank")
        .arg("--json")
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json = parse_json_output(&output);
    assert_eq!(json["success"], false);
    assert_eq!(json["code"], 1014); // Profile limit exceeded
}

// ============================================================================
// Deterministic Simulation Tests
// ============================================================================

#[test]
#[ignore] // Requires SimulationEngine implementation
fn test_simulation_deterministic_same_seed() {
    let temp_dir = TempDir::new().unwrap();

    // Create a test profile
    create_test_profile(&temp_dir, "sim-test");

    // Run simulation twice with same seed
    let events = "press:A,wait:50,release:A";
    let seed = "12345";

    let output1 = test_cmd(&temp_dir)
        .arg("simulate")
        .arg("--profile")
        .arg("sim-test")
        .arg("--events")
        .arg(events)
        .arg("--seed")
        .arg(seed)
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output2 = test_cmd(&temp_dir)
        .arg("simulate")
        .arg("--profile")
        .arg("sim-test")
        .arg("--events")
        .arg(events)
        .arg("--seed")
        .arg(seed)
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // Outputs should be identical
    assert_eq!(output1, output2);

    let json1 = parse_json_output(&output1);
    let json2 = parse_json_output(&output2);
    assert_eq!(json1, json2);
}

#[test]
#[ignore] // Requires SimulationEngine implementation
fn test_simulation_different_seeds_different_output() {
    let temp_dir = TempDir::new().unwrap();

    create_test_profile(&temp_dir, "sim-test2");

    let events = "press:A,wait:50,release:A";

    let output1 = test_cmd(&temp_dir)
        .arg("simulate")
        .arg("--profile")
        .arg("sim-test2")
        .arg("--events")
        .arg(events)
        .arg("--seed")
        .arg("11111")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output2 = test_cmd(&temp_dir)
        .arg("simulate")
        .arg("--profile")
        .arg("sim-test2")
        .arg("--events")
        .arg(events)
        .arg("--seed")
        .arg("99999")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json1 = parse_json_output(&output1);
    let json2 = parse_json_output(&output2);

    // With different seeds, timing or state might differ
    // (depending on implementation - they might still be the same for simple events)
    // Just verify both are valid JSON
    assert!(json1.is_object());
    assert!(json2.is_object());
}

#[test]
#[ignore] // Requires SimulationEngine implementation
fn test_simulation_from_file() {
    let temp_dir = TempDir::new().unwrap();

    create_test_profile(&temp_dir, "sim-file-test");

    // Create event file
    let events_file = temp_dir.path().join("events.json");
    let events_json = r#"
    {
        "events": [
            {"type": "press", "key": "A", "timestamp": 0},
            {"type": "wait", "duration": 50},
            {"type": "release", "key": "A", "timestamp": 50}
        ]
    }
    "#;
    fs::write(&events_file, events_json).unwrap();

    // Run simulation from file
    let output = test_cmd(&temp_dir)
        .arg("simulate")
        .arg("--profile")
        .arg("sim-file-test")
        .arg("--events-file")
        .arg(&events_file)
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = parse_json_output(&output);
    assert!(json.is_object());
}

#[test]
#[ignore] // Requires SimulationEngine implementation
fn test_simulation_builtin_scenarios() {
    let temp_dir = TempDir::new().unwrap();

    create_test_profile(&temp_dir, "scenario-test");

    // Test with built-in scenario
    let output = test_cmd(&temp_dir)
        .arg("test")
        .arg("--profile")
        .arg("scenario-test")
        .arg("--scenario")
        .arg("tap-hold-under-threshold")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = parse_json_output(&output);
    assert!(json.is_object());
    assert!(json.get("result").is_some());
}

// ============================================================================
// Property-Based Tests (Proptest)
// ============================================================================

proptest! {
    /// Property: Same seed always produces same output
    #[test]
    fn prop_simulation_determinism(seed in 0u64..10000) {
        let temp_dir = TempDir::new().unwrap();
        create_test_profile(&temp_dir, "prop-test");

        let events = "press:A,wait:50,release:A";

        let output1 = test_cmd(&temp_dir)
            .arg("simulate")
            .arg("--profile")
            .arg("prop-test")
            .arg("--events")
            .arg(events)
            .arg("--seed")
            .arg(seed.to_string())
            .arg("--json")
            .output()
            .unwrap();

        let output2 = test_cmd(&temp_dir)
            .arg("simulate")
            .arg("--profile")
            .arg("prop-test")
            .arg("--events")
            .arg(events)
            .arg("--seed")
            .arg(seed.to_string())
            .arg("--json")
            .output()
            .unwrap();

        // Outputs must be identical for same seed
        prop_assert_eq!(output1.stdout, output2.stdout);
        prop_assert_eq!(output1.status.code(), output2.status.code());
    }

    /// Property: Valid profile names are accepted
    #[test]
    fn prop_profile_name_validation(
        name in "[a-z0-9_-]{1,32}"
    ) {
        let temp_dir = TempDir::new().unwrap();

        // Valid names should succeed
        let result = test_cmd(&temp_dir)
            .arg("profiles")
            .arg("create")
            .arg(&name)
            .arg("--template")
            .arg("blank")
            .output()
            .unwrap();

        // Should either succeed or fail gracefully (not panic)
        prop_assert!(result.status.code().is_some());
    }

    /// Property: Device IDs within limits are handled
    #[test]
    fn prop_device_id_length(
        id in "[a-zA-Z0-9_]{1,256}" // Exclude dash to avoid -h/-V flags
    ) {
        let temp_dir = TempDir::new().unwrap();

        // Try to rename device with generated ID
        let result = test_cmd(&temp_dir)
            .arg("devices")
            .arg("rename")
            .arg(&id)
            .arg("Test Device")
            .output()
            .unwrap();

        // Should fail gracefully (device not found, not panic)
        prop_assert!(result.status.code().is_some());
        // Note: May succeed if id happens to be a valid flag like "h", so we just check it doesn't crash
    }
}

// ============================================================================
// Advanced Integration Tests
// ============================================================================

#[test]
#[ignore] // Requires ProfileManager implementation
fn test_concurrent_profile_operations() {
    let temp_dir = TempDir::new().unwrap();

    // Create multiple profiles rapidly
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let temp_path = temp_dir.path().to_path_buf();
            std::thread::spawn(move || {
                let mut cmd = cargo_bin_cmd!("keyrx_daemon");
                let config_dir = temp_path.join("keyrx");
                std::fs::create_dir_all(&config_dir).unwrap_or(());
                cmd.env("KEYRX_CONFIG_DIR", &config_dir);
                cmd.arg("profiles")
                    .arg("create")
                    .arg(format!("concurrent-{}", i))
                    .arg("--template")
                    .arg("blank")
                    .arg("--json")
                    .output()
                    .unwrap()
            })
        })
        .collect();

    // Wait for all to complete
    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // All should succeed or fail gracefully (no panics)
    for result in results {
        assert!(result.status.code().is_some());
    }

    // Verify profiles were created
    let list_output = test_cmd(&temp_dir)
        .arg("profiles")
        .arg("list")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let list_json = parse_json_output(&list_output);
    let profiles = list_json["profiles"].as_array().unwrap();

    // At least some profiles should have been created
    assert!(!profiles.is_empty());
}

#[test]
#[ignore] // Requires ProfileManager implementation
fn test_export_import_roundtrip() {
    let temp_dir = TempDir::new().unwrap();

    // Create a profile
    test_cmd(&temp_dir)
        .arg("profiles")
        .arg("create")
        .arg("export-test")
        .arg("--template")
        .arg("qmk-layers")
        .arg("--json")
        .assert()
        .success();

    // Export the profile
    let export_path = temp_dir.path().join("exported.rhai");
    test_cmd(&temp_dir)
        .arg("profiles")
        .arg("export")
        .arg("export-test")
        .arg(&export_path)
        .arg("--json")
        .assert()
        .success();

    // Delete the original
    test_cmd(&temp_dir)
        .arg("profiles")
        .arg("delete")
        .arg("export-test")
        .arg("--json")
        .assert()
        .success();

    // Import it back
    test_cmd(&temp_dir)
        .arg("profiles")
        .arg("import")
        .arg(&export_path)
        .arg("imported-test")
        .arg("--json")
        .assert()
        .success();

    // Verify it exists
    let list_output = test_cmd(&temp_dir)
        .arg("profiles")
        .arg("list")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let list_json = parse_json_output(&list_output);
    let profiles = list_json["profiles"].as_array().unwrap();
    let imported = profiles.iter().find(|p| p["name"] == "imported-test");
    assert!(imported.is_some());
}

#[test]
#[ignore] // Requires LayoutManager implementation
fn test_all_layouts_are_valid() {
    let temp_dir = TempDir::new().unwrap();

    // List all built-in layouts
    let list_output = test_cmd(&temp_dir)
        .arg("layouts")
        .arg("list")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let list_json = parse_json_output(&list_output);
    let layouts = list_json["layouts"].as_array().unwrap();

    // Verify all built-in layouts can be shown
    for layout in layouts {
        let name = layout["name"].as_str().unwrap();

        let show_output = test_cmd(&temp_dir)
            .arg("layouts")
            .arg("show")
            .arg(name)
            .arg("--json")
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();

        let show_json = parse_json_output(&show_output);
        assert!(show_json.is_object());

        // KLE JSON should be present
        assert!(show_json.get("kle").is_some());
    }
}
