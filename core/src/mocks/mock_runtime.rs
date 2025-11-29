//! Mock script runtime for testing.

use crate::engine::{KeyCode, RemapAction};
use crate::scripting::RemapRegistry;
use crate::traits::ScriptRuntime;
use anyhow::Result;
use std::collections::HashSet;

/// Mock script runtime for testing without real Rhai execution.
pub struct MockRuntime {
    /// Defined hooks.
    defined_hooks: HashSet<String>,
    /// Whether hooks should succeed.
    hooks_succeed: bool,
    /// Registry for key remappings.
    registry: RemapRegistry,
}

impl MockRuntime {
    /// Create a new mock runtime.
    pub fn new() -> Self {
        Self {
            defined_hooks: HashSet::new(),
            hooks_succeed: true,
            registry: RemapRegistry::new(),
        }
    }

    /// Define a hook.
    pub fn define_hook(&mut self, hook: &str) {
        self.defined_hooks.insert(hook.to_string());
    }

    /// Set whether hook calls should succeed.
    pub fn set_hooks_succeed(&mut self, succeed: bool) {
        self.hooks_succeed = succeed;
    }

    /// Get mutable access to the registry for test setup.
    pub fn registry_mut(&mut self) -> &mut RemapRegistry {
        &mut self.registry
    }
}

impl Default for MockRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptRuntime for MockRuntime {
    fn execute(&mut self, _script: &str) -> Result<()> {
        Ok(())
    }

    fn call_hook(&mut self, hook: &str) -> Result<()> {
        if self.hooks_succeed && self.defined_hooks.contains(hook) {
            Ok(())
        } else if !self.defined_hooks.contains(hook) {
            anyhow::bail!("Hook '{}' not defined", hook)
        } else {
            anyhow::bail!("Hook '{}' failed", hook)
        }
    }

    fn load_file(&mut self, _path: &str) -> Result<()> {
        Ok(())
    }

    fn run_script(&mut self) -> Result<()> {
        // Mock runtime doesn't execute real scripts
        Ok(())
    }

    fn has_hook(&self, hook: &str) -> bool {
        self.defined_hooks.contains(hook)
    }

    fn lookup_remap(&self, key: KeyCode) -> RemapAction {
        self.registry.lookup(key)
    }
}
