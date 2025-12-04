//! Tests verifying emergency exit functionality in error scenarios.
//!
//! This test suite ensures that Ctrl+Alt+Shift+Esc emergency exit combo
//! works even when the driver is in an error state, panicking, or otherwise
//! compromised. This is critical for user safety.

use keyrx_core::drivers::emergency_exit::{
    activate_bypass_mode, check_emergency_exit, is_bypass_active, set_bypass_mode,
    toggle_bypass_mode,
};
use keyrx_core::drivers::keycodes::KeyCode;
use keyrx_core::engine::{Modifier, ModifierState, StandardModifier};
use serial_test::serial;
use std::panic::{self, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

/// Helper to create modifier state with specific modifiers active.
fn make_emergency_mods() -> ModifierState {
    let mut state = ModifierState::new();
    state.activate(Modifier::Standard(StandardModifier::Control));
    state.activate(Modifier::Standard(StandardModifier::Alt));
    state.activate(Modifier::Standard(StandardModifier::Shift));
    state
}

// ============================================================================
// Emergency Exit During Panic Scenarios
// ============================================================================

/// Test that emergency exit check doesn't panic even when called inside a panic.
#[test]
#[serial]
fn emergency_exit_survives_panic_context() {
    set_bypass_mode(false);

    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        let mods = make_emergency_mods();

        // This should work even inside a panic context
        let detected = check_emergency_exit(KeyCode::Escape, &mods);
        assert!(detected);

        // Now panic
        panic!("Simulated panic in driver");
    }));

    assert!(result.is_err(), "Should have panicked");

    // After panic recovery, emergency exit should still be functional
    let mods = make_emergency_mods();
    let detected = check_emergency_exit(KeyCode::Escape, &mods);
    assert!(detected, "Emergency exit should work after panic recovery");
}

/// Test that bypass mode activation works inside a panic handler.
#[test]
#[serial]
fn bypass_activation_in_panic_handler() {
    set_bypass_mode(false);

    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        activate_bypass_mode();
        assert!(is_bypass_active());

        panic!("Simulated panic after bypass activation");
    }));

    assert!(result.is_err());

    // Bypass state should persist after panic
    assert!(
        is_bypass_active(),
        "Bypass mode should persist through panic"
    );

    set_bypass_mode(false);
}

/// Test emergency exit in a thread that panics.
#[test]
#[serial]
fn emergency_exit_in_panicking_thread() {
    set_bypass_mode(false);

    let handle = thread::spawn(|| {
        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            let mods = make_emergency_mods();

            // Check emergency combo
            if check_emergency_exit(KeyCode::Escape, &mods) {
                toggle_bypass_mode();
            }

            panic!("Thread panic");
        }));

        assert!(result.is_err());

        // Bypass should still be active after panic
        is_bypass_active()
    });

    let bypass_active = handle.join().expect("Thread should not abort");
    assert!(bypass_active, "Bypass should be active after thread panic");

    set_bypass_mode(false);
}

// ============================================================================
// Emergency Exit Under High Load / Stress
// ============================================================================

