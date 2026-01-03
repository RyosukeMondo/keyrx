use keyrx_core::config::{DeviceConfig, DeviceIdentifier};
use rhai::{Engine, EvalAltResult};
use std::sync::{Arc, Mutex};

use crate::parser::core::ParserState;

pub fn register_device_function(engine: &mut Engine, state: Arc<Mutex<ParserState>>) {
    let state_clone_start = Arc::clone(&state);
    engine.register_fn(
        "device_start",
        move |pattern: &str| -> Result<(), Box<EvalAltResult>> {
            // SAFETY: Mutex cannot be poisoned - no panic paths while lock is held
            #[allow(clippy::unwrap_used)]
            let mut state = state_clone_start.lock().unwrap();

            if let Some(device) = state.current_device.take() {
                state.devices.push(device);
            }

            state.current_device = Some(DeviceConfig {
                identifier: DeviceIdentifier {
                    pattern: pattern.to_string(),
                },
                mappings: Vec::new(),
            });

            Ok(())
        },
    );

    let state_clone_end = Arc::clone(&state);
    engine.register_fn("device_end", move || -> Result<(), Box<EvalAltResult>> {
        // SAFETY: Mutex cannot be poisoned - no panic paths while lock is held
        #[allow(clippy::unwrap_used)]
        let mut state = state_clone_end.lock().unwrap();

        if let Some(device) = state.current_device.take() {
            state.devices.push(device);
            Ok(())
        } else {
            Err("device_end() called without matching device_start()".into())
        }
    });
}
