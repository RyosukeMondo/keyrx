//! Tests for mock service implementations.

use std::collections::HashMap;

use super::*;
use crate::config::models::{DeviceInstanceId, LayoutType, ProfileSlot};
use crate::services::device::{DeviceServiceError, DeviceView};
use crate::services::runtime::RuntimeServiceError;
use crate::services::traits::{DeviceServiceTrait, ProfileServiceTrait, RuntimeServiceTrait};

fn test_device(key: &str) -> DeviceView {
    DeviceView {
        key: key.to_string(),
        vendor_id: 0x1234,
        product_id: 0x5678,
        serial_number: "test".to_string(),
        label: None,
        remap_enabled: false,
        profile_id: None,
        connected: true,
    }
}

fn test_layout(id: &str) -> crate::config::models::VirtualLayout {
    crate::config::models::VirtualLayout {
        id: id.to_string(),
        name: format!("Layout {}", id),
        layout_type: LayoutType::Semantic,
        keys: vec![],
    }
}

fn test_hardware_profile(id: &str) -> crate::config::models::HardwareProfile {
    crate::config::models::HardwareProfile {
        id: id.to_string(),
        vendor_id: 0x1234,
        product_id: 0x5678,
        name: Some(format!("Profile {}", id)),
        virtual_layout_id: "layout-1".to_string(),
        wiring: HashMap::new(),
    }
}

fn test_keymap(id: &str) -> crate::config::models::Keymap {
    crate::config::models::Keymap {
        id: id.to_string(),
        name: format!("Keymap {}", id),
        virtual_layout_id: "layout-1".to_string(),
        layers: vec![],
    }
}

// MockDeviceService tests
#[tokio::test]
async fn test_mock_device_service_list_devices() {
    let devices = vec![test_device("1234:5678:test")];
    let mock = MockDeviceService::new().with_devices(devices.clone());

    let result = mock.list_devices().await.unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].key, "1234:5678:test");
    assert_eq!(mock.get_call_count("list_devices"), 1);
}

#[tokio::test]
async fn test_mock_device_service_list_error() {
    let mock = MockDeviceService::new()
        .with_list_error(DeviceServiceError::Io(std::io::Error::other("test error")));

    let result = mock.list_devices().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_mock_device_service_get_device() {
    let devices = vec![test_device("1234:5678:test")];
    let mock = MockDeviceService::new().with_devices(devices);

    let result = mock.get_device("1234:5678:test").await.unwrap();
    assert_eq!(result.key, "1234:5678:test");
    assert_eq!(mock.get_call_count("get_device"), 1);
}

#[tokio::test]
async fn test_mock_device_service_get_device_not_found() {
    let mock = MockDeviceService::new();

    let result = mock.get_device("unknown").await;
    assert!(matches!(result, Err(DeviceServiceError::DeviceNotFound(_))));
}

#[tokio::test]
async fn test_mock_device_service_call_tracking() {
    let mock = MockDeviceService::new().with_devices(vec![test_device("key")]);

    let _ = mock.list_devices().await;
    let _ = mock.list_devices().await;
    let _ = mock.get_device("key").await;

    assert_eq!(mock.get_call_count("list_devices"), 2);
    assert_eq!(mock.get_call_count("get_device"), 1);
    assert_eq!(mock.get_call_count("set_remap_enabled"), 0);
}

// MockProfileService tests
#[test]
fn test_mock_profile_service_list_virtual_layouts() {
    let layouts = vec![test_layout("1"), test_layout("2")];
    let mock = MockProfileService::new().with_virtual_layouts(layouts);

    let result = mock.list_virtual_layouts().unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(mock.get_call_count("list_virtual_layouts"), 1);
}

#[test]
fn test_mock_profile_service_save_virtual_layout() {
    let mock = MockProfileService::new();

    let layout = test_layout("new");
    let result = mock.save_virtual_layout(layout.clone()).unwrap();
    assert_eq!(result.id, "new");
    assert_eq!(mock.get_call_count("save_virtual_layout"), 1);

    // Verify it was stored
    let layouts = mock.list_virtual_layouts().unwrap();
    assert_eq!(layouts.len(), 1);
}

#[test]
fn test_mock_profile_service_delete_virtual_layout() {
    let layouts = vec![test_layout("1"), test_layout("2")];
    let mock = MockProfileService::new().with_virtual_layouts(layouts);

    mock.delete_virtual_layout("1").unwrap();
    assert_eq!(mock.get_call_count("delete_virtual_layout"), 1);

    let remaining = mock.list_virtual_layouts().unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].id, "2");
}

