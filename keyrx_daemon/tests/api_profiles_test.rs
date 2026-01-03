//! Profile validation integration tests.
//!
//! These tests verify that profile compilation and validation work correctly:
//! - Valid Rhai profile templates compile successfully
//! - Invalid Rhai syntax is rejected with compilation errors
//! - Compilation errors include helpful messages
//!
//! **Important**: These tests must be run serially because they modify the
//! global HOME environment variable to create isolated test directories.
//!
//! Run with: `cargo test -p keyrx_daemon --test api_profiles_test -- --test-threads=1`

mod common;

use common::test_app::TestApp;
use serde_json::json;
use std::fs;

/// Helper function to create a profile file directly in the filesystem.
///
/// This bypasses the API's create endpoint and directly writes the profile file,
/// which allows us to test validation independently.
fn create_profile_file(app: &TestApp, name: &str, content: &str) {
    let profiles_dir = app.config_path().join("profiles");
    fs::create_dir_all(&profiles_dir).expect("Failed to create profiles directory");

    let rhai_path = profiles_dir.join(format!("{}.rhai", name));
    fs::write(&rhai_path, content).expect("Failed to write profile file");
}

/// Test that a valid profile template compiles successfully.
///
/// This test creates a profile with valid Rhai syntax and verifies that:
/// 1. The profile can be activated via POST /api/profiles/:name/activate
/// 2. Activation returns success with compilation metadata
#[tokio::test]
async fn test_valid_profile_compiles_successfully() {
    let app = TestApp::new().await;

    // Create profile file with valid Rhai configuration
    let valid_config = r#"
// Valid Rhai configuration
device_start("*");
  map("VK_A", "VK_B");
device_end();
"#;

    create_profile_file(&app, "test-valid", valid_config);

    // Activate the profile to trigger compilation
    let activate_response = app
        .post("/api/profiles/test-valid/activate", &json!({}))
        .await;

    let status = activate_response.status();
    let body = activate_response.text().await.unwrap();

    assert!(
        status.is_success(),
        "Valid profile should compile successfully. Status: {}, Body: {}",
        status,
        body
    );

    // Parse response to verify compilation metadata
    let response_json: serde_json::Value = serde_json::from_str(&body).unwrap();

    assert_eq!(
        response_json["success"], true,
        "Response should indicate success"
    );
    assert!(
        response_json["compile_time_ms"].is_number(),
        "Response should include compile_time_ms"
    );
}

/// Test that an invalid profile template returns compilation errors.
///
/// This test verifies that profiles with invalid Rhai syntax are rejected
/// during activation and return error messages.
#[tokio::test]
async fn test_invalid_profile_returns_compilation_errors() {
    let app = TestApp::new().await;

    // Create profile with invalid Rhai configuration (unclosed device_start)
    let invalid_config = r#"
// Invalid Rhai configuration - missing device_end()
device_start("*");
  map("VK_A", "VK_B");
// device_end() is missing!
"#;

    create_profile_file(&app, "test-invalid", invalid_config);

    // Attempt to activate the invalid profile
    let activate_response = app
        .post("/api/profiles/test-invalid/activate", &json!({}))
        .await;

    let status = activate_response.status();
    let body = activate_response.text().await.unwrap();

    // Should fail with client error (4xx) or server error (5xx)
    assert!(
        status.is_client_error() || status.is_server_error(),
        "Invalid profile should fail to activate. Status: {}, Body: {}",
        status,
        body
    );

    // Response should contain error information
    assert!(
        !body.is_empty(),
        "Error response should not be empty. Body: {}",
        body
    );
}

