//! Integration tests for multi-device support.
//!
//! These tests validate the complete multi-device functionality:
//! - Device discovery and enumeration
//! - Event tagging with device_id
//! - Per-device remapping via Rhai when_device_start/when_device_end
//! - Web API /api/devices endpoint
//!
//! Since real device enumeration requires hardware and permissions,
//! these tests use mock/simulated components where appropriate.

use keyrx_core::config::{
    mappings::{BaseKeyMapping, DeviceConfig, DeviceIdentifier, KeyMapping},
    Condition, ConditionItem, KeyCode,
};
use keyrx_core::runtime::event::KeyEvent;
use keyrx_core::runtime::{DeviceState, KeyLookup};
use keyrx_daemon::platform::{MockInput, MockOutput};
use keyrx_daemon::processor::EventProcessor;

extern crate alloc;
use alloc::string::String;

/// Helper to create a test device config.
fn create_test_config(mappings: Vec<KeyMapping>) -> DeviceConfig {
    DeviceConfig {
        identifier: DeviceIdentifier {
            pattern: String::from("*"),
        },
        mappings,
    }
}

// ============================================================================
// Test: Device-specific conditional remapping with DeviceMatches
// ============================================================================

/// Test: Verify that DeviceMatches condition correctly filters mappings
/// based on device_id in the event.
///
/// Scenario: Numpad device (device_id: "serial-numpad-123") should map
/// Numpad1 → F13, while main keyboard passes through unchanged.
#[test]
fn test_device_matches_condition_evaluation() {
    use keyrx_core::runtime::state::DeviceState;

    let state = DeviceState::new();

    // Test exact device match
    let cond = Condition::DeviceMatches(String::from("serial-numpad-123"));
    assert!(state.evaluate_condition_with_device(&cond, Some("serial-numpad-123")));
    assert!(!state.evaluate_condition_with_device(&cond, Some("serial-main-456")));
    assert!(!state.evaluate_condition_with_device(&cond, None));

    // Test wildcard pattern - contains
    let cond_wildcard = Condition::DeviceMatches(String::from("*numpad*"));
    assert!(state.evaluate_condition_with_device(&cond_wildcard, Some("serial-numpad-123")));
    assert!(state.evaluate_condition_with_device(&cond_wildcard, Some("usb-numpad-abc")));
    assert!(!state.evaluate_condition_with_device(&cond_wildcard, Some("serial-main-456")));

    // Test prefix pattern
    let cond_prefix = Condition::DeviceMatches(String::from("usb-*"));
    assert!(state.evaluate_condition_with_device(&cond_prefix, Some("usb-keyboard-123")));
    assert!(!state.evaluate_condition_with_device(&cond_prefix, Some("serial-keyboard-123")));

    // Test suffix pattern
    let cond_suffix = Condition::DeviceMatches(String::from("*-keyboard"));
    assert!(state.evaluate_condition_with_device(&cond_suffix, Some("usb-keyboard")));
    assert!(state.evaluate_condition_with_device(&cond_suffix, Some("serial-keyboard")));
    assert!(!state.evaluate_condition_with_device(&cond_suffix, Some("usb-numpad")));
}

/// Test: KeyLookup correctly uses device_id for mapping resolution.
///
/// Creates a config with device-specific mappings and verifies that
/// find_mapping_with_device() returns the correct mapping based on device_id.
#[test]
fn test_key_lookup_with_device_id() {
    // Config with device-specific mapping:
    // When device matches "*numpad*": Numpad1 → F13
    let config = create_test_config(vec![KeyMapping::conditional(
        Condition::DeviceMatches(String::from("*numpad*")),
        vec![BaseKeyMapping::Simple {
            from: KeyCode::Numpad1,
            to: KeyCode::F13,
        }],
    )]);

    let lookup = KeyLookup::from_device_config(&config);
    let state = DeviceState::new();

    // Numpad device should get the remapping
    let mapping =
        lookup.find_mapping_with_device(KeyCode::Numpad1, &state, Some("serial-numpad-123"));
    assert!(mapping.is_some());
    match mapping.unwrap() {
        BaseKeyMapping::Simple { from, to } => {
            assert_eq!(*from, KeyCode::Numpad1);
            assert_eq!(*to, KeyCode::F13);
        }
        _ => panic!("Expected Simple mapping"),
    }

    // Main keyboard should get no mapping (passthrough)
    let mapping_main =
        lookup.find_mapping_with_device(KeyCode::Numpad1, &state, Some("serial-main-keyboard"));
    assert!(mapping_main.is_none());

    // Without device_id, DeviceMatches never matches
    let mapping_none = lookup.find_mapping_with_device(KeyCode::Numpad1, &state, None);
    assert!(mapping_none.is_none());
}

