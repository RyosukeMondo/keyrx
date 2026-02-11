//! Tests for TapHoldProcessor basic functionality.

use super::*;

#[test]
fn test_processor_new() {
    let processor: TapHoldProcessor<8> = TapHoldProcessor::new();
    assert_eq!(processor.pending_count(), 0);
    assert_eq!(processor.hold_count(), 0);
}

#[test]
fn test_processor_default() {
    let processor: TapHoldProcessor<8> = TapHoldProcessor::default();
    assert_eq!(processor.pending_count(), 0);
}

#[test]
fn test_processor_register_tap_hold() {
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    assert!(processor.register_tap_hold(KeyCode::CapsLock, config));
    assert!(processor.is_tap_hold_key(KeyCode::CapsLock));
    assert!(!processor.is_tap_hold_key(KeyCode::Tab));
}

#[test]
fn test_processor_register_duplicate_fails() {
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config1 = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    let config2 = TapHoldConfig::from_ms(KeyCode::Tab, 1, 300);

    assert!(processor.register_tap_hold(KeyCode::CapsLock, config1));
    assert!(!processor.register_tap_hold(KeyCode::CapsLock, config2)); // Duplicate
}

#[test]
fn test_processor_register_at_capacity() {
    let mut processor: TapHoldProcessor<2> = TapHoldProcessor::new();

    let config1 = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    let config2 = TapHoldConfig::from_ms(KeyCode::Tab, 1, 200);
    let config3 = TapHoldConfig::from_ms(KeyCode::Space, 2, 200);

    assert!(processor.register_tap_hold(KeyCode::CapsLock, config1));
    assert!(processor.register_tap_hold(KeyCode::Tab, config2));
    assert!(!processor.register_tap_hold(KeyCode::Space, config3)); // At capacity
}

#[test]
fn test_processor_get_config() {
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 5, 250);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    let retrieved = processor.get_config(KeyCode::CapsLock);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().tap_key(), KeyCode::Escape);
    assert_eq!(retrieved.unwrap().hold_modifier(), 5);
    assert_eq!(retrieved.unwrap().threshold_us(), 250_000);

    assert!(processor.get_config(KeyCode::Tab).is_none());
}

#[test]
fn test_processor_press_non_tap_hold_key() {
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    // Press a key that's not registered as tap-hold
    let outputs = processor.process_press(KeyCode::A, 0);
    assert!(outputs.is_empty());
    assert_eq!(processor.pending_count(), 0);
}

#[test]
fn test_processor_press_tap_hold_key() {
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    // Press the tap-hold key
    let outputs = processor.process_press(KeyCode::CapsLock, 0);
    assert!(outputs.is_empty()); // No immediate output
    assert!(processor.is_pending(KeyCode::CapsLock));
    assert_eq!(processor.pending_count(), 1);
}

#[test]
fn test_processor_tap_quick_release() {
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    // Press at t=0
    let _ = processor.process_press(KeyCode::CapsLock, 0);
    assert!(processor.is_pending(KeyCode::CapsLock));

    // Quick release at t=100ms (under 200ms threshold)
    let outputs = processor.process_release(KeyCode::CapsLock, 100_000);

    // Should emit tap key press + release
    assert_eq!(outputs.len(), 2);

    match outputs[0] {
        TapHoldOutput::KeyEvent {
            key,
            is_press,
            timestamp_us,
        } => {
            assert_eq!(key, KeyCode::Escape);
            assert!(is_press);
            assert_eq!(timestamp_us, 100_000);
        }
        _ => panic!("Expected key press"),
    }

    match outputs[1] {
        TapHoldOutput::KeyEvent {
            key,
            is_press,
            timestamp_us,
        } => {
            assert_eq!(key, KeyCode::Escape);
            assert!(!is_press);
            assert_eq!(timestamp_us, 100_000);
        }
        _ => panic!("Expected key release"),
    }

    // Should no longer be pending
    assert!(!processor.is_pending(KeyCode::CapsLock));
    assert_eq!(processor.pending_count(), 0);
}

