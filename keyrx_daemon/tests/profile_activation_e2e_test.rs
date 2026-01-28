//! End-to-end tests for profile activation via REST API and Web UI workflow.
//!
//! These tests verify the complete profile activation flow:
//! - Creating profiles via POST /api/profiles
//! - Activating profiles via POST /api/profiles/:name/activate
//! - Verifying activation persistence via GET /api/profiles/active
//! - Checking active status in profile listings
//! - Testing activation across ProfileManager reloads (daemon restart simulation)
//!
//! **Important**: These tests must be run serially because they modify the
//! global HOME environment variable to create isolated test directories.
//!
//! Run with: `cargo test -p keyrx_daemon --test profile_activation_e2e_test -- --test-threads=1`

mod common;

use common::test_app::TestApp;
use serde_json::json;
use serial_test::serial;

/// Test complete web UI workflow: create profile → activate → verify active state.
///
/// This simulates the exact user workflow:
/// 1. POST /api/profiles - Create new profile
/// 2. POST /api/profiles/:name/activate - Activate the profile
/// 3. GET /api/profiles/active - Verify it's marked as active
/// 4. GET /api/profiles - Verify profile list shows it as active (isActive: true)
#[tokio::test]
#[serial]
async fn test_create_and_activate_profile_via_api() {
    let app = TestApp::new().await;

    // Step 1: Create a new profile
    let create_response = app
        .post(
            "/api/profiles",
            &json!({
                "name": "test-profile",
                "template": "blank"
            }),
        )
        .await;

    let create_status = create_response.status();
    let create_body = create_response.text().await.unwrap();

    assert!(
        create_status.is_success(),
        "Profile creation should succeed. Status: {}, Body: {}",
        create_status,
        create_body
    );

    // Step 2: Activate the profile
    let activate_response = app
        .post("/api/profiles/test-profile/activate", &json!({}))
        .await;

    let activate_status = activate_response.status();
    let activate_body = activate_response.text().await.unwrap();

    assert!(
        activate_status.is_success(),
        "Profile activation should succeed. Status: {}, Body: {}",
        activate_status,
        activate_body
    );

    // Parse activation response
    let activate_json: serde_json::Value =
        serde_json::from_str(&activate_body).expect("Activation response should be valid JSON");

    assert_eq!(
        activate_json["success"], true,
        "Activation should report success"
    );
    assert_eq!(
        activate_json["profile"], "test-profile",
        "Activation should return profile name"
    );

    // Step 3: Verify via GET /api/profiles/active
    let active_response = app.get("/api/profiles/active").await;
    let active_body = active_response.text().await.unwrap();
    let active_json: serde_json::Value = serde_json::from_str(&active_body).unwrap();

    assert_eq!(
        active_json["active_profile"], "test-profile",
        "GET /api/profiles/active should return the activated profile name"
    );

    // Step 4: Verify profile list shows active indicator
    let list_response = app.get("/api/profiles").await;
    let list_body = list_response.text().await.unwrap();
    let list_json: serde_json::Value = serde_json::from_str(&list_body).unwrap();

    let profiles = list_json["profiles"]
        .as_array()
        .expect("Response should have profiles array");

    let test_profile = profiles
        .iter()
        .find(|p| p["name"] == "test-profile")
        .expect("test-profile should be in the list");

    assert_eq!(
        test_profile["isActive"], true,
        "Profile should have isActive=true in the list"
    );
}

/// Test activation persistence across ProfileManager reloads.
///
/// This simulates daemon restart by creating a new ProfileManager instance
/// with the same config directory. The active profile should be restored.
#[tokio::test]
#[serial]
async fn test_activation_persists_across_reload() {
    let app = TestApp::new().await;

    // Create and activate a profile
    app.post(
        "/api/profiles",
        &json!({
            "name": "persistent-profile",
            "template": "simple_remap"
        }),
    )
    .await;

    let activate_response = app
        .post("/api/profiles/persistent-profile/activate", &json!({}))
        .await;

    assert!(
        activate_response.status().is_success(),
        "Initial activation should succeed"
    );

    // Verify active profile via API
    let active_response = app.get("/api/profiles/active").await;
    let active_body = active_response.text().await.unwrap();
    let active_json: serde_json::Value = serde_json::from_str(&active_body).unwrap();

    assert_eq!(
        active_json["active_profile"], "persistent-profile",
        "Profile should be active before reload"
    );

    // Simulate daemon restart by creating a new ProfileManager with same config dir
    // Note: TestApp uses the same config_dir for the lifetime of the test
    use keyrx_daemon::config::ProfileManager;

    let config_path = app.config_path();
    let reloaded_pm =
        ProfileManager::new(config_path.clone()).expect("ProfileManager reload should succeed");

    let restored_active = reloaded_pm
        .get_active()
        .expect("Should successfully get active profile")
        .expect("Active profile should be restored");

    assert_eq!(
        restored_active, "persistent-profile",
        "Active profile should be restored after ProfileManager reload"
    );

    // Double-check via API after reload
    let active_response2 = app.get("/api/profiles/active").await;
    let active_body2 = active_response2.text().await.unwrap();
    let active_json2: serde_json::Value = serde_json::from_str(&active_body2).unwrap();

    assert_eq!(
        active_json2["active_profile"], "persistent-profile",
        "Active profile should still be returned via API after reload"
    );
}

