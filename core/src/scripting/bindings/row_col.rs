//! Row-column API functions for position-based remapping.
//!
//! These functions provide position-based remapping using (row, col) coordinates
//! instead of key names. Useful for layout-agnostic configurations and custom
//! devices like stream decks.
//!
//! Functions: remap_rc, tap_hold_rc, block_rc, combo_rc, layer_map_rc

use crate::engine::{HoldAction, LayerAction};
use crate::scripting::builtins::{
    layer_error, with_layer_view, LayerMapAction, LayerView, PendingOp, PendingOps,
};
use crate::scripting::helpers::parse_key_or_error;
use crate::scripting::row_col_resolver::RowColResolver;
use keyrx_core_macros::rhai_doc;
use rhai::{Array, Engine, EvalAltResult, Position};
use std::sync::Arc;

/// Remap a key by its physical position (row, column).
///
/// Maps a physical key position to a target key without needing to know the key name.
/// This enables layout-agnostic configurations that work across QWERTY, Dvorak, Colemak, etc.
///
/// # Parameters
/// * `row` - 0-based row number from device profile
/// * `col` - 0-based column number from device profile
/// * `to` - Target key name (same as regular `remap`)
///
/// # Returns
/// Result indicating success or error
///
/// # Examples
/// ```
/// // remap_rc(3, 0, "Escape");     // Home row, 1st key (CapsLock) → Escape
/// // remap_rc(3, 1, "B");          // Home row, 2nd key (A on QWERTY) → B
/// // remap_rc(0, 16, "Delete");    // Top row, last key → Delete
/// ```
///
/// # Notes
/// - Requires device profile to be loaded
/// - Positions are from device discovery (run `keyrx devices`)
/// - See `./scripts/show_key_position.sh all` for your keyboard layout
#[rhai_doc(module = "remapping")]
fn remap_rc_impl(
    row: i64,
    col: i64,
    to: &str,
    ops: &PendingOps,
    resolver: &RowColResolver,
) -> std::result::Result<(), Box<EvalAltResult>> {
    // Resolve (row, col) → KeyCode
    let from_key = resolver.resolve(row as u8, col as u8).map_err(|e| {
        Box::new(EvalAltResult::ErrorRuntime(
            format!("remap_rc: {}", e).into(),
            Position::NONE,
        ))
    })?;

    // Parse target key
    let to_key = parse_key_or_error(to, "remap_rc")?;

    tracing::debug!(
        service = "keyrx",
        event = "remap_rc_registered",
        component = "scripting_runtime",
        row = row,
        col = col,
        from = from_key.name(),
        to = to,
        "Registered position-based remap"
    );

    // Register remap (same as regular API)
    if let Ok(mut ops) = ops.lock() {
        ops.push(PendingOp::Remap {
            from: from_key,
            to: to_key,
        });
    }

    Ok(())
}

pub fn register_remap_rc(
    engine: &mut Engine,
    pending_ops: &PendingOps,
    resolver: &Arc<RowColResolver>,
) {
    __register_doc_remap_rc_impl();
    let ops = Arc::clone(pending_ops);
    let res = Arc::clone(resolver);
    engine.register_fn(
        "remap_rc",
        move |row: i64, col: i64, to: &str| -> std::result::Result<(), Box<EvalAltResult>> {
            remap_rc_impl(row, col, to, &ops, &res)
        },
    );
}

