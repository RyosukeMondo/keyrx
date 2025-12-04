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

fn register_debug(engine: &mut Engine) {
    engine.register_fn("print_debug", |msg: &str| {
        tracing::debug!(
            service = "keyrx",
            event = "script_debug",
            component = "scripting_runtime",
            script_message = msg,
            "Script debug output"
        );
    });
}

fn register_remap(engine: &mut Engine, pending_ops: &PendingOps) {
    let ops = Arc::clone(pending_ops);
    engine.register_fn(
        "remap",
        move |from: &str, to: &str| -> std::result::Result<(), Box<EvalAltResult>> {
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
        },
    );
}

fn register_block(engine: &mut Engine, pending_ops: &PendingOps) {
    let ops = Arc::clone(pending_ops);
    engine.register_fn(
        "block",
        move |key: &str| -> std::result::Result<(), Box<EvalAltResult>> {
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
        },
    );
}

fn register_pass(engine: &mut Engine, pending_ops: &PendingOps) {
    let ops = Arc::clone(pending_ops);
    engine.register_fn(
        "pass",
        move |key: &str| -> std::result::Result<(), Box<EvalAltResult>> {
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
        },
    );
}

fn register_tap_hold(engine: &mut Engine, pending_ops: &PendingOps) {
    let ops = Arc::clone(pending_ops);
    engine.register_fn(
        "tap_hold",
        move |key: &str, tap: &str, hold: &str| -> std::result::Result<(), Box<EvalAltResult>> {
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
        },
    );
}

fn register_tap_hold_mod(engine: &mut Engine, pending_ops: &PendingOps) {
    let ops = Arc::clone(pending_ops);
    engine.register_fn(
        "tap_hold_mod",
        move |key: &str,
              tap: &str,
              modifier_id: i64|
              -> std::result::Result<(), Box<EvalAltResult>> {
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
        },
    );
}

