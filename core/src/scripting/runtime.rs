//! Rhai runtime implementation.
//!
//! This module contains the core `RhaiRuntime` struct and the `ScriptRuntime`
//! trait implementation. Function bindings are in `bindings.rs` and helper
//! types are in `builtins.rs`.

use super::bindings::register_all_functions;
use super::builtins::{LayerView, ModifierPreview, ModifierView, PendingOps};
use super::cache::ScriptCache;
use super::pending_ops::PendingOpsApplier;
use super::registry_sync::RegistrySyncer;
use super::row_col_resolver::RowColResolver;
use super::sandbox::{ResourceConfig, SandboxError, ScriptSandbox};
use crate::config::script_cache_dir;
use crate::discovery::storage::read_profile;
use crate::discovery::types::DeviceId;
use crate::engine::{KeyCode, LayerStack};
use crate::errors::{runtime::*, KeyrxError};
use crate::keyrx_err;
use crate::scripting::RemapRegistry;
use crate::traits::ScriptRuntime;
use rhai::{Engine, Scope, AST};
use std::collections::HashSet;
use std::fs;
use std::sync::{Arc, Mutex};

/// Production Rhai script runtime.
///
/// Uses a pending operations pattern to avoid `Rc<RefCell>`:
/// - Script functions push operations to a shared `Arc<Mutex<Vec>>`
/// - After script execution, operations are applied to the owned registry
///
/// The runtime integrates with ScriptSandbox to enforce:
/// - Capability-based function access control
/// - Resource limits (CPU, memory, recursion)
/// - Input validation
pub struct RhaiRuntime {
    engine: Engine,
    ast: Option<AST>,
    cache: Option<ScriptCache>,
    defined_hooks: HashSet<String>,
    registry: RemapRegistry,
    pending_ops: PendingOps,
    syncer: RegistrySyncer,
    sandbox: Arc<ScriptSandbox>,
    resolver: Arc<RowColResolver>,
}

impl RhaiRuntime {
    /// Create a new Rhai runtime.
    pub fn new() -> Result<Self, KeyrxError> {
        Self::with_config(ResourceConfig::default())
    }

    /// Create a new Rhai runtime with custom resource limits.
    pub fn with_config(config: ResourceConfig) -> Result<Self, KeyrxError> {
        Self::with_config_and_cache(config, Some(default_cache()))
    }

    /// Create a new Rhai runtime with a provided cache.
    pub fn with_cache(cache: ScriptCache) -> Result<Self, KeyrxError> {
        Self::with_config_and_cache(ResourceConfig::default(), Some(cache))
    }

    /// Disable script caching for this runtime.
    pub fn disable_cache(&mut self) {
        self.cache = None;
    }

    /// Access the script cache if enabled.
    pub fn script_cache(&self) -> Option<&ScriptCache> {
        self.cache.as_ref()
    }

    fn with_config_and_cache(
        config: ResourceConfig,
        cache: Option<ScriptCache>,
    ) -> Result<Self, KeyrxError> {
        let mut engine = Engine::new();

        // Create sandbox with custom configuration
        let sandbox = ScriptSandbox::default();
        sandbox.configure_engine(&mut engine);

        // Apply custom resource limits
        engine.set_max_operations(config.max_instructions);
        engine.set_max_call_levels(config.max_recursion as usize);

        // Shared pending operations storage
        let pending_ops: PendingOps = Arc::new(Mutex::new(Vec::new()));
        let layer_view: LayerView = Arc::new(Mutex::new(LayerStack::new()));
        let modifier_view: ModifierView = Arc::new(Mutex::new(ModifierPreview::new()));

        // Row-column resolver (starts without profile, can be loaded later)
        let resolver = Arc::new(RowColResolver::without_profile());

        // Register script functions
        register_all_functions(
            &mut engine,
            &pending_ops,
            &layer_view,
            &modifier_view,
            &resolver,
        );

        let syncer = RegistrySyncer::new(layer_view, modifier_view);

        Ok(Self {
            engine,
            ast: None,
            cache,
            defined_hooks: HashSet::new(),
            registry: RemapRegistry::new(),
            pending_ops,
            syncer,
            sandbox: Arc::new(sandbox),
            resolver,
        })
    }

    fn load_script_from_cache(&mut self, script: &str) -> bool {
        if let Some(cache) = &self.cache {
            if let Some(ast) = cache.get(script) {
                tracing::debug!(
                    service = "keyrx",
                    event = "script_cache_hit",
                    component = "rhai_runtime",
                    "Loaded script from cache"
                );
                self.ast = Some(ast);
                self.scan_for_hooks();
                return true;
            }
        }

        false
    }

    /// Load device profile for row-column API support.
    ///
    /// This enables the `*_rc()` functions to resolve (row, col) positions to KeyCodes.
    /// If the profile cannot be loaded, row-column functions will fail with helpful errors.
    ///
    /// # Arguments
    /// * `device_id` - Device identifier (vendor_id, product_id)
    ///
    /// # Returns
    /// * `Ok(())` - Profile loaded successfully
    /// * `Err(KeyrxError)` - Profile not found or invalid
    pub fn load_device_profile(&mut self, device_id: DeviceId) -> Result<(), KeyrxError> {
        let profile = read_profile(device_id)
            .map_err(|e| keyrx_err!(SCRIPT_COMPILATION_FAILED, error = e.to_string()))?;

        tracing::debug!(
            service = "keyrx",
            event = "device_profile_loaded",
            component = "rhai_runtime",
            device = ?device_id,
            name = ?profile.name,
            rows = profile.rows,
            "Device profile loaded for row-column API"
        );

        // Load profile into the existing resolver (which is shared with registered functions)
        self.resolver.load_profile(Arc::new(profile));

        Ok(())
    }