#[test]
fn test_mock_profile_service_hardware_profiles() {
    let profiles = vec![test_hardware_profile("hp1")];
    let mock = MockProfileService::new().with_hardware_profiles(profiles);

    let result = mock.list_hardware_profiles().unwrap();
    assert_eq!(result.len(), 1);

    let new_profile = test_hardware_profile("hp2");
    mock.save_hardware_profile(new_profile).unwrap();

    let result = mock.list_hardware_profiles().unwrap();
    assert_eq!(result.len(), 2);
}

#[test]
fn test_mock_profile_service_keymaps() {
    let keymaps = vec![test_keymap("km1")];
    let mock = MockProfileService::new().with_keymaps(keymaps);

    let result = mock.list_keymaps().unwrap();
    assert_eq!(result.len(), 1);

    mock.delete_keymap("km1").unwrap();
    let result = mock.list_keymaps().unwrap();
    assert_eq!(result.len(), 0);
}

#[test]
fn test_mock_profile_service_error_handling() {
    let mock = MockProfileService::new().with_list_layouts_error("storage failure");

    let result = mock.list_virtual_layouts();
    assert!(result.is_err());
}

#[test]
fn test_mock_profile_service_call_tracking() {
    let mock = MockProfileService::new();

    let _ = mock.list_virtual_layouts();
    let _ = mock.list_virtual_layouts();
    let _ = mock.list_hardware_profiles();
    let _ = mock.save_keymap(test_keymap("1"));

    assert_eq!(mock.get_call_count("list_virtual_layouts"), 2);
    assert_eq!(mock.get_call_count("list_hardware_profiles"), 1);
    assert_eq!(mock.get_call_count("save_keymap"), 1);
    assert_eq!(mock.get_call_count("delete_keymap"), 0);
}

// MockRuntimeService tests
fn test_device_id() -> DeviceInstanceId {
    DeviceInstanceId {
        vendor_id: 0x1234,
        product_id: 0x5678,
        serial: Some("test-serial".to_string()),
    }
}

fn test_slot(id: &str, priority: u32) -> ProfileSlot {
    ProfileSlot {
        id: id.to_string(),
        hardware_profile_id: "hw-profile-1".to_string(),
        keymap_id: "keymap-1".to_string(),
        active: true,
        priority,
    }
}

#[test]
fn test_mock_runtime_service_get_config() {
    let mock = MockRuntimeService::new();

    let result = mock.get_config().unwrap();
    assert!(result.devices.is_empty());
    assert_eq!(mock.get_call_count("get_config"), 1);
}

#[test]
fn test_mock_runtime_service_add_slot() {
    let mock = MockRuntimeService::new();
    let device = test_device_id();
    let slot = test_slot("slot-1", 100);

    let result = mock.add_slot(device.clone(), slot).unwrap();
    assert_eq!(result.devices.len(), 1);
    assert_eq!(result.devices[0].slots.len(), 1);
    assert_eq!(result.devices[0].slots[0].id, "slot-1");
    assert_eq!(mock.get_call_count("add_slot"), 1);
}

