//! Tap-hold and combo functions: tap_hold, tap_hold_mod, combo.

use crate::engine::{HoldAction, LayerAction, VirtualModifiers};
use crate::scripting::builtins::{parse_keys_array, PendingOp, PendingOps};
use crate::scripting::helpers::parse_key_or_error;
use crate::scripting::sandbox::validation::InputValidator;
use crate::scripting::sandbox::validators::RangeValidator;
use keyrx_core_macros::rhai_doc;
use rhai::{Array, Engine, EvalAltResult, Position};
use std::sync::Arc;

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

pub fn register_tap_hold(engine: &mut Engine, pending_ops: &PendingOps) {
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

pub fn register_tap_hold_mod(engine: &mut Engine, pending_ops: &PendingOps) {
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

pub fn register_combo(engine: &mut Engine, pending_ops: &PendingOps) {
    __register_doc_combo_impl();
    let ops = Arc::clone(pending_ops);
    engine.register_fn(
        "combo",
        move |keys: Array, action_key: &str| -> std::result::Result<(), Box<EvalAltResult>> {
            combo_impl(keys, action_key, &ops)
        },
    );
}