fn register_combo(engine: &mut Engine, pending_ops: &PendingOps) {
    let ops = Arc::clone(pending_ops);
    engine.register_fn(
        "combo",
        move |keys: Array, action_key: &str| -> std::result::Result<(), Box<EvalAltResult>> {
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

fn register_layer_define(engine: &mut Engine, pending_ops: &PendingOps, layer_view: &LayerView) {
    let ops = Arc::clone(pending_ops);
    let view = Arc::clone(layer_view);
    engine.register_fn(
        "layer_define",
        move |name: &str, transparent: bool| -> std::result::Result<(), Box<EvalAltResult>> {
            let normalized = normalize_layer_name(name, "layer_define")?;
            with_layer_view(&view, |stack| {
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
        },
    );
}

fn register_layer_map(engine: &mut Engine, pending_ops: &PendingOps, layer_view: &LayerView) {
    let ops = Arc::clone(pending_ops);
    let view = Arc::clone(layer_view);
    engine.register_fn(
        "layer_map",
        move |layer_name: &str,
              key: &str,
              action: &str|
              -> std::result::Result<(), Box<EvalAltResult>> {
            let normalized = normalize_layer_name(layer_name, "layer_map")?;
            let key_code = parse_key_or_error(key, "layer_map")?;
            let parsed_action = parse_layer_action(action, "layer_map", &view)?;

            // Require layer to be defined in the preview state to give early feedback.
            with_layer_view(&view, |stack| {
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
        },
    );
}

fn register_layer_push(engine: &mut Engine, pending_ops: &PendingOps, layer_view: &LayerView) {
    let ops = Arc::clone(pending_ops);
    let view = Arc::clone(layer_view);
    engine.register_fn(
        "layer_push",
        move |name: &str| -> std::result::Result<(), Box<EvalAltResult>> {
            let normalized = normalize_layer_name(name, "layer_push")?;
            with_layer_view(&view, |stack| {
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
        },
    );
}

fn register_layer_pop(engine: &mut Engine, pending_ops: &PendingOps, layer_view: &LayerView) {
    let ops = Arc::clone(pending_ops);
    let view = Arc::clone(layer_view);
    engine.register_fn(
        "layer_pop",
        move || -> std::result::Result<(), Box<EvalAltResult>> {
            with_layer_view(&view, |stack| {
                let _ = stack.pop();
                Ok(())
            })?;

            if let Ok(mut ops) = ops.lock() {
                ops.push(PendingOp::LayerPop);
            }
            Ok(())
        },
    );
}

fn register_layer_toggle(engine: &mut Engine, pending_ops: &PendingOps, layer_view: &LayerView) {
    let ops = Arc::clone(pending_ops);
    let view = Arc::clone(layer_view);
    engine.register_fn(
        "layer_toggle",
        move |name: &str| -> std::result::Result<(), Box<EvalAltResult>> {
            let normalized = normalize_layer_name(name, "layer_toggle")?;
            with_layer_view(&view, |stack| {
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
        },
    );
}

fn register_is_layer_active(engine: &mut Engine, layer_view: &LayerView) {
    let view = Arc::clone(layer_view);
    engine.register_fn(
        "is_layer_active",
        move |name: &str| -> std::result::Result<bool, Box<EvalAltResult>> {
            let normalized = normalize_layer_name(name, "is_layer_active")?;
            let active = with_layer_view(&view, |stack| Ok(stack.is_active_by_name(&normalized)))?;
            Ok(active)
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

fn register_define_modifier(
    engine: &mut Engine,
    pending_ops: &PendingOps,
    modifier_view: &ModifierView,
) {
    let ops = Arc::clone(pending_ops);
    let view = Arc::clone(modifier_view);
    engine.register_fn(
        "define_modifier",
        move |name: &str| -> std::result::Result<i64, Box<EvalAltResult>> {
            let normalized = normalize_modifier_name(name, "define_modifier")?;
            let id = with_modifier_view(&view, |preview| preview.define(&normalized))?;

            if let Ok(mut ops) = ops.lock() {
                ops.push(PendingOp::DefineModifier {
                    name: normalized.clone(),
                    id,
                });
            }

            Ok(i64::from(id))
        },
    );
}

fn register_modifier_on(
    engine: &mut Engine,
    pending_ops: &PendingOps,
    modifier_view: &ModifierView,
) {
    let ops = Arc::clone(pending_ops);
    let view = Arc::clone(modifier_view);
    engine.register_fn(
        "modifier_on",
        move |name: &str| -> std::result::Result<(), Box<EvalAltResult>> {
            let normalized = normalize_modifier_name(name, "modifier_on")?;
            let id = with_modifier_view(&view, |preview| {
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
        },
    );
}

fn register_modifier_off(
    engine: &mut Engine,
    pending_ops: &PendingOps,
    modifier_view: &ModifierView,
) {
    let ops = Arc::clone(pending_ops);
    let view = Arc::clone(modifier_view);
    engine.register_fn(
        "modifier_off",
        move |name: &str| -> std::result::Result<(), Box<EvalAltResult>> {
            let normalized = normalize_modifier_name(name, "modifier_off")?;
            let id = with_modifier_view(&view, |preview| {
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
        },
    );
}

fn register_modifier_one_shot(
    engine: &mut Engine,
    pending_ops: &PendingOps,
    modifier_view: &ModifierView,
) {
    let ops = Arc::clone(pending_ops);
    let view = Arc::clone(modifier_view);
    engine.register_fn(
        "one_shot",
        move |name: &str| -> std::result::Result<(), Box<EvalAltResult>> {
            let normalized = normalize_modifier_name(name, "one_shot")?;
            let id = with_modifier_view(&view, |preview| {
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
        },
    );
}

fn register_is_modifier_active(engine: &mut Engine, modifier_view: &ModifierView) {
    let view = Arc::clone(modifier_view);
    engine.register_fn(
        "is_modifier_active",
        move |name: &str| -> std::result::Result<bool, Box<EvalAltResult>> {
            let normalized = normalize_modifier_name(name, "is_modifier_active")?;
            let active = with_modifier_view(&view, |preview| {
                let id = preview.id_for(&normalized, "is_modifier_active")?;
                Ok(preview.is_active(id))
            })?;
            Ok(active)
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

fn register_set_tap_timeout(engine: &mut Engine, pending_ops: &PendingOps) {
    let ops = Arc::clone(pending_ops);
    engine.register_fn(
        "set_tap_timeout",
        move |ms: i64| -> std::result::Result<(), Box<EvalAltResult>> {
            let value = validate_timeout(ms, "set_tap_timeout", false)?;
            if let Ok(mut ops) = ops.lock() {
                ops.push(PendingOp::SetTiming(TimingUpdate::TapTimeout(value)));
            }
            Ok(())
        },
    );
}

fn register_set_combo_timeout(engine: &mut Engine, pending_ops: &PendingOps) {
    let ops = Arc::clone(pending_ops);
    engine.register_fn(
        "set_combo_timeout",
        move |ms: i64| -> std::result::Result<(), Box<EvalAltResult>> {
            let value = validate_timeout(ms, "set_combo_timeout", false)?;
            if let Ok(mut ops) = ops.lock() {
                ops.push(PendingOp::SetTiming(TimingUpdate::ComboTimeout(value)));
            }
            Ok(())
        },
    );
}

fn register_set_hold_delay(engine: &mut Engine, pending_ops: &PendingOps) {
    let ops = Arc::clone(pending_ops);
    engine.register_fn(
        "set_hold_delay",
        move |ms: i64| -> std::result::Result<(), Box<EvalAltResult>> {
            let value = validate_timeout(ms, "set_hold_delay", true)?;
            if let Ok(mut ops) = ops.lock() {
                ops.push(PendingOp::SetTiming(TimingUpdate::HoldDelay(value)));
            }
            Ok(())
        },
    );
}

fn register_set_eager_tap(engine: &mut Engine, pending_ops: &PendingOps) {
    let ops = Arc::clone(pending_ops);
    engine.register_fn(
        "set_eager_tap",
        move |enabled: bool| -> std::result::Result<(), Box<EvalAltResult>> {
            if let Ok(mut ops) = ops.lock() {
                ops.push(PendingOp::SetTiming(TimingUpdate::EagerTap(enabled)));
            }
            Ok(())
        },
    );
}

fn register_set_permissive_hold(engine: &mut Engine, pending_ops: &PendingOps) {
    let ops = Arc::clone(pending_ops);
    engine.register_fn(
        "set_permissive_hold",
        move |enabled: bool| -> std::result::Result<(), Box<EvalAltResult>> {
            if let Ok(mut ops) = ops.lock() {
                ops.push(PendingOp::SetTiming(TimingUpdate::PermissiveHold(enabled)));
            }
            Ok(())
        },
    );
}

fn register_set_retro_tap(engine: &mut Engine, pending_ops: &PendingOps) {
    let ops = Arc::clone(pending_ops);
    engine.register_fn(
        "set_retro_tap",
        move |enabled: bool| -> std::result::Result<(), Box<EvalAltResult>> {
            if let Ok(mut ops) = ops.lock() {
                ops.push(PendingOp::SetTiming(TimingUpdate::RetroTap(enabled)));
            }
            Ok(())
        },
    );
}
