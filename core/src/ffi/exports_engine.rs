//! Engine control FFI exports.
//!
//! Functions for controlling the KeyRx engine state, including bypass mode
//! and state event callbacks.
#![allow(unsafe_code)]

use super::callbacks::{callback_registry, StateEventCallback};
use crate::drivers::emergency_exit::{is_bypass_active, set_bypass_mode};
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
/// The payload is a JSON blob with fields: layers, modifiers, held, pending, event, latency_us, timing.
#[no_mangle]
pub extern "C" fn keyrx_on_state(callback: Option<StateEventCallback>) {
    callback_registry().set_state(callback);

    emit_state_snapshot(FfiState {
        layers: vec!["base".into()],
        modifiers: Vec::new(),
        held: Vec::new(),
        pending: Vec::new(),
        event: Some("engine_ready".into()),
        latency_us: Some(0),
        timing: TimingConfig::default(),
    });
}

#[derive(Serialize)]
struct FfiState {
    layers: Vec<String>,
    modifiers: Vec<String>,
    held: Vec<String>,
    pending: Vec<String>,
    event: Option<String>,
    latency_us: Option<u64>,
    timing: TimingConfig,
}

fn emit_state_snapshot(state: FfiState) {
    callback_registry().invoke_state(&state);
}

/// Expose a safe-ish API for internal callers to emit state snapshots to the FFI listeners.
/// Engine subsystems can call this when state changes.
pub fn publish_state_snapshot(
    layers: Vec<String>,
    modifiers: Vec<String>,
    held: Vec<String>,
    pending: Vec<String>,
    event: Option<String>,
    latency_us: Option<u64>,
    timing: TimingConfig,
) {
    emit_state_snapshot(FfiState {
        layers,
        modifiers,
        held,
        pending,
        event,
        latency_us,
        timing,
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
