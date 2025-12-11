#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::field_reassign_with_default,
    clippy::useless_conversion,
    clippy::assertions_on_constants,
    clippy::manual_div_ceil,
    clippy::manual_strip,
    clippy::len_zero,
    clippy::redundant_closure,
    clippy::manual_range_contains,
    clippy::default_constructed_unit_structs,
    clippy::clone_on_copy,
    clippy::io_other_error,
    clippy::bool_assert_comparison,
    clippy::approx_constant,
    clippy::let_unit_value,
    clippy::while_let_on_iterator,
    clippy::await_holding_lock,
    clippy::unnecessary_cast,
    clippy::drop_non_drop,
    clippy::needless_range_loop,
    unused_imports,
    unused_variables,
    dead_code,
    unsafe_code,
    clippy::collapsible_if,
    clippy::bool_comparison,
    unexpected_cfgs
)]
//! End-to-end integration tests for the Revolutionary Mapping feature.
//!
//! These tests simulate complete user workflows from device connection
//! through to profile assignment and remapping, verifying that all components
//! work together correctly:
//!
//! - DeviceRegistry (runtime device state)
//! - ProfileRegistry (profile storage)
//! - DeviceBindings (persistent device-profile bindings)
//! - DeviceDefinitionLibrary (device layouts)
//! - Pipeline components (DeviceResolver, ProfileResolver, CoordinateTranslator)
//!
//! Test scenarios:
//! 1. Two identical devices with different profiles (isolation)
//! 2. Profile swapping on a live device
//! 3. Per-device remap toggle (passthrough vs active)
//! 4. Device disconnect/reconnect with binding persistence
//! 5. Macro pad with custom 5×5 matrix layout

use keyrx_core::definitions::library::DeviceDefinitionLibrary;
use keyrx_core::drivers::keycodes::KeyCode;
use keyrx_core::engine::{CoordinateTranslator, DeviceResolver, InputEvent, ProfileResolver};
use keyrx_core::identity::DeviceIdentity;
use keyrx_core::registry::{
    DeviceBinding, DeviceBindings, DeviceRegistry, KeyAction, LayoutType, PhysicalPosition,
    Profile, ProfileRegistry,
};
use serial_test::serial;
use std::sync::Arc;
use tempfile::TempDir;

// ============================================================================
// Test Fixtures and Helpers
// ============================================================================

/// Create a test device definition library with a 5×5 macro pad
fn create_macro_pad_definition() -> (TempDir, Arc<DeviceDefinitionLibrary>) {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("macro-pad-5x5.toml");

    let toml_content = r#"
name = "Generic 5×5 Macro Pad"
vendor_id = 0xFEED
product_id = 0xBEEF
manufacturer = "Custom Devices"

[layout]
layout_type = "matrix"
rows = 5
cols = 5

[matrix_map]
"1" = { row = 0, col = 0 }
"2" = { row = 0, col = 1 }
"3" = { row = 0, col = 2 }
"4" = { row = 0, col = 3 }
"5" = { row = 0, col = 4 }
"6" = { row = 1, col = 0 }
"7" = { row = 1, col = 1 }
"8" = { row = 1, col = 2 }
"9" = { row = 1, col = 3 }
"10" = { row = 1, col = 4 }
"11" = { row = 2, col = 0 }
"12" = { row = 2, col = 1 }
"13" = { row = 2, col = 2 }
"14" = { row = 2, col = 3 }
"15" = { row = 2, col = 4 }
"16" = { row = 3, col = 0 }
"17" = { row = 3, col = 1 }
"18" = { row = 3, col = 2 }
"19" = { row = 3, col = 3 }
"20" = { row = 3, col = 4 }
"21" = { row = 4, col = 0 }
"22" = { row = 4, col = 1 }
"23" = { row = 4, col = 2 }
"24" = { row = 4, col = 3 }
"25" = { row = 4, col = 4 }

[visual]
key_width = 60
key_height = 60
key_spacing = 4
"#;

    std::fs::write(&file_path, toml_content).unwrap();

    let mut library = DeviceDefinitionLibrary::new();
    library.load_from_directory(temp_dir.path()).unwrap();

    (temp_dir, Arc::new(library))
}

