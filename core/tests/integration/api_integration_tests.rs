//! Integration tests for API layer with real service implementations.
//!
//! These tests verify ApiContext works correctly with real services
//! (DeviceService, ProfileService, RuntimeService) using temporary
//! filesystem directories for isolation.

use std::collections::HashMap;
use std::sync::Arc;

use keyrx_core::api::ApiContext;
use keyrx_core::config::models::{
    DeviceInstanceId, HardwareProfile, Keymap, KeymapLayer, LayoutType, ProfileSlot, VirtualKeyDef,
    VirtualLayout,
};
use keyrx_core::config::ConfigManager;
use keyrx_core::registry::DeviceBindings;
use keyrx_core::services::{DeviceService, ProfileService, RuntimeService};
use tempfile::TempDir;

/// Creates an ApiContext with real services using a temporary directory.
fn create_api_with_temp_dir(temp_dir: &TempDir) -> ApiContext {
    let config_manager = ConfigManager::new(temp_dir.path());
    let bindings_path = temp_dir.path().join("device_bindings.json");
    let bindings = DeviceBindings::with_path(bindings_path);

    ApiContext::new(
        Arc::new(DeviceService::new(None, bindings)),
        Arc::new(ProfileService::new(config_manager.clone())),
        Arc::new(RuntimeService::new(config_manager)),
    )
}

/// Creates a sample VirtualLayout for testing.
fn sample_layout(id: &str, name: &str) -> VirtualLayout {
    VirtualLayout {
        id: id.to_string(),
        name: name.to_string(),
        layout_type: LayoutType::Matrix,
        keys: vec![VirtualKeyDef {
            id: "key-a".to_string(),
            label: "A".to_string(),
            position: None,
            size: None,
        }],
    }
}

/// Creates a sample HardwareProfile for testing.
fn sample_hardware_profile(id: &str, layout_id: &str) -> HardwareProfile {
    HardwareProfile {
        id: id.to_string(),
        vendor_id: 0x1234,
        product_id: 0x5678,
        name: Some("Test Hardware".to_string()),
        virtual_layout_id: layout_id.to_string(),
        wiring: HashMap::from([(4, "key-a".to_string())]),
    }
}

/// Creates a sample Keymap for testing.
fn sample_keymap(id: &str, layout_id: &str) -> Keymap {
    Keymap {
        id: id.to_string(),
        name: "Base Keymap".to_string(),
        virtual_layout_id: layout_id.to_string(),
        layers: vec![KeymapLayer {
            name: "default".to_string(),
            bindings: HashMap::new(),
        }],
        combos: vec![],
    }
}

/// Creates a sample ProfileSlot for testing.
fn sample_slot(id: &str, hw_profile_id: &str, keymap_id: &str) -> ProfileSlot {
    ProfileSlot {
        id: id.to_string(),
        hardware_profile_id: hw_profile_id.to_string(),
        keymap_id: keymap_id.to_string(),
        active: true,
        priority: 1,
    }
}

/// Creates a sample DeviceInstanceId for testing.
fn sample_device_instance() -> DeviceInstanceId {
    DeviceInstanceId {
        vendor_id: 0x1234,
        product_id: 0x5678,
        serial: Some("TEST-SERIAL-001".to_string()),
    }
}

// ============================================================================
// Tests: Empty State
// ============================================================================

#[test]
fn test_api_with_real_services_empty_state() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let api = create_api_with_temp_dir(&temp_dir);

    // All list operations should return empty on fresh state
    let layouts = api.list_virtual_layouts().expect("list layouts");
    assert!(layouts.is_empty(), "expected empty layouts on fresh state");

    let profiles = api
        .list_hardware_profiles()
        .expect("list hardware profiles");
    assert!(
        profiles.is_empty(),
        "expected empty hardware profiles on fresh state"
    );

    let keymaps = api.list_keymaps().expect("list keymaps");
    assert!(keymaps.is_empty(), "expected empty keymaps on fresh state");

    let config = api.get_runtime_config().expect("get runtime config");
    assert!(
        config.devices.is_empty(),
        "expected empty runtime config on fresh state"
    );
}

