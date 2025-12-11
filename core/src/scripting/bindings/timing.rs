//! Timing configuration functions.
//!
//! Functions: set_tap_timeout, set_combo_timeout, set_hold_delay,
//! set_eager_tap, set_permissive_hold, set_retro_tap

use crate::scripting::builtins::{validate_timeout, PendingOp, PendingOps, TimingUpdate};
use keyrx_core_macros::rhai_doc;
use rhai::{Engine, EvalAltResult};
use std::sync::Arc;

pub fn register_timing_functions(engine: &mut Engine, pending_ops: &PendingOps) {
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
