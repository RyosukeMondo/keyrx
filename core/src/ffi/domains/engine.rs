//! Engine domain FFI implementation.
#![allow(dead_code)] // TODO: Remove when #[ffi_export] is uncommented (task 20)
//!
//! Implements the FfiExportable trait for engine control, including bypass mode
//! and state event callbacks. Migrated from exports_engine.rs.
#![allow(unsafe_code)]

use crate::drivers::emergency_exit::{is_bypass_active, set_bypass_mode};
use crate::engine::state::{change::StateChange, snapshot::StateSnapshot};
use crate::ffi::context::FfiContext;
use crate::ffi::error::{FfiError, FfiResult};
use crate::ffi::events::{EventRegistry, EventType};
use crate::ffi::traits::FfiExportable;
// use keyrx_ffi_macros::ffi_export; // TODO: Uncomment when exports_*.rs files are removed (task 20)
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

/// Engine domain FFI implementation.
pub struct EngineFfi;

impl FfiExportable for EngineFfi {
    const DOMAIN: &'static str = "engine";

    fn init(ctx: &mut FfiContext) -> Result<(), FfiError> {
        if ctx.has_domain(Self::DOMAIN) {
            return Err(FfiError::invalid_input("engine domain already initialized"));
        }

        // Engine is primarily stateless - just stores bypass mode and emits events
        ctx.set_domain(Self::DOMAIN, ());

        Ok(())
    }

    fn cleanup(ctx: &mut FfiContext) {
        // Clean up engine domain marker
        ctx.remove_domain(Self::DOMAIN);
    }
}

/// Global event registry for Engine domain.
/// This will be moved into FfiContext in a future refactor.
pub(crate) fn global_event_registry() -> &'static EventRegistry {
    static REGISTRY: OnceLock<EventRegistry> = OnceLock::new();
    REGISTRY.get_or_init(EventRegistry::new)
}

// ─── FFI Exports ───────────────────────────────────────────────────────────

/// Check if emergency bypass mode is currently active.
///
/// When bypass mode is active, all key remapping is disabled.
///
/// Returns JSON: `ok:true` or `ok:false`
// #[ffi_export] // TODO: Uncomment when exports_*.rs files are removed (task 20)
pub fn is_bypass_mode_active() -> FfiResult<bool> {
    Ok(is_bypass_active())
}

/// Set the emergency bypass mode state.
///
/// # Arguments
/// * `active` - If true, enable bypass mode (disable remapping).
///   If false, disable bypass mode (re-enable remapping).
///
/// Returns JSON: `ok:null`
// #[ffi_export] // TODO: Uncomment when exports_*.rs files are removed (task 20)
pub fn set_bypass_mode_state(active: bool) -> FfiResult<()> {
    set_bypass_mode(active);
    Ok(())
}

// ─── State Event Publishing API ────────────────────────────────────────────

/// FFI state event containing a snapshot and optional metadata.
#[derive(Clone, Serialize, Deserialize, keyrx_ffi_macros::FfiMarshaler)]
#[ffi(strategy = "json")]
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
#[derive(Clone, Serialize, Deserialize, keyrx_ffi_macros::FfiMarshaler)]
#[ffi(strategy = "json")]
struct ChangeEvent {
    /// The state change that occurred.
    #[serde(flatten)]
    change: StateChange,
    /// Optional event type identifier.
    event_type: String,
}

fn emit_state_event(event: StateEvent) {
    global_event_registry().invoke(EventType::EngineState, &event);
}

fn emit_change_event(event: ChangeEvent) {
    global_event_registry().invoke(EventType::EngineState, &event);
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
    _timing: crate::engine::TimingConfig,
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
    fn test_engine_ffi_init() {
        let mut ctx = FfiContext::new();
        let result = EngineFfi::init(&mut ctx);
        assert!(result.is_ok());
        assert!(ctx.has_domain(EngineFfi::DOMAIN));
    }

    #[test]
    fn test_engine_ffi_double_init_fails() {
        let mut ctx = FfiContext::new();
        EngineFfi::init(&mut ctx).unwrap();

        // Second init should fail
        let result = EngineFfi::init(&mut ctx);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "INVALID_INPUT");
    }

    #[test]
    fn test_engine_ffi_cleanup() {
        let mut ctx = FfiContext::new();
        EngineFfi::init(&mut ctx).unwrap();
        assert!(ctx.has_domain(EngineFfi::DOMAIN));

        EngineFfi::cleanup(&mut ctx);
        assert!(!ctx.has_domain(EngineFfi::DOMAIN));
    }

    #[test]
    fn test_bypass_mode() {
        // Ensure clean state
        reset_bypass_mode();

        // Test bypass mode active
        let result = is_bypass_mode_active();
        assert!(result.is_ok());
        assert!(!result.unwrap());

        // Activate bypass
        let result = set_bypass_mode_state(true);
        assert!(result.is_ok());

        let result = is_bypass_mode_active();
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Deactivate bypass
        let result = set_bypass_mode_state(false);
        assert!(result.is_ok());

        let result = is_bypass_mode_active();
        assert!(result.is_ok());
        assert!(!result.unwrap());

        // Clean up
        reset_bypass_mode();
    }
}
