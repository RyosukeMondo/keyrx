#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::print_stdout, clippy::print_stderr, clippy::field_reassign_with_default, clippy::useless_conversion, clippy::assertions_on_constants, clippy::manual_div_ceil, clippy::manual_strip, clippy::len_zero, clippy::redundant_closure, clippy::manual_range_contains, clippy::default_constructed_unit_structs, clippy::clone_on_copy, clippy::io_other_error, clippy::bool_assert_comparison, clippy::approx_constant, clippy::let_unit_value, clippy::while_let_on_iterator, clippy::await_holding_lock, clippy::unnecessary_cast, clippy::drop_non_drop, clippy::needless_range_loop, unused_imports, unused_variables, dead_code, unsafe_code, clippy::collapsible_if, clippy::bool_comparison, unexpected_cfgs)]
//! Comprehensive tests for the registry module.
//!
//! This test suite covers:
//! - DeviceRegistry CRUD operations and concurrent access
//! - ProfileRegistry save/load/validation
//! - DeviceBindings persistence and corruption recovery
//! - Event emission from DeviceRegistry
//! - Thread safety and concurrent access scenarios

use keyrx_core::drivers::keycodes::KeyCode;
use keyrx_core::identity::DeviceIdentity;
use keyrx_core::registry::{
    DeviceBinding, DeviceBindings, DeviceEvent, DeviceRegistry, DeviceState, KeyAction, LayoutType,
    PhysicalPosition, Profile, ProfileRegistry,
};
use serial_test::serial;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::sync::Barrier;

// ============================================================================
// Helper Functions
// ============================================================================

fn test_identity(serial: &str) -> DeviceIdentity {
    DeviceIdentity::new(0x1234, 0x5678, serial.to_string())
}

fn test_profile(name: &str, layout: LayoutType) -> Profile {
    Profile::new(name, layout)
}

// ============================================================================
// DeviceRegistry CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_device_registry_register() {
    let (registry, mut rx) = DeviceRegistry::new();
    let identity = test_identity("TEST001");

    let result = registry.register_device(identity.clone()).await;
    assert!(result, "First registration should return true");

    // Verify event emission
    let event = rx.recv().await.expect("Should receive event");
    assert_eq!(
        event,
        DeviceEvent::Registered {
            identity: identity.clone()
        }
    );

    // Verify device is registered
    assert!(registry.is_registered(&identity).await);
    assert_eq!(registry.device_count().await, 1);

    // Verify device state
    let state = registry.get_device_state(&identity).await.unwrap();
    assert_eq!(state.identity, identity);
    assert!(!state.remap_enabled);
    assert_eq!(state.profile_id, None);
}

#[tokio::test]
async fn test_device_registry_register_idempotent() {
    let (registry, mut rx) = DeviceRegistry::new();
    let identity = test_identity("TEST001");

    let result1 = registry.register_device(identity.clone()).await;
    let result2 = registry.register_device(identity.clone()).await;

    assert!(result1, "First registration should succeed");
    assert!(!result2, "Second registration should return false");
    assert_eq!(registry.device_count().await, 1);

    // Only one event should be emitted
    let _ = rx.recv().await.unwrap();
    assert!(rx.try_recv().is_err(), "No second event should be emitted");
}

#[tokio::test]
async fn test_device_registry_unregister() {
    let (registry, mut rx) = DeviceRegistry::new();
    let identity = test_identity("TEST001");

    registry.register_device(identity.clone()).await;
    let _ = rx.recv().await; // Consume register event

    let result = registry.unregister_device(&identity).await;
    assert!(result, "Unregister should return true");

    // Verify event
    let event = rx.recv().await.unwrap();
    assert_eq!(
        event,
        DeviceEvent::Unregistered {
            identity: identity.clone()
        }
    );

    assert!(!registry.is_registered(&identity).await);
    assert_eq!(registry.device_count().await, 0);
}

#[tokio::test]
async fn test_device_registry_unregister_nonexistent() {
    let (registry, _rx) = DeviceRegistry::new();
    let identity = test_identity("NONEXISTENT");

    let result = registry.unregister_device(&identity).await;
    assert!(
        !result,
        "Unregister of nonexistent device should return false"
    );
}

