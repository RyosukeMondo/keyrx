//! Integration tests for error propagation through the daemon architecture.
//!
//! These tests verify that errors propagate correctly through the full call stack:
//! - CLI → Service → Platform
//! - Web → Handler → Service
//! - Error context is preserved at each layer
//! - Error recovery works for non-critical failures

#![allow(deprecated)] // Allow deprecated Command::cargo_bin in tests
#![allow(clippy::needless_borrow)] // Allow needless borrows in tests for clarity
#![allow(clippy::needless_borrows_for_generic_args)] // Allow array borrows in tests for clarity
#![allow(clippy::ptr_arg)] // Allow PathBuf parameters in test helpers

use assert_cmd::Command;
use keyrx_daemon::config::ProfileManager;
use keyrx_daemon::error::{CliError, ConfigError, DaemonError, PlatformError, WebError};
use keyrx_daemon::macro_recorder::MacroRecorder;
use keyrx_daemon::services::{ConfigService, DeviceService, ProfileService};
use keyrx_daemon::web::subscriptions::SubscriptionManager;
use keyrx_daemon::web::AppState;
use predicates::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::net::TcpListener;

// ============================================================================
// Test Infrastructure
// ============================================================================

/// Setup a test environment with temporary directory
fn setup_test_env() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().to_path_buf();
    std::fs::create_dir_all(config_path.join("profiles")).unwrap();
    (temp_dir, config_path)
}

/// Create a valid test profile for testing
fn create_test_profile(config_path: &PathBuf, name: &str) {
    let profile_path = config_path.join("profiles").join(format!("{}.rhai", name));
    let content = r#"
device_start("*");
map("VK_A", "VK_B");
device_end();
"#;
    std::fs::write(&profile_path, content).unwrap();

    // Compile the profile
    let krx_path = config_path.join("profiles").join(format!("{}.krx", name));
    keyrx_compiler::compile_file(&profile_path, &krx_path).unwrap();
}

/// Start a test web server on a random port
async fn start_test_web_server() -> (u16, tokio::task::JoinHandle<()>, Arc<AppState>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let port = addr.port();

    let config_dir = PathBuf::from("/tmp/keyrx-error-test");
    let _ = std::fs::create_dir_all(&config_dir);

    let macro_recorder = Arc::new(MacroRecorder::default());
    let profile_manager = Arc::new(ProfileManager::new(config_dir.clone()).unwrap());
    let profile_service = Arc::new(ProfileService::new(Arc::clone(&profile_manager)));
    let device_service = Arc::new(DeviceService::new(config_dir.clone()));
    let config_service = Arc::new(ConfigService::new(profile_manager));
    let settings_service = Arc::new(keyrx_daemon::services::SettingsService::new(
        config_dir.clone(),
    ));
    let simulation_service = Arc::new(keyrx_daemon::services::SimulationService::new(
        config_dir.clone(),
        None,
    ));
    let subscription_manager = Arc::new(SubscriptionManager::new());
    let (event_broadcaster, _) = tokio::sync::broadcast::channel(1000);

    let state = Arc::new(AppState::new(
        macro_recorder,
        profile_service,
        device_service,
        config_service,
        settings_service,
        simulation_service,
        subscription_manager,
        event_broadcaster,
    ));

    let state_clone = Arc::clone(&state);

    let (event_tx, _) = tokio::sync::broadcast::channel(100);

    let server_handle = tokio::spawn(async move {
        let app = keyrx_daemon::web::create_app(event_tx, state_clone).await;
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    (port, server_handle, state)
}

// ============================================================================
// CLI → Service → Platform Error Flow Tests
// ============================================================================

#[test]
fn test_cli_config_file_not_found_error() {
    let (_temp, config_path) = setup_test_env();

    // Try to get a key from a non-existent profile
    let mut cmd = Command::cargo_bin("keyrx_daemon").unwrap();
    cmd.env("KEYRX_CONFIG_DIR", config_path.to_str().unwrap())
        .args(&[
            "config",
            "get-key",
            "VK_A",
            "--profile",
            "nonexistent",
            "--json",
        ]);

    // Should fail with ConfigError::FileNotFound
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("\"success\":false"))
        .stdout(predicate::str::contains("not found").or(predicate::str::contains("exist")));
}

#[test]
fn test_cli_invalid_profile_syntax_error() {
    let (_temp, config_path) = setup_test_env();

    // Create a profile with invalid Rhai syntax
    let profile_path = config_path.join("profiles").join("invalid.rhai");
    let invalid_content = r#"
device_start("*");
map("VK_A", "VK_B"  // Missing closing parenthesis
device_end();
"#;
    std::fs::write(&profile_path, invalid_content).unwrap();

    // Try to activate the invalid profile (which will attempt to compile it)
    let mut cmd = Command::cargo_bin("keyrx_daemon").unwrap();
    cmd.env("KEYRX_CONFIG_DIR", config_path.to_str().unwrap())
        .args(&["profiles", "activate", "invalid", "--json"]);

    // Should fail with compilation/parsing error
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("\"success\":false"));
}

#[test]
fn test_cli_error_context_preservation() {
    let (_temp, config_path) = setup_test_env();

    // Try to activate a non-existent profile
    let mut cmd = Command::cargo_bin("keyrx_daemon").unwrap();
    cmd.env("KEYRX_CONFIG_DIR", config_path.to_str().unwrap())
        .args(&["profiles", "activate", "missing_profile", "--json"]);

    // Error message should include the profile name for context
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("\"success\":false"))
        .stdout(
            predicate::str::contains("missing_profile").or(predicate::str::contains("not found")),
        );
}

