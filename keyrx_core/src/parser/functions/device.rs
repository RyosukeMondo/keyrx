//! Device block functions for Rhai DSL.
//!
//! Provides device_start() and device_end() functions.

use crate::config::{DeviceConfig, DeviceIdentifier};
use crate::parser::state::{call_line, ParserState};
use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::sync::Arc;
use rhai::{Engine, EvalAltResult, NativeCallContext};
use spin::Mutex;

/// Register device_start and device_end functions with the Rhai engine.
pub fn register_device_functions(engine: &mut Engine, state: Arc<Mutex<ParserState>>) {
    let state_clone_start = Arc::clone(&state);
    engine.register_fn(
        "device_start",
        move |ctx: NativeCallContext, pattern: &str| -> Result<(), Box<EvalAltResult>> {
            let mut state = state_clone_start.lock();

            if let Some(device) = state.current_device.take() {
                state.scopes.check_all_closed()?;
                state.devices.push(device);
            }
            state.scopes.start_device(pattern, call_line(&ctx))?;

            state.current_device = Some(DeviceConfig {
                identifier: DeviceIdentifier {
                    pattern: pattern.to_string(),
                },
                mappings: alloc::vec::Vec::new(),
            });

            Ok(())
        },
    );

    let state_clone_end = Arc::clone(&state);
    engine.register_fn("device_end", move || -> Result<(), Box<EvalAltResult>> {
        let mut state = state_clone_end.lock();

        if let Some(device) = state.current_device.take() {
            state.scopes.check_all_closed()?;
            state.scopes.end_device();
            state.devices.push(device);
            Ok(())
        } else {
            Err("device_end() called without matching device_start()".into())
        }
    });
}
