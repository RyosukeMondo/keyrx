//! Rhai function bindings for the scripting runtime.
//!
//! This module contains all the `register_*` functions that add
//! KeyRx-specific functions to the Rhai engine:
//! - Remapping functions (remap, block, pass, tap_hold, combo)
//! - Layer functions (layer_define, layer_map, layer_push, layer_pop, layer_toggle)
//! - Modifier functions (define_modifier, modifier_on, modifier_off, one_shot)
//! - Timing functions (set_tap_timeout, set_combo_timeout, etc.)

use super::builtins::{
    layer_error, normalize_layer_name, normalize_modifier_name, parse_keys_array,
    parse_layer_action, validate_timeout, with_layer_view, with_modifier_view, LayerView,
    ModifierView, PendingOp, PendingOps, TimingUpdate,
};
use super::helpers::parse_key_or_error;
use crate::engine::{HoldAction, LayerAction, VirtualModifiers};
use crate::scripting::sandbox::validation::InputValidator;
use crate::scripting::sandbox::validators::RangeValidator;
use keyrx_core_macros::rhai_doc;
use rhai::{Array, Engine, EvalAltResult, Position};
use std::sync::Arc;

/// Register all KeyRx functions with the Rhai engine.
pub fn register_all_functions(
    engine: &mut Engine,
    pending_ops: &PendingOps,
    layer_view: &LayerView,
    modifier_view: &ModifierView,
) {
    register_debug(engine);
    register_remap(engine, pending_ops);
    register_block(engine, pending_ops);
    register_pass(engine, pending_ops);
    register_tap_hold(engine, pending_ops);
    register_tap_hold_mod(engine, pending_ops);
    register_combo(engine, pending_ops);
    register_layer_functions(engine, pending_ops, layer_view);
    register_modifier_functions(engine, pending_ops, modifier_view);
    register_timing_functions(engine, pending_ops);
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

/// Remap a key to another key.
///
/// Creates a basic key remapping that translates one keypress into another.
/// When the `from` key is pressed, it will be replaced with the `to` key.
///
/// # Parameters
/// * `from` - The key to remap (e.g., "A", "CapsLock", "Esc")
/// * `to` - The key to map it to (e.g., "B", "Ctrl", "Tab")
///
/// # Returns
/// Result indicating success or error
///
/// # Examples
/// ```
/// remap("CapsLock", "Esc");
/// remap("A", "B");
/// remap("F13", "AudioMute");
/// ```
///
/// # Notes
/// Keys are case-insensitive. Both physical key names and key codes are supported.
#[rhai_doc(module = "remapping")]
fn remap_impl(
    from: &str,
    to: &str,
    ops: &PendingOps,
) -> std::result::Result<(), Box<EvalAltResult>> {
    let from_key = parse_key_or_error(from, "remap")?;
    let to_key = parse_key_or_error(to, "remap")?;
    tracing::debug!(
        service = "keyrx",
        event = "remap_registered",
        component = "scripting_runtime",
        from = from,
        to = to,
        "Registered remap"
    );
    if let Ok(mut ops) = ops.lock() {
        ops.push(PendingOp::Remap {
            from: from_key,
            to: to_key,
        });
    }
    Ok(())
}

fn register_remap(engine: &mut Engine, pending_ops: &PendingOps) {
    __register_doc_remap_impl();
    let ops = Arc::clone(pending_ops);
    engine.register_fn(
        "remap",
        move |from: &str, to: &str| -> std::result::Result<(), Box<EvalAltResult>> {
            remap_impl(from, to, &ops)
        },
    );
}

/// Block a key from being sent to the system.
///
/// Prevents the specified key from having any effect when pressed.
/// The key press will be completely ignored by the system.
///
/// # Parameters
/// * `key` - The key to block (e.g., "CapsLock", "Esc", "F1")
///
/// # Returns
/// Result indicating success or error
///
/// # Examples
/// ```
/// block("CapsLock");
/// block("F1");
/// block("PrintScreen");
/// ```
///
/// # Notes
/// Blocked keys cannot be used in combos or other bindings.
#[rhai_doc(module = "remapping")]
fn block_impl(key: &str, ops: &PendingOps) -> std::result::Result<(), Box<EvalAltResult>> {
    let key_code = parse_key_or_error(key, "block")?;
    tracing::debug!(
        service = "keyrx",
        event = "block_registered",
        component = "scripting_runtime",
        key = key,
        "Registered block"
    );
    if let Ok(mut ops) = ops.lock() {
        ops.push(PendingOp::Block { key: key_code });
    }
    Ok(())
}

fn register_block(engine: &mut Engine, pending_ops: &PendingOps) {
    __register_doc_block_impl();
    let ops = Arc::clone(pending_ops);
    engine.register_fn(
        "block",
        move |key: &str| -> std::result::Result<(), Box<EvalAltResult>> { block_impl(key, &ops) },
    );
}

/// Pass a key through unchanged.
///
/// Explicitly allows a key to pass through to the system without modification.
/// Useful for documenting intent or overriding previous bindings.
///
/// # Parameters
/// * `key` - The key to pass through (e.g., "A", "Enter", "Space")
///
/// # Returns
/// Result indicating success or error
///
/// # Examples
/// ```
/// pass("A");
/// pass("Enter");
/// pass("Space");
/// ```
///
/// # Notes
/// In most cases, keys pass through by default. This is mainly used for clarity.
#[rhai_doc(module = "remapping")]
fn pass_impl(key: &str, ops: &PendingOps) -> std::result::Result<(), Box<EvalAltResult>> {
    let key_code = parse_key_or_error(key, "pass")?;
    tracing::debug!(
        service = "keyrx",
        event = "pass_registered",
        component = "scripting_runtime",
        key = key,
        "Registered pass"
    );
    if let Ok(mut ops) = ops.lock() {
        ops.push(PendingOp::Pass { key: key_code });
    }
    Ok(())
}

fn register_pass(engine: &mut Engine, pending_ops: &PendingOps) {
    __register_doc_pass_impl();
    let ops = Arc::clone(pending_ops);
    engine.register_fn(
        "pass",
        move |key: &str| -> std::result::Result<(), Box<EvalAltResult>> { pass_impl(key, &ops) },
    );
}

/// Configure a key to perform different actions on tap vs hold.
///
/// Creates dual-function keys that behave differently based on press duration.
/// Quick tap produces one key, holding produces another.
///
/// # Parameters
/// * `key` - The physical key to bind (e.g., "CapsLock", "Space", "Tab")
/// * `tap` - The key sent on quick tap (e.g., "Esc", "Space", "Enter")
/// * `hold` - The key sent when held (e.g., "Ctrl", "Shift", "Alt")
///
/// # Returns
/// Result indicating success or error
///
/// # Examples
/// ```
/// tap_hold("CapsLock", "Esc", "Ctrl");
/// tap_hold("Space", "Space", "Shift");
/// tap_hold("Tab", "Tab", "Hyper");
/// ```
///
/// # Notes
/// Timing is controlled by set_tap_timeout(). Default is 200ms.
#[rhai_doc(module = "tap_hold")]
fn tap_hold_impl(
    key: &str,
    tap: &str,
    hold: &str,
    ops: &PendingOps,
) -> std::result::Result<(), Box<EvalAltResult>> {
    let key_code = parse_key_or_error(key, "tap_hold")?;
    let tap_code = parse_key_or_error(tap, "tap_hold")?;
    let hold_code = parse_key_or_error(hold, "tap_hold")?;

    if let Ok(mut ops) = ops.lock() {
        ops.push(PendingOp::TapHold {
            key: key_code,
            tap: tap_code,
            hold: HoldAction::Key(hold_code),
        });
    }
    Ok(())
}

fn register_tap_hold(engine: &mut Engine, pending_ops: &PendingOps) {
    __register_doc_tap_hold_impl();
    let ops = Arc::clone(pending_ops);
    engine.register_fn(
        "tap_hold",
        move |key: &str, tap: &str, hold: &str| -> std::result::Result<(), Box<EvalAltResult>> {
            tap_hold_impl(key, tap, hold, &ops)
        },
    );
}

/// Configure a key to send a tap key or activate a virtual modifier when held.
///
/// Like tap_hold, but the hold action activates a virtual modifier instead of a key.
/// Useful for creating custom modifier keys.
///
/// # Parameters
/// * `key` - The physical key to bind (e.g., "CapsLock", "Space", "Tab")
/// * `tap` - The key sent on quick tap (e.g., "Esc", "Space", "Enter")
/// * `modifier_id` - The virtual modifier ID to activate when held (0-255)
///
/// # Returns
/// Result indicating success or error
///
/// # Examples
/// ```
/// let hyper = define_modifier("Hyper");
/// tap_hold_mod("CapsLock", "Esc", hyper);
/// tap_hold_mod("Space", "Space", hyper);
/// ```
///
/// # Notes
/// Modifier ID must be obtained from define_modifier(). Range is 0-255.
#[rhai_doc(module = "tap_hold")]
fn tap_hold_mod_impl(
    key: &str,
    tap: &str,
    modifier_id: i64,
    ops: &PendingOps,
) -> std::result::Result<(), Box<EvalAltResult>> {
    let key_code = parse_key_or_error(key, "tap_hold_mod")?;
    let tap_code = parse_key_or_error(tap, "tap_hold_mod")?;

    // Validate modifier_id range using RangeValidator
    let validator = RangeValidator::new(0i64, VirtualModifiers::MAX_ID as i64);
    validator.validate(&modifier_id).map_err(|e| {
        Box::new(EvalAltResult::ErrorRuntime(
            format!("tap_hold_mod: modifier id {}", e).into(),
            rhai::Position::NONE,
        ))
    })?;

    let modifier = modifier_id as u8;

    if let Ok(mut ops) = ops.lock() {
        ops.push(PendingOp::TapHold {
            key: key_code,
            tap: tap_code,
            hold: HoldAction::Modifier(modifier),
        });
    }
    Ok(())
}

fn register_tap_hold_mod(engine: &mut Engine, pending_ops: &PendingOps) {
    __register_doc_tap_hold_mod_impl();
    let ops = Arc::clone(pending_ops);
    engine.register_fn(
        "tap_hold_mod",
        move |key: &str,
              tap: &str,
              modifier_id: i64|
              -> std::result::Result<(), Box<EvalAltResult>> {
            tap_hold_mod_impl(key, tap, modifier_id, &ops)
        },
    );
}

/// Define a key combination that triggers an action.
///
/// Creates a chord of multiple keys pressed together that produces a single output.
/// All keys must be pressed within the combo timeout window.
///
/// # Parameters
/// * `keys` - Array of keys that form the combo (2-4 keys, e.g., ["J", "K"])
/// * `action_key` - The key to send when combo is triggered (e.g., "Esc", "Enter")
///
/// # Returns
/// Result indicating success or error
///
/// # Examples
/// ```
/// combo(["J", "K"], "Esc");
/// combo(["A", "S", "D"], "Enter");
/// combo(["H", "L"], "Left");
/// ```
///
/// # Notes
/// Combo size must be 2-4 keys. Timing controlled by set_combo_timeout(). Default is 50ms.
#[rhai_doc(module = "combos")]
fn combo_impl(
    keys: Array,
    action_key: &str,
    ops: &PendingOps,
) -> std::result::Result<(), Box<EvalAltResult>> {
    let parsed_keys = parse_keys_array(keys)?;
    let action = parse_key_or_error(action_key, "combo")?;

    // Validate combo size using range check
    let len = parsed_keys.len();
    if !(2..=4).contains(&len) {
        return Err(Box::new(EvalAltResult::ErrorRuntime(
            format!(
                "combo: combo size {} violates constraint: must have 2-4 keys",
                len
            )
            .into(),
            Position::NONE,
        )));
    }

    if let Ok(mut ops) = ops.lock() {
        ops.push(PendingOp::Combo {
            keys: parsed_keys,
            action: LayerAction::Remap(action),
        });
    }
    Ok(())
}

fn register_combo(engine: &mut Engine, pending_ops: &PendingOps) {
    __register_doc_combo_impl();
    let ops = Arc::clone(pending_ops);
    engine.register_fn(
        "combo",
        move |keys: Array, action_key: &str| -> std::result::Result<(), Box<EvalAltResult>> {
            combo_impl(keys, action_key, &ops)
        },
    );
}

fn register_layer_functions(engine: &mut Engine, pending_ops: &PendingOps, layer_view: &LayerView) {
    register_layer_define(engine, pending_ops, layer_view);
    register_layer_map(engine, pending_ops, layer_view);
    register_layer_push(engine, pending_ops, layer_view);
    register_layer_pop(engine, pending_ops, layer_view);
    register_layer_toggle(engine, pending_ops, layer_view);
    register_is_layer_active(engine, layer_view);
}

/// Define a new layer.
///
/// Creates a named layer that can be activated and have keys mapped to it.
/// Layers stack on top of the base layer to provide different key mappings.
///
/// # Parameters
/// * `name` - The name of the layer (e.g., "Nav", "Symbols", "Numbers")
/// * `transparent` - If true, unmapped keys fall through to layers below
///
/// # Returns
/// Result indicating success or error
///
/// # Examples
/// ```
/// layer_define("Nav", true);
/// layer_define("Symbols", false);
/// layer_define("Numbers", true);
/// ```
///
/// # Notes
/// Layer names are case-insensitive. Transparent layers allow key fallthrough.
#[rhai_doc(module = "layers")]
fn layer_define_impl(
    name: &str,
    transparent: bool,
    ops: &PendingOps,
    view: &LayerView,
) -> std::result::Result<(), Box<EvalAltResult>> {
    let normalized = normalize_layer_name(name, "layer_define")?;
    with_layer_view(view, |stack| {
        stack.define_or_update_named(&normalized, transparent);
        Ok(())
    })?;

    if let Ok(mut ops) = ops.lock() {
        ops.push(PendingOp::LayerDefine {
            name: normalized,
            transparent,
        });
    }
    Ok(())
}

fn register_layer_define(engine: &mut Engine, pending_ops: &PendingOps, layer_view: &LayerView) {
    __register_doc_layer_define_impl();
    let ops = Arc::clone(pending_ops);
    let view = Arc::clone(layer_view);
    engine.register_fn(
        "layer_define",
        move |name: &str, transparent: bool| -> std::result::Result<(), Box<EvalAltResult>> {
            layer_define_impl(name, transparent, &ops, &view)
        },
    );
}

/// Map a key to an action on a specific layer.
///
/// Assigns behavior to a key when a particular layer is active.
/// The action can be a key remap, layer push/pop, or other actions.
///
/// # Parameters
/// * `layer_name` - The layer to map on (must be defined first)
/// * `key` - The key to map (e.g., "H", "J", "K", "L")
/// * `action` - The action to perform (e.g., "Left", "Down", "Up", "Right", "push:Nav")
///
/// # Returns
/// Result indicating success or error
///
/// # Examples
/// ```
/// layer_map("Nav", "H", "Left");
/// layer_map("Nav", "J", "Down");
/// layer_map("Nav", "K", "Up");
/// layer_map("Nav", "L", "Right");
/// layer_map("Nav", "Space", "push:Symbols");
/// ```
///
/// # Notes
/// Layer must be defined before mapping keys. Actions support: remap, push:layer, pop:layer, toggle:layer.
#[rhai_doc(module = "layers")]
fn layer_map_impl(
    layer_name: &str,
    key: &str,
    action: &str,
    ops: &PendingOps,
    view: &LayerView,
) -> std::result::Result<(), Box<EvalAltResult>> {
    let normalized = normalize_layer_name(layer_name, "layer_map")?;
    let key_code = parse_key_or_error(key, "layer_map")?;
    let parsed_action = parse_layer_action(action, "layer_map", view)?;

    // Require layer to be defined in the preview state to give early feedback.
    with_layer_view(view, |stack| {
        if stack.layer_id_by_name(&normalized).is_none() {
            return Err(layer_error(
                "layer_map",
                format!("layer '{}' is not defined", normalized),
            ));
        }
        Ok(())
    })?;

    if let Ok(mut ops) = ops.lock() {
        ops.push(PendingOp::LayerMap {
            layer: normalized,
            key: key_code,
            action: parsed_action,
        });
    }
    Ok(())
}

fn register_layer_map(engine: &mut Engine, pending_ops: &PendingOps, layer_view: &LayerView) {
    __register_doc_layer_map_impl();
    let ops = Arc::clone(pending_ops);
    let view = Arc::clone(layer_view);
    engine.register_fn(
        "layer_map",
        move |layer_name: &str,
              key: &str,
              action: &str|
              -> std::result::Result<(), Box<EvalAltResult>> {
            layer_map_impl(layer_name, key, action, &ops, &view)
        },
    );
}

/// Push a layer onto the layer stack.
///
/// Activates a layer by pushing it onto the stack, making it the active layer.
/// Keys will be looked up in this layer first.
///
/// # Parameters
/// * `name` - The layer name to activate
///
/// # Returns
/// Result indicating success or error
///
/// # Examples
/// ```
/// layer_push("Nav");
/// layer_push("Symbols");
/// ```
#[rhai_doc(module = "layers")]
fn layer_push_impl(
    name: &str,
    ops: &PendingOps,
    view: &LayerView,
) -> std::result::Result<(), Box<EvalAltResult>> {
    let normalized = normalize_layer_name(name, "layer_push")?;
    with_layer_view(view, |stack| {
        let layer_id = stack.layer_id_by_name(&normalized).ok_or_else(|| {
            layer_error(
                "layer_push",
                format!("layer '{}' is not defined", normalized),
            )
        })?;

        let pushed = stack.push(layer_id);
        if !pushed {
            return Err(layer_error(
                "layer_push",
                "layer push failed (already top or base layer)",
            ));
        }
        Ok(())
    })?;

    if let Ok(mut ops) = ops.lock() {
        ops.push(PendingOp::LayerPush { name: normalized });
    }
    Ok(())
}

/// Pop the top layer from the layer stack.
///
/// Deactivates the currently active layer by popping it from the stack.
/// The next layer down becomes active.
///
/// # Returns
/// Result indicating success or error
///
/// # Examples
/// ```
/// layer_pop();
/// ```
#[rhai_doc(module = "layers")]
fn layer_pop_impl(
    ops: &PendingOps,
    view: &LayerView,
) -> std::result::Result<(), Box<EvalAltResult>> {
    with_layer_view(view, |stack| {
        let _ = stack.pop();
        Ok(())
    })?;

    if let Ok(mut ops) = ops.lock() {
        ops.push(PendingOp::LayerPop);
    }
    Ok(())
}

/// Toggle a layer on or off.
///
/// If the layer is active, deactivates it. If inactive, activates it.
/// Useful for toggling between two states.
///
/// # Parameters
/// * `name` - The layer name to toggle
///
/// # Returns
/// Result indicating success or error
///
/// # Examples
/// ```
/// layer_toggle("Nav");
/// layer_toggle("Symbols");
/// ```
#[rhai_doc(module = "layers")]
fn layer_toggle_impl(
    name: &str,
    ops: &PendingOps,
    view: &LayerView,
) -> std::result::Result<(), Box<EvalAltResult>> {
    let normalized = normalize_layer_name(name, "layer_toggle")?;
    with_layer_view(view, |stack| {
        let layer_id = stack.layer_id_by_name(&normalized).ok_or_else(|| {
            layer_error(
                "layer_toggle",
                format!("layer '{}' is not defined", normalized),
            )
        })?;

        let toggled = stack.toggle(layer_id);
        if !toggled {
            return Err(layer_error(
                "layer_toggle",
                "layer toggle failed (base layer cannot be toggled)",
            ));
        }
        Ok(())
    })?;

    if let Ok(mut ops) = ops.lock() {
        ops.push(PendingOp::LayerToggle { name: normalized });
    }
    Ok(())
}

/// Check if a layer is currently active.
///
/// Returns true if the specified layer is currently on the layer stack.
///
/// # Parameters
/// * `name` - The layer name to check
///
/// # Returns
/// Boolean indicating if the layer is active
///
/// # Examples
/// ```
/// if is_layer_active("Nav") {
///     print_debug("Nav layer is active");
/// }
/// ```
#[rhai_doc(module = "layers")]
fn is_layer_active_impl(
    name: &str,
    view: &LayerView,
) -> std::result::Result<bool, Box<EvalAltResult>> {
    let normalized = normalize_layer_name(name, "is_layer_active")?;
    let active = with_layer_view(view, |stack| Ok(stack.is_active_by_name(&normalized)))?;
    Ok(active)
}

fn register_layer_push(engine: &mut Engine, pending_ops: &PendingOps, layer_view: &LayerView) {
    __register_doc_layer_push_impl();
    let ops = Arc::clone(pending_ops);
    let view = Arc::clone(layer_view);
    engine.register_fn(
        "layer_push",
        move |name: &str| -> std::result::Result<(), Box<EvalAltResult>> {
            layer_push_impl(name, &ops, &view)
        },
    );
}

fn register_layer_pop(engine: &mut Engine, pending_ops: &PendingOps, layer_view: &LayerView) {
    __register_doc_layer_pop_impl();
    let ops = Arc::clone(pending_ops);
    let view = Arc::clone(layer_view);
    engine.register_fn(
        "layer_pop",
        move || -> std::result::Result<(), Box<EvalAltResult>> { layer_pop_impl(&ops, &view) },
    );
}

fn register_layer_toggle(engine: &mut Engine, pending_ops: &PendingOps, layer_view: &LayerView) {
    __register_doc_layer_toggle_impl();
    let ops = Arc::clone(pending_ops);
    let view = Arc::clone(layer_view);
    engine.register_fn(
        "layer_toggle",
        move |name: &str| -> std::result::Result<(), Box<EvalAltResult>> {
            layer_toggle_impl(name, &ops, &view)
        },
    );
}

fn register_is_layer_active(engine: &mut Engine, layer_view: &LayerView) {
    __register_doc_is_layer_active_impl();
    let view = Arc::clone(layer_view);
    engine.register_fn(
        "is_layer_active",
        move |name: &str| -> std::result::Result<bool, Box<EvalAltResult>> {
            is_layer_active_impl(name, &view)
        },
    );
}

fn register_modifier_functions(
    engine: &mut Engine,
    pending_ops: &PendingOps,
    modifier_view: &ModifierView,
) {
    register_define_modifier(engine, pending_ops, modifier_view);
    register_modifier_on(engine, pending_ops, modifier_view);
    register_modifier_off(engine, pending_ops, modifier_view);
    register_modifier_one_shot(engine, pending_ops, modifier_view);
    register_is_modifier_active(engine, modifier_view);
}

/// Define a virtual modifier.
///
/// Creates a custom modifier that can be activated and used in bindings.
/// Returns a unique modifier ID for use with tap_hold_mod and other functions.
///
/// # Parameters
/// * `name` - The modifier name (e.g., "Hyper", "Meh", "MyMod")
///
/// # Returns
/// The modifier ID (0-255) for use in bindings
///
/// # Examples
/// ```
/// let hyper = define_modifier("Hyper");
/// let meh = define_modifier("Meh");
/// ```
#[rhai_doc(module = "modifiers")]
fn define_modifier_impl(
    name: &str,
    ops: &PendingOps,
    view: &ModifierView,
) -> std::result::Result<i64, Box<EvalAltResult>> {
    let normalized = normalize_modifier_name(name, "define_modifier")?;
    let id = with_modifier_view(view, |preview| preview.define(&normalized))?;

    if let Ok(mut ops) = ops.lock() {
        ops.push(PendingOp::DefineModifier {
            name: normalized.clone(),
            id,
        });
    }

    Ok(i64::from(id))
}

/// Activate a virtual modifier.
///
/// Turns on the specified modifier, affecting subsequent key presses.
/// The modifier stays active until explicitly deactivated.
///
/// # Parameters
/// * `name` - The modifier name to activate
///
/// # Returns
/// Result indicating success or error
///
/// # Examples
/// ```
/// modifier_on("Hyper");
/// modifier_on("Meh");
/// ```
#[rhai_doc(module = "modifiers")]
fn modifier_on_impl(
    name: &str,
    ops: &PendingOps,
    view: &ModifierView,
) -> std::result::Result<(), Box<EvalAltResult>> {
    let normalized = normalize_modifier_name(name, "modifier_on")?;
    let id = with_modifier_view(view, |preview| {
        let id = preview.id_for(&normalized, "modifier_on")?;
        preview.activate(id);
        Ok(id)
    })?;

    if let Ok(mut ops) = ops.lock() {
        ops.push(PendingOp::ModifierActivate {
            name: normalized,
            id,
        });
    }

    Ok(())
}

/// Deactivate a virtual modifier.
///
/// Turns off the specified modifier.
///
/// # Parameters
/// * `name` - The modifier name to deactivate
///
/// # Returns
/// Result indicating success or error
///
/// # Examples
/// ```
/// modifier_off("Hyper");
/// modifier_off("Meh");
/// ```
#[rhai_doc(module = "modifiers")]
fn modifier_off_impl(
    name: &str,
    ops: &PendingOps,
    view: &ModifierView,
) -> std::result::Result<(), Box<EvalAltResult>> {
    let normalized = normalize_modifier_name(name, "modifier_off")?;
    let id = with_modifier_view(view, |preview| {
        let id = preview.id_for(&normalized, "modifier_off")?;
        preview.deactivate(id);
        Ok(id)
    })?;

    if let Ok(mut ops) = ops.lock() {
        ops.push(PendingOp::ModifierDeactivate {
            name: normalized,
            id,
        });
    }

    Ok(())
}

/// Activate a modifier for one keypress only.
///
/// The modifier activates for the next key press and then auto-deactivates.
/// Useful for sticky modifier keys.
///
/// # Parameters
/// * `name` - The modifier name to activate as one-shot
///
/// # Returns
/// Result indicating success or error
///
/// # Examples
/// ```
/// one_shot("Shift");
/// one_shot("Ctrl");
/// ```
#[rhai_doc(module = "modifiers")]
fn one_shot_impl(
    name: &str,
    ops: &PendingOps,
    view: &ModifierView,
) -> std::result::Result<(), Box<EvalAltResult>> {
    let normalized = normalize_modifier_name(name, "one_shot")?;
    let id = with_modifier_view(view, |preview| {
        let id = preview.id_for(&normalized, "one_shot")?;
        preview.one_shot(id);
        Ok(id)
    })?;

    if let Ok(mut ops) = ops.lock() {
        ops.push(PendingOp::ModifierOneShot {
            name: normalized,
            id,
        });
    }

    Ok(())
}

/// Check if a modifier is currently active.
///
/// Returns true if the specified modifier is currently activated.
///
/// # Parameters
/// * `name` - The modifier name to check
///
/// # Returns
/// Boolean indicating if the modifier is active
///
/// # Examples
/// ```
/// if is_modifier_active("Hyper") {
///     print_debug("Hyper is active");
/// }
/// ```
#[rhai_doc(module = "modifiers")]
fn is_modifier_active_impl(
    name: &str,
    view: &ModifierView,
) -> std::result::Result<bool, Box<EvalAltResult>> {
    let normalized = normalize_modifier_name(name, "is_modifier_active")?;
    let active = with_modifier_view(view, |preview| {
        let id = preview.id_for(&normalized, "is_modifier_active")?;
        Ok(preview.is_active(id))
    })?;
    Ok(active)
}

fn register_define_modifier(
    engine: &mut Engine,
    pending_ops: &PendingOps,
    modifier_view: &ModifierView,
) {
    __register_doc_define_modifier_impl();
    let ops = Arc::clone(pending_ops);
    let view = Arc::clone(modifier_view);
    engine.register_fn(
        "define_modifier",
        move |name: &str| -> std::result::Result<i64, Box<EvalAltResult>> {
            define_modifier_impl(name, &ops, &view)
        },
    );
}

fn register_modifier_on(
    engine: &mut Engine,
    pending_ops: &PendingOps,
    modifier_view: &ModifierView,
) {
    __register_doc_modifier_on_impl();
    let ops = Arc::clone(pending_ops);
    let view = Arc::clone(modifier_view);
    engine.register_fn(
        "modifier_on",
        move |name: &str| -> std::result::Result<(), Box<EvalAltResult>> {
            modifier_on_impl(name, &ops, &view)
        },
    );
}

fn register_modifier_off(
    engine: &mut Engine,
    pending_ops: &PendingOps,
    modifier_view: &ModifierView,
) {
    __register_doc_modifier_off_impl();
    let ops = Arc::clone(pending_ops);
    let view = Arc::clone(modifier_view);
    engine.register_fn(
        "modifier_off",
        move |name: &str| -> std::result::Result<(), Box<EvalAltResult>> {
            modifier_off_impl(name, &ops, &view)
        },
    );
}

fn register_modifier_one_shot(
    engine: &mut Engine,
    pending_ops: &PendingOps,
    modifier_view: &ModifierView,
) {
    __register_doc_one_shot_impl();
    let ops = Arc::clone(pending_ops);
    let view = Arc::clone(modifier_view);
    engine.register_fn(
        "one_shot",
        move |name: &str| -> std::result::Result<(), Box<EvalAltResult>> {
            one_shot_impl(name, &ops, &view)
        },
    );
}

fn register_is_modifier_active(engine: &mut Engine, modifier_view: &ModifierView) {
    __register_doc_is_modifier_active_impl();
    let view = Arc::clone(modifier_view);
    engine.register_fn(
        "is_modifier_active",
        move |name: &str| -> std::result::Result<bool, Box<EvalAltResult>> {
            is_modifier_active_impl(name, &view)
        },
    );
}

fn register_timing_functions(engine: &mut Engine, pending_ops: &PendingOps) {
    register_set_tap_timeout(engine, pending_ops);
    register_set_combo_timeout(engine, pending_ops);
    register_set_hold_delay(engine, pending_ops);
    register_set_eager_tap(engine, pending_ops);
    register_set_permissive_hold(engine, pending_ops);
    register_set_retro_tap(engine, pending_ops);
}

/// Set the tap timeout duration for tap-hold keys.
///
/// Controls how long a key can be held before it's considered a "hold" instead of a "tap".
///
/// # Parameters
/// * `ms` - Timeout in milliseconds (typically 100-300ms)
///
/// # Returns
/// Result indicating success or error
///
/// # Examples
/// ```
/// set_tap_timeout(200);
/// set_tap_timeout(150);
/// ```
///
/// # Notes
/// Default is 200ms. Lower values make holds trigger faster but may cause accidental holds.
#[rhai_doc(module = "timing")]
fn set_tap_timeout_impl(ms: i64, ops: &PendingOps) -> std::result::Result<(), Box<EvalAltResult>> {
    let value = validate_timeout(ms, "set_tap_timeout", false)?;
    if let Ok(mut ops) = ops.lock() {
        ops.push(PendingOp::SetTiming(TimingUpdate::TapTimeout(value)));
    }
    Ok(())
}

/// Set the combo timeout duration.
///
/// Controls how quickly keys must be pressed together to form a combo.
///
/// # Parameters
/// * `ms` - Timeout in milliseconds (typically 30-100ms)
///
/// # Returns
/// Result indicating success or error
///
/// # Examples
/// ```
/// set_combo_timeout(50);
/// set_combo_timeout(30);
/// ```
///
/// # Notes
/// Default is 50ms. Lower values require faster simultaneous presses.
#[rhai_doc(module = "timing")]
fn set_combo_timeout_impl(
    ms: i64,
    ops: &PendingOps,
) -> std::result::Result<(), Box<EvalAltResult>> {
    let value = validate_timeout(ms, "set_combo_timeout", false)?;
    if let Ok(mut ops) = ops.lock() {
        ops.push(PendingOp::SetTiming(TimingUpdate::ComboTimeout(value)));
    }
    Ok(())
}

/// Set the hold delay before a hold action is triggered.
///
/// Adds a delay before the hold action activates, allowing faster taps.
///
/// # Parameters
/// * `ms` - Delay in milliseconds (0 disables, typically 0-200ms)
///
/// # Returns
/// Result indicating success or error
///
/// # Examples
/// ```
/// set_hold_delay(100);
/// set_hold_delay(0);
/// ```
///
/// # Notes
/// Default is 0 (no delay). Higher values prevent accidental holds but delay activation.
#[rhai_doc(module = "timing")]
fn set_hold_delay_impl(ms: i64, ops: &PendingOps) -> std::result::Result<(), Box<EvalAltResult>> {
    let value = validate_timeout(ms, "set_hold_delay", true)?;
    if let Ok(mut ops) = ops.lock() {
        ops.push(PendingOp::SetTiming(TimingUpdate::HoldDelay(value)));
    }
    Ok(())
}

/// Enable or disable eager tap mode.
///
/// When enabled, tap is sent immediately on key press rather than waiting for release.
/// Improves responsiveness but may affect hold detection.
///
/// # Parameters
/// * `enabled` - true to enable, false to disable
///
/// # Returns
/// Result indicating success or error
///
/// # Examples
/// ```
/// set_eager_tap(true);
/// set_eager_tap(false);
/// ```
///
/// # Notes
/// Default is false. Enabling improves typing speed but may cause issues with fast hold activation.
#[rhai_doc(module = "timing")]
fn set_eager_tap_impl(
    enabled: bool,
    ops: &PendingOps,
) -> std::result::Result<(), Box<EvalAltResult>> {
    if let Ok(mut ops) = ops.lock() {
        ops.push(PendingOp::SetTiming(TimingUpdate::EagerTap(enabled)));
    }
    Ok(())
}

/// Enable or disable permissive hold mode.
///
/// When enabled, any other key press during hold time activates the hold action.
/// Makes holds trigger more reliably during fast typing.
///
/// # Parameters
/// * `enabled` - true to enable, false to disable
///
/// # Returns
/// Result indicating success or error
///
/// # Examples
/// ```
/// set_permissive_hold(true);
/// set_permissive_hold(false);
/// ```
///
/// # Notes
/// Default is false. Useful for home-row mods to prevent accidental holds.
#[rhai_doc(module = "timing")]
fn set_permissive_hold_impl(
    enabled: bool,
    ops: &PendingOps,
) -> std::result::Result<(), Box<EvalAltResult>> {
    if let Ok(mut ops) = ops.lock() {
        ops.push(PendingOp::SetTiming(TimingUpdate::PermissiveHold(enabled)));
    }
    Ok(())
}

/// Enable or disable retro tap mode.
///
/// When enabled, releasing a tap-hold key without pressing other keys sends the tap action,
/// even if held longer than tap timeout.
///
/// # Parameters
/// * `enabled` - true to enable, false to disable
///
/// # Returns
/// Result indicating success or error
///
/// # Examples
/// ```
/// set_retro_tap(true);
/// set_retro_tap(false);
/// ```
///
/// # Notes
/// Default is false. Useful for modifier keys that should still type when pressed alone.
#[rhai_doc(module = "timing")]
fn set_retro_tap_impl(
    enabled: bool,
    ops: &PendingOps,
) -> std::result::Result<(), Box<EvalAltResult>> {
    if let Ok(mut ops) = ops.lock() {
        ops.push(PendingOp::SetTiming(TimingUpdate::RetroTap(enabled)));
    }
    Ok(())
}

fn register_set_tap_timeout(engine: &mut Engine, pending_ops: &PendingOps) {
    __register_doc_set_tap_timeout_impl();
    let ops = Arc::clone(pending_ops);
    engine.register_fn(
        "set_tap_timeout",
        move |ms: i64| -> std::result::Result<(), Box<EvalAltResult>> {
            set_tap_timeout_impl(ms, &ops)
        },
    );
}

fn register_set_combo_timeout(engine: &mut Engine, pending_ops: &PendingOps) {
    __register_doc_set_combo_timeout_impl();
    let ops = Arc::clone(pending_ops);
    engine.register_fn(
        "set_combo_timeout",
        move |ms: i64| -> std::result::Result<(), Box<EvalAltResult>> {
            set_combo_timeout_impl(ms, &ops)
        },
    );
}

fn register_set_hold_delay(engine: &mut Engine, pending_ops: &PendingOps) {
    __register_doc_set_hold_delay_impl();
    let ops = Arc::clone(pending_ops);
    engine.register_fn(
        "set_hold_delay",
        move |ms: i64| -> std::result::Result<(), Box<EvalAltResult>> {
            set_hold_delay_impl(ms, &ops)
        },
    );
}

fn register_set_eager_tap(engine: &mut Engine, pending_ops: &PendingOps) {
    __register_doc_set_eager_tap_impl();
    let ops = Arc::clone(pending_ops);
    engine.register_fn(
        "set_eager_tap",
        move |enabled: bool| -> std::result::Result<(), Box<EvalAltResult>> {
            set_eager_tap_impl(enabled, &ops)
        },
    );
}

fn register_set_permissive_hold(engine: &mut Engine, pending_ops: &PendingOps) {
    __register_doc_set_permissive_hold_impl();
    let ops = Arc::clone(pending_ops);
    engine.register_fn(
        "set_permissive_hold",
        move |enabled: bool| -> std::result::Result<(), Box<EvalAltResult>> {
            set_permissive_hold_impl(enabled, &ops)
        },
    );
}

fn register_set_retro_tap(engine: &mut Engine, pending_ops: &PendingOps) {
    __register_doc_set_retro_tap_impl();
    let ops = Arc::clone(pending_ops);
    engine.register_fn(
        "set_retro_tap",
        move |enabled: bool| -> std::result::Result<(), Box<EvalAltResult>> {
            set_retro_tap_impl(enabled, &ops)
        },
    );
}
