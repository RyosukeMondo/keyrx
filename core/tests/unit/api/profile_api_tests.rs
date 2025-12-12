//! Unit tests for profile API methods using MockProfileService.
//!
//! These tests verify ApiContext profile operations work correctly with mocked
//! dependencies, enabling fast, isolated testing without I/O.

use std::collections::HashMap;
use std::sync::Arc;

use keyrx_core::api::ApiContext;
use keyrx_core::config::models::{HardwareProfile, Keymap, KeymapLayer, LayoutType, VirtualLayout};
use keyrx_core::services::{MockDeviceService, MockProfileService, MockRuntimeService};

/// Creates a test virtual layout with the given ID.
fn test_layout(id: &str) -> VirtualLayout {
    VirtualLayout {
        id: id.to_string(),
        name: format!("Layout {}", id),
        layout_type: LayoutType::Semantic,
        keys: vec![],
    }
}

/// Creates a test hardware profile with the given ID.
fn test_hardware_profile(id: &str) -> HardwareProfile {
    HardwareProfile {
        id: id.to_string(),
        vendor_id: 0x1234,
        product_id: 0x5678,
        name: Some(format!("Profile {}", id)),
        virtual_layout_id: "layout-1".to_string(),
        wiring: HashMap::new(),
    }
}

/// Creates a test keymap with the given ID.
fn test_keymap(id: &str) -> Keymap {
    Keymap {
        id: id.to_string(),
        name: format!("Keymap {}", id),
        virtual_layout_id: "layout-1".to_string(),
        layers: vec![KeymapLayer {
            name: "base".to_string(),
            bindings: HashMap::new(),
        }],
        combos: vec![],
    }
}

/// Helper to create ApiContext with mocked profile service.
fn create_api_with_profile_mock(profile_mock: MockProfileService) -> ApiContext {
    ApiContext::new(
        Arc::new(MockDeviceService::new()),
        Arc::new(profile_mock),
        Arc::new(MockRuntimeService::new()),
    )
}

#[test]
fn test_list_virtual_layouts_returns_mock_data() {
    let layouts = vec![test_layout("layout-1"), test_layout("layout-2")];

    let mock = MockProfileService::new().with_virtual_layouts(layouts.clone());
    let api = create_api_with_profile_mock(mock);

    let result = api.list_virtual_layouts().unwrap();

    assert_eq!(result.len(), 2);
    assert_eq!(result[0].id, "layout-1");
    assert_eq!(result[1].id, "layout-2");
}

#[test]
fn test_list_virtual_layouts_empty() {
    let mock = MockProfileService::new();
    let api = create_api_with_profile_mock(mock);

    let result = api.list_virtual_layouts().unwrap();

    assert!(result.is_empty());
}

#[test]
fn test_save_virtual_layout_new() {
    let mock = MockProfileService::new();
    let api = create_api_with_profile_mock(mock);

    let layout = test_layout("new-layout");
    let result = api.save_virtual_layout(layout.clone()).unwrap();

    assert_eq!(result.id, "new-layout");
    assert_eq!(result.name, "Layout new-layout");
}

#[test]
fn test_save_virtual_layout_updates_existing() {
    let existing = vec![test_layout("layout-1")];
    let mock = MockProfileService::new().with_virtual_layouts(existing);
    let api = create_api_with_profile_mock(mock);

    let mut updated = test_layout("layout-1");
    updated.name = "Updated Layout".to_string();

    let result = api.save_virtual_layout(updated).unwrap();

    assert_eq!(result.id, "layout-1");
    assert_eq!(result.name, "Updated Layout");
}

#[test]
fn test_delete_virtual_layout_success() {
    let layouts = vec![test_layout("layout-1"), test_layout("layout-2")];
    let mock = MockProfileService::new().with_virtual_layouts(layouts);
    let api = create_api_with_profile_mock(mock);

    let result = api.delete_virtual_layout("layout-1".to_string());

    assert!(result.is_ok());
}

#[test]
fn test_list_hardware_profiles_returns_mock_data() {
    let profiles = vec![
        test_hardware_profile("hp-1"),
        test_hardware_profile("hp-2"),
        test_hardware_profile("hp-3"),
    ];

    let mock = MockProfileService::new().with_hardware_profiles(profiles);
    let api = create_api_with_profile_mock(mock);

    let result = api.list_hardware_profiles().unwrap();

    assert_eq!(result.len(), 3);
    assert_eq!(result[0].id, "hp-1");
    assert_eq!(result[1].id, "hp-2");
    assert_eq!(result[2].id, "hp-3");
}

#[test]
fn test_save_hardware_profile_new() {
    let mock = MockProfileService::new();
    let api = create_api_with_profile_mock(mock);

    let profile = test_hardware_profile("new-profile");
    let result = api.save_hardware_profile(profile).unwrap();

    assert_eq!(result.id, "new-profile");
    assert_eq!(result.name, Some("Profile new-profile".to_string()));
}

#[test]
fn test_delete_hardware_profile_success() {
    let profiles = vec![test_hardware_profile("hp-1")];
    let mock = MockProfileService::new().with_hardware_profiles(profiles);
    let api = create_api_with_profile_mock(mock);

    let result = api.delete_hardware_profile("hp-1".to_string());

    assert!(result.is_ok());
}

