use keyrx_core::config::KeyMapping;
use rhai::{Engine, EvalAltResult};
use std::sync::{Arc, Mutex};

use crate::parser::core::ParserState;
use crate::parser::validators::{
    parse_key_name, parse_lock_id, parse_modifier_id, parse_virtual_key,
};

pub fn register_map_function(engine: &mut Engine, state: Arc<Mutex<ParserState>>) {
    let state_clone = Arc::clone(&state);
    engine.register_fn(
        "map",
        move |from: &str, to: &str| -> Result<(), Box<EvalAltResult>> {
            let mut state = state_clone.lock().unwrap();
            let from_key =
                parse_key_name(from).map_err(|e| format!("Invalid 'from' key: {}", e))?;

            let mapping = if to.starts_with("VK_") {
                let to_key =
                    parse_virtual_key(to).map_err(|e| format!("Invalid 'to' key: {}", e))?;
                KeyMapping::simple(from_key, to_key)
            } else if to.starts_with("MD_") {
                let modifier_id =
                    parse_modifier_id(to).map_err(|e| format!("Invalid modifier ID: {}", e))?;
                KeyMapping::modifier(from_key, modifier_id)
            } else if to.starts_with("LK_") {
                let lock_id = parse_lock_id(to).map_err(|e| format!("Invalid lock ID: {}", e))?;
                KeyMapping::lock(from_key, lock_id)
            } else {
                return Err(format!(
                    "Output must have VK_, MD_, or LK_ prefix: {} -> use VK_{} for virtual key",
                    to, to
                )
                .into());
            };

            if let Some(ref mut device) = state.current_device {
                device.mappings.push(mapping);
                Ok(())
            } else {
                Err("map() must be called inside a device() block".into())
            }
        },
    );
}