#[test]
fn test_processor_tap_at_threshold_boundary() {
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    // Press at t=0
    let _ = processor.process_press(KeyCode::CapsLock, 0);

    // Release at t=199,999μs (just under 200ms threshold)
    let outputs = processor.process_release(KeyCode::CapsLock, 199_999);

    // Should still be a tap
    assert_eq!(outputs.len(), 2);
    match outputs[0] {
        TapHoldOutput::KeyEvent { key, is_press, .. } => {
            assert_eq!(key, KeyCode::Escape);
            assert!(is_press);
        }
        _ => panic!("Expected key press"),
    }
}

#[test]
fn test_processor_hold_via_timeout() {
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    // Press at t=0
    let _ = processor.process_press(KeyCode::CapsLock, 0);
    assert!(processor.is_pending(KeyCode::CapsLock));

    // Check timeouts at t=100ms - should not trigger
    let outputs = processor.check_timeouts(100_000);
    assert!(outputs.is_empty());
    assert!(processor.is_pending(KeyCode::CapsLock));

    // Check timeouts at t=250ms - should trigger hold
    let outputs = processor.check_timeouts(250_000);
    assert_eq!(outputs.len(), 1);
    match outputs[0] {
        TapHoldOutput::ActivateModifier { modifier_id } => {
            assert_eq!(modifier_id, 0);
        }
        _ => panic!("Expected activate modifier"),
    }

    // Should now be in hold state
    assert!(processor.is_hold(KeyCode::CapsLock));
    assert!(!processor.is_pending(KeyCode::CapsLock));
}

#[test]
fn test_processor_hold_release() {
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    // Press at t=0
    let _ = processor.process_press(KeyCode::CapsLock, 0);

    // Timeout to enter hold state
    let _ = processor.check_timeouts(250_000);
    assert!(processor.is_hold(KeyCode::CapsLock));

    // Release from hold state
    let outputs = processor.process_release(KeyCode::CapsLock, 300_000);
    assert_eq!(outputs.len(), 1);
    match outputs[0] {
        TapHoldOutput::DeactivateModifier { modifier_id } => {
            assert_eq!(modifier_id, 0);
        }
        _ => panic!("Expected deactivate modifier"),
    }

    // Should be gone from processor
    assert!(!processor.is_pending(KeyCode::CapsLock));
    assert!(!processor.is_hold(KeyCode::CapsLock));
    assert_eq!(processor.pending_count(), 0);
    assert_eq!(processor.hold_count(), 0);
}

#[test]
fn test_processor_delayed_hold_release() {
    // Test case where key is released after threshold but timeout wasn't checked
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    // Press at t=0
    let _ = processor.process_press(KeyCode::CapsLock, 0);

    // Release at t=300ms (after threshold) WITHOUT calling check_timeouts
    // This simulates a case where timeout checking was delayed
    let outputs = processor.process_release(KeyCode::CapsLock, 300_000);

    // Should activate then immediately deactivate (delayed hold)
    assert_eq!(outputs.len(), 2);
    match outputs[0] {
        TapHoldOutput::ActivateModifier { modifier_id } => {
            assert_eq!(modifier_id, 0);
        }
        _ => panic!("Expected activate modifier"),
    }
    match outputs[1] {
        TapHoldOutput::DeactivateModifier { modifier_id } => {
            assert_eq!(modifier_id, 0);
        }
        _ => panic!("Expected deactivate modifier"),
    }
}

#[test]
fn test_processor_multiple_concurrent_tap_holds() {
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    // Register two tap-hold keys
    let config_caps = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    let config_tab = TapHoldConfig::from_ms(KeyCode::Tab, 1, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config_caps);
    processor.register_tap_hold(KeyCode::Tab, config_tab);

    // Press CapsLock at t=0
    let _ = processor.process_press(KeyCode::CapsLock, 0);
    assert_eq!(processor.pending_count(), 1);

    // Press Tab at t=50ms
    let _ = processor.process_press(KeyCode::Tab, 50_000);
    assert_eq!(processor.pending_count(), 2);

    // Quick release Tab at t=100ms (tap)
    let outputs = processor.process_release(KeyCode::Tab, 100_000);
    assert_eq!(outputs.len(), 2);
    match outputs[0] {
        TapHoldOutput::KeyEvent { key, is_press, .. } => {
            assert_eq!(key, KeyCode::Tab);
            assert!(is_press);
        }
        _ => panic!("Expected tab press"),
    }
    assert_eq!(processor.pending_count(), 1);

    // CapsLock times out at t=250ms
    let outputs = processor.check_timeouts(250_000);
    assert_eq!(outputs.len(), 1);
    assert!(processor.is_hold(KeyCode::CapsLock));
    assert_eq!(processor.hold_count(), 1);
}

