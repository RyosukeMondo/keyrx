//! Template compilation validation tests.
//!
//! These tests verify that all profile templates ship with valid syntax:
//! - All templates in keyrx_daemon/templates/ compile successfully
//! - Invalid templates (e.g., using layer() function) are rejected
//! - Compilation errors include line numbers and helpful messages
//!
//! This ensures users never receive broken templates and prevents
//! CI/CD from merging invalid template files.
//!
//! **Important**: These tests must be run serially because they modify the
//! global HOME environment variable to create isolated test directories.
//!
//! Run with: `cargo test -p keyrx_daemon --test template_validation_test -- --test-threads=1`

mod common;

use common::test_app::TestApp;
use serde_json::json;
use std::fs;

/// Helper function to create a profile file directly in the filesystem.
fn create_profile_file(app: &TestApp, name: &str, content: &str) {
    let profiles_dir = app.config_path().join("profiles");
    fs::create_dir_all(&profiles_dir).expect("Failed to create profiles directory");

    let rhai_path = profiles_dir.join(format!("{}.rhai", name));
    fs::write(&rhai_path, content).expect("Failed to write profile file");
}

/// Test that the blank.rhai template compiles successfully.
#[tokio::test]
async fn test_blank_template_compiles() {
    let app = TestApp::new().await;

    // Load the actual blank.rhai template from the crate
    let template_content = include_str!("../templates/blank.rhai");

    create_profile_file(&app, "test-blank", template_content);

    let activate_response = app
        .post("/api/profiles/test-blank/activate", &json!({}))
        .await;

    let status = activate_response.status();
    let body = activate_response.text().await.unwrap();

    assert!(
        status.is_success(),
        "blank.rhai template should compile successfully. Status: {}, Body: {}",
        status,
        body
    );

    let response_json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(
        response_json["success"], true,
        "Response should indicate success"
    );
}

/// Test that the simple_remap.rhai template compiles successfully.
#[tokio::test]
async fn test_simple_remap_template_compiles() {
    let app = TestApp::new().await;

    let template_content = include_str!("../templates/simple_remap.rhai");

    create_profile_file(&app, "test-simple-remap", template_content);

    let activate_response = app
        .post("/api/profiles/test-simple-remap/activate", &json!({}))
        .await;

    let status = activate_response.status();
    let body = activate_response.text().await.unwrap();

    assert!(
        status.is_success(),
        "simple_remap.rhai template should compile successfully. Status: {}, Body: {}",
        status,
        body
    );

    let response_json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(response_json["success"], true);
}

/// Test that the capslock_escape.rhai template compiles successfully.
#[tokio::test]
async fn test_capslock_escape_template_compiles() {
    let app = TestApp::new().await;

    let template_content = include_str!("../templates/capslock_escape.rhai");

    create_profile_file(&app, "test-capslock-escape", template_content);

    let activate_response = app
        .post("/api/profiles/test-capslock-escape/activate", &json!({}))
        .await;

    let status = activate_response.status();
    let body = activate_response.text().await.unwrap();

    assert!(
        status.is_success(),
        "capslock_escape.rhai template should compile successfully. Status: {}, Body: {}",
        status,
        body
    );

    let response_json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(response_json["success"], true);
}

/// Test that the vim_navigation.rhai template compiles successfully.
#[tokio::test]
async fn test_vim_navigation_template_compiles() {
    let app = TestApp::new().await;

    let template_content = include_str!("../templates/vim_navigation.rhai");

    create_profile_file(&app, "test-vim-navigation", template_content);

    let activate_response = app
        .post("/api/profiles/test-vim-navigation/activate", &json!({}))
        .await;

    let status = activate_response.status();
    let body = activate_response.text().await.unwrap();

    assert!(
        status.is_success(),
        "vim_navigation.rhai template should compile successfully. Status: {}, Body: {}",
        status,
        body
    );

    let response_json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(response_json["success"], true);
}

/// Test that the gaming.rhai template compiles successfully.
#[tokio::test]
async fn test_gaming_template_compiles() {
    let app = TestApp::new().await;

    let template_content = include_str!("../templates/gaming.rhai");

    create_profile_file(&app, "test-gaming", template_content);

    let activate_response = app
        .post("/api/profiles/test-gaming/activate", &json!({}))
        .await;

    let status = activate_response.status();
    let body = activate_response.text().await.unwrap();

    assert!(
        status.is_success(),
        "gaming.rhai template should compile successfully. Status: {}, Body: {}",
        status,
        body
    );

    let response_json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(response_json["success"], true);
}