/// Configure tap-hold behavior for a key by its physical position.
///
/// Maps a physical key position to tap-hold behavior without needing to know the key name.
///
/// # Parameters
/// * `row` - 0-based row number from device profile
/// * `col` - 0-based column number from device profile
/// * `tap` - Key to send on quick press (e.g., "A", "Space")
/// * `hold` - Key/modifier to activate on hold (e.g., "LeftCtrl", "layer_toggle('nav')")
///
/// # Returns
/// Result indicating success or error
///
/// # Examples
/// ```
/// // Home row modifiers (row 3 on standard keyboards)
/// // tap_hold_rc(3, 1, "A", "LeftCtrl");   // tap=A, hold=Ctrl
/// // tap_hold_rc(3, 2, "S", "LeftAlt");    // tap=S, hold=Alt
/// // tap_hold_rc(3, 3, "D", "LeftMeta");   // tap=D, hold=Meta
/// // tap_hold_rc(3, 4, "F", "LeftShift");  // tap=F, hold=Shift
/// ```
///
/// # Notes
/// - Works across any keyboard layout (QWERTY/Dvorak/Colemak)
/// - Requires device profile to be loaded
/// - Threshold timing controlled by `set_tap_timeout(ms)`
#[rhai_doc(module = "remapping")]
fn tap_hold_rc_impl(
    row: i64,
    col: i64,
    tap: &str,
    hold: &str,
    ops: &PendingOps,
    resolver: &RowColResolver,
) -> std::result::Result<(), Box<EvalAltResult>> {
    // Resolve (row, col) → KeyCode
    let key = resolver.resolve(row as u8, col as u8).map_err(|e| {
        Box::new(EvalAltResult::ErrorRuntime(
            format!("tap_hold_rc: {}", e).into(),
            Position::NONE,
        ))
    })?;

    // Parse tap and hold keys
    let tap_key = parse_key_or_error(tap, "tap_hold_rc")?;
    let hold_key = parse_key_or_error(hold, "tap_hold_rc")?;

    tracing::debug!(
        service = "keyrx",
        event = "tap_hold_rc_registered",
        component = "scripting_runtime",
        row = row,
        col = col,
        key = key.name(),
        tap = tap,
        hold = hold,
        "Registered position-based tap-hold"
    );

    // Register tap-hold (same as regular API)
    if let Ok(mut ops) = ops.lock() {
        ops.push(PendingOp::TapHold {
            key,
            tap: tap_key,
            hold: HoldAction::Key(hold_key),
        });
    }

    Ok(())
}

pub fn register_tap_hold_rc(
    engine: &mut Engine,
    pending_ops: &PendingOps,
    resolver: &Arc<RowColResolver>,
) {
    __register_doc_tap_hold_rc_impl();
    let ops = Arc::clone(pending_ops);
    let res = Arc::clone(resolver);
    engine.register_fn(
        "tap_hold_rc",
        move |row: i64,
              col: i64,
              tap: &str,
              hold: &str|
              -> std::result::Result<(), Box<EvalAltResult>> {
            tap_hold_rc_impl(row, col, tap, hold, &ops, &res)
        },
    );
}

/// Block a key by its physical position.
///
/// Prevents a physical key position from having any effect when pressed,
/// without needing to know the key name.
///
/// # Parameters
/// * `row` - 0-based row number from device profile
/// * `col` - 0-based column number from device profile
///
/// # Returns
/// Result indicating success or error
///
/// # Examples
/// ```
/// // block_rc(1, 13);  // Block Insert key (often hit by accident)
/// // block_rc(0, 16);  // Block Delete key (top-right corner)
/// // block_rc(0, 15);  // Block Pause key
/// ```
///
/// # Notes
/// - Works regardless of keyboard layout
/// - Requires device profile to be loaded
/// - See `./scripts/show_key_position.sh all` for positions
#[rhai_doc(module = "remapping")]
fn block_rc_impl(
    row: i64,
    col: i64,
    ops: &PendingOps,
    resolver: &RowColResolver,
) -> std::result::Result<(), Box<EvalAltResult>> {
    // Resolve (row, col) → KeyCode
    let key = resolver.resolve(row as u8, col as u8).map_err(|e| {
        Box::new(EvalAltResult::ErrorRuntime(
            format!("block_rc: {}", e).into(),
            Position::NONE,
        ))
    })?;

    tracing::debug!(
        service = "keyrx",
        event = "block_rc_registered",
        component = "scripting_runtime",
        row = row,
        col = col,
        key = key.name(),
        "Registered position-based block"
    );

    // Register block (same as regular API)
    if let Ok(mut ops) = ops.lock() {
        ops.push(PendingOp::Block { key });
    }

    Ok(())
}

pub fn register_block_rc(
    engine: &mut Engine,
    pending_ops: &PendingOps,
    resolver: &Arc<RowColResolver>,
) {
    __register_doc_block_rc_impl();
    let ops = Arc::clone(pending_ops);
    let res = Arc::clone(resolver);
    engine.register_fn(
        "block_rc",
        move |row: i64, col: i64| -> std::result::Result<(), Box<EvalAltResult>> {
            block_rc_impl(row, col, &ops, &res)
        },
    );
}

pub fn register_combo_rc(
    engine: &mut Engine,
    pending_ops: &PendingOps,
    resolver: &Arc<RowColResolver>,
) {
    let ops = Arc::clone(pending_ops);
    let resolver_clone = Arc::clone(resolver);

    engine.register_fn("combo_rc", move |positions: Array, action_key: &str| {
        combo_rc_impl(positions, action_key, &ops, &resolver_clone)
    });
}