#[tokio::test]
async fn test_device_registry_set_remap_enabled() {
    let (registry, mut rx) = DeviceRegistry::new();
    let identity = test_identity("TEST001");

    registry.register_device(identity.clone()).await;
    let _ = rx.recv().await;

    // Enable remap
    registry.set_remap_enabled(&identity, true).await.unwrap();

    // Verify event
    let event = rx.recv().await.unwrap();
    assert_eq!(
        event,
        DeviceEvent::RemapStateChanged {
            identity: identity.clone(),
            enabled: true
        }
    );

    // Verify state
    let state = registry.get_device_state(&identity).await.unwrap();
    assert!(state.remap_enabled);
}

#[tokio::test]
async fn test_device_registry_set_remap_no_change_no_event() {
    let (registry, mut rx) = DeviceRegistry::new();
    let identity = test_identity("TEST001");

    registry.register_device(identity.clone()).await;
    let _ = rx.recv().await;

    // Set to false (already false by default)
    registry.set_remap_enabled(&identity, false).await.unwrap();

    // No event should be emitted
    assert!(
        rx.try_recv().is_err(),
        "No event should be emitted when state doesn't change"
    );
}

#[tokio::test]
async fn test_device_registry_assign_profile() {
    let (registry, mut rx) = DeviceRegistry::new();
    let identity = test_identity("TEST001");
    let profile_id = "profile-123".to_string();

    registry.register_device(identity.clone()).await;
    let _ = rx.recv().await;

    registry
        .assign_profile(&identity, profile_id.clone())
        .await
        .unwrap();

    // Verify event
    let event = rx.recv().await.unwrap();
    assert_eq!(
        event,
        DeviceEvent::ProfileAssigned {
            identity: identity.clone(),
            profile_id: profile_id.clone()
        }
    );

    // Verify state
    let state = registry.get_device_state(&identity).await.unwrap();
    assert_eq!(state.profile_id, Some(profile_id));
}

#[tokio::test]
async fn test_device_registry_assign_profile_same_no_event() {
    let (registry, mut rx) = DeviceRegistry::new();
    let identity = test_identity("TEST001");
    let profile_id = "profile-123".to_string();

    registry.register_device(identity.clone()).await;
    let _ = rx.recv().await;

    // Assign profile twice
    registry
        .assign_profile(&identity, profile_id.clone())
        .await
        .unwrap();
    let _ = rx.recv().await;

    registry
        .assign_profile(&identity, profile_id.clone())
        .await
        .unwrap();

    // No second event should be emitted
    assert!(
        rx.try_recv().is_err(),
        "No event when assigning same profile"
    );
}

#[tokio::test]
async fn test_device_registry_unassign_profile() {
    let (registry, mut rx) = DeviceRegistry::new();
    let identity = test_identity("TEST001");
    let profile_id = "profile-123".to_string();

    registry.register_device(identity.clone()).await;
    let _ = rx.recv().await;

    registry
        .assign_profile(&identity, profile_id)
        .await
        .unwrap();
    let _ = rx.recv().await;

    registry.unassign_profile(&identity).await.unwrap();

    // Verify event
    let event = rx.recv().await.unwrap();
    assert_eq!(
        event,
        DeviceEvent::ProfileUnassigned {
            identity: identity.clone()
        }
    );

    // Verify state
    let state = registry.get_device_state(&identity).await.unwrap();
    assert_eq!(state.profile_id, None);
}

#[tokio::test]
async fn test_device_registry_set_user_label() {
    let (registry, mut rx) = DeviceRegistry::new();
    let identity = test_identity("TEST001");
    let label = Some("My Keyboard".to_string());

    registry.register_device(identity.clone()).await;
    let _ = rx.recv().await;

    registry
        .set_user_label(&identity, label.clone())
        .await
        .unwrap();

    // Verify event
    let event = rx.recv().await.unwrap();
    assert_eq!(
        event,
        DeviceEvent::LabelChanged {
            identity: identity.clone(),
            label: label.clone()
        }
    );

    // Verify state
    let state = registry.get_device_state(&identity).await.unwrap();
    assert_eq!(state.identity.user_label, label);
}

