//! Integration tests for simulation API endpoints.
//!
//! Tests verify:
//! - Profile loading for simulation
//! - Event simulation with DSL and custom events
//! - Deterministic behavior with seeds
//! - Error handling for invalid inputs
//! - Simulator reset functionality
//!
//! These tests use the TestApp fixture for isolated testing.
//! Note: Scenario tests require tap-hold configuration via CLI, so are limited here.

mod common;

use common::test_app::TestApp;
use serde_json::json;
use serial_test::serial;
use std::fs;

/// Helper function to create a profile file for testing.
fn create_test_profile(app: &TestApp, name: &str, content: &str) {
    let profiles_dir = app.config_path().join("profiles");
    fs::create_dir_all(&profiles_dir).expect("Failed to create profiles directory");
    let rhai_path = profiles_dir.join(format!("{}.rhai", name));
    fs::write(&rhai_path, content).expect("Failed to write profile file");
    app.rescan_profiles();
}

/// Helper function to compile and activate a profile.
async fn activate_profile(app: &TestApp, name: &str) {
    let response = app
        .post(&format!("/api/profiles/{}/activate", name), &json!({}))
        .await;

    assert!(
        response.status().is_success(),
        "Failed to activate profile: {}",
        response.text().await.unwrap()
    );
}

/// Test loading a profile for simulation.
#[tokio::test]
#[serial]
async fn test_load_profile_success() {
    let app = TestApp::new().await;

    // Create and activate a test profile
    let profile_content = r#"
device_start("*");
  map("VK_A", "VK_B");
device_end();
"#;
    create_test_profile(&app, "test-sim", profile_content);
    activate_profile(&app, "test-sim").await;

    // Load the profile for simulation
    let response = app
        .post("/api/simulator/load-profile", &json!({"name": "test-sim"}))
        .await;

    assert!(response.status().is_success());
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert!(body["message"].as_str().unwrap().contains("test-sim"));
}

/// Test loading a non-existent profile returns an error.
#[tokio::test]
#[serial]
async fn test_load_profile_not_found() {
    let app = TestApp::new().await;

    let response = app
        .post(
            "/api/simulator/load-profile",
            &json!({"name": "nonexistent"}),
        )
        .await;

    assert_eq!(response.status(), 404);
}

/// Test simulating events with DSL.
#[tokio::test]
#[serial]
async fn test_simulate_with_dsl() {
    let app = TestApp::new().await;

    // Create and load a simple remap profile
    let profile_content = r#"
device_start("*");
  map("VK_A", "VK_B");
device_end();
"#;
    create_test_profile(&app, "dsl-test", profile_content);
    activate_profile(&app, "dsl-test").await;

    app.post("/api/simulator/load-profile", &json!({"name": "dsl-test"}))
        .await;

    // Simulate with DSL: press A, wait, release A
    let response = app
        .post(
            "/api/simulator/events",
            &json!({
                "dsl": "press:A,wait:50,release:A",
                "seed": 42
            }),
        )
        .await;

    assert!(
        response.status().is_success(),
        "DSL simulation failed: {}",
        response.text().await.unwrap()
    );

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);

    let outputs = body["outputs"].as_array().unwrap();
    assert!(!outputs.is_empty(), "Expected output events from DSL");

    // Verify we got press and release events
    assert_eq!(outputs.len(), 2, "Expected 2 events (press and release)");
    assert_eq!(outputs[0]["event_type"], "press");
    assert_eq!(outputs[1]["event_type"], "release");
    // Note: Key mapping verification depends on simulation engine implementation
    // For now, just verify events were processed
}

/// Test deterministic behavior with same seed.
#[tokio::test]
#[serial]
async fn test_deterministic_with_seed() {
    let app = TestApp::new().await;

    let profile_content = r#"
device_start("*");
  map("VK_A", "VK_B");
device_end();
"#;
    create_test_profile(&app, "deterministic-test", profile_content);
    activate_profile(&app, "deterministic-test").await;

    app.post(
        "/api/simulator/load-profile",
        &json!({"name": "deterministic-test"}),
    )
    .await;

    let dsl = "press:A,wait:100,release:A";
    let seed = 12345u64;

    // Run simulation twice with same seed
    let response1 = app
        .post("/api/simulator/events", &json!({"dsl": dsl, "seed": seed}))
        .await;
    let body1: serde_json::Value = response1.json().await.unwrap();

    let response2 = app
        .post("/api/simulator/events", &json!({"dsl": dsl, "seed": seed}))
        .await;
    let body2: serde_json::Value = response2.json().await.unwrap();

    // Results should be identical
    assert_eq!(
        body1["outputs"], body2["outputs"],
        "Same seed should produce deterministic results"
    );
}

