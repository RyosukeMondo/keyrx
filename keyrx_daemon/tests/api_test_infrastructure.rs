//! Integration tests for test infrastructure.
//!
//! This file verifies that the TestApp fixture works correctly
//! and can be used for API integration testing.

mod common;

use common::test_app::TestApp;

#[tokio::test]
async fn test_infrastructure_creates_isolated_app() {
    let app = TestApp::new().await;
    let config_path = app.config_path();

    // Verify temp directory exists and is isolated
    assert!(config_path.exists());
    assert!(config_path.is_dir());

    // Verify we can make HTTP requests
    let response = app.get("/api/status").await;
    // Server should respond (status code exists)
    assert!(response.status().as_u16() > 0);
}

#[tokio::test]
async fn test_infrastructure_supports_parallel_tests() {
    // Create multiple apps concurrently
    let (app1, app2, app3) = tokio::join!(TestApp::new(), TestApp::new(), TestApp::new());

    // Verify all have different config directories
    assert_ne!(app1.config_path(), app2.config_path());
    assert_ne!(app2.config_path(), app3.config_path());
    assert_ne!(app1.config_path(), app3.config_path());

    // Verify all have different ports
    assert_ne!(app1.base_url, app2.base_url);
    assert_ne!(app2.base_url, app3.base_url);
    assert_ne!(app1.base_url, app3.base_url);
}

#[tokio::test]
async fn test_http_helpers_work() {
    let app = TestApp::new().await;

    // Test GET
    let response = app.get("/api/status").await;
    assert!(response.status().as_u16() > 0);

    // Test POST (even if endpoint doesn't exist, should get response)
    let body = serde_json::json!({"test": "data"});
    let response = app.post("/api/test", &body).await;
    assert!(response.status().as_u16() > 0);

    // Test PATCH
    let response = app.patch("/api/test/123", &body).await;
    assert!(response.status().as_u16() > 0);

    // Test DELETE
    let response = app.delete("/api/test/123").await;
    assert!(response.status().as_u16() > 0);
}
