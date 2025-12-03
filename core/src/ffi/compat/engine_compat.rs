//! Backward-compatible shims for Engine FFI.
//!
//! These functions maintain the old function signatures from exports_engine.rs
//! and forward calls to the new EngineFfi implementation via EventRegistry.
//!
//! All functions are marked as deprecated with migration guidance.
#![allow(unsafe_code)]

use super::super::callbacks::StateEventCallback;
use crate::engine::state::snapshot::StateSnapshot;
use crate::ffi::domains::engine::{global_event_registry, publish_state_snapshot};
use crate::ffi::events::EventType;

/// Register a callback for engine state snapshots.
///
/// **Deprecated**: Use `EventRegistry::register(EventType::EngineState, callback)` instead.
///
/// The payload is a JSON blob containing a StateSnapshot with optional event metadata.
#[deprecated(
    note = "Use EventRegistry::register(EventType::EngineState, callback) for unified event management"
)]
#[no_mangle]
pub extern "C" fn keyrx_on_state(callback: Option<StateEventCallback>) {
    global_event_registry().register(EventType::EngineState, callback);

    // Emit an initial "engine_ready" event with an empty state snapshot
    publish_state_snapshot(StateSnapshot::empty(), Some("engine_ready".into()), Some(0));
}
