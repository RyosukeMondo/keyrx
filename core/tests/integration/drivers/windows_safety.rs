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
//! Integration tests for Windows driver safety wrappers.
//!
//! Tests panic catching, thread-local state management, and hook safety mechanisms.

use crossbeam_channel::unbounded;
use keyrx_core::drivers::windows::safety::callback::{HookAction, HookCallback};
use keyrx_core::drivers::windows::safety::thread_local::ThreadLocalState;
use keyrx_core::engine::{InputEvent, KeyCode};
use windows::Win32::Foundation::{LPARAM, WPARAM};

#[test]
fn test_hook_callback_normal_execution() {
    let callback = HookCallback::new(|ncode, _wparam, _lparam| {
        if ncode < 0 {
            HookAction::PassThrough
        } else {
            HookAction::Suppress
        }
    });

    let result = callback.invoke(-1, WPARAM(0), LPARAM(0));
    assert_eq!(result.0, 0); // PassThrough

    let result = callback.invoke(0, WPARAM(0), LPARAM(0));
    assert_eq!(result.0, 1); // Suppress
}

#[test]
fn test_hook_callback_catches_panic() {
    let callback = HookCallback::new(|_ncode, _wparam, _lparam| {
        panic!("Test panic in callback");
    });

    // Should not panic - should catch and return PassThrough
    let result = callback.invoke(0, WPARAM(0), LPARAM(0));
    assert_eq!(result.0, 0); // PassThrough (default fallback)
}

#[test]
fn test_hook_callback_catches_panic_and_continues() {
    let callback = HookCallback::new(|ncode, _wparam, _lparam| {
        if ncode == 999 {
            panic!("Intentional panic");
        }
        HookAction::Suppress
    });

    // First call panics but is caught
    let result1 = callback.invoke(999, WPARAM(0), LPARAM(0));
    assert_eq!(result1.0, 0); // PassThrough (fallback)

    // Second call should work normally
    let result2 = callback.invoke(0, WPARAM(0), LPARAM(0));
    assert_eq!(result2.0, 1); // Suppress (normal behavior)
}

#[test]
fn test_hook_action_to_lresult_conversion() {
    assert_eq!(HookAction::PassThrough.to_lresult().0, 0);
    assert_eq!(HookAction::Suppress.to_lresult().0, 1);
}

#[test]
fn test_thread_local_state_sender_lifecycle() {
    // Create a test channel
    let (tx, _rx) = unbounded::<InputEvent>();

    // Initialize sender
    ThreadLocalState::init_sender(tx);

    // Verify we can access it
    let sender_exists = ThreadLocalState::is_sender_initialized();
    assert!(sender_exists);

    // Cleanup
    ThreadLocalState::cleanup();

    // Verify it's gone
    let sender_exists_after = ThreadLocalState::is_sender_initialized();
    assert!(!sender_exists_after);
}

#[test]
fn test_thread_local_state_sender_clone() {
    let (tx, _rx) = unbounded::<InputEvent>();
    ThreadLocalState::init_sender(tx);

    // Clone the sender
    let cloned = ThreadLocalState::with_sender(|sender| sender.clone());

    assert!(cloned.is_some());

    // Cleanup
    ThreadLocalState::cleanup();
}

#[test]
fn test_thread_local_state_key_states() {
    // Mark key as pressed
    ThreadLocalState::mark_key_pressed(65); // 'A' key
    assert!(ThreadLocalState::is_key_pressed(65));
    assert!(!ThreadLocalState::is_key_pressed(66)); // 'B' key

    // Mark another key
    ThreadLocalState::mark_key_pressed(66);
    assert!(ThreadLocalState::is_key_pressed(65));
    assert!(ThreadLocalState::is_key_pressed(66));

    // Mark key as released
    ThreadLocalState::mark_key_released(65);
    assert!(!ThreadLocalState::is_key_pressed(65));
    assert!(ThreadLocalState::is_key_pressed(66));

    // Clear all
    ThreadLocalState::cleanup();
    assert!(!ThreadLocalState::is_key_pressed(65));
    assert!(!ThreadLocalState::is_key_pressed(66));
}

#[test]
fn test_thread_local_state_key_repeat_detection() {
    // First press - not a repeat
    assert!(!ThreadLocalState::is_key_pressed(65));
    ThreadLocalState::mark_key_pressed(65);

    // Key is now pressed
    assert!(ThreadLocalState::is_key_pressed(65));

    // If we get another press event, it's a repeat
    // (the actual repeat detection happens in the driver code)

    // Release and press again
    ThreadLocalState::mark_key_released(65);
    assert!(!ThreadLocalState::is_key_pressed(65));

    ThreadLocalState::mark_key_pressed(65);
    assert!(ThreadLocalState::is_key_pressed(65));

    // Cleanup
    ThreadLocalState::cleanup();
}

