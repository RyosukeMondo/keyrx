//! Rhai engine configuration for validation.
//!
//! Creates a sandboxed Rhai engine that collects pending operations
//! without executing them against a real registry.

use std::sync::{Arc, Mutex};

use rhai::{Engine, EvalAltResult};

use crate::engine::{HoldAction, KeyCode, LayerAction};
use crate::scripting::helpers::parse_key_or_error;
use crate::scripting::{LayerMapAction, PendingOp, TimingUpdate};

use super::super::types::{SourceLocation, ValidationError};

/// Thread-safe pending operations storage for validation.
pub type PendingOps = Arc<Mutex<Vec<PendingOp>>>;

/// Create a Rhai engine configured for validation (no actual registry).
pub fn create_validation_engine(pending_ops: &PendingOps) -> Engine {
    let mut engine = Engine::new();

    // Sandbox settings
    engine.set_max_expr_depths(64, 64);
    engine.set_max_operations(100_000);

    register_key_mapping_functions(&mut engine, pending_ops);
    register_layer_functions(&mut engine, pending_ops);
    register_modifier_functions(&mut engine, pending_ops);
    register_timing_functions(&mut engine, pending_ops);
    register_combo_functions(&mut engine, pending_ops);

    engine
}

/// Register key mapping functions: remap, block, pass, tap_hold.
fn register_key_mapping_functions(engine: &mut Engine, pending_ops: &PendingOps) {
    let ops_remap = Arc::clone(pending_ops);
    engine.register_fn(
        "remap",
        move |from: &str, to: &str| -> Result<(), Box<EvalAltResult>> {
            let from_key = parse_key_or_error(from, "remap")?;
            let to_key = parse_key_or_error(to, "remap")?;
            if let Ok(mut guard) = ops_remap.lock() {
                guard.push(PendingOp::Remap {
                    from: from_key,
                    to: to_key,
                });
            }
            Ok(())
        },
    );

    let ops_block = Arc::clone(pending_ops);
    engine.register_fn(
        "block",
        move |key: &str| -> Result<(), Box<EvalAltResult>> {
            let key_code = parse_key_or_error(key, "block")?;
            if let Ok(mut guard) = ops_block.lock() {
                guard.push(PendingOp::Block { key: key_code });
            }
            Ok(())
        },
    );

    let ops_pass = Arc::clone(pending_ops);
    engine.register_fn("pass", move |key: &str| -> Result<(), Box<EvalAltResult>> {
        let key_code = parse_key_or_error(key, "pass")?;
        if let Ok(mut guard) = ops_pass.lock() {
            guard.push(PendingOp::Pass { key: key_code });
        }
        Ok(())
    });

    let ops_tap_hold = Arc::clone(pending_ops);
    engine.register_fn(
        "tap_hold",
        move |key: &str, tap: &str, hold: &str| -> Result<(), Box<EvalAltResult>> {
            let key_code = parse_key_or_error(key, "tap_hold")?;
            let tap_key = parse_key_or_error(tap, "tap_hold")?;
            let hold_key = parse_key_or_error(hold, "tap_hold")?;
            if let Ok(mut guard) = ops_tap_hold.lock() {
                guard.push(PendingOp::TapHold {
                    key: key_code,
                    tap: tap_key,
                    hold: HoldAction::Key(hold_key),
                });
            }
            Ok(())
        },
    );
}

/// Register layer functions: define_layer, layer_push, layer_toggle, layer_pop.
fn register_layer_functions(engine: &mut Engine, pending_ops: &PendingOps) {
    let ops_layer = Arc::clone(pending_ops);
    engine.register_fn("define_layer", move |name: &str| {
        if let Ok(mut guard) = ops_layer.lock() {
            guard.push(PendingOp::LayerDefine {
                name: name.to_string(),
                transparent: false,
            });
        }
    });

    let ops_layer_t = Arc::clone(pending_ops);
    engine.register_fn("define_layer", move |name: &str, transparent: bool| {
        if let Ok(mut guard) = ops_layer_t.lock() {
            guard.push(PendingOp::LayerDefine {
                name: name.to_string(),
                transparent,
            });
        }
    });

    let ops_push = Arc::clone(pending_ops);
    engine.register_fn("layer_push", move |name: &str| {
        if let Ok(mut guard) = ops_push.lock() {
            guard.push(PendingOp::LayerPush {
                name: name.to_string(),
            });
        }
    });

    let ops_toggle = Arc::clone(pending_ops);
    engine.register_fn("layer_toggle", move |name: &str| {
        if let Ok(mut guard) = ops_toggle.lock() {
            guard.push(PendingOp::LayerToggle {
                name: name.to_string(),
            });
        }
    });

    let ops_pop = Arc::clone(pending_ops);
    engine.register_fn("layer_pop", move || {
        if let Ok(mut guard) = ops_pop.lock() {
            guard.push(PendingOp::LayerPop);
        }
    });
}

