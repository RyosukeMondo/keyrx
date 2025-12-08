//! Script runtime trait for Rhai-based keyboard remapping scripts.
//!
//! This module defines the [`ScriptRuntime`] trait, which abstracts over script
//! execution engines. The primary implementation is [`RhaiRuntime`](crate::scripting::RhaiRuntime),
//! which uses the Rhai scripting language to define keyboard remappings.
//!
//! # Method Call Order
//!
//! The `ScriptRuntime` trait methods must be called in a specific order for
//! file-based scripts:
//!
//! 1. [`load_file`](ScriptRuntime::load_file) - Compile the script from disk
//! 2. [`run_script`](ScriptRuntime::run_script) - Execute top-level statements (remaps, blocks)
//! 3. [`call_hook`](ScriptRuntime::call_hook) - Call lifecycle hooks (e.g., `on_init`)
//! 4. [`lookup_remap`](ScriptRuntime::lookup_remap) - Query remapping rules during key processing
//!
//! For inline scripts, use [`execute`](ScriptRuntime::execute) which combines
//! compilation and execution in one step.
//!
//! # Error Handling
//!
//! All fallible methods return `Result<(), KeyrxError>`. Errors can occur:
//!
//! - **Script compilation**: Syntax errors, file not found
//! - **Script execution**: Runtime errors, invalid key names, division by zero
//! - **Hook calls**: Hook not defined, runtime errors within hook
//!
//! Invalid key names in `remap()`, `block()`, or `pass()` calls return errors that
//! can be caught in scripts using Rhai's `try/catch` mechanism.
//!
//! # Example Usage
//!
//! ```ignore
//! use keyrx_core::traits::ScriptRuntime;
//! use keyrx_core::scripting::RhaiRuntime;
//! use keyrx_core::engine::KeyCode;
//!
//! // Create runtime
//! let mut runtime = RhaiRuntime::new()?;
//!
//! // Load and execute a script file
//! runtime.load_file("config.rhai")?;
//! runtime.run_script()?;
//!
//! // Call initialization hook if defined
//! if runtime.has_hook("on_init") {
//!     runtime.call_hook("on_init")?;
//! }
//!
//! // Query remappings during key processing
//! match runtime.lookup_remap(KeyCode::CapsLock) {
//!     RemapAction::Remap(target) => println!("Remap to {:?}", target),
//!     RemapAction::Block => println!("Block key"),
//!     RemapAction::Pass => println!("Pass through"),
//! }
//! ```
//!
//! # Implementations
//!
//! - [`RhaiRuntime`](crate::scripting::RhaiRuntime): Production implementation using the Rhai engine
//! - [`MockRuntime`](crate::mocks::MockRuntime): Test mock for unit tests without real script execution
//!
//! # Thread Safety
//!
//! The trait does not require `Send` or `Sync` by default. The `RhaiRuntime`
//! implementation uses `Arc<Mutex>` internally for safe concurrent access to
//! pending operations, but the runtime itself should be used from a single thread.
//!
//! For multi-threaded usage, wrap the runtime in appropriate synchronization
//! primitives based on your application's needs.

use crate::engine::{KeyCode, RemapAction};
use crate::errors::KeyrxError;
use std::sync::{Arc, Mutex};

/// Trait for script execution runtime.
///
/// This trait abstracts the script engine, allowing the remapping engine to work
/// with different implementations (production Rhai, test mocks) using the same
/// interface.
///
/// # Method Call Order
///
/// For file-based scripts, methods must be called in this order:
///
/// ```text
/// load_file() -> run_script() -> call_hook() -> lookup_remap()
///     │              │               │               │
///     │              │               │               └── Repeated during key processing
///     │              │               └── Optional, only if hook exists
///     │              └── Executes top-level statements
///     └── Compiles script, scans for hooks
/// ```
///
/// For inline scripts (testing, REPL), use `execute()` directly:
///
/// ```text
/// execute() -> lookup_remap()
/// ```
///
/// # Error Handling
///
/// All methods that can fail return `Result<(), KeyrxError>`. Implementors should:
/// - Return descriptive error messages that help diagnose issues
/// - Include file paths and line numbers when available
/// - Not panic on recoverable errors
///
/// # Implementations
///
/// - [`RhaiRuntime`](crate::scripting::RhaiRuntime): Production Rhai engine
/// - [`MockRuntime`](crate::mocks::MockRuntime): Test mock for script simulation
pub trait ScriptRuntime {
    /// Execute a script string directly.
    ///
    /// Compiles and executes the script in one step. Useful for inline scripts,
    /// REPL interactions, or testing. Any `remap()`, `block()`, or `pass()` calls
    /// in the script are applied immediately.
    ///
    /// # Arguments
    ///
    /// * `script` - The Rhai script source code to execute
    ///
    /// # Returns
    ///
    /// - `Ok(())` - Script executed successfully
    /// - `Err(_)` - Compilation or runtime error
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The script has syntax errors
    /// - A runtime error occurs (invalid key name, division by zero, etc.)
    /// - Resource limits are exceeded (max operations, recursion depth)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut runtime = RhaiRuntime::new()?;
    /// runtime.execute(r#"remap("CapsLock", "Escape");"#)?;
    /// assert_eq!(runtime.lookup_remap(KeyCode::CapsLock), RemapAction::Remap(KeyCode::Escape));
    /// ```
    fn execute(&mut self, script: &str) -> Result<(), KeyrxError>;