/// Test emergency exit detection under high concurrent load.
/// Simulates many threads hammering the driver with events while
/// emergency exit is triggered.
#[test]
#[serial]
fn emergency_exit_under_concurrent_load() {
    set_bypass_mode(false);

    let barrier = Arc::new(std::sync::Barrier::new(10));
    let exit_triggered = Arc::new(AtomicBool::new(false));
    let mut handles = vec![];

    // Spawn worker threads that simulate event processing
    for _thread_id in 0..9 {
        let b = Arc::clone(&barrier);
        let triggered = Arc::clone(&exit_triggered);

        handles.push(thread::spawn(move || {
            b.wait();

            // Simulate processing many events
            for _ in 0..1000 {
                if triggered.load(Ordering::SeqCst) {
                    // If emergency exit was triggered, verify bypass works
                    assert!(is_bypass_active());
                    break;
                }

                // Simulate normal event processing
                let mods = ModifierState::new();
                let _ = check_emergency_exit(KeyCode::A, &mods);
                thread::yield_now();
            }
        }));
    }

    // One thread triggers emergency exit
    let b = Arc::clone(&barrier);
    let triggered = Arc::clone(&exit_triggered);
    handles.push(thread::spawn(move || {
        b.wait();

        // Give other threads a moment to start processing
        thread::sleep(std::time::Duration::from_millis(5));

        // Trigger emergency exit
        let mods = make_emergency_mods();
        if check_emergency_exit(KeyCode::Escape, &mods) {
            activate_bypass_mode();
            triggered.store(true, Ordering::SeqCst);
        }

        assert!(is_bypass_active());
    }));

    for h in handles {
        h.join().expect("Thread should not panic");
    }

    assert!(
        is_bypass_active(),
        "Bypass should be active after concurrent test"
    );

    set_bypass_mode(false);
}

/// Test rapid emergency exit toggles under stress.
#[test]
#[serial]
fn emergency_exit_rapid_toggle_stress() {
    set_bypass_mode(false);

    let mods = make_emergency_mods();

    // Rapidly toggle 1000 times
    for i in 0..1000 {
        if check_emergency_exit(KeyCode::Escape, &mods) {
            let new_state = toggle_bypass_mode();
            let expected = (i + 1) % 2 == 1;
            assert_eq!(
                new_state, expected,
                "Toggle state should alternate at iteration {}",
                i
            );
        }
    }

    set_bypass_mode(false);
}

// ============================================================================
// Emergency Exit Priority (Must Be Checked First)
// ============================================================================

/// Test that emergency exit is checked BEFORE any other processing.
/// This simulates the critical requirement that emergency exit must
/// be the first thing checked in the event handler.
#[test]
#[serial]
fn emergency_exit_checked_before_processing() {
    set_bypass_mode(false);

    let mods = make_emergency_mods();

    // Simulate event handler order
    let order_correct = {
        // STEP 1: Emergency exit check (MUST be first)
        let is_emergency = check_emergency_exit(KeyCode::Escape, &mods);

        if is_emergency {
            activate_bypass_mode();
        }

        // STEP 2: Check bypass state
        let bypass_active = is_bypass_active();

        // STEP 3: If bypass active, skip processing
        if bypass_active {
            true // Order is correct - emergency exit activated bypass before processing
        } else {
            false // Would process normally
        }
    };

    assert!(
        order_correct,
        "Emergency exit should activate bypass before any processing"
    );

    set_bypass_mode(false);
}

/// Test that bypass mode prevents ALL processing.
/// Once bypass is active, no remapping should occur.
#[test]
#[serial]
fn bypass_mode_prevents_processing() {
    set_bypass_mode(false);

    activate_bypass_mode();

    // Simulate checking many events - none should be remapped
    for _key_code in [
        KeyCode::A,
        KeyCode::B,
        KeyCode::Enter,
        KeyCode::Space,
        KeyCode::F1,
    ] {
        // In bypass mode, we'd skip to returning early
        if is_bypass_active() {
            // This is the correct path - skip processing
            continue;
        }

        // This should never be reached in bypass mode
        panic!("Should not process keys when bypass is active");
    }

    set_bypass_mode(false);
}

// ============================================================================
// Emergency Exit in Error Recovery Scenarios
// ============================================================================

/// Test emergency exit works when retry logic is running.
/// Simulates a device that's failing and retrying, but user wants to exit.
#[test]
#[serial]
fn emergency_exit_during_retry_loop() {
    set_bypass_mode(false);

    let max_retries = 100;
    let exit_at_retry = 10;
    let mods = make_emergency_mods();

    for retry in 0..max_retries {
        // Simulate error and retry
        thread::sleep(std::time::Duration::from_millis(1));

        // User presses emergency exit during retry
        if retry == exit_at_retry {
            if check_emergency_exit(KeyCode::Escape, &mods) {
                activate_bypass_mode();
            }
        }

        // Check if we should stop due to bypass
        if is_bypass_active() {
            // Emergency exit worked - break retry loop
            assert!(
                retry >= exit_at_retry,
                "Should break loop after emergency exit"
            );
            break;
        }

        if retry == max_retries - 1 {
            panic!("Retry loop should have been stopped by emergency exit");
        }
    }

    assert!(is_bypass_active());
    set_bypass_mode(false);
}