#[test]
fn test_processor_release_non_pending_key() {
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    // Release a key that was never pressed
    let outputs = processor.process_release(KeyCode::CapsLock, 0);
    assert!(outputs.is_empty());
}

#[test]
fn test_processor_double_press_ignored() {
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    // First press
    let _ = processor.process_press(KeyCode::CapsLock, 0);
    assert_eq!(processor.pending_count(), 1);

    // Second press (shouldn't change anything)
    let outputs = processor.process_press(KeyCode::CapsLock, 50_000);
    assert!(outputs.is_empty());
    assert_eq!(processor.pending_count(), 1);
}

#[test]
fn test_processor_clear() {
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    let _ = processor.process_press(KeyCode::CapsLock, 0);
    assert_eq!(processor.pending_count(), 1);

    processor.clear();
    assert_eq!(processor.pending_count(), 0);
    assert!(processor.is_tap_hold_key(KeyCode::CapsLock)); // Config preserved
}

#[test]
fn test_processor_reset() {
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    let _ = processor.process_press(KeyCode::CapsLock, 0);
    assert_eq!(processor.pending_count(), 1);
    assert!(processor.is_tap_hold_key(KeyCode::CapsLock));

    processor.reset();
    assert_eq!(processor.pending_count(), 0);
    assert!(!processor.is_tap_hold_key(KeyCode::CapsLock)); // Config cleared too
}

#[test]
fn test_processor_exact_threshold_is_hold() {
    // At exactly the threshold, it should be a hold, not a tap
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    // Press at t=0
    let _ = processor.process_press(KeyCode::CapsLock, 0);

    // Release at exactly 200ms (threshold)
    let outputs = processor.process_release(KeyCode::CapsLock, 200_000);

    // Should be treated as hold (activate then deactivate)
    assert_eq!(outputs.len(), 2);
    match outputs[0] {
        TapHoldOutput::ActivateModifier { modifier_id } => {
            assert_eq!(modifier_id, 0);
        }
        _ => panic!("Expected activate modifier at exact threshold"),
    }
}

#[test]
fn test_processor_realistic_ctrl_escape_scenario() {
    // CapsLock: tap = Escape, hold = Ctrl (modifier 0)
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    // Scenario 1: Quick tap for Escape
    let _ = processor.process_press(KeyCode::CapsLock, 0);
    let outputs = processor.process_release(KeyCode::CapsLock, 50_000); // 50ms

    assert_eq!(outputs.len(), 2);
    match &outputs[0] {
        TapHoldOutput::KeyEvent { key, is_press, .. } => {
            assert_eq!(*key, KeyCode::Escape);
            assert!(*is_press);
        }
        _ => panic!("Expected Escape press"),
    }
    match &outputs[1] {
        TapHoldOutput::KeyEvent { key, is_press, .. } => {
            assert_eq!(*key, KeyCode::Escape);
            assert!(!*is_press);
        }
        _ => panic!("Expected Escape release"),
    }

    // Scenario 2: Hold for Ctrl
    let _ = processor.process_press(KeyCode::CapsLock, 1_000_000); // t=1s
    let outputs = processor.check_timeouts(1_250_000); // 250ms later

    assert_eq!(outputs.len(), 1);
    match outputs[0] {
        TapHoldOutput::ActivateModifier { modifier_id } => {
            assert_eq!(modifier_id, 0); // Ctrl
        }
        _ => panic!("Expected Ctrl activation"),
    }

    // Release Ctrl
    let outputs = processor.process_release(KeyCode::CapsLock, 1_500_000);
    assert_eq!(outputs.len(), 1);
    match outputs[0] {
        TapHoldOutput::DeactivateModifier { modifier_id } => {
            assert_eq!(modifier_id, 0);
        }
        _ => panic!("Expected Ctrl deactivation"),
    }
}

