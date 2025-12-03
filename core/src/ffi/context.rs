//! FFI context management with handle-based state.
//!
//! Provides instance-scoped state management to replace global statics,
//! enabling parallel test execution and proper resource lifecycle management.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Opaque handle for FFI context instances.
///
/// Used to identify and access FFI context state across the FFI boundary.
/// Handles are unique per context and can be safely passed to C/Dart.
pub type FfiHandle = u64;

/// FFI context holding all instance-scoped state.
///
/// Replaces global `OnceLock<Mutex<...>>` patterns with handle-based,
/// instance-scoped state management. Each context has:
/// - A unique handle for identification
/// - Domain-specific state storage (keyed by type)
/// - Thread-safe access to state
///
/// # Example
/// ```
/// # use keyrx_core::ffi::context::FfiContext;
/// let mut ctx = FfiContext::new();
/// let handle = ctx.handle();
///
/// // Store domain-specific state
/// struct DiscoveryState { active: bool }
/// ctx.set_domain("discovery", DiscoveryState { active: true });
///
/// // Retrieve domain state
/// let state = ctx.get_domain::<DiscoveryState>("discovery");
/// assert!(state.is_some());
/// ```
pub struct FfiContext {
    /// Unique handle for this context
    handle: FfiHandle,

    /// Domain-specific state storage.
    /// Keys are domain names (e.g., "discovery", "validation")
    /// Values are boxed domain state structs wrapped in Arc<RwLock<>> for thread safety
    domains: HashMap<&'static str, Arc<RwLock<Box<dyn Any + Send + Sync>>>>,
}

impl FfiContext {
    /// Create a new FFI context with a unique handle.
    ///
    /// # Example
    /// ```
    /// # use keyrx_core::ffi::context::FfiContext;
    /// let ctx = FfiContext::new();
    /// assert!(ctx.handle() > 0);
    /// ```
    pub fn new() -> Self {
        Self {
            handle: Self::generate_handle(),
            domains: HashMap::new(),
        }
    }

    /// Get the unique handle for this context.
    pub fn handle(&self) -> FfiHandle {
        self.handle
    }

    /// Store domain-specific state.
    ///
    /// # Example
    /// ```
    /// # use keyrx_core::ffi::context::FfiContext;
    /// struct MyDomainState { value: i32 }
    /// let mut ctx = FfiContext::new();
    /// ctx.set_domain("my_domain", MyDomainState { value: 42 });
    /// ```
    pub fn set_domain<T: Any + Send + Sync>(&mut self, domain: &'static str, state: T) {
        self.domains
            .insert(domain, Arc::new(RwLock::new(Box::new(state))));
    }

