use rhai::Engine;
use std::sync::{Arc, Mutex};

use crate::parser::core::ParserState;

pub mod conditional;
pub mod device;
pub mod hold_only;
pub mod import;
pub mod map;
pub mod modifiers;
pub mod sequence;
pub mod tap_hold;

/// Registers every DSL function except `load` on `engine`. THE list of what
/// a script may call: the main script and each imported file use it, so an
/// imported file can never lack a function the main script has.
pub fn register_dsl(engine: &mut Engine, state: &Arc<Mutex<ParserState>>) {
    map::register_map_function(engine, Arc::clone(state));
    tap_hold::register_tap_hold_function(engine, Arc::clone(state));
    hold_only::register_hold_only_function(engine, Arc::clone(state));
    conditional::register_when_functions(engine, Arc::clone(state));
    sequence::register_sequence_function(engine, Arc::clone(state));
    modifiers::register_modifier_functions(engine);
    device::register_device_function(engine, Arc::clone(state));
}