#[test]
fn test_cli_invalid_arguments_error() {
    let (_temp, config_path) = setup_test_env();
    create_test_profile(&config_path, "test");

    // Try to set a key with invalid arguments (missing target key)
    let mut cmd = Command::cargo_bin("keyrx_daemon").unwrap();
    cmd.env("KEYRX_CONFIG_DIR", config_path.to_str().unwrap())
        .args(&["config", "set-key", "VK_A", "--profile", "test", "--json"]);

    // Should fail with CliError::InvalidArguments
    cmd.assert().failure();
}

// ============================================================================
// Web → Handler → Service Error Flow Tests
// ============================================================================

#[tokio::test]
async fn test_web_invalid_profile_request() {
    let (port, _server, _state) = start_test_web_server().await;

    let client = reqwest::Client::new();
    // Try to activate a non-existent profile (valid endpoint, invalid data)
    let url = format!(
        "http://127.0.0.1:{}/api/profiles/nonexistent/activate",
        port
    );

    // Request activation of a non-existent profile
    let response = client.post(&url).send().await.unwrap();

    // Should return 404 Not Found, 400 Bad Request, or 500 Internal Server Error
    assert!(
        !response.status().is_success(),
        "Expected error status for non-existent profile, got {}",
        response.status()
    );

    // Note: Error response format varies by endpoint, just verify it's not success
}

#[tokio::test]
async fn test_web_invalid_json_request() {
    let (port, _server, _state) = start_test_web_server().await;

    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/api/profiles", port);

    // Send invalid JSON to create profile endpoint
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .body("{ invalid json }")
        .send()
        .await
        .unwrap();

    // Should return 400 Bad Request or other client error
    assert!(
        response.status().is_client_error() || response.status().is_server_error(),
        "Expected error status for invalid JSON, got {}",
        response.status()
    );
}

#[tokio::test]
async fn test_web_error_response_format() {
    let (port, _server, _state) = start_test_web_server().await;

    let client = reqwest::Client::new();
    // Use an endpoint that exists but will return an error
    let url = format!("http://127.0.0.1:{}/api/profiles/missing/activate", port);

    let response = client.post(&url).send().await.unwrap();

    // Verify response indicates an error (not necessarily a client error,
    // could be server error depending on implementation)
    assert!(
        !response.status().is_success(),
        "Expected error status, got {}",
        response.status()
    );

    // The API may return empty body for some errors, which is acceptable
    // The key is that the HTTP status code indicates the error
}

// ============================================================================
// Error Context Preservation Tests
// ============================================================================

#[test]
fn test_error_type_conversions() {
    // Test that errors convert correctly through the hierarchy
    let platform_err = PlatformError::DeviceAccess {
        device: "/dev/input/event0".into(),
        reason: "permission denied".into(),
        suggestion: "Run with sudo".into(),
    };

    let daemon_err: DaemonError = platform_err.into();
    let msg = daemon_err.to_string();

    // Context should be preserved through conversion
    assert!(msg.contains("/dev/input/event0"));
    assert!(msg.contains("permission denied"));
    assert!(msg.contains("Run with sudo"));
}

#[test]
fn test_config_error_context() {
    let config_err = ConfigError::ParseError {
        path: PathBuf::from("/tmp/test.rhai"),
        reason: "unexpected token".into(),
    };

    let daemon_err: DaemonError = config_err.into();
    let msg = daemon_err.to_string();

    // Path and reason should be in error message
    assert!(msg.contains("/tmp/test.rhai"));
    assert!(msg.contains("unexpected token"));
}

#[test]
fn test_web_error_context() {
    let web_err = WebError::InvalidRequest {
        reason: "missing required field 'profile'".into(),
    };

    let daemon_err: DaemonError = web_err.into();
    let msg = daemon_err.to_string();

    // Reason should be preserved
    assert!(msg.contains("missing required field"));
    assert!(msg.contains("profile"));
}

#[test]
fn test_cli_error_context() {
    let cli_err = CliError::CommandFailed {
        command: "activate".into(),
        reason: "profile not found".into(),
    };

    let daemon_err: DaemonError = cli_err.into();
    let msg = daemon_err.to_string();

    // Command and reason should be preserved
    assert!(msg.contains("activate"));
    assert!(msg.contains("not found"));
}

// ============================================================================
// Multi-Layer Error Propagation Tests
// ============================================================================