    /// Get immutable reference to domain state.
    ///
    /// Returns `None` if the domain doesn't exist or the type doesn't match.
    ///
    /// # Example
    /// ```
    /// # use keyrx_core::ffi::context::FfiContext;
    /// # struct MyState { value: i32 }
    /// # let mut ctx = FfiContext::new();
    /// # ctx.set_domain("test", MyState { value: 42 });
    /// let state = ctx.get_domain::<MyState>("test");
    /// assert!(state.is_some());
    /// ```
    pub fn get_domain<T: Any + Send + Sync>(
        &self,
        domain: &'static str,
    ) -> Option<std::sync::RwLockReadGuard<'_, Box<dyn Any + Send + Sync>>> {
        self.domains.get(domain).and_then(|arc| {
            let guard = arc.read().ok()?;
            // Check if the type matches before returning
            if (**guard).type_id() == TypeId::of::<T>() {
                Some(guard)
            } else {
                None
            }
        })
    }

    /// Get mutable reference to domain state.
    ///
    /// Returns `None` if the domain doesn't exist or the type doesn't match.
    ///
    /// # Example
    /// ```
    /// # use keyrx_core::ffi::context::FfiContext;
    /// # struct MyState { value: i32 }
    /// # let mut ctx = FfiContext::new();
    /// # ctx.set_domain("test", MyState { value: 42 });
    /// let state = ctx.get_domain_mut::<MyState>("test");
    /// assert!(state.is_some());
    /// ```
    pub fn get_domain_mut<T: Any + Send + Sync>(
        &self,
        domain: &'static str,
    ) -> Option<std::sync::RwLockWriteGuard<'_, Box<dyn Any + Send + Sync>>> {
        self.domains.get(domain).and_then(|arc| {
            let guard = arc.write().ok()?;
            // Check if the type matches before returning
            if (**guard).type_id() == TypeId::of::<T>() {
                Some(guard)
            } else {
                None
            }
        })
    }

    /// Remove domain state.
    ///
    /// Returns `true` if the domain existed and was removed.
    pub fn remove_domain(&mut self, domain: &'static str) -> bool {
        self.domains.remove(domain).is_some()
    }

    /// Check if a domain exists in this context.
    pub fn has_domain(&self, domain: &'static str) -> bool {
        self.domains.contains_key(domain)
    }

    /// Clean up all domain state.
    ///
    /// Should be called when disposing of an FFI context.
    pub fn cleanup(&mut self) {
        self.domains.clear();
    }

    /// Generate a unique handle for a new context.
    ///
    /// Uses a simple atomic counter. In production, this could be enhanced
    /// with more sophisticated handle generation (e.g., random IDs).
    fn generate_handle() -> FfiHandle {
        use std::sync::atomic::{AtomicU64, Ordering};
        static HANDLE_COUNTER: AtomicU64 = AtomicU64::new(1);
        HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst)
    }
}

impl Default for FfiContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Global registry for FFI context handles.
///
/// Manages the mapping from opaque handles to context instances,
/// providing safe access across the FFI boundary.
pub struct FfiContextRegistry {
    contexts: RwLock<HashMap<FfiHandle, Arc<RwLock<FfiContext>>>>,
}

impl FfiContextRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            contexts: RwLock::new(HashMap::new()),
        }
    }

    /// Register a new context and return its handle.
    pub fn register(&self, ctx: FfiContext) -> FfiHandle {
        let handle = ctx.handle();
        if let Ok(mut contexts) = self.contexts.write() {
            contexts.insert(handle, Arc::new(RwLock::new(ctx)));
        }
        handle
    }

    /// Get a context by handle.
    pub fn get(&self, handle: FfiHandle) -> Option<Arc<RwLock<FfiContext>>> {
        self.contexts
            .read()
            .ok()
            .and_then(|contexts| contexts.get(&handle).cloned())
    }

    /// Unregister and drop a context.
    pub fn unregister(&self, handle: FfiHandle) -> bool {
        self.contexts
            .write()
            .ok()
            .and_then(|mut contexts| contexts.remove(&handle))
            .is_some()
    }

    /// Get the number of registered contexts.
    pub fn count(&self) -> usize {
        self.contexts
            .read()
            .ok()
            .map_or(0, |contexts| contexts.len())
    }
}

impl Default for FfiContextRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global registry instance.
///
/// This is the only global state needed - it manages handles to contexts.
/// Each context itself contains no global state.
static CONTEXT_REGISTRY: std::sync::OnceLock<FfiContextRegistry> = std::sync::OnceLock::new();