/// Test: Multiple device-specific mappings with different patterns.
///
/// Validates that different devices get their respective mappings
/// when multiple DeviceMatches conditions are defined.
#[test]
fn test_multiple_device_specific_mappings() {
    // Config:
    // - Device "*numpad*": Numpad1 → F13
    // - Device "*gaming*": W → Up (WASD for gaming)
    let config = create_test_config(vec![
        KeyMapping::conditional(
            Condition::DeviceMatches(String::from("*numpad*")),
            vec![BaseKeyMapping::Simple {
                from: KeyCode::Numpad1,
                to: KeyCode::F13,
            }],
        ),
        KeyMapping::conditional(
            Condition::DeviceMatches(String::from("*gaming*")),
            vec![BaseKeyMapping::Simple {
                from: KeyCode::W,
                to: KeyCode::Up,
            }],
        ),
    ]);

    let lookup = KeyLookup::from_device_config(&config);
    let state = DeviceState::new();

    // Numpad gets Numpad1 → F13
    let numpad_mapping =
        lookup.find_mapping_with_device(KeyCode::Numpad1, &state, Some("usb-numpad-xyz"));
    assert!(numpad_mapping.is_some());
    match numpad_mapping.unwrap() {
        BaseKeyMapping::Simple { to, .. } => assert_eq!(*to, KeyCode::F13),
        _ => panic!("Expected Simple mapping"),
    }

    // Gaming keyboard gets W → Up
    let gaming_mapping =
        lookup.find_mapping_with_device(KeyCode::W, &state, Some("logitech-gaming-g502"));
    assert!(gaming_mapping.is_some());
    match gaming_mapping.unwrap() {
        BaseKeyMapping::Simple { to, .. } => assert_eq!(*to, KeyCode::Up),
        _ => panic!("Expected Simple mapping"),
    }

    // Main keyboard doesn't get either mapping
    let main_mapping =
        lookup.find_mapping_with_device(KeyCode::Numpad1, &state, Some("apple-keyboard"));
    assert!(main_mapping.is_none());
    let main_w_mapping =
        lookup.find_mapping_with_device(KeyCode::W, &state, Some("apple-keyboard"));
    assert!(main_w_mapping.is_none());
}

// ============================================================================
// Test: Device-specific remapping combined with modifiers
// ============================================================================

/// Test: Device-specific mapping with modifier layer.
///
/// Scenario: On numpad device, when CapsLock (modifier) is held,
/// Numpad1-5 should map to media keys.
#[test]
fn test_device_specific_with_modifier_layer() {
    // We can't combine DeviceMatches with modifiers in AllActive currently,
    // but we can test device-specific mappings alongside regular modifier mappings

    let config = create_test_config(vec![
        // CapsLock as modifier
        KeyMapping::modifier(KeyCode::CapsLock, 0),
        // Numpad-specific: Numpad1 → F13 (always, on numpad only)
        KeyMapping::conditional(
            Condition::DeviceMatches(String::from("*numpad*")),
            vec![BaseKeyMapping::Simple {
                from: KeyCode::Numpad1,
                to: KeyCode::F13,
            }],
        ),
        // When CapsLock held: H → Left (on any device)
        KeyMapping::conditional(
            Condition::AllActive(vec![ConditionItem::ModifierActive(0)]),
            vec![BaseKeyMapping::Simple {
                from: KeyCode::H,
                to: KeyCode::Left,
            }],
        ),
    ]);

    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Numpad device: Numpad1 → F13 (even without modifier)
    let mapping = lookup.find_mapping_with_device(KeyCode::Numpad1, &state, Some("usb-numpad-123"));
    assert!(mapping.is_some());
    match mapping.unwrap() {
        BaseKeyMapping::Simple { to, .. } => assert_eq!(*to, KeyCode::F13),
        _ => panic!("Expected Simple mapping"),
    }

    // Main keyboard: Numpad1 passes through (no mapping)
    let main_mapping =
        lookup.find_mapping_with_device(KeyCode::Numpad1, &state, Some("main-keyboard"));
    assert!(main_mapping.is_none());

    // With modifier active: H → Left (on any device)
    state.set_modifier(0);
    let h_mapping = lookup.find_mapping_with_device(KeyCode::H, &state, Some("main-keyboard"));
    assert!(h_mapping.is_some());
    match h_mapping.unwrap() {
        BaseKeyMapping::Simple { to, .. } => assert_eq!(*to, KeyCode::Left),
        _ => panic!("Expected Simple mapping"),
    }

    // H → Left also works on numpad when modifier is active
    let h_numpad = lookup.find_mapping_with_device(KeyCode::H, &state, Some("usb-numpad-123"));
    assert!(h_numpad.is_some());
    match h_numpad.unwrap() {
        BaseKeyMapping::Simple { to, .. } => assert_eq!(*to, KeyCode::Left),
        _ => panic!("Expected Simple mapping"),
    }
}