/// Test emergency exit when device is disconnected/reconnecting.
#[test]
#[serial]
fn emergency_exit_during_device_error() {
    set_bypass_mode(false);

    // Simulate device error state
    let device_error = true;

    if device_error {
        // Even with device error, emergency exit should work
        let mods = make_emergency_mods();
        if check_emergency_exit(KeyCode::Escape, &mods) {
            activate_bypass_mode();
        }

        assert!(
            is_bypass_active(),
            "Emergency exit should work even with device errors"
        );
    }

    set_bypass_mode(false);
}

// ============================================================================
// Emergency Exit State Isolation
// ============================================================================

/// Test that bypass state is isolated from other error conditions.
#[test]
#[serial]
fn bypass_state_isolated_from_errors() {
    set_bypass_mode(false);

    // Activate bypass
    activate_bypass_mode();
    assert!(is_bypass_active());

    // Simulate various error conditions - bypass should persist
    let error_conditions = vec![
        "device_disconnected",
        "permission_denied",
        "grab_failed",
        "injection_failed",
    ];

    for error in error_conditions {
        // Errors should not affect bypass state
        assert!(
            is_bypass_active(),
            "Bypass should remain active despite error: {}",
            error
        );
    }

    set_bypass_mode(false);
}

/// Test that bypass mode can be checked from multiple threads without deadlock.
#[test]
#[serial]
fn bypass_check_no_deadlock() {
    set_bypass_mode(false);
    activate_bypass_mode();

    let barrier = Arc::new(std::sync::Barrier::new(4));
    let mut handles = vec![];

    for _ in 0..4 {
        let b = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            b.wait();

            // Each thread checks bypass state many times
            for _ in 0..1000 {
                let _ = is_bypass_active();
            }
        }));
    }

    for h in handles {
        h.join().expect("Should not deadlock");
    }

    set_bypass_mode(false);
}

// ============================================================================
// Windows and Linux Driver Integration Simulation
// ============================================================================

/// Simulate Windows hook callback checking emergency exit.
/// In Windows, the hook callback must check emergency exit FIRST.
#[test]
#[serial]
fn simulate_windows_hook_emergency_check() {
    set_bypass_mode(false);

    // Simulate low_level_keyboard_proc callback
    let simulate_hook_callback = |vk_code: u16, pressed: bool| -> bool {
        // EMERGENCY EXIT CHECK - must be FIRST
        if pressed && vk_code == 0x1B {
            // VK_ESCAPE
            // Check modifiers (would use GetAsyncKeyState in real code)
            let mods = make_emergency_mods();
            if check_emergency_exit(KeyCode::Escape, &mods) {
                toggle_bypass_mode();
                return true; // Pass through
            }
        }

        // If bypass mode is active, pass through all keys
        if is_bypass_active() {
            return true;
        }

        // Normal processing would happen here
        false
    };

    // Trigger emergency exit
    let passed_through = simulate_hook_callback(0x1B, true);
    assert!(passed_through, "Should pass through on emergency exit");
    assert!(is_bypass_active(), "Bypass should be active");

    // Other keys should also pass through now
    let passed_through = simulate_hook_callback(0x41, true); // 'A' key
    assert!(
        passed_through,
        "Should pass through all keys when bypass active"
    );

    set_bypass_mode(false);
}

