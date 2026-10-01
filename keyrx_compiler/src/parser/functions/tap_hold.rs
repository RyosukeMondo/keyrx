//! Tap/hold style functions for the Rhai DSL: `tap_hold`,
//! `tap_hold_timeout_only` (key, tap, hold, threshold_ms) and `one_shot`
//! (key, modifier [, timeout_ms]). Mapping creation is shared with
//! keyrx_core's parser via `keyrx_core::parser::builders`.

use keyrx_core::config::BaseKeyMapping;
use keyrx_core::parser::builders;
use rhai::{Engine, EvalAltResult, NativeCallContext};
use std::sync::{Arc, Mutex};

use crate::parser::core::{call_line, with_state, ParserState};

type Built = Result<BaseKeyMapping, String>;

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
            with_state(&state, |s| s.push_mapping(mapping, name, call_line(&ctx)))
        },
    );
}

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
            with_state(&two, |s| {
                s.push_mapping(mapping, "one_shot", call_line(&ctx))
            })
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
            with_state(&three, |s| {
                s.push_mapping(mapping, "one_shot", call_line(&ctx))
            })
        },
    );
}
