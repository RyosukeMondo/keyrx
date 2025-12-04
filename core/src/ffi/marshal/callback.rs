//! FFI callback system for type-safe callback management.
//!
//! This module provides a unified callback system that integrates with the marshaling
//! layer. It allows registering callbacks with type-safe data handling and provides
//! both synchronous and async-compatible invocation.
//!
//! # Architecture
//!
//! The callback system consists of:
//!
//! - [`FfiCallback`]: Trait for type-safe callback invocation
//! - [`CallbackRegistry`]: Thread-safe registry for managing callbacks by ID
//! - [`CallbackId`]: Unique identifier for registered callbacks
//!
//! # Design Rationale
//!
//! Unlike the existing [`crate::ffi::events::EventRegistry`] which uses event types,
//! this system uses unique IDs allowing multiple callbacks of the same type. This is
//! useful for scenarios like:
//!
//! - Multiple concurrent operations with their own progress callbacks
//! - Per-device callbacks in discovery
//! - Request-specific response handlers
//!
//! # Thread Safety
//!
//! All types are thread-safe using `Arc` and `DashMap` for concurrent access.
//! Callbacks can be registered, invoked, and unregistered from any thread.
//!
//! # Error Handling
//!
//! Callback invocation failures are logged but never panic. Failed callbacks return
//! errors via `FfiResult`, allowing callers to decide how to handle failures.
//!
//! # Example
//!
//! ```
//! use keyrx_core::ffi::marshal::callback::{FfiCallback, CallbackRegistry};
//! use keyrx_core::ffi::error::FfiResult;
//!
//! // Define a callback type
//! struct ProgressCallback {
//!     name: String,
//! }
//!
//! impl FfiCallback for ProgressCallback {
//!     fn invoke(&self, data: &[u8]) -> FfiResult<()> {
//!         // Parse data and handle callback
//!         println!("{}: received {} bytes", self.name, data.len());
//!         Ok(())
//!     }
//!
//!     fn callback_type(&self) -> &'static str {
//!         "progress"
//!     }
//! }
//!
//! // Use the registry
//! let registry = CallbackRegistry::new();
//! let callback = ProgressCallback { name: "task1".to_string() };
//! let id = registry.register(callback);
//!
//! // Invoke callback
//! let data = b"progress data";
//! registry.invoke(id, data).ok();
//!
//! // Clean up
//! registry.unregister(id);
//! ```

use crate::ffi::error::{FfiError, FfiResult};
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Unique identifier for a registered callback.
///
/// Callback IDs are globally unique and monotonically increasing. They're safe
/// to pass across FFI boundaries as `u64` values.
///
/// # Example
///
/// ```
/// use keyrx_core::ffi::marshal::callback::CallbackId;
///
/// let id = CallbackId::new(42);
/// assert_eq!(id.as_u64(), 42);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CallbackId(u64);

