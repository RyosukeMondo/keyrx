//! Integration tests for the `keyrx test` CLI command.
//!
//! Tests built-in scenario execution, pass/fail reporting, and JSON output.
//!
//! Note: These tests use thread-local HOME override via scoped environment changes.

use keyrx_core::config::{ConfigRoot, DeviceConfig, DeviceIdentifier, Metadata, Version};
use std::fs;
use std::io::Write;
use std::sync::Mutex;
use tempfile::TempDir;

// Global mutex to serialize tests that modify environment variables
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Create a test profile directory with a real, valid KRX file.
/// Returns the config dir path for use with KEYRX_CONFIG_DIR.
///
/// Writes a real, valid `.krx` file: the CLI's `execute` now loads and
/// validates the config like the daemon does, so a placeholder byte string
/// is no longer a valid fixture.
fn create_test_profile(dir: &TempDir, name: &str) -> std::path::PathBuf {
    let config_dir = dir.path().join("keyrx");
    let profiles_dir = config_dir.join("profiles");
    fs::create_dir_all(&profiles_dir).unwrap();

    let krx_path = profiles_dir.join(format!("{}.krx", name));
    let config = ConfigRoot {
        version: Version::current(),
        devices: vec![DeviceConfig {
            identifier: DeviceIdentifier {
                pattern: "*".to_string(),
            },
            mappings: vec![],
        }],
        metadata: Metadata {
            compilation_timestamp: 0,
            compiler_version: "test".to_string(),
            source_hash: "test".to_string(),
        },
    };
    let bytes = keyrx_compiler::serialize::serialize(&config).unwrap();
    let mut file = fs::File::create(&krx_path).unwrap();
    file.write_all(&bytes).unwrap();

    config_dir
}

#[test]
fn test_run_all_scenarios() {
    let _lock = ENV_LOCK.lock().unwrap();
    let temp_dir = TempDir::new().unwrap();
    let config_dir = create_test_profile(&temp_dir, "default");
    std::env::set_var("KEYRX_CONFIG_DIR", &config_dir);

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
    let config_dir = create_test_profile(&temp_dir, "default");
    std::env::set_var("KEYRX_CONFIG_DIR", &config_dir);

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
    let config_dir = create_test_profile(&temp_dir, "default");
    std::env::set_var("KEYRX_CONFIG_DIR", &config_dir);

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
    let config_dir = temp_dir.path().join("keyrx");
    fs::create_dir_all(config_dir.join("profiles")).unwrap();
    std::env::set_var("KEYRX_CONFIG_DIR", &config_dir);

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
    let config_dir = create_test_profile(&temp_dir, "test");
    std::env::set_var("KEYRX_CONFIG_DIR", &config_dir);

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
    let config_dir = create_test_profile(&temp_dir, "default");
    std::env::set_var("KEYRX_CONFIG_DIR", &config_dir);

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
fn test_none_profile_falls_back_to_active_profile() {
    let _lock = ENV_LOCK.lock().unwrap();
    let temp_dir = TempDir::new().unwrap();
    let config_dir = create_test_profile(&temp_dir, "default");
    // `profile: None` no longer guesses "default" - it resolves the actual
    // active profile via `<config_dir>/.active` (see
    // `daemon::live_config::read_active_profile_name`).
    fs::write(config_dir.join(".active"), b"default").unwrap();
    std::env::set_var("KEYRX_CONFIG_DIR", &config_dir);

    use keyrx_daemon::cli::test::{execute, TestArgs};

    // Don't specify a profile - should fall back to the active profile
    let args = TestArgs {
        profile: None,
        scenario: "all".to_string(),
        json: false,
    };

    let result = execute(args);
    assert!(result.is_ok());
}