/// Create a test profile for a 5×5 macro pad with specific mappings
fn create_macro_pad_profile(name: &str) -> Profile {
    let mut profile = Profile::new(name, LayoutType::Matrix);

    // Map buttons to various actions
    // Top row: number keys 1-5
    profile.set_action(PhysicalPosition::new(0, 0), KeyAction::key(KeyCode::Key1));
    profile.set_action(PhysicalPosition::new(0, 1), KeyAction::key(KeyCode::Key2));
    profile.set_action(PhysicalPosition::new(0, 2), KeyAction::key(KeyCode::Key3));
    profile.set_action(PhysicalPosition::new(0, 3), KeyAction::key(KeyCode::Key4));
    profile.set_action(PhysicalPosition::new(0, 4), KeyAction::key(KeyCode::Key5));

    // Second row: F1-F5
    profile.set_action(PhysicalPosition::new(1, 0), KeyAction::key(KeyCode::F1));
    profile.set_action(PhysicalPosition::new(1, 1), KeyAction::key(KeyCode::F2));
    profile.set_action(PhysicalPosition::new(1, 2), KeyAction::key(KeyCode::F3));
    profile.set_action(PhysicalPosition::new(1, 3), KeyAction::key(KeyCode::F4));
    profile.set_action(PhysicalPosition::new(1, 4), KeyAction::key(KeyCode::F5));

    // Third row: common shortcuts
    profile.set_action(PhysicalPosition::new(2, 2), KeyAction::key(KeyCode::Enter));
    profile.set_action(
        PhysicalPosition::new(2, 3),
        KeyAction::key(KeyCode::Backspace),
    );

    // Bottom rows: media keys
    profile.set_action(
        PhysicalPosition::new(4, 0),
        KeyAction::key(KeyCode::VolumeMute),
    );
    profile.set_action(
        PhysicalPosition::new(4, 1),
        KeyAction::key(KeyCode::VolumeDown),
    );
    profile.set_action(
        PhysicalPosition::new(4, 2),
        KeyAction::key(KeyCode::VolumeUp),
    );

    profile
}

/// Create a simple test profile for testing profile swapping
fn create_simple_profile(name: &str, key_code: KeyCode) -> Profile {
    let mut profile = Profile::new(name, LayoutType::Matrix);
    // Map position (0,0) to the given key
    profile.set_action(PhysicalPosition::new(0, 0), KeyAction::key(key_code));
    profile
}

/// Create an InputEvent for testing
fn create_test_event(scancode: u16, pressed: bool, serial: &str) -> InputEvent {
    InputEvent::with_full_identity(
        KeyCode::Unknown(0),
        pressed,
        1000,
        Some("/dev/input/event0".to_string()),
        false,
        false,
        scancode,
        Some(serial.to_string()),
        Some(0xFEED),
        Some(0xBEEF),
    )
}

// ============================================================================
// E2E Test 1: Two Identical Devices with Different Profiles
// ============================================================================

