//! Pending operation handling for the Rhai scripting runtime.
//!
//! This module contains the logic for applying pending operations (PendingOp)
//! from script execution to the remap registry.

use super::builtins::{apply_timing_update, LayerMapAction, PendingOp, PendingOps};
use super::registry_sync::RegistrySyncer;
use crate::engine::{KeyCode, LayerAction, TimingConfig};
use crate::scripting::RemapRegistry;

/// Applies pending operations to the registry.
///
/// This struct encapsulates the logic for processing operations that were
/// queued during script execution and applying them to the registry.
pub struct PendingOpsApplier<'a> {
    registry: &'a mut RemapRegistry,
}

impl<'a> PendingOpsApplier<'a> {
    /// Create a new applier for the given registry.
    pub fn new(registry: &'a mut RemapRegistry) -> Self {
        Self { registry }
    }

    /// Apply all pending operations and sync views.
    pub fn apply_all(&mut self, pending_ops: &PendingOps, syncer: &mut RegistrySyncer) {
        self.apply_ops(pending_ops);
        syncer.sync_from_registry(self.registry);
    }

    /// Apply pending operations to the registry.
    fn apply_ops(&mut self, pending_ops: &PendingOps) {
        if let Ok(mut ops) = pending_ops.lock() {
            for op in ops.drain(..) {
                self.apply_single_op(op);
            }
        }
    }

    /// Apply a single pending operation.
    fn apply_single_op(&mut self, op: PendingOp) {
        match op {
            PendingOp::Remap { from, to } => self.apply_remap(from, to),
            PendingOp::Block { key } => self.apply_block(key),
            PendingOp::Pass { key } => self.apply_pass(key),
            PendingOp::TapHold { key, tap, hold } => self.apply_tap_hold(key, tap, hold),
            PendingOp::Combo { keys, action } => self.apply_combo(keys, action),
            PendingOp::LayerDefine { name, transparent } => {
                self.apply_layer_define(&name, transparent)
            }
            PendingOp::LayerMap { layer, key, action } => self.apply_layer_map(&layer, key, action),
            PendingOp::LayerPush { name } => self.apply_layer_push(&name),
            PendingOp::LayerToggle { name } => self.apply_layer_toggle(&name),
            PendingOp::LayerPop => self.apply_layer_pop(),
            PendingOp::DefineModifier { name, id } => self.apply_define_modifier(&name, id),
            PendingOp::ModifierActivate { name, id } => self.apply_modifier_activate(&name, id),
            PendingOp::ModifierDeactivate { name, id } => self.apply_modifier_deactivate(&name, id),
            PendingOp::ModifierOneShot { name, id } => self.apply_modifier_one_shot(&name, id),
            PendingOp::SetTiming(update) => self.apply_timing(update),
        }
    }

    fn apply_remap(&mut self, from: KeyCode, to: KeyCode) {
        self.registry.remap(from, to);
    }

    fn apply_block(&mut self, key: KeyCode) {
        self.registry.block(key);
    }

    fn apply_pass(&mut self, key: KeyCode) {
        self.registry.pass(key);
    }

    fn apply_tap_hold(&mut self, key: KeyCode, tap: KeyCode, hold: crate::engine::HoldAction) {
        self.registry.register_tap_hold(key, tap, hold);
    }

    fn apply_combo(&mut self, keys: Vec<KeyCode>, action: LayerAction) {
        if !self.registry.register_combo(&keys, action) {
            tracing::warn!(
                service = "keyrx",
                event = "rhai_combo_register_failed",
                component = "scripting_runtime",
                keys = ?keys,
                "Combo registration rejected (invalid key count)"
            );
        }
    }

    fn apply_layer_define(&mut self, name: &str, transparent: bool) {
        if let Err(err) = self.registry.define_layer(name, transparent) {
            tracing::warn!(
                service = "keyrx",
                event = "rhai_layer_define_failed",
                component = "scripting_runtime",
                layer = name,
                error = %err,
                "Layer definition failed"
            );
        }
    }