#[test]
fn test_list_keymaps_returns_mock_data() {
    let keymaps = vec![test_keymap("km-1"), test_keymap("km-2")];

    let mock = MockProfileService::new().with_keymaps(keymaps);
    let api = create_api_with_profile_mock(mock);

    let result = api.list_keymaps().unwrap();

    assert_eq!(result.len(), 2);
    assert_eq!(result[0].id, "km-1");
    assert_eq!(result[1].id, "km-2");
}

#[test]
fn test_save_keymap_new() {
    let mock = MockProfileService::new();
    let api = create_api_with_profile_mock(mock);

    let keymap = test_keymap("new-keymap");
    let result = api.save_keymap(keymap).unwrap();

    assert_eq!(result.id, "new-keymap");
    assert_eq!(result.name, "Keymap new-keymap");
}

#[test]
fn test_delete_keymap_success() {
    let keymaps = vec![test_keymap("km-1")];
    let mock = MockProfileService::new().with_keymaps(keymaps);
    let api = create_api_with_profile_mock(mock);

    let result = api.delete_keymap("km-1".to_string());

    assert!(result.is_ok());
}

#[test]
fn test_profile_list_layouts_error() {
    let mock = MockProfileService::new().with_list_layouts_error("storage failure");
    let api = create_api_with_profile_mock(mock);

    let result = api.list_virtual_layouts();

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("storage failure"));
}

#[test]
fn test_profile_save_layout_error() {
    let mock = MockProfileService::new().with_save_layout_error("write failed");
    let api = create_api_with_profile_mock(mock);

    let layout = test_layout("new-layout");
    let result = api.save_virtual_layout(layout);

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("write failed"));
}

#[test]
fn test_profile_delete_layout_error() {
    let mock = MockProfileService::new().with_delete_layout_error("delete failed");
    let api = create_api_with_profile_mock(mock);

    let result = api.delete_virtual_layout("any-id".to_string());

    assert!(result.is_err());
}

#[test]
fn test_profile_list_profiles_error() {
    let mock = MockProfileService::new().with_list_profiles_error("disk error");
    let api = create_api_with_profile_mock(mock);

    let result = api.list_hardware_profiles();

    assert!(result.is_err());
}

#[test]
fn test_profile_keymaps_error() {
    let mock = MockProfileService::new().with_list_keymaps_error("io error");
    let api = create_api_with_profile_mock(mock);

    let result = api.list_keymaps();

    assert!(result.is_err());
}

#[test]
fn test_mock_tracks_profile_method_calls() {
    let mock = Arc::new(MockProfileService::new());
    let api = ApiContext::new(
        Arc::new(MockDeviceService::new()),
        mock.clone(),
        Arc::new(MockRuntimeService::new()),
    );

    // Perform multiple operations
    let _ = api.list_virtual_layouts();
    let _ = api.list_virtual_layouts();
    let _ = api.save_virtual_layout(test_layout("1"));
    let _ = api.delete_virtual_layout("1".to_string());
    let _ = api.list_hardware_profiles();
    let _ = api.save_hardware_profile(test_hardware_profile("hp-1"));
    let _ = api.delete_hardware_profile("hp-1".to_string());
    let _ = api.list_keymaps();
    let _ = api.save_keymap(test_keymap("km-1"));
    let _ = api.delete_keymap("km-1".to_string());

    // Verify call counts
    assert_eq!(mock.get_call_count("list_virtual_layouts"), 2);
    assert_eq!(mock.get_call_count("save_virtual_layout"), 1);
    assert_eq!(mock.get_call_count("delete_virtual_layout"), 1);
    assert_eq!(mock.get_call_count("list_hardware_profiles"), 1);
    assert_eq!(mock.get_call_count("save_hardware_profile"), 1);
    assert_eq!(mock.get_call_count("delete_hardware_profile"), 1);
    assert_eq!(mock.get_call_count("list_keymaps"), 1);
    assert_eq!(mock.get_call_count("save_keymap"), 1);
    assert_eq!(mock.get_call_count("delete_keymap"), 1);
}

#[test]
fn test_crud_workflow_virtual_layouts() {
    let mock = MockProfileService::new();
    let api = create_api_with_profile_mock(mock);

    // Create
    let layout = test_layout("workflow-layout");
    let saved = api.save_virtual_layout(layout).unwrap();
    assert_eq!(saved.id, "workflow-layout");

    // Read
    let layouts = api.list_virtual_layouts().unwrap();
    assert_eq!(layouts.len(), 1);

    // Update
    let mut updated = test_layout("workflow-layout");
    updated.name = "Updated Name".to_string();
    let saved = api.save_virtual_layout(updated).unwrap();
    assert_eq!(saved.name, "Updated Name");

    // Verify still one item (updated, not added)
    let layouts = api.list_virtual_layouts().unwrap();
    assert_eq!(layouts.len(), 1);

    // Delete
    api.delete_virtual_layout("workflow-layout".to_string())
        .unwrap();
    let layouts = api.list_virtual_layouts().unwrap();
    assert!(layouts.is_empty());
}

#[test]
fn test_api_context_accepts_trait_objects_for_profile() {
    // Verify ApiContext::new() correctly accepts Arc<dyn ProfileServiceTrait>
    let profile_mock: Arc<dyn keyrx_core::services::ProfileServiceTrait> =
        Arc::new(MockProfileService::new().with_virtual_layouts(vec![test_layout("test")]));

    let api = ApiContext::new(
        Arc::new(MockDeviceService::new()),
        profile_mock,
        Arc::new(MockRuntimeService::new()),
    );

    let result = api.list_virtual_layouts().unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id, "test");
}
