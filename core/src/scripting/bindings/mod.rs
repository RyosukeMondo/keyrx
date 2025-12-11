//! Rhai function bindings for the scripting runtime.
//!
//! This module contains all the `register_*` functions that add
//! KeyRx-specific functions to the Rhai engine:
//!
//! - **Remapping**: `remap`, `block`, `pass` (see [`remapping`])
//! - **Tap-hold**: `tap_hold`, `tap_hold_mod`, `combo` (see [`tap_hold`])
//! - **Layers**: `layer_define`, `layer_map`, `layer_push`, `layer_pop`, `layer_toggle` (see [`layers`])
//! - **Modifiers**: `define_modifier`, `modifier_on`, `modifier_off`, `one_shot` (see [`modifiers`])
//! - **Timing**: `set_tap_timeout`, `set_combo_timeout`, etc. (see [`timing`])
//! - **Row-Column API**: Position-based variants (see [`row_col`])

mod layers;
mod layouts;
mod modifiers;
mod remapping;
mod row_col;
mod tap_hold;
mod timing;

use super::builtins::{LayerView, ModifierView, PendingOps};
use super::row_col_resolver::RowColResolver;
use keyrx_core_macros::rhai_doc;
use rhai::Engine;
use std::sync::Arc;

/// Register all KeyRx functions with the Rhai engine.
///
/// This also registers type documentation for the Rhai API.
pub fn register_all_functions(
    engine: &mut Engine,
    pending_ops: &PendingOps,
    layer_view: &LayerView,
    modifier_view: &ModifierView,
    resolver: &Arc<RowColResolver>,
) {
    // Initialize documentation registry and register types
    super::docs::registry::initialize();
    super::docs::register_all_types();

    // Register functions by category
    register_debug(engine);
    remapping::register_remap(engine, pending_ops);
    remapping::register_block(engine, pending_ops);
    remapping::register_pass(engine, pending_ops);
    tap_hold::register_tap_hold(engine, pending_ops);
    tap_hold::register_tap_hold_mod(engine, pending_ops);
    tap_hold::register_combo(engine, pending_ops);
    layers::register_layer_functions(engine, pending_ops, layer_view);
    layouts::register_layout_functions(engine, pending_ops);
    modifiers::register_modifier_functions(engine, pending_ops, modifier_view);
    timing::register_timing_functions(engine, pending_ops);

    // Register row-column API functions
    row_col::register_remap_rc(engine, pending_ops, resolver);
    row_col::register_tap_hold_rc(engine, pending_ops, resolver);
    row_col::register_block_rc(engine, pending_ops, resolver);
    row_col::register_combo_rc(engine, pending_ops, resolver);
    row_col::register_layer_map_rc(engine, pending_ops, resolver, layer_view);
}

/// Print a debug message to the KeyRx logs.
///
/// This function outputs debug messages that can be viewed in the KeyRx logs.
/// Useful for debugging scripts and understanding execution flow.
///
/// # Parameters
/// * `msg` - The debug message to print
///
/// # Examples
/// ```
/// print_debug("Script started");
/// print_debug("Current layer: " + layer_name);
/// ```
///
/// # Notes
/// Debug messages are only visible when debug logging is enabled in KeyRx configuration.
#[rhai_doc(module = "debug")]
fn print_debug_impl(msg: &str) {
    tracing::debug!(
        service = "keyrx",
        event = "script_debug",
        component = "scripting_runtime",
        script_message = msg,
        "Script debug output"
    );
}

fn register_debug(engine: &mut Engine) {
    __register_doc_print_debug_impl();
    engine.register_fn("print_debug", print_debug_impl);
}
