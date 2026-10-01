use keyrx_core::parser::builders;
use rhai::{Engine, EvalAltResult, NativeCallContext};
use std::sync::{Arc, Mutex};

use crate::parser::core::{call_line, with_state, ParserState};

pub fn register_tap_hold_function(engine: &mut Engine, state: Arc<Mutex<ParserState>>) {
    let state_clone = Arc::clone(&state);
    engine.register_fn(
        "tap_hold",
        move |ctx: NativeCallContext,
              key: &str,
              tap: &str,
              hold: &str,
              threshold_ms: i64|
              -> Result<(), Box<EvalAltResult>> {
            let mapping = builders::build_tap_hold(key, tap, hold, threshold_ms as u16)
                .map_err(|e| -> Box<EvalAltResult> { e.into() })?;
            with_state(&state_clone, |s| {
                s.push_mapping(mapping, "tap_hold", call_line(&ctx))
            })
        },
    );
}