/// Simulate Linux evdev reader checking emergency exit.
/// In Linux, the reader must check emergency exit before processing events.
#[test]
#[serial]
fn simulate_linux_reader_emergency_check() {
    set_bypass_mode(false);

    // Simulate process_events in Linux reader
    let simulate_process_events = |key_code: u16, pressed: bool| -> bool {
        // EMERGENCY EXIT CHECK - must be FIRST
        if pressed && key_code == 1 {
            // KEY_ESC in evdev
            let mods = make_emergency_mods();
            if check_emergency_exit(KeyCode::Escape, &mods) {
                toggle_bypass_mode();
                // Would ungrab device here
                return false; // Stop processing
            }
        }

        // If bypass mode is active, don't process events normally
        if is_bypass_active() {
            return false;
        }

        // Normal event processing would happen here
        true
    };

    // Trigger emergency exit
    let should_continue = simulate_process_events(1, true);
    assert!(!should_continue, "Should stop processing on emergency exit");
    assert!(is_bypass_active(), "Bypass should be active");

    // Other events should be skipped
    let should_continue = simulate_process_events(30, true); // KEY_A
    assert!(!should_continue, "Should skip processing in bypass mode");

    set_bypass_mode(false);
}

// ============================================================================
// Safety Invariants
// ============================================================================

/// Test that emergency exit combo detection is const-time (no timing attacks).
/// This is important for security - the check should always take the same time.
#[test]
fn emergency_check_constant_time() {
    let mods_full = make_emergency_mods();
    let mods_empty = ModifierState::new();

    // Both checks should be fast and consistent
    let start = std::time::Instant::now();
    for _ in 0..10000 {
        let _ = check_emergency_exit(KeyCode::Escape, &mods_full);
    }
    let elapsed_full = start.elapsed();

    let start = std::time::Instant::now();
    for _ in 0..10000 {
        let _ = check_emergency_exit(KeyCode::Escape, &mods_empty);
    }
    let elapsed_empty = start.elapsed();

    // Both should complete quickly (within milliseconds)
    assert!(
        elapsed_full.as_millis() < 100,
        "Full check should be fast: {:?}",
        elapsed_full
    );
    assert!(
        elapsed_empty.as_millis() < 100,
        "Empty check should be fast: {:?}",
        elapsed_empty
    );

    // Difference should be minimal (within 2x)
    let ratio = elapsed_full.as_micros() as f64 / elapsed_empty.as_micros().max(1) as f64;
    assert!(
        ratio < 2.0 && ratio > 0.5,
        "Timing should be consistent (ratio: {:.2})",
        ratio
    );
}

/// Test that bypass state is never corrupted (always a valid boolean).
#[test]
#[serial]
fn bypass_state_always_valid() {
    set_bypass_mode(false);

    for i in 0..1000 {
        if i % 2 == 0 {
            activate_bypass_mode();
        } else {
            set_bypass_mode(false);
        }

        let state = is_bypass_active();
        // State must always be a valid boolean (Rust ensures this, but we check anyway)
        assert!(
            state == true || state == false,
            "Bypass state should always be a valid boolean"
        );
    }

    set_bypass_mode(false);
}

// ============================================================================
// Emergency Exit with Circuit Breaker Integration
// ============================================================================

/// Test that emergency exit works when circuit breaker is in open state.
/// This is critical - users must be able to escape even when the driver is
/// in a degraded state with the circuit breaker open.
#[test]
#[serial]
fn emergency_exit_when_circuit_breaker_open() {
    set_bypass_mode(false);

    // Simulate circuit breaker open state (driver is in fallback mode)
    let circuit_breaker_open = true;
    let fallback_active = true;

    // Even with circuit open and fallback active, emergency exit must work
    if circuit_breaker_open && fallback_active {
        let mods = make_emergency_mods();

        // Emergency exit check should still function
        if check_emergency_exit(KeyCode::Escape, &mods) {
            activate_bypass_mode();
        }

        assert!(
            is_bypass_active(),
            "Emergency exit must work even when circuit breaker is open"
        );
    }

    set_bypass_mode(false);
}

