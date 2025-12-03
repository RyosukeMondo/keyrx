//! Engine control FFI exports.
//!
//! Functions for controlling the KeyRx engine state, including bypass mode
//! and state event callbacks.
#![allow(unsafe_code)]

use super::callbacks::{callback_registry, StateEventCallback};
use crate::drivers::emergency_exit::{is_bypass_active, set_bypass_mode};
use crate::engine::state::{change::StateChange, snapshot::StateSnapshot};
use crate::engine::TimingConfig;
use serde::Serialize;

/// Check if emergency bypass mode is currently active.
///
/// When bypass mode is active, all key remapping is disabled.
///
/// # Safety
/// This function is safe to call from any thread.
#[no_mangle]
pub extern "C" fn keyrx_is_bypass_active() -> bool {
    is_bypass_active()
}

/// Set the emergency bypass mode state.
///
/// # Arguments
///
/// * `active` - If true, enable bypass mode (disable remapping).
///   If false, disable bypass mode (re-enable remapping).
///
/// # Safety
/// This function is safe to call from any thread.
#[no_mangle]
pub extern "C" fn keyrx_set_bypass(active: bool) {
    set_bypass_mode(active);
}

/// Register a callback for engine state snapshots.
///
/// The payload is a JSON blob containing a StateSnapshot with optional event metadata.
#[no_mangle]
pub extern "C" fn keyrx_on_state(callback: Option<StateEventCallback>) {
    callback_registry().set_state(callback);

    // Emit an initial "engine_ready" event with an empty state snapshot
    emit_state_event(StateEvent {
        snapshot: StateSnapshot::empty(),
        event: Some("engine_ready".into()),
        latency_us: Some(0),
    });
}

/// FFI state event containing a snapshot and optional metadata.
#[derive(Serialize)]
struct StateEvent {
    /// State snapshot at this point in time.
    #[serde(flatten)]
    snapshot: StateSnapshot,
    /// Optional event name (e.g., "engine_ready", "key_down", "layer_change").
    event: Option<String>,
    /// Optional latency measurement in microseconds.
    latency_us: Option<u64>,
}

/// FFI change event containing a state change.
#[derive(Serialize)]
struct ChangeEvent {
    /// The state change that occurred.
    #[serde(flatten)]
    change: StateChange,
    /// Optional event type identifier.
    event_type: String,
}

fn emit_state_event(event: StateEvent) {
    callback_registry().invoke_state(&event);
}

fn emit_change_event(event: ChangeEvent) {
    callback_registry().invoke_state(&event);
}

/// Publish a state snapshot to FFI listeners.
///
/// This is the primary API for engine subsystems to emit state updates.
pub fn publish_state_snapshot(
    snapshot: StateSnapshot,
    event: Option<String>,
    latency_us: Option<u64>,
) {
    emit_state_event(StateEvent {
        snapshot,
        event,
        latency_us,
    });
}

/// Publish a state change event to FFI listeners.
///
/// This allows engine subsystems to emit granular change events in addition to snapshots.
pub fn publish_state_change(change: StateChange) {
    emit_change_event(ChangeEvent {
        change,
        event_type: "state_change".into(),
    });
}

/// Legacy compatibility function for old publish_state_snapshot signature.
///
/// Deprecated: Use publish_state_snapshot(StateSnapshot, ...) instead.
#[deprecated(note = "Use publish_state_snapshot(StateSnapshot, ...) instead")]
pub fn publish_state_snapshot_legacy(
    layers: Vec<String>,
    modifiers: Vec<String>,
    held: Vec<String>,
    pending: Vec<String>,
    event: Option<String>,
    latency_us: Option<u64>,
    _timing: TimingConfig,
) {
    // Convert legacy format to StateSnapshot format
    // This is a best-effort conversion for backward compatibility
    let _ = (layers, modifiers, held, pending);

    // For now, emit an empty snapshot with the event metadata
    // Full conversion would require mapping string names back to IDs
    emit_state_event(StateEvent {
        snapshot: StateSnapshot::empty(),
        event,
        latency_us,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drivers::emergency_exit::reset_bypass_mode;

    #[test]
    fn bypass_active_returns_current_state() {
        // Ensure clean state
        reset_bypass_mode();
        assert!(!keyrx_is_bypass_active());

        // Activate bypass
        keyrx_set_bypass(true);
        assert!(keyrx_is_bypass_active());

        // Deactivate bypass
        keyrx_set_bypass(false);
        assert!(!keyrx_is_bypass_active());

        // Clean up
        reset_bypass_mode();
    }
}