/// Test switching between profiles.
///
/// Verifies that activating a different profile correctly updates the active state.
#[tokio::test]
#[serial]
async fn test_switch_active_profile() {
    let app = TestApp::new().await;

    // Create two profiles
    app.post(
        "/api/profiles",
        &json!({ "name": "profile-a", "template": "blank" }),
    )
    .await;

    app.post(
        "/api/profiles",
        &json!({ "name": "profile-b", "template": "simple_remap" }),
    )
    .await;

    // Activate profile-a
    let activate_a = app
        .post("/api/profiles/profile-a/activate", &json!({}))
        .await;
    assert!(activate_a.status().is_success());

    // Verify profile-a is active
    let active1 = app.get("/api/profiles/active").await;
    let active1_json: serde_json::Value =
        serde_json::from_str(&active1.text().await.unwrap()).unwrap();
    assert_eq!(active1_json["active_profile"], "profile-a");

    // Activate profile-b (switch)
    let activate_b = app
        .post("/api/profiles/profile-b/activate", &json!({}))
        .await;
    assert!(activate_b.status().is_success());

    // Verify profile-b is now active
    let active2 = app.get("/api/profiles/active").await;
    let active2_json: serde_json::Value =
        serde_json::from_str(&active2.text().await.unwrap()).unwrap();
    assert_eq!(
        active2_json["active_profile"], "profile-b",
        "Active profile should switch to profile-b"
    );

    // Verify profile list shows correct active states
    let list_response = app.get("/api/profiles").await;
    let list_json: serde_json::Value =
        serde_json::from_str(&list_response.text().await.unwrap()).unwrap();

    let profiles = list_json["profiles"].as_array().unwrap();

    let profile_a = profiles.iter().find(|p| p["name"] == "profile-a").unwrap();
    let profile_b = profiles.iter().find(|p| p["name"] == "profile-b").unwrap();

    assert_eq!(
        profile_a["isActive"], false,
        "profile-a should no longer be active"
    );
    assert_eq!(profile_b["isActive"], true, "profile-b should be active");
}

/// Test deleting the active profile clears the active state.
///
/// When a user deletes the currently active profile, there should be no active profile.
#[tokio::test]
#[serial]
async fn test_delete_active_profile_clears_state() {
    let app = TestApp::new().await;

    // Create and activate a profile
    app.post(
        "/api/profiles",
        &json!({ "name": "to-delete", "template": "blank" }),
    )
    .await;

    app.post("/api/profiles/to-delete/activate", &json!({}))
        .await;

    // Verify it's active
    let active1 = app.get("/api/profiles/active").await;
    let active1_json: serde_json::Value =
        serde_json::from_str(&active1.text().await.unwrap()).unwrap();
    assert_eq!(active1_json["active_profile"], "to-delete");

    // Delete the active profile
    let delete_response = app.delete("/api/profiles/to-delete").await;
    assert!(
        delete_response.status().is_success(),
        "Profile deletion should succeed"
    );

    // Verify no active profile now
    let active2 = app.get("/api/profiles/active").await;
    let active2_json: serde_json::Value =
        serde_json::from_str(&active2.text().await.unwrap()).unwrap();
    assert!(
        active2_json["active_profile"].is_null(),
        "Active profile should be null after deleting the active profile"
    );
}

/// Test activating a non-existent profile returns error.
///
/// Attempting to activate a profile that doesn't exist should return 404 or 500.
#[tokio::test]
#[serial]
async fn test_activate_nonexistent_profile_returns_error() {
    let app = TestApp::new().await;

    let activate_response = app
        .post("/api/profiles/nonexistent-profile/activate", &json!({}))
        .await;

    assert!(
        activate_response.status().is_client_error()
            || activate_response.status().is_server_error(),
        "Activating non-existent profile should return error status"
    );
}

/// Test GET /api/profiles/active returns null when no profile is active.
///
/// On fresh installation or after deleting all profiles, there should be no active profile.
#[tokio::test]
#[serial]
async fn test_get_active_profile_when_none_active() {
    let app = TestApp::new().await;

    let response = app.get("/api/profiles/active").await;
    let json: serde_json::Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();

    assert!(
        json["active_profile"].is_null(),
        "active_profile should be null when no profile is active"
    );
}

/// Test that all profiles have isActive=false when none is active.
///
/// The profile list should show all profiles as inactive when no profile is active.
#[tokio::test]
#[serial]
async fn test_profile_list_shows_none_active() {
    let app = TestApp::new().await;

    // Create two profiles but don't activate either
    app.post(
        "/api/profiles",
        &json!({ "name": "profile-x", "template": "blank" }),
    )
    .await;

    app.post(
        "/api/profiles",
        &json!({ "name": "profile-y", "template": "blank" }),
    )
    .await;

    // Get profile list
    let list_response = app.get("/api/profiles").await;
    let list_json: serde_json::Value =
        serde_json::from_str(&list_response.text().await.unwrap()).unwrap();

    let profiles = list_json["profiles"].as_array().unwrap();

    // All profiles should have isActive=false
    for profile in profiles {
        assert_eq!(
            profile["isActive"], false,
            "Profile {} should not be active",
            profile["name"]
        );
    }
}