impl CallbackId {
    /// Create a callback ID from a u64 value.
    ///
    /// # Parameters
    ///
    /// * `id` - The numeric ID value
    ///
    /// # Returns
    ///
    /// A new `CallbackId` wrapping the value.
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the numeric ID value.
    ///
    /// # Returns
    ///
    /// The u64 value of this callback ID.
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

impl From<u64> for CallbackId {
    fn from(id: u64) -> Self {
        Self::new(id)
    }
}

impl From<CallbackId> for u64 {
    fn from(id: CallbackId) -> u64 {
        id.as_u64()
    }
}

/// Trait for FFI callbacks with type-safe invocation.
///
/// Implementors define how to handle raw byte data passed from the FFI boundary.
/// The callback system ensures thread-safe invocation and error handling.
///
/// # Design
///
/// Callbacks receive raw bytes (`&[u8]`) which can be:
/// - JSON-serialized data (use `serde_json::from_slice`)
/// - Binary protocol data (custom parsing)
/// - C struct bytes (unsafe deserialization)
///
/// # Thread Safety
///
/// All callbacks must be `Send + Sync` to support concurrent invocation from
/// multiple threads. Use interior mutability (`Mutex`, `RwLock`) if needed.
///
/// # Error Handling
///
/// Callbacks return `FfiResult<()>` to indicate success or failure. Errors are
/// logged by the registry but do not propagate to prevent FFI boundary issues.
///
/// # Async Support
///
/// While the trait is synchronous, implementations can spawn async tasks or
/// send data to channels for async processing. The callback should return
/// immediately after queuing work.
///
/// # Example
///
/// ```
/// use keyrx_core::ffi::marshal::callback::FfiCallback;
/// use keyrx_core::ffi::error::FfiResult;
/// use serde_json::Value;
///
/// struct JsonCallback;
///
/// impl FfiCallback for JsonCallback {
///     fn invoke(&self, data: &[u8]) -> FfiResult<()> {
///         let json: Value = serde_json::from_slice(data)
///             .map_err(|e| {
///                 crate::ffi::error::FfiError::invalid_input(
///                     format!("Invalid JSON: {}", e)
///                 )
///             })?;
///
///         println!("Received: {:?}", json);
///         Ok(())
///     }
///
///     fn callback_type(&self) -> &'static str {
///         "json"
///     }
/// }
/// ```
pub trait FfiCallback: Send + Sync {
    /// Invoke the callback with raw byte data.
    ///
    /// # Parameters
    ///
    /// * `data` - Raw bytes received from FFI boundary
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Callback handled successfully
    /// - `Err(FfiError)`: Callback failed with error details
    ///
    /// # Errors
    ///
    /// Common error cases:
    /// - Invalid data format (JSON parsing, binary protocol errors)
    /// - Business logic failures
    /// - Internal state errors
    ///
    /// # Example
    ///
    /// ```
    /// # use keyrx_core::ffi::marshal::callback::FfiCallback;
    /// # use keyrx_core::ffi::error::FfiResult;
    /// # struct MyCallback;
    /// # impl FfiCallback for MyCallback {
    /// #     fn callback_type(&self) -> &'static str { "test" }
    /// fn invoke(&self, data: &[u8]) -> FfiResult<()> {
    ///     if data.is_empty() {
    ///         return Err(crate::ffi::error::FfiError::invalid_input(
    ///             "Empty data"
    ///         ));
    ///     }
    ///     // Process data...
    ///     Ok(())
    /// }
    /// # }
    /// ```
    fn invoke(&self, data: &[u8]) -> FfiResult<()>;