#[tokio::test]
#[serial]
async fn test_e2e_two_identical_devices_different_profiles() {
    // Setup
    let temp = TempDir::new().unwrap();
    let (registry, _rx) = DeviceRegistry::new();
    let profile_registry = Arc::new(ProfileRegistry::with_directory(
        temp.path().join("profiles"),
    ));
    let (_temp_defs, library) = create_macro_pad_definition();
    let mut device_bindings = DeviceBindings::with_path(temp.path().join("device_bindings.json"));

    // Create two identical devices (same VID:PID, different serials)
    let device1 = DeviceIdentity::new(0xFEED, 0xBEEF, "SERIAL001".to_string());
    let device2 = DeviceIdentity::new(0xFEED, 0xBEEF, "SERIAL002".to_string());

    // Register both devices
    registry.register_device(device1.clone()).await;
    registry.register_device(device2.clone()).await;

    // Create different profiles for each device
    let profile1 = create_simple_profile("Device 1 Profile", KeyCode::A);
    let profile2 = create_simple_profile("Device 2 Profile", KeyCode::B);

    profile_registry.save_profile(&profile1).await.unwrap();
    profile_registry.save_profile(&profile2).await.unwrap();

    // Assign different profiles to each device
    registry
        .assign_profile(&device1, profile1.id.clone())
        .await
        .unwrap();
    registry
        .assign_profile(&device2, profile2.id.clone())
        .await
        .unwrap();

    // Enable remap on both
    registry.set_remap_enabled(&device1, true).await.unwrap();
    registry.set_remap_enabled(&device2, true).await.unwrap();

    // Save bindings
    device_bindings.set_binding(
        device1.clone(),
        DeviceBinding::with_profile(profile1.id.clone()),
    );
    device_bindings.set_binding(
        device2.clone(),
        DeviceBinding::with_profile(profile2.id.clone()),
    );
    device_bindings.save().unwrap();

    // Create pipeline components
    let device_resolver = DeviceResolver::new(registry.clone());
    let profile_resolver = ProfileResolver::new(profile_registry.clone());
    let coord_translator = CoordinateTranslator::new(library);

    // Simulate input from device 1
    let event1 = create_test_event(1, true, "SERIAL001");
    let device_state1 = device_resolver.resolve(&event1).await.unwrap().unwrap();
    assert_eq!(device_state1.identity, device1);
    assert_eq!(device_state1.profile_id, Some(profile1.id.clone()));

    let profile1_loaded = profile_resolver
        .resolve(&device_state1.profile_id.unwrap())
        .await
        .unwrap();
    let pos1 = coord_translator
        .translate(&device_state1.identity, event1.scan_code)
        .await
        .unwrap();
    let action1 = profile1_loaded.get_action(&pos1).unwrap();
    assert_eq!(action1, &KeyAction::key(KeyCode::A));

    // Simulate input from device 2 (same physical button, different serial)
    let event2 = create_test_event(1, true, "SERIAL002");
    let device_state2 = device_resolver.resolve(&event2).await.unwrap().unwrap();
    assert_eq!(device_state2.identity, device2);
    assert_eq!(device_state2.profile_id, Some(profile2.id.clone()));

    let profile2_loaded = profile_resolver
        .resolve(&device_state2.profile_id.unwrap())
        .await
        .unwrap();
    let pos2 = coord_translator
        .translate(&device_state2.identity, event2.scan_code)
        .await
        .unwrap();
    let action2 = profile2_loaded.get_action(&pos2).unwrap();
    assert_eq!(action2, &KeyAction::key(KeyCode::B));

    // Verify isolation: same button press, different outputs
    assert_eq!(pos1, pos2); // Same physical position
    assert_ne!(action1, action2); // Different actions

    println!("✅ E2E Test 1: Two identical devices with different profiles - PASSED");
}

// ============================================================================
// E2E Test 2: Profile Swapping on Live Device
// ============================================================================

