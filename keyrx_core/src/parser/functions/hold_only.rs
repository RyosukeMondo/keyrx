//! HoldOnly function for Rhai DSL.
//!
//! Provides hold_only(key, hold) and hold_only(key, hold, threshold_ms) functions.

use crate::parser::builders;
use crate::parser::state::{call_line, ParserState};
use alloc::boxed::Box;
use alloc::sync::Arc;
use rhai::{Engine, EvalAltResult, NativeCallContext};
use spin::Mutex;

/// Threshold used by the 2-argument `hold_only(key, hold)`.
pub const DEFAULT_HOLD_ONLY_THRESHOLD_MS: u16 = 200;

/// Register hold_only functions with the Rhai engine.
pub fn register_hold_only_functions(engine: &mut Engine, state: Arc<Mutex<ParserState>>) {
    // 2-arg overload: hold_only(key, hold) with the default threshold
    let state_2arg = Arc::clone(&state);
    engine.register_fn(
        "hold_only",
        move |ctx: NativeCallContext, key: &str, hold: &str| -> Result<(), Box<EvalAltResult>> {
            let mapping = builders::build_hold_only(key, hold, DEFAULT_HOLD_ONLY_THRESHOLD_MS)
                .map_err(|e| -> Box<EvalAltResult> { e.into() })?;
            state_2arg
                .lock()
                .push_mapping(mapping, "hold_only", call_line(&ctx))
        },
    );

    // 3-arg overload: hold_only(key, hold, threshold_ms)
    let state_3arg = Arc::clone(&state);
    engine.register_fn(
        "hold_only",
        move |ctx: NativeCallContext,
              key: &str,
              hold: &str,
              threshold_ms: i64|
              -> Result<(), Box<EvalAltResult>> {
            let mapping = builders::build_hold_only(key, hold, threshold_ms as u16)
                .map_err(|e| -> Box<EvalAltResult> { e.into() })?;
            state_3arg
                .lock()
                .push_mapping(mapping, "hold_only", call_line(&ctx))
        },
    );
}
