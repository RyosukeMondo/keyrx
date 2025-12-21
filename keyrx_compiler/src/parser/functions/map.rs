use keyrx_core::config::{BaseKeyMapping, KeyMapping};
use rhai::{Engine, EvalAltResult};
use std::sync::{Arc, Mutex};

use crate::parser::core::ParserState;
use crate::parser::functions::modifiers::ModifiedKey;
use crate::parser::validators::{
    parse_lock_id, parse_modifier_id, parse_physical_key, parse_virtual_key,
};

pub fn register_map_function(engine: &mut Engine, state: Arc<Mutex<ParserState>>) {
    let state_clone = Arc::clone(&state);
    engine.register_fn(
        "map",
        move |from: &str, to: &str| -> Result<(), Box<EvalAltResult>> {
            let mut state = state_clone.lock().unwrap();
            let from_key =
                parse_physical_key(from).map_err(|e| format!("Invalid 'from' key: {}", e))?;

            let base_mapping = if to.starts_with("VK_") {
                let to_key =
                    parse_virtual_key(to).map_err(|e| format!("Invalid 'to' key: {}", e))?;
                BaseKeyMapping::Simple {
                    from: from_key,
                    to: to_key,
                }
            } else if to.starts_with("MD_") {
                let modifier_id =
                    parse_modifier_id(to).map_err(|e| format!("Invalid modifier ID: {}", e))?;
                BaseKeyMapping::Modifier {
                    from: from_key,
                    modifier_id,
                }
            } else if to.starts_with("LK_") {
                let lock_id = parse_lock_id(to).map_err(|e| format!("Invalid lock ID: {}", e))?;
                BaseKeyMapping::Lock {
                    from: from_key,
                    lock_id,
                }
            } else {
                return Err(format!(
                    "Output must have VK_, MD_, or LK_ prefix: {} -> use VK_{} for virtual key",
                    to, to
                )
                .into());
            };

            // If we're inside a conditional block, add to the conditional stack
            if let Some((_condition, ref mut mappings)) = state.conditional_stack.last_mut() {
                mappings.push(base_mapping);
                Ok(())
            } else if let Some(ref mut device) = state.current_device {
                // Otherwise, add to current device
                device.mappings.push(KeyMapping::Base(base_mapping));
                Ok(())
            } else {
                Err("map() must be called inside a device() block".into())
            }
        },
    );

    // map(from, ModifiedKey) overload - creates ModifiedOutput mapping
    let state_clone = Arc::clone(&state);
    engine.register_fn(
        "map",
        move |from: &str, to: ModifiedKey| -> Result<(), Box<EvalAltResult>> {
            let mut state = state_clone.lock().unwrap();
            let from_key =
                parse_physical_key(from).map_err(|e| format!("Invalid 'from' key: {}", e))?;

            let base_mapping = BaseKeyMapping::ModifiedOutput {
                from: from_key,
                to: to.key,
                shift: to.shift,
                ctrl: to.ctrl,
                alt: to.alt,
                win: to.win,
            };

            // If we're inside a conditional block, add to the conditional stack
            if let Some((_condition, ref mut mappings)) = state.conditional_stack.last_mut() {
                mappings.push(base_mapping);
                Ok(())
            } else if let Some(ref mut device) = state.current_device {
                // Otherwise, add to current device
                device.mappings.push(KeyMapping::Base(base_mapping));
                Ok(())
            } else {
                Err("map() must be called inside a device() block".into())
            }
        },
    );
}