#[tokio::test]
async fn test_device_list_empty_state() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let api = create_api_with_temp_dir(&temp_dir);

    // With no registry and no bindings, should return empty list
    let devices = api.list_devices().await.expect("list devices");
    assert!(
        devices.is_empty(),
        "expected empty devices on fresh state with no registry"
    );
}

// ============================================================================
// Tests: Profile CRUD with Filesystem
// ============================================================================

#[test]
fn test_virtual_layout_crud_with_filesystem() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let api = create_api_with_temp_dir(&temp_dir);

    // Create
    let layout = sample_layout("layout-1", "Test Layout");
    let saved = api
        .save_virtual_layout(layout.clone())
        .expect("save layout");
    assert_eq!(saved.id, "layout-1");
    assert_eq!(saved.name, "Test Layout");

    // Read
    let layouts = api.list_virtual_layouts().expect("list layouts");
    assert_eq!(layouts.len(), 1);
    assert_eq!(layouts[0].id, "layout-1");

    // Update
    let updated_layout = VirtualLayout {
        id: "layout-1".to_string(),
        name: "Updated Layout".to_string(),
        layout_type: LayoutType::Semantic,
        keys: vec![],
    };
    let saved_updated = api
        .save_virtual_layout(updated_layout)
        .expect("update layout");
    assert_eq!(saved_updated.name, "Updated Layout");

    // Verify update persisted
    let layouts_after = api.list_virtual_layouts().expect("list after update");
    assert_eq!(layouts_after.len(), 1);
    assert_eq!(layouts_after[0].name, "Updated Layout");

    // Delete
    api.delete_virtual_layout("layout-1".to_string())
        .expect("delete layout");
    let layouts_final = api.list_virtual_layouts().expect("list after delete");
    assert!(layouts_final.is_empty());
}

#[test]
fn test_hardware_profile_crud_with_filesystem() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let api = create_api_with_temp_dir(&temp_dir);

    // First create a layout (dependency)
    let layout = sample_layout("layout-1", "Base Layout");
    api.save_virtual_layout(layout).expect("save layout");

    // Create hardware profile
    let profile = sample_hardware_profile("hw-1", "layout-1");
    let saved = api
        .save_hardware_profile(profile.clone())
        .expect("save hardware profile");
    assert_eq!(saved.id, "hw-1");

    // Read
    let profiles = api.list_hardware_profiles().expect("list profiles");
    assert_eq!(profiles.len(), 1);
    assert_eq!(profiles[0].id, "hw-1");

    // Update
    let updated_profile = HardwareProfile {
        id: "hw-1".to_string(),
        name: Some("Updated Hardware".to_string()),
        ..profile
    };
    api.save_hardware_profile(updated_profile)
        .expect("update profile");

    let profiles_after = api.list_hardware_profiles().expect("list after update");
    assert_eq!(profiles_after[0].name, Some("Updated Hardware".to_string()));

    // Delete
    api.delete_hardware_profile("hw-1".to_string())
        .expect("delete profile");
    let profiles_final = api.list_hardware_profiles().expect("list after delete");
    assert!(profiles_final.is_empty());
}

#[test]
fn test_keymap_crud_with_filesystem() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let api = create_api_with_temp_dir(&temp_dir);

    // First create a layout (dependency)
    let layout = sample_layout("layout-1", "Base Layout");
    api.save_virtual_layout(layout).expect("save layout");

    // Create keymap
    let keymap = sample_keymap("km-1", "layout-1");
    let saved = api.save_keymap(keymap.clone()).expect("save keymap");
    assert_eq!(saved.id, "km-1");
    assert_eq!(saved.name, "Base Keymap");

    // Read
    let keymaps = api.list_keymaps().expect("list keymaps");
    assert_eq!(keymaps.len(), 1);
    assert_eq!(keymaps[0].id, "km-1");

    // Update
    let updated_keymap = Keymap {
        id: "km-1".to_string(),
        name: "Updated Keymap".to_string(),
        ..keymap
    };
    api.save_keymap(updated_keymap).expect("update keymap");

    let keymaps_after = api.list_keymaps().expect("list after update");
    assert_eq!(keymaps_after[0].name, "Updated Keymap");

    // Delete
    api.delete_keymap("km-1".to_string())
        .expect("delete keymap");
    let keymaps_final = api.list_keymaps().expect("list after delete");
    assert!(keymaps_final.is_empty());
}

