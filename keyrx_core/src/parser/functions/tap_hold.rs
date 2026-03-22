//! TapHold function for Rhai DSL.
//!
//! Provides tap_hold(key, tap, hold, threshold_ms) function.

use crate::config::KeyMapping;
use crate::parser::builders;
use crate::parser::state::ParserState;
use alloc::boxed::Box;
use alloc::sync::Arc;
use rhai::{Engine, EvalAltResult};
use spin::Mutex;

/// Register tap_hold function with the Rhai engine.
pub fn register_tap_hold_function(engine: &mut Engine, state: Arc<Mutex<ParserState>>) {
    let state_clone = Arc::clone(&state);
    engine.register_fn(
        "tap_hold",
        move |key: &str,
              tap: &str,
              hold: &str,
              threshold_ms: i64|
              -> Result<(), Box<EvalAltResult>> {
            let mut state = state_clone.lock();
            let base_mapping = builders::build_tap_hold(key, tap, hold, threshold_ms as u16)
                .map_err(|e| -> Box<EvalAltResult> { e.into() })?;

            // If we're inside a conditional block, add to the conditional stack
            if let Some((_condition, ref mut mappings)) = state.conditional_stack.last_mut() {
                mappings.push(base_mapping);
                Ok(())
            } else if let Some(ref mut device) = state.current_device {
                // Otherwise, add to current device
                device.mappings.push(KeyMapping::Base(base_mapping));
                Ok(())
            } else {
                Err("tap_hold() must be called inside a device_start() block".into())
            }
        },
    );
}