#[tokio::test]
async fn test_device_registry_list_devices_sorted() {
    let (registry, _rx) = DeviceRegistry::new();

    let id1 = test_identity("TEST001");
    let id2 = test_identity("TEST002");
    let id3 = test_identity("TEST003");

    registry.register_device(id1.clone()).await;
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    registry.register_device(id2.clone()).await;
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    registry.register_device(id3.clone()).await;

    let devices = registry.list_devices().await;
    assert_eq!(devices.len(), 3);

    // Should be sorted by connection time
    assert_eq!(devices[0].identity, id1);
    assert_eq!(devices[1].identity, id2);
    assert_eq!(devices[2].identity, id3);
}

#[tokio::test]
async fn test_device_registry_get_device_state_nonexistent() {
    let (registry, _rx) = DeviceRegistry::new();
    let identity = test_identity("NONEXISTENT");

    let state = registry.get_device_state(&identity).await;
    assert!(state.is_none());
}

#[tokio::test]
async fn test_device_registry_timestamps_update() {
    let (registry, _rx) = DeviceRegistry::new();
    let identity = test_identity("TEST001");

    registry.register_device(identity.clone()).await;
    let state1 = registry.get_device_state(&identity).await.unwrap();

    // Sleep to ensure timestamp changes
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    registry.set_remap_enabled(&identity, true).await.unwrap();
    let state2 = registry.get_device_state(&identity).await.unwrap();

    // connected_at should not change
    assert_eq!(state1.connected_at, state2.connected_at);

    // updated_at should change
    assert_ne!(state1.updated_at, state2.updated_at);
}

// ============================================================================
// DeviceRegistry Concurrent Access Tests
// ============================================================================

