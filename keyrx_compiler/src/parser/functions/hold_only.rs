use keyrx_core::config::KeyMapping;
use keyrx_core::parser::builders;
use rhai::{Engine, EvalAltResult};
use std::sync::{Arc, Mutex};

use crate::parser::core::ParserState;

pub fn register_hold_only_function(engine: &mut Engine, state: Arc<Mutex<ParserState>>) {
    // 2-arg: hold_only(key, hold) with default 200ms threshold
    let state_2arg = Arc::clone(&state);
    engine.register_fn(
        "hold_only",
        move |key: &str, hold: &str| -> Result<(), Box<EvalAltResult>> {
            // SAFETY: Mutex cannot be poisoned - no panic paths while lock is held
            #[allow(clippy::unwrap_used)]
            let mut state = state_2arg.lock().unwrap();
            let base_mapping = builders::build_hold_only(key, hold, 200)
                .map_err(|e| -> Box<EvalAltResult> { e.into() })?;

            if let Some((_condition, ref mut mappings)) = state.conditional_stack.last_mut() {
                mappings.push(base_mapping);
                Ok(())
            } else if let Some(ref mut device) = state.current_device {
                device.mappings.push(KeyMapping::Base(base_mapping));
                Ok(())
            } else {
                Err("hold_only() must be called inside a device() block".into())
            }
        },
    );

    // 3-arg: hold_only(key, hold, threshold_ms)
    let state_3arg = Arc::clone(&state);
    engine.register_fn(
        "hold_only",
        move |key: &str, hold: &str, threshold_ms: i64| -> Result<(), Box<EvalAltResult>> {
            // SAFETY: Mutex cannot be poisoned - no panic paths while lock is held
            #[allow(clippy::unwrap_used)]
            let mut state = state_3arg.lock().unwrap();
            let base_mapping = builders::build_hold_only(key, hold, threshold_ms as u16)
                .map_err(|e| -> Box<EvalAltResult> { e.into() })?;

            if let Some((_condition, ref mut mappings)) = state.conditional_stack.last_mut() {
                mappings.push(base_mapping);
                Ok(())
            } else if let Some(ref mut device) = state.current_device {
                device.mappings.push(KeyMapping::Base(base_mapping));
                Ok(())
            } else {
                Err("hold_only() must be called inside a device() block".into())
            }
        },
    );
}