// ============================================================================
// Tests: Runtime Config with Real Services
// ============================================================================

#[test]
fn test_runtime_config_slot_management() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let api = create_api_with_temp_dir(&temp_dir);

    // Setup dependencies
    let layout = sample_layout("layout-1", "Test Layout");
    api.save_virtual_layout(layout).expect("save layout");

    let hw_profile = sample_hardware_profile("hw-1", "layout-1");
    api.save_hardware_profile(hw_profile)
        .expect("save hw profile");

    let keymap = sample_keymap("km-1", "layout-1");
    api.save_keymap(keymap).expect("save keymap");

    let device = sample_device_instance();
    let slot = sample_slot("slot-1", "hw-1", "km-1");

    // Add slot
    let config = api
        .runtime_add_slot(device.clone(), slot)
        .expect("add slot");
    assert_eq!(config.devices.len(), 1);
    assert_eq!(config.devices[0].slots.len(), 1);
    assert_eq!(config.devices[0].slots[0].id, "slot-1");

    // Verify persistence
    let loaded_config = api.get_runtime_config().expect("get config");
    assert_eq!(loaded_config.devices.len(), 1);

    // Reorder slot
    let config_reordered = api
        .runtime_reorder_slot(device.clone(), "slot-1".to_string(), 10)
        .expect("reorder slot");
    assert_eq!(config_reordered.devices[0].slots[0].priority, 10);

    // Set slot inactive
    let config_inactive = api
        .runtime_set_slot_active(device.clone(), "slot-1".to_string(), false)
        .expect("set inactive");
    assert!(!config_inactive.devices[0].slots[0].active);

    // Remove slot
    let config_removed = api
        .runtime_remove_slot(device, "slot-1".to_string())
        .expect("remove slot");
    assert!(
        config_removed.devices[0].slots.is_empty()
            || config_removed
                .devices
                .iter()
                .all(|d| d.slots.iter().all(|s| s.id != "slot-1"))
    );
}

#[test]
fn test_runtime_multiple_slots_ordering() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let api = create_api_with_temp_dir(&temp_dir);

    // Setup dependencies
    let layout = sample_layout("layout-1", "Test Layout");
    api.save_virtual_layout(layout).expect("save layout");

    let hw_profile = sample_hardware_profile("hw-1", "layout-1");
    api.save_hardware_profile(hw_profile)
        .expect("save hw profile");

    let keymap = sample_keymap("km-1", "layout-1");
    api.save_keymap(keymap).expect("save keymap");

    let device = sample_device_instance();

    // Add multiple slots with different priorities
    let slot_low = ProfileSlot {
        id: "slot-low".to_string(),
        hardware_profile_id: "hw-1".to_string(),
        keymap_id: "km-1".to_string(),
        active: true,
        priority: 1,
    };
    let slot_high = ProfileSlot {
        id: "slot-high".to_string(),
        hardware_profile_id: "hw-1".to_string(),
        keymap_id: "km-1".to_string(),
        active: true,
        priority: 10,
    };

    api.runtime_add_slot(device.clone(), slot_low)
        .expect("add low slot");
    let config = api
        .runtime_add_slot(device, slot_high)
        .expect("add high slot");

    // Slots should be sorted by priority descending
    assert_eq!(config.devices[0].slots.len(), 2);
    assert_eq!(config.devices[0].slots[0].id, "slot-high"); // priority 10 first
    assert_eq!(config.devices[0].slots[1].id, "slot-low"); // priority 1 second
}

// ============================================================================
// Tests: Error Handling with Real I/O
// ============================================================================

