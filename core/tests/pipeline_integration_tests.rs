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
//! Integration tests for the revolutionary mapping pipeline.
//!
//! Tests the end-to-end flow of the revolutionary mapping pipeline:
//! 1. Device resolution (InputEvent -> DeviceState)
//! 2. Profile resolution (profile_id -> Profile)
//! 3. Coordinate translation (scancode -> PhysicalPosition)
//! 4. Profile mapping (PhysicalPosition -> KeyAction)
//! 5. Final output
//!
//! Also tests passthrough mode, error handling, and latency targets.

use keyrx_core::definitions::library::DeviceDefinitionLibrary;
use keyrx_core::drivers::keycodes::KeyCode;
use keyrx_core::engine::{CoordinateTranslator, DeviceResolver, InputEvent, ProfileResolver};
use keyrx_core::identity::DeviceIdentity;
use keyrx_core::registry::device::DeviceRegistry;
use keyrx_core::registry::profile::{KeyAction, LayoutType, PhysicalPosition, Profile};
use keyrx_core::registry::ProfileRegistry;
use serial_test::serial;
use std::sync::Arc;
use std::time::Instant;
use tempfile::TempDir;

/// Create a test device definition library with a simple 3x3 matrix device
fn create_test_definition_library() -> (TempDir, Arc<DeviceDefinitionLibrary>) {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test-device.toml");

    let toml_content = r#"
name = "Test Matrix Device"
vendor_id = 0x1234
product_id = 0x5678
manufacturer = "Test Manufacturer"

[layout]
layout_type = "matrix"
rows = 3
cols = 3

[matrix_map]
"1" = { row = 0, col = 0 }
"2" = { row = 0, col = 1 }
"3" = { row = 0, col = 2 }
"4" = { row = 1, col = 0 }
"5" = { row = 1, col = 1 }
"6" = { row = 1, col = 2 }
"7" = { row = 2, col = 0 }
"8" = { row = 2, col = 1 }
"9" = { row = 2, col = 2 }

[visual]
key_width = 80
key_height = 80
key_spacing = 4
"#;

    std::fs::write(&file_path, toml_content).unwrap();

    let mut library = DeviceDefinitionLibrary::new();
    library.load_from_directory(temp_dir.path()).unwrap();

    (temp_dir, Arc::new(library))
}

/// Create a test profile with some mappings
fn create_test_profile(profile_name: &str) -> Profile {
    let mut profile = Profile::new(profile_name, LayoutType::Matrix);

    // Map some positions to different keys
    profile.set_action(
        PhysicalPosition::new(0, 0),
        KeyAction::key(KeyCode::Q), // Button 1 -> Q
    );
    profile.set_action(
        PhysicalPosition::new(0, 1),
        KeyAction::key(KeyCode::W), // Button 2 -> W
    );
    profile.set_action(
        PhysicalPosition::new(1, 1),
        KeyAction::key(KeyCode::Space), // Button 5 -> Space
    );

    profile
}

/// Create a test event with full device identity
fn create_event_with_identity(scancode: u16, pressed: bool, timestamp: u64) -> InputEvent {
    InputEvent::with_full_identity(
        KeyCode::Unknown(0),
        pressed,
        timestamp,
        Some("/dev/input/event0".to_string()),
        false,
        false,
        scancode,
        Some("TEST123".to_string()),
        Some(0x1234),
        Some(0x5678),
    )
}

#[tokio::test]
#[serial]
async fn test_device_resolution_registered_device() {
    // Setup
    let (registry, _rx) = DeviceRegistry::new();
    let resolver = DeviceResolver::new(registry.clone());

    // Register a device
    let identity = DeviceIdentity::new(0x1234, 0x5678, "TEST123".to_string());
    registry.register_device(identity.clone()).await;
    registry.set_remap_enabled(&identity, true).await.unwrap();

    // Create event from that device
    let event = create_event_with_identity(1, true, 1000);

    // Resolve - should find the device with remap enabled
    let result = resolver.resolve(&event).await.unwrap();
    assert!(result.is_some());

    let state = result.unwrap();
    assert_eq!(state.identity, identity);
    assert!(state.remap_enabled);
}