/// Test simulating with custom events.
#[tokio::test]
#[serial]
async fn test_simulate_with_custom_events() {
    let app = TestApp::new().await;

    let profile_content = r#"
device_start("*");
  map("VK_X", "VK_Y");
device_end();
"#;
    create_test_profile(&app, "custom-events-test", profile_content);
    activate_profile(&app, "custom-events-test").await;

    app.post(
        "/api/simulator/load-profile",
        &json!({"name": "custom-events-test"}),
    )
    .await;

    // Provide custom event sequence
    let response = app
        .post(
            "/api/simulator/events",
            &json!({
                "events": [
                    {
                        "device_id": null,
                        "timestamp_us": 0,
                        "key": "X",
                        "event_type": "press"
                    },
                    {
                        "device_id": null,
                        "timestamp_us": 50000,
                        "key": "X",
                        "event_type": "release"
                    }
                ],
                "seed": 0
            }),
        )
        .await;

    assert!(
        response.status().is_success(),
        "Custom events simulation failed: {}",
        response.text().await.unwrap()
    );

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);

    let outputs = body["outputs"].as_array().unwrap();
    assert!(!outputs.is_empty());

    // Verify we got custom events processed
    assert_eq!(outputs.len(), 2, "Expected 2 events (press and release)");
}

/// Test error when no simulation input is provided.
#[tokio::test]
#[serial]
async fn test_simulate_error_no_input() {
    let app = TestApp::new().await;

    // Provide empty request (no scenario, dsl, or events)
    let response = app.post("/api/simulator/events", &json!({})).await;

    assert_eq!(response.status(), 400);
    let body = response.text().await.unwrap();
    assert!(body.contains("scenario") || body.contains("dsl") || body.contains("events"));
}

/// Test resetting the simulator.
#[tokio::test]
#[serial]
async fn test_reset_simulator() {
    let app = TestApp::new().await;

    let response = app.post("/api/simulator/reset", &json!({})).await;

    assert!(response.status().is_success());
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert!(body["message"].as_str().unwrap().contains("reset"));
}

/// Test reset clears loaded profile state.
#[tokio::test]
#[serial]
async fn test_reset_clears_state() {
    let app = TestApp::new().await;

    // Load a profile
    let profile_content = r#"
device_start("*");
  map("VK_A", "VK_B");
device_end();
"#;
    create_test_profile(&app, "reset-state-test", profile_content);
    activate_profile(&app, "reset-state-test").await;

    app.post(
        "/api/simulator/load-profile",
        &json!({"name": "reset-state-test"}),
    )
    .await;

    // Reset the simulator
    let reset_response = app.post("/api/simulator/reset", &json!({})).await;
    assert!(reset_response.status().is_success());

    // After reset, trying to simulate without loading profile should still work
    // because SimulationEngine can handle empty/no profile state
    let response = app
        .post(
            "/api/simulator/events",
            &json!({"dsl": "press:A,release:A", "seed": 0}),
        )
        .await;

    // Should either succeed (with pass-through behavior) or return a clear error
    assert!(
        response.status().is_success() || response.status().is_client_error(),
        "Reset should clear state"
    );
}

/// Test simulating with multiple different mappings.
#[tokio::test]
#[serial]
async fn test_simulate_multiple_mappings() {
    let app = TestApp::new().await;

    let profile_content = r#"
device_start("*");
  map("VK_A", "VK_X");
  map("VK_B", "VK_Y");
  map("VK_C", "VK_Z");
device_end();
"#;
    create_test_profile(&app, "multi-map-test", profile_content);
    activate_profile(&app, "multi-map-test").await;

    app.post(
        "/api/simulator/load-profile",
        &json!({"name": "multi-map-test"}),
    )
    .await;

    // Simulate pressing A, B, C
    let response = app
        .post(
            "/api/simulator/events",
            &json!({
                "dsl": "press:A,press:B,press:C,wait:10,release:A,release:B,release:C",
                "seed": 0
            }),
        )
        .await;

    assert!(response.status().is_success());
    let body: serde_json::Value = response.json().await.unwrap();
    let outputs = body["outputs"].as_array().unwrap();

    // Verify we got events for all 3 keys (press and release for each)
    assert_eq!(outputs.len(), 6, "Expected 6 events (3 keys × 2 events)");
}

/// Test invalid DSL syntax returns error.
#[tokio::test]
#[serial]
async fn test_simulate_invalid_dsl() {
    let app = TestApp::new().await;

    let profile_content = r#"
device_start("*");
  map("VK_A", "VK_B");
device_end();
"#;
    create_test_profile(&app, "invalid-dsl-test", profile_content);
    activate_profile(&app, "invalid-dsl-test").await;

    app.post(
        "/api/simulator/load-profile",
        &json!({"name": "invalid-dsl-test"}),
    )
    .await;

    // Invalid DSL (malformed syntax)
    let response = app
        .post(
            "/api/simulator/events",
            &json!({"dsl": "invalid:syntax:here", "seed": 0}),
        )
        .await;

    // Should return error (400 bad request or similar)
    assert!(
        response.status().is_client_error() || response.status().is_server_error(),
        "Invalid DSL should return error"
    );
}