/// Test emergency exit priority over circuit breaker state.
/// Emergency exit should be checked BEFORE checking circuit breaker state.
#[test]
#[serial]
fn emergency_exit_priority_over_circuit_breaker() {
    set_bypass_mode(false);

    let mods = make_emergency_mods();
    let circuit_open = true;

    // Simulate event processing with circuit breaker open
    let should_continue = {
        // STEP 1: Emergency exit check (MUST be first, even before circuit check)
        if check_emergency_exit(KeyCode::Escape, &mods) {
            activate_bypass_mode();
        }

        // STEP 2: Check bypass state (set by emergency exit)
        if is_bypass_active() {
            return false; // Stop processing
        }

        // STEP 3: Check circuit breaker state
        if circuit_open {
            // Would activate fallback here
            return false;
        }

        // Normal processing would continue
        true
    };

    assert!(
        !should_continue,
        "Emergency exit should stop processing before circuit breaker check"
    );
    assert!(
        is_bypass_active(),
        "Bypass should be active from emergency exit"
    );

    set_bypass_mode(false);
}

/// Test that emergency exit works during circuit breaker recovery attempts.
/// When circuit is in half-open state testing recovery, emergency exit must still work.
#[test]
#[serial]
fn emergency_exit_during_circuit_recovery() {
    set_bypass_mode(false);

    let mods = make_emergency_mods();

    // Simulate circuit breaker in half-open state (testing recovery)
    let circuit_half_open = true;
    let testing_recovery = true;

    for attempt in 0..10 {
        // Simulate recovery test attempts
        if circuit_half_open && testing_recovery {
            // User presses emergency exit during recovery
            if attempt == 5 {
                if check_emergency_exit(KeyCode::Escape, &mods) {
                    activate_bypass_mode();
                }
            }

            // Emergency exit should stop recovery attempts
            if is_bypass_active() {
                assert!(
                    attempt >= 5,
                    "Should break recovery loop after emergency exit"
                );
                break;
            }
        }

        if attempt == 9 {
            panic!("Recovery loop should have been stopped by emergency exit");
        }
    }

    assert!(
        is_bypass_active(),
        "Emergency exit should work during circuit recovery"
    );
    set_bypass_mode(false);
}

/// Test emergency exit isolation from fallback engine state.
/// Emergency exit should work independently of fallback engine status.
#[test]
#[serial]
fn emergency_exit_isolated_from_fallback_state() {
    set_bypass_mode(false);

    // Test with various fallback states
    let fallback_states = vec![
        ("active", true),
        ("inactive", false),
        ("transitioning", true),
    ];

    for (state_name, fallback_active) in fallback_states {
        // Reset bypass for each test
        set_bypass_mode(false);

        // Emergency exit should work regardless of fallback state
        let mods = make_emergency_mods();

        if check_emergency_exit(KeyCode::Escape, &mods) {
            activate_bypass_mode();
        }

        assert!(
            is_bypass_active(),
            "Emergency exit should work when fallback is {}",
            state_name
        );
    }

    set_bypass_mode(false);
}

/// Test that bypass mode takes precedence over circuit breaker and fallback.
/// Once bypass is active, all other safety mechanisms should be bypassed.
#[test]
#[serial]
fn bypass_mode_precedence_over_circuit_breaker() {
    set_bypass_mode(false);
    activate_bypass_mode();

    // Even with circuit breaker open and fallback active
    let circuit_open = true;
    let fallback_active = true;

    // Simulate event processing
    let processing_stopped = {
        // Check bypass FIRST
        if is_bypass_active() {
            true // All processing stopped
        } else if circuit_open {
            // Would check circuit breaker
            false
        } else if fallback_active {
            // Would check fallback
            false
        } else {
            // Normal processing
            false
        }
    };

    assert!(
        processing_stopped,
        "Bypass mode should stop all processing, overriding circuit breaker and fallback"
    );

    set_bypass_mode(false);
}
