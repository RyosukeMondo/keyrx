//! Rhai runtime implementation.

use crate::engine::KeyCode;
use crate::scripting::RemapRegistry;
use crate::traits::ScriptRuntime;
use anyhow::{anyhow, Result};
use rhai::{Engine, Scope, AST};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

/// Thread-safe pending operations storage.
/// This uses Arc<Mutex> instead of Rc<RefCell> for thread safety
/// and to allow the closure to own a reference.
type PendingOps = Arc<Mutex<Vec<PendingOp>>>;

/// A pending operation to be applied to the registry after script execution.
#[derive(Debug, Clone)]
enum PendingOp {
    Remap { from: KeyCode, to: KeyCode },
    Block { key: KeyCode },
    Pass { key: KeyCode },
}

/// Production Rhai script runtime.
///
/// Uses a pending operations pattern to avoid `Rc<RefCell>`:
/// - Script functions push operations to a shared Arc<Mutex<Vec>>
/// - After script execution, operations are applied to the owned registry
pub struct RhaiRuntime {
    engine: Engine,
    ast: Option<AST>,
    defined_hooks: HashSet<String>,
    registry: RemapRegistry,
    pending_ops: PendingOps,
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

        // Register core functions
        engine.register_fn("print_debug", |msg: &str| {
            tracing::debug!("{}", msg);
        });

        // Register remap function: remap(from, to)
        let ops = Arc::clone(&pending_ops);
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
            tracing::debug!("Registered remap: {} -> {}", from, to);
            if let Ok(mut ops) = ops.lock() {
                ops.push(PendingOp::Remap {
                    from: from_key,
                    to: to_key,
                });
            }
        });

        // Register block function: block(key)
        let ops = Arc::clone(&pending_ops);
        engine.register_fn("block", move |key: &str| {
            let key_code = match KeyCode::from_name(key) {
                Some(k) => k,
                None => {
                    tracing::warn!("Unknown key in block(): '{}'", key);
                    return;
                }
            };
            tracing::debug!("Registered block: {}", key);
            if let Ok(mut ops) = ops.lock() {
                ops.push(PendingOp::Block { key: key_code });
            }
        });

        // Register pass function: pass(key)
        let ops = Arc::clone(&pending_ops);
        engine.register_fn("pass", move |key: &str| {
            let key_code = match KeyCode::from_name(key) {
                Some(k) => k,
                None => {
                    tracing::warn!("Unknown key in pass(): '{}'", key);
                    return;
                }
            };
            tracing::debug!("Registered pass: {}", key);
            if let Ok(mut ops) = ops.lock() {
                ops.push(PendingOp::Pass { key: key_code });
            }
        });

        Ok(Self {
            engine,
            ast: None,
            defined_hooks: HashSet::new(),
            registry: RemapRegistry::new(),
            pending_ops,
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
                }
            }
        }
    }

    /// Get a reference to the remap registry.
    pub fn registry(&self) -> &RemapRegistry {
        &self.registry
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
            .map_err(|e| anyhow!("Script execution failed: {}", e))?;

        self.apply_pending_ops();
        Ok(())
    }

    fn call_hook(&mut self, hook: &str) -> Result<()> {
        let ast = self.ast.as_ref().ok_or_else(|| anyhow!("No script loaded"))?;

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
    use crate::engine::RemapAction;

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
    fn unknown_key_logs_warning_but_continues() {
        let mut runtime = RhaiRuntime::new().unwrap();
        // Should not panic, just log warning
        runtime.execute(r#"remap("InvalidKey", "B");"#).unwrap();
        runtime.execute(r#"remap("A", "InvalidKey");"#).unwrap();
        // Valid mappings should still work
        runtime.execute(r#"remap("C", "D");"#).unwrap();
        assert_eq!(
            runtime.lookup_remap(KeyCode::C),
            RemapAction::Remap(KeyCode::D)
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
}
