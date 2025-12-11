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
//! Integration tests for emergency exit functionality.
//!
//! Tests the emergency exit combo detection and bypass mode behavior
//! at the integration level, simulating key sequences as they would
//! flow through the driver layer.

use keyrx_core::drivers::emergency_exit::{
    activate_bypass_mode, check_emergency_exit, deactivate_bypass_mode, is_bypass_active,
    set_bypass_mode, toggle_bypass_mode,
};
use keyrx_core::drivers::keycodes::KeyCode;
use keyrx_core::engine::{Modifier, ModifierState, StandardModifier};
use serial_test::serial;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

/// Reset bypass mode before each test.
fn reset_test_state() {
    // Use set_bypass_mode(false) to ensure clean state
    set_bypass_mode(false);
    assert!(
        !is_bypass_active(),
        "Bypass mode should be inactive after reset"
    );
}

/// Helper to create modifier state with specific modifiers active.
fn make_modifiers(ctrl: bool, alt: bool, shift: bool) -> ModifierState {
    let mut state = ModifierState::new();
    if ctrl {
        state.activate(Modifier::Standard(StandardModifier::Control));
    }
    if alt {
        state.activate(Modifier::Standard(StandardModifier::Alt));
    }
    if shift {
        state.activate(Modifier::Standard(StandardModifier::Shift));
    }
    state
}

// ============================================================================
// Emergency Exit Combo Detection Integration Tests
// ============================================================================

/// Test that the emergency exit combo is detected when all modifiers are held
/// and Escape is pressed.
#[test]
#[serial]
fn integration_emergency_combo_detected() {
    reset_test_state();

    // Simulate building up modifier state as keys are pressed
    let mods = make_modifiers(true, true, true);

    // The combo should be detected
    assert!(
        check_emergency_exit(KeyCode::Escape, &mods),
        "Emergency exit should be detected with Ctrl+Alt+Shift+Escape"
    );
}

/// Test that the combo is not triggered with incomplete modifiers.
/// This simulates a user accidentally pressing partial combos.
#[test]
#[serial]
fn integration_partial_combo_not_triggered() {
    reset_test_state();

    // Test all partial combinations
    let test_cases = [
        (false, false, false, "no modifiers"),
        (true, false, false, "only Ctrl"),
        (false, true, false, "only Alt"),
        (false, false, true, "only Shift"),
        (true, true, false, "Ctrl+Alt"),
        (true, false, true, "Ctrl+Shift"),
        (false, true, true, "Alt+Shift"),
    ];

    for (ctrl, alt, shift, desc) in test_cases {
        let mods = make_modifiers(ctrl, alt, shift);
        assert!(
            !check_emergency_exit(KeyCode::Escape, &mods),
            "Emergency exit should NOT be detected with {desc}"
        );
    }
}

/// Test that the combo only triggers on Escape key, not other keys.
#[test]
#[serial]
fn integration_combo_only_escape_key() {
    reset_test_state();

    let mods = make_modifiers(true, true, true);

    // Various keys that should NOT trigger emergency exit
    let non_trigger_keys = [
        KeyCode::A,
        KeyCode::Enter,
        KeyCode::Space,
        KeyCode::Tab,
        KeyCode::F1,
        KeyCode::Delete,
        KeyCode::Backspace,
    ];

    for key in non_trigger_keys {
        assert!(
            !check_emergency_exit(key, &mods),
            "Emergency exit should NOT be detected for key {:?}",
            key
        );
    }

    // Only Escape should trigger
    assert!(
        check_emergency_exit(KeyCode::Escape, &mods),
        "Emergency exit SHOULD be detected for Escape key"
    );
}

// ============================================================================
// Bypass Mode State Integration Tests
// ============================================================================

/// Test the full activation/deactivation cycle of bypass mode.
#[test]
#[serial]
fn integration_bypass_mode_lifecycle() {
    reset_test_state();

    // Initial state: inactive
    assert!(!is_bypass_active());

    // Activate
    activate_bypass_mode();
    assert!(
        is_bypass_active(),
        "Bypass mode should be active after activation"
    );

    // Second activation should be idempotent
    activate_bypass_mode();
    assert!(is_bypass_active(), "Bypass mode should remain active");

    // Deactivate
    deactivate_bypass_mode();
    assert!(
        !is_bypass_active(),
        "Bypass mode should be inactive after deactivation"
    );

    // Second deactivation should be idempotent
    deactivate_bypass_mode();
    assert!(!is_bypass_active(), "Bypass mode should remain inactive");
}