#[tokio::test]
async fn test_device_registry_concurrent_register() {
    let (registry, _rx) = DeviceRegistry::new();
    let mut handles = vec![];

    // Spawn 20 tasks that concurrently register devices
    for i in 0..20 {
        let reg = registry.clone();
        let handle = tokio::spawn(async move {
            let identity = test_identity(&format!("CONCURRENT{:03}", i));
            reg.register_device(identity).await
        });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // All 20 devices should be registered
    assert_eq!(registry.device_count().await, 20);
}

#[tokio::test]
async fn test_device_registry_concurrent_read_write() {
    let (registry, _rx) = DeviceRegistry::new();
    let identity = test_identity("CONCURRENT");

    registry.register_device(identity.clone()).await;

    let barrier = Arc::new(Barrier::new(10));
    let mut handles = vec![];

    // Spawn 10 tasks: 5 readers, 5 writers
    for i in 0..10 {
        let reg = registry.clone();
        let id = identity.clone();
        let bar = Arc::clone(&barrier);

        let handle = tokio::spawn(async move {
            // Wait for all tasks to be ready
            bar.wait().await;

            if i < 5 {
                // Reader: just get state
                reg.get_device_state(&id).await
            } else {
                // Writer: toggle remap
                let _ = reg.set_remap_enabled(&id, i % 2 == 0).await;
                reg.get_device_state(&id).await
            }
        });
        handles.push(handle);
    }

    // Wait for all to complete - should not deadlock
    for handle in handles {
        handle.await.unwrap();
    }

    // Device should still be registered
    assert!(registry.is_registered(&identity).await);
}

#[tokio::test]
async fn test_device_registry_concurrent_same_device() {
    let (registry, _rx) = DeviceRegistry::new();
    let identity = test_identity("SAME");

    let mut handles = vec![];

    // Try to register the same device from 10 tasks
    for _ in 0..10 {
        let reg = registry.clone();
        let id = identity.clone();
        let handle = tokio::spawn(async move { reg.register_device(id).await });
        handles.push(handle);
    }

    let mut success_count = 0;
    for handle in handles {
        if handle.await.unwrap() {
            success_count += 1;
        }
    }

    // Only one should succeed
    assert_eq!(success_count, 1);
    assert_eq!(registry.device_count().await, 1);
}

// ============================================================================
// ProfileRegistry Save/Load Tests
// ============================================================================

#[tokio::test]
#[serial]
async fn test_profile_registry_save_and_get() {
    let temp = tempdir().unwrap();
    let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

    let mut profile = test_profile("Test Profile", LayoutType::Matrix);
    profile.set_action(PhysicalPosition::new(0, 0), KeyAction::key(KeyCode::A));

    // Save profile
    registry.save_profile(&profile).await.unwrap();

    // Get profile back
    let loaded = registry.get_profile(&profile.id).await.unwrap();
    assert_eq!(loaded.id, profile.id);
    assert_eq!(loaded.name, profile.name);
    assert_eq!(loaded.layout_type, profile.layout_type);
    assert_eq!(loaded.mappings.len(), 1);
}

#[tokio::test]
#[serial]
async fn test_profile_registry_cache_hit() {
    let temp = tempdir().unwrap();
    let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

    let profile = test_profile("Cached", LayoutType::Standard);
    registry.save_profile(&profile).await.unwrap();

    // First get - cold cache
    let arc1 = registry.get_profile(&profile.id).await.unwrap();

    // Second get - should hit cache and return same Arc
    let arc2 = registry.get_profile(&profile.id).await.unwrap();

    assert!(
        Arc::ptr_eq(&arc1, &arc2),
        "Should return same Arc from cache"
    );
}

#[tokio::test]
#[serial]
async fn test_profile_registry_delete() {
    let temp = tempdir().unwrap();
    let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

    let profile = test_profile("To Delete", LayoutType::Matrix);
    registry.save_profile(&profile).await.unwrap();

    // Verify it exists
    assert!(registry.get_profile(&profile.id).await.is_ok());

    // Delete it
    registry.delete_profile(&profile.id).await.unwrap();

    // Verify it's gone
    assert!(registry.get_profile(&profile.id).await.is_err());
    assert_eq!(registry.cache_size().await, 0);
}

#[tokio::test]
#[serial]
async fn test_profile_registry_list_profiles() {
    let temp = tempdir().unwrap();
    let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

    let profile1 = test_profile("Profile 1", LayoutType::Matrix);
    let profile2 = test_profile("Profile 2", LayoutType::Standard);
    let profile3 = test_profile("Profile 3", LayoutType::Split);

    registry.save_profile(&profile1).await.unwrap();
    registry.save_profile(&profile2).await.unwrap();
    registry.save_profile(&profile3).await.unwrap();

    let list = registry.list_profiles().await;
    assert_eq!(list.len(), 3);
    assert!(list.contains(&profile1.id));
    assert!(list.contains(&profile2.id));
    assert!(list.contains(&profile3.id));
}

#[tokio::test]
#[serial]
async fn test_profile_registry_find_compatible() {
    let temp = tempdir().unwrap();
    let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

    let matrix1 = test_profile("Matrix 1", LayoutType::Matrix);
    let matrix2 = test_profile("Matrix 2", LayoutType::Matrix);
    let standard = test_profile("Standard", LayoutType::Standard);

    registry.save_profile(&matrix1).await.unwrap();
    registry.save_profile(&matrix2).await.unwrap();
    registry.save_profile(&standard).await.unwrap();

    let compatible = registry.find_compatible_profiles(&LayoutType::Matrix).await;
    assert_eq!(compatible.len(), 2);

    let ids: Vec<String> = compatible.iter().map(|p| p.id.clone()).collect();
    assert!(ids.contains(&matrix1.id));
    assert!(ids.contains(&matrix2.id));
    assert!(!ids.contains(&standard.id));
}

#[tokio::test]
#[serial]
async fn test_profile_registry_load_all() {
    let temp = tempdir().unwrap();
    let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

    // Save some profiles
    let profile1 = test_profile("Profile 1", LayoutType::Matrix);
    let profile2 = test_profile("Profile 2", LayoutType::Standard);

    registry.save_profile(&profile1).await.unwrap();
    registry.save_profile(&profile2).await.unwrap();

    // Clear cache
    registry.clear_cache().await;
    assert_eq!(registry.cache_size().await, 0);

    // Load all profiles
    let count = registry.load_all_profiles().await.unwrap();
    assert_eq!(count, 2);
    assert_eq!(registry.cache_size().await, 2);
}

#[tokio::test]
#[serial]
async fn test_profile_registry_atomic_write() {
    let temp = tempdir().unwrap();
    let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

    let profile = test_profile("Atomic", LayoutType::Matrix);
    let profile_path = temp.path().join(format!("{}.json", profile.id));
    let tmp_path = profile_path.with_extension("json.tmp");

    registry.save_profile(&profile).await.unwrap();

    // Verify final file exists and temp file is cleaned up
    assert!(profile_path.exists());
    assert!(!tmp_path.exists());
}

// ============================================================================
// ProfileRegistry Validation Tests
// ============================================================================

#[tokio::test]
#[serial]
async fn test_profile_registry_validation_empty_name() {
    let temp = tempdir().unwrap();
    let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

    let mut profile = test_profile("Test", LayoutType::Matrix);
    profile.name = "".to_string();

    let result = registry.save_profile(&profile).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("empty"));
}