/// Test that profile validation catches the old layer() syntax.
///
/// This test verifies that profiles using the deprecated layer() function
/// (instead of device_start/device_end) are rejected with clear error messages.
#[tokio::test]
async fn test_invalid_layer_syntax_rejected() {
    let app = TestApp::new().await;

    // Create profile using invalid layer() syntax
    let invalid_layer_config = r#"
// Invalid: Using old layer() syntax instead of device_start/device_end
layer("base", "*");
map("VK_A", "VK_B");
"#;

    create_profile_file(&app, "test-layer-syntax", invalid_layer_config);

    // Attempt to activate profile with invalid syntax
    let activate_response = app
        .post("/api/profiles/test-layer-syntax/activate", &json!({}))
        .await;

    let status = activate_response.status();
    let body = activate_response.text().await.unwrap();

    // Should fail with error
    assert!(
        status.is_client_error() || status.is_server_error(),
        "Profile with layer() syntax should fail. Status: {}, Body: {}",
        status,
        body
    );

    // Error message should mention the compilation failure
    assert!(
        !body.is_empty(),
        "Error response should contain error details"
    );
}

/// Test that profile activation provides helpful error messages.
///
/// This test verifies that compilation errors include context that helps
/// users understand what went wrong.
#[tokio::test]
async fn test_compilation_errors_are_helpful() {
    let app = TestApp::new().await;

    // Create profile with clear syntax error (undefined function)
    let invalid_config = r#"
device_start("*");
  undefined_function("VK_A", "VK_B");
device_end();
"#;

    create_profile_file(&app, "test-helpful-errors", invalid_config);

    // Activate to trigger compilation
    let activate_response = app
        .post("/api/profiles/test-helpful-errors/activate", &json!({}))
        .await;

    let status = activate_response.status();
    let body = activate_response.text().await.unwrap();

    // Should fail
    assert!(
        status.is_client_error() || status.is_server_error(),
        "Invalid function should cause compilation failure"
    );

    // Error message should not be empty
    assert!(!body.is_empty(), "Error should include helpful message");

    // Error should contain some indication of what failed
    assert!(
        body.len() > 10,
        "Error message should be reasonably detailed, got: {}",
        body
    );
}

/// Test that multiple valid profiles can be compiled independently.
///
/// This test verifies that the compilation system can handle multiple
/// profiles without cross-contamination.
#[tokio::test]
async fn test_multiple_profiles_compile_independently() {
    let app = TestApp::new().await;

    // Create two profiles with different configurations
    let config1 = r#"
device_start("*");
  map("VK_A", "VK_B");
device_end();
"#;

    let config2 = r#"
device_start("*");
  map("VK_Q", "VK_W");
device_end();
"#;

    create_profile_file(&app, "profile-1", config1);
    create_profile_file(&app, "profile-2", config2);

    // Activate first profile
    let activate1 = app
        .post("/api/profiles/profile-1/activate", &json!({}))
        .await;
    let status1 = activate1.status();
    let body1 = activate1.text().await.unwrap();

    assert!(
        status1.is_success(),
        "Profile 1 should compile. Status: {}, Body: {}",
        status1,
        body1
    );

    // Activate second profile
    let activate2 = app
        .post("/api/profiles/profile-2/activate", &json!({}))
        .await;
    let status2 = activate2.status();
    let body2 = activate2.text().await.unwrap();

    assert!(
        status2.is_success(),
        "Profile 2 should compile. Status: {}, Body: {}",
        status2,
        body2
    );
}

/// Test that valid profile with comments and whitespace compiles.
///
/// This test verifies that the compiler correctly handles comments and
/// various whitespace formatting.
#[tokio::test]
async fn test_profile_with_comments_and_whitespace() {
    let app = TestApp::new().await;

    let config_with_comments = r#"
// This is a comment
device_start("*");  // Inline comment

  // Remap A to B
  map("VK_A", "VK_B");

  // Remap Q to W
  map("VK_Q", "VK_W");

device_end();  // End of device block

// Final comment
"#;

    create_profile_file(&app, "test-comments", config_with_comments);

    let activate_response = app
        .post("/api/profiles/test-comments/activate", &json!({}))
        .await;

    let status = activate_response.status();
    let body = activate_response.text().await.unwrap();

    assert!(
        status.is_success(),
        "Profile with comments should compile. Status: {}, Body: {}",
        status,
        body
    );
}
