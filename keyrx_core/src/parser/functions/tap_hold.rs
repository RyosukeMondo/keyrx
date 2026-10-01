//! TapHold function for Rhai DSL.
//!
//! Provides tap_hold(key, tap, hold, threshold_ms) function.

use crate::parser::builders;
use crate::parser::state::{call_line, ParserState};
use alloc::boxed::Box;
use alloc::sync::Arc;
use rhai::{Engine, EvalAltResult, NativeCallContext};
use spin::Mutex;

/// Register tap_hold function with the Rhai engine.
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
            state_clone
                .lock()
                .push_mapping(mapping, "tap_hold", call_line(&ctx))
        },
    );
}