    /// Get the row-column resolver.
    pub fn resolver(&self) -> &RowColResolver {
        &self.resolver
    }

    /// Get a reference to the sandbox.
    pub fn sandbox(&self) -> &ScriptSandbox {
        &self.sandbox
    }

    /// Check if a function is allowed in the current sandbox mode.
    ///
    /// This can be used to pre-validate function calls before execution.
    pub fn check_function_allowed(&self, function_name: &str) -> Result<(), SandboxError> {
        self.sandbox.check_function_allowed(function_name)
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

    /// Apply pending operations to the registry and sync views.
    fn apply_pending_ops(&mut self) {
        let mut applier = PendingOpsApplier::new(&mut self.registry);
        applier.apply_all(&self.pending_ops, &mut self.syncer);
    }

    /// Get a reference to the remap registry.
    pub fn registry(&self) -> &RemapRegistry {
        &self.registry
    }

    /// Get a mutable reference to the Rhai engine.
    ///
    /// This allows registering additional functions (e.g., test primitives)
    /// before executing scripts.
    pub fn engine_mut(&mut self) -> &mut Engine {
        &mut self.engine
    }
}

fn default_cache() -> ScriptCache {
    ScriptCache::new(script_cache_dir())
}

impl ScriptRuntime for RhaiRuntime {
    fn execute(&mut self, script: &str) -> Result<(), KeyrxError> {
        // Check resources before execution
        self.sandbox
            .check_resources()
            .map_err(|e| keyrx_err!(SCRIPT_EXECUTION_FAILED, error = e.to_string()))?;

        self.engine
            .run(script)
            .map_err(|e| keyrx_err!(SCRIPT_EXECUTION_FAILED, error = e.to_string()))?;

        self.apply_pending_ops();
        Ok(())
    }

    fn call_hook(&mut self, hook: &str) -> Result<(), KeyrxError> {
        // Check resources before execution
        self.sandbox
            .check_resources()
            .map_err(|e| keyrx_err!(SCRIPT_EXECUTION_FAILED, error = e.to_string()))?;

        let ast = self
            .ast
            .as_ref()
            .ok_or_else(|| keyrx_err!(SCRIPT_HOOK_NOT_FOUND, hook = hook))?;

        self.engine
            .call_fn::<()>(&mut Scope::new(), ast, hook, ())
            .map_err(|e| {
                keyrx_err!(
                    SCRIPT_EXECUTION_FAILED,
                    error = format!("Hook '{}' call failed: {}", hook, e)
                )
            })?;

        self.apply_pending_ops();
        Ok(())
    }

    fn load_file(&mut self, path: &str) -> Result<(), KeyrxError> {
        let script = fs::read_to_string(path).map_err(|e| {
            keyrx_err!(
                SCRIPT_COMPILATION_FAILED,
                error = format!("{} ({})", e, path)
            )
        })?;

        if self.load_script_from_cache(&script) {
            return Ok(());
        }

        let ast = self.engine.compile(&script).map_err(|e| {
            keyrx_err!(
                SCRIPT_COMPILATION_FAILED,
                error = format!("{} ({})", e, path)
            )
        })?;

        if let Some(cache) = &self.cache {
            cache.put(&script, &ast);
        }

        self.ast = Some(ast);
        self.scan_for_hooks();
        Ok(())
    }

    fn run_script(&mut self) -> Result<(), KeyrxError> {
        // Check resources before execution
        self.sandbox
            .check_resources()
            .map_err(|e| keyrx_err!(SCRIPT_EXECUTION_FAILED, error = e.to_string()))?;

        let ast = self
            .ast
            .as_ref()
            .ok_or_else(|| keyrx_err!(SCRIPT_HOOK_NOT_FOUND, hook = "script"))?;

        self.engine
            .run_ast(ast)
            .map_err(|e| keyrx_err!(SCRIPT_EXECUTION_FAILED, error = e.to_string()))?;

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
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn new_runtime_has_empty_registry() {
        let runtime = RhaiRuntime::new().unwrap();
        assert_eq!(runtime.lookup_remap(KeyCode::A), RemapAction::Pass);
    }

    #[test]
    fn load_file_hits_cache_on_second_load() {
        let script_dir = tempdir().unwrap();
        let cache_dir = tempdir().unwrap();
        let script_path = script_dir.path().join("script.rhai");

        fs::write(
            &script_path,
            r#"
remap("A", "B");
"#,
        )
        .unwrap();

        let mut runtime =
            RhaiRuntime::with_cache(ScriptCache::new(cache_dir.path().to_path_buf())).unwrap();

        runtime
            .load_file(script_path.to_str().expect("utf-8 path"))
            .unwrap();
        let first_stats = runtime.script_cache().unwrap().stats();
        assert_eq!(first_stats.misses, 1);
        assert_eq!(first_stats.hits, 0);
        assert_eq!(first_stats.entries, 1);

        runtime
            .load_file(script_path.to_str().expect("utf-8 path"))
            .unwrap();
        let second_stats = runtime.script_cache().unwrap().stats();
        assert_eq!(second_stats.hits, 1);
        assert_eq!(second_stats.entries, 1);
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
