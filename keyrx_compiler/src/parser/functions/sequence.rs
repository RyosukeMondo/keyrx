use keyrx_core::parser::builders;
use rhai::{Array, Engine, EvalAltResult, NativeCallContext};
use std::sync::{Arc, Mutex};

use crate::parser::core::{call_line, with_state, ParserState};

pub fn register_sequence_function(engine: &mut Engine, state: Arc<Mutex<ParserState>>) {
    let state_clone = Arc::clone(&state);
    engine.register_fn(
        "sequence",
        move |ctx: NativeCallContext, key: &str, keys: Array| -> Result<(), Box<EvalAltResult>> {
            let key_strs: Vec<String> = keys
                .iter()
                .map(|v| {
                    v.clone()
                        .into_string()
                        .map_err(|_| "Sequence keys must be strings")
                })
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| -> Box<EvalAltResult> { e.into() })?;

            let mapping = builders::build_sequence(key, &key_strs)
                .map_err(|e| -> Box<EvalAltResult> { e.into() })?;
            with_state(&state_clone, |s| {
                s.push_mapping(mapping, "sequence", call_line(&ctx))
            })
        },
    );
}
