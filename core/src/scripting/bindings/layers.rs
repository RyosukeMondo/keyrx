//! Layer functions: layer_define, layer_map, layer_push, layer_pop, layer_toggle, is_layer_active.

use crate::scripting::builtins::{
    layer_error, normalize_layer_name, parse_layer_action, with_layer_view, LayerView, PendingOp,
    PendingOps,
};
use crate::scripting::helpers::parse_key_or_error;
use keyrx_core_macros::rhai_doc;
use rhai::{Engine, EvalAltResult};
use std::sync::Arc;

pub fn register_layer_functions(
    engine: &mut Engine,
    pending_ops: &PendingOps,
    layer_view: &LayerView,
) {
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
/// layer_define("Nav", true);
/// layer_map("Nav", "H", "Left");
/// layer_map("Nav", "J", "Down");
/// layer_map("Nav", "K", "Up");
/// layer_map("Nav", "L", "Right");
/// layer_define("Symbols", false);
/// layer_map("Nav", "Space", "layer_push:Symbols");
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
/// layer_define("Nav", true);
/// layer_define("Symbols", false);
/// layer_push("Nav");
/// layer_push("Symbols");
/// layer_pop();
/// layer_pop();
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
/// layer_define("Nav", true);
/// layer_define("Symbols", false);
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
/// layer_define("Nav", true);
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
