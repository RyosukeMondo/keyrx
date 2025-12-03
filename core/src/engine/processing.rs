//! Event processing sub-functions extracted from process_event_traced.
//!
//! This module contains focused helper functions for each stage of event processing,
//! improving testability and maintainability of the advanced engine.

use crate::engine::{
    decision_engine::pass_through_event, DecisionType, EngineTracer, InputEvent, KeyCode,
    KeyStateTracker, OutputAction,
};
use std::collections::HashSet;

/// Result of validating an event and checking safe mode.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether to immediately return (synthetic event or safe mode toggle/active).
    pub early_return: bool,
    /// The output to return if early_return is true.
    pub early_output: Vec<OutputAction>,
    /// Whether safe mode was toggled.
    pub safe_mode_toggled: bool,
}

/// Validate event and check safe mode conditions.
///
/// Returns information about whether to short-circuit processing due to:
/// - Synthetic events (always pass through)
/// - Safe mode toggle (Ctrl+Alt+Shift+Escape chord)
/// - Active safe mode (all events pass through)
pub fn validate_and_check_safe_mode(
    event: &InputEvent,
    key_state: &KeyStateTracker,
    safe_mode: bool,
) -> ValidationResult {
    // Synthetic events always pass through
    if event.is_synthetic {
        return ValidationResult {
            early_return: true,
            early_output: vec![OutputAction::PassThrough],
            safe_mode_toggled: false,
        };
    }

    // Check safe mode toggle chord (Ctrl+Alt+Shift+Escape)
    if crate::engine::decision_engine::check_safe_mode_toggle(event, key_state) {
        return ValidationResult {
            early_return: true,
            early_output: vec![OutputAction::PassThrough],
            safe_mode_toggled: true,
        };
    }

    // Safe mode active - pass through everything
    if safe_mode {
        return ValidationResult {
            early_return: true,
            early_output: vec![OutputAction::PassThrough],
            safe_mode_toggled: false,
        };
    }

    ValidationResult {
        early_return: false,
        early_output: Vec::new(),
        safe_mode_toggled: false,
    }
}

/// Update key state tracker based on the event.
///
/// Tracks key press/release state for timing-based decisions.
pub fn update_key_state(event: &InputEvent, key_state: &mut KeyStateTracker) {
    if event.pressed {
        key_state.key_down(event.key, event.timestamp_us, event.is_repeat);
    } else {
        key_state.key_up(event.key);
    }
}

/// Result of decision resolution phase.
#[derive(Debug, Default)]
pub struct DecisionResult {
    /// Output actions accumulated so far.
    pub outputs: Vec<OutputAction>,
    /// Whether any action consumed the event.
    pub consumed: bool,
    /// The decision type for tracing.
    pub decision_type: DecisionType,
    /// Whether layer actions should be skipped.
    pub skip_layer_actions: bool,
    /// Whether the event was blocked for a pending combo.
    pub blocked_for_combo: bool,
}

/// Apply decision outcomes to engine state.
///
/// Handles blocked releases tracking based on event and decision results.
pub fn apply_decision(
    event: &InputEvent,
    blocked_releases: &mut HashSet<KeyCode>,
    mut result: DecisionResult,
) -> DecisionResult {
    // Block if combo is pending
    if result.blocked_for_combo {
        result.outputs.push(OutputAction::Block);
        result.consumed = true;
    }

    // Handle blocked releases on key up
    if !event.pressed {
        let was_blocked = blocked_releases.remove(&event.key);
        if was_blocked && !result.consumed {
            result.outputs.push(OutputAction::Block);
            result.consumed = true;
        }
    }

    // Default to pass-through if not consumed
    if !result.consumed {
        result.outputs.push(pass_through_event(event));
    }

    result
}