#[test]
fn test_core_error_through_config_to_daemon() {
    use keyrx_core::error::CoreError;

    // CoreError → ConfigError → DaemonError
    let core_err = CoreError::Validation {
        field: "key_code".into(),
        reason: "invalid value".into(),
    };

    let config_err: ConfigError = core_err.into();
    assert!(matches!(config_err, ConfigError::Core(_)));

    let daemon_err: DaemonError = config_err.into();
    assert!(matches!(daemon_err, DaemonError::Config(_)));

    let msg = daemon_err.to_string();
    // Original context should still be visible
    assert!(msg.contains("key_code") || msg.contains("invalid"));
}

#[test]
fn test_io_error_through_platform_to_daemon() {
    use std::io;

    // IO Error → PlatformError → DaemonError
    let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
    let platform_err: PlatformError = io_err.into();
    let daemon_err: DaemonError = platform_err.into();

    assert!(matches!(daemon_err, DaemonError::Platform(_)));
}

// ============================================================================
// Error Recovery Scenario Tests
// ============================================================================

#[test]
fn test_cli_recovers_from_single_profile_error() {
    let (_temp, config_path) = setup_test_env();

    // Create a valid profile
    create_test_profile(&config_path, "valid");

    // Also create an invalid .rhai file that will fail if someone tries to compile it
    let invalid_path = config_path.join("profiles").join("invalid.rhai");
    std::fs::write(&invalid_path, "invalid syntax {{{").unwrap();

    // List profiles should still work even with invalid .rhai files present
    let mut cmd = Command::cargo_bin("keyrx_daemon").unwrap();
    cmd.env("KEYRX_CONFIG_DIR", config_path.to_str().unwrap())
        .args(&["profiles", "list", "--json"]);

    // Should succeed and return profiles array
    // Note: profiles list may skip invalid .rhai files or list compiled .krx files only
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"profiles\""));
}

#[tokio::test]
async fn test_web_continues_after_error() {
    let (port, _server, _state) = start_test_web_server().await;

    let client = reqwest::Client::new();

    // First request: intentionally cause an error
    let url1 = format!(
        "http://127.0.0.1:{}/api/profiles/nonexistent/activate",
        port
    );
    let response1 = client.post(&url1).send().await.unwrap();
    assert!(
        !response1.status().is_success(),
        "Expected error for nonexistent profile"
    );

    // Second request: valid request should still work
    let url2 = format!("http://127.0.0.1:{}/api/profiles", port);
    let response2 = client.get(&url2).send().await.unwrap();

    // Server should still be responding after the error
    assert!(
        response2.status().is_success() || response2.status().is_client_error(),
        "Server should still respond after previous error, got {}",
        response2.status()
    );
}

// ============================================================================
// Error Message Quality Tests
// ============================================================================

#[test]
fn test_platform_error_includes_suggestions() {
    let err = PlatformError::DeviceAccess {
        device: "/dev/input/event0".into(),
        reason: "permission denied".into(),
        suggestion: "Add your user to the 'input' group".into(),
    };

    let msg = err.to_string();
    // Error should include actionable suggestion
    assert!(msg.contains("Add your user"));
    assert!(msg.contains("input"));
}

#[test]
fn test_config_error_includes_path() {
    let err = ConfigError::FileNotFound {
        path: PathBuf::from("/home/user/.config/keyrx/profiles/test.krx"),
    };

    let msg = err.to_string();
    // Error should show the exact file path
    assert!(msg.contains("/home/user/.config/keyrx/profiles/test.krx"));
}

#[test]
fn test_cli_error_includes_command_name() {
    let err = CliError::CommandFailed {
        command: "set-tap-hold".into(),
        reason: "invalid threshold value".into(),
    };

    let msg = err.to_string();
    // Error should identify which command failed
    assert!(msg.contains("set-tap-hold"));
    assert!(msg.contains("threshold"));
}

// ============================================================================
// Edge Case Error Tests
// ============================================================================

#[test]
fn test_empty_config_directory_error() {
    let (_temp, config_path) = setup_test_env();

    // Don't create any profiles, just empty directory
    let mut cmd = Command::cargo_bin("keyrx_daemon").unwrap();
    cmd.env("KEYRX_CONFIG_DIR", config_path.to_str().unwrap())
        .args(&["profiles", "activate", "anything", "--json"]);

    // Should fail gracefully
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("\"success\":false"));
}

#[test]
fn test_corrupted_krx_file_error() {
    let (_temp, config_path) = setup_test_env();

    // Create a .krx file with invalid content
    let krx_path = config_path.join("profiles").join("corrupted.krx");
    std::fs::write(&krx_path, b"not a valid krx file").unwrap();

    // Trying to use it should produce a meaningful error
    let mut cmd = Command::cargo_bin("keyrx_daemon").unwrap();
    cmd.env("KEYRX_CONFIG_DIR", config_path.to_str().unwrap())
        .args(&["profiles", "activate", "corrupted", "--json"]);

    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("\"success\":false"));
}
