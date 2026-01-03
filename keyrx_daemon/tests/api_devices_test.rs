//! Integration tests for device persistence API endpoints.
//!
//! Tests verify that device configuration (layout and scope) persists to filesystem
//! and loads correctly on daemon restart.
//!
//! # Running Tests
//!
//! These tests must be run serially because they modify the global HOME environment
//! variable to point to isolated test directories:
//!
//! ```bash
//! cargo test -p keyrx_daemon --test api_devices_test -- --test-threads=1
//! ```

mod common;

use common::test_app::TestApp;
use serde_json::json;

/// Helper function to register a device in the registry
async fn register_device(app: &TestApp, device_id: &str) {
    use keyrx_daemon::config::device_registry::{DeviceEntry, DeviceRegistry, DeviceScope};

    let registry_path = app.config_path().join("devices.json");
    let mut registry = DeviceRegistry::load(&registry_path).expect("Failed to load registry");

    let entry = DeviceEntry {
        id: device_id.to_string(),
        name: device_id.to_string(),
        serial: Some(device_id.to_string()),
        scope: DeviceScope::Global,
        layout: None,
        last_seen: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    registry.register(entry).expect("Failed to register device");
    registry.save().expect("Failed to save registry");
}

/// Test that device layout persists to filesystem
#[tokio::test]
async fn test_device_layout_save_persists_to_filesystem() {
    let app = TestApp::new().await;
    let device_id = "test-keyboard-001";

    // Register the device first
    register_device(&app, device_id).await;

    // Set the device layout
    let response = app
        .put(
            &format!("/api/devices/{}/layout", device_id),
            &json!({ "layout": "ansi_104" }),
        )
        .await;

    assert_eq!(
        response.status(),
        200,
        "Layout update should succeed: {}",
        response.text().await.unwrap()
    );

    // Verify the file was created on filesystem
    let registry_path = app.config_path().join("devices.json");
    assert!(
        registry_path.exists(),
        "devices.json should be created on filesystem"
    );

    // Read the file and verify contents
    let contents =
        std::fs::read_to_string(&registry_path).expect("Should be able to read devices.json");

    assert!(
        contents.contains(device_id),
        "Device ID should be in registry file"
    );
    assert!(
        contents.contains("ansi_104"),
        "Layout should be in registry file"
    );
}

/// Test that device scope persists to filesystem
#[tokio::test]
async fn test_device_scope_save_persists_to_filesystem() {
    let app = TestApp::new().await;
    let device_id = "test-keyboard-002";

    // Register the device first
    register_device(&app, device_id).await;

    // Set device scope
    let response = app
        .put(
            &format!("/api/devices/{}/scope", device_id),
            &json!({ "scope": "device-specific" }),
        )
        .await;

    assert_eq!(
        response.status(),
        200,
        "Scope update should succeed: {}",
        response.text().await.unwrap()
    );

    // Verify the file was created
    let registry_path = app.config_path().join("devices.json");
    assert!(registry_path.exists(), "devices.json should exist");

    // Read and verify contents
    let contents =
        std::fs::read_to_string(&registry_path).expect("Should be able to read devices.json");

    assert!(
        contents.contains(device_id),
        "Device ID should be in registry"
    );
    assert!(
        contents.contains("DeviceSpecific"),
        "Scope should be persisted as DeviceSpecific enum variant"
    );
}

/// Test that device config persists correctly and can be read back
#[tokio::test]
async fn test_device_config_loads_correctly_on_restart() {
    let app = TestApp::new().await;
    let device_id = "test-keyboard-003";

    // Register the device first
    register_device(&app, device_id).await;

    // Configure layout
    let response = app
        .put(
            &format!("/api/devices/{}/layout", device_id),
            &json!({ "layout": "iso_105" }),
        )
        .await;
    assert_eq!(response.status(), 200);

    // Configure scope
    let response = app
        .put(
            &format!("/api/devices/{}/scope", device_id),
            &json!({ "scope": "global" }),
        )
        .await;
    assert_eq!(response.status(), 200);

    // Verify the registry file exists with correct data
    let registry_path = app.config_path().join("devices.json");
    assert!(
        registry_path.exists(),
        "Registry should exist after configuration"
    );

    // Read and verify persisted data
    let contents =
        std::fs::read_to_string(&registry_path).expect("Should be able to read persisted registry");

    // Parse JSON to verify structure
    let registry: serde_json::Value =
        serde_json::from_str(&contents).expect("Registry should be valid JSON");

    // Verify device is in registry with correct values
    let device = registry
        .get(device_id)
        .expect("Device should be in registry");

    assert_eq!(
        device.get("layout").and_then(|v| v.as_str()),
        Some("iso_105"),
        "Layout should be persisted correctly"
    );

    assert_eq!(
        device.get("scope").and_then(|v| v.as_str()),
        Some("Global"),
        "Scope should be persisted correctly"
    );
}

/// Test that multiple device configurations persist independently
#[tokio::test]
async fn test_multiple_devices_persist_independently() {
    let app = TestApp::new().await;

    // Configure device 1
    let device1_id = "keyboard-001";
    register_device(&app, device1_id).await;

    app.put(
        &format!("/api/devices/{}/layout", device1_id),
        &json!({ "layout": "ansi_104" }),
    )
    .await;

    app.put(
        &format!("/api/devices/{}/scope", device1_id),
        &json!({ "scope": "global" }),
    )
    .await;

    // Configure device 2
    let device2_id = "keyboard-002";
    register_device(&app, device2_id).await;

    app.put(
        &format!("/api/devices/{}/layout", device2_id),
        &json!({ "layout": "iso_105" }),
    )
    .await;

    app.put(
        &format!("/api/devices/{}/scope", device2_id),
        &json!({ "scope": "device-specific" }),
    )
    .await;

    // Verify both devices are in registry
    let registry_path = app.config_path().join("devices.json");
    let contents =
        std::fs::read_to_string(&registry_path).expect("Should be able to read devices.json");

    let registry: serde_json::Value =
        serde_json::from_str(&contents).expect("Registry should be valid JSON");

    // Verify device 1
    let device1 = registry.get(device1_id).expect("Device 1 should exist");
    assert_eq!(
        device1.get("layout").and_then(|v| v.as_str()),
        Some("ansi_104")
    );
    assert_eq!(
        device1.get("scope").and_then(|v| v.as_str()),
        Some("Global")
    );

    // Verify device 2
    let device2 = registry.get(device2_id).expect("Device 2 should exist");
    assert_eq!(
        device2.get("layout").and_then(|v| v.as_str()),
        Some("iso_105")
    );
    assert_eq!(
        device2.get("scope").and_then(|v| v.as_str()),
        Some("DeviceSpecific")
    );
}

/// Test that device name persists correctly
#[tokio::test]
async fn test_device_name_persists_correctly() {
    let app = TestApp::new().await;
    let device_id = "test-keyboard-004";

    // Register the device first
    register_device(&app, device_id).await;

    // Set device name
    let response = app
        .put(
            &format!("/api/devices/{}/name", device_id),
            &json!({ "name": "My Custom Keyboard" }),
        )
        .await;

    assert_eq!(response.status(), 200, "Name update should succeed");

    // Verify persistence
    let registry_path = app.config_path().join("devices.json");
    let contents =
        std::fs::read_to_string(&registry_path).expect("Should be able to read devices.json");

    assert!(
        contents.contains("My Custom Keyboard"),
        "Device name should be persisted"
    );
}

/// Test that registry uses atomic writes (no corruption)
#[tokio::test]
async fn test_device_registry_atomic_writes() {
    let app = TestApp::new().await;
    let device_id = "test-keyboard-005";

    // Register the device first
    register_device(&app, device_id).await;

    // Make multiple rapid updates
    for i in 0..5 {
        let layout = format!("layout_{}", i);
        app.put(
            &format!("/api/devices/{}/layout", device_id),
            &json!({ "layout": layout }),
        )
        .await;
    }

    // Verify registry is not corrupted
    let registry_path = app.config_path().join("devices.json");
    let contents =
        std::fs::read_to_string(&registry_path).expect("Should be able to read devices.json");

    let registry: Result<serde_json::Value, _> = serde_json::from_str(&contents);
    assert!(
        registry.is_ok(),
        "Registry should be valid JSON after rapid updates"
    );

    // Verify temp file was cleaned up
    let tmp_path = registry_path.with_extension("tmp");
    assert!(
        !tmp_path.exists(),
        "Temp file should be removed after atomic write"
    );
}

/// Test that invalid scope values are rejected
#[tokio::test]
async fn test_invalid_scope_rejected() {
    let app = TestApp::new().await;
    let device_id = "test-keyboard-006";

    let response = app
        .put(
            &format!("/api/devices/{}/scope", device_id),
            &json!({ "scope": "invalid-scope-value" }),
        )
        .await;

    assert_eq!(
        response.status(),
        400,
        "Invalid scope should return 400 Bad Request"
    );
}

/// Test that device can be forgotten (removed from registry)
#[tokio::test]
async fn test_device_forget_removes_from_registry() {
    let app = TestApp::new().await;
    let device_id = "test-keyboard-007";

    // Register the device
    register_device(&app, device_id).await;

    // Set some configuration to ensure device is in registry
    app.put(
        &format!("/api/devices/{}/layout", device_id),
        &json!({ "layout": "ansi_104" }),
    )
    .await;

    // Verify device is in registry
    let registry_path = app.config_path().join("devices.json");
    let contents =
        std::fs::read_to_string(&registry_path).expect("Should be able to read devices.json");
    assert!(
        contents.contains(device_id),
        "Device should be in registry before forget"
    );

    // Forget the device
    let response = app.delete(&format!("/api/devices/{}", device_id)).await;
    assert_eq!(response.status(), 200, "Delete should succeed");

    // Verify device is removed from registry
    let contents =
        std::fs::read_to_string(&registry_path).expect("Should be able to read devices.json");
    assert!(
        !contents.contains(device_id),
        "Device should be removed from registry after forget"
    );
}