    /// Call a named hook function defined in the loaded script.
    ///
    /// Hooks are user-defined functions in the script that are called at specific
    /// lifecycle points. The most common hook is `on_init`, called after the script
    /// loads to perform additional setup.
    ///
    /// # Arguments
    ///
    /// * `hook` - The name of the function to call (e.g., `"on_init"`)
    ///
    /// # Returns
    ///
    /// - `Ok(())` - Hook executed successfully
    /// - `Err(_)` - Hook not defined or runtime error
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No script has been loaded (call [`load_file`](Self::load_file) first)
    /// - The hook function is not defined in the script
    /// - A runtime error occurs within the hook
    ///
    /// # Preconditions
    ///
    /// - A script must be loaded via [`load_file`](Self::load_file) before calling hooks
    /// - Use [`has_hook`](Self::has_hook) to check if a hook exists before calling
    ///
    /// # Example
    ///
    /// ```ignore
    /// runtime.load_file("config.rhai")?;
    /// runtime.run_script()?;
    ///
    /// // Only call hook if it's defined
    /// if runtime.has_hook("on_init") {
    ///     runtime.call_hook("on_init")?;
    /// }
    /// ```
    fn call_hook(&mut self, hook: &str) -> Result<(), KeyrxError>;

    /// Load and compile a script file.
    ///
    /// Reads the script from disk, compiles it, and scans for defined functions
    /// (hooks). Does **not** execute the script; call [`run_script`](Self::run_script)
    /// after loading to execute top-level statements.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the `.rhai` script file
    ///
    /// # Returns
    ///
    /// - `Ok(())` - Script loaded and compiled successfully
    /// - `Err(_)` - File not found or compilation error
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file does not exist or cannot be read
    /// - The path is not valid UTF-8
    /// - The script has syntax errors
    ///
    /// # Post-conditions
    ///
    /// After successful loading:
    /// - [`has_hook`](Self::has_hook) will correctly report defined functions
    /// - [`run_script`](Self::run_script) can be called to execute top-level statements
    ///
    /// # Example
    ///
    /// ```ignore
    /// runtime.load_file("~/.config/keyrx/config.rhai")?;
    /// // Script is compiled but not yet executed
    /// assert!(runtime.has_hook("on_init")); // Can check for hooks
    /// ```
    fn load_file(&mut self, path: &str) -> Result<(), KeyrxError>;

    /// Execute the loaded script's top-level statements.
    ///
    /// Runs all statements outside of function definitions. This is where
    /// `remap()`, `block()`, and `pass()` calls typically appear in user scripts.
    /// Must be called after [`load_file`](Self::load_file).
    ///
    /// # Returns
    ///
    /// - `Ok(())` - Script executed successfully
    /// - `Err(_)` - No script loaded or runtime error
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No script has been loaded (call [`load_file`](Self::load_file) first)
    /// - A runtime error occurs during execution
    /// - An invalid key name is used in `remap()`, `block()`, or `pass()`
    ///
    /// # Preconditions
    ///
    /// - A script must be loaded via [`load_file`](Self::load_file)
    ///
    /// # Post-conditions
    ///
    /// After successful execution:
    /// - All remappings defined in top-level code are registered
    /// - [`lookup_remap`](Self::lookup_remap) will return the configured actions
    ///
    /// # Example
    ///
    /// ```ignore
    /// runtime.load_file("config.rhai")?;
    /// runtime.run_script()?;
    /// // Now remappings are registered and can be queried
    /// let action = runtime.lookup_remap(KeyCode::CapsLock);
    /// ```
    fn run_script(&mut self) -> Result<(), KeyrxError>;