// --- Permissive Hold Tests ---

#[test]
fn test_processor_permissive_hold_basic() {
    // Test: CapsLock (hold=Ctrl) + A pressed quickly should yield Ctrl+A
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    // Press CapsLock at t=0 (enters Pending state)
    let _ = processor.process_press(KeyCode::CapsLock, 0);
    assert!(processor.is_pending(KeyCode::CapsLock));
    assert!(processor.has_pending_keys());

    // User types 'A' at t=50ms (before 200ms threshold)
    // This should trigger permissive hold
    let outputs = processor.process_other_key_press(KeyCode::A);

    // CapsLock's modifier should be activated
    assert_eq!(outputs.len(), 1);
    match outputs[0] {
        TapHoldOutput::ActivateModifier { modifier_id } => {
            assert_eq!(modifier_id, 0);
        }
        _ => panic!("Expected modifier activation"),
    }

    // CapsLock should now be in Hold state
    assert!(processor.is_hold(KeyCode::CapsLock));
    assert!(!processor.is_pending(KeyCode::CapsLock));
}

#[test]
fn test_processor_permissive_hold_no_pending_keys() {
    // Test: No pending tap-hold keys - process_other_key_press returns empty
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    // No keys pressed - no pending keys
    assert!(!processor.has_pending_keys());

    // Process other key - should return empty
    let outputs = processor.process_other_key_press(KeyCode::A);
    assert!(outputs.is_empty());
}

#[test]
fn test_processor_permissive_hold_multiple_pending() {
    // Test: Multiple pending tap-hold keys should all transition to Hold
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config_caps = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    let config_tab = TapHoldConfig::from_ms(KeyCode::Tab, 1, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config_caps);
    processor.register_tap_hold(KeyCode::Tab, config_tab);

    // Press both tap-hold keys
    let _ = processor.process_press(KeyCode::CapsLock, 0);
    let _ = processor.process_press(KeyCode::Tab, 50_000);

    assert!(processor.is_pending(KeyCode::CapsLock));
    assert!(processor.is_pending(KeyCode::Tab));
    assert_eq!(processor.pending_count(), 2);

    // Press a regular key - both should transition to Hold
    let outputs = processor.process_other_key_press(KeyCode::A);

    // Both modifiers should be activated
    assert_eq!(outputs.len(), 2);

    // Verify both are now in Hold state
    assert!(processor.is_hold(KeyCode::CapsLock));
    assert!(processor.is_hold(KeyCode::Tab));
    assert_eq!(processor.hold_count(), 2);
}

#[test]
fn test_processor_permissive_hold_press_same_key() {
    // Test: Pressing the same pending key again should NOT trigger permissive hold
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    // Press CapsLock
    let _ = processor.process_press(KeyCode::CapsLock, 0);
    assert!(processor.is_pending(KeyCode::CapsLock));

    // "Press" CapsLock again via process_other_key_press
    // (This simulates edge case where key repeat might trigger this)
    let outputs = processor.process_other_key_press(KeyCode::CapsLock);

    // Should NOT trigger permissive hold
    assert!(outputs.is_empty());
    assert!(processor.is_pending(KeyCode::CapsLock)); // Still pending
}

#[test]
fn test_processor_permissive_hold_then_release() {
    // Test: Full flow - permissive hold then release
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    // Press CapsLock
    let _ = processor.process_press(KeyCode::CapsLock, 0);

    // Press 'A' - triggers permissive hold
    let outputs = processor.process_other_key_press(KeyCode::A);
    assert_eq!(outputs.len(), 1);
    assert!(processor.is_hold(KeyCode::CapsLock));

    // Release CapsLock
    let outputs = processor.process_release(KeyCode::CapsLock, 100_000);

    // Should deactivate the modifier
    assert_eq!(outputs.len(), 1);
    match outputs[0] {
        TapHoldOutput::DeactivateModifier { modifier_id } => {
            assert_eq!(modifier_id, 0);
        }
        _ => panic!("Expected deactivate modifier"),
    }

    // No longer tracking this key
    assert!(!processor.is_hold(KeyCode::CapsLock));
    assert!(!processor.is_pending(KeyCode::CapsLock));
}

