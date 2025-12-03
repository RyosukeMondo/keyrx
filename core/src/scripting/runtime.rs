//! Rhai runtime implementation.
//!
//! This module contains the core `RhaiRuntime` struct and the `ScriptRuntime`
//! trait implementation. Function bindings are in `bindings.rs` and helper
//! types are in `builtins.rs`.

use super::bindings::register_all_functions;
use super::builtins::{
    apply_timing_update, LayerMapAction, LayerView, ModifierPreview, ModifierView, PendingOp,
    PendingOps,
};
use crate::engine::{KeyCode, LayerAction, LayerStack, TimingConfig};
use crate::scripting::RemapRegistry;
use crate::traits::ScriptRuntime;
use anyhow::{anyhow, Result};
use rhai::{Engine, Scope, AST};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

/// Production Rhai script runtime.
///
/// Uses a pending operations pattern to avoid `Rc<RefCell>`:
/// - Script functions push operations to a shared `Arc<Mutex<Vec>>`
/// - After script execution, operations are applied to the owned registry
pub struct RhaiRuntime {
    engine: Engine,
    ast: Option<AST>,
    defined_hooks: HashSet<String>,
    registry: RemapRegistry,
    pending_ops: PendingOps,
    layer_view: LayerView,
    modifier_view: ModifierView,
}

impl RhaiRuntime {
    /// Create a new Rhai runtime.
    pub fn new() -> Result<Self> {
        let mut engine = Engine::new();

        // Sandbox: disable dangerous operations
        engine.set_max_expr_depths(64, 64);
        engine.set_max_operations(100_000);

        // Shared pending operations storage
        let pending_ops: PendingOps = Arc::new(Mutex::new(Vec::new()));
        let layer_view: LayerView = Arc::new(Mutex::new(LayerStack::new()));
        let modifier_view: ModifierView = Arc::new(Mutex::new(ModifierPreview::new()));

        // Register script functions
        register_all_functions(&mut engine, &pending_ops, &layer_view, &modifier_view);

        Ok(Self {
            engine,
            ast: None,
            defined_hooks: HashSet::new(),
            registry: RemapRegistry::new(),
            pending_ops,
            layer_view,
            modifier_view,
        })
    }

    /// Check if a function is defined in the loaded script.
    fn scan_for_hooks(&mut self) {
        self.defined_hooks.clear();
        if let Some(ast) = &self.ast {
            for fn_def in ast.iter_functions() {
                self.defined_hooks.insert(fn_def.name.to_string());
            }
        }
    }

