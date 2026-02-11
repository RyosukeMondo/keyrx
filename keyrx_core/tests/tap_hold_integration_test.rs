//! Integration tests for tap-hold functionality
//!
//! These tests verify the complete tap-hold pipeline:
//! - Configuration with tap-hold mappings
//! - Event processing through process_event
//! - Timeout handling with check_tap_hold_timeouts
//! - Permissive hold behavior
//! - Realistic usage patterns
//!
//! Tests use programmatically constructed configs to simulate
//! what would be produced by compiling Rhai configurations.

use keyrx_core::config::{
    BaseKeyMapping, Condition, DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping,
};
use keyrx_core::runtime::{
    check_tap_hold_timeouts, process_event, DeviceState, KeyEvent, KeyLookup,
};

// ============================================================================
// Test Helpers
// ============================================================================

/// Creates a DeviceConfig with the given mappings
fn create_config(mappings: Vec<KeyMapping>) -> DeviceConfig {
    DeviceConfig {
        identifier: DeviceIdentifier {
            pattern: String::from("*"),
        },
        mappings,
    }
}

/// Simulates a key tap (press and release within threshold)
fn tap_key(
    keycode: KeyCode,
    press_time: u64,
    release_time: u64,
    lookup: &KeyLookup,
    state: &mut DeviceState,
) -> Vec<KeyEvent> {
    let mut outputs = Vec::new();
    outputs.extend(process_event(
        KeyEvent::press(keycode).with_timestamp(press_time),
        lookup,
        state,
    ));
    outputs.extend(process_event(
        KeyEvent::release(keycode).with_timestamp(release_time),
        lookup,
        state,
    ));
    outputs
}

/// Simulates a key hold (press, wait past threshold, then release)
fn hold_key(
    keycode: KeyCode,
    press_time: u64,
    timeout_time: u64,
    release_time: u64,
    lookup: &KeyLookup,
    state: &mut DeviceState,
) -> Vec<KeyEvent> {
    let mut outputs = Vec::new();
    outputs.extend(process_event(
        KeyEvent::press(keycode).with_timestamp(press_time),
        lookup,
        state,
    ));
    outputs.extend(check_tap_hold_timeouts(timeout_time, state));
    outputs.extend(process_event(
        KeyEvent::release(keycode).with_timestamp(release_time),
        lookup,
        state,
    ));
    outputs
}

// ============================================================================
// Basic Tap-Hold Tests
// ============================================================================