fn combo_rc_impl(
    positions: Array,
    action_key: &str,
    ops: &PendingOps,
    resolver: &RowColResolver,
) -> std::result::Result<(), Box<EvalAltResult>> {
    // Parse each position array [row, col] and resolve to KeyCode
    let mut parsed_keys = Vec::new();

    for (idx, pos_value) in positions.iter().enumerate() {
        // Each position should be an array [row, col]
        let pos_array = pos_value.clone().try_cast::<Array>().ok_or_else(|| {
            Box::new(EvalAltResult::ErrorRuntime(
                format!(
                    "combo_rc: position {} must be an array [row, col], got: {:?}",
                    idx, pos_value
                )
                .into(),
                Position::NONE,
            ))
        })?;

        if pos_array.len() != 2 {
            return Err(Box::new(EvalAltResult::ErrorRuntime(
                format!(
                    "combo_rc: position {} must be [row, col] with 2 elements, got {} elements",
                    idx,
                    pos_array.len()
                )
                .into(),
                Position::NONE,
            )));
        }

        let row = pos_array[0].clone().try_cast::<i64>().ok_or_else(|| {
            Box::new(EvalAltResult::ErrorRuntime(
                format!(
                    "combo_rc: position {} row must be an integer, got: {:?}",
                    idx, pos_array[0]
                )
                .into(),
                Position::NONE,
            ))
        })?;

        let col = pos_array[1].clone().try_cast::<i64>().ok_or_else(|| {
            Box::new(EvalAltResult::ErrorRuntime(
                format!(
                    "combo_rc: position {} col must be an integer, got: {:?}",
                    idx, pos_array[1]
                )
                .into(),
                Position::NONE,
            ))
        })?;

        // Resolve (row, col) to KeyCode
        let key_code = resolver.resolve(row as u8, col as u8).map_err(|e| {
            Box::new(EvalAltResult::ErrorRuntime(
                format!("combo_rc: position {}: {}", idx, e).into(),
                Position::NONE,
            ))
        })?;

        parsed_keys.push(key_code);
    }

    // Validate combo size (2-4 keys)
    let len = parsed_keys.len();
    if !(2..=4).contains(&len) {
        return Err(Box::new(EvalAltResult::ErrorRuntime(
            format!(
                "combo_rc: combo size {} violates constraint: must have 2-4 keys",
                len
            )
            .into(),
            Position::NONE,
        )));
    }

    // Parse action key
    let action = parse_key_or_error(action_key, "combo_rc")?;

    // Register the combo
    if let Ok(mut ops) = ops.lock() {
        ops.push(PendingOp::Combo {
            keys: parsed_keys,
            action: LayerAction::Remap(action),
        });
    }

    Ok(())
}

pub fn register_layer_map_rc(
    engine: &mut Engine,
    pending_ops: &PendingOps,
    resolver: &Arc<RowColResolver>,
    layer_view: &LayerView,
) {
    let ops = Arc::clone(pending_ops);
    let resolver_clone = Arc::clone(resolver);
    let view = Arc::clone(layer_view);

    engine.register_fn(
        "layer_map_rc",
        move |layer: i64, row: i64, col: i64, to: &str| {
            layer_map_rc_impl(layer, row, col, to, &ops, &resolver_clone, &view)
        },
    );
}

fn layer_map_rc_impl(
    layer: i64,
    row: i64,
    col: i64,
    to: &str,
    ops: &PendingOps,
    resolver: &RowColResolver,
    layer_view: &LayerView,
) -> std::result::Result<(), Box<EvalAltResult>> {
    // Validate layer number
    if !(0..=255).contains(&layer) {
        return Err(Box::new(EvalAltResult::ErrorRuntime(
            format!("layer_map_rc: layer {} out of range (0-255)", layer).into(),
            Position::NONE,
        )));
    }

    // Convert layer number to string name
    let layer_name = layer.to_string();

    // Check if layer is defined
    with_layer_view(layer_view, |stack| {
        if stack.layer_id_by_name(&layer_name).is_none() {
            return Err(layer_error(
                "layer_map_rc",
                format!(
                    "layer '{}' is not defined. Use layer_define({}) first",
                    layer_name, layer
                ),
            ));
        }
        Ok(())
    })?;

    // Resolve (row, col) to KeyCode
    let from_key = resolver.resolve(row as u8, col as u8).map_err(|e| {
        Box::new(EvalAltResult::ErrorRuntime(
            format!("layer_map_rc: {}", e).into(),
            Position::NONE,
        ))
    })?;

    // Parse target key
    let to_key = parse_key_or_error(to, "layer_map_rc")?;

    // Register layer mapping
    if let Ok(mut ops) = ops.lock() {
        ops.push(PendingOp::LayerMap {
            layer: layer_name,
            key: from_key,
            action: LayerMapAction::Remap(to_key),
        });
    }

    Ok(())
}
