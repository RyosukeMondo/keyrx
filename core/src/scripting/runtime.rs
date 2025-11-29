//! Rhai runtime implementation.

use crate::engine::KeyCode;
use crate::scripting::RemapRegistry;
use crate::traits::ScriptRuntime;
use anyhow::{anyhow, Result};
use rhai::{Engine, AST};
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

/// Production Rhai script runtime.
pub struct RhaiRuntime {
    engine: Engine,
    ast: Option<AST>,
    defined_hooks: HashSet<String>,
    registry: Rc<RefCell<RemapRegistry>>,
}

impl RhaiRuntime {
    /// Create a new Rhai runtime.
    pub fn new() -> Result<Self> {
        let mut engine = Engine::new();

        // Sandbox: disable dangerous operations
        engine.set_max_expr_depths(64, 64);
        engine.set_max_operations(100_000);

        // Create shared registry for script access
        let registry = Rc::new(RefCell::new(RemapRegistry::new()));

        // Register core functions
        engine.register_fn("print_debug", |msg: &str| {
            tracing::debug!("{}", msg);
        });

        // Register remap function: remap(from, to)
        let registry_clone = Rc::clone(&registry);
        engine.register_fn("remap", move |from: &str, to: &str| {
            let from_key = match KeyCode::from_name(from) {
                Some(k) => k,
                None => {
                    tracing::warn!("Unknown key in remap(): '{}'", from);
                    return;
                }
            };
            let to_key = match KeyCode::from_name(to) {
                Some(k) => k,
                None => {
                    tracing::warn!("Unknown key in remap(): '{}'", to);
                    return;
                }
            };
            registry_clone.borrow_mut().remap(from_key, to_key);
            tracing::debug!("Registered remap: {} -> {}", from, to);
        });

        // Register block function: block(key)
        let registry_clone = Rc::clone(&registry);
        engine.register_fn("block", move |key: &str| {
            let key_code = match KeyCode::from_name(key) {
                Some(k) => k,
                None => {
                    tracing::warn!("Unknown key in block(): '{}'", key);
                    return;
                }
            };
            registry_clone.borrow_mut().block(key_code);
            tracing::debug!("Registered block: {}", key);
        });

        // Register pass function: pass(key)
        let registry_clone = Rc::clone(&registry);
        engine.register_fn("pass", move |key: &str| {
            let key_code = match KeyCode::from_name(key) {
                Some(k) => k,
                None => {
                    tracing::warn!("Unknown key in pass(): '{}'", key);
                    return;
                }
            };
            registry_clone.borrow_mut().pass(key_code);
            tracing::debug!("Registered pass: {}", key);
        });

        Ok(Self {
            engine,
            ast: None,
            defined_hooks: HashSet::new(),
            registry,
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

    /// Get a reference to the remap registry.
    ///
    /// Returns a clone of the internal `Rc<RefCell<RemapRegistry>>` so callers
    /// can access the registry with interior mutability.
    pub fn registry(&self) -> Rc<RefCell<RemapRegistry>> {
        Rc::clone(&self.registry)
    }
}

impl Default for RhaiRuntime {
    fn default() -> Self {
        Self::new().expect("Failed to create RhaiRuntime")
    }
}

impl ScriptRuntime for RhaiRuntime {
    fn execute(&mut self, script: &str) -> Result<()> {
        self.engine
            .run(script)
            .map_err(|e| anyhow!("Script execution failed: {}", e))
    }

    fn call_hook(&mut self, hook: &str) -> Result<()> {
        let ast = self.ast.as_ref().ok_or_else(|| anyhow!("No script loaded"))?;

        self.engine
            .call_fn::<()>(&mut rhai::Scope::new(), ast, hook, ())
            .map_err(|e| anyhow!("Hook '{}' call failed: {}", hook, e))
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

    fn has_hook(&self, hook: &str) -> bool {
        self.defined_hooks.contains(hook)
    }

    fn lookup_remap(&self, key: KeyCode) -> crate::engine::RemapAction {
        self.registry.borrow().lookup(key)
    }
}