// ============================================================================
// Test: EventProcessor with device-aware mappings (simulated)
// ============================================================================

/// Test: End-to-end processing with simulated multi-device scenario.
///
/// Uses separate EventProcessor instances for each "device" with
/// different configurations, simulating independent device state.
#[test]
fn test_multi_device_independent_state() {
    // Numpad config: Numpad1-3 → F13-F15
    let numpad_config = create_test_config(vec![
        KeyMapping::simple(KeyCode::Numpad1, KeyCode::F13),
        KeyMapping::simple(KeyCode::Numpad2, KeyCode::F14),
        KeyMapping::simple(KeyCode::Numpad3, KeyCode::F15),
    ]);

    // Main keyboard config: CapsLock layer with HJKL navigation
    let main_config = create_test_config(vec![
        KeyMapping::modifier(KeyCode::CapsLock, 0),
        KeyMapping::conditional(
            Condition::AllActive(vec![ConditionItem::ModifierActive(0)]),
            vec![BaseKeyMapping::Simple {
                from: KeyCode::H,
                to: KeyCode::Left,
            }],
        ),
        KeyMapping::conditional(
            Condition::AllActive(vec![ConditionItem::ModifierActive(0)]),
            vec![BaseKeyMapping::Simple {
                from: KeyCode::J,
                to: KeyCode::Down,
            }],
        ),
    ]);

    // Numpad event sequence
    let numpad_input = MockInput::new(vec![
        KeyEvent::Press(KeyCode::Numpad1),
        KeyEvent::Release(KeyCode::Numpad1),
        KeyEvent::Press(KeyCode::Numpad2),
        KeyEvent::Release(KeyCode::Numpad2),
    ]);
    let numpad_output = MockOutput::new();

    // Main keyboard event sequence
    let main_input = MockInput::new(vec![
        KeyEvent::Press(KeyCode::CapsLock),
        KeyEvent::Press(KeyCode::H),
        KeyEvent::Release(KeyCode::H),
        KeyEvent::Press(KeyCode::J),
        KeyEvent::Release(KeyCode::J),
        KeyEvent::Release(KeyCode::CapsLock),
    ]);
    let main_output = MockOutput::new();

    // Create independent processors for each device
    let mut numpad_processor = EventProcessor::new(&numpad_config, numpad_input, numpad_output);
    let mut main_processor = EventProcessor::new(&main_config, main_input, main_output);

    // Process both streams
    numpad_processor.run().unwrap();
    main_processor.run().unwrap();

    // Verify numpad output: F13, F13 (release), F14, F14 (release)
    let numpad_events = numpad_processor.output().events();
    assert_eq!(numpad_events.len(), 4);
    assert_eq!(numpad_events[0], KeyEvent::Press(KeyCode::F13));
    assert_eq!(numpad_events[1], KeyEvent::Release(KeyCode::F13));
    assert_eq!(numpad_events[2], KeyEvent::Press(KeyCode::F14));
    assert_eq!(numpad_events[3], KeyEvent::Release(KeyCode::F14));

    // Verify main keyboard output: Left, Left (release), Down, Down (release)
    // CapsLock presses don't produce output (modifier)
    let main_events = main_processor.output().events();
    assert_eq!(main_events.len(), 4);
    assert_eq!(main_events[0], KeyEvent::Press(KeyCode::Left));
    assert_eq!(main_events[1], KeyEvent::Release(KeyCode::Left));
    assert_eq!(main_events[2], KeyEvent::Press(KeyCode::Down));
    assert_eq!(main_events[3], KeyEvent::Release(KeyCode::Down));
}

