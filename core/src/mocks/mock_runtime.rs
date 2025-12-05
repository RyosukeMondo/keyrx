//! Mock script runtime for testing.

use crate::engine::{KeyCode, RemapAction};
use crate::errors::{runtime::*, KeyrxError};
use crate::keyrx_err;
use crate::scripting::RemapRegistry;
use crate::traits::ScriptRuntime;
use std::collections::HashSet;
use std::time::Duration;

/// Represents a recorded method call on MockRuntime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MockRuntimeCall {
    /// execute() was called with the given script.
    Execute(String),
    /// call_hook() was called with the given hook name.
    CallHook(String),
    /// load_file() was called with the given path.
    LoadFile(String),
    /// run_script() was called.
    RunScript,
    /// has_hook() was called with the given hook name.
    HasHook(String),
    /// lookup_remap() was called with the given key.
    LookupRemap(KeyCode),
}

/// Mock script runtime for testing without real Rhai execution.
pub struct MockRuntime {
    /// Defined hooks.
    defined_hooks: HashSet<String>,
    /// Whether hooks should succeed.
    hooks_succeed: bool,
    /// Registry for key remappings.
    registry: RemapRegistry,
    /// History of all method calls for verification.
    call_history: Vec<MockRuntimeCall>,
    /// Optional delay to simulate slow lookups.
    lookup_delay: Duration,
}

impl MockRuntime {
    /// Create a new mock runtime.
    pub fn new() -> Self {
        Self {
            defined_hooks: HashSet::new(),
            hooks_succeed: true,
            registry: RemapRegistry::new(),
            call_history: Vec::new(),
            lookup_delay: Duration::ZERO,
        }
    }

    /// Configure a key remapping.
    ///
    /// # Example
    /// ```ignore
    /// let mock = MockRuntime::new()
    ///     .with_remap(KeyCode::CapsLock, KeyCode::Escape);
    /// ```
    pub fn with_remap(mut self, from: KeyCode, to: KeyCode) -> Self {
        self.registry.remap(from, to);
        self
    }

    /// Configure a key to be blocked.
    ///
    /// # Example
    /// ```ignore
    /// let mock = MockRuntime::new()
    ///     .with_block(KeyCode::CapsLock);
    /// ```
    pub fn with_block(mut self, key: KeyCode) -> Self {
        self.registry.block(key);
        self
    }

    /// Configure a hook to be defined.
    ///
    /// # Example
    /// ```ignore
    /// let mock = MockRuntime::new()
    ///     .with_hook("on_init");
    /// ```
    pub fn with_hook(mut self, hook: impl Into<String>) -> Self {
        self.defined_hooks.insert(hook.into());
        self
    }

    /// Define a hook (mutable version).
    pub fn define_hook(&mut self, hook: &str) {
        self.defined_hooks.insert(hook.to_string());
    }

    /// Configure a simulated delay for remap lookups.
    pub fn with_lookup_delay(mut self, delay: Duration) -> Self {
        self.lookup_delay = delay;
        self
    }

    /// Set whether hook calls should succeed.
    pub fn set_hooks_succeed(&mut self, succeed: bool) {
        self.hooks_succeed = succeed;
    }

    /// Get mutable access to the registry for test setup.
    pub fn registry_mut(&mut self) -> &mut RemapRegistry {
        &mut self.registry
    }

    /// Get the history of all method calls.
    ///
    /// Useful for verifying the order and types of operations performed.
    pub fn call_history(&self) -> &[MockRuntimeCall] {
        &self.call_history
    }

    /// Clear the call history.
    pub fn clear_call_history(&mut self) {
        self.call_history.clear();
    }
}

impl Default for MockRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptRuntime for MockRuntime {
    fn execute(&mut self, script: &str) -> Result<(), KeyrxError> {
        self.call_history
            .push(MockRuntimeCall::Execute(script.to_string()));
        Ok(())
    }