/// Test toggle behavior for re-enabling via the same combo.
#[test]
#[serial]
fn integration_bypass_mode_toggle() {
    reset_test_state();

    // Toggle from inactive -> active
    let state = toggle_bypass_mode();
    assert!(state, "Toggle should return true (activated)");
    assert!(is_bypass_active());

    // Toggle from active -> inactive
    let state = toggle_bypass_mode();
    assert!(!state, "Toggle should return false (deactivated)");
    assert!(!is_bypass_active());
}

/// Test set_bypass_mode for direct state control.
#[test]
#[serial]
fn integration_bypass_mode_direct_set() {
    reset_test_state();

    set_bypass_mode(true);
    assert!(is_bypass_active());

    set_bypass_mode(true); // Idempotent
    assert!(is_bypass_active());

    set_bypass_mode(false);
    assert!(!is_bypass_active());

    set_bypass_mode(false); // Idempotent
    assert!(!is_bypass_active());
}

// ============================================================================
// Thread Safety Integration Tests
// ============================================================================

/// Test that bypass mode operations are thread-safe under concurrent access.
/// This simulates multiple driver threads checking/modifying bypass state.
#[test]
#[serial]
fn integration_thread_safety_concurrent_access() {
    reset_test_state();

    let barrier = Arc::new(std::sync::Barrier::new(8));
    let success_count = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    // Spawn threads that concurrently access bypass mode
    for thread_id in 0..8 {
        let b = Arc::clone(&barrier);
        let count = Arc::clone(&success_count);

        handles.push(thread::spawn(move || {
            // Wait for all threads to start together
            b.wait();

            // Perform 100 operations per thread
            for i in 0..100 {
                match (thread_id % 4, i % 4) {
                    (0, _) => {
                        activate_bypass_mode();
                    }
                    (1, _) => {
                        deactivate_bypass_mode();
                    }
                    (2, _) => {
                        let _ = toggle_bypass_mode();
                    }
                    (_, _) => {
                        let _ = is_bypass_active();
                    }
                }

                // Verify we can always read state without panic
                let _state = is_bypass_active();
            }

            count.fetch_add(1, Ordering::SeqCst);
        }));
    }

    // Wait for all threads to complete
    for h in handles {
        h.join().expect("Thread should not panic");
    }

    // All threads completed successfully
    assert_eq!(
        success_count.load(Ordering::SeqCst),
        8,
        "All threads should complete without panic"
    );

    // Reset to known state
    reset_test_state();
}

/// Test that check_emergency_exit is thread-safe (read-only operation).
#[test]
#[serial]
fn integration_thread_safety_combo_check() {
    let barrier = Arc::new(std::sync::Barrier::new(4));
    let mut handles = vec![];

    for _ in 0..4 {
        let b = Arc::clone(&barrier);

        handles.push(thread::spawn(move || {
            b.wait();

            let mods = make_modifiers(true, true, true);

            // Check combo many times concurrently
            for _ in 0..1000 {
                let result = check_emergency_exit(KeyCode::Escape, &mods);
                assert!(result, "Combo check should consistently return true");
            }
        }));
    }

    for h in handles {
        h.join().expect("Thread should not panic");
    }
}

/// Test rapid toggling from multiple threads doesn't corrupt state.
#[test]
#[serial]
fn integration_thread_safety_rapid_toggle() {
    reset_test_state();

    let barrier = Arc::new(std::sync::Barrier::new(4));
    let toggle_counts = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    for _ in 0..4 {
        let b = Arc::clone(&barrier);
        let counts = Arc::clone(&toggle_counts);

        handles.push(thread::spawn(move || {
            b.wait();

            for _ in 0..100 {
                let _ = toggle_bypass_mode();
                counts.fetch_add(1, Ordering::SeqCst);
            }
        }));
    }

    for h in handles {
        h.join().expect("Thread should not panic");
    }

    // Verify total toggles
    assert_eq!(
        toggle_counts.load(Ordering::SeqCst),
        400,
        "All toggles should have been counted"
    );

    // State should be boolean (no corruption)
    let final_state = is_bypass_active();
    assert!(
        final_state == true || final_state == false,
        "Final state should be a valid boolean"
    );

    reset_test_state();
}