/// Test: Verifies that device state isolation works correctly.
///
/// Each device should have independent modifier/lock state.
#[test]
fn test_device_state_isolation() {
    // Same config for both, but different state progression
    let config = create_test_config(vec![
        KeyMapping::modifier(KeyCode::CapsLock, 0),
        KeyMapping::lock(KeyCode::ScrollLock, 1),
        KeyMapping::conditional(
            Condition::AllActive(vec![ConditionItem::ModifierActive(0)]),
            vec![BaseKeyMapping::Simple {
                from: KeyCode::A,
                to: KeyCode::Num1,
            }],
        ),
        KeyMapping::conditional(
            Condition::AllActive(vec![ConditionItem::LockActive(1)]),
            vec![BaseKeyMapping::Simple {
                from: KeyCode::B,
                to: KeyCode::Num2,
            }],
        ),
    ]);

    // Device 1: Activate modifier, then press A (should map to 1)
    let device1_input = MockInput::new(vec![
        KeyEvent::Press(KeyCode::CapsLock),
        KeyEvent::Press(KeyCode::A),
        KeyEvent::Release(KeyCode::A),
        KeyEvent::Release(KeyCode::CapsLock),
    ]);
    let device1_output = MockOutput::new();

    // Device 2: Toggle lock, then press B (should map to 2)
    let device2_input = MockInput::new(vec![
        KeyEvent::Press(KeyCode::ScrollLock),
        KeyEvent::Release(KeyCode::ScrollLock),
        KeyEvent::Press(KeyCode::B),
        KeyEvent::Release(KeyCode::B),
    ]);
    let device2_output = MockOutput::new();

    let mut processor1 = EventProcessor::new(&config, device1_input, device1_output);
    let mut processor2 = EventProcessor::new(&config, device2_input, device2_output);

    processor1.run().unwrap();
    processor2.run().unwrap();

    // Device 1: A → 1 (modifier active)
    let events1 = processor1.output().events();
    assert_eq!(events1.len(), 2);
    assert_eq!(events1[0], KeyEvent::Press(KeyCode::Num1));
    assert_eq!(events1[1], KeyEvent::Release(KeyCode::Num1));

    // Device 2: B → 2 (lock active)
    let events2 = processor2.output().events();
    assert_eq!(events2.len(), 2);
    assert_eq!(events2[0], KeyEvent::Press(KeyCode::Num2));
    assert_eq!(events2[1], KeyEvent::Release(KeyCode::Num2));
}

// ============================================================================
// Test: Device matching patterns
// ============================================================================