    /// Apply pending operations to the registry.
    fn apply_pending_ops(&mut self) {
        if let Ok(mut ops) = self.pending_ops.lock() {
            for op in ops.drain(..) {
                match op {
                    PendingOp::Remap { from, to } => {
                        self.registry.remap(from, to);
                    }
                    PendingOp::Block { key } => {
                        self.registry.block(key);
                    }
                    PendingOp::Pass { key } => {
                        self.registry.pass(key);
                    }
                    PendingOp::TapHold { key, tap, hold } => {
                        self.registry.register_tap_hold(key, tap, hold);
                    }
                    PendingOp::Combo { keys, action } => {
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
                    PendingOp::LayerDefine { name, transparent } => {
                        if let Err(err) = self.registry.define_layer(&name, transparent) {
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
                    PendingOp::LayerMap { layer, key, action } => {
                        match Self::resolve_layer_action(&self.registry, action) {
                            Ok(resolved) => {
                                if let Err(err) =
                                    self.registry.map_layer(&layer, key, resolved.clone())
                                {
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
                    PendingOp::LayerPush { name } => {
                        if let Err(err) = self.registry.push_layer(&name) {
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
                    PendingOp::LayerToggle { name } => {
                        if let Err(err) = self.registry.toggle_layer(&name) {
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
                    PendingOp::LayerPop => {
                        let _ = self.registry.pop_layer();
                    }
                    PendingOp::DefineModifier { name, id } => {
                        if let Err(err) = self.registry.define_modifier_with_id(&name, Some(id)) {
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
                    PendingOp::ModifierActivate { name, id } => {
                        if self.registry.modifier_id(&name).is_none() {
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
                    PendingOp::ModifierDeactivate { name, id } => {
                        if self.registry.modifier_id(&name).is_none() {
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
                    PendingOp::ModifierOneShot { name, id } => {
                        if self.registry.modifier_id(&name).is_none() {
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
                    PendingOp::SetTiming(update) => {
                        let mut timing: TimingConfig = self.registry.timing_config().clone();
                        apply_timing_update(&mut timing, update);
                        self.registry.set_timing_config(timing);
                    }
                }
            }
        }

        if let Ok(mut view) = self.layer_view.lock() {
            *view = self.registry.layers().clone();
        }

        if let Ok(mut view) = self.modifier_view.lock() {
            view.sync_from_registry(&self.registry);
        }
    }

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

    /// Get a reference to the remap registry.
    pub fn registry(&self) -> &RemapRegistry {
        &self.registry
    }
}

impl ScriptRuntime for RhaiRuntime {
    fn execute(&mut self, script: &str) -> Result<()> {
        self.engine
            .run(script)
            .map_err(|e| anyhow!("Script execution failed: {}", e))?;

        self.apply_pending_ops();
        Ok(())
    }

    fn call_hook(&mut self, hook: &str) -> Result<()> {
        let ast = self
            .ast
            .as_ref()
            .ok_or_else(|| anyhow!("No script loaded"))?;

        self.engine
            .call_fn::<()>(&mut Scope::new(), ast, hook, ())
            .map_err(|e| anyhow!("Hook '{}' call failed: {}", hook, e))?;

        self.apply_pending_ops();
        Ok(())
    }

    fn load_file(&mut self, path: &str) -> Result<()> {
        let ast = self
            .engine
            .compile_file(path.into())
            .map_err(|e| anyhow!("Failed to compile script '{}': {}", path, e))?;

        self.ast = Some(ast);
        self.scan_for_hooks();
        Ok(())
    }

    fn run_script(&mut self) -> Result<()> {
        let ast = self
            .ast
            .as_ref()
            .ok_or_else(|| anyhow!("No script loaded"))?;

        self.engine
            .run_ast(ast)
            .map_err(|e| anyhow!("Script execution failed: {}", e))?;

        self.apply_pending_ops();
        Ok(())
    }

    fn has_hook(&self, hook: &str) -> bool {
        self.defined_hooks.contains(hook)
    }

    fn lookup_remap(&self, key: KeyCode) -> crate::engine::RemapAction {
        self.registry.lookup(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{HoldAction, LayerAction, Modifier, RemapAction, TimingConfig};

    #[test]
    fn new_runtime_has_empty_registry() {
        let runtime = RhaiRuntime::new().unwrap();
        assert_eq!(runtime.lookup_remap(KeyCode::A), RemapAction::Pass);
    }

    #[test]
    fn execute_remap_registers_mapping() {
        let mut runtime = RhaiRuntime::new().unwrap();
        runtime.execute(r#"remap("A", "B");"#).unwrap();
        assert_eq!(
            runtime.lookup_remap(KeyCode::A),
            RemapAction::Remap(KeyCode::B)
        );
    }

    #[test]
    fn execute_block_registers_block() {
        let mut runtime = RhaiRuntime::new().unwrap();
        runtime.execute(r#"block("CapsLock");"#).unwrap();
        assert_eq!(runtime.lookup_remap(KeyCode::CapsLock), RemapAction::Block);
    }

    #[test]
    fn execute_pass_registers_pass() {
        let mut runtime = RhaiRuntime::new().unwrap();
        runtime.execute(r#"remap("A", "B"); pass("A");"#).unwrap();
        assert_eq!(runtime.lookup_remap(KeyCode::A), RemapAction::Pass);
    }

    #[test]
    fn unknown_key_returns_error() {
        let mut runtime = RhaiRuntime::new().unwrap();
        // Invalid keys should cause script errors
        let result = runtime.execute(r#"remap("InvalidKey", "B");"#);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("InvalidKey"));

        let result = runtime.execute(r#"remap("A", "InvalidKey");"#);
        assert!(result.is_err());

        let result = runtime.execute(r#"block("InvalidKey");"#);
        assert!(result.is_err());

        let result = runtime.execute(r#"pass("InvalidKey");"#);
        assert!(result.is_err());

        // Valid mappings should still work
        runtime.execute(r#"remap("C", "D");"#).unwrap();
        assert_eq!(
            runtime.lookup_remap(KeyCode::C),
            RemapAction::Remap(KeyCode::D)
        );
    }

    #[test]
    fn errors_are_catchable_in_scripts() {
        let mut runtime = RhaiRuntime::new().unwrap();
        // Using try/catch in Rhai scripts should work
        let result = runtime.execute(
            r#"
            let caught = false;
            try {
                remap("InvalidKey", "B");
            } catch {
                caught = true;
            }
            // After catching the error, valid remaps should still work
            remap("A", "B");
            "#,
        );
        assert!(result.is_ok());
        assert_eq!(
            runtime.lookup_remap(KeyCode::A),
            RemapAction::Remap(KeyCode::B)
        );
    }

    #[test]
    fn multiple_remaps_work() {
        let mut runtime = RhaiRuntime::new().unwrap();
        runtime
            .execute(
                r#"
            remap("A", "B");
            remap("C", "D");
            block("CapsLock");
        "#,
            )
            .unwrap();
        assert_eq!(
            runtime.lookup_remap(KeyCode::A),
            RemapAction::Remap(KeyCode::B)
        );
        assert_eq!(
            runtime.lookup_remap(KeyCode::C),
            RemapAction::Remap(KeyCode::D)
        );
        assert_eq!(runtime.lookup_remap(KeyCode::CapsLock), RemapAction::Block);
    }

    #[test]
    fn execute_tap_hold_registers_binding() {
        let mut runtime = RhaiRuntime::new().unwrap();
        runtime
            .execute(r#"tap_hold("CapsLock", "Escape", "LeftCtrl");"#)
            .unwrap();

        let binding = runtime.registry().tap_hold(KeyCode::CapsLock).unwrap();
        assert_eq!(binding.tap, KeyCode::Escape);
        assert_eq!(binding.hold, HoldAction::Key(KeyCode::LeftCtrl));
    }

    #[test]
    fn execute_tap_hold_mod_registers_modifier_binding() {
        let mut runtime = RhaiRuntime::new().unwrap();
        runtime
            .execute(r#"tap_hold_mod("CapsLock", "Escape", 2);"#)
            .unwrap();

        let binding = runtime.registry().tap_hold(KeyCode::CapsLock).unwrap();
        assert_eq!(binding.tap, KeyCode::Escape);
        assert_eq!(binding.hold, HoldAction::Modifier(2));
    }

    #[test]
    fn tap_hold_mod_rejects_out_of_range_modifier() {
        let mut runtime = RhaiRuntime::new().unwrap();
        let result = runtime.execute(r#"tap_hold_mod("CapsLock", "Escape", 999);"#);
        assert!(result.is_err());
        assert!(runtime.registry().tap_hold(KeyCode::CapsLock).is_none());
    }

    #[test]
    fn execute_combo_registers_definition() {
        let mut runtime = RhaiRuntime::new().unwrap();
        runtime.execute(r#"combo(["A", "B"], "Escape");"#).unwrap();

        let action = runtime.registry().combos().find(&[KeyCode::A, KeyCode::B]);
        assert_eq!(action, Some(&LayerAction::Remap(KeyCode::Escape)));
    }

    #[test]
    fn combo_requires_between_two_and_four_keys() {
        let mut runtime = RhaiRuntime::new().unwrap();
        assert!(runtime.execute(r#"combo(["A"], "Escape");"#).is_err());
        assert!(runtime
            .execute(r#"combo(["A","B","C","D","E"], "Escape");"#)
            .is_err());
    }

    #[test]
    fn combo_rejects_non_string_keys() {
        let mut runtime = RhaiRuntime::new().unwrap();
        let err = runtime.execute(r#"combo([1, "B"], "Escape");"#);
        assert!(err.is_err());
    }

    #[test]
    fn timing_functions_update_config() {
        let mut runtime = RhaiRuntime::new().unwrap();
        runtime
            .execute(
                r#"
                set_tap_timeout(350);
                set_combo_timeout(75);
                set_hold_delay(10);
                set_eager_tap(true);
                set_permissive_hold(false);
                set_retro_tap(true);
            "#,
            )
            .unwrap();

        let timing = runtime.registry().timing_config();
        assert_eq!(timing.tap_timeout_ms, 350);
        assert_eq!(timing.combo_timeout_ms, 75);
        assert_eq!(timing.hold_delay_ms, 10);
        assert!(timing.eager_tap);
        assert!(!timing.permissive_hold);
        assert!(timing.retro_tap);
    }

    #[test]
    fn timing_functions_validate_ranges() {
        let mut runtime = RhaiRuntime::new().unwrap();
        assert!(runtime.execute(r#"set_tap_timeout(0);"#).is_err());
        assert!(runtime.execute(r#"set_combo_timeout(6000);"#).is_err());
        assert!(runtime.execute(r#"set_hold_delay(-1);"#).is_err());

        let timing = runtime.registry().timing_config();
        assert_eq!(*timing, TimingConfig::default());
    }

    #[test]
    fn layer_functions_apply_and_query() {
        let mut runtime = RhaiRuntime::new().unwrap();
        runtime
            .execute(
                r#"
                layer_define("nav", true);
                layer_map("nav", "A", "Escape");
                layer_push("nav");
                if !is_layer_active("nav") {
                    throw "nav should be active";
                }
            "#,
            )
            .unwrap();

        let registry = runtime.registry();
        assert!(registry.is_layer_active("nav").unwrap());
        let action = registry.layers().lookup(KeyCode::A);
        assert_eq!(action, Some(&LayerAction::Remap(KeyCode::Escape)));
    }

    #[test]
    fn layer_map_supports_toggle_actions() {
        let mut runtime = RhaiRuntime::new().unwrap();
        runtime
            .execute(
                r#"
                layer_define("nav", false);
                layer_define("fn", false);
                layer_map("nav", "B", "layer_toggle:fn");
            "#,
            )
            .unwrap();

        runtime.execute(r#"layer_push("nav");"#).unwrap();
        let nav_id = runtime.registry().layer_id("nav").unwrap();
        let fn_id = runtime.registry().layer_id("fn").unwrap();

        assert_eq!(
            runtime.registry().layers().lookup(KeyCode::B),
            Some(&LayerAction::LayerToggle(fn_id))
        );
        assert_eq!(nav_id, 1);
        assert_eq!(fn_id, 2);
    }

    #[test]
    fn define_modifier_and_activate_it() {
        let mut runtime = RhaiRuntime::new().unwrap();
        runtime
            .execute(
                r#"
                let id = define_modifier("hyper");
                if id != 0 { throw "unexpected modifier id"; }
                modifier_on("hyper");
            "#,
            )
            .unwrap();

        let registry = runtime.registry();
        let id = registry.modifier_id("hyper").unwrap();
        assert_eq!(id, 0);
        assert!(registry.modifier_state().is_active(Modifier::Virtual(id)));
    }

    #[test]
    fn one_shot_marks_modifier_as_active_once() {
        let mut runtime = RhaiRuntime::new().unwrap();
        runtime
            .execute(
                r#"
                define_modifier("hyper");
                one_shot("hyper");
            "#,
            )
            .unwrap();

        let registry = runtime.registry();
        let id = registry.modifier_id("hyper").unwrap();
        let mut snapshot = registry.modifier_state();
        assert!(snapshot.is_active(Modifier::Virtual(id)));
        assert!(snapshot.consume_one_shot(Modifier::Virtual(id)));
        assert!(!snapshot.is_active(Modifier::Virtual(id)));
    }

    #[test]
    fn modifier_functions_require_definition() {
        let mut runtime = RhaiRuntime::new().unwrap();
        assert!(runtime.execute(r#"modifier_on("hyper");"#).is_err());
        assert!(runtime.execute(r#"modifier_off("hyper");"#).is_err());
        assert!(runtime.execute(r#"one_shot("hyper");"#).is_err());
        assert!(runtime.execute(r#"is_modifier_active("hyper");"#).is_err());
    }
}
