//! Unit tests for runtime API methods using MockRuntimeService.
//!
//! These tests verify ApiContext runtime operations work correctly with mocked
//! dependencies, enabling fast, isolated testing without I/O.

use std::sync::Arc;

use keyrx_core::api::ApiContext;
use keyrx_core::config::models::{DeviceInstanceId, DeviceSlots, ProfileSlot, RuntimeConfig};
use keyrx_core::services::{MockDeviceService, MockProfileService, MockRuntimeService};

/// Creates a test device instance ID.
fn test_device_id() -> DeviceInstanceId {
    DeviceInstanceId {
        vendor_id: 0x1234,
        product_id: 0x5678,
        serial: Some("test-serial".to_string()),
    }
}

/// Creates a test device instance ID with custom serial.
fn test_device_id_with_serial(serial: &str) -> DeviceInstanceId {
    DeviceInstanceId {
        vendor_id: 0x1234,
        product_id: 0x5678,
        serial: Some(serial.to_string()),
    }
}

/// Creates a test profile slot with the given ID and priority.
fn test_slot(id: &str, priority: u32) -> ProfileSlot {
    ProfileSlot {
        id: id.to_string(),
        hardware_profile_id: "hw-profile-1".to_string(),
        keymap_id: "keymap-1".to_string(),
        active: true,
        priority,
    }
}

/// Creates a test profile slot with custom active state.
fn test_slot_with_active(id: &str, priority: u32, active: bool) -> ProfileSlot {
    ProfileSlot {
        id: id.to_string(),
        hardware_profile_id: "hw-profile-1".to_string(),
        keymap_id: "keymap-1".to_string(),
        active,
        priority,
    }
}

/// Creates a runtime config with the given device and slots.
fn test_runtime_config(device: DeviceInstanceId, slots: Vec<ProfileSlot>) -> RuntimeConfig {
    RuntimeConfig {
        devices: vec![DeviceSlots { device, slots }],
    }
}

/// Helper to create ApiContext with mocked runtime service.
fn create_api_with_runtime_mock(runtime_mock: MockRuntimeService) -> ApiContext {
    ApiContext::new(
        Arc::new(MockDeviceService::new()),
        Arc::new(MockProfileService::new()),
        Arc::new(runtime_mock),
    )
}

#[test]
fn test_get_runtime_config_empty() {
    let mock = MockRuntimeService::new();
    let api = create_api_with_runtime_mock(mock);

    let result = api.get_runtime_config().unwrap();

    assert!(result.devices.is_empty());
}

#[test]
fn test_get_runtime_config_with_data() {
    let device = test_device_id();
    let slots = vec![test_slot("slot-1", 100), test_slot("slot-2", 50)];
    let config = test_runtime_config(device.clone(), slots);

    let mock = MockRuntimeService::new().with_config(config);
    let api = create_api_with_runtime_mock(mock);

    let result = api.get_runtime_config().unwrap();

    assert_eq!(result.devices.len(), 1);
    assert_eq!(result.devices[0].device, device);
    assert_eq!(result.devices[0].slots.len(), 2);
    assert_eq!(result.devices[0].slots[0].id, "slot-1");
    assert_eq!(result.devices[0].slots[1].id, "slot-2");
}

#[test]
fn test_add_slot_to_empty_config() {
    let mock = MockRuntimeService::new();
    let api = create_api_with_runtime_mock(mock);

    let device = test_device_id();
    let slot = test_slot("new-slot", 100);

    let result = api.runtime_add_slot(device.clone(), slot).unwrap();

    assert_eq!(result.devices.len(), 1);
    assert_eq!(result.devices[0].device, device);
    assert_eq!(result.devices[0].slots.len(), 1);
    assert_eq!(result.devices[0].slots[0].id, "new-slot");
    assert_eq!(result.devices[0].slots[0].priority, 100);
}

#[test]
fn test_add_slot_to_existing_device() {
    let device = test_device_id();
    let existing_config = test_runtime_config(device.clone(), vec![test_slot("slot-1", 100)]);

    let mock = MockRuntimeService::new().with_config(existing_config);
    let api = create_api_with_runtime_mock(mock);

    let new_slot = test_slot("slot-2", 50);
    let result = api.runtime_add_slot(device.clone(), new_slot).unwrap();

    assert_eq!(result.devices[0].slots.len(), 2);
}