/// Test activation with invalid Rhai syntax returns compilation error.
///
/// If a profile's .rhai file has invalid syntax, activation should fail with error details.
#[tokio::test]
#[serial]
async fn test_activation_fails_for_invalid_syntax() {
    use std::fs;

    let app = TestApp::new().await;

    // Create profile with valid template first
    app.post(
        "/api/profiles",
        &json!({ "name": "broken-profile", "template": "blank" }),
    )
    .await;

    // Manually corrupt the .rhai file with invalid syntax
    let profiles_dir = app.config_path().join("profiles");
    let rhai_path = profiles_dir.join("broken-profile.rhai");

    fs::write(
        &rhai_path,
        "device_start(\"*\");\n// Missing device_end() - invalid!",
    )
    .expect("Failed to write corrupted profile");

    // Attempt to activate the broken profile
    let activate_response = app
        .post("/api/profiles/broken-profile/activate", &json!({}))
        .await;

    let status = activate_response.status();
    let body = activate_response.text().await.unwrap();

    assert!(
        status.is_client_error() || status.is_server_error(),
        "Activation of broken profile should fail. Status: {}, Body: {}",
        status,
        body
    );

    // Response should indicate failure
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
        // If it's JSON, check for success=false
        if json.get("success").is_some() {
            assert_eq!(
                json["success"], false,
                "Response should indicate compilation failure"
            );
        }
    }
}

/// Test activation timing metadata is returned.
///
/// The activation response should include compile_time_ms and reload_time_ms.
#[tokio::test]
#[serial]
async fn test_activation_returns_timing_metadata() {
    let app = TestApp::new().await;

    // Create and activate a profile
    app.post(
        "/api/profiles",
        &json!({ "name": "timed-profile", "template": "simple_remap" }),
    )
    .await;

    let activate_response = app
        .post("/api/profiles/timed-profile/activate", &json!({}))
        .await;

    assert!(activate_response.status().is_success());

    let json: serde_json::Value =
        serde_json::from_str(&activate_response.text().await.unwrap()).unwrap();

    // Check timing fields exist and are numbers
    assert!(
        json["compile_time_ms"].is_number(),
        "Response should include compile_time_ms"
    );
    assert!(
        json["reload_time_ms"].is_number(),
        "Response should include reload_time_ms"
    );

    // Timing should be reasonable (< 10 seconds for simple profile)
    let compile_time = json["compile_time_ms"].as_u64().unwrap();
    assert!(
        compile_time < 10000,
        "Compile time should be reasonable: {}ms",
        compile_time
    );
}

/// Test concurrent activation requests are serialized correctly.
///
/// Multiple simultaneous activation requests should not corrupt the active profile state.
#[tokio::test]
#[serial]
async fn test_concurrent_activations_are_serialized() {
    let app = TestApp::new().await;

    // Create multiple profiles
    for i in 1..=3 {
        app.post(
            "/api/profiles",
            &json!({
                "name": format!("concurrent-{}", i),
                "template": "blank"
            }),
        )
        .await;
    }

    // Send concurrent activation requests
    let mut handles = vec![];

    for i in 1..=3 {
        let app_clone = app.clone_client();
        let profile_name = format!("concurrent-{}", i);
        let handle = tokio::spawn(async move {
            app_clone
                .post(
                    &format!("/api/profiles/{}/activate", profile_name),
                    &json!({}),
                )
                .await
        });
        handles.push((i, handle));
    }

    // Wait for all activations to complete
    let mut success_count = 0;
    for (_i, handle) in handles {
        let response = handle.await.unwrap();
        if response.status().is_success() {
            success_count += 1;
        }
    }

    // At least one should succeed (likely all, since they're serialized by the lock)
    assert!(
        success_count >= 1,
        "At least one concurrent activation should succeed"
    );

    // Verify exactly one profile is active
    let active_response = app.get("/api/profiles/active").await;
    let active_json: serde_json::Value =
        serde_json::from_str(&active_response.text().await.unwrap()).unwrap();

    let active_profile = active_json["active_profile"]
        .as_str()
        .expect("Should have an active profile");

    // Active profile should be one of the ones we tried to activate
    assert!(
        active_profile.starts_with("concurrent-"),
        "Active profile should be one of the concurrent profiles: {}",
        active_profile
    );

    // Verify profile list shows exactly one active
    let list_response = app.get("/api/profiles").await;
    let list_json: serde_json::Value =
        serde_json::from_str(&list_response.text().await.unwrap()).unwrap();

    let active_count = list_json["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|p| p["isActive"] == true)
        .count();

    assert_eq!(
        active_count, 1,
        "Exactly one profile should be marked active after concurrent activations"
    );
}
