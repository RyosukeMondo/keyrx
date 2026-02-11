//! Tests for TapHoldState state machine.

use super::*;

// --- TapHoldState Tests ---

#[test]
fn test_state_new() {
    let config = TapHoldConfig::new(KeyCode::Escape, 0, 200_000);
    let state = TapHoldState::new(KeyCode::CapsLock, config);

    assert_eq!(state.key(), KeyCode::CapsLock);
    assert_eq!(state.phase(), TapHoldPhase::Idle);
    assert_eq!(state.press_time(), 0);
    assert_eq!(state.tap_key(), KeyCode::Escape);
    assert_eq!(state.hold_modifier(), 0);
    assert_eq!(state.threshold_us(), 200_000);
}

#[test]
fn test_state_transition_idle_to_pending() {
    let config = TapHoldConfig::new(KeyCode::Escape, 0, 200_000);
    let mut state = TapHoldState::new(KeyCode::CapsLock, config);

    state.transition_to_pending(1000);

    assert_eq!(state.phase(), TapHoldPhase::Pending);
    assert_eq!(state.press_time(), 1000);
}

#[test]
fn test_state_transition_pending_to_hold() {
    let config = TapHoldConfig::new(KeyCode::Escape, 0, 200_000);
    let mut state = TapHoldState::new(KeyCode::CapsLock, config);

    state.transition_to_pending(1000);
    state.transition_to_hold();

    assert_eq!(state.phase(), TapHoldPhase::Hold);
    assert_eq!(state.press_time(), 1000); // press_time preserved
}

#[test]
fn test_state_transition_to_idle() {
    let config = TapHoldConfig::new(KeyCode::Escape, 0, 200_000);
    let mut state = TapHoldState::new(KeyCode::CapsLock, config);

    // From Pending
    state.transition_to_pending(1000);
    state.transition_to_idle();
    assert_eq!(state.phase(), TapHoldPhase::Idle);
    assert_eq!(state.press_time(), 0);

    // From Hold
    state.transition_to_pending(2000);
    state.transition_to_hold();
    state.transition_to_idle();
    assert_eq!(state.phase(), TapHoldPhase::Idle);
    assert_eq!(state.press_time(), 0);
}

#[test]
fn test_state_reset() {
    let config = TapHoldConfig::new(KeyCode::Escape, 0, 200_000);
    let mut state = TapHoldState::new(KeyCode::CapsLock, config);

    state.transition_to_pending(5000);
    state.reset();

    assert_eq!(state.phase(), TapHoldPhase::Idle);
    assert_eq!(state.press_time(), 0);
}

#[test]
fn test_is_threshold_exceeded() {
    let config = TapHoldConfig::new(KeyCode::Escape, 0, 200_000); // 200ms
    let mut state = TapHoldState::new(KeyCode::CapsLock, config);

    state.transition_to_pending(1_000_000); // pressed at 1s

    // Before threshold
    assert!(!state.is_threshold_exceeded(1_100_000)); // 100ms elapsed
    assert!(!state.is_threshold_exceeded(1_199_999)); // just under

    // At threshold
    assert!(state.is_threshold_exceeded(1_200_000)); // exactly 200ms

    // After threshold
    assert!(state.is_threshold_exceeded(1_300_000)); // 300ms elapsed
}

#[test]
fn test_elapsed() {
    let config = TapHoldConfig::new(KeyCode::Escape, 0, 200_000);
    let mut state = TapHoldState::new(KeyCode::CapsLock, config);

    state.transition_to_pending(1_000_000);

    assert_eq!(state.elapsed(1_000_000), 0);
    assert_eq!(state.elapsed(1_100_000), 100_000);
    assert_eq!(state.elapsed(1_500_000), 500_000);
}

#[test]
fn test_elapsed_saturates_on_underflow() {
    let config = TapHoldConfig::new(KeyCode::Escape, 0, 200_000);
    let mut state = TapHoldState::new(KeyCode::CapsLock, config);

    state.transition_to_pending(1_000_000);

    // Current time before press time (shouldn't happen, but handle gracefully)
    assert_eq!(state.elapsed(500_000), 0);
}

#[test]
fn test_tap_scenario() {
    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    let mut state = TapHoldState::new(KeyCode::CapsLock, config);

    // Key pressed
    state.transition_to_pending(0);
    assert!(state.phase().is_pending());

    // Quick release (100ms < 200ms threshold)
    let release_time = 100_000; // 100ms
    assert!(!state.is_threshold_exceeded(release_time));

    // Would emit tap key (Escape) - transition back to idle
    state.transition_to_idle();
    assert!(state.phase().is_idle());
}

#[test]
fn test_hold_scenario() {
    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    let mut state = TapHoldState::new(KeyCode::CapsLock, config);

    // Key pressed
    state.transition_to_pending(0);
    assert!(state.phase().is_pending());

    // Threshold exceeded (300ms > 200ms)
    let check_time = 300_000;
    assert!(state.is_threshold_exceeded(check_time));

    // Transition to hold
    state.transition_to_hold();
    assert!(state.phase().is_hold());

    // Key released
    state.transition_to_idle();
    assert!(state.phase().is_idle());
}

#[test]
fn test_permissive_hold_scenario() {
    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    let mut state = TapHoldState::new(KeyCode::CapsLock, config);

    // Key pressed
    state.transition_to_pending(0);

    // Another key pressed before threshold (50ms < 200ms)
    // This triggers permissive hold
    let interrupt_time = 50_000;
    assert!(!state.is_threshold_exceeded(interrupt_time));

    // Immediately transition to hold (permissive hold behavior)
    state.transition_to_hold();
    assert!(state.phase().is_hold());
}

#[test]
fn test_config_accessors() {
    let config = TapHoldConfig::new(KeyCode::Tab, 3, 150_000);
    let state = TapHoldState::new(KeyCode::Space, config);

    assert_eq!(state.config().tap_key(), KeyCode::Tab);
    assert_eq!(state.config().hold_modifier(), 3);
    assert_eq!(state.config().threshold_us(), 150_000);
}