#[tokio::test]
#[serial]
async fn test_profile_registry_validation_invalid_uuid() {
    let temp = tempdir().unwrap();
    let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

    let mut profile = test_profile("Test", LayoutType::Matrix);
    profile.id = "not-a-uuid".to_string();

    let result = registry.save_profile(&profile).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("UUID"));
}

#[tokio::test]
#[serial]
async fn test_profile_registry_validation_invalid_timestamp() {
    let temp = tempdir().unwrap();
    let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

    let mut profile = test_profile("Test", LayoutType::Matrix);
    profile.created_at = "not-a-timestamp".to_string();

    let result = registry.save_profile(&profile).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("timestamp"));
}

#[tokio::test]
#[serial]
async fn test_profile_registry_cache_invalidation() {
    let temp = tempdir().unwrap();
    let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

    let profile = test_profile("Cache Test", LayoutType::Matrix);
    registry.save_profile(&profile).await.unwrap();

    // Profile should be in cache
    assert_eq!(registry.cache_size().await, 1);

    // Invalidate cache
    registry.invalidate_cache(&profile.id).await;
    assert_eq!(registry.cache_size().await, 0);

    // Should still be able to load from disk
    let loaded = registry.get_profile(&profile.id).await.unwrap();
    assert_eq!(loaded.id, profile.id);
    assert_eq!(registry.cache_size().await, 1);
}

#[tokio::test]
#[serial]
async fn test_profile_registry_corrupted_file_skip() {
    let temp = tempdir().unwrap();
    let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

    // Create a valid profile
    let valid = test_profile("Valid", LayoutType::Matrix);
    registry.save_profile(&valid).await.unwrap();

    // Create a corrupted file
    let corrupted_path = temp.path().join("corrupted.json");
    std::fs::write(&corrupted_path, "not valid json").unwrap();

    // Clear cache and reload
    registry.clear_cache().await;
    let count = registry.load_all_profiles().await.unwrap();

    // Should load only the valid profile
    assert_eq!(count, 1);
}

// ============================================================================
// DeviceBindings Persistence Tests
// ============================================================================

#[test]
fn test_device_bindings_new() {
    let bindings = DeviceBindings::new();
    assert!(bindings.is_empty());
    assert_eq!(bindings.len(), 0);
}

#[test]
fn test_device_bindings_set_and_get() {
    let mut bindings = DeviceBindings::new();
    let device = test_identity("TEST001");
    let binding = DeviceBinding::with_profile("profile-123".to_string());

    bindings.set_binding(device.clone(), binding.clone());

    let mut retrieved = bindings
        .get_binding(&device)
        .cloned()
        .expect("binding should be present");
    assert!(
        retrieved.bound_at.is_some(),
        "binding should record bound_at timestamp"
    );
    retrieved.bound_at = None;

    let mut expected = binding.clone();
    expected.bound_at = None;
    assert_eq!(retrieved, expected);
    assert_eq!(bindings.len(), 1);
}

#[test]
fn test_device_bindings_remove() {
    let mut bindings = DeviceBindings::new();
    let device = test_identity("TEST001");
    let binding = DeviceBinding::new();

    bindings.set_binding(device.clone(), binding.clone());
    assert_eq!(bindings.len(), 1);

    let mut removed = bindings
        .remove_binding(&device)
        .expect("binding should be removed");
    assert!(
        removed.bound_at.is_some(),
        "removal should return binding with timestamp"
    );
    removed.bound_at = None;

    let mut expected = binding;
    expected.bound_at = None;
    assert_eq!(removed, expected);
    assert_eq!(bindings.len(), 0);
}

