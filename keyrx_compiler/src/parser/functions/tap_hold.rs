use keyrx_core::config::{BaseKeyMapping, KeyMapping};
use rhai::{Engine, EvalAltResult};
use std::sync::{Arc, Mutex};

use crate::parser::core::ParserState;
use crate::parser::validators::{parse_modifier_id, parse_physical_key, parse_virtual_key};

pub fn register_tap_hold_function(engine: &mut Engine, state: Arc<Mutex<ParserState>>) {
    let state_clone = Arc::clone(&state);
    engine.register_fn(
        "tap_hold",
        move |key: &str,
              tap: &str,
              hold: &str,
              threshold_ms: i64|
              -> Result<(), Box<EvalAltResult>> {
            let mut state = state_clone.lock().unwrap();
            let from_key = parse_physical_key(key).map_err(|e| format!("Invalid key: {}", e))?;

            if !tap.starts_with("VK_") {
                return Err(
                    format!("tap_hold tap parameter must have VK_ prefix, got: {}", tap).into(),
                );
            }
            let tap_key = parse_virtual_key(tap).map_err(|e| format!("Invalid tap key: {}", e))?;

            if !hold.starts_with("MD_") {
                return Err(format!(
                    "tap_hold hold parameter must have MD_ prefix, got: {}",
                    hold
                )
                .into());
            }
            let hold_modifier =
                parse_modifier_id(hold).map_err(|e| format!("Invalid hold modifier: {}", e))?;

            let base_mapping = BaseKeyMapping::TapHold {
                from: from_key,
                tap: tap_key,
                hold_modifier,
                threshold_ms: threshold_ms as u16,
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
                Err("tap_hold() must be called inside a device() block".into())
            }
        },
    );
}