#[tokio::test]
#[serial]
async fn test_device_resolution_unregistered_device() {
    // Setup
    let (registry, _rx) = DeviceRegistry::new();
    let resolver = DeviceResolver::new(registry);

    // Create event from unregistered device
    let event = create_event_with_identity(1, true, 1000);

    // Resolve - should return None
    let result = resolver.resolve(&event).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
#[serial]
async fn test_profile_resolution_with_caching() {
    // Setup
    let temp = TempDir::new().unwrap();
    let profile_registry = Arc::new(ProfileRegistry::with_directory(temp.path().to_path_buf()));
    let resolver = ProfileResolver::new(profile_registry.clone());

    // Create and save a profile
    let profile = create_test_profile("Test Profile");
    let profile_id = profile.id.clone();
    profile_registry.save_profile(&profile).await.unwrap();

    // First resolve (cold - loads from disk)
    let start = Instant::now();
    let resolved1 = resolver.resolve(&profile_id).await.unwrap();
    let cold_latency = start.elapsed();

    assert_eq!(resolved1.id, profile_id);
    assert_eq!(resolved1.name, "Test Profile");

    // Second resolve (warm - hits cache)
    let start = Instant::now();
    let resolved2 = resolver.resolve(&profile_id).await.unwrap();
    let warm_latency = start.elapsed();

    // Verify caching works (both point to same Arc)
    assert!(Arc::ptr_eq(&resolved1, &resolved2));

    // Verify latency targets
    println!(
        "Cold profile lookup: {}μs, Warm profile lookup: {}μs",
        cold_latency.as_micros(),
        warm_latency.as_micros()
    );
    assert!(
        warm_latency.as_micros() < 100,
        "Warm profile lookup should be <100μs, got {}μs",
        warm_latency.as_micros()
    );
}

#[tokio::test]
#[serial]
async fn test_coordinate_translation() {
    // Setup
    let (_temp, library) = create_test_definition_library();
    let translator = CoordinateTranslator::new(library);

    let device = DeviceIdentity::new(0x1234, 0x5678, "TEST123".to_string());

    // Test translation of various scancodes
    let pos1 = translator.translate(&device, 1).await.unwrap();
    assert_eq!(pos1, PhysicalPosition::new(0, 0));

    let pos5 = translator.translate(&device, 5).await.unwrap();
    assert_eq!(pos5, PhysicalPosition::new(1, 1));

    let pos9 = translator.translate(&device, 9).await.unwrap();
    assert_eq!(pos9, PhysicalPosition::new(2, 2));
}

#[tokio::test]
#[serial]
async fn test_coordinate_translation_latency() {
    // Setup
    let (_temp, library) = create_test_definition_library();
    let translator = CoordinateTranslator::new(library);

    let device = DeviceIdentity::new(0x1234, 0x5678, "TEST123".to_string());

    // First translation (cold - builds cache)
    let start = Instant::now();
    let _pos = translator.translate(&device, 1).await.unwrap();
    let cold_latency = start.elapsed();

    // Subsequent translations (hot - uses cache)
    let mut hot_latencies = vec![];
    for scancode in 2..=9 {
        let start = Instant::now();
        let _pos = translator.translate(&device, scancode).await.unwrap();
        hot_latencies.push(start.elapsed());
    }

    let avg_hot_latency =
        hot_latencies.iter().sum::<std::time::Duration>() / hot_latencies.len() as u32;

    println!(
        "Cold translation: {}μs, Average hot translation: {}μs",
        cold_latency.as_micros(),
        avg_hot_latency.as_micros()
    );

    // Verify latency target for hot path
    assert!(
        avg_hot_latency.as_micros() < 20,
        "Hot translation should be <20μs, got {}μs",
        avg_hot_latency.as_micros()
    );
}

#[tokio::test]
#[serial]
async fn test_full_pipeline_end_to_end() {
    // Setup all pipeline components
    let (registry, _rx) = DeviceRegistry::new();
    let temp = TempDir::new().unwrap();
    let profile_registry = Arc::new(ProfileRegistry::with_directory(temp.path().to_path_buf()));
    let (_temp_defs, library) = create_test_definition_library();

    // Create resolvers
    let device_resolver = DeviceResolver::new(registry.clone());
    let profile_resolver = ProfileResolver::new(profile_registry.clone());
    let coord_translator = CoordinateTranslator::new(library);

    // Register a device
    let identity = DeviceIdentity::new(0x1234, 0x5678, "TEST123".to_string());
    registry.register_device(identity.clone()).await;
    registry.set_remap_enabled(&identity, true).await.unwrap();

    // Create and assign a profile
    let profile = create_test_profile("Test Profile");
    let profile_id = profile.id.clone();
    profile_registry.save_profile(&profile).await.unwrap();
    registry
        .assign_profile(&identity, profile_id.clone())
        .await
        .unwrap();

    // Simulate an input event (scancode 1 pressed)
    let event = create_event_with_identity(1, true, 1000);

    // Step 1: Resolve device
    let device_state = device_resolver.resolve(&event).await.unwrap();
    assert!(device_state.is_some());
    let device_state = device_state.unwrap();
    assert!(device_state.remap_enabled);
    assert_eq!(device_state.profile_id, Some(profile_id.clone()));

    // Step 2: Resolve profile
    let profile = profile_resolver
        .resolve(&device_state.profile_id.unwrap())
        .await
        .unwrap();
    assert_eq!(profile.name, "Test Profile");

    // Step 3: Translate scancode to physical position
    let physical_pos = coord_translator
        .translate(&device_state.identity, event.scan_code)
        .await
        .unwrap();
    assert_eq!(physical_pos, PhysicalPosition::new(0, 0));

    // Step 4: Look up action in profile
    let action = profile.get_action(&physical_pos);
    assert!(action.is_some());
    assert_eq!(action.unwrap(), &KeyAction::key(KeyCode::Q));

    // Verify the complete flow: Scancode 1 -> (0,0) -> Q
    println!(
        "✅ Full pipeline: Scancode {} -> {:?} -> KeyCode::Q",
        event.scan_code, physical_pos
    );
}

#[tokio::test]
#[serial]
async fn test_passthrough_mode_device_disabled() {
    // Setup
    let (registry, _rx) = DeviceRegistry::new();
    let resolver = DeviceResolver::new(registry.clone());

    // Register device with remap DISABLED
    let identity = DeviceIdentity::new(0x1234, 0x5678, "TEST123".to_string());
    registry.register_device(identity.clone()).await;
    registry.set_remap_enabled(&identity, false).await.unwrap();

    // Create event
    let event = create_event_with_identity(1, true, 1000);

    // Resolve device
    let device_state = resolver.resolve(&event).await.unwrap();
    assert!(device_state.is_some());
    let device_state = device_state.unwrap();

    // Device should be disabled for remapping
    assert!(!device_state.remap_enabled);

    // In real pipeline, this would trigger passthrough
    println!(
        "✅ Device {} is disabled, should passthrough",
        device_state.identity.to_key()
    );
}

#[tokio::test]
#[serial]
async fn test_error_handling_no_device_definition() {
    // Setup with empty library (no definitions)
    let _temp_dir = TempDir::new().unwrap();
    let library = DeviceDefinitionLibrary::new();
    let translator = CoordinateTranslator::new(Arc::new(library));

    // Try to translate for unknown device
    let device = DeviceIdentity::new(0x9999, 0x8888, "UNKNOWN".to_string());
    let result = translator.translate(&device, 1).await;

    // Should return DeviceDefinitionNotFound error
    assert!(result.is_err());
    println!("✅ Error handling: Unknown device returns error as expected");
}

#[tokio::test]
#[serial]
async fn test_error_handling_unmapped_scancode() {
    // Setup
    let (_temp, library) = create_test_definition_library();
    let translator = CoordinateTranslator::new(library);

    let device = DeviceIdentity::new(0x1234, 0x5678, "TEST123".to_string());

    // Try to translate scancode that doesn't exist in definition
    let result = translator.translate(&device, 999).await;

    // Should return ScancodeNotMapped error
    assert!(result.is_err());
    println!("✅ Error handling: Unmapped scancode returns error as expected");
}

#[tokio::test]
#[serial]
async fn test_error_handling_no_profile() {
    // Setup
    let temp = TempDir::new().unwrap();
    let profile_registry = Arc::new(ProfileRegistry::with_directory(temp.path().to_path_buf()));
    let resolver = ProfileResolver::new(profile_registry);

    // Try to resolve non-existent profile
    let result = resolver.resolve("nonexistent-profile-id").await;

    // Should return NotFound error
    assert!(result.is_err());
    println!("✅ Error handling: Non-existent profile returns error as expected");
}

#[tokio::test]
#[serial]
async fn test_two_identical_devices_different_profiles() {
    // Setup
    let (registry, _rx) = DeviceRegistry::new();
    let temp = TempDir::new().unwrap();
    let profile_registry = Arc::new(ProfileRegistry::with_directory(temp.path().to_path_buf()));

    // Register two identical devices (same VID:PID, different serials)
    let device1 = DeviceIdentity::new(0x1234, 0x5678, "SERIAL001".to_string());
    let device2 = DeviceIdentity::new(0x1234, 0x5678, "SERIAL002".to_string());

    registry.register_device(device1.clone()).await;
    registry.register_device(device2.clone()).await;

    registry.set_remap_enabled(&device1, true).await.unwrap();
    registry.set_remap_enabled(&device2, true).await.unwrap();

    // Create different profiles
    let profile1 = create_test_profile("Profile 1");
    let profile2 = create_test_profile("Profile 2");

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

    // Verify isolation
    let state1 = registry.get_device_state(&device1).await.unwrap();
    let state2 = registry.get_device_state(&device2).await.unwrap();

    assert_eq!(state1.profile_id, Some(profile1.id));
    assert_eq!(state2.profile_id, Some(profile2.id));

    println!("✅ Two identical devices have independent profiles");
}

#[tokio::test]
#[serial]
async fn test_profile_change_live() {
    // Setup
    let (registry, _rx) = DeviceRegistry::new();
    let temp = TempDir::new().unwrap();
    let profile_registry = Arc::new(ProfileRegistry::with_directory(temp.path().to_path_buf()));

    // Register device
    let identity = DeviceIdentity::new(0x1234, 0x5678, "TEST123".to_string());
    registry.register_device(identity.clone()).await;

    // Create profiles
    let profile1 = create_test_profile("Profile A");
    let profile2 = create_test_profile("Profile B");

    profile_registry.save_profile(&profile1).await.unwrap();
    profile_registry.save_profile(&profile2).await.unwrap();

    // Assign profile 1
    registry
        .assign_profile(&identity, profile1.id.clone())
        .await
        .unwrap();

    let state = registry.get_device_state(&identity).await.unwrap();
    assert_eq!(state.profile_id, Some(profile1.id.clone()));

    // Switch to profile 2
    registry
        .assign_profile(&identity, profile2.id.clone())
        .await
        .unwrap();

    let state = registry.get_device_state(&identity).await.unwrap();
    assert_eq!(state.profile_id, Some(profile2.id));

    println!("✅ Profile switching works live");
}

#[tokio::test]
#[serial]
async fn test_toggle_remap_per_device() {
    // Setup
    let (registry, _rx) = DeviceRegistry::new();

    // Register device
    let identity = DeviceIdentity::new(0x1234, 0x5678, "TEST123".to_string());
    registry.register_device(identity.clone()).await;

    // Initially disabled
    let state = registry.get_device_state(&identity).await.unwrap();
    assert!(!state.remap_enabled);

    // Enable
    registry.set_remap_enabled(&identity, true).await.unwrap();
    let state = registry.get_device_state(&identity).await.unwrap();
    assert!(state.remap_enabled);

    // Disable again
    registry.set_remap_enabled(&identity, false).await.unwrap();
    let state = registry.get_device_state(&identity).await.unwrap();
    assert!(!state.remap_enabled);

    println!("✅ Per-device remap toggle works");
}

#[tokio::test]
#[serial]
async fn test_device_reconnect_persists_binding() {
    // Setup
    let (registry, _rx) = DeviceRegistry::new();
    let temp = TempDir::new().unwrap();
    let profile_registry = Arc::new(ProfileRegistry::with_directory(temp.path().to_path_buf()));

    // Register device and assign profile
    let identity = DeviceIdentity::new(0x1234, 0x5678, "TEST123".to_string());
    registry.register_device(identity.clone()).await;

    let profile = create_test_profile("Persistent Profile");
    profile_registry.save_profile(&profile).await.unwrap();

    registry
        .assign_profile(&identity, profile.id.clone())
        .await
        .unwrap();
    registry.set_remap_enabled(&identity, true).await.unwrap();

    // Simulate disconnect
    registry.unregister_device(&identity).await;
    assert!(registry.get_device_state(&identity).await.is_none());

    // Simulate reconnect
    registry.register_device(identity.clone()).await;

    // Note: In a real implementation with persistence, bindings would be
    // restored. For now, this test just verifies the device can be
    // re-registered and re-configured.
    registry
        .assign_profile(&identity, profile.id.clone())
        .await
        .unwrap();

    let state = registry.get_device_state(&identity).await.unwrap();
    assert_eq!(state.profile_id, Some(profile.id));

    println!("✅ Device reconnect works (bindings would persist with storage)");
}

#[tokio::test]
#[serial]
async fn test_concurrent_device_resolution() {
    // Setup
    let (registry, _rx) = DeviceRegistry::new();
    let resolver = DeviceResolver::new(registry.clone());

    // Register device
    let identity = DeviceIdentity::new(0x1234, 0x5678, "TEST123".to_string());
    registry.register_device(identity.clone()).await;
    registry.set_remap_enabled(&identity, true).await.unwrap();

    // Spawn multiple concurrent resolve operations
    let mut handles = vec![];
    for i in 0..50 {
        let resolver_clone = resolver.clone();
        let handle = tokio::spawn(async move {
            let event = create_event_with_identity(1, true, 1000 + i);
            resolver_clone.resolve(&event).await
        });
        handles.push(handle);
    }

    // All should succeed
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    println!("✅ Concurrent device resolution works");
}

#[tokio::test]
#[serial]
async fn test_latency_benchmark_full_pipeline() {
    // Setup
    let (registry, _rx) = DeviceRegistry::new();
    let temp = TempDir::new().unwrap();
    let profile_registry = Arc::new(ProfileRegistry::with_directory(temp.path().to_path_buf()));
    let (_temp_defs, library) = create_test_definition_library();

    let device_resolver = DeviceResolver::new(registry.clone());
    let profile_resolver = ProfileResolver::new(profile_registry.clone());
    let coord_translator = CoordinateTranslator::new(library);

    // Setup device and profile
    let identity = DeviceIdentity::new(0x1234, 0x5678, "TEST123".to_string());
    registry.register_device(identity.clone()).await;
    registry.set_remap_enabled(&identity, true).await.unwrap();

    let profile = create_test_profile("Benchmark Profile");
    let profile_id = profile.id.clone();
    profile_registry.save_profile(&profile).await.unwrap();
    registry
        .assign_profile(&identity, profile_id.clone())
        .await
        .unwrap();

    // Warm up caches with initial lookups
    let event = create_event_with_identity(1, true, 1000);
    let _ = device_resolver.resolve(&event).await.unwrap();
    let _ = profile_resolver.resolve(&profile_id).await.unwrap();
    let _ = coord_translator.translate(&identity, 1).await.unwrap();

    // Benchmark hot path (all caches warm)
    let mut latencies = vec![];
    for scancode in 1..=9 {
        let event = create_event_with_identity(scancode, true, 1000);

        let start = Instant::now();

        // Full pipeline
        let device_state = device_resolver.resolve(&event).await.unwrap().unwrap();
        let profile = profile_resolver
            .resolve(&device_state.profile_id.unwrap())
            .await
            .unwrap();
        let physical_pos = coord_translator
            .translate(&device_state.identity, event.scan_code)
            .await
            .unwrap();
        let _action = profile.get_action(&physical_pos);

        latencies.push(start.elapsed());
    }

    let avg_latency = latencies.iter().sum::<std::time::Duration>() / latencies.len() as u32;
    let max_latency = latencies.iter().max().unwrap();

    println!(
        "Pipeline latency - Average: {}μs, Max: {}μs",
        avg_latency.as_micros(),
        max_latency.as_micros()
    );

    // Verify <1ms target (requirement 10)
    assert!(
        max_latency.as_micros() < 1000,
        "Pipeline should complete in <1ms, got {}μs",
        max_latency.as_micros()
    );

    println!("✅ Full pipeline latency target met (<1ms)");
}