/// Register modifier functions: define_modifier, modifier_activate/deactivate/one_shot.
fn register_modifier_functions(engine: &mut Engine, pending_ops: &PendingOps) {
    let ops_mod = Arc::clone(pending_ops);
    engine.register_fn("define_modifier", move |name: &str| {
        if let Ok(mut guard) = ops_mod.lock() {
            guard.push(PendingOp::DefineModifier {
                name: name.to_string(),
                id: 0, // ID assigned at runtime
            });
        }
    });

    let ops_act = Arc::clone(pending_ops);
    engine.register_fn("modifier_activate", move |name: &str| {
        if let Ok(mut guard) = ops_act.lock() {
            guard.push(PendingOp::ModifierActivate {
                name: name.to_string(),
                id: 0,
            });
        }
    });

    let ops_deact = Arc::clone(pending_ops);
    engine.register_fn("modifier_deactivate", move |name: &str| {
        if let Ok(mut guard) = ops_deact.lock() {
            guard.push(PendingOp::ModifierDeactivate {
                name: name.to_string(),
                id: 0,
            });
        }
    });

    let ops_os = Arc::clone(pending_ops);
    engine.register_fn("modifier_one_shot", move |name: &str| {
        if let Ok(mut guard) = ops_os.lock() {
            guard.push(PendingOp::ModifierOneShot {
                name: name.to_string(),
                id: 0,
            });
        }
    });
}

/// Register timing functions: tap_timeout, combo_timeout, hold_delay.
fn register_timing_functions(engine: &mut Engine, pending_ops: &PendingOps) {
    let ops_tap_to = Arc::clone(pending_ops);
    engine.register_fn("tap_timeout", move |ms: i64| {
        if let Ok(mut guard) = ops_tap_to.lock() {
            guard.push(PendingOp::SetTiming(TimingUpdate::TapTimeout(ms as u32)));
        }
    });

    let ops_combo_to = Arc::clone(pending_ops);
    engine.register_fn("combo_timeout", move |ms: i64| {
        if let Ok(mut guard) = ops_combo_to.lock() {
            guard.push(PendingOp::SetTiming(TimingUpdate::ComboTimeout(ms as u32)));
        }
    });

    let ops_hold = Arc::clone(pending_ops);
    engine.register_fn("hold_delay", move |ms: i64| {
        if let Ok(mut guard) = ops_hold.lock() {
            guard.push(PendingOp::SetTiming(TimingUpdate::HoldDelay(ms as u32)));
        }
    });
}

/// Register combo and layer_map functions.
fn register_combo_functions(engine: &mut Engine, pending_ops: &PendingOps) {
    let ops_combo = Arc::clone(pending_ops);
    engine.register_fn("combo", move |keys: rhai::Array, action: &str| {
        let mut key_codes = Vec::new();
        for key in keys {
            if let Ok(s) = key.clone().into_string() {
                if let Some(kc) = KeyCode::from_name(&s) {
                    key_codes.push(kc);
                }
            }
        }
        let layer_action = if action == "block" {
            LayerAction::Block
        } else if let Some(kc) = KeyCode::from_name(action) {
            LayerAction::Remap(kc)
        } else {
            return;
        };

        if let Ok(mut guard) = ops_combo.lock() {
            guard.push(PendingOp::Combo {
                keys: key_codes,
                action: layer_action,
            });
        }
    });

    let ops_map = Arc::clone(pending_ops);
    engine.register_fn("layer_map", move |layer: &str, key: &str, action: &str| {
        let key_code = match KeyCode::from_name(key) {
            Some(k) => k,
            None => return,
        };
        let map_action = if action == "block" {
            LayerMapAction::Block
        } else if action == "pass" {
            LayerMapAction::Pass
        } else if let Some(kc) = KeyCode::from_name(action) {
            LayerMapAction::Remap(kc)
        } else {
            return;
        };

        if let Ok(mut guard) = ops_map.lock() {
            guard.push(PendingOp::LayerMap {
                layer: layer.to_string(),
                key: key_code,
                action: map_action,
            });
        }
    });
}

/// Convert a Rhai parse error to a ValidationError.
pub fn parse_error_to_validation_error(err: Box<rhai::EvalAltResult>) -> ValidationError {
    let (line, col) = match err.position() {
        rhai::Position::NONE => (0, None),
        pos => (pos.line().unwrap_or(0), pos.position()),
    };

    let mut error = ValidationError::new("E000", format!("Parse error: {}", err));
    if line > 0 {
        let mut loc = SourceLocation::new(line);
        if let Some(c) = col {
            loc = loc.with_column(c);
        }
        error = error.with_location(loc);
    }
    error
}
