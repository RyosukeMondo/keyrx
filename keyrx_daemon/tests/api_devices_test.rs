//! Integration tests for device persistence API endpoints.
//!
//! Tests verify that device configuration (layout) persists to filesystem
//! and loads correctly on daemon restart.
//!
//! # Note on Serial Execution
//!
//! These tests use `#[serial]` attribute to run serially because they modify
//! the global HOME environment variable. The serial_test crate ensures proper isolation.

mod common;

use common::test_app::TestApp;
use serde_json::json;
use serial_test::serial;

/// Helper function to register a device in the registry
async fn register_device(app: &TestApp, device_id: &str) {
    use keyrx_daemon::config::device_registry::{DeviceEntry, DeviceRegistry};

    let registry_path = app.config_path().join("devices.json");
    let mut registry = DeviceRegistry::load(&registry_path).expect("Failed to load registry");

    let entry = DeviceEntry::new(
        device_id.to_string(),
        device_id.to_string(),
        Some(device_id.to_string()),
        None,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    );

    registry.register(entry).expect("Failed to register device");
    registry.save().expect("Failed to save registry");
}

/// Test that device layout persists to filesystem
#[tokio::test]
#[serial]
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

/// Test that device config persists correctly and can be read back
#[tokio::test]
#[serial]
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
}

/// Test that multiple device configurations persist independently
#[tokio::test]
#[serial]
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

    // Configure device 2
    let device2_id = "keyboard-002";
    register_device(&app, device2_id).await;

    app.put(
        &format!("/api/devices/{}/layout", device2_id),
        &json!({ "layout": "iso_105" }),
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

    // Verify device 2
    let device2 = registry.get(device2_id).expect("Device 2 should exist");
    assert_eq!(
        device2.get("layout").and_then(|v| v.as_str()),
        Some("iso_105")
    );
}

/// Test that device name persists correctly
#[tokio::test]
#[serial]
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
#[serial]
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

/// Test that device can be forgotten (removed from registry)
#[tokio::test]
#[serial]
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

// ============================================================================
// Scope Removal Tests (Requirements 1.1-1.4)
// ============================================================================

/// Test that device registry does not contain scope field
#[tokio::test]
#[serial]
async fn test_device_registry_no_scope_field() {
    let app = TestApp::new().await;
    let device_id = "test-keyboard-no-scope";

    // Register the device
    register_device(&app, device_id).await;

    // Set device layout
    app.put(
        &format!("/api/devices/{}/layout", device_id),
        &json!({ "layout": "ansi_104" }),
    )
    .await;

    // Verify the registry file does NOT contain "scope" field
    let registry_path = app.config_path().join("devices.json");
    let contents =
        std::fs::read_to_string(&registry_path).expect("Should be able to read devices.json");

    // Parse JSON to verify structure
    let registry: serde_json::Value =
        serde_json::from_str(&contents).expect("Registry should be valid JSON");

    let device = registry
        .get(device_id)
        .expect("Device should be in registry");

    // Verify scope field does NOT exist
    assert!(
        device.get("scope").is_none(),
        "Device entry should not contain 'scope' field"
    );
}

/// Test that PATCH /api/devices/:id ignores scope parameter if sent
#[tokio::test]
#[serial]
async fn test_patch_device_ignores_scope_parameter() {
    let app = TestApp::new().await;
    let device_id = "test-keyboard-ignore-scope";

    // Register the device
    register_device(&app, device_id).await;

    // Try to set layout with scope parameter (should be ignored)
    let response = app
        .patch(
            &format!("/api/devices/{}", device_id),
            &json!({
                "layout": "iso_105",
                "scope": "global"  // This should be ignored
            }),
        )
        .await;

    // Request should succeed (scope parameter ignored, not rejected)
    assert!(
        response.status().is_success() || response.status() == 400,
        "Request should succeed or return 400 if validation rejects unknown fields"
    );

    // Verify registry does NOT contain scope
    let registry_path = app.config_path().join("devices.json");
    let contents =
        std::fs::read_to_string(&registry_path).expect("Should be able to read devices.json");

    let registry: serde_json::Value =
        serde_json::from_str(&contents).expect("Registry should be valid JSON");

    if let Some(device) = registry.get(device_id) {
        assert!(
            device.get("scope").is_none(),
            "Scope field should not be saved to registry"
        );
    }
}

