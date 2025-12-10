#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::print_stdout, clippy::print_stderr, clippy::field_reassign_with_default, clippy::useless_conversion, clippy::assertions_on_constants, clippy::manual_div_ceil, clippy::manual_strip, clippy::len_zero, clippy::redundant_closure, clippy::manual_range_contains, clippy::default_constructed_unit_structs, clippy::clone_on_copy, clippy::io_other_error, clippy::bool_assert_comparison, clippy::approx_constant, clippy::let_unit_value, clippy::while_let_on_iterator, clippy::await_holding_lock, clippy::unnecessary_cast, clippy::drop_non_drop, clippy::needless_range_loop, unused_imports, unused_variables, dead_code, unsafe_code, clippy::collapsible_if, clippy::bool_comparison, unexpected_cfgs)]
//! Unit tests for KeyStateTracker.

use keyrx_core::engine::state::KeyStateTracker;
use keyrx_core::engine::KeyCode;
use std::collections::HashSet;

#[test]
fn tracks_press_and_release() {
    let mut tracker = KeyStateTracker::new();
    assert!(tracker.key_down(KeyCode::A, 100, false));
    assert!(tracker.is_pressed(KeyCode::A));
    assert_eq!(tracker.press_time(KeyCode::A), Some(100));

    let pressed_at = tracker.key_up(KeyCode::A);
    assert_eq!(pressed_at, Some(100));
    assert!(!tracker.is_pressed(KeyCode::A));
    assert!(tracker.is_empty());
}

#[test]
fn ignores_auto_repeat_when_already_pressed() {
    let mut tracker = KeyStateTracker::new();
    assert!(tracker.key_down(KeyCode::A, 100, false));
    assert!(!tracker.key_down(KeyCode::A, 200, true));
    assert_eq!(tracker.press_time(KeyCode::A), Some(100));
    assert_eq!(tracker.len(), 1);
}

#[test]
fn duplicate_non_repeat_does_not_reset_timestamp() {
    let mut tracker = KeyStateTracker::new();
    assert!(tracker.key_down(KeyCode::A, 100, false));
    assert!(!tracker.key_down(KeyCode::A, 200, false));
    assert_eq!(tracker.press_time(KeyCode::A), Some(100));
}

#[test]
fn key_up_on_non_pressed_is_noop() {
    let mut tracker = KeyStateTracker::new();
    assert_eq!(tracker.key_up(KeyCode::B), None);
    assert!(tracker.is_empty());
}

#[test]
fn iterates_pressed_keys() {
    let mut tracker = KeyStateTracker::new();
    tracker.key_down(KeyCode::A, 10, false);
    tracker.key_down(KeyCode::LeftShift, 20, false);

    let keys: HashSet<_> = tracker.pressed_keys().collect();
    assert!(keys.contains(&KeyCode::A));
    assert!(keys.contains(&KeyCode::LeftShift));
    assert_eq!(keys.len(), 2);
}
