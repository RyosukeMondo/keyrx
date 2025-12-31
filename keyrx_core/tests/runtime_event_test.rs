//! Tests for runtime event processing
//!
//! This module contains comprehensive tests for the event processing logic
//! in keyrx_core::runtime::event, including:
//! - Simple, modifier, lock, and modified output mappings
//! - Conditional mappings and layer switching
//! - Tap-hold behavior (tap, hold, permissive hold)
//! - Property-based tests for invariants

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use keyrx_core::config::{
    BaseKeyMapping, Condition, DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping,
};
use keyrx_core::runtime::{
    check_tap_hold_timeouts, process_event, DeviceState, KeyEvent, KeyLookup,
};

/// Helper to create a test DeviceConfig with given mappings
fn create_test_config(mappings: Vec<KeyMapping>) -> DeviceConfig {
    DeviceConfig {
        identifier: DeviceIdentifier {
            pattern: String::from("*"),
        },
        mappings,
    }
}

#[test]
fn test_process_event_passthrough_no_mapping() {
    // Test passthrough: unmapped key returns original event
    let config = create_test_config(vec![]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Test Press event
    let input_press = KeyEvent::Press(KeyCode::A);
    let output = process_event(input_press, &lookup, &mut state);
    assert_eq!(output.len(), 1);
    assert_eq!(output[0], KeyEvent::Press(KeyCode::A));

    // Test Release event
    let input_release = KeyEvent::Release(KeyCode::A);
    let output = process_event(input_release, &lookup, &mut state);
    assert_eq!(output.len(), 1);
    assert_eq!(output[0], KeyEvent::Release(KeyCode::A));
}

#[test]
fn test_process_event_simple_mapping() {
    // Test Simple mapping: A -> B
    let config = create_test_config(vec![KeyMapping::simple(KeyCode::A, KeyCode::B)]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Test Press(A) returns Press(B)
    let input_press = KeyEvent::Press(KeyCode::A);
    let output = process_event(input_press, &lookup, &mut state);
    assert_eq!(output.len(), 1);
    assert_eq!(output[0], KeyEvent::Press(KeyCode::B));

    // Test Release(A) returns Release(B)
    let input_release = KeyEvent::Release(KeyCode::A);
    let output = process_event(input_release, &lookup, &mut state);
    assert_eq!(output.len(), 1);
    assert_eq!(output[0], KeyEvent::Release(KeyCode::B));
}

#[test]
fn test_process_event_modifier_mapping() {
    // Test Modifier mapping: sets state on press, clears on release, no output
    let config = create_test_config(vec![KeyMapping::modifier(KeyCode::CapsLock, 0)]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Verify modifier is initially inactive
    assert!(!state.is_modifier_active(0));

    // Press should set modifier and return empty Vec
    let input_press = KeyEvent::Press(KeyCode::CapsLock);
    let output = process_event(input_press, &lookup, &mut state);
    assert_eq!(output.len(), 0);
    assert!(state.is_modifier_active(0));

    // Release should clear modifier and return empty Vec
    let input_release = KeyEvent::Release(KeyCode::CapsLock);
    let output = process_event(input_release, &lookup, &mut state);
    assert_eq!(output.len(), 0);
    assert!(!state.is_modifier_active(0));
}

#[test]
fn test_modifier_multiple_cycles_no_sticky() {
    // Regression test: verify modifier doesn't become sticky after multiple press/release cycles
    // This catches the bug where MD_10 would stay active after release
    let config = create_test_config(vec![KeyMapping::modifier(KeyCode::N, 10)]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Verify initial state
    assert!(!state.is_modifier_active(10), "MD_10 should start inactive");

    // Cycle 1
    let _ = process_event(KeyEvent::Press(KeyCode::N), &lookup, &mut state);
    assert!(
        state.is_modifier_active(10),
        "MD_10 should activate on press"
    );
    let _ = process_event(KeyEvent::Release(KeyCode::N), &lookup, &mut state);
    assert!(
        !state.is_modifier_active(10),
        "MD_10 should deactivate on release"
    );

    // Cycle 2
    let _ = process_event(KeyEvent::Press(KeyCode::N), &lookup, &mut state);
    assert!(
        state.is_modifier_active(10),
        "MD_10 should activate on second press"
    );
    let _ = process_event(KeyEvent::Release(KeyCode::N), &lookup, &mut state);
    assert!(
        !state.is_modifier_active(10),
        "MD_10 should deactivate on second release"
    );

    // Cycle 3
    let _ = process_event(KeyEvent::Press(KeyCode::N), &lookup, &mut state);
    assert!(
        state.is_modifier_active(10),
        "MD_10 should activate on third press"
    );
    let _ = process_event(KeyEvent::Release(KeyCode::N), &lookup, &mut state);
    assert!(
        !state.is_modifier_active(10),
        "MD_10 should deactivate on third release"
    );
}

#[test]
fn test_modifier_rapid_sequence_cleanup() {
    // Test rapid press/release sequences to ensure proper cleanup
    let config = create_test_config(vec![KeyMapping::modifier(KeyCode::B, 0)]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Rapid sequence: press, release, press, release, press, release
    for i in 0..10 {
        let _ = process_event(KeyEvent::Press(KeyCode::B), &lookup, &mut state);
        assert!(
            state.is_modifier_active(0),
            "MD_00 should be active after press {}",
            i
        );
        let _ = process_event(KeyEvent::Release(KeyCode::B), &lookup, &mut state);
        assert!(
            !state.is_modifier_active(0),
            "MD_00 should be inactive after release {}",
            i
        );
    }

    // Final verification
    assert!(!state.is_modifier_active(0), "MD_00 should end inactive");
}

#[test]
fn test_modifier_state_independent_per_id() {
    // Test that different modifier IDs are independent and don't interfere
    let config = create_test_config(vec![
        KeyMapping::modifier(KeyCode::B, 0),
        KeyMapping::modifier(KeyCode::V, 1),
        KeyMapping::modifier(KeyCode::M, 2),
    ]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Activate MD_00
    let _ = process_event(KeyEvent::Press(KeyCode::B), &lookup, &mut state);
    assert!(state.is_modifier_active(0));
    assert!(!state.is_modifier_active(1));
    assert!(!state.is_modifier_active(2));

    // Activate MD_01 (MD_00 still active)
    let _ = process_event(KeyEvent::Press(KeyCode::V), &lookup, &mut state);
    assert!(state.is_modifier_active(0));
    assert!(state.is_modifier_active(1));
    assert!(!state.is_modifier_active(2));

    // Deactivate MD_00 (MD_01 still active)
    let _ = process_event(KeyEvent::Release(KeyCode::B), &lookup, &mut state);
    assert!(
        !state.is_modifier_active(0),
        "MD_00 should deactivate independently"
    );
    assert!(state.is_modifier_active(1), "MD_01 should remain active");
    assert!(!state.is_modifier_active(2));

    // Activate MD_02
    let _ = process_event(KeyEvent::Press(KeyCode::M), &lookup, &mut state);
    assert!(!state.is_modifier_active(0));
    assert!(state.is_modifier_active(1));
    assert!(state.is_modifier_active(2));

    // Deactivate all
    let _ = process_event(KeyEvent::Release(KeyCode::V), &lookup, &mut state);
    let _ = process_event(KeyEvent::Release(KeyCode::M), &lookup, &mut state);
    assert!(!state.is_modifier_active(0));
    assert!(!state.is_modifier_active(1));
    assert!(!state.is_modifier_active(2));
}

#[test]
fn test_modifier_no_output_events() {
    // Verify that modifier mappings NEVER produce output events
    // This is critical: modifiers only modify state, never inject keys
    let config = create_test_config(vec![KeyMapping::modifier(KeyCode::CapsLock, 5)]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Press should return empty Vec
    let output_press = process_event(KeyEvent::Press(KeyCode::CapsLock), &lookup, &mut state);
    assert_eq!(
        output_press.len(),
        0,
        "Modifier press should produce no output"
    );

    // Release should return empty Vec
    let output_release = process_event(KeyEvent::Release(KeyCode::CapsLock), &lookup, &mut state);
    assert_eq!(
        output_release.len(),
        0,
        "Modifier release should produce no output"
    );
}

#[test]
fn test_process_event_lock_mapping() {
    // Test Lock mapping: toggles on press, no output
    let config = create_test_config(vec![KeyMapping::lock(KeyCode::ScrollLock, 1)]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Verify lock is initially inactive
    assert!(!state.is_lock_active(1));

    // First press: toggle ON
    let input_press = KeyEvent::Press(KeyCode::ScrollLock);
    let output = process_event(input_press, &lookup, &mut state);
    assert_eq!(output.len(), 0);
    assert!(state.is_lock_active(1));

    // Release: do nothing
    let input_release = KeyEvent::Release(KeyCode::ScrollLock);
    let output = process_event(input_release, &lookup, &mut state);
    assert_eq!(output.len(), 0);
    assert!(state.is_lock_active(1)); // Still ON

    // Second press: toggle OFF
    let input_press = KeyEvent::Press(KeyCode::ScrollLock);
    let output = process_event(input_press, &lookup, &mut state);
    assert_eq!(output.len(), 0);
    assert!(!state.is_lock_active(1));

    // Release: do nothing
    let input_release = KeyEvent::Release(KeyCode::ScrollLock);
    let output = process_event(input_release, &lookup, &mut state);
    assert_eq!(output.len(), 0);
    assert!(!state.is_lock_active(1)); // Still OFF

    // Third press: toggle ON again
    let input_press = KeyEvent::Press(KeyCode::ScrollLock);
    let output = process_event(input_press, &lookup, &mut state);
    assert_eq!(output.len(), 0);
    assert!(state.is_lock_active(1));
}

#[test]
fn test_process_event_modified_output_shift() {
    // Test ModifiedOutput: Shift+1 sequence
    let config = create_test_config(vec![KeyMapping::modified_output(
        KeyCode::A,
        KeyCode::Num1,
        true,  // shift
        false, // ctrl
        false, // alt
        false, // win
    )]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Press should emit: Press(LShift), Press(Num1)
    let input_press = KeyEvent::Press(KeyCode::A);
    let output = process_event(input_press, &lookup, &mut state);
    assert_eq!(output.len(), 2);
    assert_eq!(output[0], KeyEvent::Press(KeyCode::LShift));
    assert_eq!(output[1], KeyEvent::Press(KeyCode::Num1));

    // Release should emit: Release(Num1), Release(LShift) (reverse order)
    let input_release = KeyEvent::Release(KeyCode::A);
    let output = process_event(input_release, &lookup, &mut state);
    assert_eq!(output.len(), 2);
    assert_eq!(output[0], KeyEvent::Release(KeyCode::Num1));
    assert_eq!(output[1], KeyEvent::Release(KeyCode::LShift));
}

#[test]
fn test_process_event_modified_output_ctrl_alt() {
    // Test ModifiedOutput: Ctrl+Alt+C sequence
    let config = create_test_config(vec![KeyMapping::modified_output(
        KeyCode::A,
        KeyCode::C,
        false, // shift
        true,  // ctrl
        true,  // alt
        false, // win
    )]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Press should emit: Press(LCtrl), Press(LAlt), Press(C)
    let input_press = KeyEvent::Press(KeyCode::A);
    let output = process_event(input_press, &lookup, &mut state);
    assert_eq!(output.len(), 3);
    assert_eq!(output[0], KeyEvent::Press(KeyCode::LCtrl));
    assert_eq!(output[1], KeyEvent::Press(KeyCode::LAlt));
    assert_eq!(output[2], KeyEvent::Press(KeyCode::C));

    // Release should emit in reverse: Release(C), Release(LAlt), Release(LCtrl)
    let input_release = KeyEvent::Release(KeyCode::A);
    let output = process_event(input_release, &lookup, &mut state);
    assert_eq!(output.len(), 3);
    assert_eq!(output[0], KeyEvent::Release(KeyCode::C));
    assert_eq!(output[1], KeyEvent::Release(KeyCode::LAlt));
    assert_eq!(output[2], KeyEvent::Release(KeyCode::LCtrl));
}

#[test]
fn test_process_event_modified_output_all_modifiers() {
    // Test ModifiedOutput: Shift+Ctrl+Alt+Win+Key sequence
    let config = create_test_config(vec![KeyMapping::modified_output(
        KeyCode::A,
        KeyCode::Z,
        true, // shift
        true, // ctrl
        true, // alt
        true, // win
    )]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Press should emit all modifiers then key
    let input_press = KeyEvent::Press(KeyCode::A);
    let output = process_event(input_press, &lookup, &mut state);
    assert_eq!(output.len(), 5);
    assert_eq!(output[0], KeyEvent::Press(KeyCode::LShift));
    assert_eq!(output[1], KeyEvent::Press(KeyCode::LCtrl));
    assert_eq!(output[2], KeyEvent::Press(KeyCode::LAlt));
    assert_eq!(output[3], KeyEvent::Press(KeyCode::LMeta));
    assert_eq!(output[4], KeyEvent::Press(KeyCode::Z));

    // Release should emit in reverse order
    let input_release = KeyEvent::Release(KeyCode::A);
    let output = process_event(input_release, &lookup, &mut state);
    assert_eq!(output.len(), 5);
    assert_eq!(output[0], KeyEvent::Release(KeyCode::Z));
    assert_eq!(output[1], KeyEvent::Release(KeyCode::LMeta));
    assert_eq!(output[2], KeyEvent::Release(KeyCode::LAlt));
    assert_eq!(output[3], KeyEvent::Release(KeyCode::LCtrl));
    assert_eq!(output[4], KeyEvent::Release(KeyCode::LShift));
}

#[test]
fn test_process_event_conditional_mapping_true() {
    // Test Conditional mapping: when modifier active, apply conditional mapping
    let config = create_test_config(vec![
        // CapsLock activates MD_00
        KeyMapping::modifier(KeyCode::CapsLock, 0),
        // when(MD_00): H -> Left
        KeyMapping::conditional(
            Condition::ModifierActive(0),
            vec![BaseKeyMapping::Simple {
                from: KeyCode::H,
                to: KeyCode::Left,
            }],
        ),
    ]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Activate modifier
    state.set_modifier(0);

    // With modifier active, H should map to Left
    let input_press = KeyEvent::Press(KeyCode::H);
    let output = process_event(input_press, &lookup, &mut state);
    assert_eq!(output.len(), 1);
    assert_eq!(output[0], KeyEvent::Press(KeyCode::Left));

    let input_release = KeyEvent::Release(KeyCode::H);
    let output = process_event(input_release, &lookup, &mut state);
    assert_eq!(output.len(), 1);
    assert_eq!(output[0], KeyEvent::Release(KeyCode::Left));
}

#[test]
fn test_process_event_conditional_mapping_false() {
    // Test Conditional mapping: when modifier NOT active, passthrough
    let config = create_test_config(vec![
        // CapsLock activates MD_00
        KeyMapping::modifier(KeyCode::CapsLock, 0),
        // when(MD_00): H -> Left
        KeyMapping::conditional(
            Condition::ModifierActive(0),
            vec![BaseKeyMapping::Simple {
                from: KeyCode::H,
                to: KeyCode::Left,
            }],
        ),
    ]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Modifier is NOT active, H should passthrough
    let input_press = KeyEvent::Press(KeyCode::H);
    let output = process_event(input_press, &lookup, &mut state);
    assert_eq!(output.len(), 1);
    assert_eq!(output[0], KeyEvent::Press(KeyCode::H)); // Passthrough

    let input_release = KeyEvent::Release(KeyCode::H);
    let output = process_event(input_release, &lookup, &mut state);
    assert_eq!(output.len(), 1);
    assert_eq!(output[0], KeyEvent::Release(KeyCode::H)); // Passthrough
}

#[test]
fn test_process_event_conditional_with_unconditional_fallback() {
    // Test conditional with fallback: if condition false, use unconditional mapping
    let config = create_test_config(vec![
        // when(MD_00): H -> Left (conditional)
        KeyMapping::conditional(
            Condition::ModifierActive(0),
            vec![BaseKeyMapping::Simple {
                from: KeyCode::H,
                to: KeyCode::Left,
            }],
        ),
        // H -> Down (unconditional fallback)
        KeyMapping::simple(KeyCode::H, KeyCode::Down),
    ]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Modifier NOT active: should use unconditional mapping (H -> Down)
    let input_press = KeyEvent::Press(KeyCode::H);
    let output = process_event(input_press, &lookup, &mut state);
    assert_eq!(output.len(), 1);
    assert_eq!(output[0], KeyEvent::Press(KeyCode::Down));

    // Activate modifier
    state.set_modifier(0);

    // Modifier active: should use conditional mapping (H -> Left)
    let input_press = KeyEvent::Press(KeyCode::H);
    let output = process_event(input_press, &lookup, &mut state);
    assert_eq!(output.len(), 1);
    assert_eq!(output[0], KeyEvent::Press(KeyCode::Left));
}

#[test]
fn test_process_event_vim_navigation_layer() {
    // Test realistic Vim navigation: CapsLock as layer, HJKL -> arrow keys
    let config = create_test_config(vec![
        // CapsLock activates MD_00
        KeyMapping::modifier(KeyCode::CapsLock, 0),
        // when(MD_00): H -> Left
        KeyMapping::conditional(
            Condition::ModifierActive(0),
            vec![BaseKeyMapping::Simple {
                from: KeyCode::H,
                to: KeyCode::Left,
            }],
        ),
        // when(MD_00): J -> Down
        KeyMapping::conditional(
            Condition::ModifierActive(0),
            vec![BaseKeyMapping::Simple {
                from: KeyCode::J,
                to: KeyCode::Down,
            }],
        ),
        // when(MD_00): K -> Up
        KeyMapping::conditional(
            Condition::ModifierActive(0),
            vec![BaseKeyMapping::Simple {
                from: KeyCode::K,
                to: KeyCode::Up,
            }],
        ),
        // when(MD_00): L -> Right
        KeyMapping::conditional(
            Condition::ModifierActive(0),
            vec![BaseKeyMapping::Simple {
                from: KeyCode::L,
                to: KeyCode::Right,
            }],
        ),
    ]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Press CapsLock to activate layer
    let _ = process_event(KeyEvent::Press(KeyCode::CapsLock), &lookup, &mut state);
    assert!(state.is_modifier_active(0));

    // Test H -> Left
    let output = process_event(KeyEvent::Press(KeyCode::H), &lookup, &mut state);
    assert_eq!(output[0], KeyEvent::Press(KeyCode::Left));

    // Test J -> Down
    let output = process_event(KeyEvent::Press(KeyCode::J), &lookup, &mut state);
    assert_eq!(output[0], KeyEvent::Press(KeyCode::Down));

    // Test K -> Up
    let output = process_event(KeyEvent::Press(KeyCode::K), &lookup, &mut state);
    assert_eq!(output[0], KeyEvent::Press(KeyCode::Up));

    // Test L -> Right
    let output = process_event(KeyEvent::Press(KeyCode::L), &lookup, &mut state);
    assert_eq!(output[0], KeyEvent::Press(KeyCode::Right));

    // Release CapsLock to deactivate layer
    let _ = process_event(KeyEvent::Release(KeyCode::CapsLock), &lookup, &mut state);
    assert!(!state.is_modifier_active(0));

    // H should now passthrough (no mapping when layer inactive)
    let output = process_event(KeyEvent::Press(KeyCode::H), &lookup, &mut state);
    assert_eq!(output[0], KeyEvent::Press(KeyCode::H));
}

#[test]
fn test_process_event_conditional_with_modified_output() {
    // Test conditional ModifiedOutput: when(MD_00): Y -> Ctrl+Z
    // This is the bug report case: modifier keys not applied in when clauses
    let config = create_test_config(vec![
        // CapsLock activates MD_00
        KeyMapping::modifier(KeyCode::CapsLock, 0),
        // when(MD_00): Y -> Ctrl+Z
        KeyMapping::conditional(
            Condition::ModifierActive(0),
            vec![BaseKeyMapping::ModifiedOutput {
                from: KeyCode::Y,
                to: KeyCode::Z,
                shift: false,
                ctrl: true,
                alt: false,
                win: false,
            }],
        ),
    ]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Activate modifier MD_00
    let _ = process_event(KeyEvent::Press(KeyCode::CapsLock), &lookup, &mut state);
    assert!(state.is_modifier_active(0));

    // Press Y with modifier active - should output Ctrl+Z (LCtrl press, then Z press)
    let output = process_event(KeyEvent::Press(KeyCode::Y), &lookup, &mut state);
    assert_eq!(output.len(), 2, "Expected 2 events: LCtrl press + Z press");
    assert_eq!(
        output[0],
        KeyEvent::Press(KeyCode::LCtrl),
        "First should be LCtrl press"
    );
    assert_eq!(
        output[1],
        KeyEvent::Press(KeyCode::Z),
        "Second should be Z press"
    );

    // Release Y - should output Z release, then LCtrl release
    let output = process_event(KeyEvent::Release(KeyCode::Y), &lookup, &mut state);
    assert_eq!(
        output.len(),
        2,
        "Expected 2 events: Z release + LCtrl release"
    );
    assert_eq!(
        output[0],
        KeyEvent::Release(KeyCode::Z),
        "First should be Z release"
    );
    assert_eq!(
        output[1],
        KeyEvent::Release(KeyCode::LCtrl),
        "Second should be LCtrl release"
    );
}

#[test]
fn test_process_event_conditional_with_modified_output_all_mods() {
    // Test conditional ModifiedOutput with all modifiers: when(MD_00): A -> Ctrl+Alt+Shift+Win+B
    let config = create_test_config(vec![
        KeyMapping::modifier(KeyCode::CapsLock, 0),
        KeyMapping::conditional(
            Condition::ModifierActive(0),
            vec![BaseKeyMapping::ModifiedOutput {
                from: KeyCode::A,
                to: KeyCode::B,
                shift: true,
                ctrl: true,
                alt: true,
                win: true,
            }],
        ),
    ]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Activate modifier
    state.set_modifier(0);

    // Press A - should output all modifiers then B
    let output = process_event(KeyEvent::Press(KeyCode::A), &lookup, &mut state);
    assert_eq!(output.len(), 5, "Expected 5 events: 4 modifiers + key");
    assert_eq!(output[0], KeyEvent::Press(KeyCode::LShift));
    assert_eq!(output[1], KeyEvent::Press(KeyCode::LCtrl));
    assert_eq!(output[2], KeyEvent::Press(KeyCode::LAlt));
    assert_eq!(output[3], KeyEvent::Press(KeyCode::LMeta));
    assert_eq!(output[4], KeyEvent::Press(KeyCode::B));

    // Release A - should release in reverse order
    let output = process_event(KeyEvent::Release(KeyCode::A), &lookup, &mut state);
    assert_eq!(output.len(), 5);
    assert_eq!(output[0], KeyEvent::Release(KeyCode::B));
    assert_eq!(output[1], KeyEvent::Release(KeyCode::LMeta));
    assert_eq!(output[2], KeyEvent::Release(KeyCode::LAlt));
    assert_eq!(output[3], KeyEvent::Release(KeyCode::LCtrl));
    assert_eq!(output[4], KeyEvent::Release(KeyCode::LShift));
}

#[test]
fn test_keyevent_keycode_helper() {
    // Test KeyEvent::keycode() helper method
    let press = KeyEvent::Press(KeyCode::A);
    assert_eq!(press.keycode(), KeyCode::A);

    let release = KeyEvent::Release(KeyCode::B);
    assert_eq!(release.keycode(), KeyCode::B);
}

#[test]
fn test_keyevent_derives() {
    use alloc::format;

    // Test that KeyEvent has all expected derives
    let event1 = KeyEvent::Press(KeyCode::A);
    let event2 = KeyEvent::Press(KeyCode::A);
    let event3 = KeyEvent::Release(KeyCode::A);

    // Test Clone trait
    let cloned = event1.clone();
    assert_eq!(cloned, event1);

    // Test PartialEq and Eq
    assert_eq!(event1, event2);
    assert_ne!(event1, event3);

    // Test Debug (should not panic)
    let _ = format!("{:?}", event1);

    // Test Hash - verify that equal events produce equal hashes
    use core::hash::{Hash, Hasher};

    // Simple test hasher that accumulates hash values
    struct TestHasher {
        value: u64,
    }

    impl Hasher for TestHasher {
        fn finish(&self) -> u64 {
            self.value
        }

        fn write(&mut self, bytes: &[u8]) {
            for &byte in bytes {
                self.value = self.value.wrapping_mul(31).wrapping_add(byte as u64);
            }
        }
    }

    let mut hasher1 = TestHasher { value: 0 };
    event1.hash(&mut hasher1);

    let mut hasher2 = TestHasher { value: 0 };
    event2.hash(&mut hasher2);

    // Equal values should have equal hashes
    assert_eq!(hasher1.finish(), hasher2.finish());
}

#[test]
fn test_keyevent_timestamp() {
    // Test timestamp functionality
    let event = KeyEvent::press(KeyCode::A);
    assert_eq!(event.timestamp_us(), 0); // Default is 0

    let event_with_ts = event.with_timestamp(1000);
    assert_eq!(event_with_ts.timestamp_us(), 1000);
    assert_eq!(event_with_ts.keycode(), KeyCode::A);
    assert!(event_with_ts.is_press());
}

#[test]
fn test_keyevent_device_id() {
    // Test device_id functionality
    let event = KeyEvent::press(KeyCode::A);
    assert_eq!(event.device_id(), None); // Default is None

    let event_with_device = event.with_device_id(String::from("usb-NumericKeypad-123"));
    assert_eq!(event_with_device.device_id(), Some("usb-NumericKeypad-123"));
    assert_eq!(event_with_device.keycode(), KeyCode::A);
    assert!(event_with_device.is_press());
}

#[test]
fn test_keyevent_device_id_with_timestamp() {
    // Test combining device_id with timestamp
    let event = KeyEvent::press(KeyCode::B)
        .with_timestamp(5000)
        .with_device_id(String::from("laptop-keyboard"));

    assert_eq!(event.keycode(), KeyCode::B);
    assert_eq!(event.timestamp_us(), 5000);
    assert_eq!(event.device_id(), Some("laptop-keyboard"));
    assert!(event.is_press());
}

#[test]
fn test_keyevent_with_keycode() {
    // Test with_keycode preserves type, timestamp, and device_id
    let event = KeyEvent::press(KeyCode::A)
        .with_timestamp(500)
        .with_device_id(String::from("test-device"));
    let remapped = event.with_keycode(KeyCode::B);

    assert_eq!(remapped.keycode(), KeyCode::B);
    assert!(remapped.is_press());
    assert_eq!(remapped.timestamp_us(), 500);
    assert_eq!(remapped.device_id(), Some("test-device"));
}

#[test]
fn test_keyevent_opposite() {
    // Test opposite preserves keycode, timestamp, and device_id
    let press = KeyEvent::press(KeyCode::A)
        .with_timestamp(1000)
        .with_device_id(String::from("numpad"));
    let release = press.opposite();

    assert!(release.is_release());
    assert_eq!(release.keycode(), KeyCode::A);
    assert_eq!(release.timestamp_us(), 1000);
    assert_eq!(release.device_id(), Some("numpad"));

    // Opposite of opposite is original type
    let press_again = release.opposite();
    assert!(press_again.is_press());
    assert_eq!(press_again.device_id(), Some("numpad"));
}

#[test]
fn test_keyevent_type() {
    // Test event_type accessor
    let press = KeyEvent::press(KeyCode::A);
    assert_eq!(press.event_type(), keyrx_core::runtime::KeyEventType::Press);

    let release = KeyEvent::release(KeyCode::A);
    assert_eq!(
        release.event_type(),
        keyrx_core::runtime::KeyEventType::Release
    );
}

// --- Tap-Hold Integration Tests ---

#[test]
fn test_process_event_tap_hold_tap_behavior() {
    // Test TapHold mapping: quick press and release produces tap key
    // CapsLock: tap=Escape, hold=modifier 0, threshold=200ms
    let config = create_test_config(vec![KeyMapping::tap_hold(
        KeyCode::CapsLock,
        KeyCode::Escape,
        0,
        200, // 200ms threshold
    )]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Press CapsLock at t=0
    let input_press = KeyEvent::press(KeyCode::CapsLock).with_timestamp(0);
    let output = process_event(input_press, &lookup, &mut state);
    assert!(
        output.is_empty(),
        "Press should produce no output (pending)"
    );

    // Quick release at t=100ms (under 200ms threshold) - this is a TAP
    let input_release = KeyEvent::release(KeyCode::CapsLock).with_timestamp(100_000);
    let output = process_event(input_release, &lookup, &mut state);
    assert_eq!(
        output.len(),
        2,
        "Tap should produce press+release of tap key"
    );
    assert_eq!(output[0].keycode(), KeyCode::Escape);
    assert!(output[0].is_press());
    assert_eq!(output[1].keycode(), KeyCode::Escape);
    assert!(output[1].is_release());
}

#[test]
fn test_process_event_tap_hold_hold_behavior() {
    // Test TapHold: hold past threshold, then release
    let config = create_test_config(vec![KeyMapping::tap_hold(
        KeyCode::CapsLock,
        KeyCode::Escape,
        0,
        200, // 200ms threshold
    )]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Press CapsLock at t=0
    let input_press = KeyEvent::press(KeyCode::CapsLock).with_timestamp(0);
    let _ = process_event(input_press, &lookup, &mut state);

    // Simulate timeout check (would be called by daemon)
    // For now, we test the "delayed hold" behavior: release after threshold
    // Release at t=300ms (over 200ms threshold)
    let input_release = KeyEvent::release(KeyCode::CapsLock).with_timestamp(300_000);
    let output = process_event(input_release, &lookup, &mut state);

    // Should activate and immediately deactivate the hold modifier
    // (since we didn't call check_timeouts, the release handles the delayed hold)
    assert!(
        output.is_empty(),
        "Hold release should produce no key events"
    );
    // The modifier state should be clean (activated then deactivated)
    assert!(
        !state.is_modifier_active(0),
        "Modifier should be inactive after release"
    );
}

#[test]
fn test_process_event_tap_hold_permissive_hold() {
    // Test Permissive Hold: press tap-hold key, then press another key
    let config = create_test_config(vec![
        KeyMapping::tap_hold(
            KeyCode::CapsLock,
            KeyCode::Escape,
            0,
            200, // 200ms threshold
        ),
        KeyMapping::simple(KeyCode::A, KeyCode::B),
    ]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Press CapsLock at t=0 (enters pending state)
    let input_press = KeyEvent::press(KeyCode::CapsLock).with_timestamp(0);
    let _ = process_event(input_press, &lookup, &mut state);
    assert!(!state.is_modifier_active(0), "Modifier not active yet");

    // Press A at t=50ms (before threshold) - should trigger permissive hold
    let input_a = KeyEvent::press(KeyCode::A).with_timestamp(50_000);
    let output = process_event(input_a, &lookup, &mut state);

    // Modifier should now be active (permissive hold triggered)
    assert!(
        state.is_modifier_active(0),
        "Modifier should be active after permissive hold"
    );

    // Output should include the remapped key (A -> B)
    // The B event should come after permissive hold activation
    assert!(
        output
            .iter()
            .any(|e| e.keycode() == KeyCode::B && e.is_press()),
        "Should output Press(B)"
    );
}

#[test]
fn test_check_tap_hold_timeouts_triggers_hold() {
    // Test that check_tap_hold_timeouts properly detects timeout and triggers hold
    let config = create_test_config(vec![KeyMapping::tap_hold(
        KeyCode::CapsLock,
        KeyCode::Escape,
        0,
        200, // 200ms threshold
    )]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Press CapsLock at t=0 (enters pending state)
    let input_press = KeyEvent::press(KeyCode::CapsLock).with_timestamp(0);
    let output = process_event(input_press, &lookup, &mut state);

    // Initial press should produce no output (pending state)
    assert!(
        output.is_empty(),
        "Press should produce no output in pending state"
    );
    assert!(
        !state.is_modifier_active(0),
        "Modifier should not be active yet"
    );

    // Check timeouts at t=150ms (before threshold) - should not trigger
    let timeout_events = check_tap_hold_timeouts(150_000, &mut state);
    assert!(
        timeout_events.is_empty(),
        "Should not trigger timeout before threshold"
    );
    assert!(!state.is_modifier_active(0), "Modifier still not active");

    // Check timeouts at t=250ms (after threshold) - should trigger hold
    let _timeout_events = check_tap_hold_timeouts(250_000, &mut state);
    assert!(
        state.is_modifier_active(0),
        "Modifier should be active after timeout"
    );
    // The timeout check may or may not produce events depending on implementation
    // The key assertion is that the modifier is now active
}

#[test]
fn test_check_tap_hold_timeouts_no_pending() {
    // Test that check_tap_hold_timeouts returns empty when no pending keys
    let mut state = DeviceState::new();

    // No pending tap-hold keys, should return empty
    let timeout_events = check_tap_hold_timeouts(1_000_000, &mut state);
    assert!(
        timeout_events.is_empty(),
        "Should be empty with no pending keys"
    );
}

/// CRITICAL BUG TEST: Conditional mapping with base layer ModifiedOutput
///
/// Scenario (user's actual bug):
/// - Base layer: Num2 -> S+7 (Shift+7)
/// - MD_00 layer: Num2 -> Left
/// - MD_00 is activated via TapHold on CapsLock
///
/// When user holds CapsLock (tap-hold pending) and presses Num2:
/// - Expected: Left arrow (conditional mapping takes precedence)
/// - Bug: Shift+Left (base layer's Shift leaks through)
///
/// Root cause: Mapping lookup happens BEFORE permissive hold activates the modifier
#[test]
fn test_tap_hold_permissive_hold_with_conditional_vs_base_modified_output() {
    // Setup: TapHold for MD_00, base ModifiedOutput, conditional Simple
    let config = create_test_config(vec![
        // CapsLock: tap=Escape, hold=MD_00
        KeyMapping::tap_hold(KeyCode::CapsLock, KeyCode::Escape, 0, 200),
        // Base layer: Num2 -> Shift+7 (ModifiedOutput)
        KeyMapping::modified_output(
            KeyCode::Num2,
            KeyCode::Num7,
            true,  // shift
            false, // ctrl
            false, // alt
            false, // win
        ),
        // MD_00 layer: Num2 -> Left (Simple, should take precedence when MD_00 active)
        KeyMapping::conditional(
            Condition::ModifierActive(0),
            vec![BaseKeyMapping::Simple {
                from: KeyCode::Num2,
                to: KeyCode::Left,
            }],
        ),
    ]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Step 1: Press CapsLock at t=0 (enters pending state)
    let output = process_event(
        KeyEvent::press(KeyCode::CapsLock).with_timestamp(0),
        &lookup,
        &mut state,
    );
    assert!(output.is_empty(), "TapHold press should produce no output");
    assert!(
        !state.is_modifier_active(0),
        "MD_00 should not be active yet (pending)"
    );

    // Step 2: Press Num2 at t=50ms (triggers permissive hold)
    // CRITICAL: This should output Left, NOT Shift+7
    let output = process_event(
        KeyEvent::press(KeyCode::Num2).with_timestamp(50_000),
        &lookup,
        &mut state,
    );

    // After permissive hold, MD_00 should be active
    assert!(
        state.is_modifier_active(0),
        "MD_00 should be active after permissive hold"
    );

    // THE CRITICAL ASSERTION: Should output just Left, not Shift+7
    // If this fails, the bug is confirmed
    assert_eq!(
        output.len(),
        1,
        "Should output 1 event (Left), got {} events: {:?}",
        output.len(),
        output
    );
    assert_eq!(
        output[0].keycode(),
        KeyCode::Left,
        "Should output Left, not Shift+7. Got: {:?}",
        output[0]
    );
    assert!(output[0].is_press(), "Should be Press event");

    // Verify NO Shift key was emitted
    let has_shift = output.iter().any(|e| e.keycode() == KeyCode::LShift);
    assert!(
        !has_shift,
        "Shift should NOT be in output! Base layer ModifiedOutput leaked through"
    );
}

/// Test that conditional mapping is selected AFTER permissive hold activates modifier
#[test]
fn test_permissive_hold_activates_before_lookup() {
    let config = create_test_config(vec![
        // TapHold: CapsLock -> MD_00
        KeyMapping::tap_hold(KeyCode::CapsLock, KeyCode::Escape, 0, 200),
        // Conditional: when(MD_00): A -> B
        KeyMapping::conditional(
            Condition::ModifierActive(0),
            vec![BaseKeyMapping::Simple {
                from: KeyCode::A,
                to: KeyCode::B,
            }],
        ),
    ]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Press CapsLock (pending)
    let _ = process_event(
        KeyEvent::press(KeyCode::CapsLock).with_timestamp(0),
        &lookup,
        &mut state,
    );

    // Press A - should trigger permissive hold AND use conditional mapping
    let output = process_event(
        KeyEvent::press(KeyCode::A).with_timestamp(50_000),
        &lookup,
        &mut state,
    );

    // Modifier should be active
    assert!(state.is_modifier_active(0));

    // Should output B (conditional mapping), not A (passthrough)
    assert_eq!(output.len(), 1);
    assert_eq!(
        output[0].keycode(),
        KeyCode::B,
        "Should use conditional mapping after permissive hold activates MD_00"
    );
}

// Property-based tests using proptest
#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    // Strategy to generate arbitrary KeyCode values (A-Z range for simplicity)
    fn keycode_strategy() -> impl Strategy<Value = KeyCode> {
        prop::sample::select(vec![
            KeyCode::A,
            KeyCode::B,
            KeyCode::C,
            KeyCode::D,
            KeyCode::E,
            KeyCode::F,
            KeyCode::G,
            KeyCode::H,
            KeyCode::I,
            KeyCode::J,
            KeyCode::K,
            KeyCode::L,
            KeyCode::M,
            KeyCode::N,
            KeyCode::O,
            KeyCode::P,
            KeyCode::Q,
            KeyCode::R,
            KeyCode::S,
            KeyCode::T,
            KeyCode::U,
            KeyCode::V,
            KeyCode::W,
            KeyCode::X,
            KeyCode::Y,
            KeyCode::Z,
        ])
    }

    // Strategy to generate arbitrary KeyEvent values
    fn keyevent_strategy() -> impl Strategy<Value = KeyEvent> {
        (keycode_strategy(), prop::bool::ANY).prop_map(|(keycode, is_press)| {
            if is_press {
                KeyEvent::Press(keycode)
            } else {
                KeyEvent::Release(keycode)
            }
        })
    }

    proptest! {
        /// Property test: No event loss for Simple mappings
        #[test]
        fn prop_no_event_loss(events in prop::collection::vec(keyevent_strategy(), 1..100)) {
            let config = create_test_config(vec![KeyMapping::simple(KeyCode::A, KeyCode::B)]);
            let lookup = KeyLookup::from_device_config(&config);
            let mut state = DeviceState::new();

            let mut total_output_count = 0;
            for event in &events {
                let outputs = process_event(event.clone(), &lookup, &mut state);
                total_output_count += outputs.len();
            }

            prop_assert_eq!(events.len(), total_output_count,
                "Event loss detected: {} inputs produced {} outputs",
                events.len(), total_output_count);
        }

        /// Property test: Deterministic execution
        #[test]
        fn prop_deterministic(events in prop::collection::vec(keyevent_strategy(), 1..50)) {
            let config = create_test_config(vec![KeyMapping::simple(KeyCode::A, KeyCode::B)]);
            let lookup = KeyLookup::from_device_config(&config);

            let mut state1 = DeviceState::new();
            let mut outputs1 = Vec::new();
            for event in &events {
                let result = process_event(event.clone(), &lookup, &mut state1);
                outputs1.extend(result);
            }

            let mut state2 = DeviceState::new();
            let mut outputs2 = Vec::new();
            for event in &events {
                let result = process_event(event.clone(), &lookup, &mut state2);
                outputs2.extend(result);
            }

            prop_assert_eq!(outputs1, outputs2,
                "Non-deterministic behavior: same inputs produced different outputs");
        }

        /// Property test: Modifier events produce no output
        #[test]
        fn prop_modifier_no_output(events in prop::collection::vec(keyevent_strategy(), 1..50)) {
            let config = create_test_config(vec![KeyMapping::modifier(KeyCode::A, 0)]);
            let lookup = KeyLookup::from_device_config(&config);
            let mut state = DeviceState::new();

            for event in &events {
                let outputs = process_event(event.clone(), &lookup, &mut state);

                if event.keycode() == KeyCode::A {
                    prop_assert!(outputs.is_empty(),
                        "Modifier mapping produced output: {:?}", outputs);
                }
            }
        }

        /// Property test: Lock events produce no output
        #[test]
        fn prop_lock_no_output(events in prop::collection::vec(keyevent_strategy(), 1..50)) {
            let config = create_test_config(vec![KeyMapping::lock(KeyCode::A, 0)]);
            let lookup = KeyLookup::from_device_config(&config);
            let mut state = DeviceState::new();

            for event in &events {
                let outputs = process_event(event.clone(), &lookup, &mut state);

                if event.keycode() == KeyCode::A {
                    prop_assert!(outputs.is_empty(),
                        "Lock mapping produced output: {:?}", outputs);
                }
            }
        }

        /// Property test: Passthrough preserves event type
        #[test]
        fn prop_passthrough_preserves_event(events in prop::collection::vec(keyevent_strategy(), 1..50)) {
            let config = create_test_config(vec![]);
            let lookup = KeyLookup::from_device_config(&config);
            let mut state = DeviceState::new();

            for event in &events {
                let outputs = process_event(event.clone(), &lookup, &mut state);

                prop_assert_eq!(outputs.len(), 1,
                    "Passthrough produced {} outputs for 1 input", outputs.len());

                prop_assert_eq!(&outputs[0], event,
                    "Passthrough modified event: {:?} became {:?}", event, outputs[0]);
            }
        }

        /// Property test: Simple mapping preserves event type
        #[test]
        fn prop_simple_preserves_type(events in prop::collection::vec(keyevent_strategy(), 1..50)) {
            let config = create_test_config(vec![KeyMapping::simple(KeyCode::A, KeyCode::B)]);
            let lookup = KeyLookup::from_device_config(&config);
            let mut state = DeviceState::new();

            for event in &events {
                let outputs = process_event(event.clone(), &lookup, &mut state);

                prop_assert_eq!(outputs.len(), 1,
                    "Simple mapping produced {} outputs for 1 input", outputs.len());

                // Verify event type is preserved
                let same_type = (event.is_press() && outputs[0].is_press()) ||
                               (event.is_release() && outputs[0].is_release());
                prop_assert!(same_type,
                    "Event type not preserved: {:?} became {:?}", event, outputs[0]);
            }
        }
    }
}
