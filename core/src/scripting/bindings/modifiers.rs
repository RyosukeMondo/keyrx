//! Virtual modifier functions: define_modifier, modifier_on, modifier_off, one_shot, is_modifier_active.

use crate::scripting::builtins::{
    normalize_modifier_name, with_modifier_view, ModifierView, PendingOp, PendingOps,
};
use keyrx_core_macros::rhai_doc;
use rhai::{Engine, EvalAltResult};
use std::sync::Arc;

pub fn register_modifier_functions(
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
