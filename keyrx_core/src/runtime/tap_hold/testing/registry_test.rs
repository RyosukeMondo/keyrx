//! Tests for PendingKeyRegistry.

use super::*;

// --- PendingKeyRegistry Tests ---

/// Helper to create a pending state for testing
fn make_pending_state(key: KeyCode, tap: KeyCode, modifier: u8, press_time: u64) -> TapHoldState {
    let config = TapHoldConfig::from_ms(tap, modifier, 200);
    let mut state = TapHoldState::new(key, config);
    state.transition_to_pending(press_time);
    state
}

#[test]
fn test_registry_new() {
    let registry: PendingKeyRegistry<32> = PendingKeyRegistry::new();
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
    assert!(!registry.is_full());
    assert_eq!(registry.capacity(), 32);
}

#[test]
fn test_registry_default() {
    let registry: PendingKeyRegistry<8> = PendingKeyRegistry::default();
    assert!(registry.is_empty());
    assert_eq!(registry.capacity(), 8);
}

#[test]
fn test_registry_add_single() {
    let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();
    let state = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 1000);

    assert!(registry.add(state));
    assert_eq!(registry.len(), 1);
    assert!(!registry.is_empty());
    assert!(registry.contains(KeyCode::CapsLock));
}

#[test]
fn test_registry_add_multiple() {
    let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();

    let state1 = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 1000);
    let state2 = make_pending_state(KeyCode::Tab, KeyCode::Tab, 1, 2000);
    let state3 = make_pending_state(KeyCode::Space, KeyCode::Space, 2, 3000);

    assert!(registry.add(state1));
    assert!(registry.add(state2));
    assert!(registry.add(state3));

    assert_eq!(registry.len(), 3);
    assert!(registry.contains(KeyCode::CapsLock));
    assert!(registry.contains(KeyCode::Tab));
    assert!(registry.contains(KeyCode::Space));
}

#[test]
fn test_registry_add_duplicate_fails() {
    let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();

    let state1 = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 1000);
    let state2 = make_pending_state(KeyCode::CapsLock, KeyCode::Tab, 1, 2000);

    assert!(registry.add(state1));
    assert!(!registry.add(state2)); // Same key, should fail
    assert_eq!(registry.len(), 1);
}

#[test]
fn test_registry_add_when_full() {
    let mut registry: PendingKeyRegistry<2> = PendingKeyRegistry::new();

    let state1 = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 1000);
    let state2 = make_pending_state(KeyCode::Tab, KeyCode::Tab, 1, 2000);
    let state3 = make_pending_state(KeyCode::Space, KeyCode::Space, 2, 3000);

    assert!(registry.add(state1));
    assert!(registry.add(state2));
    assert!(registry.is_full());
    assert!(!registry.add(state3)); // Should fail, registry full
    assert_eq!(registry.len(), 2);
}

#[test]
fn test_registry_remove() {
    let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();

    let state1 = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 1000);
    let state2 = make_pending_state(KeyCode::Tab, KeyCode::Tab, 1, 2000);

    registry.add(state1);
    registry.add(state2);

    assert!(registry.remove(KeyCode::CapsLock));
    assert_eq!(registry.len(), 1);
    assert!(!registry.contains(KeyCode::CapsLock));
    assert!(registry.contains(KeyCode::Tab));
}

#[test]
fn test_registry_remove_nonexistent() {
    let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();
    let state = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 1000);

    registry.add(state);
    assert!(!registry.remove(KeyCode::Tab)); // Not in registry
    assert_eq!(registry.len(), 1);
}

#[test]
fn test_registry_get() {
    let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();
    let state = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 1000);
    registry.add(state);

    let retrieved = registry.get(KeyCode::CapsLock);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().key(), KeyCode::CapsLock);
    assert_eq!(retrieved.unwrap().tap_key(), KeyCode::Escape);

    assert!(registry.get(KeyCode::Tab).is_none());
}