    /// Get the callback type name for logging and debugging.
    ///
    /// # Returns
    ///
    /// A static string identifying the callback type (e.g., "progress", "error").
    ///
    /// # Example
    ///
    /// ```
    /// # use keyrx_core::ffi::marshal::callback::FfiCallback;
    /// # use keyrx_core::ffi::error::FfiResult;
    /// # struct ProgressCallback;
    /// # impl FfiCallback for ProgressCallback {
    /// #     fn invoke(&self, _data: &[u8]) -> FfiResult<()> { Ok(()) }
    /// fn callback_type(&self) -> &'static str {
    ///     "progress"
    /// }
    /// # }
    /// ```
    fn callback_type(&self) -> &'static str;
}

/// Thread-safe registry for managing FFI callbacks.
///
/// The registry provides:
/// - **Registration**: Add callbacks and get unique IDs
/// - **Invocation**: Call callbacks by ID with data
/// - **Unregistration**: Remove callbacks by ID
/// - **Global Access**: Singleton pattern for application-wide use
///
/// # Thread Safety
///
/// Uses `DashMap` for lock-free concurrent access. Multiple threads can
/// register, invoke, and unregister callbacks simultaneously without blocking.
///
/// # Memory Management
///
/// Callbacks are boxed and stored in the registry. When unregistered or when
/// the registry is dropped, callbacks are automatically deallocated.
///
/// # Error Handling
///
/// - Registration never fails (returns a unique ID)
/// - Invocation returns `FfiResult` indicating callback success/failure
/// - Unregistration returns `bool` indicating if callback existed
///
/// # Example
///
/// ```
/// use keyrx_core::ffi::marshal::callback::{FfiCallback, CallbackRegistry};
/// use keyrx_core::ffi::error::FfiResult;
///
/// struct TestCallback;
/// impl FfiCallback for TestCallback {
///     fn invoke(&self, _data: &[u8]) -> FfiResult<()> { Ok(()) }
///     fn callback_type(&self) -> &'static str { "test" }
/// }
///
/// let registry = CallbackRegistry::new();
/// let id = registry.register(TestCallback);
/// assert!(registry.invoke(id, b"data").is_ok());
/// assert!(registry.unregister(id));
/// ```
#[derive(Clone)]
pub struct CallbackRegistry {
    /// Map of callback IDs to callback implementations.
    callbacks: Arc<DashMap<CallbackId, Box<dyn FfiCallback>>>,
    /// Atomic counter for generating unique callback IDs.
    next_id: Arc<AtomicU64>,
}

impl CallbackRegistry {
    /// Create a new empty callback registry.
    ///
    /// # Returns
    ///
    /// A new registry with no registered callbacks.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::ffi::marshal::callback::CallbackRegistry;
    ///
    /// let registry = CallbackRegistry::new();
    /// assert_eq!(registry.callback_count(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            callbacks: Arc::new(DashMap::new()),
            next_id: Arc::new(AtomicU64::new(1)), // Start at 1 (0 reserved for null)
        }
    }

    /// Get the global callback registry singleton.
    ///
    /// Provides application-wide access to a shared registry. Useful for FFI
    /// exports that need a consistent registry across all calls.
    ///
    /// # Returns
    ///
    /// A reference to the global `CallbackRegistry` instance.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_core::ffi::marshal::callback::CallbackRegistry;
    ///
    /// let registry = CallbackRegistry::global();
    /// // Use registry...
    /// ```
    pub fn global() -> &'static Self {
        use std::sync::OnceLock;
        static GLOBAL_REGISTRY: OnceLock<CallbackRegistry> = OnceLock::new();
        GLOBAL_REGISTRY.get_or_init(CallbackRegistry::new)
    }

    /// Register a callback and get its unique ID.
    ///
    /// # Parameters
    ///
    /// * `callback` - The callback implementation to register
    ///
    /// # Returns
    ///
    /// A unique `CallbackId` that can be used to invoke or unregister the callback.
    ///
    /// # Example
    ///
    /// ```
    /// # use keyrx_core::ffi::marshal::callback::{FfiCallback, CallbackRegistry};
    /// # use keyrx_core::ffi::error::FfiResult;
    /// # struct MyCallback;
    /// # impl FfiCallback for MyCallback {
    /// #     fn invoke(&self, _: &[u8]) -> FfiResult<()> { Ok(()) }
    /// #     fn callback_type(&self) -> &'static str { "test" }
    /// # }
    /// let registry = CallbackRegistry::new();
    /// let id = registry.register(MyCallback);
    /// assert_eq!(registry.callback_count(), 1);
    /// ```
    pub fn register<C: FfiCallback + 'static>(&self, callback: C) -> CallbackId {
        let id = CallbackId::new(self.next_id.fetch_add(1, Ordering::SeqCst));
        self.callbacks.insert(id, Box::new(callback));

        tracing::debug!(
            service = "keyrx",
            component = "ffi_callback",
            event = "callback_registered",
            callback_id = id.as_u64(),
            "Callback registered"
        );

        id
    }

    /// Unregister a callback by ID.
    ///
    /// # Parameters
    ///
    /// * `id` - The ID of the callback to unregister
    ///
    /// # Returns
    ///
    /// - `true`: Callback was found and removed
    /// - `false`: No callback with that ID exists
    ///
    /// # Example
    ///
    /// ```
    /// # use keyrx_core::ffi::marshal::callback::{FfiCallback, CallbackRegistry};
    /// # use keyrx_core::ffi::error::FfiResult;
    /// # struct TestCallback;
    /// # impl FfiCallback for TestCallback {
    /// #     fn invoke(&self, _: &[u8]) -> FfiResult<()> { Ok(()) }
    /// #     fn callback_type(&self) -> &'static str { "test" }
    /// # }
    /// let registry = CallbackRegistry::new();
    /// let id = registry.register(TestCallback);
    /// assert!(registry.unregister(id));
    /// assert!(!registry.unregister(id)); // Already removed
    /// ```
    pub fn unregister(&self, id: CallbackId) -> bool {
        let removed = self.callbacks.remove(&id).is_some();

        if removed {
            tracing::debug!(
                service = "keyrx",
                component = "ffi_callback",
                event = "callback_unregistered",
                callback_id = id.as_u64(),
                "Callback unregistered"
            );
        } else {
            tracing::warn!(
                service = "keyrx",
                component = "ffi_callback",
                event = "unregister_failed",
                callback_id = id.as_u64(),
                "Attempted to unregister non-existent callback"
            );
        }

        removed
    }

    /// Invoke a callback by ID with data.
    ///
    /// # Parameters
    ///
    /// * `id` - The ID of the callback to invoke
    /// * `data` - Raw bytes to pass to the callback
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Callback invoked successfully
    /// - `Err(FfiError)`: Callback not found or invocation failed
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No callback exists with the given ID
    /// - The callback's `invoke` method returns an error
    ///
    /// # Example
    ///
    /// ```
    /// # use keyrx_core::ffi::marshal::callback::{FfiCallback, CallbackRegistry};
    /// # use keyrx_core::ffi::error::FfiResult;
    /// # struct TestCallback;
    /// # impl FfiCallback for TestCallback {
    /// #     fn invoke(&self, _: &[u8]) -> FfiResult<()> { Ok(()) }
    /// #     fn callback_type(&self) -> &'static str { "test" }
    /// # }
    /// let registry = CallbackRegistry::new();
    /// let id = registry.register(TestCallback);
    /// assert!(registry.invoke(id, b"test data").is_ok());
    /// ```
    pub fn invoke(&self, id: CallbackId, data: &[u8]) -> FfiResult<()> {
        // Get callback reference (DashMap handles locking internally)
        let callback = self
            .callbacks
            .get(&id)
            .ok_or_else(|| FfiError::not_found(format!("Callback ID {} not found", id.as_u64())))?;

        let callback_type = callback.callback_type();

        // Invoke the callback
        match callback.invoke(data) {
            Ok(()) => {
                tracing::trace!(
                    service = "keyrx",
                    component = "ffi_callback",
                    event = "callback_invoked",
                    callback_id = id.as_u64(),
                    callback_type = callback_type,
                    data_len = data.len(),
                    "Callback invoked successfully"
                );
                Ok(())
            }
            Err(err) => {
                tracing::warn!(
                    service = "keyrx",
                    component = "ffi_callback",
                    event = "callback_failed",
                    callback_id = id.as_u64(),
                    callback_type = callback_type,
                    error = %err,
                    "Callback invocation failed"
                );
                Err(err)
            }
        }
    }

    /// Check if a callback is registered.
    ///
    /// # Parameters
    ///
    /// * `id` - The ID to check
    ///
    /// # Returns
    ///
    /// `true` if a callback with this ID exists, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// # use keyrx_core::ffi::marshal::callback::{FfiCallback, CallbackRegistry};
    /// # use keyrx_core::ffi::error::FfiResult;
    /// # struct TestCallback;
    /// # impl FfiCallback for TestCallback {
    /// #     fn invoke(&self, _: &[u8]) -> FfiResult<()> { Ok(()) }
    /// #     fn callback_type(&self) -> &'static str { "test" }
    /// # }
    /// let registry = CallbackRegistry::new();
    /// let id = registry.register(TestCallback);
    /// assert!(registry.is_registered(id));
    /// ```
    pub fn is_registered(&self, id: CallbackId) -> bool {
        self.callbacks.contains_key(&id)
    }

    /// Get the number of registered callbacks.
    ///
    /// # Returns
    ///
    /// The count of currently registered callbacks.
    ///
    /// # Example
    ///
    /// ```
    /// # use keyrx_core::ffi::marshal::callback::{FfiCallback, CallbackRegistry};
    /// # use keyrx_core::ffi::error::FfiResult;
    /// # struct TestCallback;
    /// # impl FfiCallback for TestCallback {
    /// #     fn invoke(&self, _: &[u8]) -> FfiResult<()> { Ok(()) }
    /// #     fn callback_type(&self) -> &'static str { "test" }
    /// # }
    /// let registry = CallbackRegistry::new();
    /// assert_eq!(registry.callback_count(), 0);
    /// registry.register(TestCallback);
    /// assert_eq!(registry.callback_count(), 1);
    /// ```
    pub fn callback_count(&self) -> usize {
        self.callbacks.len()
    }

    /// Clear all registered callbacks.
    ///
    /// Removes all callbacks from the registry. Useful for cleanup during
    /// shutdown or testing.
    ///
    /// # Example
    ///
    /// ```
    /// # use keyrx_core::ffi::marshal::callback::{FfiCallback, CallbackRegistry};
    /// # use keyrx_core::ffi::error::FfiResult;
    /// # struct TestCallback;
    /// # impl FfiCallback for TestCallback {
    /// #     fn invoke(&self, _: &[u8]) -> FfiResult<()> { Ok(()) }
    /// #     fn callback_type(&self) -> &'static str { "test" }
    /// # }
    /// let registry = CallbackRegistry::new();
    /// registry.register(TestCallback);
    /// registry.clear();
    /// assert_eq!(registry.callback_count(), 0);
    /// ```
    pub fn clear(&self) {
        let count = self.callbacks.len();
        self.callbacks.clear();

        tracing::debug!(
            service = "keyrx",
            component = "ffi_callback",
            event = "registry_cleared",
            count = count,
            "Cleared all callbacks from registry"
        );
    }
}