    fn apply_layer_map(&mut self, layer: &str, key: KeyCode, action: LayerMapAction) {
        match resolve_layer_action(self.registry, action) {
            Ok(resolved) => {
                if let Err(err) = self.registry.map_layer(layer, key, resolved.clone()) {
                    tracing::warn!(
                        service = "keyrx",
                        event = "rhai_layer_map_failed",
                        component = "scripting_runtime",
                        layer = layer,
                        key = ?key,
                        action = ?resolved,
                        error = %err,
                        "Layer mapping failed"
                    );
                }
            }
            Err(err) => {
                tracing::warn!(
                    service = "keyrx",
                    event = "rhai_layer_map_resolve_failed",
                    component = "scripting_runtime",
                    layer = layer,
                    key = ?key,
                    error = %err,
                    "Failed to resolve layer action"
                );
            }
        }
    }

    fn apply_layer_push(&mut self, name: &str) {
        if let Err(err) = self.registry.push_layer(name) {
            tracing::warn!(
                service = "keyrx",
                event = "rhai_layer_push_failed",
                component = "scripting_runtime",
                layer = name,
                error = %err,
                "Layer push failed"
            );
        }
    }

    fn apply_layer_toggle(&mut self, name: &str) {
        if let Err(err) = self.registry.toggle_layer(name) {
            tracing::warn!(
                service = "keyrx",
                event = "rhai_layer_toggle_failed",
                component = "scripting_runtime",
                layer = name,
                error = %err,
                "Layer toggle failed"
            );
        }
    }

    fn apply_layer_pop(&mut self) {
        let _ = self.registry.pop_layer();
    }

    fn apply_define_modifier(&mut self, name: &str, id: u8) {
        if let Err(err) = self.registry.define_modifier_with_id(name, Some(id)) {
            tracing::warn!(
                service = "keyrx",
                event = "rhai_define_modifier_failed",
                component = "scripting_runtime",
                modifier = name,
                error = %err,
                "Modifier definition failed"
            );
        }
    }

    fn apply_modifier_activate(&mut self, name: &str, id: u8) {
        if self.registry.modifier_id(name).is_none() {
            tracing::warn!(
                service = "keyrx",
                event = "rhai_modifier_activate_undefined",
                component = "scripting_runtime",
                modifier = name,
                "Modifier activation ignored (undefined)"
            );
        } else {
            self.registry.activate_modifier(id);
        }
    }

    fn apply_modifier_deactivate(&mut self, name: &str, id: u8) {
        if self.registry.modifier_id(name).is_none() {
            tracing::warn!(
                service = "keyrx",
                event = "rhai_modifier_deactivate_undefined",
                component = "scripting_runtime",
                modifier = name,
                "Modifier deactivation ignored (undefined)"
            );
        } else {
            self.registry.deactivate_modifier(id);
        }
    }

    fn apply_modifier_one_shot(&mut self, name: &str, id: u8) {
        if self.registry.modifier_id(name).is_none() {
            tracing::warn!(
                service = "keyrx",
                event = "rhai_modifier_one_shot_undefined",
                component = "scripting_runtime",
                modifier = name,
                "Modifier one-shot ignored (undefined)"
            );
        } else {
            self.registry.one_shot_modifier(id);
        }
    }

    fn apply_timing(&mut self, update: super::builtins::TimingUpdate) {
        let mut timing: TimingConfig = self.registry.timing_config().clone();
        apply_timing_update(&mut timing, update);
        self.registry.set_timing_config(timing);
    }
}

/// Resolve a LayerMapAction to a concrete LayerAction.
fn resolve_layer_action(
    registry: &RemapRegistry,
    action: LayerMapAction,
) -> Result<LayerAction, String> {
    match action {
        LayerMapAction::Remap(key) => Ok(LayerAction::Remap(key)),
        LayerMapAction::Block => Ok(LayerAction::Block),
        LayerMapAction::Pass => Ok(LayerAction::Pass),
        LayerMapAction::TapHold { tap, hold } => Ok(LayerAction::TapHold { tap, hold }),
        LayerMapAction::LayerPush(name) => {
            let id = registry
                .layer_id(&name)
                .ok_or_else(|| format!("layer '{}' is not defined", name))?;
            Ok(LayerAction::LayerPush(id))
        }
        LayerMapAction::LayerToggle(name) => {
            let id = registry
                .layer_id(&name)
                .ok_or_else(|| format!("layer '{}' is not defined", name))?;
            Ok(LayerAction::LayerToggle(id))
        }
        LayerMapAction::LayerPop => Ok(LayerAction::LayerPop),
    }
}