/// Emit tracing spans for the processed event.
///
/// Records input reception, decision making, and output generation spans
/// for performance analysis via OpenTelemetry.
pub fn trace_event(
    tracer: Option<&EngineTracer>,
    event: &InputEvent,
    decision_type: DecisionType,
    start_time: std::time::Instant,
    active_layer_ids: &[u32],
    outputs: &[OutputAction],
) {
    if let Some(t) = tracer {
        // Input span is already emitted at the start of processing
        let _input_span = t.span_input_received(event);

        let latency_us = start_time.elapsed().as_micros() as u64;
        let _decision_span = t.span_decision_made(decision_type, latency_us, active_layer_ids);
        let _output_span = t.span_output_generated(outputs);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key_down(key: KeyCode, ts: u64) -> InputEvent {
        InputEvent::key_down(key, ts)
    }

    fn key_up(key: KeyCode, ts: u64) -> InputEvent {
        InputEvent::key_up(key, ts)
    }

    fn synthetic_event(key: KeyCode, ts: u64) -> InputEvent {
        let mut event = InputEvent::key_down(key, ts);
        event.is_synthetic = true;
        event
    }

    #[test]
    fn validate_synthetic_event_returns_early() {
        let key_state = KeyStateTracker::new();
        let event = synthetic_event(KeyCode::A, 0);

        let result = validate_and_check_safe_mode(&event, &key_state, false);
        assert!(result.early_return);
        assert_eq!(result.early_output, vec![OutputAction::PassThrough]);
        assert!(!result.safe_mode_toggled);
    }

    #[test]
    fn validate_safe_mode_active_passes_through() {
        let key_state = KeyStateTracker::new();
        let event = key_down(KeyCode::A, 0);

        let result = validate_and_check_safe_mode(&event, &key_state, true);
        assert!(result.early_return);
        assert_eq!(result.early_output, vec![OutputAction::PassThrough]);
        assert!(!result.safe_mode_toggled);
    }

    #[test]
    fn validate_safe_mode_toggle_chord() {
        let mut key_state = KeyStateTracker::new();
        key_state.key_down(KeyCode::LeftCtrl, 0, false);
        key_state.key_down(KeyCode::LeftAlt, 0, false);
        key_state.key_down(KeyCode::LeftShift, 0, false);
        let event = key_down(KeyCode::Escape, 0);

        let result = validate_and_check_safe_mode(&event, &key_state, false);
        assert!(result.early_return);
        assert!(result.safe_mode_toggled);
    }

    #[test]
    fn validate_normal_event_continues() {
        let key_state = KeyStateTracker::new();
        let event = key_down(KeyCode::A, 0);

        let result = validate_and_check_safe_mode(&event, &key_state, false);
        assert!(!result.early_return);
        assert!(!result.safe_mode_toggled);
    }

    #[test]
    fn update_key_state_tracks_press() {
        let mut key_state = KeyStateTracker::new();
        let event = key_down(KeyCode::A, 100);

        update_key_state(&event, &mut key_state);
        assert!(key_state.is_pressed(KeyCode::A));
        assert_eq!(key_state.press_time(KeyCode::A), Some(100));
    }

    #[test]
    fn update_key_state_tracks_release() {
        let mut key_state = KeyStateTracker::new();
        key_state.key_down(KeyCode::A, 0, false);

        let event = key_up(KeyCode::A, 100);
        update_key_state(&event, &mut key_state);
        assert!(!key_state.is_pressed(KeyCode::A));
    }

    #[test]
    fn apply_decision_adds_block_for_combo() {
        let event = key_down(KeyCode::A, 0);
        let mut blocked_releases = HashSet::new();
        let decision = DecisionResult {
            outputs: vec![],
            consumed: false,
            decision_type: DecisionType::PassThrough,
            skip_layer_actions: false,
            blocked_for_combo: true,
        };

        let result = apply_decision(&event, &mut blocked_releases, decision);
        assert!(result.outputs.contains(&OutputAction::Block));
        assert!(result.consumed);
    }

    #[test]
    fn apply_decision_handles_blocked_release() {
        let event = key_up(KeyCode::A, 100);
        let mut blocked_releases = HashSet::new();
        blocked_releases.insert(KeyCode::A);
        let decision = DecisionResult {
            outputs: vec![],
            consumed: false,
            decision_type: DecisionType::PassThrough,
            skip_layer_actions: false,
            blocked_for_combo: false,
        };

        let result = apply_decision(&event, &mut blocked_releases, decision);
        assert!(result.outputs.contains(&OutputAction::Block));
        assert!(result.consumed);
        assert!(!blocked_releases.contains(&KeyCode::A));
    }

    #[test]
    fn apply_decision_passes_through_unconsumed() {
        let event = key_down(KeyCode::A, 0);
        let mut blocked_releases = HashSet::new();
        let decision = DecisionResult {
            outputs: vec![],
            consumed: false,
            decision_type: DecisionType::PassThrough,
            skip_layer_actions: false,
            blocked_for_combo: false,
        };

        let result = apply_decision(&event, &mut blocked_releases, decision);
        assert_eq!(result.outputs, vec![OutputAction::KeyDown(KeyCode::A)]);
    }

    #[test]
    fn apply_decision_preserves_consumed_outputs() {
        let event = key_down(KeyCode::A, 0);
        let mut blocked_releases = HashSet::new();
        let decision = DecisionResult {
            outputs: vec![OutputAction::KeyDown(KeyCode::B)],
            consumed: true,
            decision_type: DecisionType::Remap,
            skip_layer_actions: false,
            blocked_for_combo: false,
        };

        let result = apply_decision(&event, &mut blocked_releases, decision);
        assert_eq!(result.outputs, vec![OutputAction::KeyDown(KeyCode::B)]);
        assert!(result.consumed);
    }
}
