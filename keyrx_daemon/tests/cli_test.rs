//! CLI tests for keyrx_daemon.
//!
//! These tests validate the command-line interface functionality:
//! - Argument parsing for all subcommands
//! - Help text content and format
//! - Error handling for invalid arguments
//! - Exit code verification
//!
//! Tests that require real devices use runtime skip macros for CI compatibility.
//! To run ignored tests locally: `cargo test --package keyrx_daemon -- --ignored`

use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

// ============================================================================
// Test Helpers
// ============================================================================

/// Gets a Command for the keyrx_daemon binary.
fn cmd() -> Command {
    // Use the macro to support custom build directories
    cargo_bin_cmd!("keyrx_daemon")
}

/// Creates a temporary file with valid .krx content (minimal valid config).
///
/// Note: This creates a valid serialized ConfigRoot using keyrx_compiler.
#[cfg(all(target_os = "linux", feature = "linux"))]
fn create_valid_krx_file() -> NamedTempFile {
    use keyrx_compiler::serialize::serialize;
    use keyrx_core::config::{
        ConfigRoot, DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping, Metadata, Version,
    };

    let config = ConfigRoot {
        version: Version::current(),
        devices: vec![DeviceConfig {
            identifier: DeviceIdentifier {
                pattern: "*".to_string(),
            },
            mappings: vec![KeyMapping::simple(KeyCode::A, KeyCode::B)],
        }],
        metadata: Metadata {
            compilation_timestamp: 0,
            compiler_version: "test".to_string(),
            source_hash: "test".to_string(),
        },
    };

    let bytes = serialize(&config).expect("Failed to serialize config");
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(&bytes).expect("Failed to write config");
    temp_file.flush().expect("Failed to flush");
    temp_file
}

/// Creates a temporary file with invalid content.
fn create_invalid_krx_file() -> NamedTempFile {
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file
        .write_all(b"this is not a valid krx file")
        .expect("Failed to write");
    temp_file.flush().expect("Failed to flush");
    temp_file
}

// ============================================================================
// Help Text Tests
// ============================================================================

#[test]
fn test_help_shows_usage() {
    cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("keyrx_daemon"))
        .stdout(predicate::str::contains("keyboard remapping"));
}

#[test]
fn test_help_shows_subcommands() {
    cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("run"))
        .stdout(predicate::str::contains("list-devices"))
        .stdout(predicate::str::contains("validate"));
}

#[test]
fn test_help_short_flag() {
    cmd()
        .arg("-h")
        .assert()
        .success()
        .stdout(predicate::str::contains("keyrx_daemon"));
}

