//! Integration tests for the `keyrx test` CLI command.
//!
//! Tests built-in scenario execution, pass/fail reporting, and JSON output.
//!
//! Note: These tests use thread-local HOME override via scoped environment changes.

use std::fs;
use std::io::Write;
use std::sync::Mutex;
use tempfile::TempDir;

// Global mutex to serialize tests that modify environment variables
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Create a test profile directory with a dummy KRX file.
fn create_test_profile(dir: &TempDir, name: &str) {
    let profiles_dir = dir.path().join(".config").join("keyrx").join("profiles");
    fs::create_dir_all(&profiles_dir).unwrap();

    let krx_path = profiles_dir.join(format!("{}.krx", name));
    let mut file = fs::File::create(&krx_path).unwrap();
    file.write_all(b"test krx data").unwrap();
}

#[test]
fn test_run_all_scenarios() {
    let _lock = ENV_LOCK.lock().unwrap();
    let temp_dir = TempDir::new().unwrap();
    std::env::set_var("HOME", temp_dir.path());

    create_test_profile(&temp_dir, "default");

    // Import the execute function
    use keyrx_daemon::cli::test::{execute, TestArgs};

    let args = TestArgs {
        profile: Some("default".to_string()),
        scenario: "all".to_string(),
        json: false,
    };

    // Execute should succeed
    let result = execute(args);
    if let Err(e) = &result {
        eprintln!("Error: {}", e);
    }
    assert!(result.is_ok());
}

#[test]
fn test_run_specific_scenario() {
    let _lock = ENV_LOCK.lock().unwrap();
    let temp_dir = TempDir::new().unwrap();
    std::env::set_var("HOME", temp_dir.path());

    create_test_profile(&temp_dir, "default");

    use keyrx_daemon::cli::test::{execute, TestArgs};

    let args = TestArgs {
        profile: Some("default".to_string()),
        scenario: "tap-hold-under-threshold".to_string(),
        json: false,
    };

    let result = execute(args);
    assert!(result.is_ok());
}

#[test]
fn test_invalid_scenario_name() {
    let _lock = ENV_LOCK.lock().unwrap();
    let temp_dir = TempDir::new().unwrap();
    std::env::set_var("HOME", temp_dir.path());

    create_test_profile(&temp_dir, "default");

    use keyrx_daemon::cli::test::{execute, TestArgs};

    let args = TestArgs {
        profile: Some("default".to_string()),
        scenario: "invalid-scenario".to_string(),
        json: false,
    };

    let result = execute(args);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Unknown scenario"));
}

#[test]
fn test_profile_not_found() {
    let _lock = ENV_LOCK.lock().unwrap();
    let temp_dir = TempDir::new().unwrap();
    std::env::set_var("HOME", temp_dir.path());

    // Don't create a profile

    use keyrx_daemon::cli::test::{execute, TestArgs};

    let args = TestArgs {
        profile: Some("nonexistent".to_string()),
        scenario: "all".to_string(),
        json: false,
    };

    let result = execute(args);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Profile not found"));
}

#[test]
fn test_json_output_format() {
    let _lock = ENV_LOCK.lock().unwrap();
    let temp_dir = TempDir::new().unwrap();
    std::env::set_var("HOME", temp_dir.path());

    create_test_profile(&temp_dir, "test");

    use keyrx_daemon::cli::test::{execute, TestArgs};

    let args = TestArgs {
        profile: Some("test".to_string()),
        scenario: "all".to_string(),
        json: true,
    };

    // JSON output should succeed
    let result = execute(args);
    assert!(result.is_ok());
}

#[test]
fn test_all_scenario_names() {
    let _lock = ENV_LOCK.lock().unwrap();
    let temp_dir = TempDir::new().unwrap();
    std::env::set_var("HOME", temp_dir.path());

    create_test_profile(&temp_dir, "default");

    use keyrx_daemon::cli::test::{execute, TestArgs};

    let scenarios = vec![
        "tap-hold-under-threshold",
        "tap-hold-over-threshold",
        "permissive-hold",
        "cross-device-modifiers",
        "macro-sequence",
    ];

    for scenario in scenarios {
        let args = TestArgs {
            profile: Some("default".to_string()),
            scenario: scenario.to_string(),
            json: false,
        };

        let result = execute(args);
        assert!(
            result.is_ok(),
            "Scenario '{}' should execute successfully",
            scenario
        );
    }
}

#[test]
fn test_default_profile_fallback() {
    let _lock = ENV_LOCK.lock().unwrap();
    let temp_dir = TempDir::new().unwrap();
    std::env::set_var("HOME", temp_dir.path());

    create_test_profile(&temp_dir, "default");

    use keyrx_daemon::cli::test::{execute, TestArgs};

    // Don't specify a profile - should use "default"
    let args = TestArgs {
        profile: None,
        scenario: "all".to_string(),
        json: false,
    };

    let result = execute(args);
    assert!(result.is_ok());
}