#[test]
fn test_device_bindings_save_and_load() {
    let temp = tempdir().unwrap();
    let file_path = temp.path().join("bindings.json");

    // Create and populate bindings
    let mut bindings = DeviceBindings::with_path(file_path.clone());
    let device1 = test_identity("ABC123");
    let device2 = test_identity("XYZ789");

    bindings.set_binding(
        device1.clone(),
        DeviceBinding::with_profile("profile-1".to_string()),
    );
    bindings.set_binding(device2.clone(), DeviceBinding::disabled());

    // Save
    bindings.save().unwrap();
    assert!(file_path.exists());

    // Load into new instance
    let mut loaded = DeviceBindings::with_path(file_path);
    loaded.load().unwrap();

    assert_eq!(loaded.len(), 2);
    assert_eq!(
        loaded.get_binding(&device1).unwrap().profile_id,
        Some("profile-1".to_string())
    );
    assert!(!loaded.get_binding(&device2).unwrap().remap_enabled);
}

#[test]
fn test_device_bindings_load_nonexistent() {
    let temp = tempdir().unwrap();
    let file_path = temp.path().join("nonexistent.json");

    let mut bindings = DeviceBindings::with_path(file_path);
    let result = bindings.load();

    assert!(result.is_ok());
    assert!(bindings.is_empty());
}

// ============================================================================
// DeviceBindings Corruption Recovery Tests
// ============================================================================

#[test]
fn test_device_bindings_corrupted_file_recovery() {
    let temp = tempdir().unwrap();
    let file_path = temp.path().join("device_bindings.json");

    // Write invalid JSON
    std::fs::write(&file_path, b"{ this is not valid json }").unwrap();

    let mut bindings = DeviceBindings::with_path(file_path.clone());
    let result = bindings.load();

    // Should succeed but with empty bindings
    assert!(result.is_ok());
    assert!(bindings.is_empty());

    // Backup should be created
    let backup_files: Vec<_> = std::fs::read_dir(temp.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.starts_with("device_bindings.") && name.ends_with(".bak")
        })
        .collect();

    assert_eq!(backup_files.len(), 1, "Backup file should be created");
}

#[test]
fn test_device_bindings_atomic_write() {
    let temp = tempdir().unwrap();
    let file_path = temp.path().join("atomic.json");

    let mut bindings = DeviceBindings::with_path(file_path.clone());
    let device = test_identity("TEST001");
    bindings.set_binding(device, DeviceBinding::new());

    bindings.save().unwrap();

    // Verify temp file was cleaned up
    let tmp_path = file_path.with_extension("json.tmp");
    assert!(!tmp_path.exists(), "Temp file should be cleaned up");

    // Verify final file exists
    assert!(file_path.exists(), "Final file should exist");
}

#[test]
fn test_device_bindings_clear() {
    let mut bindings = DeviceBindings::new();
    let device1 = test_identity("TEST001");
    let device2 = test_identity("TEST002");

    bindings.set_binding(device1, DeviceBinding::new());
    bindings.set_binding(device2, DeviceBinding::new());
    assert_eq!(bindings.len(), 2);

    bindings.clear();
    assert!(bindings.is_empty());
}

