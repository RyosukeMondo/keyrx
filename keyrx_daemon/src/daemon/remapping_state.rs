//! The remapping/routing engine now lives in `keyrx_core::runtime::remapper`
//! so the daemon's live event loop and the `keyrx_core::simulate` driver run
//! the SAME code - a discrepancy between "what the daemon does" and "what
//! `simulate` says" can only be a bug in the engine, not two
//! implementations that drifted apart. This module just re-exports it under
//! its long-standing daemon name.

// The routing/state-sharing behavior itself is tested where it lives now:
// keyrx_core::runtime::remapper. This crate's own tests
// (windows_remap_pipeline_test.rs, live_profile_switch_test.rs) cover the
// daemon-side wiring (process_one_event, run_event_loop) against this
// re-exported type.
pub use keyrx_core::runtime::remapper::{
    active_layer, layer_modifiers, Remapped, Remapper as RemappingState, Routed,
};
