use keyrx_core::parser::builders;
use rhai::{Engine, EvalAltResult, NativeCallContext};
use std::sync::{Arc, Mutex};

use crate::parser::core::{call_line, with_state, ParserState};
use crate::parser::functions::modifiers::ModifiedKey;

pub fn register_map_function(engine: &mut Engine, state: Arc<Mutex<ParserState>>) {
    let state_clone = Arc::clone(&state);
    engine.register_fn(
        "map",
        move |ctx: NativeCallContext, from: &str, to: &str| -> Result<(), Box<EvalAltResult>> {
            let mapping =
                builders::build_map(from, to).map_err(|e| -> Box<EvalAltResult> { e.into() })?;
            with_state(&state_clone, |s| {
                s.push_mapping(mapping, "map", call_line(&ctx))
            })
        },
    );

    // map(from, ModifiedKey) overload - creates ModifiedOutput mapping
    let state_clone = Arc::clone(&state);
    engine.register_fn(
        "map",
        move |ctx: NativeCallContext,
              from: &str,
              to: ModifiedKey|
              -> Result<(), Box<EvalAltResult>> {
            let mapping =
                builders::build_modified_map(from, to.key, to.shift, to.ctrl, to.alt, to.win)
                    .map_err(|e| -> Box<EvalAltResult> { e.into() })?;
            with_state(&state_clone, |s| {
                s.push_mapping(mapping, "map", call_line(&ctx))
            })
        },
    );
}