#[test]
fn test_registry_get_mut() {
    let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();
    let state = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 1000);
    registry.add(state);

    // Modify the state
    if let Some(s) = registry.get_mut(KeyCode::CapsLock) {
        s.transition_to_hold();
    }

    let retrieved = registry.get(KeyCode::CapsLock);
    assert!(retrieved.unwrap().phase().is_hold());
}

#[test]
fn test_registry_iter() {
    let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();

    let state1 = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 1000);
    let state2 = make_pending_state(KeyCode::Tab, KeyCode::Tab, 1, 2000);

    registry.add(state1);
    registry.add(state2);

    let keys: Vec<_> = registry.iter().map(|s| s.key()).collect();
    assert_eq!(keys.len(), 2);
    assert!(keys.contains(&KeyCode::CapsLock));
    assert!(keys.contains(&KeyCode::Tab));
}

#[test]
fn test_registry_iter_mut() {
    let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();

    let state1 = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 1000);
    let state2 = make_pending_state(KeyCode::Tab, KeyCode::Tab, 1, 2000);

    registry.add(state1);
    registry.add(state2);

    // Transition all to hold
    for s in registry.iter_mut() {
        s.transition_to_hold();
    }

    // Verify all are in Hold state
    for s in registry.iter() {
        assert!(s.phase().is_hold());
    }
}

#[test]
fn test_registry_clear() {
    let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();

    let state1 = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 1000);
    let state2 = make_pending_state(KeyCode::Tab, KeyCode::Tab, 1, 2000);

    registry.add(state1);
    registry.add(state2);
    assert_eq!(registry.len(), 2);

    registry.clear();
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
}

#[test]
fn test_registry_check_timeouts_no_timeouts() {
    let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();

    // 200ms threshold, pressed at time 0
    let state = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 0);
    registry.add(state);

    // Check at 100ms - no timeout
    let results = registry.check_timeouts(100_000);
    assert!(results.is_empty());

    // State should still be Pending
    assert!(registry
        .get(KeyCode::CapsLock)
        .unwrap()
        .phase()
        .is_pending());
}

#[test]
fn test_registry_check_timeouts_single_timeout() {
    let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();

    // 200ms threshold, pressed at time 0
    let state = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 0);
    registry.add(state);

    // Check at 300ms - timeout!
    let results = registry.check_timeouts(300_000);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].key, KeyCode::CapsLock);
    assert_eq!(results[0].hold_modifier, 0);

    // State should now be Hold
    assert!(registry.get(KeyCode::CapsLock).unwrap().phase().is_hold());
}

#[test]
fn test_registry_check_timeouts_at_exact_threshold() {
    let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();

    // 200ms threshold, pressed at time 0
    let state = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 0);
    registry.add(state);

    // Check at exactly 200ms - should timeout
    let results = registry.check_timeouts(200_000);
    assert_eq!(results.len(), 1);
}

#[test]
fn test_registry_check_timeouts_multiple() {
    let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();

    // CapsLock: 200ms threshold, pressed at 0
    let state1 = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 0);
    // Tab: 200ms threshold, pressed at 100ms
    let state2 = make_pending_state(KeyCode::Tab, KeyCode::Tab, 1, 100_000);

    registry.add(state1);
    registry.add(state2);

    // At 250ms: CapsLock times out (250ms > 200ms), Tab doesn't (150ms < 200ms)
    let results = registry.check_timeouts(250_000);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].key, KeyCode::CapsLock);

    // At 350ms: Tab also times out (250ms > 200ms)
    let results = registry.check_timeouts(350_000);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].key, KeyCode::Tab);
}

#[test]
fn test_registry_check_timeouts_ignores_hold_state() {
    let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();

    let state = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 0);
    registry.add(state);

    // First timeout check
    let results = registry.check_timeouts(300_000);
    assert_eq!(results.len(), 1);

    // Second timeout check - should not report again since now in Hold state
    let results = registry.check_timeouts(400_000);
    assert!(results.is_empty());
}