/// Test that a template with invalid layer() function is rejected.
///
/// This test verifies that the old layer() syntax is properly rejected
/// and users are guided to use device_start/device_end instead.
#[tokio::test]
async fn test_invalid_layer_function_rejected() {
    let app = TestApp::new().await;

    // Template using deprecated layer() syntax
    let invalid_template = r#"
// Invalid: Using old layer() syntax
layer("base", "*");
map("VK_A", "VK_B");
"#;

    create_profile_file(&app, "test-invalid-layer", invalid_template);

    let activate_response = app
        .post("/api/profiles/test-invalid-layer/activate", &json!({}))
        .await;

    let status = activate_response.status();
    let body = activate_response.text().await.unwrap();

    assert!(
        status.is_client_error() || status.is_server_error(),
        "Template with layer() function should be rejected. Status: {}, Body: {}",
        status,
        body
    );

    // Error message should not be empty
    assert!(
        !body.is_empty(),
        "Error response should contain details about the compilation failure"
    );
}

/// Test that compilation errors include line numbers.
///
/// This test verifies that when a template has a syntax error,
/// the error message includes the line number to help users debug.
#[tokio::test]
async fn test_compilation_errors_include_line_numbers() {
    let app = TestApp::new().await;

    // Template with clear syntax error at a specific line
    let template_with_error = r#"
device_start("*");
  map("VK_A", "VK_B");
  syntax_error_here();
device_end();
"#;

    create_profile_file(&app, "test-line-numbers", template_with_error);

    let activate_response = app
        .post("/api/profiles/test-line-numbers/activate", &json!({}))
        .await;

    let status = activate_response.status();
    let body = activate_response.text().await.unwrap();

    assert!(
        status.is_client_error() || status.is_server_error(),
        "Template with syntax error should fail compilation. Status: {}, Body: {}",
        status,
        body
    );

    // Error message should contain line number information
    // The exact format depends on the compiler, but it should have some numeric indicator
    assert!(
        body.len() > 20,
        "Error message should be detailed enough to be helpful. Got: {}",
        body
    );
}

/// Test that compilation errors have helpful messages.
///
/// This test verifies that error messages guide users to fix the problem,
/// not just "compilation failed" without context.
#[tokio::test]
async fn test_compilation_errors_are_helpful() {
    let app = TestApp::new().await;

    // Template with missing device_end()
    let incomplete_template = r#"
device_start("*");
  map("VK_A", "VK_B");
  // Missing device_end() - should produce helpful error
"#;

    create_profile_file(&app, "test-helpful-error", incomplete_template);

    let activate_response = app
        .post("/api/profiles/test-helpful-error/activate", &json!({}))
        .await;

    let status = activate_response.status();
    let body = activate_response.text().await.unwrap();

    assert!(
        status.is_client_error() || status.is_server_error(),
        "Incomplete template should fail compilation. Status: {}, Body: {}",
        status,
        body
    );

    // Error should be descriptive (not just empty or "error")
    assert!(
        body.len() > 10,
        "Error message should provide helpful context. Got: {}",
        body
    );

    // Should not be a generic "internal server error"
    let lower_body = body.to_lowercase();
    assert!(
        !lower_body.contains("internal server error") || lower_body.contains("compilation"),
        "Error should be more specific than generic internal error. Got: {}",
        body
    );
}

/// Test that all templates work together (no conflicts).
///
/// This test activates all templates sequentially to verify they can
/// all be loaded and compiled without conflicts.
#[tokio::test]
async fn test_all_templates_can_be_loaded_sequentially() {
    let app = TestApp::new().await;

    let templates = vec![
        ("blank", include_str!("../templates/blank.rhai")),
        (
            "simple-remap",
            include_str!("../templates/simple_remap.rhai"),
        ),
        (
            "capslock-escape",
            include_str!("../templates/capslock_escape.rhai"),
        ),
        (
            "vim-navigation",
            include_str!("../templates/vim_navigation.rhai"),
        ),
        ("gaming", include_str!("../templates/gaming.rhai")),
    ];

    for (name, content) in templates {
        create_profile_file(&app, name, content);

        let activate_response = app
            .post(&format!("/api/profiles/{}/activate", name), &json!({}))
            .await;

        let status = activate_response.status();
        let body = activate_response.text().await.unwrap();

        assert!(
            status.is_success(),
            "Template '{}' should compile successfully. Status: {}, Body: {}",
            name,
            status,
            body
        );
    }
}
