use keyrx_core::config::KeyMapping;
use keyrx_core::parser::builders;
use rhai::{Array, Engine, EvalAltResult};
use std::sync::{Arc, Mutex};

use crate::parser::core::ParserState;

pub fn register_sequence_function(engine: &mut Engine, state: Arc<Mutex<ParserState>>) {
    let state_clone = Arc::clone(&state);
    engine.register_fn(
        "sequence",
        move |key: &str, keys: Array| -> Result<(), Box<EvalAltResult>> {
            // SAFETY: Mutex cannot be poisoned - no panic paths while lock is held
            #[allow(clippy::unwrap_used)]
            let mut state = state_clone.lock().unwrap();

            let key_strs: Vec<String> = keys
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
                Err("sequence() must be called inside a device() block".into())
            }
        },
    );
}