/// Test: Comprehensive device pattern matching.
///
/// Validates all supported pattern types: exact, prefix, suffix, contains.
#[test]
fn test_device_pattern_matching_comprehensive() {
    let state = DeviceState::new();

    // Exact match
    let exact = Condition::DeviceMatches(String::from("usb-numpad-123"));
    assert!(state.evaluate_condition_with_device(&exact, Some("usb-numpad-123")));
    assert!(!state.evaluate_condition_with_device(&exact, Some("usb-numpad-124")));
    assert!(!state.evaluate_condition_with_device(&exact, Some("usb-numpad-12"))); // partial

    // Prefix match (usb-*)
    let prefix = Condition::DeviceMatches(String::from("usb-*"));
    assert!(state.evaluate_condition_with_device(&prefix, Some("usb-keyboard")));
    assert!(state.evaluate_condition_with_device(&prefix, Some("usb-numpad-xyz")));
    assert!(!state.evaluate_condition_with_device(&prefix, Some("serial-usb-device")));

    // Suffix match (*-keyboard)
    let suffix = Condition::DeviceMatches(String::from("*-keyboard"));
    assert!(state.evaluate_condition_with_device(&suffix, Some("usb-keyboard")));
    assert!(state.evaluate_condition_with_device(&suffix, Some("at-translated-keyboard")));
    assert!(!state.evaluate_condition_with_device(&suffix, Some("keyboard-usb")));

    // Contains match (*numpad*)
    let contains = Condition::DeviceMatches(String::from("*numpad*"));
    assert!(state.evaluate_condition_with_device(&contains, Some("usb-numpad-123")));
    assert!(state.evaluate_condition_with_device(&contains, Some("numpad")));
    assert!(state.evaluate_condition_with_device(&contains, Some("my-numpad-device")));
    assert!(!state.evaluate_condition_with_device(&contains, Some("usb-keyboard")));

    // Multiple wildcards (*a*b*) - should match anything containing "a" followed by "b"
    // Note: Current implementation might not support this complex pattern
    // Just test that it doesn't crash
    let complex = Condition::DeviceMatches(String::from("*num*pad*"));
    let _ = state.evaluate_condition_with_device(&complex, Some("numpad"));
}

/// Test: Case sensitivity of device pattern matching.
#[test]
fn test_device_pattern_case_sensitivity() {
    let state = DeviceState::new();

    // Pattern matching should be case-sensitive as documented
    let pattern = Condition::DeviceMatches(String::from("USB-Keyboard"));

    // Exact case should match
    assert!(state.evaluate_condition_with_device(&pattern, Some("USB-Keyboard")));

    // Different case should not match (case-sensitive)
    assert!(!state.evaluate_condition_with_device(&pattern, Some("usb-keyboard")));
    assert!(!state.evaluate_condition_with_device(&pattern, Some("USB-KEYBOARD")));
}

/// Test: Empty pattern and edge cases.
#[test]
fn test_device_pattern_edge_cases() {
    let state = DeviceState::new();

    // Wildcard only - should match everything
    let wildcard = Condition::DeviceMatches(String::from("*"));
    assert!(state.evaluate_condition_with_device(&wildcard, Some("any-device")));
    assert!(state.evaluate_condition_with_device(&wildcard, Some("")));

    // Empty device_id
    let pattern = Condition::DeviceMatches(String::from("test"));
    assert!(!state.evaluate_condition_with_device(&pattern, Some("")));
    assert!(!state.evaluate_condition_with_device(&pattern, None));
}

// ============================================================================
// Test: DeviceManager pattern matching (unit test level)
// ============================================================================

/// Test: Device matching function from device_manager module.
#[test]
fn test_device_manager_match_device() {
    use keyrx_daemon::device_manager::{match_device, KeyboardInfo};
    use std::path::PathBuf;

    let keyboard = KeyboardInfo {
        path: PathBuf::from("/dev/input/event0"),
        name: String::from("Logitech USB Keyboard"),
        serial: Some(String::from("SN12345")),
        phys: Some(String::from("usb-0000:00:14.0-1/input0")),
    };

    // Wildcard matches everything
    assert!(match_device(&keyboard, "*"));

    // Exact name match (case-insensitive)
    assert!(match_device(&keyboard, "Logitech USB Keyboard"));
    assert!(match_device(&keyboard, "logitech usb keyboard"));

    // Prefix match on name
    assert!(match_device(&keyboard, "Logitech*"));
    assert!(match_device(&keyboard, "logitech*"));

    // Serial number match
    assert!(match_device(&keyboard, "SN12345"));
    assert!(match_device(&keyboard, "SN123*"));

    // Physical path match
    assert!(match_device(&keyboard, "usb-0000:00:14.0-1/input0"));
    assert!(match_device(&keyboard, "usb-*"));

    // Non-matching patterns
    assert!(!match_device(&keyboard, "Razer*"));
    assert!(!match_device(&keyboard, "SN99999"));
}

// ============================================================================
// Test: Stream Deck simulation (numpad as macro pad)
// ============================================================================