#[test]
fn test_processor_permissive_hold_already_in_hold() {
    // Test: Key already in Hold state - process_other_key_press doesn't duplicate
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    // Press CapsLock
    let _ = processor.process_press(KeyCode::CapsLock, 0);

    // Let it timeout to Hold state
    let _ = processor.check_timeouts(300_000);
    assert!(processor.is_hold(KeyCode::CapsLock));

    // Press another key - already in Hold, so no new activations
    let outputs = processor.process_other_key_press(KeyCode::A);
    assert!(outputs.is_empty());

    // Still in Hold state
    assert!(processor.is_hold(KeyCode::CapsLock));
}

#[test]
fn test_processor_has_pending_keys() {
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    // Initially no pending keys
    assert!(!processor.has_pending_keys());

    // Press tap-hold key
    let _ = processor.process_press(KeyCode::CapsLock, 0);
    assert!(processor.has_pending_keys());

    // Release (tap)
    let _ = processor.process_release(KeyCode::CapsLock, 50_000);
    assert!(!processor.has_pending_keys());
}

#[test]
fn test_processor_permissive_hold_realistic_ctrl_a() {
    // Realistic scenario: User types Ctrl+A using CapsLock as Ctrl
    // CapsLock: tap = Escape, hold = Ctrl (modifier 0)
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    // t=0: User presses CapsLock (intending to use it as Ctrl)
    let _ = processor.process_press(KeyCode::CapsLock, 0);
    assert!(processor.is_pending(KeyCode::CapsLock));

    // t=30ms: User quickly presses 'A' (faster than 200ms threshold)
    // The caller should:
    // 1. Call process_other_key_press(A) BEFORE processing A
    let permissive_outputs = processor.process_other_key_press(KeyCode::A);

    // 2. Verify Ctrl was activated
    assert_eq!(permissive_outputs.len(), 1);
    match permissive_outputs[0] {
        TapHoldOutput::ActivateModifier { modifier_id } => {
            assert_eq!(modifier_id, 0); // Ctrl
        }
        _ => panic!("Expected Ctrl activation"),
    }

    // 3. CapsLock is now in Hold state (Ctrl active)
    assert!(processor.is_hold(KeyCode::CapsLock));

    // 4. Caller processes 'A' normally (which will now be Ctrl+A)

    // t=50ms: User releases 'A' (caller handles this normally)

    // t=100ms: User releases CapsLock
    let release_outputs = processor.process_release(KeyCode::CapsLock, 100_000);

    // Ctrl should be deactivated
    assert_eq!(release_outputs.len(), 1);
    match release_outputs[0] {
        TapHoldOutput::DeactivateModifier { modifier_id } => {
            assert_eq!(modifier_id, 0);
        }
        _ => panic!("Expected Ctrl deactivation"),
    }

    // Flow complete - system returned to normal state
    assert!(!processor.is_hold(KeyCode::CapsLock));
    assert_eq!(processor.hold_count(), 0);
}

#[test]
fn test_processor_permissive_hold_press_another_tap_hold() {
    // Test: Pressing another registered tap-hold key while one is pending
    // Should trigger permissive hold for the first, then put second in pending
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config_caps = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    let config_tab = TapHoldConfig::from_ms(KeyCode::Tab, 1, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config_caps);
    processor.register_tap_hold(KeyCode::Tab, config_tab);

    // Press CapsLock
    let _ = processor.process_press(KeyCode::CapsLock, 0);
    assert!(processor.is_pending(KeyCode::CapsLock));

    // Now press Tab (also a tap-hold key)
    // This should NOT trigger permissive hold via process_other_key_press
    // because we should use process_press for tap-hold keys
    // Let's verify process_other_key_press handles this edge case
    let outputs = processor.process_other_key_press(KeyCode::Tab);
    // Tab is not in pending yet (it would be added via process_press)
    // So CapsLock should transition to Hold
    assert_eq!(outputs.len(), 1);
    assert!(processor.is_hold(KeyCode::CapsLock));

    // Now properly press Tab
    let _ = processor.process_press(KeyCode::Tab, 50_000);
    assert!(processor.is_pending(KeyCode::Tab));
}

// =======================================================================
// Task 11: Comprehensive Permissive Hold Unit Tests
// =======================================================================