#[test]
fn test_thread_local_state_isolation() {
    // This test verifies that thread-local state is actually thread-local
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    let success = Arc::new(AtomicBool::new(true));
    let success_clone = success.clone();

    // Set state in main thread
    ThreadLocalState::mark_key_pressed(65);
    assert!(ThreadLocalState::is_key_pressed(65));

    // Spawn a new thread
    let handle = std::thread::spawn(move || {
        // In new thread, state should be independent
        if ThreadLocalState::is_key_pressed(65) {
            success_clone.store(false, Ordering::SeqCst);
        }

        // Mark different key in this thread
        ThreadLocalState::mark_key_pressed(66);

        if !ThreadLocalState::is_key_pressed(66) {
            success_clone.store(false, Ordering::SeqCst);
        }
    });

    handle.join().unwrap();

    // Verify main thread state unchanged
    assert!(ThreadLocalState::is_key_pressed(65));
    assert!(!ThreadLocalState::is_key_pressed(66)); // Key 66 was only set in other thread

    // Verify isolation worked
    assert!(success.load(Ordering::SeqCst));

    // Cleanup
    ThreadLocalState::cleanup();
}

#[test]
fn test_thread_local_state_multiple_init() {
    // Initialize sender
    let (tx1, _rx1) = unbounded::<InputEvent>();
    ThreadLocalState::init_sender(tx1);

    // Initialize again with different sender - should replace
    let (tx2, rx2) = unbounded::<InputEvent>();
    ThreadLocalState::init_sender(tx2);

    // Send through the sender
    ThreadLocalState::with_sender(|sender| {
        // Should be able to send
        let _ = sender.send(InputEvent {
            key: KeyCode::A,
            pressed: true,
            ..Default::default()
        });
    });

    // Verify we can receive on the second receiver
    assert!(rx2.try_recv().is_ok());

    // Cleanup
    ThreadLocalState::cleanup();
}

#[test]
fn test_hook_callback_preserves_passthrough_semantic() {
    // Test that ncode < 0 always results in PassThrough
    let callback = HookCallback::new(|ncode, _wparam, _lparam| {
        if ncode < 0 {
            HookAction::PassThrough
        } else {
            HookAction::Suppress
        }
    });

    // Negative ncode should always pass through
    for ncode in -10..0 {
        let result = callback.invoke(ncode, WPARAM(0), LPARAM(0));
        assert_eq!(result.0, 0, "ncode {} should pass through", ncode);
    }

    // Non-negative ncode should suppress (according to this callback)
    for ncode in 0..10 {
        let result = callback.invoke(ncode, WPARAM(0), LPARAM(0));
        assert_eq!(result.0, 1, "ncode {} should suppress", ncode);
    }
}

#[test]
fn test_callback_with_complex_logic() {
    // Test a more complex callback that processes different event types
    let callback = HookCallback::new(|ncode, wparam, _lparam| {
        if ncode < 0 {
            return HookAction::PassThrough;
        }

        // Simulate different event types via wparam
        match wparam.0 as u32 {
            256 => HookAction::Suppress, // WM_KEYDOWN
            257 => HookAction::Suppress, // WM_KEYUP
            _ => HookAction::PassThrough,
        }
    });

    // Test different scenarios
    assert_eq!(callback.invoke(0, WPARAM(256), LPARAM(0)).0, 1); // Suppress
    assert_eq!(callback.invoke(0, WPARAM(257), LPARAM(0)).0, 1); // Suppress
    assert_eq!(callback.invoke(0, WPARAM(999), LPARAM(0)).0, 0); // PassThrough
    assert_eq!(callback.invoke(-1, WPARAM(256), LPARAM(0)).0, 0); // PassThrough (ncode < 0)
}

#[test]
fn test_thread_local_state_concurrent_keys() {
    // Test pressing multiple keys simultaneously
    ThreadLocalState::cleanup();

    // Press several keys
    for key in 65..75 {
        ThreadLocalState::mark_key_pressed(key);
    }

    // Verify all are pressed
    for key in 65..75 {
        assert!(
            ThreadLocalState::is_key_pressed(key),
            "Key {} should be pressed",
            key
        );
    }

    // Release some keys
    for key in 65..70 {
        ThreadLocalState::mark_key_released(key);
    }

    // Verify correct state
    for key in 65..70 {
        assert!(
            !ThreadLocalState::is_key_pressed(key),
            "Key {} should be released",
            key
        );
    }
    for key in 70..75 {
        assert!(
            ThreadLocalState::is_key_pressed(key),
            "Key {} should still be pressed",
            key
        );
    }

    // Cleanup
    ThreadLocalState::cleanup();
}

#[test]
fn test_cleanup_is_idempotent() {
    // Setup state
    let (tx, _rx) = unbounded::<InputEvent>();
    ThreadLocalState::init_sender(tx);
    ThreadLocalState::mark_key_pressed(65);

    // Multiple cleanups should be safe
    ThreadLocalState::cleanup();
    ThreadLocalState::cleanup();
    ThreadLocalState::cleanup();

    // State should still be clean
    assert!(!ThreadLocalState::is_sender_initialized());
    assert!(!ThreadLocalState::is_key_pressed(65));
}
