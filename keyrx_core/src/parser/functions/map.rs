//! Map function for Rhai DSL.
//!
//! Provides map(from, to) function with overloads for string and ModifiedKey.

use crate::config::{BaseKeyMapping, KeyMapping};
use crate::parser::builders;
use crate::parser::functions::modifiers::ModifiedKey;
use crate::parser::state::ParserState;
use alloc::boxed::Box;
use alloc::sync::Arc;
use rhai::{Engine, EvalAltResult};
use spin::Mutex;

/// Register map functions with the Rhai engine.
pub fn register_map_functions(engine: &mut Engine, state: Arc<Mutex<ParserState>>) {
    // map(from: &str, to: &str)
    let state_clone = Arc::clone(&state);
    engine.register_fn(
        "map",
        move |from: &str, to: &str| -> Result<(), Box<EvalAltResult>> {
            let mut state = state_clone.lock();
            let base_mapping: BaseKeyMapping =
                builders::build_map(from, to).map_err(|e| -> Box<EvalAltResult> { e.into() })?;

            // If we're inside a conditional block, add to the conditional stack
            if let Some((_condition, ref mut mappings)) = state.conditional_stack.last_mut() {
                mappings.push(base_mapping);
                Ok(())
            } else if let Some(ref mut device) = state.current_device {
                // Otherwise, add to current device
                device.mappings.push(KeyMapping::Base(base_mapping));
                Ok(())
            } else {
                Err("map() must be called inside a device_start() block".into())
            }
        },
    );

    // map(from: &str, to: ModifiedKey) - creates ModifiedOutput mapping
    let state_clone = Arc::clone(&state);
    engine.register_fn(
        "map",
        move |from: &str, to: ModifiedKey| -> Result<(), Box<EvalAltResult>> {
            let mut state = state_clone.lock();
            let base_mapping =
                builders::build_modified_map(from, to.key, to.shift, to.ctrl, to.alt, to.win)
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
                Err("map() must be called inside a device_start() block".into())
            }
        },
    );
}
