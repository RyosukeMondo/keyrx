//! Script runtime trait for Rhai integration.

use crate::engine::{KeyCode, RemapAction};
use anyhow::Result;

/// Trait for script execution runtime.
///
/// Implementations:
/// - `RhaiRuntime`: Production Rhai engine
/// - `MockRuntime`: Test mock for script simulation
pub trait ScriptRuntime {
    /// Execute a script string and return success/failure.
    fn execute(&mut self, script: &str) -> Result<()>;

    /// Call a named hook function.
    fn call_hook(&mut self, hook: &str) -> Result<()>;

    /// Load a script file.
    fn load_file(&mut self, path: &str) -> Result<()>;

    /// Check if a hook is defined.
    fn has_hook(&self, hook: &str) -> bool;

    /// Look up the remap action for a key.
    ///
    /// Returns the action to take for the given key code.
    /// Implementations should return `RemapAction::Pass` for unmapped keys.
    fn lookup_remap(&self, key: KeyCode) -> RemapAction;
}