#[test]
fn test_add_slot_sorts_by_priority() {
    let mock = MockRuntimeService::new();
    let api = create_api_with_runtime_mock(mock);

    let device = test_device_id();

    // Add slots in reverse priority order
    api.runtime_add_slot(device.clone(), test_slot("low", 10))
        .unwrap();
    api.runtime_add_slot(device.clone(), test_slot("high", 100))
        .unwrap();
    let result = api
        .runtime_add_slot(device.clone(), test_slot("mid", 50))
        .unwrap();

    // Should be sorted descending by priority
    let slots = &result.devices[0].slots;
    assert_eq!(slots[0].id, "high");
    assert_eq!(slots[1].id, "mid");
    assert_eq!(slots[2].id, "low");
}

#[test]
fn test_remove_slot_success() {
    let device = test_device_id();
    let config = test_runtime_config(
        device.clone(),
        vec![test_slot("slot-1", 100), test_slot("slot-2", 50)],
    );

    let mock = MockRuntimeService::new().with_config(config);
    let api = create_api_with_runtime_mock(mock);

    let result = api
        .runtime_remove_slot(device.clone(), "slot-1".to_string())
        .unwrap();

    assert_eq!(result.devices[0].slots.len(), 1);
    assert_eq!(result.devices[0].slots[0].id, "slot-2");
}

#[test]
fn test_remove_slot_not_found() {
    let device = test_device_id();
    let config = test_runtime_config(device.clone(), vec![test_slot("slot-1", 100)]);

    let mock = MockRuntimeService::new().with_config(config);
    let api = create_api_with_runtime_mock(mock);

    let result = api.runtime_remove_slot(device.clone(), "nonexistent".to_string());

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("nonexistent") || error_msg.contains("not found"));
}

#[test]
fn test_remove_slot_device_not_found() {
    let mock = MockRuntimeService::new();
    let api = create_api_with_runtime_mock(mock);

    let device = test_device_id();
    let result = api.runtime_remove_slot(device, "slot-1".to_string());

    assert!(result.is_err());
}

#[test]
fn test_reorder_slot_success() {
    let device = test_device_id();
    let config = test_runtime_config(
        device.clone(),
        vec![test_slot("slot-1", 100), test_slot("slot-2", 50)],
    );

    let mock = MockRuntimeService::new().with_config(config);
    let api = create_api_with_runtime_mock(mock);

    // Reorder slot-2 to highest priority
    let result = api
        .runtime_reorder_slot(device.clone(), "slot-2".to_string(), 200)
        .unwrap();

    // slot-2 should now be first due to higher priority
    assert_eq!(result.devices[0].slots[0].id, "slot-2");
    assert_eq!(result.devices[0].slots[0].priority, 200);
    assert_eq!(result.devices[0].slots[1].id, "slot-1");
}

#[test]
fn test_reorder_slot_not_found() {
    let device = test_device_id();
    let config = test_runtime_config(device.clone(), vec![test_slot("slot-1", 100)]);

    let mock = MockRuntimeService::new().with_config(config);
    let api = create_api_with_runtime_mock(mock);

    let result = api.runtime_reorder_slot(device, "nonexistent".to_string(), 200);

    assert!(result.is_err());
}

#[test]
fn test_set_slot_active_success() {
    let device = test_device_id();
    let config = test_runtime_config(
        device.clone(),
        vec![test_slot_with_active("slot-1", 100, true)],
    );

    let mock = MockRuntimeService::new().with_config(config);
    let api = create_api_with_runtime_mock(mock);

    // Deactivate the slot
    let result = api
        .runtime_set_slot_active(device.clone(), "slot-1".to_string(), false)
        .unwrap();

    assert!(!result.devices[0].slots[0].active);

    // Reactivate it
    let result = api
        .runtime_set_slot_active(device.clone(), "slot-1".to_string(), true)
        .unwrap();

    assert!(result.devices[0].slots[0].active);
}

#[test]
fn test_set_slot_active_slot_not_found() {
    let device = test_device_id();
    let config = test_runtime_config(device.clone(), vec![test_slot("slot-1", 100)]);

    let mock = MockRuntimeService::new().with_config(config);
    let api = create_api_with_runtime_mock(mock);

    let result = api.runtime_set_slot_active(device, "nonexistent".to_string(), false);

    assert!(result.is_err());
}

#[test]
fn test_runtime_error_handling_get_config() {
    let mock = MockRuntimeService::new().with_get_config_error("storage failure");
    let api = create_api_with_runtime_mock(mock);

    let result = api.get_runtime_config();

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("storage failure"));
}

