use keyrx_core::parser::builders;
use rhai::{Engine, EvalAltResult, NativeCallContext};
use std::sync::{Arc, Mutex};

use crate::parser::core::{call_line, with_state, ParserState};

/// Threshold used by the 2-argument `hold_only(key, hold)`.
pub const DEFAULT_HOLD_ONLY_THRESHOLD_MS: u16 = 200;

pub fn register_hold_only_function(engine: &mut Engine, state: Arc<Mutex<ParserState>>) {
    // 2-arg: hold_only(key, hold) with the default threshold
    let state_2arg = Arc::clone(&state);
    engine.register_fn(
        "hold_only",
        move |ctx: NativeCallContext, key: &str, hold: &str| -> Result<(), Box<EvalAltResult>> {
            let mapping = builders::build_hold_only(key, hold, DEFAULT_HOLD_ONLY_THRESHOLD_MS)
                .map_err(|e| -> Box<EvalAltResult> { e.into() })?;
            with_state(&state_2arg, |s| {
                s.push_mapping(mapping, "hold_only", call_line(&ctx))
            })
        },
    );

    // 3-arg: hold_only(key, hold, threshold_ms)
    let state_3arg = Arc::clone(&state);
    engine.register_fn(
        "hold_only",
        move |ctx: NativeCallContext,
              key: &str,
              hold: &str,
              threshold_ms: i64|
              -> Result<(), Box<EvalAltResult>> {
            let mapping = builders::build_hold_only(
                key,
                hold,
                builders::threshold_ms("hold_only", threshold_ms)?,
            )
            .map_err(|e| -> Box<EvalAltResult> { e.into() })?;
            with_state(&state_3arg, |s| {
                s.push_mapping(mapping, "hold_only", call_line(&ctx))
            })
        },
    );
}
