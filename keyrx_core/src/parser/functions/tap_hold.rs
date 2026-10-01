//! Tap/hold style functions for Rhai DSL.
//!
//! Provides `tap_hold`, `tap_hold_timeout_only` (key, tap, hold,
//! threshold_ms) and `one_shot` (key, modifier [, timeout_ms]).

use crate::parser::builders;
use crate::parser::state::{call_line, ParserState};
use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;
use rhai::{Engine, EvalAltResult, NativeCallContext};
use spin::Mutex;

type Built = Result<crate::config::BaseKeyMapping, String>;

fn register_four_arg(
    engine: &mut Engine,
    state: &Arc<Mutex<ParserState>>,
    name: &'static str,
    build: fn(&str, &str, &str, u16) -> Built,
) {
    let state = Arc::clone(state);
    engine.register_fn(
        name,
        move |ctx: NativeCallContext,
              key: &str,
              tap: &str,
              hold: &str,
              threshold_ms: i64|
              -> Result<(), Box<EvalAltResult>> {
            let threshold = builders::threshold_ms(name, threshold_ms)?;
            let mapping =
                build(key, tap, hold, threshold).map_err(|e| -> Box<EvalAltResult> { e.into() })?;
            state.lock().push_mapping(mapping, name, call_line(&ctx))
        },
    );
}

/// Register tap_hold, tap_hold_timeout_only and one_shot with the Rhai engine.
pub fn register_tap_hold_function(engine: &mut Engine, state: Arc<Mutex<ParserState>>) {
    register_four_arg(engine, &state, "tap_hold", builders::build_tap_hold);
    register_four_arg(
        engine,
        &state,
        "tap_hold_timeout_only",
        builders::build_tap_hold_timeout_only,
    );

    let two = Arc::clone(&state);
    engine.register_fn(
        "one_shot",
        move |ctx: NativeCallContext,
              key: &str,
              modifier: &str|
              -> Result<(), Box<EvalAltResult>> {
            let mapping = builders::build_one_shot(key, modifier, 0)?;
            two.lock()
                .push_mapping(mapping, "one_shot", call_line(&ctx))
        },
    );
    let three = Arc::clone(&state);
    engine.register_fn(
        "one_shot",
        move |ctx: NativeCallContext,
              key: &str,
              modifier: &str,
              timeout_ms: i64|
              -> Result<(), Box<EvalAltResult>> {
            let mapping = builders::build_one_shot(key, modifier, timeout_ms)?;
            three
                .lock()
                .push_mapping(mapping, "one_shot", call_line(&ctx))
        },
    );
}