#[test]
fn test_mock_runtime_service_add_slot_sorts_by_priority() {
    let mock = MockRuntimeService::new();
    let device = test_device_id();

    // Add slots in reverse priority order
    mock.add_slot(device.clone(), test_slot("low", 10)).unwrap();
    mock.add_slot(device.clone(), test_slot("high", 100))
        .unwrap();
    mock.add_slot(device.clone(), test_slot("mid", 50)).unwrap();

    let config = mock.get_config().unwrap();
    let slots = &config.devices[0].slots;

    // Should be sorted descending by priority
    assert_eq!(slots[0].id, "high");
    assert_eq!(slots[1].id, "mid");
    assert_eq!(slots[2].id, "low");
}

#[test]
fn test_mock_runtime_service_remove_slot() {
    let mock = MockRuntimeService::new();
    let device = test_device_id();

    mock.add_slot(device.clone(), test_slot("slot-1", 100))
        .unwrap();
    mock.add_slot(device.clone(), test_slot("slot-2", 50))
        .unwrap();

    let result = mock.remove_slot(device.clone(), "slot-1").unwrap();
    assert_eq!(result.devices[0].slots.len(), 1);
    assert_eq!(result.devices[0].slots[0].id, "slot-2");
    assert_eq!(mock.get_call_count("remove_slot"), 1);
}

#[test]
fn test_mock_runtime_service_remove_slot_not_found() {
    let mock = MockRuntimeService::new();
    let device = test_device_id();

    mock.add_slot(device.clone(), test_slot("slot-1", 100))
        .unwrap();

    let result = mock.remove_slot(device.clone(), "nonexistent");
    assert!(matches!(result, Err(RuntimeServiceError::SlotNotFound(_))));
}

#[test]
fn test_mock_runtime_service_remove_slot_device_not_found() {
    let mock = MockRuntimeService::new();
    let device = test_device_id();

    let result = mock.remove_slot(device, "slot-1");
    assert!(matches!(
        result,
        Err(RuntimeServiceError::DeviceNotFound(_))
    ));
}

#[test]
fn test_mock_runtime_service_reorder_slot() {
    let mock = MockRuntimeService::new();
    let device = test_device_id();

    mock.add_slot(device.clone(), test_slot("slot-1", 100))
        .unwrap();
    mock.add_slot(device.clone(), test_slot("slot-2", 50))
        .unwrap();

    // Reorder slot-2 to highest priority
    let result = mock.reorder_slot(device.clone(), "slot-2", 200).unwrap();
    assert_eq!(result.devices[0].slots[0].id, "slot-2");
    assert_eq!(result.devices[0].slots[0].priority, 200);
    assert_eq!(mock.get_call_count("reorder_slot"), 1);
}

#[test]
fn test_mock_runtime_service_set_slot_active() {
    let mock = MockRuntimeService::new();
    let device = test_device_id();

    mock.add_slot(device.clone(), test_slot("slot-1", 100))
        .unwrap();

    let result = mock
        .set_slot_active(device.clone(), "slot-1", false)
        .unwrap();
    assert!(!result.devices[0].slots[0].active);
    assert_eq!(mock.get_call_count("set_slot_active"), 1);

    // Toggle back
    let result = mock
        .set_slot_active(device.clone(), "slot-1", true)
        .unwrap();
    assert!(result.devices[0].slots[0].active);
}

#[test]
fn test_mock_runtime_service_error_handling() {
    let mock = MockRuntimeService::new().with_get_config_error("storage failure");

    let result = mock.get_config();
    assert!(result.is_err());
}

#[test]
fn test_mock_runtime_service_call_tracking() {
    let mock = MockRuntimeService::new();
    let device = test_device_id();

    let _ = mock.get_config();
    let _ = mock.get_config();
    let _ = mock.add_slot(device.clone(), test_slot("1", 100));
    let _ = mock.remove_slot(device.clone(), "1");

    assert_eq!(mock.get_call_count("get_config"), 2);
    assert_eq!(mock.get_call_count("add_slot"), 1);
    assert_eq!(mock.get_call_count("remove_slot"), 1);
    assert_eq!(mock.get_call_count("reorder_slot"), 0);
}