    fn call_hook(&mut self, hook: &str) -> Result<(), KeyrxError> {
        self.call_history
            .push(MockRuntimeCall::CallHook(hook.to_string()));
        if self.hooks_succeed && self.defined_hooks.contains(hook) {
            Ok(())
        } else if !self.defined_hooks.contains(hook) {
            Err(keyrx_err!(SCRIPT_HOOK_NOT_FOUND, hook = hook))
        } else {
            Err(keyrx_err!(
                SCRIPT_EXECUTION_FAILED,
                error = format!("Hook '{}' failed", hook)
            ))
        }
    }

    fn load_file(&mut self, path: &str) -> Result<(), KeyrxError> {
        self.call_history
            .push(MockRuntimeCall::LoadFile(path.to_string()));
        Ok(())
    }

    fn run_script(&mut self) -> Result<(), KeyrxError> {
        self.call_history.push(MockRuntimeCall::RunScript);
        // Mock runtime doesn't execute real scripts
        Ok(())
    }

    fn has_hook(&self, hook: &str) -> bool {
        // Note: has_hook is &self, so we can't record it in call_history
        // without interior mutability. For simplicity, we skip recording this.
        self.defined_hooks.contains(hook)
    }

    fn lookup_remap(&self, key: KeyCode) -> RemapAction {
        // Note: lookup_remap is &self, so we can't record it in call_history
        // without interior mutability. For simplicity, we skip recording this.
        if !self.lookup_delay.is_zero() {
            std::thread::sleep(self.lookup_delay);
        }
        self.registry.lookup(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call_tracking_execute() {
        let mut runtime = MockRuntime::new();
        runtime.execute("test script").unwrap();

        assert_eq!(
            runtime.call_history(),
            &[MockRuntimeCall::Execute("test script".to_string())]
        );
    }

    #[test]
    fn test_call_tracking_multiple_calls() {
        let mut runtime = MockRuntime::new();
        runtime.define_hook("on_init");

        runtime.load_file("test.rhai").unwrap();
        runtime.run_script().unwrap();
        runtime.call_hook("on_init").unwrap();

        assert_eq!(
            runtime.call_history(),
            &[
                MockRuntimeCall::LoadFile("test.rhai".to_string()),
                MockRuntimeCall::RunScript,
                MockRuntimeCall::CallHook("on_init".to_string()),
            ]
        );
    }

    #[test]
    fn test_with_remap_builder() {
        let runtime = MockRuntime::new().with_remap(KeyCode::CapsLock, KeyCode::Escape);

        assert_eq!(
            runtime.lookup_remap(KeyCode::CapsLock),
            RemapAction::Remap(KeyCode::Escape)
        );
    }

    #[test]
    fn test_with_block_builder() {
        let runtime = MockRuntime::new().with_block(KeyCode::CapsLock);

        assert_eq!(runtime.lookup_remap(KeyCode::CapsLock), RemapAction::Block);
    }

    #[test]
    fn test_with_hook_builder() {
        let runtime = MockRuntime::new().with_hook("on_init");

        assert!(runtime.has_hook("on_init"));
        assert!(!runtime.has_hook("on_exit"));
    }

    #[test]
    fn test_chained_builders() {
        let runtime = MockRuntime::new()
            .with_remap(KeyCode::A, KeyCode::B)
            .with_block(KeyCode::CapsLock)
            .with_hook("on_init");

        assert_eq!(
            runtime.lookup_remap(KeyCode::A),
            RemapAction::Remap(KeyCode::B)
        );
        assert_eq!(runtime.lookup_remap(KeyCode::CapsLock), RemapAction::Block);
        assert!(runtime.has_hook("on_init"));
    }

    #[test]
    fn test_clear_call_history() {
        let mut runtime = MockRuntime::new();
        runtime.execute("test").unwrap();
        assert!(!runtime.call_history().is_empty());

        runtime.clear_call_history();
        assert!(runtime.call_history().is_empty());
    }

    #[test]
    fn test_unmapped_key_returns_pass() {
        let runtime = MockRuntime::new();
        assert_eq!(runtime.lookup_remap(KeyCode::A), RemapAction::Pass);
    }
}
