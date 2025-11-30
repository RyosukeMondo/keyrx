//! Rhai runtime implementation.

use super::helpers::parse_key_or_error;
use crate::engine::{HoldAction, KeyCode, LayerAction, LayerStack, VirtualModifiers};
use crate::scripting::RemapRegistry;
use crate::traits::ScriptRuntime;
use anyhow::{anyhow, Result};
use rhai::{Array, Engine, EvalAltResult, Position, Scope, AST};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

/// Thread-safe pending operations storage.
/// This uses Arc<Mutex> instead of Rc<RefCell> for thread safety
/// and to allow the closure to own a reference.
type PendingOps = Arc<Mutex<Vec<PendingOp>>>;
type LayerView = Arc<Mutex<LayerStack>>;

/// A pending operation to be applied to the registry after script execution.
#[derive(Debug, Clone)]
enum PendingOp {
    Remap {
        from: KeyCode,
        to: KeyCode,
    },
    Block {
        key: KeyCode,
    },
    Pass {
        key: KeyCode,
    },
    TapHold {
        key: KeyCode,
        tap: KeyCode,
        hold: HoldAction,
    },
    Combo {
        keys: Vec<KeyCode>,
        action: LayerAction,
    },
    LayerDefine {
        name: String,
        transparent: bool,
    },
    LayerMap {
        layer: String,
        key: KeyCode,
        action: LayerMapAction,
    },
    LayerPush {
        name: String,
    },
    LayerToggle {
        name: String,
    },
    LayerPop,
}

