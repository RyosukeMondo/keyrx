//! Rhai runtime implementation.

use crate::traits::ScriptRuntime;
use anyhow::{anyhow, Result};
use rhai::{Engine, AST};
use std::collections::HashSet;

/// Production Rhai script runtime.
pub struct RhaiRuntime {
    engine: Engine,
    ast: Option<AST>,
    defined_hooks: HashSet<String>,
}

impl RhaiRuntime {
    /// Create a new Rhai runtime.
    pub fn new() -> Result<Self> {
        let mut engine = Engine::new();

        // Sandbox: disable dangerous operations
        engine.set_max_expr_depths(64, 64);
        engine.set_max_operations(100_000);

        // Register core functions
        engine.register_fn("print_debug", |msg: &str| {
            tracing::debug!("{}", msg);
        });

        Ok(Self {
            engine,
            ast: None,
            defined_hooks: HashSet::new(),
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
}