#[test]
fn test_runtime_error_handling_add_slot() {
    let mock = MockRuntimeService::new().with_add_slot_error("write failed");
    let api = create_api_with_runtime_mock(mock);

    let result = api.runtime_add_slot(test_device_id(), test_slot("slot-1", 100));

    assert!(result.is_err());
}

#[test]
fn test_runtime_error_handling_remove_slot() {
    let device = test_device_id();
    let config = test_runtime_config(device.clone(), vec![test_slot("slot-1", 100)]);

    let mock = MockRuntimeService::new()
        .with_config(config)
        .with_remove_slot_error("remove failed");
    let api = create_api_with_runtime_mock(mock);

    let result = api.runtime_remove_slot(device, "slot-1".to_string());

    assert!(result.is_err());
}

#[test]
fn test_mock_tracks_method_calls() {
    let mock = Arc::new(MockRuntimeService::new());
    let api = ApiContext::new(
        Arc::new(MockDeviceService::new()),
        Arc::new(MockProfileService::new()),
        mock.clone(),
    );

    let device = test_device_id();

    // Perform multiple operations
    let _ = api.get_runtime_config();
    let _ = api.get_runtime_config();
    let _ = api.runtime_add_slot(device.clone(), test_slot("slot-1", 100));
    let _ = api.runtime_add_slot(device.clone(), test_slot("slot-2", 50));
    let _ = api.runtime_remove_slot(device.clone(), "slot-1".to_string());
    let _ = api.runtime_reorder_slot(device.clone(), "slot-2".to_string(), 200);
    let _ = api.runtime_set_slot_active(device.clone(), "slot-2".to_string(), false);

    // Verify call counts
    assert_eq!(mock.get_call_count("get_config"), 2);
    assert_eq!(mock.get_call_count("add_slot"), 2);
    assert_eq!(mock.get_call_count("remove_slot"), 1);
    assert_eq!(mock.get_call_count("reorder_slot"), 1);
    assert_eq!(mock.get_call_count("set_slot_active"), 1);
}

#[test]
fn test_multiple_devices_config() {
    let device1 = test_device_id_with_serial("device-1");
    let device2 = test_device_id_with_serial("device-2");

    let config = RuntimeConfig {
        devices: vec![
            DeviceSlots {
                device: device1.clone(),
                slots: vec![test_slot("slot-1a", 100)],
            },
            DeviceSlots {
                device: device2.clone(),
                slots: vec![test_slot("slot-2a", 80), test_slot("slot-2b", 60)],
            },
        ],
    };

    let mock = MockRuntimeService::new().with_config(config);
    let api = create_api_with_runtime_mock(mock);

    let result = api.get_runtime_config().unwrap();

    assert_eq!(result.devices.len(), 2);

    // Verify first device
    let dev1 = result.devices.iter().find(|d| d.device == device1).unwrap();
    assert_eq!(dev1.slots.len(), 1);
    assert_eq!(dev1.slots[0].id, "slot-1a");

    // Verify second device
    let dev2 = result.devices.iter().find(|d| d.device == device2).unwrap();
    assert_eq!(dev2.slots.len(), 2);
}

#[test]
fn test_api_context_accepts_trait_objects_for_runtime() {
    // Verify ApiContext::new() correctly accepts Arc<dyn RuntimeServiceTrait>
    let runtime_mock: Arc<dyn keyrx_core::services::RuntimeServiceTrait> =
        Arc::new(MockRuntimeService::new());

    let api = ApiContext::new(
        Arc::new(MockDeviceService::new()),
        Arc::new(MockProfileService::new()),
        runtime_mock,
    );

    let result = api.get_runtime_config().unwrap();
    assert!(result.devices.is_empty());
}

#[test]
fn test_slot_upsert_updates_existing() {
    let device = test_device_id();
    let config = test_runtime_config(device.clone(), vec![test_slot("slot-1", 100)]);

    let mock = MockRuntimeService::new().with_config(config);
    let api = create_api_with_runtime_mock(mock);

    // Add slot with same ID but different priority
    let updated_slot = test_slot("slot-1", 200);
    let result = api.runtime_add_slot(device.clone(), updated_slot).unwrap();

    // Should still have only one slot (upserted, not added)
    assert_eq!(result.devices[0].slots.len(), 1);
    assert_eq!(result.devices[0].slots[0].id, "slot-1");
    assert_eq!(result.devices[0].slots[0].priority, 200);
}