#[tokio::test]
#[serial]
async fn test_e2e_profile_swap_changes_behavior() {
    // Setup
    let temp = TempDir::new().unwrap();
    let (registry, _rx) = DeviceRegistry::new();
    let profile_registry = Arc::new(ProfileRegistry::with_directory(
        temp.path().join("profiles"),
    ));
    let (_temp_defs, library) = create_macro_pad_definition();

    let device = DeviceIdentity::new(0xFEED, 0xBEEF, "TESTDEV".to_string());
    registry.register_device(device.clone()).await;
    registry.set_remap_enabled(&device, true).await.unwrap();

    // Create two different profiles
    let profile_a = create_simple_profile("Profile A", KeyCode::X);
    let profile_b = create_simple_profile("Profile B", KeyCode::Y);

    profile_registry.save_profile(&profile_a).await.unwrap();
    profile_registry.save_profile(&profile_b).await.unwrap();

    // Create pipeline
    let device_resolver = DeviceResolver::new(registry.clone());
    let profile_resolver = ProfileResolver::new(profile_registry.clone());
    let coord_translator = CoordinateTranslator::new(library);

    // Assign profile A and test
    registry
        .assign_profile(&device, profile_a.id.clone())
        .await
        .unwrap();

    let event = create_test_event(1, true, "TESTDEV");
    let state = device_resolver.resolve(&event).await.unwrap().unwrap();
    let profile = profile_resolver
        .resolve(&state.profile_id.unwrap())
        .await
        .unwrap();
    let pos = coord_translator
        .translate(&state.identity, event.scan_code)
        .await
        .unwrap();
    let action = profile.get_action(&pos).unwrap();
    assert_eq!(action, &KeyAction::key(KeyCode::X));

    // Swap to profile B and test again
    registry
        .assign_profile(&device, profile_b.id.clone())
        .await
        .unwrap();

    let event = create_test_event(1, true, "TESTDEV");
    let state = device_resolver.resolve(&event).await.unwrap().unwrap();
    let profile = profile_resolver
        .resolve(&state.profile_id.unwrap())
        .await
        .unwrap();
    let pos = coord_translator
        .translate(&state.identity, event.scan_code)
        .await
        .unwrap();
    let action = profile.get_action(&pos).unwrap();
    assert_eq!(action, &KeyAction::key(KeyCode::Y));

    println!("✅ E2E Test 2: Profile swap changes behavior - PASSED");
}

// ============================================================================
// E2E Test 3: Per-Device Remap Toggle (Passthrough vs Active)
// ============================================================================

#[tokio::test]
#[serial]
async fn test_e2e_remap_toggle_per_device() {
    // Setup
    let temp = TempDir::new().unwrap();
    let (registry, _rx) = DeviceRegistry::new();
    let profile_registry = Arc::new(ProfileRegistry::with_directory(
        temp.path().join("profiles"),
    ));

    let device = DeviceIdentity::new(0xFEED, 0xBEEF, "TOGGLE_TEST".to_string());
    registry.register_device(device.clone()).await;

    let profile = create_simple_profile("Toggle Profile", KeyCode::Z);
    profile_registry.save_profile(&profile).await.unwrap();
    registry
        .assign_profile(&device, profile.id.clone())
        .await
        .unwrap();

    let device_resolver = DeviceResolver::new(registry.clone());

    // Initially disabled (default)
    let event = create_test_event(1, true, "TOGGLE_TEST");
    let state = device_resolver.resolve(&event).await.unwrap().unwrap();
    assert!(!state.remap_enabled, "Should be disabled by default");

    // Enable remap
    registry.set_remap_enabled(&device, true).await.unwrap();
    let state = device_resolver.resolve(&event).await.unwrap().unwrap();
    assert!(state.remap_enabled, "Should be enabled after toggle");

    // Disable again
    registry.set_remap_enabled(&device, false).await.unwrap();
    let state = device_resolver.resolve(&event).await.unwrap().unwrap();
    assert!(!state.remap_enabled, "Should be disabled after toggle");

    println!("✅ E2E Test 3: Per-device remap toggle - PASSED");
}

// ============================================================================
// E2E Test 4: Device Disconnect/Reconnect with Binding Persistence
// ============================================================================