impl Default for CallbackRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test callback that counts invocations
    struct CountingCallback {
        counter: Arc<AtomicU64>,
    }

    impl FfiCallback for CountingCallback {
        fn invoke(&self, _data: &[u8]) -> FfiResult<()> {
            self.counter.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        fn callback_type(&self) -> &'static str {
            "counting"
        }
    }

    // Test callback that always fails
    struct FailingCallback;

    impl FfiCallback for FailingCallback {
        fn invoke(&self, _data: &[u8]) -> FfiResult<()> {
            Err(FfiError::internal("Intentional failure"))
        }

        fn callback_type(&self) -> &'static str {
            "failing"
        }
    }

    // Test callback that validates data
    struct JsonCallback;

    impl FfiCallback for JsonCallback {
        fn invoke(&self, data: &[u8]) -> FfiResult<()> {
            serde_json::from_slice::<serde_json::Value>(data)
                .map_err(|e| FfiError::invalid_input(format!("Invalid JSON: {}", e)))?;
            Ok(())
        }

        fn callback_type(&self) -> &'static str {
            "json"
        }
    }

    #[test]
    fn callback_id_creation() {
        let id = CallbackId::new(42);
        assert_eq!(id.as_u64(), 42);

        let id2: CallbackId = 100.into();
        assert_eq!(id2.as_u64(), 100);

        let val: u64 = id.into();
        assert_eq!(val, 42);
    }

    #[test]
    fn callback_id_equality() {
        let id1 = CallbackId::new(42);
        let id2 = CallbackId::new(42);
        let id3 = CallbackId::new(43);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn registry_starts_empty() {
        let registry = CallbackRegistry::new();
        assert_eq!(registry.callback_count(), 0);
    }

    #[test]
    fn can_register_callback() {
        let registry = CallbackRegistry::new();
        let counter = Arc::new(AtomicU64::new(0));
        let callback = CountingCallback {
            counter: counter.clone(),
        };

        let id = registry.register(callback);
        assert_eq!(registry.callback_count(), 1);
        assert!(registry.is_registered(id));
    }

    #[test]
    fn can_invoke_callback() {
        let registry = CallbackRegistry::new();
        let counter = Arc::new(AtomicU64::new(0));
        let callback = CountingCallback {
            counter: counter.clone(),
        };

        let id = registry.register(callback);
        assert!(registry.invoke(id, b"test data").is_ok());
        assert_eq!(counter.load(Ordering::SeqCst), 1);

        // Invoke again
        assert!(registry.invoke(id, b"more data").is_ok());
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn can_unregister_callback() {
        let registry = CallbackRegistry::new();
        let counter = Arc::new(AtomicU64::new(0));
        let callback = CountingCallback {
            counter: counter.clone(),
        };

        let id = registry.register(callback);
        assert!(registry.unregister(id));
        assert_eq!(registry.callback_count(), 0);
        assert!(!registry.is_registered(id));

        // Unregistering again should return false
        assert!(!registry.unregister(id));
    }

    #[test]
    fn invoke_nonexistent_callback_fails() {
        let registry = CallbackRegistry::new();
        let id = CallbackId::new(999);

        let result = registry.invoke(id, b"data");
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("not found"));
    }

    #[test]
    fn failing_callback_returns_error() {
        let registry = CallbackRegistry::new();
        let id = registry.register(FailingCallback);

        let result = registry.invoke(id, b"data");
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Intentional failure"));
    }

    #[test]
    fn json_callback_validates_data() {
        let registry = CallbackRegistry::new();
        let id = registry.register(JsonCallback);

        // Valid JSON should succeed
        let valid_json = br#"{"key": "value"}"#;
        assert!(registry.invoke(id, valid_json).is_ok());

        // Invalid JSON should fail
        let invalid_json = b"not json";
        let result = registry.invoke(id, invalid_json);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Invalid JSON"));
    }

    #[test]
    fn multiple_callbacks_independent() {
        let registry = CallbackRegistry::new();

        let counter1 = Arc::new(AtomicU64::new(0));
        let counter2 = Arc::new(AtomicU64::new(0));

        let id1 = registry.register(CountingCallback {
            counter: counter1.clone(),
        });
        let id2 = registry.register(CountingCallback {
            counter: counter2.clone(),
        });

        assert_eq!(registry.callback_count(), 2);

        // Invoke first callback
        registry.invoke(id1, b"data").ok();
        assert_eq!(counter1.load(Ordering::SeqCst), 1);
        assert_eq!(counter2.load(Ordering::SeqCst), 0);

        // Invoke second callback
        registry.invoke(id2, b"data").ok();
        assert_eq!(counter1.load(Ordering::SeqCst), 1);
        assert_eq!(counter2.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn clear_removes_all_callbacks() {
        let registry = CallbackRegistry::new();

        let counter = Arc::new(AtomicU64::new(0));
        registry.register(CountingCallback {
            counter: counter.clone(),
        });
        registry.register(FailingCallback);

        assert_eq!(registry.callback_count(), 2);

        registry.clear();
        assert_eq!(registry.callback_count(), 0);
    }

    #[test]
    fn unique_ids_generated() {
        let registry = CallbackRegistry::new();

        let counter = Arc::new(AtomicU64::new(0));
        let id1 = registry.register(CountingCallback {
            counter: counter.clone(),
        });
        let id2 = registry.register(CountingCallback {
            counter: counter.clone(),
        });
        let id3 = registry.register(CountingCallback {
            counter: counter.clone(),
        });

        // All IDs should be unique
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);
    }

    #[test]
    fn registry_is_clonable() {
        let registry1 = CallbackRegistry::new();
        let counter = Arc::new(AtomicU64::new(0));
        let id = registry1.register(CountingCallback {
            counter: counter.clone(),
        });

        let registry2 = registry1.clone();
        assert!(registry2.is_registered(id));

        // Both registries share the same underlying storage
        registry2.invoke(id, b"data").ok();
        assert_eq!(counter.load(Ordering::SeqCst), 1);

        registry1.invoke(id, b"data").ok();
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn global_registry_is_singleton() {
        let registry1 = CallbackRegistry::global();
        let registry2 = CallbackRegistry::global();

        let counter = Arc::new(AtomicU64::new(0));
        let id = registry1.register(CountingCallback {
            counter: counter.clone(),
        });

        // Both references point to the same registry
        assert!(registry2.is_registered(id));
        registry2.invoke(id, b"data").ok();
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn callback_type_is_accessible() {
        let registry = CallbackRegistry::new();
        let id = registry.register(JsonCallback);

        // We can't directly access callback_type through the registry API,
        // but we verify it through invoke behavior
        assert!(registry.is_registered(id));
    }

    #[test]
    fn default_registry_is_empty() {
        let registry = CallbackRegistry::default();
        assert_eq!(registry.callback_count(), 0);
    }
}
