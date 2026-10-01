//! Sequence function for Rhai DSL.
//!
//! Provides sequence(key, [keys...]) for multi-key output from a single keypress.

use crate::parser::builders;
use crate::parser::state::{call_line, ParserState};
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use rhai::{Array, Engine, EvalAltResult, NativeCallContext};
use spin::Mutex;

/// Register sequence function with the Rhai engine.
pub fn register_sequence_functions(engine: &mut Engine, state: Arc<Mutex<ParserState>>) {
    let state_clone = Arc::clone(&state);
    engine.register_fn(
        "sequence",
        move |ctx: NativeCallContext, key: &str, keys: Array| -> Result<(), Box<EvalAltResult>> {
            let key_strs: Vec<alloc::string::String> = keys
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
            state_clone
                .lock()
                .push_mapping(mapping, "sequence", call_line(&ctx))
        },
    );
}