#[tokio::test]
#[serial]
async fn test_e2e_disconnect_reconnect_binding_persists() {
    // Setup
    let temp = TempDir::new().unwrap();
    let (registry, _rx) = DeviceRegistry::new();
    let profile_registry = Arc::new(ProfileRegistry::with_directory(
        temp.path().join("profiles"),
    ));
    let bindings_path = temp.path().join("device_bindings.json");

    let device = DeviceIdentity::new(0xFEED, 0xBEEF, "PERSIST_TEST".to_string());
    let profile = create_simple_profile("Persistent Profile", KeyCode::P);
    profile_registry.save_profile(&profile).await.unwrap();

    // Initial connection and setup
    registry.register_device(device.clone()).await;
    registry
        .assign_profile(&device, profile.id.clone())
        .await
        .unwrap();
    registry.set_remap_enabled(&device, true).await.unwrap();

    // Save binding to disk
    let mut bindings = DeviceBindings::with_path(bindings_path.clone());
    bindings.set_binding(
        device.clone(),
        DeviceBinding::with_profile(profile.id.clone()),
    );
    bindings.save().unwrap();

    // Verify initial state
    let state = registry.get_device_state(&device).await.unwrap();
    assert_eq!(state.profile_id, Some(profile.id.clone()));
    assert!(state.remap_enabled);

    // Simulate disconnect
    registry.unregister_device(&device).await;
    assert!(registry.get_device_state(&device).await.is_none());

    // Simulate reconnect and restore from persistent storage
    registry.register_device(device.clone()).await;

    // Load bindings and restore
    let mut restored_bindings = DeviceBindings::with_path(bindings_path);
    restored_bindings.load().unwrap();

    if let Some(binding) = restored_bindings.get_binding(&device) {
        if let Some(profile_id) = &binding.profile_id {
            registry
                .assign_profile(&device, profile_id.clone())
                .await
                .unwrap();
            registry
                .set_remap_enabled(&device, binding.remap_enabled)
                .await
                .unwrap();
        }
    }

    // Verify state is restored
    let state = registry.get_device_state(&device).await.unwrap();
    assert_eq!(state.profile_id, Some(profile.id));
    assert!(state.remap_enabled);

    println!("✅ E2E Test 4: Disconnect/reconnect binding persistence - PASSED");
}

// ============================================================================
// E2E Test 5: 5×5 Macro Pad with Custom Layout
// ============================================================================

#[tokio::test]
#[serial]
async fn test_e2e_macro_pad_5x5_layout() {
    // Setup
    let temp = TempDir::new().unwrap();
    let (registry, _rx) = DeviceRegistry::new();
    let profile_registry = Arc::new(ProfileRegistry::with_directory(
        temp.path().join("profiles"),
    ));
    let (_temp_defs, library) = create_macro_pad_definition();

    let device = DeviceIdentity::new(0xFEED, 0xBEEF, "MACROPAD5X5".to_string());
    registry.register_device(device.clone()).await;
    registry.set_remap_enabled(&device, true).await.unwrap();

    // Create and assign macro pad profile
    let profile = create_macro_pad_profile("5×5 Macro Pad Profile");
    profile_registry.save_profile(&profile).await.unwrap();
    registry
        .assign_profile(&device, profile.id.clone())
        .await
        .unwrap();

    // Create pipeline
    let device_resolver = DeviceResolver::new(registry.clone());
    let profile_resolver = ProfileResolver::new(profile_registry.clone());
    let coord_translator = CoordinateTranslator::new(library);

    // Test mapping for various buttons
    let test_cases = vec![
        (1, 0, 0, KeyCode::Key1),        // Top-left
        (5, 0, 4, KeyCode::Key5),        // Top-right
        (6, 1, 0, KeyCode::F1),          // Second row, first button
        (10, 1, 4, KeyCode::F5),         // Second row, last button
        (13, 2, 2, KeyCode::Enter),      // Center button
        (21, 4, 0, KeyCode::VolumeMute), // Bottom-left
        (25, 4, 4, KeyCode::VolumeUp),   // Bottom-right
    ];

    for (scancode, expected_row, expected_col, expected_key) in test_cases {
        let event = create_test_event(scancode, true, "MACROPAD5X5");

        // Full pipeline
        let state = device_resolver.resolve(&event).await.unwrap().unwrap();
        let profile = profile_resolver
            .resolve(&state.profile_id.unwrap())
            .await
            .unwrap();
        let pos = coord_translator
            .translate(&state.identity, event.scan_code)
            .await
            .unwrap();

        // Verify position
        assert_eq!(
            pos.row, expected_row,
            "Scancode {} should map to row {}",
            scancode, expected_row
        );
        assert_eq!(
            pos.col, expected_col,
            "Scancode {} should map to col {}",
            scancode, expected_col
        );

        // Verify action
        if let Some(action) = profile.get_action(&pos) {
            assert_eq!(
                action,
                &KeyAction::key(expected_key),
                "Position ({},{}) should output {:?}",
                expected_row,
                expected_col,
                expected_key
            );
        }
    }

    // Verify the profile covers a 5×5 matrix
    assert_eq!(profile.layout_type, LayoutType::Matrix);

    // Count mapped positions
    let mapped_count = profile.mappings.len();
    assert!(
        mapped_count >= 15,
        "Should have at least 15 mappings, got {}",
        mapped_count
    );

    println!("✅ E2E Test 5: 5×5 Macro Pad with custom layout - PASSED");
}

