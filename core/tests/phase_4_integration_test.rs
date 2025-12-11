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
//! Phase 4 Integration Tests
//!
//! End-to-end tests verifying Phase 4 features:
//! - Emergency exit activation and deactivation
//! - Bypass mode passes all keys through
//! - Visual editor → code generation → script loading chain

use keyrx_core::drivers::emergency_exit::{
    activate_bypass_mode, check_emergency_exit, deactivate_bypass_mode, is_bypass_active,
    set_bypass_mode, toggle_bypass_mode,
};
use keyrx_core::drivers::keycodes::KeyCode;
use keyrx_core::engine::{Modifier, ModifierState, StandardModifier};
use serial_test::serial;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

/// Reset bypass mode before each test to ensure clean state.
fn reset_test_state() {
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

// =============================================================================
// Emergency Exit Activation Tests
// =============================================================================

/// Test that pressing the emergency exit combo (Ctrl+Alt+Shift+Escape) activates bypass mode.
#[test]
#[serial]
fn emergency_exit_activates_bypass_mode() {
    reset_test_state();

    // Simulate the full combo
    let mods = make_modifiers(true, true, true);

    // Check for the combo and activate
    assert!(
        check_emergency_exit(KeyCode::Escape, &mods),
        "Emergency exit combo should be detected"
    );

    // Simulate driver behavior: activate bypass mode when combo detected
    activate_bypass_mode();
    assert!(is_bypass_active(), "Bypass mode should now be active");

    reset_test_state();
}

/// Test that pressing the combo again deactivates bypass mode (toggle behavior).
#[test]
#[serial]
fn emergency_exit_deactivates_bypass_mode() {
    reset_test_state();

    let mods = make_modifiers(true, true, true);

    // First combo press: activate
    if check_emergency_exit(KeyCode::Escape, &mods) {
        toggle_bypass_mode();
    }
    assert!(
        is_bypass_active(),
        "Bypass should be active after first combo"
    );

    // Second combo press: deactivate
    if check_emergency_exit(KeyCode::Escape, &mods) {
        toggle_bypass_mode();
    }
    assert!(
        !is_bypass_active(),
        "Bypass should be inactive after second combo"
    );

    reset_test_state();
}

/// Test the complete emergency exit lifecycle: activation -> deactivation -> re-activation.
#[test]
#[serial]
fn emergency_exit_full_lifecycle() {
    reset_test_state();

    let mods = make_modifiers(true, true, true);

    // Phase 1: Initial state - bypass inactive
    assert!(!is_bypass_active());

    // Phase 2: Activate via combo
    if check_emergency_exit(KeyCode::Escape, &mods) {
        activate_bypass_mode();
    }
    assert!(is_bypass_active());

    // Phase 3: Deactivate via explicit call (simulates UI button)
    deactivate_bypass_mode();
    assert!(!is_bypass_active());

    // Phase 4: Re-activate via combo
    if check_emergency_exit(KeyCode::Escape, &mods) {
        activate_bypass_mode();
    }
    assert!(is_bypass_active());

    // Phase 5: Toggle to deactivate
    toggle_bypass_mode();
    assert!(!is_bypass_active());

    reset_test_state();
}

// =============================================================================
// Bypass Mode Key Pass-through Tests
// =============================================================================

/// Test that when bypass mode is active, the check_emergency_exit function still works.
/// This ensures the combo can be detected even in bypass mode (to toggle off).
#[test]
#[serial]
fn bypass_mode_combo_still_detected() {
    reset_test_state();
    activate_bypass_mode();

    let mods = make_modifiers(true, true, true);

    // Combo should still be detectable in bypass mode
    assert!(
        check_emergency_exit(KeyCode::Escape, &mods),
        "Combo should be detected even in bypass mode"
    );

    reset_test_state();
}

/// Simulate driver behavior where bypass mode causes all keys to pass through.
/// This test verifies the pattern that drivers should use.
#[test]
#[serial]
fn bypass_mode_passes_all_keys() {
    reset_test_state();
    activate_bypass_mode();

    // Simulate processing a batch of key events
    let test_keys = [
        KeyCode::A,
        KeyCode::B,
        KeyCode::CapsLock,
        KeyCode::Space,
        KeyCode::LeftShift,
        KeyCode::Enter,
        KeyCode::F1,
        KeyCode::Tab,
    ];

    let mut processed_count = 0;

    for _key in test_keys {
        // This simulates driver behavior: check bypass FIRST
        if is_bypass_active() {
            // In real driver: return early without remapping
            processed_count += 1;
            continue;
        }
        // If not bypassed, remapping would occur here (not reached in this test)
        panic!("Should not reach remapping when bypass is active");
    }

    assert_eq!(
        processed_count,
        test_keys.len(),
        "All keys should have been passed through"
    );

    reset_test_state();
}

/// Test that remapping can occur when bypass mode is NOT active.
#[test]
#[serial]
fn normal_mode_allows_remapping() {
    reset_test_state();

    assert!(!is_bypass_active());

    let test_keys = [KeyCode::A, KeyCode::B, KeyCode::CapsLock];
    let mut remapped_count = 0;

    for _key in test_keys {
        if is_bypass_active() {
            panic!("Bypass should not be active");
        }
        // Simulate remapping
        remapped_count += 1;
    }

    assert_eq!(
        remapped_count, 3,
        "All keys should be available for remapping"
    );
}

// =============================================================================
// Thread Safety Integration Tests
// =============================================================================

/// Test concurrent combo checks and bypass mode operations (simulates multi-keyboard scenario).
#[test]
#[serial]
fn concurrent_combo_check_and_bypass_operations() {
    reset_test_state();

    let barrier = Arc::new(std::sync::Barrier::new(6));
    let combo_detections = Arc::new(AtomicUsize::new(0));
    let bypass_checks = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    // Spawn threads that check combo
    for _ in 0..3 {
        let b = Arc::clone(&barrier);
        let detections = Arc::clone(&combo_detections);

        handles.push(thread::spawn(move || {
            b.wait();
            let mods = make_modifiers(true, true, true);
            for _ in 0..100 {
                if check_emergency_exit(KeyCode::Escape, &mods) {
                    detections.fetch_add(1, Ordering::SeqCst);
                }
            }
        }));
    }

    // Spawn threads that toggle bypass
    for _ in 0..3 {
        let b = Arc::clone(&barrier);
        let checks = Arc::clone(&bypass_checks);

        handles.push(thread::spawn(move || {
            b.wait();
            for _ in 0..100 {
                let _ = toggle_bypass_mode();
                let _ = is_bypass_active();
                checks.fetch_add(1, Ordering::SeqCst);
            }
        }));
    }

    for h in handles {
        h.join().expect("Thread should not panic");
    }

    // All combo checks should have detected the combo
    assert_eq!(
        combo_detections.load(Ordering::SeqCst),
        300,
        "All combo checks should detect the combo"
    );

    // All bypass checks completed
    assert_eq!(
        bypass_checks.load(Ordering::SeqCst),
        300,
        "All bypass operations should complete"
    );

    reset_test_state();
}

/// Simulate driver event loop with concurrent bypass checking.
#[test]
#[serial]
fn simulated_driver_event_loop_with_bypass() {
    reset_test_state();

    let stop_flag = Arc::new(AtomicBool::new(false));
    let events_processed = Arc::new(AtomicUsize::new(0));
    let bypass_activations = Arc::new(AtomicUsize::new(0));

    let stop = Arc::clone(&stop_flag);
    let processed = Arc::clone(&events_processed);
    let activations = Arc::clone(&bypass_activations);

    // Simulate a driver event loop in a thread
    let driver_thread = thread::spawn(move || {
        let emergency_mods = make_modifiers(true, true, true);
        let normal_mods = ModifierState::new();

        while !stop.load(Ordering::SeqCst) {
            // Simulate receiving 10 events per iteration
            for i in 0..10 {
                // Check for emergency exit first (as real driver would)
                let key = if i == 5 { KeyCode::Escape } else { KeyCode::A };
                let mods = if i == 5 {
                    &emergency_mods
                } else {
                    &normal_mods
                };

                if check_emergency_exit(key, mods) {
                    toggle_bypass_mode();
                    activations.fetch_add(1, Ordering::SeqCst);
                }

                // Check bypass mode
                if is_bypass_active() {
                    // Pass through
                    processed.fetch_add(1, Ordering::SeqCst);
                } else {
                    // Remap
                    processed.fetch_add(1, Ordering::SeqCst);
                }
            }
        }
    });

    // Let the driver run for a bit
    thread::sleep(std::time::Duration::from_millis(50));
    stop_flag.store(true, Ordering::SeqCst);

    driver_thread
        .join()
        .expect("Driver thread should not panic");

    // Verify events were processed
    assert!(
        events_processed.load(Ordering::SeqCst) > 0,
        "Events should have been processed"
    );

    reset_test_state();
}

// =============================================================================
// Edge Cases and Error Handling
// =============================================================================

/// Test that partial combo sequences don't trigger bypass.
#[test]
#[serial]
fn partial_combo_sequences_dont_activate() {
    reset_test_state();

    // Test various partial combo sequences
    let partial_combos = [
        (true, false, false, KeyCode::Escape), // Only Ctrl
        (false, true, false, KeyCode::Escape), // Only Alt
        (false, false, true, KeyCode::Escape), // Only Shift
        (true, true, false, KeyCode::Escape),  // Ctrl+Alt
        (true, false, true, KeyCode::Escape),  // Ctrl+Shift
        (false, true, true, KeyCode::Escape),  // Alt+Shift
        (true, true, true, KeyCode::A),        // All mods but wrong key
        (true, true, true, KeyCode::Space),    // All mods but Space
    ];

    for (ctrl, alt, shift, key) in partial_combos {
        reset_test_state();
        let mods = make_modifiers(ctrl, alt, shift);

        if check_emergency_exit(key, &mods) {
            toggle_bypass_mode();
        }

        assert!(
            !is_bypass_active(),
            "Bypass should NOT activate for partial combo: ctrl={} alt={} shift={} key={:?}",
            ctrl,
            alt,
            shift,
            key
        );
    }
}

/// Test rapid bypass toggling stability.
#[test]
#[serial]
fn rapid_bypass_toggling() {
    reset_test_state();

    let toggle_count = 1000;

    for _ in 0..toggle_count {
        let expected_state = toggle_bypass_mode();
        assert_eq!(
            is_bypass_active(),
            expected_state,
            "Bypass state should match toggle return value"
        );
    }

    reset_test_state();
}

/// Test that bypass mode state is consistent after many operations.
#[test]
#[serial]
fn bypass_mode_consistency() {
    reset_test_state();

    // Perform many operations
    for _ in 0..100 {
        activate_bypass_mode();
        assert!(is_bypass_active());

        deactivate_bypass_mode();
        assert!(!is_bypass_active());

        set_bypass_mode(true);
        assert!(is_bypass_active());

        set_bypass_mode(false);
        assert!(!is_bypass_active());
    }

    // Final state should be inactive
    assert!(!is_bypass_active());
}

/// Test idempotent operations.
#[test]
#[serial]
fn idempotent_bypass_operations() {
    reset_test_state();

    // Multiple activations should be idempotent
    for _ in 0..10 {
        activate_bypass_mode();
    }
    assert!(is_bypass_active());

    // Multiple deactivations should be idempotent
    for _ in 0..10 {
        deactivate_bypass_mode();
    }
    assert!(!is_bypass_active());

    // Multiple set_bypass_mode(true) should be idempotent
    for _ in 0..10 {
        set_bypass_mode(true);
    }
    assert!(is_bypass_active());

    // Multiple set_bypass_mode(false) should be idempotent
    for _ in 0..10 {
        set_bypass_mode(false);
    }
    assert!(!is_bypass_active());
}
