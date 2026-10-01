//! Map function for Rhai DSL.
//!
//! Provides map(from, to) function with overloads for string and ModifiedKey.

use crate::parser::builders;
use crate::parser::functions::modifiers::ModifiedKey;
use crate::parser::state::{call_line, ParserState};
use alloc::boxed::Box;
use alloc::sync::Arc;
use rhai::{Engine, EvalAltResult, NativeCallContext};
use spin::Mutex;

/// Register map functions with the Rhai engine.
pub fn register_map_functions(engine: &mut Engine, state: Arc<Mutex<ParserState>>) {
    // map(from: &str, to: &str)
    let state_clone = Arc::clone(&state);
    engine.register_fn(
        "map",
        move |ctx: NativeCallContext, from: &str, to: &str| -> Result<(), Box<EvalAltResult>> {
            let mapping =
                builders::build_map(from, to).map_err(|e| -> Box<EvalAltResult> { e.into() })?;
            state_clone
                .lock()
                .push_mapping(mapping, "map", call_line(&ctx))
        },
    );

    // map(from: &str, to: ModifiedKey) - creates a ModifiedOutput mapping
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
            state_clone
                .lock()
                .push_mapping(mapping, "map", call_line(&ctx))
        },
    );
}