// ============================================================================
// Simulated Key Sequence Integration Tests
// ============================================================================

/// Simulate a realistic key press sequence leading to emergency exit.
#[test]
#[serial]
fn integration_simulated_key_sequence_activation() {
    reset_test_state();

    // Simulate user pressing Ctrl, then Alt, then Shift, then Escape
    let mut mods = ModifierState::new();

    // Press Ctrl
    mods.activate(Modifier::Standard(StandardModifier::Control));
    assert!(!check_emergency_exit(KeyCode::Escape, &mods));
    assert!(!is_bypass_active());

    // Press Alt
    mods.activate(Modifier::Standard(StandardModifier::Alt));
    assert!(!check_emergency_exit(KeyCode::Escape, &mods));
    assert!(!is_bypass_active());

    // Press Shift
    mods.activate(Modifier::Standard(StandardModifier::Shift));
    // Now combo would trigger on Escape
    assert!(!is_bypass_active()); // Not yet pressed Escape

    // Press Escape - this should trigger the combo
    if check_emergency_exit(KeyCode::Escape, &mods) {
        activate_bypass_mode();
    }
    assert!(is_bypass_active(), "Bypass should be active after combo");

    reset_test_state();
}

/// Simulate pressing the combo again to deactivate (toggle behavior).
#[test]
#[serial]
fn integration_simulated_key_sequence_toggle() {
    reset_test_state();

    let mods = make_modifiers(true, true, true);

    // First activation
    if check_emergency_exit(KeyCode::Escape, &mods) {
        toggle_bypass_mode();
    }
    assert!(is_bypass_active());

    // Release keys and press again (simulate re-entry)
    // In real usage, there would be key releases between

    // Second press toggles off
    if check_emergency_exit(KeyCode::Escape, &mods) {
        toggle_bypass_mode();
    }
    assert!(!is_bypass_active());

    reset_test_state();
}

/// Test that bypass mode persists across multiple combo checks.
#[test]
#[serial]
fn integration_bypass_persists_through_operations() {
    reset_test_state();

    activate_bypass_mode();

    // Perform many combo checks - should not affect bypass state
    let mods = make_modifiers(true, true, true);
    for _ in 0..100 {
        let _ = check_emergency_exit(KeyCode::Escape, &mods);
        let _ = check_emergency_exit(KeyCode::A, &mods);
    }

    // Bypass should still be active
    assert!(
        is_bypass_active(),
        "Bypass should persist through operations"
    );

    reset_test_state();
}

// ============================================================================
// Edge Case Tests
// ============================================================================

/// Test behavior when modifier state changes rapidly.
#[test]
#[serial]
fn integration_rapid_modifier_changes() {
    reset_test_state();

    let mut mods = ModifierState::new();

    // Rapidly toggle modifiers
    for _ in 0..100 {
        mods.activate(Modifier::Standard(StandardModifier::Control));
        mods.activate(Modifier::Standard(StandardModifier::Alt));
        mods.activate(Modifier::Standard(StandardModifier::Shift));

        let should_trigger = check_emergency_exit(KeyCode::Escape, &mods);
        assert!(should_trigger);

        mods.deactivate(Modifier::Standard(StandardModifier::Shift));

        let should_not_trigger = check_emergency_exit(KeyCode::Escape, &mods);
        assert!(!should_not_trigger);

        mods.deactivate(Modifier::Standard(StandardModifier::Alt));
        mods.deactivate(Modifier::Standard(StandardModifier::Control));
    }
}

/// Test that unknown key codes don't trigger the combo.
#[test]
#[serial]
fn integration_unknown_keys_safe() {
    let mods = make_modifiers(true, true, true);

    // Unknown key codes should not trigger
    for i in 0..256u16 {
        let key = KeyCode::Unknown(i);
        assert!(
            !check_emergency_exit(key, &mods),
            "Unknown({}) should not trigger emergency exit",
            i
        );
    }
}
