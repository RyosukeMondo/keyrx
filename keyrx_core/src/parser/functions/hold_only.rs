//! HoldOnly function for Rhai DSL.
//!
//! Provides hold_only(key, hold) and hold_only(key, hold, threshold_ms) functions.

use crate::config::KeyMapping;
use crate::parser::builders;
use crate::parser::state::ParserState;
use alloc::boxed::Box;
use alloc::sync::Arc;
use rhai::{Engine, EvalAltResult};
use spin::Mutex;

/// Register hold_only functions with the Rhai engine.
pub fn register_hold_only_functions(engine: &mut Engine, state: Arc<Mutex<ParserState>>) {
    // 2-arg overload: hold_only(key, hold) with default 200ms threshold
    let state_2arg = Arc::clone(&state);
    engine.register_fn(
        "hold_only",
        move |key: &str, hold: &str| -> Result<(), Box<EvalAltResult>> {
            let mut state = state_2arg.lock();
            let base_mapping = builders::build_hold_only(key, hold, 200)
                .map_err(|e| -> Box<EvalAltResult> { e.into() })?;

            if let Some((_condition, ref mut mappings)) = state.conditional_stack.last_mut() {
                mappings.push(base_mapping);
                Ok(())
            } else if let Some(ref mut device) = state.current_device {
                device.mappings.push(KeyMapping::Base(base_mapping));
                Ok(())
            } else {
                Err("hold_only() must be called inside a device_start() block".into())
            }
        },
    );

    // 3-arg overload: hold_only(key, hold, threshold_ms)
    let state_3arg = Arc::clone(&state);
    engine.register_fn(
        "hold_only",
        move |key: &str, hold: &str, threshold_ms: i64| -> Result<(), Box<EvalAltResult>> {
            let mut state = state_3arg.lock();
            let base_mapping = builders::build_hold_only(key, hold, threshold_ms as u16)
                .map_err(|e| -> Box<EvalAltResult> { e.into() })?;

            if let Some((_condition, ref mut mappings)) = state.conditional_stack.last_mut() {
                mappings.push(base_mapping);
                Ok(())
            } else if let Some(ref mut device) = state.current_device {
                device.mappings.push(KeyMapping::Base(base_mapping));
                Ok(())
            } else {
                Err("hold_only() must be called inside a device_start() block".into())
            }
        },
    );
}
