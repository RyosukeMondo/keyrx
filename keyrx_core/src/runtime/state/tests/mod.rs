use super::*;
extern crate alloc;
use alloc::vec;

use crate::config::{Condition, ConditionItem, KeyCode};
use crate::runtime::tap_hold::TapHoldConfig;

// Property-based tests verifying state management invariants using proptest to
// generate random test cases, ensuring correctness across a wide range of inputs.
#[cfg(not(target_arch = "wasm32"))]
mod proptests;

// Device pattern matching tests.
mod device_pattern_tests;

// IME (Input Method Editor) state tests.
mod ime_tests;

#[test]
fn test_new_creates_zeroed_state() {
    let state = DeviceState::new();
    assert!(!state.is_modifier_active(0));
    assert!(!state.is_modifier_active(127));
    assert!(!state.is_modifier_active(254));
    assert!(!state.is_lock_active(0));
    assert!(!state.is_lock_active(127));
    assert!(!state.is_lock_active(254));
}

#[test]
fn test_set_modifier_valid_ids() {
    let mut state = DeviceState::new();

    // Test ID 0
    assert!(state.set_modifier(0));
    assert!(state.is_modifier_active(0));

    // Test ID 127 (middle)
    assert!(state.set_modifier(127));
    assert!(state.is_modifier_active(127));

    // Test ID 254 (max valid)
    assert!(state.set_modifier(254));
    assert!(state.is_modifier_active(254));
}

#[test]
fn test_set_modifier_invalid_id() {
    let mut state = DeviceState::new();

    // ID 255 should be rejected
    assert!(!state.set_modifier(255));
    // Modifier should not be set
    assert!(!state.is_modifier_active(255));
}

#[test]
fn test_clear_modifier() {
    let mut state = DeviceState::new();

    // Set then clear
    state.set_modifier(0);
    assert!(state.is_modifier_active(0));
    assert!(state.clear_modifier(0));
    assert!(!state.is_modifier_active(0));

    // Clear invalid ID
    assert!(!state.clear_modifier(255));
}

#[test]
fn test_toggle_lock_cycles() {
    let mut state = DeviceState::new();

    // OFF → ON
    assert!(state.toggle_lock(0));
    assert!(state.is_lock_active(0));

    // ON → OFF
    assert!(state.toggle_lock(0));
    assert!(!state.is_lock_active(0));

    // OFF → ON again
    assert!(state.toggle_lock(0));
    assert!(state.is_lock_active(0));
}

#[test]
fn test_toggle_lock_invalid_id() {
    let mut state = DeviceState::new();

    // ID 255 should be rejected
    assert!(!state.toggle_lock(255));
    assert!(!state.is_lock_active(255));
}

#[test]
fn test_evaluate_condition_modifier_active() {
    let mut state = DeviceState::new();
    state.set_modifier(0);

    let cond = Condition::ModifierActive(0);
    assert!(state.evaluate_condition(&cond));

    let cond_inactive = Condition::ModifierActive(1);
    assert!(!state.evaluate_condition(&cond_inactive));
}

#[test]
fn test_evaluate_condition_lock_active() {
    let mut state = DeviceState::new();
    state.toggle_lock(1);

    let cond = Condition::LockActive(1);
    assert!(state.evaluate_condition(&cond));

    let cond_inactive = Condition::LockActive(2);
    assert!(!state.evaluate_condition(&cond_inactive));
}

#[test]
fn test_evaluate_condition_all_active() {
    let mut state = DeviceState::new();
    state.set_modifier(0);
    state.toggle_lock(1);

    // Both conditions true
    let cond = Condition::AllActive(vec![
        ConditionItem::ModifierActive(0),
        ConditionItem::LockActive(1),
    ]);
    assert!(state.evaluate_condition(&cond));

    // One condition false
    let cond_partial = Condition::AllActive(vec![
        ConditionItem::ModifierActive(0),
        ConditionItem::LockActive(2), // Not active
    ]);
    assert!(!state.evaluate_condition(&cond_partial));

    // All conditions false
    let cond_none = Condition::AllActive(vec![
        ConditionItem::ModifierActive(10),
        ConditionItem::LockActive(11),
    ]);
    assert!(!state.evaluate_condition(&cond_none));
}

#[test]
fn test_evaluate_condition_not_active() {
    let mut state = DeviceState::new();
    state.set_modifier(0);

    // NOT(inactive) = true
    let cond_true = Condition::NotActive(vec![ConditionItem::ModifierActive(1)]);
    assert!(state.evaluate_condition(&cond_true));

    // NOT(active) = false
    let cond_false = Condition::NotActive(vec![ConditionItem::ModifierActive(0)]);
    assert!(!state.evaluate_condition(&cond_false));

    // NOT(MD_00 AND LK_01) with MD_00 active, LK_01 inactive = false (not all inactive)
    let cond_mixed = Condition::NotActive(vec![
        ConditionItem::ModifierActive(0), // Active
        ConditionItem::LockActive(1),     // Inactive
    ]);
    assert!(!state.evaluate_condition(&cond_mixed));

    // NOT(MD_10 AND LK_11) with both inactive = true
    let cond_both_inactive = Condition::NotActive(vec![
        ConditionItem::ModifierActive(10),
        ConditionItem::LockActive(11),
    ]);
    assert!(state.evaluate_condition(&cond_both_inactive));
}

#[test]
fn test_multiple_modifiers_independent() {
    let mut state = DeviceState::new();

    state.set_modifier(0);
    state.set_modifier(1);
    state.set_modifier(254);

    assert!(state.is_modifier_active(0));
    assert!(state.is_modifier_active(1));
    assert!(state.is_modifier_active(254));

    state.clear_modifier(1);
    assert!(state.is_modifier_active(0));
    assert!(!state.is_modifier_active(1));
    assert!(state.is_modifier_active(254));
}

#[test]
fn test_multiple_locks_independent() {
    let mut state = DeviceState::new();

    state.toggle_lock(0); // ON
    state.toggle_lock(1); // ON
    state.toggle_lock(2); // ON

    assert!(state.is_lock_active(0));
    assert!(state.is_lock_active(1));
    assert!(state.is_lock_active(2));

    state.toggle_lock(1); // OFF
    assert!(state.is_lock_active(0));
    assert!(!state.is_lock_active(1));
    assert!(state.is_lock_active(2));
}

#[test]
fn test_tap_hold_processor_accessors() {
    let mut state = DeviceState::new();

    // Test mutable accessor - register a tap-hold config
    let config = TapHoldConfig::new(KeyCode::Escape, 0, 200_000);
    let added = state
        .tap_hold_processor()
        .register_tap_hold(KeyCode::CapsLock, config);
    assert!(added);

    // Test immutable accessor - verify the config was registered
    assert!(state
        .tap_hold_processor_ref()
        .is_tap_hold_key(KeyCode::CapsLock));
    assert!(!state.tap_hold_processor_ref().is_tap_hold_key(KeyCode::A));
}
