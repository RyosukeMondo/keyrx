//! Tests for TapHoldPhase, TapHoldConfig, and TapHoldOutput types.

use super::*;

#[test]
fn test_phase_default_is_idle() {
    let phase = TapHoldPhase::default();
    assert_eq!(phase, TapHoldPhase::Idle);
}

#[test]
fn test_phase_is_idle() {
    assert!(TapHoldPhase::Idle.is_idle());
    assert!(!TapHoldPhase::Pending.is_idle());
    assert!(!TapHoldPhase::Hold.is_idle());
}

#[test]
fn test_phase_is_pending() {
    assert!(!TapHoldPhase::Idle.is_pending());
    assert!(TapHoldPhase::Pending.is_pending());
    assert!(!TapHoldPhase::Hold.is_pending());
}

#[test]
fn test_phase_is_hold() {
    assert!(!TapHoldPhase::Idle.is_hold());
    assert!(!TapHoldPhase::Pending.is_hold());
    assert!(TapHoldPhase::Hold.is_hold());
}

#[test]
fn test_phase_as_str() {
    assert_eq!(TapHoldPhase::Idle.as_str(), "Idle");
    assert_eq!(TapHoldPhase::Pending.as_str(), "Pending");
    assert_eq!(TapHoldPhase::Hold.as_str(), "Hold");
}

#[test]
fn test_phase_display() {
    use alloc::format;
    assert_eq!(format!("{}", TapHoldPhase::Idle), "Idle");
    assert_eq!(format!("{}", TapHoldPhase::Pending), "Pending");
    assert_eq!(format!("{}", TapHoldPhase::Hold), "Hold");
}

// --- TapHoldConfig Tests ---

#[test]
fn test_config_new() {
    let config = TapHoldConfig::new(KeyCode::Escape, 5, 200_000);

    assert_eq!(config.tap_key(), KeyCode::Escape);
    assert_eq!(config.hold_modifier(), 5);
    assert_eq!(config.threshold_us(), 200_000);
}

#[test]
fn test_config_from_ms() {
    let config = TapHoldConfig::from_ms(KeyCode::Tab, 0, 150);

    assert_eq!(config.tap_key(), KeyCode::Tab);
    assert_eq!(config.hold_modifier(), 0);
    assert_eq!(config.threshold_us(), 150_000);
}

#[test]
fn test_config_from_ms_max_value() {
    // u16::MAX = 65535ms = 65,535,000μs
    let config = TapHoldConfig::from_ms(KeyCode::A, 254, u16::MAX);
    assert_eq!(config.threshold_us(), 65_535_000);
}

// --- TapHoldOutput Tests ---

#[test]
fn test_tap_hold_output_key_press() {
    let output = TapHoldOutput::key_press(KeyCode::Escape, 1000);
    match output {
        TapHoldOutput::KeyEvent {
            key,
            is_press,
            timestamp_us,
        } => {
            assert_eq!(key, KeyCode::Escape);
            assert!(is_press);
            assert_eq!(timestamp_us, 1000);
        }
        _ => panic!("Expected KeyEvent"),
    }
}

#[test]
fn test_tap_hold_output_key_release() {
    let output = TapHoldOutput::key_release(KeyCode::Tab, 2000);
    match output {
        TapHoldOutput::KeyEvent {
            key,
            is_press,
            timestamp_us,
        } => {
            assert_eq!(key, KeyCode::Tab);
            assert!(!is_press);
            assert_eq!(timestamp_us, 2000);
        }
        _ => panic!("Expected KeyEvent"),
    }
}

#[test]
fn test_tap_hold_output_activate_modifier() {
    let output = TapHoldOutput::activate_modifier(5);
    match output {
        TapHoldOutput::ActivateModifier { modifier_id } => {
            assert_eq!(modifier_id, 5);
        }
        _ => panic!("Expected ActivateModifier"),
    }
}

#[test]
fn test_tap_hold_output_deactivate_modifier() {
    let output = TapHoldOutput::deactivate_modifier(10);
    match output {
        TapHoldOutput::DeactivateModifier { modifier_id } => {
            assert_eq!(modifier_id, 10);
        }
        _ => panic!("Expected DeactivateModifier"),
    }
}
