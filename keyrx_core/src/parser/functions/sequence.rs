//! Sequence function for Rhai DSL.
//!
//! Provides sequence(key, [keys...]) for multi-key output from a single keypress.

use crate::config::KeyMapping;
use crate::parser::builders;
use crate::parser::state::ParserState;
use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec::Vec;
use rhai::{Array, Engine, EvalAltResult};
use spin::Mutex;

/// Register sequence function with the Rhai engine.
pub fn register_sequence_functions(engine: &mut Engine, state: Arc<Mutex<ParserState>>) {
    let state_clone = Arc::clone(&state);
    engine.register_fn(
        "sequence",
        move |key: &str, keys: Array| -> Result<(), Box<EvalAltResult>> {
            let mut state = state_clone.lock();

            let key_strs: Vec<alloc::string::String> = keys
                .iter()
                .map(|v| {
                    v.clone()
                        .into_string()
                        .map_err(|_| "Sequence keys must be strings")
                })
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| -> Box<EvalAltResult> { e.into() })?;

            let base_mapping = builders::build_sequence(key, &key_strs)
                .map_err(|e| -> Box<EvalAltResult> { e.into() })?;

            if let Some((_condition, ref mut mappings)) = state.conditional_stack.last_mut() {
                mappings.push(base_mapping);
                Ok(())
            } else if let Some(ref mut device) = state.current_device {
                device.mappings.push(KeyMapping::Base(base_mapping));
                Ok(())
            } else {
                Err("sequence() must be called inside a device_start() block".into())
            }
        },
    );
}