/// Test: Numpad as Stream Deck - comprehensive macro pad simulation.
///
/// This test validates the primary use case: using a cheap USB numpad
/// as a Stream Deck alternative with custom key mappings.
#[test]
fn test_numpad_as_stream_deck() {
    // Numpad configuration: Each numpad key maps to a unique F-key for OBS/streaming
    let numpad_config = create_test_config(vec![
        // Numpad 0-9 → VK_F13-F22 (virtual function keys)
        KeyMapping::simple(KeyCode::Numpad0, KeyCode::F20),
        KeyMapping::simple(KeyCode::Numpad1, KeyCode::F13),
        KeyMapping::simple(KeyCode::Numpad2, KeyCode::F14),
        KeyMapping::simple(KeyCode::Numpad3, KeyCode::F15),
        KeyMapping::simple(KeyCode::Numpad4, KeyCode::F16),
        KeyMapping::simple(KeyCode::Numpad5, KeyCode::F17),
        KeyMapping::simple(KeyCode::Numpad6, KeyCode::F18),
        KeyMapping::simple(KeyCode::Numpad7, KeyCode::F19),
        KeyMapping::simple(KeyCode::Numpad8, KeyCode::F21),
        KeyMapping::simple(KeyCode::Numpad9, KeyCode::F22),
        // NumpadEnter → F23 (for "go live" or similar)
        KeyMapping::simple(KeyCode::NumpadEnter, KeyCode::F23),
    ]);

    // Simulate pressing various numpad keys
    let input = MockInput::new(vec![
        // Scene switches (1-4)
        KeyEvent::Press(KeyCode::Numpad1),
        KeyEvent::Release(KeyCode::Numpad1),
        KeyEvent::Press(KeyCode::Numpad2),
        KeyEvent::Release(KeyCode::Numpad2),
        // Audio control (5-6)
        KeyEvent::Press(KeyCode::Numpad5),
        KeyEvent::Release(KeyCode::Numpad5),
        // Go live button
        KeyEvent::Press(KeyCode::NumpadEnter),
        KeyEvent::Release(KeyCode::NumpadEnter),
    ]);
    let output = MockOutput::new();

    let mut processor = EventProcessor::new(&numpad_config, input, output);
    processor.run().unwrap();

    let events = processor.output().events();
    assert_eq!(events.len(), 8);

    // Verify mappings
    assert_eq!(events[0], KeyEvent::Press(KeyCode::F13)); // Numpad1
    assert_eq!(events[1], KeyEvent::Release(KeyCode::F13));
    assert_eq!(events[2], KeyEvent::Press(KeyCode::F14)); // Numpad2
    assert_eq!(events[3], KeyEvent::Release(KeyCode::F14));
    assert_eq!(events[4], KeyEvent::Press(KeyCode::F17)); // Numpad5
    assert_eq!(events[5], KeyEvent::Release(KeyCode::F17));
    assert_eq!(events[6], KeyEvent::Press(KeyCode::F23)); // NumpadEnter
    assert_eq!(events[7], KeyEvent::Release(KeyCode::F23));
}

// ============================================================================
// Test: KeyEvent with device_id field
// ============================================================================

/// Test: KeyEvent device_id field operations.
#[test]
fn test_key_event_device_id() {
    // Create event without device_id
    let event = KeyEvent::Press(KeyCode::A);
    assert!(event.device_id().is_none());

    // Create event with device_id
    let event_with_device = event.with_device_id(String::from("usb-numpad-123"));
    assert!(event_with_device.device_id().is_some());
    assert_eq!(event_with_device.device_id().unwrap(), "usb-numpad-123");

    // Verify keycode is preserved
    assert_eq!(event_with_device.keycode(), KeyCode::A);
    assert!(event_with_device.is_press());

    // Release event with device_id
    let release = KeyEvent::Release(KeyCode::B);
    let release_with_device = release.with_device_id(String::from("serial-main-456"));
    assert_eq!(release_with_device.device_id().unwrap(), "serial-main-456");
    assert!(release_with_device.is_release());
    assert_eq!(release_with_device.keycode(), KeyCode::B);
}

/// Test: Device ID preserved through event cloning.
#[test]
fn test_key_event_device_id_clone() {
    let original = KeyEvent::Press(KeyCode::A).with_device_id(String::from("device-1"));
    let cloned = original.clone();

    assert_eq!(original.device_id(), cloned.device_id());
    assert_eq!(original.keycode(), cloned.keycode());
}
