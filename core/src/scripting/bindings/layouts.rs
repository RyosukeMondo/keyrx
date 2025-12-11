//! Layout composition functions.
//!
//! Functions: layout_define, layout_enable, layout_disable, layout_remove, layout_set_priority

use crate::scripting::builtins::{PendingOp, PendingOps};
use crate::scripting::sandbox::validation::InputValidator;
use crate::scripting::sandbox::validators::RangeValidator;
use keyrx_core_macros::rhai_doc;
use rhai::{Engine, EvalAltResult, Position};
use std::sync::Arc;

pub fn register_layout_functions(engine: &mut Engine, pending_ops: &PendingOps) {
    register_layout_define(engine, pending_ops);
    register_layout_enable(engine, pending_ops);
    register_layout_disable(engine, pending_ops);
    register_layout_remove(engine, pending_ops);
    register_layout_priority(engine, pending_ops);
}

/// Define or update a layout with priority for composition.
///
/// Creates a layout entry that can participate in the compositor alongside the
/// default layout. Layout IDs must be unique.
///
/// # Parameters
/// * `id` - Stable layout identifier (e.g., "coding", "gaming")
/// * `name` - Human-friendly name
/// * `priority` - Priority for conflict resolution (higher wins; ties use recency)
#[rhai_doc(module = "layouts")]
fn layout_define_impl(
    id: &str,
    name: &str,
    priority: i64,
    ops: &PendingOps,
) -> std::result::Result<(), Box<EvalAltResult>> {
    let normalized_id = normalize_layout_id(id, "layout_define")?;
    let normalized_name = normalize_layout_name(name, "layout_define")?;
    let priority = normalize_priority(priority, "layout_define")?;

    if let Ok(mut ops) = ops.lock() {
        ops.push(PendingOp::LayoutDefine {
            id: normalized_id,
            name: normalized_name,
            priority,
        });
    }

    Ok(())
}

fn register_layout_define(engine: &mut Engine, pending_ops: &PendingOps) {
    __register_doc_layout_define_impl();
    let ops = Arc::clone(pending_ops);
    engine.register_fn(
        "layout_define",
        move |id: &str, name: &str, priority: i64| -> std::result::Result<(), Box<EvalAltResult>> {
            layout_define_impl(id, name, priority, &ops)
        },
    );
}

/// Enable a layout for composition.
///
/// # Parameters
/// * `id` - Layout identifier to enable
#[rhai_doc(module = "layouts")]
fn layout_enable_impl(id: &str, ops: &PendingOps) -> std::result::Result<(), Box<EvalAltResult>> {
    let normalized_id = normalize_layout_id(id, "layout_enable")?;
    if let Ok(mut ops) = ops.lock() {
        ops.push(PendingOp::LayoutEnable { id: normalized_id });
    }
    Ok(())
}

fn register_layout_enable(engine: &mut Engine, pending_ops: &PendingOps) {
    __register_doc_layout_enable_impl();
    let ops = Arc::clone(pending_ops);
    engine.register_fn(
        "layout_enable",
        move |id: &str| -> std::result::Result<(), Box<EvalAltResult>> {
            layout_enable_impl(id, &ops)
        },
    );
}

/// Disable a layout from composition.
///
/// # Parameters
/// * `id` - Layout identifier to disable
#[rhai_doc(module = "layouts")]
fn layout_disable_impl(id: &str, ops: &PendingOps) -> std::result::Result<(), Box<EvalAltResult>> {
    let normalized_id = normalize_layout_id(id, "layout_disable")?;
    if let Ok(mut ops) = ops.lock() {
        ops.push(PendingOp::LayoutDisable { id: normalized_id });
    }
    Ok(())
}

fn register_layout_disable(engine: &mut Engine, pending_ops: &PendingOps) {
    __register_doc_layout_disable_impl();
    let ops = Arc::clone(pending_ops);
    engine.register_fn(
        "layout_disable",
        move |id: &str| -> std::result::Result<(), Box<EvalAltResult>> {
            layout_disable_impl(id, &ops)
        },
    );
}

/// Remove a layout from the compositor (default layout is preserved).
///
/// # Parameters
/// * `id` - Layout identifier to remove
#[rhai_doc(module = "layouts")]
fn layout_remove_impl(id: &str, ops: &PendingOps) -> std::result::Result<(), Box<EvalAltResult>> {
    let normalized_id = normalize_layout_id(id, "layout_remove")?;
    if let Ok(mut ops) = ops.lock() {
        ops.push(PendingOp::LayoutRemove { id: normalized_id });
    }
    Ok(())
}

fn register_layout_remove(engine: &mut Engine, pending_ops: &PendingOps) {
    __register_doc_layout_remove_impl();
    let ops = Arc::clone(pending_ops);
    engine.register_fn(
        "layout_remove",
        move |id: &str| -> std::result::Result<(), Box<EvalAltResult>> {
            layout_remove_impl(id, &ops)
        },
    );
}

/// Update layout priority (higher wins).
///
/// # Parameters
/// * `id` - Layout identifier
/// * `priority` - New priority (higher = higher precedence)
#[rhai_doc(module = "layouts")]
fn layout_set_priority_impl(
    id: &str,
    priority: i64,
    ops: &PendingOps,
) -> std::result::Result<(), Box<EvalAltResult>> {
    let normalized_id = normalize_layout_id(id, "layout_set_priority")?;
    let priority = normalize_priority(priority, "layout_set_priority")?;
    if let Ok(mut ops) = ops.lock() {
        ops.push(PendingOp::LayoutSetPriority {
            id: normalized_id,
            priority,
        });
    }
    Ok(())
}

fn register_layout_priority(engine: &mut Engine, pending_ops: &PendingOps) {
    __register_doc_layout_set_priority_impl();
    let ops = Arc::clone(pending_ops);
    engine.register_fn(
        "layout_set_priority",
        move |id: &str, priority: i64| -> std::result::Result<(), Box<EvalAltResult>> {
            layout_set_priority_impl(id, priority, &ops)
        },
    );
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn layout_error(fn_name: &str, message: impl Into<String>) -> Box<EvalAltResult> {
    Box::new(EvalAltResult::ErrorRuntime(
        format!("{}: {}", fn_name, message.into()).into(),
        Position::NONE,
    ))
}

fn normalize_layout_id(id: &str, fn_name: &str) -> Result<String, Box<EvalAltResult>> {
    let trimmed = id.trim();
    if trimmed.is_empty() {
        return Err(layout_error(fn_name, "layout id cannot be empty"));
    }
    if trimmed.contains(':') {
        return Err(layout_error(
            fn_name,
            "layout id cannot contain ':' characters",
        ));
    }
    Ok(trimmed.to_string())
}

fn normalize_layout_name(name: &str, fn_name: &str) -> Result<String, Box<EvalAltResult>> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(layout_error(fn_name, "layout name cannot be empty"));
    }
    Ok(trimmed.to_string())
}

fn normalize_priority(priority: i64, fn_name: &str) -> Result<i32, Box<EvalAltResult>> {
    let validator = RangeValidator::new(i32::MIN as i64, i32::MAX as i64);
    validator.validate(&priority).map_err(|e| {
        Box::new(EvalAltResult::ErrorRuntime(
            format!("{}: priority {}", fn_name, e).into(),
            Position::NONE,
        ))
    })?;
    Ok(priority as i32)
}
