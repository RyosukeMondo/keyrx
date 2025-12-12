//! Basic key remapping functions: remap, block, pass.

use crate::scripting::builtins::{PendingOp, PendingOps};
use crate::scripting::helpers::parse_key_or_error;
use keyrx_core_macros::rhai_doc;
use rhai::{Engine, EvalAltResult};
use std::sync::Arc;

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
/// remap("F12", "F1");
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

pub fn register_remap(engine: &mut Engine, pending_ops: &PendingOps) {
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

pub fn register_block(engine: &mut Engine, pending_ops: &PendingOps) {
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

pub fn register_pass(engine: &mut Engine, pending_ops: &PendingOps) {
    __register_doc_pass_impl();
    let ops = Arc::clone(pending_ops);
    engine.register_fn(
        "pass",
        move |key: &str| -> std::result::Result<(), Box<EvalAltResult>> { pass_impl(key, &ops) },
    );
}