    /// Check if a hook function is defined in the loaded script.
    ///
    /// Use this to conditionally call hooks that may or may not be defined
    /// by the user's script. This avoids errors when calling optional hooks.
    ///
    /// # Arguments
    ///
    /// * `hook` - The name of the function to check for
    ///
    /// # Returns
    ///
    /// - `true` if the function is defined in the loaded script
    /// - `false` if no script is loaded or the function is not defined
    ///
    /// # Example
    ///
    /// ```ignore
    /// runtime.load_file("config.rhai")?;
    ///
    /// // Only call hooks that exist
    /// if runtime.has_hook("on_init") {
    ///     runtime.call_hook("on_init")?;
    /// }
    ///
    /// // on_exit might not be defined
    /// if runtime.has_hook("on_exit") {
    ///     runtime.call_hook("on_exit")?;
    /// }
    /// ```
    fn has_hook(&self, hook: &str) -> bool;

    /// Look up the remapping action for a key.
    ///
    /// Returns the action to take when the given key is pressed. This is called
    /// by the remapping engine for every key event to determine how to handle it.
    ///
    /// # Arguments
    ///
    /// * `key` - The key code to look up
    ///
    /// # Returns
    ///
    /// The action to take for this key:
    /// - [`RemapAction::Remap(target)`](RemapAction::Remap) - Replace with `target` key
    /// - [`RemapAction::Block`](RemapAction::Block) - Suppress the key entirely
    /// - [`RemapAction::Pass`](RemapAction::Pass) - Let the key through unchanged
    ///
    /// # Default Behavior
    ///
    /// Keys that have not been configured via `remap()`, `block()`, or `pass()`
    /// should return [`RemapAction::Pass`](RemapAction::Pass). This ensures
    /// unconfigured keys work normally.
    ///
    /// # Performance
    ///
    /// This method is called for every key press and release, so implementations
    /// should ensure O(1) lookup time. The default `RhaiRuntime` uses a `HashMap`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// runtime.execute(r#"
    ///     remap("CapsLock", "Escape");
    ///     block("Insert");
    /// "#)?;
    ///
    /// // Remapped key
    /// assert_eq!(
    ///     runtime.lookup_remap(KeyCode::CapsLock),
    ///     RemapAction::Remap(KeyCode::Escape)
    /// );
    ///
    /// // Blocked key
    /// assert_eq!(runtime.lookup_remap(KeyCode::Insert), RemapAction::Block);
    ///
    /// // Unconfigured key - passes through
    /// assert_eq!(runtime.lookup_remap(KeyCode::A), RemapAction::Pass);
    /// ```
    fn lookup_remap(&self, key: KeyCode) -> RemapAction;
}

impl<T: ScriptRuntime> ScriptRuntime for Arc<Mutex<T>> {
    fn execute(&mut self, script: &str) -> Result<(), KeyrxError> {
        self.lock()
            .map_err(|_| KeyrxError::from(anyhow::anyhow!("ScriptRuntime lock poisoned")))?
            .execute(script)
    }

    fn call_hook(&mut self, hook: &str) -> Result<(), KeyrxError> {
        self.lock()
            .map_err(|_| KeyrxError::from(anyhow::anyhow!("ScriptRuntime lock poisoned")))?
            .call_hook(hook)
    }

    fn load_file(&mut self, path: &str) -> Result<(), KeyrxError> {
        self.lock()
            .map_err(|_| KeyrxError::from(anyhow::anyhow!("ScriptRuntime lock poisoned")))?
            .load_file(path)
    }

    fn run_script(&mut self) -> Result<(), KeyrxError> {
        self.lock()
            .map_err(|_| KeyrxError::from(anyhow::anyhow!("ScriptRuntime lock poisoned")))?
            .run_script()
    }

    fn has_hook(&self, hook: &str) -> bool {
        match self.lock() {
            Ok(guard) => guard.has_hook(hook),
            Err(_) => false,
        }
    }

    fn lookup_remap(&self, key: KeyCode) -> RemapAction {
        match self.lock() {
            Ok(guard) => guard.lookup_remap(key),
            Err(_) => RemapAction::Pass, // Fallback if lock fails
        }
    }
}