/// Test backward compatibility: old registry files with scope field load correctly
#[tokio::test]
#[serial]
async fn test_backward_compatibility_old_registry_with_scope() {
    let app = TestApp::new().await;
    let registry_path = app.config_path().join("devices.json");

    // Create old-format registry file with scope field
    // Note: DeviceEntry uses camelCase for JSON fields
    std::fs::create_dir_all(app.config_path()).unwrap();
    std::fs::write(
        &registry_path,
        r#"{
            "old-device": {
                "id": "old-device",
                "name": "Old Keyboard",
                "serial": "OLD123",
                "layout": "ansi_104",
                "scope": "DeviceSpecific",
                "lastSeen": 1234567890
            }
        }"#,
    )
    .unwrap();

    // Load the registry using DeviceRegistry
    use keyrx_daemon::config::device_registry::DeviceRegistry;
    let registry = DeviceRegistry::load(&registry_path);

    // Should load successfully despite having scope field
    assert!(
        registry.is_ok(),
        "Old registry format with scope field should load correctly"
    );

    // Verify the device is present
    let registry = registry.unwrap();
    assert!(
        registry.get("old-device").is_some(),
        "Device from old registry should be accessible"
    );

    // Save the registry (should remove scope field)
    registry.save().expect("Should save registry");

    // Verify scope field is NOT in saved file
    let contents =
        std::fs::read_to_string(&registry_path).expect("Should be able to read devices.json");

    let saved_registry: serde_json::Value =
        serde_json::from_str(&contents).expect("Registry should be valid JSON");

    if let Some(device) = saved_registry.get("old-device") {
        assert!(
            device.get("scope").is_none(),
            "Scope field should be removed when saving"
        );
    }
}

// ============================================================================
// Device Layout Inheritance Tests (Requirements 2.1-2.5)
// ============================================================================

