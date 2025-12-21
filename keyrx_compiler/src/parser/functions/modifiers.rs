use keyrx_core::config::{KeyCode, KeyMapping};
use rhai::{Engine, EvalAltResult};
use std::sync::{Arc, Mutex};

use crate::parser::core::ParserState;
use crate::parser::validators::{parse_key_name, parse_virtual_key};

pub fn register_modifier_functions(engine: &mut Engine, state: Arc<Mutex<ParserState>>) {
    register_single_mod_fn(
        engine,
        Arc::clone(&state),
        "with_shift",
        true,
        false,
        false,
        false,
    );
    register_single_mod_fn(
        engine,
        Arc::clone(&state),
        "with_ctrl",
        false,
        true,
        false,
        false,
    );
    register_single_mod_fn(
        engine,
        Arc::clone(&state),
        "with_alt",
        false,
        false,
        true,
        false,
    );

    let state_clone = Arc::clone(&state);
    engine.register_fn(
        "with_mods",
        move |from: &str,
              to: &str,
              shift: bool,
              ctrl: bool,
              alt: bool,
              win: bool|
              -> Result<(), Box<EvalAltResult>> {
            let mut state = state_clone.lock().unwrap();
            let from_key =
                parse_key_name(from).map_err(|e| format!("Invalid 'from' key: {}", e))?;
            if !to.starts_with("VK_") {
                return Err(format!("with_mods 'to' must have VK_ prefix, got: {}", to).into());
            }
            let to_key = parse_virtual_key(to).map_err(|e| format!("Invalid 'to' key: {}", e))?;
            add_modified_output(
                &mut state,
                from_key,
                to_key,
                ModifierFlags {
                    shift,
                    ctrl,
                    alt,
                    win,
                },
                "with_mods",
            )
        },
    );
}

fn register_single_mod_fn(
    engine: &mut Engine,
    state: Arc<Mutex<ParserState>>,
    name: &'static str,
    shift: bool,
    ctrl: bool,
    alt: bool,
    win: bool,
) {
    let state_clone = Arc::clone(&state);
    engine.register_fn(
        name,
        move |from: &str, to: &str| -> Result<(), Box<EvalAltResult>> {
            let mut state = state_clone.lock().unwrap();
            let from_key =
                parse_key_name(from).map_err(|e| format!("Invalid 'from' key: {}", e))?;
            if !to.starts_with("VK_") {
                return Err(format!("{} 'to' must have VK_ prefix, got: {}", name, to).into());
            }
            let to_key = parse_virtual_key(to).map_err(|e| format!("Invalid 'to' key: {}", e))?;
            add_modified_output(
                &mut state,
                from_key,
                to_key,
                ModifierFlags {
                    shift,
                    ctrl,
                    alt,
                    win,
                },
                name,
            )
        },
    );
}

struct ModifierFlags {
    shift: bool,
    ctrl: bool,
    alt: bool,
    win: bool,
}

fn add_modified_output(
    state: &mut ParserState,
    from: KeyCode,
    to: KeyCode,
    flags: ModifierFlags,
    fn_name: &str,
) -> Result<(), Box<EvalAltResult>> {
    let mapping =
        KeyMapping::modified_output(from, to, flags.shift, flags.ctrl, flags.alt, flags.win);
    if let Some(ref mut device) = state.current_device {
        device.mappings.push(mapping);
        Ok(())
    } else {
        Err(format!("{}() must be called inside a device() block", fn_name).into())
    }
}