#[test]
fn test_tap_hold_tap_produces_tap_key() {
    // CapsLock: tap=Escape, hold=modifier 0, threshold=200ms
    let config = create_config(vec![KeyMapping::tap_hold(
        KeyCode::CapsLock,
        KeyCode::Escape,
        0,
        200, // 200ms threshold
    )]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Tap CapsLock quickly (0ms to 100ms, under 200ms threshold)
    let outputs = tap_key(KeyCode::CapsLock, 0, 100_000, &lookup, &mut state);

    // Should produce Escape press and release
    assert_eq!(outputs.len(), 2, "Tap should produce press+release");
    assert_eq!(outputs[0].keycode(), KeyCode::Escape);
    assert!(outputs[0].is_press());
    assert_eq!(outputs[1].keycode(), KeyCode::Escape);
    assert!(outputs[1].is_release());

    // Modifier should not be active
    assert!(!state.is_modifier_active(0));
}

#[test]
fn test_tap_hold_hold_activates_modifier() {
    // CapsLock: tap=Escape, hold=modifier 0, threshold=200ms
    let config = create_config(vec![KeyMapping::tap_hold(
        KeyCode::CapsLock,
        KeyCode::Escape,
        0,
        200, // 200ms threshold
    )]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Press at 0ms
    let press_output = process_event(
        KeyEvent::press(KeyCode::CapsLock).with_timestamp(0),
        &lookup,
        &mut state,
    );
    assert!(
        press_output.is_empty(),
        "Press should produce no immediate output"
    );
    assert!(
        !state.is_modifier_active(0),
        "Modifier not active during pending"
    );

    // Check timeout at 250ms (past 200ms threshold)
    let _timeout_output = check_tap_hold_timeouts(250_000, &mut state);
    assert!(
        state.is_modifier_active(0),
        "Modifier should be active after timeout"
    );

    // Release at 300ms
    let release_output = process_event(
        KeyEvent::release(KeyCode::CapsLock).with_timestamp(300_000),
        &lookup,
        &mut state,
    );
    assert!(
        release_output.is_empty(),
        "Release should produce no key events for hold"
    );
    assert!(
        !state.is_modifier_active(0),
        "Modifier should be deactivated after release"
    );
}

#[test]
fn test_tap_hold_exact_threshold_boundary() {
    // Test behavior at exactly the threshold
    let config = create_config(vec![KeyMapping::tap_hold(
        KeyCode::CapsLock,
        KeyCode::Escape,
        0,
        200, // 200ms = 200,000 microseconds
    )]);
    let lookup = KeyLookup::from_device_config(&config);

    // Test at threshold - 1 (should be tap)
    let mut state1 = DeviceState::new();
    let outputs = tap_key(KeyCode::CapsLock, 0, 199_999, &lookup, &mut state1);
    assert_eq!(
        outputs.len(),
        2,
        "Release just before threshold should be tap"
    );
    assert_eq!(outputs[0].keycode(), KeyCode::Escape);

    // Test at threshold (should be hold)
    let mut state2 = DeviceState::new();
    let press_output = process_event(
        KeyEvent::press(KeyCode::CapsLock).with_timestamp(0),
        &lookup,
        &mut state2,
    );
    assert!(press_output.is_empty());

    // Release at exactly threshold time (200ms)
    let release_output = process_event(
        KeyEvent::release(KeyCode::CapsLock).with_timestamp(200_000),
        &lookup,
        &mut state2,
    );
    // At exactly threshold, it's considered a hold (elapsed >= threshold)
    assert!(
        release_output.is_empty(),
        "Release at threshold should be hold (no tap key)"
    );
}

// ============================================================================
// Permissive Hold Tests
// ============================================================================

#[test]
fn test_permissive_hold_on_other_key_press() {
    // CapsLock: tap=Escape, hold=modifier 0
    // When holding CapsLock and pressing another key, immediately confirm hold
    let config = create_config(vec![
        KeyMapping::tap_hold(KeyCode::CapsLock, KeyCode::Escape, 0, 200),
        KeyMapping::simple(KeyCode::A, KeyCode::B),
    ]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Press CapsLock at 0ms (enters pending)
    let _ = process_event(
        KeyEvent::press(KeyCode::CapsLock).with_timestamp(0),
        &lookup,
        &mut state,
    );
    assert!(!state.is_modifier_active(0), "Modifier not active yet");

    // Press A at 50ms (before 200ms threshold) - triggers permissive hold
    let outputs = process_event(
        KeyEvent::press(KeyCode::A).with_timestamp(50_000),
        &lookup,
        &mut state,
    );

    // Modifier should now be active (permissive hold triggered)
    assert!(
        state.is_modifier_active(0),
        "Modifier should be active after permissive hold"
    );

    // Output should include the remapped key
    assert!(
        outputs
            .iter()
            .any(|e| e.keycode() == KeyCode::B && e.is_press()),
        "Should output Press(B)"
    );
}

#[test]
fn test_permissive_hold_modifier_before_key() {
    // Permissive hold activates the modifier when another key is pressed.
    // The interrupting key is processed with the state AFTER permissive hold
    // activation, ensuring the correct layer mapping is used.
    let config = create_config(vec![
        KeyMapping::tap_hold(KeyCode::CapsLock, KeyCode::Escape, 0, 200),
        // Conditional: when MD_00 active, H -> Left
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

    // Press CapsLock at 0ms
    let _ = process_event(
        KeyEvent::press(KeyCode::CapsLock).with_timestamp(0),
        &lookup,
        &mut state,
    );

    // Press H at 50ms - triggers permissive hold, H is remapped to Left
    // because mapping lookup happens after permissive hold activation
    let outputs = process_event(
        KeyEvent::press(KeyCode::H).with_timestamp(50_000),
        &lookup,
        &mut state,
    );

    // The modifier is now active (permissive hold triggered)
    assert!(
        state.is_modifier_active(0),
        "Modifier should be active after permissive hold"
    );
    // The H itself is remapped to Left because mapping is re-evaluated after modifier activation
    assert!(
        outputs
            .iter()
            .any(|e| e.keycode() == KeyCode::Left && e.is_press()),
        "H should be remapped to Left because lookup happens after modifier activation"
    );

    // Now, pressing H again should be remapped to Left
    let outputs2 = process_event(
        KeyEvent::press(KeyCode::H).with_timestamp(100_000),
        &lookup,
        &mut state,
    );
    assert!(
        outputs2
            .iter()
            .any(|e| e.keycode() == KeyCode::Left && e.is_press()),
        "Second H should be remapped to Left because modifier is now active"
    );
}

#[test]
fn test_permissive_hold_multiple_tap_holds() {
    // Two tap-hold keys: CapsLock (MD_00) and Space (MD_01)
    let config = create_config(vec![
        KeyMapping::tap_hold(KeyCode::CapsLock, KeyCode::Escape, 0, 200),
        KeyMapping::tap_hold(KeyCode::Space, KeyCode::Space, 1, 200),
    ]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Press CapsLock at 0ms
    let _ = process_event(
        KeyEvent::press(KeyCode::CapsLock).with_timestamp(0),
        &lookup,
        &mut state,
    );

    // Press Space at 50ms (both now pending)
    let _ = process_event(
        KeyEvent::press(KeyCode::Space).with_timestamp(50_000),
        &lookup,
        &mut state,
    );

    // Press A at 100ms - should trigger permissive hold for BOTH pending keys
    let _ = process_event(
        KeyEvent::press(KeyCode::A).with_timestamp(100_000),
        &lookup,
        &mut state,
    );

    // Both modifiers should now be active
    assert!(state.is_modifier_active(0), "MD_00 should be active");
    assert!(state.is_modifier_active(1), "MD_01 should be active");
}

// ============================================================================
// Realistic Usage Patterns
// ============================================================================

#[test]
fn test_capslock_as_ctrl_with_vim_layer() {
    // Realistic config: CapsLock = tap(Escape) / hold(Ctrl-like layer)
    // With conditional mappings for HJKL navigation
    let config = create_config(vec![
        KeyMapping::tap_hold(KeyCode::CapsLock, KeyCode::Escape, 0, 200),
        KeyMapping::conditional(
            Condition::ModifierActive(0),
            vec![
                BaseKeyMapping::Simple {
                    from: KeyCode::H,
                    to: KeyCode::Left,
                },
                BaseKeyMapping::Simple {
                    from: KeyCode::J,
                    to: KeyCode::Down,
                },
                BaseKeyMapping::Simple {
                    from: KeyCode::K,
                    to: KeyCode::Up,
                },
                BaseKeyMapping::Simple {
                    from: KeyCode::L,
                    to: KeyCode::Right,
                },
            ],
        ),
    ]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Scenario 1: Tap CapsLock -> Escape
    let tap_outputs = tap_key(KeyCode::CapsLock, 0, 100_000, &lookup, &mut state);
    assert_eq!(tap_outputs.len(), 2);
    assert_eq!(tap_outputs[0].keycode(), KeyCode::Escape);
    assert_eq!(tap_outputs[1].keycode(), KeyCode::Escape);

    // Scenario 2: Hold CapsLock + J -> Down
    let _ = process_event(
        KeyEvent::press(KeyCode::CapsLock).with_timestamp(1_000_000),
        &lookup,
        &mut state,
    );

    // Wait for timeout
    let _ = check_tap_hold_timeouts(1_300_000, &mut state);
    assert!(state.is_modifier_active(0));

    // Press J - should map to Down
    let j_output = process_event(
        KeyEvent::press(KeyCode::J).with_timestamp(1_400_000),
        &lookup,
        &mut state,
    );
    assert_eq!(j_output.len(), 1);
    assert_eq!(j_output[0].keycode(), KeyCode::Down);

    // Release CapsLock
    let _ = process_event(
        KeyEvent::release(KeyCode::CapsLock).with_timestamp(1_500_000),
        &lookup,
        &mut state,
    );
    assert!(!state.is_modifier_active(0));

    // J should now pass through (no mapping active)
    let j_passthrough = process_event(
        KeyEvent::press(KeyCode::J).with_timestamp(1_600_000),
        &lookup,
        &mut state,
    );
    assert_eq!(j_passthrough.len(), 1);
    assert_eq!(j_passthrough[0].keycode(), KeyCode::J);
}

#[test]
fn test_space_as_navigation_layer() {
    // Space = tap(Space) / hold(Navigation layer)
    let config = create_config(vec![
        KeyMapping::tap_hold(KeyCode::Space, KeyCode::Space, 0, 200),
        KeyMapping::conditional(
            Condition::ModifierActive(0),
            vec![
                BaseKeyMapping::Simple {
                    from: KeyCode::H,
                    to: KeyCode::Left,
                },
                BaseKeyMapping::Simple {
                    from: KeyCode::J,
                    to: KeyCode::Down,
                },
                BaseKeyMapping::Simple {
                    from: KeyCode::K,
                    to: KeyCode::Up,
                },
                BaseKeyMapping::Simple {
                    from: KeyCode::L,
                    to: KeyCode::Right,
                },
            ],
        ),
    ]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Quick tap Space -> Space
    let tap_outputs = tap_key(KeyCode::Space, 0, 100_000, &lookup, &mut state);
    assert_eq!(tap_outputs.len(), 2);
    assert_eq!(tap_outputs[0].keycode(), KeyCode::Space);
    assert_eq!(tap_outputs[1].keycode(), KeyCode::Space);

    // Hold Space, then press H (permissive hold triggers)
    let _ = process_event(
        KeyEvent::press(KeyCode::Space).with_timestamp(1_000_000),
        &lookup,
        &mut state,
    );

    // Press H immediately (triggers permissive hold, H is remapped to Left)
    let h_output = process_event(
        KeyEvent::press(KeyCode::H).with_timestamp(1_050_000),
        &lookup,
        &mut state,
    );

    assert!(
        state.is_modifier_active(0),
        "Modifier should be active via permissive hold"
    );
    // H is remapped to Left (lookup happens after permissive hold activation)
    assert!(
        h_output.iter().any(|e| e.keycode() == KeyCode::Left),
        "First H should be remapped to Left (lookup after activation)"
    );

    // Press J - this time the modifier is already active, so J -> Down
    let j_output = process_event(
        KeyEvent::press(KeyCode::J).with_timestamp(1_100_000),
        &lookup,
        &mut state,
    );
    assert!(
        j_output.iter().any(|e| e.keycode() == KeyCode::Down),
        "J should map to Down because modifier is active"
    );
}

#[test]
fn test_enter_as_shift() {
    // Enter = tap(Enter) / hold(Shift-like for capital letters)
    let config = create_config(vec![
        KeyMapping::tap_hold(KeyCode::Enter, KeyCode::Enter, 0, 150),
        KeyMapping::conditional(
            Condition::ModifierActive(0),
            vec![BaseKeyMapping::ModifiedOutput {
                from: KeyCode::A,
                to: KeyCode::A,
                shift: true,
                ctrl: false,
                alt: false,
                win: false,
            }],
        ),
    ]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Tap Enter -> Enter
    let tap_outputs = tap_key(KeyCode::Enter, 0, 100_000, &lookup, &mut state);
    assert_eq!(tap_outputs.len(), 2);
    assert_eq!(tap_outputs[0].keycode(), KeyCode::Enter);

    // Hold Enter + A -> Shift+A
    let _ = process_event(
        KeyEvent::press(KeyCode::Enter).with_timestamp(1_000_000),
        &lookup,
        &mut state,
    );
    let _ = check_tap_hold_timeouts(1_200_000, &mut state);

    let a_output = process_event(
        KeyEvent::press(KeyCode::A).with_timestamp(1_250_000),
        &lookup,
        &mut state,
    );

    // Should produce Shift+A sequence
    assert!(
        a_output
            .iter()
            .any(|e| e.keycode() == KeyCode::LShift && e.is_press()),
        "Should have Shift press"
    );
    assert!(
        a_output
            .iter()
            .any(|e| e.keycode() == KeyCode::A && e.is_press()),
        "Should have A press"
    );
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_rapid_tap_hold_sequences() {
    // Test rapid consecutive taps
    let config = create_config(vec![KeyMapping::tap_hold(
        KeyCode::CapsLock,
        KeyCode::Escape,
        0,
        200,
    )]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // 3 rapid taps in sequence
    for i in 0..3 {
        let base_time = i * 200_000; // 200ms apart
        let tap_outputs = tap_key(
            KeyCode::CapsLock,
            base_time,
            base_time + 50_000, // 50ms tap
            &lookup,
            &mut state,
        );
        assert_eq!(
            tap_outputs.len(),
            2,
            "Tap {} should produce 2 events",
            i + 1
        );
        assert_eq!(tap_outputs[0].keycode(), KeyCode::Escape);
    }
}

#[test]
fn test_tap_hold_with_unmapped_interrupt() {
    // Tap-hold interrupted by a key that has no mapping (passthrough)
    let config = create_config(vec![KeyMapping::tap_hold(
        KeyCode::CapsLock,
        KeyCode::Escape,
        0,
        200,
    )]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Press CapsLock
    let _ = process_event(
        KeyEvent::press(KeyCode::CapsLock).with_timestamp(0),
        &lookup,
        &mut state,
    );

    // Press Z (unmapped, passthrough)
    let z_output = process_event(
        KeyEvent::press(KeyCode::Z).with_timestamp(50_000),
        &lookup,
        &mut state,
    );

    // Should trigger permissive hold and pass through Z
    assert!(state.is_modifier_active(0), "Modifier should be active");
    assert!(
        z_output
            .iter()
            .any(|e| e.keycode() == KeyCode::Z && e.is_press()),
        "Z should pass through"
    );
}

#[test]
fn test_multiple_different_thresholds() {
    // Different tap-hold keys with different thresholds
    let config = create_config(vec![
        KeyMapping::tap_hold(KeyCode::CapsLock, KeyCode::Escape, 0, 200), // 200ms
        KeyMapping::tap_hold(KeyCode::Enter, KeyCode::Enter, 1, 150),     // 150ms
    ]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Press both at 0ms
    let _ = process_event(
        KeyEvent::press(KeyCode::CapsLock).with_timestamp(0),
        &lookup,
        &mut state,
    );
    let _ = process_event(
        KeyEvent::press(KeyCode::Enter).with_timestamp(0),
        &lookup,
        &mut state,
    );

    // At 175ms: Enter (150ms threshold) should timeout, CapsLock (200ms) should not
    let _ = check_tap_hold_timeouts(175_000, &mut state);
    assert!(
        state.is_modifier_active(1),
        "Enter modifier should be active"
    );
    assert!(
        !state.is_modifier_active(0),
        "CapsLock modifier should NOT be active yet"
    );

    // At 225ms: CapsLock should also timeout
    let _ = check_tap_hold_timeouts(225_000, &mut state);
    assert!(
        state.is_modifier_active(0),
        "CapsLock modifier should now be active"
    );
    assert!(
        state.is_modifier_active(1),
        "Enter modifier should still be active"
    );
}

#[test]
fn test_tap_hold_no_interference_with_simple_mappings() {
    // Tap-hold on one key shouldn't affect simple mappings on other keys
    let config = create_config(vec![
        KeyMapping::tap_hold(KeyCode::CapsLock, KeyCode::Escape, 0, 200),
        KeyMapping::simple(KeyCode::A, KeyCode::B),
        KeyMapping::simple(KeyCode::C, KeyCode::D),
    ]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // A -> B should work regardless of tap-hold state
    let a_output = process_event(
        KeyEvent::press(KeyCode::A).with_timestamp(0),
        &lookup,
        &mut state,
    );
    assert_eq!(a_output.len(), 1);
    assert_eq!(a_output[0].keycode(), KeyCode::B);

    // Now press CapsLock
    let _ = process_event(
        KeyEvent::press(KeyCode::CapsLock).with_timestamp(1_000),
        &lookup,
        &mut state,
    );

    // C -> D should still work (but triggers permissive hold)
    let c_output = process_event(
        KeyEvent::press(KeyCode::C).with_timestamp(2_000),
        &lookup,
        &mut state,
    );
    assert!(c_output.iter().any(|e| e.keycode() == KeyCode::D));
}

// ============================================================================
// State Consistency Tests
// ============================================================================

#[test]
fn test_state_consistency_after_tap() {
    let config = create_config(vec![KeyMapping::tap_hold(
        KeyCode::CapsLock,
        KeyCode::Escape,
        0,
        200,
    )]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Tap
    let _ = tap_key(KeyCode::CapsLock, 0, 100_000, &lookup, &mut state);

    // State should be clean - no modifiers active, no pending keys
    assert!(!state.is_modifier_active(0));
    assert!(!state.tap_hold_processor_ref().has_pending_keys());
}

#[test]
fn test_state_consistency_after_hold() {
    let config = create_config(vec![KeyMapping::tap_hold(
        KeyCode::CapsLock,
        KeyCode::Escape,
        0,
        200,
    )]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Hold
    let _ = hold_key(KeyCode::CapsLock, 0, 250_000, 300_000, &lookup, &mut state);

    // State should be clean
    assert!(!state.is_modifier_active(0));
    assert!(!state.tap_hold_processor_ref().has_pending_keys());
}

#[test]
fn test_timeout_idempotence() {
    // Multiple timeout checks at the same time should be idempotent
    let config = create_config(vec![KeyMapping::tap_hold(
        KeyCode::CapsLock,
        KeyCode::Escape,
        0,
        200,
    )]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Press CapsLock
    let _ = process_event(
        KeyEvent::press(KeyCode::CapsLock).with_timestamp(0),
        &lookup,
        &mut state,
    );

    // Check timeout multiple times at same time
    let _ = check_tap_hold_timeouts(250_000, &mut state);
    assert!(state.is_modifier_active(0));

    let output2 = check_tap_hold_timeouts(250_000, &mut state);
    assert!(
        output2.is_empty(),
        "Second timeout check should not produce output"
    );
    assert!(
        state.is_modifier_active(0),
        "Modifier should still be active"
    );

    let output3 = check_tap_hold_timeouts(300_000, &mut state);
    assert!(
        output3.is_empty(),
        "Third timeout check should not produce output"
    );
}