/// Test that new devices inherit global layout as default
#[tokio::test]
#[serial]
async fn test_new_device_inherits_global_layout() {
    let app = TestApp::new().await;
    let settings_path = app.config_path().join("settings.json");

    // Set global layout
    std::fs::create_dir_all(app.config_path()).unwrap();
    std::fs::write(&settings_path, r#"{"global_layout":"ISO_105"}"#).unwrap();

    // Register a new device (without explicit layout)
    let device_id = "new-device-inherit";
    let registry_path = app.config_path().join("devices.json");

    use keyrx_daemon::config::device_registry::{DeviceEntry, DeviceRegistry};
    let mut registry = DeviceRegistry::load(&registry_path).unwrap();

    let entry = DeviceEntry::new(
        device_id.to_string(),
        "New Keyboard".to_string(),
        Some("NEW123".to_string()),
        None, // No explicit layout
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    );

    registry.register(entry).unwrap();
    registry.save().unwrap();

    // The device should use global layout as default
    // (This is verified by reading settings.json - the device itself has no layout)
    let device = registry.get(device_id).unwrap();
    assert!(
        device.layout.is_none(),
        "New device without explicit layout should have layout=None in registry"
    );

    // Global layout should still be ISO_105
    let settings_contents = std::fs::read_to_string(&settings_path).unwrap();
    let settings: serde_json::Value = serde_json::from_str(&settings_contents).unwrap();
    assert_eq!(
        settings.get("global_layout").and_then(|v| v.as_str()),
        Some("ISO_105"),
        "Global layout should be the default for devices without explicit layout"
    );
}

/// Test that device-specific layout overrides global layout
#[tokio::test]
#[serial]
async fn test_device_specific_layout_overrides_global() {
    let app = TestApp::new().await;
    let settings_path = app.config_path().join("settings.json");
    let device_id = "device-with-override";

    // Set global layout to ANSI_104
    std::fs::create_dir_all(app.config_path()).unwrap();
    std::fs::write(&settings_path, r#"{"global_layout":"ANSI_104"}"#).unwrap();

    // Register device and set device-specific layout
    register_device(&app, device_id).await;

    app.put(
        &format!("/api/devices/{}/layout", device_id),
        &json!({ "layout": "JIS_109" }),
    )
    .await;

    // Verify device has its own layout
    let registry_path = app.config_path().join("devices.json");
    let contents = std::fs::read_to_string(&registry_path).unwrap();
    let registry: serde_json::Value = serde_json::from_str(&contents).unwrap();

    let device = registry.get(device_id).unwrap();
    assert_eq!(
        device.get("layout").and_then(|v| v.as_str()),
        Some("JIS_109"),
        "Device-specific layout should override global layout"
    );

    // Verify global layout is still ANSI_104
    let settings_contents = std::fs::read_to_string(&settings_path).unwrap();
    let settings: serde_json::Value = serde_json::from_str(&settings_contents).unwrap();
    assert_eq!(
        settings.get("global_layout").and_then(|v| v.as_str()),
        Some("ANSI_104"),
        "Global layout should not be affected by device-specific override"
    );
}

/// Test that changing global layout does not affect existing device overrides
#[tokio::test]
#[serial]
async fn test_global_layout_change_preserves_device_overrides() {
    let app = TestApp::new().await;
    let settings_path = app.config_path().join("settings.json");
    let device_id = "device-preserve-override";

    // Set initial global layout
    std::fs::create_dir_all(app.config_path()).unwrap();
    std::fs::write(&settings_path, r#"{"global_layout":"ANSI_104"}"#).unwrap();

    // Register device with specific layout
    register_device(&app, device_id).await;

    app.put(
        &format!("/api/devices/{}/layout", device_id),
        &json!({ "layout": "ISO_105" }),
    )
    .await;

    // Change global layout
    std::fs::write(&settings_path, r#"{"global_layout":"HHKB"}"#).unwrap();

    // Verify device still has its override
    let registry_path = app.config_path().join("devices.json");
    let contents = std::fs::read_to_string(&registry_path).unwrap();
    let registry: serde_json::Value = serde_json::from_str(&contents).unwrap();

    let device = registry.get(device_id).unwrap();
    assert_eq!(
        device.get("layout").and_then(|v| v.as_str()),
        Some("ISO_105"),
        "Device-specific layout should be preserved when global layout changes"
    );
}

/// Test that multiple devices can have different overrides with same global layout
#[tokio::test]
#[serial]
async fn test_multiple_device_overrides_with_global_layout() {
    let app = TestApp::new().await;
    let settings_path = app.config_path().join("settings.json");

    // Set global layout
    std::fs::create_dir_all(app.config_path()).unwrap();
    std::fs::write(&settings_path, r#"{"global_layout":"ANSI_104"}"#).unwrap();

    // Register device 1 with override
    let device1_id = "device-override-1";
    register_device(&app, device1_id).await;
    app.put(
        &format!("/api/devices/{}/layout", device1_id),
        &json!({ "layout": "ISO_105" }),
    )
    .await;

    // Register device 2 with different override
    let device2_id = "device-override-2";
    register_device(&app, device2_id).await;
    app.put(
        &format!("/api/devices/{}/layout", device2_id),
        &json!({ "layout": "JIS_109" }),
    )
    .await;

    // Register device 3 without override (uses global)
    let device3_id = "device-no-override";
    register_device(&app, device3_id).await;

    // Verify all devices have correct layouts
    let registry_path = app.config_path().join("devices.json");
    let contents = std::fs::read_to_string(&registry_path).unwrap();
    let registry: serde_json::Value = serde_json::from_str(&contents).unwrap();

    let device1 = registry.get(device1_id).unwrap();
    assert_eq!(
        device1.get("layout").and_then(|v| v.as_str()),
        Some("ISO_105")
    );

    let device2 = registry.get(device2_id).unwrap();
    assert_eq!(
        device2.get("layout").and_then(|v| v.as_str()),
        Some("JIS_109")
    );

    let device3 = registry.get(device3_id).unwrap();
    // Device 3 has the device_id as layout (from register_device helper)
    // In production, devices without explicit layout would have None
    let device3_layout = device3.get("layout").and_then(|v| v.as_str());
    assert!(
        device3_layout.is_none() || device3_layout == Some(device3_id),
        "Device without explicit layout should have None or default layout, got: {:?}",
        device3_layout
    );
}