#[test]
fn test_registry_pending_keys() {
    let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();

    let state1 = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 0);
    let state2 = make_pending_state(KeyCode::Tab, KeyCode::Tab, 1, 0);

    registry.add(state1);
    registry.add(state2);

    // Both pending initially
    assert_eq!(registry.pending_keys().count(), 2);

    // Transition one to Hold
    if let Some(s) = registry.get_mut(KeyCode::CapsLock) {
        s.transition_to_hold();
    }

    // Only one pending now
    assert_eq!(registry.pending_keys().count(), 1);
    let pending: Vec<_> = registry.pending_keys().collect();
    assert_eq!(pending[0].key(), KeyCode::Tab);
}

#[test]
fn test_registry_trigger_permissive_hold() {
    let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();

    let state1 = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 0);
    let state2 = make_pending_state(KeyCode::Tab, KeyCode::Tab, 1, 0);

    registry.add(state1);
    registry.add(state2);

    // Trigger permissive hold for all
    let results = registry.trigger_permissive_hold();

    // Both should be reported
    assert_eq!(results.len(), 2);

    // Both should now be in Hold state
    assert!(registry.get(KeyCode::CapsLock).unwrap().phase().is_hold());
    assert!(registry.get(KeyCode::Tab).unwrap().phase().is_hold());
}

#[test]
fn test_registry_trigger_permissive_hold_ignores_hold() {
    let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();

    let state1 = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 0);
    let state2 = make_pending_state(KeyCode::Tab, KeyCode::Tab, 1, 0);

    registry.add(state1);
    registry.add(state2);

    // Manually transition one to Hold
    registry
        .get_mut(KeyCode::CapsLock)
        .unwrap()
        .transition_to_hold();

    // Trigger permissive hold
    let results = registry.trigger_permissive_hold();

    // Only Tab should be reported (CapsLock was already Hold)
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].key, KeyCode::Tab);
}

#[test]
fn test_registry_concurrent_keys_scenario() {
    // Simulate realistic concurrent tap-hold usage:
    // User presses CapsLock(hold=Ctrl), then Tab(hold=Alt), then releases both
    let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();

    // CapsLock pressed at time 0
    let caps = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 0);
    registry.add(caps);
    assert_eq!(registry.len(), 1);

    // Tab pressed at time 100ms
    let tab = make_pending_state(KeyCode::Tab, KeyCode::Tab, 1, 100_000);
    registry.add(tab);
    assert_eq!(registry.len(), 2);

    // Check timeouts at 150ms - neither has timed out yet (200ms threshold)
    let results = registry.check_timeouts(150_000);
    assert!(results.is_empty());

    // Check timeouts at 250ms - CapsLock times out (250ms > 200ms)
    // Tab has only 150ms elapsed (250ms - 100ms), still pending
    let results = registry.check_timeouts(250_000);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].key, KeyCode::CapsLock);

    // Check timeouts at 350ms - Tab times out too (250ms elapsed > 200ms)
    let results = registry.check_timeouts(350_000);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].key, KeyCode::Tab);

    // Both in Hold state, user types 'A' - gets Ctrl+Alt+A
    // Then releases both
    registry.remove(KeyCode::CapsLock);
    registry.remove(KeyCode::Tab);
    assert!(registry.is_empty());
}

#[test]
fn test_registry_timeout_result_fields() {
    let result = TimeoutResult {
        key: KeyCode::CapsLock,
        hold_modifier: 5,
    };

    assert_eq!(result.key, KeyCode::CapsLock);
    assert_eq!(result.hold_modifier, 5);

    // Test Clone and Copy
    let result2 = result;
    assert_eq!(result, result2);
}

#[test]
fn test_registry_with_different_capacities() {
    // Small capacity
    let small: PendingKeyRegistry<2> = PendingKeyRegistry::new();
    assert_eq!(small.capacity(), 2);

    // Default capacity
    let default: PendingKeyRegistry<DEFAULT_MAX_PENDING> = PendingKeyRegistry::new();
    assert_eq!(default.capacity(), 32);

    // Large capacity
    let large: PendingKeyRegistry<64> = PendingKeyRegistry::new();
    assert_eq!(large.capacity(), 64);
}