/// Get the global context registry.
pub fn context_registry() -> &'static FfiContextRegistry {
    CONTEXT_REGISTRY.get_or_init(FfiContextRegistry::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct TestState {
        value: i32,
        name: String,
    }

    #[test]
    fn test_context_creation() {
        let ctx = FfiContext::new();
        assert!(ctx.handle() > 0);
    }

    #[test]
    fn test_unique_handles() {
        let ctx1 = FfiContext::new();
        let ctx2 = FfiContext::new();
        assert_ne!(ctx1.handle(), ctx2.handle());
    }

    #[test]
    fn test_domain_storage_and_retrieval() {
        let mut ctx = FfiContext::new();
        let state = TestState {
            value: 42,
            name: "test".to_string(),
        };

        ctx.set_domain("test_domain", state);
        assert!(ctx.has_domain("test_domain"));

        let retrieved = ctx.get_domain::<TestState>("test_domain");
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_domain_type_mismatch() {
        let mut ctx = FfiContext::new();
        ctx.set_domain(
            "test",
            TestState {
                value: 42,
                name: "test".to_string(),
            },
        );

        // Try to retrieve with wrong type
        let wrong_type = ctx.get_domain::<i32>("test");
        assert!(wrong_type.is_none());
    }

    #[test]
    fn test_domain_removal() {
        let mut ctx = FfiContext::new();
        ctx.set_domain(
            "test",
            TestState {
                value: 42,
                name: "test".to_string(),
            },
        );

        assert!(ctx.has_domain("test"));
        assert!(ctx.remove_domain("test"));
        assert!(!ctx.has_domain("test"));
        assert!(!ctx.remove_domain("test")); // Already removed
    }

    #[test]
    fn test_context_cleanup() {
        let mut ctx = FfiContext::new();
        ctx.set_domain(
            "test1",
            TestState {
                value: 1,
                name: "one".to_string(),
            },
        );
        ctx.set_domain(
            "test2",
            TestState {
                value: 2,
                name: "two".to_string(),
            },
        );

        assert!(ctx.has_domain("test1"));
        assert!(ctx.has_domain("test2"));

        ctx.cleanup();

        assert!(!ctx.has_domain("test1"));
        assert!(!ctx.has_domain("test2"));
    }

    #[test]
    fn test_registry_operations() {
        let registry = FfiContextRegistry::new();
        let ctx = FfiContext::new();
        let handle = ctx.handle();

        // Register context
        let registered_handle = registry.register(ctx);
        assert_eq!(registered_handle, handle);
        assert_eq!(registry.count(), 1);

        // Get context
        let retrieved = registry.get(handle);
        assert!(retrieved.is_some());

        // Unregister context
        assert!(registry.unregister(handle));
        assert_eq!(registry.count(), 0);
        assert!(!registry.unregister(handle)); // Already unregistered
    }

    #[test]
    fn test_multiple_contexts_in_registry() {
        let registry = FfiContextRegistry::new();

        let ctx1 = FfiContext::new();
        let ctx2 = FfiContext::new();
        let handle1 = ctx1.handle();
        let handle2 = ctx2.handle();

        registry.register(ctx1);
        registry.register(ctx2);

        assert_eq!(registry.count(), 2);
        assert!(registry.get(handle1).is_some());
        assert!(registry.get(handle2).is_some());
    }

    #[test]
    fn test_context_isolation() {
        let mut ctx1 = FfiContext::new();
        let mut ctx2 = FfiContext::new();

        ctx1.set_domain(
            "shared",
            TestState {
                value: 1,
                name: "ctx1".to_string(),
            },
        );
        ctx2.set_domain(
            "shared",
            TestState {
                value: 2,
                name: "ctx2".to_string(),
            },
        );

        // Each context has its own state
        assert!(ctx1.has_domain("shared"));
        assert!(ctx2.has_domain("shared"));

        // Removing from one doesn't affect the other
        ctx1.remove_domain("shared");
        assert!(!ctx1.has_domain("shared"));
        assert!(ctx2.has_domain("shared"));
    }

    #[test]
    fn test_mutable_domain_access() {
        let mut ctx = FfiContext::new();
        ctx.set_domain(
            "test",
            TestState {
                value: 42,
                name: "initial".to_string(),
            },
        );

        // Get mutable access and modify
        {
            let mut guard = ctx.get_domain_mut::<TestState>("test").unwrap();
            let state = guard.downcast_mut::<TestState>().unwrap();
            state.value = 100;
            state.name = "modified".to_string();
        }

        // Verify modification
        let guard = ctx.get_domain::<TestState>("test").unwrap();
        let state = guard.downcast_ref::<TestState>().unwrap();
        assert_eq!(state.value, 100);
        assert_eq!(state.name, "modified");
    }
}