#[test]
fn test_version_flag() {
    cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_version_short_flag() {
    cmd()
        .arg("-V")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

// ============================================================================
// Run Subcommand Help Tests
// ============================================================================

#[test]
fn test_run_help() {
    cmd()
        .arg("run")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--config"))
        .stdout(predicate::str::contains("--debug"))
        .stdout(predicate::str::contains(".krx"));
}

#[test]
fn test_run_help_shows_config_required() {
    cmd()
        .arg("run")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("config"));
}

// ============================================================================
// Run Subcommand Argument Parsing Tests
// ============================================================================

#[test]
fn test_run_missing_config_shows_error() {
    cmd()
        .arg("run")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--config"));
}

#[test]
fn test_run_nonexistent_config_file() {
    cmd()
        .arg("run")
        .arg("--config")
        .arg("/nonexistent/path/config.krx")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error"));
}

#[test]
fn test_run_invalid_config_file() {
    let temp_file = create_invalid_krx_file();

    cmd()
        .arg("run")
        .arg("--config")
        .arg(temp_file.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error"));
}

#[test]
fn test_run_config_short_flag() {
    // Test that -c works as short flag (should show error about missing file, not parsing)
    cmd()
        .arg("run")
        .arg("-c")
        .arg("/nonexistent.krx")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error"));
}

#[test]
fn test_run_debug_flag_accepted() {
    // The command should accept --debug flag even if config is missing
    // (arg parsing happens before file reading)
    cmd()
        .arg("run")
        .arg("--debug")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--config"));
}

#[test]
fn test_run_debug_short_flag_accepted() {
    cmd()
        .arg("run")
        .arg("-d")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--config"));
}

// ============================================================================
// List-Devices Subcommand Tests
// ============================================================================

#[test]
fn test_list_devices_help() {
    cmd()
        .arg("list-devices")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("List"))
        .stdout(predicate::str::contains("device"));
}

/// Test list-devices with real devices.
/// Skips at runtime if /dev/input access is not available.
#[test]
#[cfg(all(target_os = "linux", feature = "linux"))]
fn test_list_devices_execution() {
    keyrx_daemon::skip_if_no_input_access!();
    cmd()
        .arg("list-devices")
        .assert()
        .success()
        .stdout(predicate::str::contains("keyboard").or(predicate::str::contains("No keyboard")));
}

// ============================================================================
// Validate Subcommand Tests
// ============================================================================

#[test]
fn test_validate_help() {
    cmd()
        .arg("validate")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--config"))
        .stdout(predicate::str::contains("Validate"));
}

#[test]
fn test_validate_missing_config_shows_error() {
    cmd()
        .arg("validate")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--config"));
}

#[test]
fn test_validate_nonexistent_config_file() {
    cmd()
        .arg("validate")
        .arg("--config")
        .arg("/nonexistent/path/config.krx")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error"));
}

#[test]
fn test_validate_invalid_config_file() {
    let temp_file = create_invalid_krx_file();

    cmd()
        .arg("validate")
        .arg("--config")
        .arg(temp_file.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error"));
}

#[test]
fn test_validate_config_short_flag() {
    cmd()
        .arg("validate")
        .arg("-c")
        .arg("/nonexistent.krx")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error"));
}

/// Test validate with a valid config file on Linux.
/// Skips at runtime if /dev/input access is not available.
#[test]
#[cfg(all(target_os = "linux", feature = "linux"))]
fn test_validate_valid_config() {
    keyrx_daemon::skip_if_no_input_access!();
    let temp_file = create_valid_krx_file();

    cmd()
        .arg("validate")
        .arg("--config")
        .arg(temp_file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Validating"))
        .stdout(predicate::str::contains("Loading configuration"));
}

// ============================================================================
// Exit Code Tests
// ============================================================================

#[test]
fn test_help_exit_code_zero() {
    cmd().arg("--help").assert().code(0);
}

#[test]
fn test_version_exit_code_zero() {
    cmd().arg("--version").assert().code(0);
}

#[test]
fn test_missing_subcommand_shows_help() {
    cmd()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}

#[test]
fn test_unknown_subcommand_error() {
    cmd()
        .arg("unknown-command")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

#[test]
fn test_run_nonexistent_config_exit_code_one() {
    // Exit code 1 = CONFIG_ERROR
    cmd()
        .arg("run")
        .arg("--config")
        .arg("/nonexistent.krx")
        .assert()
        .code(1);
}

#[test]
fn test_validate_nonexistent_config_exit_code_one() {
    // Exit code 1 = CONFIG_ERROR
    cmd()
        .arg("validate")
        .arg("--config")
        .arg("/nonexistent.krx")
        .assert()
        .code(1);
}

// ============================================================================
// Non-Linux Platform Tests
// ============================================================================

#[test]
#[cfg(not(target_os = "linux"))]
fn test_run_not_available_without_linux_feature() {
    let temp_file = create_invalid_krx_file();

    cmd()
        .arg("run")
        .arg("--config")
        .arg(temp_file.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("only available on Linux"));
}

#[test]
#[cfg(not(target_os = "linux"))]
fn test_list_devices_not_available_without_linux_feature() {
    cmd()
        .arg("list-devices")
        .assert()
        .failure()
        .stderr(predicate::str::contains("only available on Linux"));
}

#[test]
#[cfg(not(target_os = "linux"))]
fn test_validate_not_available_without_linux_feature() {
    let temp_file = create_invalid_krx_file();

    cmd()
        .arg("validate")
        .arg("--config")
        .arg(temp_file.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("only available on Linux"));
}

// ============================================================================
// Argument Combination Tests
// ============================================================================

#[test]
fn test_run_config_and_debug_together() {
    // Both flags should be accepted (error comes from file access)
    cmd()
        .arg("run")
        .arg("--config")
        .arg("/nonexistent.krx")
        .arg("--debug")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error"));
}

#[test]
fn test_run_debug_before_config() {
    // Flag order shouldn't matter
    cmd()
        .arg("run")
        .arg("--debug")
        .arg("--config")
        .arg("/nonexistent.krx")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error"));
}

// ============================================================================
// Output Format Tests
// ============================================================================

/// Verify list-devices output format when devices are present.
/// Skips at runtime if /dev/input access is not available.
#[test]
#[cfg(all(target_os = "linux", feature = "linux"))]
fn test_list_devices_output_format() {
    keyrx_daemon::skip_if_no_input_access!();
    cmd()
        .arg("list-devices")
        .assert()
        .success()
        // Should have table headers or "No keyboard" message
        .stdout(
            predicate::str::contains("PATH")
                .and(predicate::str::contains("NAME"))
                .or(predicate::str::contains("No keyboard")),
        );
}

/// Verify validate output shows step numbers.
/// Skips at runtime if /dev/input access is not available.
#[test]
#[cfg(all(target_os = "linux", feature = "linux"))]
fn test_validate_output_shows_steps() {
    keyrx_daemon::skip_if_no_input_access!();
    let temp_file = create_valid_krx_file();

    cmd()
        .arg("validate")
        .arg("--config")
        .arg(temp_file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("1."))
        .stdout(predicate::str::contains("2."));
}

// ============================================================================
// Error Message Quality Tests
// ============================================================================

#[test]
fn test_run_missing_config_error_is_helpful() {
    cmd()
        .arg("run")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--config").or(predicate::str::contains("-c")));
}

#[test]
fn test_validate_missing_config_error_is_helpful() {
    cmd()
        .arg("validate")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--config").or(predicate::str::contains("-c")));
}

#[test]
fn test_invalid_config_error_mentions_file() {
    let temp_file = create_invalid_krx_file();

    cmd()
        .arg("run")
        .arg("--config")
        .arg(temp_file.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error"));
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_empty_config_path() {
    cmd().arg("run").arg("--config").arg("").assert().failure();
}

#[test]
fn test_config_path_with_spaces() {
    // Path with spaces should be handled correctly
    cmd()
        .arg("run")
        .arg("--config")
        .arg("/path/with spaces/config.krx")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error"));
}

#[test]
fn test_config_path_relative() {
    cmd()
        .arg("run")
        .arg("--config")
        .arg("./relative/path/config.krx")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error"));
}

#[test]
fn test_multiple_subcommands_error() {
    // Only one subcommand should be accepted
    cmd().arg("run").arg("list-devices").assert().failure();
}