#[test]
fn test_delete_nonexistent_layout_succeeds() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let api = create_api_with_temp_dir(&temp_dir);

    // Deleting a non-existent layout should not error (idempotent delete)
    let result = api.delete_virtual_layout("nonexistent".to_string());
    assert!(result.is_ok(), "delete nonexistent should succeed");
}

#[test]
fn test_runtime_error_device_not_found() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let api = create_api_with_temp_dir(&temp_dir);

    let device = sample_device_instance();

    // Try to remove slot from device that has no slots
    let result = api.runtime_remove_slot(device, "nonexistent-slot".to_string());
    assert!(
        result.is_err(),
        "remove slot from unknown device should error"
    );
}

#[test]
fn test_runtime_error_slot_not_found() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let api = create_api_with_temp_dir(&temp_dir);

    // Setup: add a device with one slot
    let layout = sample_layout("layout-1", "Test Layout");
    api.save_virtual_layout(layout).expect("save layout");

    let hw_profile = sample_hardware_profile("hw-1", "layout-1");
    api.save_hardware_profile(hw_profile).expect("save hw");

    let keymap = sample_keymap("km-1", "layout-1");
    api.save_keymap(keymap).expect("save keymap");

    let device = sample_device_instance();
    let slot = sample_slot("slot-1", "hw-1", "km-1");
    api.runtime_add_slot(device.clone(), slot)
        .expect("add slot");

    // Try to remove a different slot
    let result = api.runtime_remove_slot(device, "wrong-slot".to_string());
    assert!(result.is_err(), "remove wrong slot should error");
}

// ============================================================================
// Tests: Data Persistence Across Instances
// ============================================================================

#[test]
fn test_data_persists_across_api_instances() {
    let temp_dir = TempDir::new().expect("create temp dir");

    // First instance: save data
    {
        let api = create_api_with_temp_dir(&temp_dir);

        let layout = sample_layout("persist-layout", "Persistent Layout");
        api.save_virtual_layout(layout).expect("save layout");

        let keymap = sample_keymap("persist-keymap", "persist-layout");
        api.save_keymap(keymap).expect("save keymap");
    }

    // Second instance: verify data persisted
    {
        let api = create_api_with_temp_dir(&temp_dir);

        let layouts = api.list_virtual_layouts().expect("list layouts");
        assert_eq!(layouts.len(), 1);
        assert_eq!(layouts[0].id, "persist-layout");

        let keymaps = api.list_keymaps().expect("list keymaps");
        assert_eq!(keymaps.len(), 1);
        assert_eq!(keymaps[0].id, "persist-keymap");
    }
}

#[test]
fn test_runtime_config_persists_across_instances() {
    let temp_dir = TempDir::new().expect("create temp dir");

    // First instance: setup and add slot
    {
        let api = create_api_with_temp_dir(&temp_dir);

        let layout = sample_layout("layout-1", "Test Layout");
        api.save_virtual_layout(layout).expect("save layout");

        let hw_profile = sample_hardware_profile("hw-1", "layout-1");
        api.save_hardware_profile(hw_profile).expect("save hw");

        let keymap = sample_keymap("km-1", "layout-1");
        api.save_keymap(keymap).expect("save keymap");

        let device = sample_device_instance();
        let slot = sample_slot("persistent-slot", "hw-1", "km-1");
        api.runtime_add_slot(device, slot).expect("add slot");
    }

    // Second instance: verify runtime config persisted
    {
        let api = create_api_with_temp_dir(&temp_dir);

        let config = api.get_runtime_config().expect("get config");
        assert_eq!(config.devices.len(), 1);
        assert_eq!(config.devices[0].slots.len(), 1);
        assert_eq!(config.devices[0].slots[0].id, "persistent-slot");
    }
}

// ============================================================================
// Tests: ApiContext with_defaults Constructor
// ============================================================================

#[test]
fn test_api_context_with_defaults_compiles_and_constructs() {
    // This test verifies that ApiContext::with_defaults() compiles
    // and creates a valid instance. We don't actually use it for I/O
    // since it would use the real config directory.
    let _api = ApiContext::with_defaults();
    // If we get here without panic, the constructor works
}
