use keyrx_core::config::KeyMapping;
use keyrx_core::parser::builders;
use rhai::{Engine, EvalAltResult};
use std::sync::{Arc, Mutex};

use crate::parser::core::ParserState;
use crate::parser::functions::modifiers::ModifiedKey;

pub fn register_map_function(engine: &mut Engine, state: Arc<Mutex<ParserState>>) {
    let state_clone = Arc::clone(&state);
    engine.register_fn(
        "map",
        move |from: &str, to: &str| -> Result<(), Box<EvalAltResult>> {
            // SAFETY: Mutex cannot be poisoned - no panic paths while lock is held
            #[allow(clippy::unwrap_used)]
            let mut state = state_clone.lock().unwrap();
            let base_mapping =
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
                Err("map() must be called inside a device() block".into())
            }
        },
    );

    // map(from, ModifiedKey) overload - creates ModifiedOutput mapping
    let state_clone = Arc::clone(&state);
    engine.register_fn(
        "map",
        move |from: &str, to: ModifiedKey| -> Result<(), Box<EvalAltResult>> {
            // SAFETY: Mutex cannot be poisoned - no panic paths while lock is held
            #[allow(clippy::unwrap_used)]
            let mut state = state_clone.lock().unwrap();
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
                Err("map() must be called inside a device() block".into())
            }
        },
    );
}
