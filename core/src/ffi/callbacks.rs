//! Injectable callback registry for FFI.
//!
//! Provides a testable registry for FFI callbacks instead of global statics.
#![allow(unsafe_code)]

use serde::Serialize;
use std::sync::{Arc, Mutex, OnceLock};

/// Function signature for discovery event callbacks (progress, duplicate, summary).
pub type DiscoveryEventCallback = unsafe extern "C" fn(*const u8, usize);

/// Function signature for engine state event callbacks.
pub type StateEventCallback = unsafe extern "C" fn(*const u8, usize);

/// Registry holding all FFI callbacks.
///
/// Enables dependency injection for testing while maintaining C ABI compatibility.
#[derive(Default)]
pub struct CallbackRegistry {
    progress: Mutex<Option<DiscoveryEventCallback>>,
    duplicate: Mutex<Option<DiscoveryEventCallback>>,
    summary: Mutex<Option<DiscoveryEventCallback>>,
    state: Mutex<Option<StateEventCallback>>,
}

impl CallbackRegistry {
    /// Create a new empty callback registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the discovery progress callback.
    pub fn set_progress(&self, callback: Option<DiscoveryEventCallback>) {
        if let Ok(mut guard) = self.progress.lock() {
            *guard = callback;
        }
    }

    /// Set the discovery duplicate callback.
    pub fn set_duplicate(&self, callback: Option<DiscoveryEventCallback>) {
        if let Ok(mut guard) = self.duplicate.lock() {
            *guard = callback;
        }
    }

    /// Set the discovery summary callback.
    pub fn set_summary(&self, callback: Option<DiscoveryEventCallback>) {
        if let Ok(mut guard) = self.summary.lock() {
            *guard = callback;
        }
    }

    /// Set the state event callback.
    pub fn set_state(&self, callback: Option<StateEventCallback>) {
        if let Ok(mut guard) = self.state.lock() {
            *guard = callback;
        }
    }

    /// Get the current progress callback.
    pub fn progress(&self) -> Option<DiscoveryEventCallback> {
        self.progress.lock().ok().and_then(|guard| *guard)
    }

    /// Get the current duplicate callback.
    pub fn duplicate(&self) -> Option<DiscoveryEventCallback> {
        self.duplicate.lock().ok().and_then(|guard| *guard)
    }

    /// Get the current summary callback.
    pub fn summary(&self) -> Option<DiscoveryEventCallback> {
        self.summary.lock().ok().and_then(|guard| *guard)
    }

    /// Get the current state callback.
    pub fn state(&self) -> Option<StateEventCallback> {
        self.state.lock().ok().and_then(|guard| *guard)
    }

    /// Check if any discovery callback is registered.
    pub fn has_any_discovery_callback(&self) -> bool {
        self.progress().is_some() || self.duplicate().is_some() || self.summary().is_some()
    }

    /// Invoke a discovery callback with serialized JSON payload.
    pub fn invoke_discovery<T: Serialize>(
        &self,
        callback: Option<DiscoveryEventCallback>,
        payload: &T,
        event: &'static str,
    ) {
        let Some(cb) = callback else { return };

        match serde_json::to_vec(payload) {
            Ok(bytes) => unsafe {
                cb(bytes.as_ptr(), bytes.len());
            },
            Err(err) => tracing::warn!(
                service = "keyrx",
                component = "ffi_callbacks",
                event,
                error = %err,
                "Failed to serialize payload for FFI"
            ),
        }
    }

    /// Invoke the state callback with serialized JSON payload.
    pub fn invoke_state<T: Serialize>(&self, payload: &T) {
        let Some(cb) = self.state() else { return };

        match serde_json::to_vec(payload) {
            Ok(bytes) => unsafe {
                cb(bytes.as_ptr(), bytes.len());
            },
            Err(err) => tracing::warn!(
                service = "keyrx",
                component = "ffi_callbacks",
                event = "state",
                error = %err,
                "Failed to serialize state payload for FFI"
            ),
        }
    }
}

/// Global callback registry for FFI.
///
/// This provides the default registry used by FFI exports. Tests can create
/// their own `CallbackRegistry` instances for isolation.
fn global_registry() -> &'static CallbackRegistry {
    static REGISTRY: OnceLock<CallbackRegistry> = OnceLock::new();
    REGISTRY.get_or_init(CallbackRegistry::new)
}

/// Get a reference to the global callback registry.
pub fn callback_registry() -> &'static CallbackRegistry {
    global_registry()
}

/// Thread-safe wrapper for tests to use isolated registries.
pub struct IsolatedRegistry {
    inner: Arc<CallbackRegistry>,
}

impl IsolatedRegistry {
    /// Create a new isolated registry for testing.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(CallbackRegistry::new()),
        }
    }

    /// Get a reference to the inner registry.
    pub fn registry(&self) -> &CallbackRegistry {
        &self.inner
    }
}

impl Default for IsolatedRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

    unsafe extern "C" fn test_callback(_ptr: *const u8, _len: usize) {
        CALL_COUNT.fetch_add(1, Ordering::SeqCst);
    }

    #[test]
    fn registry_starts_empty() {
        let registry = CallbackRegistry::new();
        assert!(registry.progress().is_none());
        assert!(registry.duplicate().is_none());
        assert!(registry.summary().is_none());
        assert!(registry.state().is_none());
        assert!(!registry.has_any_discovery_callback());
    }

    #[test]
    fn can_set_and_get_callbacks() {
        let registry = CallbackRegistry::new();

        registry.set_progress(Some(test_callback));
        assert!(registry.progress().is_some());
        assert!(registry.has_any_discovery_callback());

        registry.set_progress(None);
        assert!(registry.progress().is_none());
    }

    #[test]
    fn isolated_registry_provides_isolation() {
        let isolated = IsolatedRegistry::new();
        let registry = isolated.registry();

        registry.set_state(Some(test_callback));
        assert!(registry.state().is_some());

        // Global registry should be unaffected
        // Note: Can't fully test this since global_registry is static,
        // but IsolatedRegistry pattern enables test isolation
    }
}