#[test]
fn test_device_bindings_serialization_format() {
    let temp = tempdir().unwrap();
    let file_path = temp.path().join("format.json");

    let mut bindings = DeviceBindings::with_path(file_path.clone());
    let device = test_identity("ABC123");
    bindings.set_binding(
        device,
        DeviceBinding::with_profile("test-profile".to_string()),
    );

    bindings.save().unwrap();

    // Read and verify JSON structure
    let content = std::fs::read_to_string(&file_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert!(json.is_object());
    assert!(json.get("1234:5678:ABC123").is_some());

    let binding_json = &json["1234:5678:ABC123"];
    assert_eq!(binding_json["profile_id"], "test-profile");
    assert_eq!(binding_json["remap_enabled"], true);
}

#[test]
fn test_device_bindings_multiple_devices() {
    let temp = tempdir().unwrap();
    let file_path = temp.path().join("multiple.json");

    let mut bindings = DeviceBindings::with_path(file_path.clone());

    // Add bindings for 5 different devices
    for i in 0..5 {
        let device = test_identity(&format!("DEVICE{:03}", i));
        let binding = DeviceBinding::with_profile(format!("profile-{}", i));
        bindings.set_binding(device, binding);
    }

    assert_eq!(bindings.len(), 5);

    // Save and reload
    bindings.save().unwrap();

    let mut loaded = DeviceBindings::with_path(file_path);
    loaded.load().unwrap();

    assert_eq!(loaded.len(), 5);

    // Verify all devices are present
    for i in 0..5 {
        let device = test_identity(&format!("DEVICE{:03}", i));
        let binding = loaded.get_binding(&device);
        assert!(binding.is_some());
        assert_eq!(binding.unwrap().profile_id, Some(format!("profile-{}", i)));
    }
}

// ============================================================================
// Integration Tests: DeviceRegistry + ProfileRegistry + DeviceBindings
// ============================================================================

#[tokio::test]
#[serial]
async fn test_integration_full_workflow() {
    let temp = tempdir().unwrap();

    // Setup registries
    let (device_registry, _rx) = DeviceRegistry::new();
    let profile_registry = ProfileRegistry::with_directory(temp.path().join("profiles"));
    let mut device_bindings = DeviceBindings::with_path(temp.path().join("bindings.json"));

    // Create and save a profile
    let profile = test_profile("Gaming Profile", LayoutType::Standard);
    profile_registry.save_profile(&profile).await.unwrap();

    // Register a device
    let device = test_identity("KEYBOARD001");
    device_registry.register_device(device.clone()).await;

    // Assign profile to device in registry
    device_registry
        .assign_profile(&device, profile.id.clone())
        .await
        .unwrap();

    // Enable remap
    device_registry
        .set_remap_enabled(&device, true)
        .await
        .unwrap();

    // Save binding persistently
    let binding = DeviceBinding::with_profile(profile.id.clone());
    device_bindings.set_binding(device.clone(), binding);
    device_bindings.save().unwrap();

    // Verify everything is connected
    let state = device_registry.get_device_state(&device).await.unwrap();
    assert_eq!(state.profile_id, Some(profile.id.clone()));
    assert!(state.remap_enabled);

    let loaded_profile = profile_registry.get_profile(&profile.id).await.unwrap();
    assert_eq!(loaded_profile.name, "Gaming Profile");

    let loaded_binding = device_bindings.get_binding(&device).unwrap();
    assert_eq!(loaded_binding.profile_id, Some(profile.id));
}

#[tokio::test]
#[serial]
async fn test_integration_device_reconnect_restores_binding() {
    let temp = tempdir().unwrap();

    let (device_registry, _rx) = DeviceRegistry::new();
    let profile_registry = ProfileRegistry::with_directory(temp.path().join("profiles"));
    let mut device_bindings = DeviceBindings::with_path(temp.path().join("bindings.json"));

    // Setup: Create profile and binding
    let profile = test_profile("Work Profile", LayoutType::Matrix);
    profile_registry.save_profile(&profile).await.unwrap();

    let device = test_identity("MACROPAD001");
    let binding = DeviceBinding::with_profile(profile.id.clone());
    device_bindings.set_binding(device.clone(), binding);
    device_bindings.save().unwrap();

    // Simulate device disconnect and reconnect
    device_registry.register_device(device.clone()).await;

    // On reconnect, restore binding from persistent storage
    let mut reloaded_bindings = DeviceBindings::with_path(temp.path().join("bindings.json"));
    reloaded_bindings.load().unwrap();

    if let Some(binding) = reloaded_bindings.get_binding(&device) {
        if let Some(profile_id) = &binding.profile_id {
            device_registry
                .assign_profile(&device, profile_id.clone())
                .await
                .unwrap();
            device_registry
                .set_remap_enabled(&device, binding.remap_enabled)
                .await
                .unwrap();
        }
    }

    // Verify state is restored
    let state = device_registry.get_device_state(&device).await.unwrap();
    assert_eq!(state.profile_id, Some(profile.id));
    assert!(state.remap_enabled);
}
