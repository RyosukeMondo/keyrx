//! Rhai runtime implementation.
//!
//! This module contains the core `RhaiRuntime` struct and the `ScriptRuntime`
//! trait implementation. Function bindings are in `bindings.rs` and helper
//! types are in `builtins.rs`.

use super::bindings::register_all_functions;
use super::builtins::PendingOp;
use super::builtins::{LayerView, ModifierPreview, ModifierView, PendingOps};
use super::cache::ScriptCache;
use super::pending_ops::PendingOpsApplier;
use super::registry_sync::RegistrySyncer;
use super::row_col_resolver::RowColResolver;
use super::sandbox::{ResourceConfig, SandboxError, ScriptSandbox};
use crate::config::script_cache_dir;
use crate::discovery::storage::read_profile;
use crate::discovery::types::DeviceId;
use crate::engine::{
    KeyCode, LayerAction, LayerStack, ModifierState, RemapAction, ResourceEnforcer,
    ResourceLimitError, ResourceLimits, TimingConfig,
};
use crate::errors::{runtime::*, KeyrxError};
use crate::keyrx_err;
use crate::scripting::registry::TapHoldBinding;
use crate::scripting::RemapRegistry;
use crate::traits::ScriptRuntime;
use rhai::{Engine, Scope, AST};
use smallvec::SmallVec;
use std::collections::HashSet;
use std::fs;
use std::mem::size_of;
use std::sync::{Arc, Mutex};
use std::time::Instant;

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
    resource_enforcer: Arc<ResourceEnforcer>,
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
            resource_enforcer: Arc::new(ResourceEnforcer::new(ResourceLimits::default())),
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

    /// Replace the shared resource enforcer (used by the engine for unified limits).
    pub fn set_resource_enforcer(&mut self, enforcer: Arc<ResourceEnforcer>) {
        self.resource_enforcer = enforcer;
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
    fn apply_pending_ops(&mut self) -> Result<(), KeyrxError> {
        // Take a snapshot of pending ops for memory estimation before draining.
        let pending_snapshot = {
            let guard = self.pending_ops.lock().map_err(|_| {
                keyrx_err!(SCRIPT_EXECUTION_FAILED, error = "pending ops lock poisoned")
            });

            match guard {
                Ok(ops) => ops.clone(),
                Err(err) => {
                    // Propagate as script execution failure to align with other runtime errors.
                    return Err(err);
                }
            }
        };

        // Enforce memory limits before mutating registry.
        self.enforce_memory_limit(&pending_snapshot)?;

        let mut applier = PendingOpsApplier::new(&mut self.registry);
        applier.apply_all(&self.pending_ops, &mut self.syncer);
        self.synchronize_memory_accounting()?;
        Ok(())
    }

    fn enforce_memory_limit(&self, pending_ops: &[PendingOp]) -> Result<(), KeyrxError> {
        let current_usage = Self::estimate_registry_usage(&self.registry);
        let projected_usage =
            current_usage.saturating_add(Self::estimate_pending_ops_memory(pending_ops));
        let snapshot = self.resource_enforcer.snapshot();

        if projected_usage > snapshot.memory_limit {
            let error = ResourceLimitError::Memory {
                used: projected_usage,
                limit: snapshot.memory_limit,
            };
            return Err(keyrx_err!(
                SCRIPT_EXECUTION_FAILED,
                error = error.to_string()
            ));
        }

        Ok(())
    }

    fn synchronize_memory_accounting(&self) -> Result<(), KeyrxError> {
        let actual_usage = Self::estimate_registry_usage(&self.registry);
        let snapshot = self.resource_enforcer.snapshot();

        if actual_usage > snapshot.memory_used {
            self.resource_enforcer
                .record_allocation(actual_usage - snapshot.memory_used)
                .map_err(|e| keyrx_err!(SCRIPT_EXECUTION_FAILED, error = e.to_string()))
        } else {
            self.resource_enforcer
                .record_deallocation(snapshot.memory_used.saturating_sub(actual_usage));
            Ok(())
        }
    }

    fn estimate_pending_ops_memory(ops: &[PendingOp]) -> usize {
        ops.iter()
            .map(|op| match op {
                PendingOp::Remap { .. }
                | PendingOp::Block { .. }
                | PendingOp::Pass { .. }
                | PendingOp::LayerPop
                | PendingOp::SetTiming(_) => size_of::<PendingOp>(),
                PendingOp::TapHold { .. } => size_of::<PendingOp>(),
                PendingOp::Combo { keys, .. } => {
                    size_of::<PendingOp>()
                        + keys.len() * size_of::<KeyCode>()
                        + size_of::<LayerAction>()
                }
                PendingOp::LayerDefine { name, .. }
                | PendingOp::LayerPush { name }
                | PendingOp::LayerToggle { name }
                | PendingOp::DefineModifier { name, .. }
                | PendingOp::ModifierActivate { name, .. }
                | PendingOp::ModifierDeactivate { name, .. }
                | PendingOp::ModifierOneShot { name, .. } => {
                    size_of::<PendingOp>() + name.len() + size_of::<usize>()
                }
                PendingOp::LayerMap { layer, .. } => {
                    size_of::<PendingOp>() + layer.len() + size_of::<LayerAction>()
                }
            })
            .sum()
    }

    fn estimate_registry_usage(registry: &RemapRegistry) -> usize {
        let mapping_bytes = registry.mappings().count() * size_of::<(KeyCode, RemapAction)>();

        let tap_hold_bytes = registry.tap_holds().count() * size_of::<(KeyCode, TapHoldBinding)>();

        let combo_bytes = registry
            .combos()
            .all()
            .map(|def| {
                size_of::<SmallVec<[KeyCode; 4]>>()
                    + def.keys.len() * size_of::<KeyCode>()
                    + size_of::<LayerAction>()
            })
            .sum::<usize>();

        let modifier_bytes = registry
            .modifier_names()
            .keys()
            .map(|name| size_of::<String>() + name.len() + size_of::<u8>())
            .sum::<usize>()
            + size_of::<ModifierState>();

        let timing_bytes = size_of::<TimingConfig>();
        let layer_bytes = size_of::<LayerStack>()
            + registry.layers().active_layers().len() * size_of::<LayerAction>();

        mapping_bytes + tap_hold_bytes + combo_bytes + modifier_bytes + timing_bytes + layer_bytes
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

        self.apply_pending_ops()?;
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

        self.apply_pending_ops()?;
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

        let compile_start = Instant::now();
        let ast = self.engine.compile(&script).map_err(|e| {
            keyrx_err!(
                SCRIPT_COMPILATION_FAILED,
                error = format!("{} ({})", e, path)
            )
        })?;
        let compile_micros = compile_start.elapsed().as_micros() as u64;

        if let Some(cache) = &self.cache {
            cache.put(&script, &ast, Some(compile_micros));
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

        self.apply_pending_ops()?;
        Ok(())
    }

    fn has_hook(&self, hook: &str) -> bool {
        self.defined_hooks.contains(hook)
    }

    fn lookup_remap(&self, key: KeyCode) -> crate::engine::RemapAction {
        self.registry.lookup(key)
    }
}
