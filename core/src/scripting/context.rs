//! Runtime context for injectable scripting state.
//!
//! This module provides `RuntimeContext` as an injectable container for
//! the Rhai runtime, replacing the global `RUNTIME_SLOT` pattern to enable
//! parallel test execution.

use super::RhaiRuntime;
use std::sync::{Arc, Mutex};

/// Injectable runtime context for thread-safe Rhai runtime access.
///
/// Replaces the global `RUNTIME_SLOT` pattern to allow tests to run in
/// parallel without `#[serial]` attributes.
///
/// # Thread Safety
///
/// Uses `Arc<Mutex<Option<*mut RhaiRuntime>>>` internally. The pointer is
/// valid only while the owning `RhaiRuntime` is alive.
///
/// # Example
///
/// ```ignore
/// let mut runtime = RhaiRuntime::new()?;
/// let ctx = RuntimeContext::new();
/// ctx.set(&mut runtime);
///
/// // Use context for FFI operations
/// ctx.with_runtime(|rt| rt.execute("remap(\"A\", \"B\");"))?;
///
/// ctx.clear();
/// ```
#[derive(Clone)]
pub struct RuntimeContext {
    inner: Arc<Mutex<RuntimeSlotInner>>,
}

struct RuntimeSlotInner {
    runtime: Option<*mut RhaiRuntime>,
    owner: Option<std::thread::ThreadId>,
}

// Safety: Access is serialized via Mutex.
#[allow(unsafe_code)]
unsafe impl Send for RuntimeSlotInner {}
#[allow(unsafe_code)]
unsafe impl Sync for RuntimeSlotInner {}

impl Default for RuntimeContext {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeContext {
    /// Create a new empty runtime context.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(RuntimeSlotInner {
                runtime: None,
                owner: None,
            })),
        }
    }

    /// Register a runtime in this context.
    ///
    /// The runtime pointer is valid only while the `RhaiRuntime` is alive.
    /// Call `clear()` before dropping the runtime.
    #[allow(unsafe_code)]
    pub fn set(&self, runtime: &mut RhaiRuntime) {
        if let Ok(mut guard) = self.inner.lock() {
            guard.runtime = Some(runtime as *mut RhaiRuntime);
            guard.owner = Some(std::thread::current().id());
        }
    }

    /// Clear the runtime reference.
    ///
    /// Must be called before the runtime is dropped.
    pub fn clear(&self) {
        if let Ok(mut guard) = self.inner.lock() {
            guard.runtime = None;
            guard.owner = None;
        }
    }

    /// Execute a closure with the registered runtime.
    ///
    /// Returns `None` if:
    /// - No runtime is registered
    /// - Called from a different thread than the one that registered the runtime
    /// - The mutex cannot be acquired
    #[allow(unsafe_code)]
    pub fn with_runtime<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&mut RhaiRuntime) -> R,
    {
        let guard = self.inner.lock().ok()?;
        let current_thread = std::thread::current().id();

        // Verify same thread owns this context
        guard.owner.filter(|id| *id == current_thread)?;

        // Safety: Verified thread ownership and mutex protection
        let runtime_ptr = guard.runtime?;
        let runtime_ref = unsafe { runtime_ptr.as_mut()? };
        Some(f(runtime_ref))
    }

    /// Check if a runtime is currently registered.
    pub fn has_runtime(&self) -> bool {
        self.inner
            .lock()
            .ok()
            .map(|guard| guard.runtime.is_some())
            .unwrap_or(false)
    }
}

/// Global runtime context for FFI compatibility.
///
/// This provides backward compatibility with the existing FFI API while
/// allowing injection of custom contexts in tests.
static GLOBAL_CONTEXT: std::sync::OnceLock<RuntimeContext> = std::sync::OnceLock::new();

/// Get the global runtime context.
///
/// For FFI functions that need a shared runtime.
pub fn global_context() -> &'static RuntimeContext {
    GLOBAL_CONTEXT.get_or_init(RuntimeContext::new)
}