// ============================================================================
// E2E Test 6: Complete Workflow Integration
// ============================================================================

#[tokio::test]
#[serial]
async fn test_e2e_complete_workflow() {
    // This test simulates the complete user workflow:
    // 1. Application starts, loads saved bindings
    // 2. Device connects
    // 3. Binding is restored from disk
    // 4. User input is processed through full pipeline
    // 5. Output is generated based on profile mapping

    let temp = TempDir::new().unwrap();
    let (registry, _rx) = DeviceRegistry::new();
    let profile_registry = Arc::new(ProfileRegistry::with_directory(
        temp.path().join("profiles"),
    ));
    let (_temp_defs, library) = create_macro_pad_definition();
    let bindings_path = temp.path().join("device_bindings.json");

    // Step 1: Create and save profile (user setup)
    let profile = create_macro_pad_profile("My Macro Pad");
    profile_registry.save_profile(&profile).await.unwrap();

    // Step 2: Create and save binding (first time device was configured)
    let device = DeviceIdentity::new(0xFEED, 0xBEEF, "WORKFLOW_TEST".to_string());
    let mut bindings = DeviceBindings::with_path(bindings_path.clone());
    bindings.set_binding(
        device.clone(),
        DeviceBinding::with_profile(profile.id.clone()),
    );
    bindings.save().unwrap();

    // Step 3: Application restart - load bindings
    let mut loaded_bindings = DeviceBindings::with_path(bindings_path);
    loaded_bindings.load().unwrap();

    // Step 4: Device connects
    registry.register_device(device.clone()).await;

    // Step 5: Restore binding from persistent storage
    if let Some(binding) = loaded_bindings.get_binding(&device) {
        if let Some(profile_id) = &binding.profile_id {
            registry
                .assign_profile(&device, profile_id.clone())
                .await
                .unwrap();
            registry
                .set_remap_enabled(&device, binding.remap_enabled)
                .await
                .unwrap();
        }
    }

    // Step 6: Create pipeline
    let device_resolver = DeviceResolver::new(registry.clone());
    let profile_resolver = ProfileResolver::new(profile_registry);
    let coord_translator = CoordinateTranslator::new(library);

    // Step 7: Process user input (button 1 pressed)
    let event = create_test_event(1, true, "WORKFLOW_TEST");

    let state = device_resolver.resolve(&event).await.unwrap().unwrap();
    assert!(state.remap_enabled);

    let profile = profile_resolver
        .resolve(&state.profile_id.unwrap())
        .await
        .unwrap();

    let pos = coord_translator
        .translate(&state.identity, event.scan_code)
        .await
        .unwrap();

    let action = profile.get_action(&pos).unwrap();
    assert_eq!(action, &KeyAction::key(KeyCode::Key1));

    println!("✅ E2E Test 6: Complete workflow integration - PASSED");
}