#[derive(Debug, Clone)]
enum LayerMapAction {
    Remap(KeyCode),
    Block,
    Pass,
    TapHold { tap: KeyCode, hold: HoldAction },
    LayerPush(String),
    LayerToggle(String),
    LayerPop,
}

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

        // Register script functions
        Self::register_remap_functions(&mut engine, &pending_ops, &layer_view);

        Ok(Self {
            engine,
            ast: None,
            defined_hooks: HashSet::new(),
            registry: RemapRegistry::new(),
            pending_ops,
            layer_view,
        })
    }

    /// Register remap-related functions for the Rhai engine.
    fn register_remap_functions(
        engine: &mut Engine,
        pending_ops: &PendingOps,
        layer_view: &LayerView,
    ) {
        Self::register_debug(engine);
        Self::register_remap(engine, pending_ops);
        Self::register_block(engine, pending_ops);
        Self::register_pass(engine, pending_ops);
        Self::register_tap_hold(engine, pending_ops);
        Self::register_tap_hold_mod(engine, pending_ops);
        Self::register_combo(engine, pending_ops);
        Self::register_layer_functions(engine, pending_ops, layer_view);
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
            move |key: &str,
                  tap: &str,
                  hold: &str|
                  -> std::result::Result<(), Box<EvalAltResult>> {
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

                let modifier = u8::try_from(modifier_id).map_err(|_| {
                    Box::new(EvalAltResult::ErrorRuntime(
                        format!(
                            "tap_hold_mod: modifier id '{}' is out of range (0-{})",
                            modifier_id,
                            VirtualModifiers::MAX_ID
                        )
                        .into(),
                        rhai::Position::NONE,
                    ))
                })?;

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
                let parsed_keys = Self::parse_keys_array(keys)?;
                let action = parse_key_or_error(action_key, "combo")?;

                if !(2..=4).contains(&parsed_keys.len()) {
                    return Err(Box::new(EvalAltResult::ErrorRuntime(
                        format!("combo: expected 2-4 keys, got {}", parsed_keys.len()).into(),
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

    fn register_layer_functions(
        engine: &mut Engine,
        pending_ops: &PendingOps,
        layer_view: &LayerView,
    ) {
        Self::register_layer_define(engine, pending_ops, layer_view);
        Self::register_layer_map(engine, pending_ops, layer_view);
        Self::register_layer_push(engine, pending_ops, layer_view);
        Self::register_layer_pop(engine, pending_ops, layer_view);
        Self::register_layer_toggle(engine, pending_ops, layer_view);
        Self::register_is_layer_active(engine, layer_view);
    }

    fn register_layer_define(
        engine: &mut Engine,
        pending_ops: &PendingOps,
        layer_view: &LayerView,
    ) {
        let ops = Arc::clone(pending_ops);
        let view = Arc::clone(layer_view);
        engine.register_fn(
            "layer_define",
            move |name: &str, transparent: bool| -> std::result::Result<(), Box<EvalAltResult>> {
                let normalized = Self::normalize_layer_name(name, "layer_define")?;
                Self::with_layer_view(&view, |stack| {
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
                let normalized = Self::normalize_layer_name(layer_name, "layer_map")?;
                let key_code = parse_key_or_error(key, "layer_map")?;
                let parsed_action = Self::parse_layer_action(action, "layer_map", &view)?;

                // Require layer to be defined in the preview state to give early feedback.
                Self::with_layer_view(&view, |stack| {
                    if stack.layer_id_by_name(&normalized).is_none() {
                        return Err(Self::layer_error(
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
                let normalized = Self::normalize_layer_name(name, "layer_push")?;
                Self::with_layer_view(&view, |stack| {
                    let layer_id = stack.layer_id_by_name(&normalized).ok_or_else(|| {
                        Self::layer_error(
                            "layer_push",
                            format!("layer '{}' is not defined", normalized),
                        )
                    })?;

                    let pushed = stack.push(layer_id);
                    if !pushed {
                        return Err(Self::layer_error(
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
                Self::with_layer_view(&view, |stack| {
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

    fn register_layer_toggle(
        engine: &mut Engine,
        pending_ops: &PendingOps,
        layer_view: &LayerView,
    ) {
        let ops = Arc::clone(pending_ops);
        let view = Arc::clone(layer_view);
        engine.register_fn(
            "layer_toggle",
            move |name: &str| -> std::result::Result<(), Box<EvalAltResult>> {
                let normalized = Self::normalize_layer_name(name, "layer_toggle")?;
                Self::with_layer_view(&view, |stack| {
                    let layer_id = stack.layer_id_by_name(&normalized).ok_or_else(|| {
                        Self::layer_error(
                            "layer_toggle",
                            format!("layer '{}' is not defined", normalized),
                        )
                    })?;

                    let toggled = stack.toggle(layer_id);
                    if !toggled {
                        return Err(Self::layer_error(
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
                let normalized = Self::normalize_layer_name(name, "is_layer_active")?;
                let active =
                    Self::with_layer_view(&view, |stack| Ok(stack.is_active_by_name(&normalized)))?;
                Ok(active)
            },
        );
    }

    fn layer_error(fn_name: &str, message: impl Into<String>) -> Box<EvalAltResult> {
        Box::new(EvalAltResult::ErrorRuntime(
            format!("{}: {}", fn_name, message.into()).into(),
            Position::NONE,
        ))
    }

    fn normalize_layer_name(name: &str, fn_name: &str) -> Result<String, Box<EvalAltResult>> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(Self::layer_error(fn_name, "layer name cannot be empty"));
        }
        if trimmed.contains(':') {
            return Err(Self::layer_error(fn_name, "layer name cannot contain ':'"));
        }
        Ok(trimmed.to_string())
    }

    fn with_layer_view<R, F>(view: &LayerView, f: F) -> Result<R, Box<EvalAltResult>>
    where
        F: FnOnce(&mut LayerStack) -> Result<R, Box<EvalAltResult>>,
    {
        let mut guard = view
            .lock()
            .map_err(|_| Self::layer_error("layer_view", "failed to lock layer view"))?;
        f(&mut guard)
    }

    fn ensure_layer_exists(
        view: &LayerView,
        name: &str,
        fn_name: &str,
    ) -> Result<(), Box<EvalAltResult>> {
        Self::with_layer_view(view, |stack| {
            if stack.layer_id_by_name(name).is_none() {
                Err(Self::layer_error(
                    fn_name,
                    format!("layer '{}' is not defined", name),
                ))
            } else {
                Ok(())
            }
        })
    }

    fn parse_layer_action(
        action: &str,
        fn_name: &str,
        view: &LayerView,
    ) -> Result<LayerMapAction, Box<EvalAltResult>> {
        let trimmed = action.trim();
        if trimmed.is_empty() {
            return Err(Self::layer_error(fn_name, "action cannot be empty"));
        }

        let lower = trimmed.to_ascii_lowercase();
        if lower == "block" {
            return Ok(LayerMapAction::Block);
        }
        if lower == "pass" {
            return Ok(LayerMapAction::Pass);
        }
        if lower == "layer_pop" {
            return Ok(LayerMapAction::LayerPop);
        }

        if lower.starts_with("remap:") {
            let target = trimmed["remap:".len()..].trim();
            let key = parse_key_or_error(target, fn_name)?;
            return Ok(LayerMapAction::Remap(key));
        }

        if lower.starts_with("layer_push:") {
            let target = trimmed["layer_push:".len()..].trim();
            let normalized = Self::normalize_layer_name(target, fn_name)?;
            Self::ensure_layer_exists(view, &normalized, fn_name)?;
            return Ok(LayerMapAction::LayerPush(normalized));
        }

        if lower.starts_with("layer_toggle:") {
            let target = trimmed["layer_toggle:".len()..].trim();
            let normalized = Self::normalize_layer_name(target, fn_name)?;
            Self::ensure_layer_exists(view, &normalized, fn_name)?;
            return Ok(LayerMapAction::LayerToggle(normalized));
        }

        if lower.starts_with("tap_hold:") {
            let parts: Vec<&str> = trimmed["tap_hold:".len()..].split(':').collect();
            if parts.len() != 2 {
                return Err(Self::layer_error(
                    fn_name,
                    "tap_hold action requires tap and hold values",
                ));
            }
            let tap = parse_key_or_error(parts[0].trim(), fn_name)?;
            let hold = parse_key_or_error(parts[1].trim(), fn_name)?;
            return Ok(LayerMapAction::TapHold {
                tap,
                hold: HoldAction::Key(hold),
            });
        }

        if lower.starts_with("tap_hold_mod:") {
            let parts: Vec<&str> = trimmed["tap_hold_mod:".len()..].split(':').collect();
            if parts.len() != 2 {
                return Err(Self::layer_error(
                    fn_name,
                    "tap_hold_mod action requires tap and modifier id",
                ));
            }
            let tap = parse_key_or_error(parts[0].trim(), fn_name)?;
            let modifier_id = parts[1].trim().parse::<u8>().map_err(|_| {
                Self::layer_error(
                    fn_name,
                    format!(
                        "tap_hold_mod modifier id must be 0-{}, got '{}'",
                        VirtualModifiers::MAX_ID,
                        parts[1].trim()
                    ),
                )
            })?;
            return Ok(LayerMapAction::TapHold {
                tap,
                hold: HoldAction::Modifier(modifier_id),
            });
        }

        // Default: treat as key remap.
        let remap_target = parse_key_or_error(trimmed, fn_name)?;
        Ok(LayerMapAction::Remap(remap_target))
    }

    fn parse_array_error(index: usize, value_type: &str) -> Box<EvalAltResult> {
        Box::new(EvalAltResult::ErrorRuntime(
            format!(
                "combo: keys must be strings, got {} at index {}",
                value_type, index
            )
            .into(),
            Position::NONE,
        ))
    }

    fn parse_keys_array(keys: Array) -> Result<Vec<KeyCode>, Box<EvalAltResult>> {
        let mut parsed = Vec::with_capacity(keys.len());

        for (idx, value) in keys.into_iter().enumerate() {
            let key_name = value
                .clone()
                .try_cast::<String>()
                .ok_or_else(|| Self::parse_array_error(idx, value.type_name()))?;

            let key_code = parse_key_or_error(&key_name, "combo")?;
            parsed.push(key_code);
        }

        Ok(parsed)
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
                }
            }
        }

        if let Ok(mut view) = self.layer_view.lock() {
            *view = self.registry.layers().clone();
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
    use crate::engine::{HoldAction, LayerAction, RemapAction};

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
}