/// Register a runtime in the global context (FFI compatibility).
pub fn set_active_runtime(runtime: &mut RhaiRuntime) {
    global_context().set(runtime);
}

/// Clear the global runtime context (FFI compatibility).
pub fn clear_active_runtime() {
    global_context().clear();
}

/// Execute a closure with the global runtime (FFI compatibility).
pub fn with_active_runtime<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut RhaiRuntime) -> R,
{
    global_context().with_runtime(f)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scripting::RhaiRuntime;
    use crate::traits::ScriptRuntime;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::thread;

    #[test]
    fn context_starts_empty() {
        let ctx = RuntimeContext::new();
        assert!(!ctx.has_runtime());
        assert!(ctx.with_runtime(|_| ()).is_none());
    }

    #[test]
    fn context_set_and_clear() {
        let ctx = RuntimeContext::new();
        let mut runtime = RhaiRuntime::new().unwrap();

        ctx.set(&mut runtime);
        assert!(ctx.has_runtime());

        ctx.clear();
        assert!(!ctx.has_runtime());
    }

    #[test]
    fn with_runtime_executes_closure() {
        let ctx = RuntimeContext::new();
        let mut runtime = RhaiRuntime::new().unwrap();

        ctx.set(&mut runtime);

        let result = ctx.with_runtime(|rt| rt.execute(r#"remap("A", "B");"#));
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        ctx.clear();
    }

    #[test]
    fn context_rejects_cross_thread_access() {
        let ctx = RuntimeContext::new();
        let mut runtime = RhaiRuntime::new().unwrap();

        ctx.set(&mut runtime);

        let ctx_clone = ctx.clone();
        let accessed = Arc::new(AtomicBool::new(false));
        let accessed_clone = accessed.clone();

        let handle = thread::spawn(move || {
            // This should return None - different thread
            let result = ctx_clone.with_runtime(|_| ());
            accessed_clone.store(result.is_some(), Ordering::SeqCst);
        });

        handle.join().unwrap();
        assert!(!accessed.load(Ordering::SeqCst));

        ctx.clear();
    }

    #[test]
    fn multiple_contexts_are_independent() {
        let ctx1 = RuntimeContext::new();
        let ctx2 = RuntimeContext::new();

        let mut runtime1 = RhaiRuntime::new().unwrap();
        let mut runtime2 = RhaiRuntime::new().unwrap();

        ctx1.set(&mut runtime1);
        ctx2.set(&mut runtime2);

        // Execute different scripts in each context
        ctx1.with_runtime(|rt| rt.execute(r#"remap("A", "B");"#))
            .unwrap()
            .unwrap();
        ctx2.with_runtime(|rt| rt.execute(r#"remap("C", "D");"#))
            .unwrap()
            .unwrap();

        // Verify independence
        use crate::engine::{KeyCode, RemapAction};

        ctx1.with_runtime(|rt| {
            assert_eq!(rt.lookup_remap(KeyCode::A), RemapAction::Remap(KeyCode::B));
            assert_eq!(rt.lookup_remap(KeyCode::C), RemapAction::Pass);
        });

        ctx2.with_runtime(|rt| {
            assert_eq!(rt.lookup_remap(KeyCode::A), RemapAction::Pass);
            assert_eq!(rt.lookup_remap(KeyCode::C), RemapAction::Remap(KeyCode::D));
        });

        ctx1.clear();
        ctx2.clear();
    }

    #[test]
    fn cloned_context_shares_runtime() {
        let ctx1 = RuntimeContext::new();
        let ctx2 = ctx1.clone();

        let mut runtime = RhaiRuntime::new().unwrap();
        ctx1.set(&mut runtime);

        // Both contexts should see the runtime
        assert!(ctx1.has_runtime());
        assert!(ctx2.has_runtime());

        ctx1.clear();

        // Both should now be empty
        assert!(!ctx1.has_runtime());
        assert!(!ctx2.has_runtime());
    }
}
