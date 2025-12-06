//! Shared runtime access for revolutionary mapping FFI domains.
//!
//! Provides a thread-safe slot for sharing live `DeviceRegistry` and
//! `ProfileRegistry` instances between the engine and FFI exports. This
//! avoids duplicating state while keeping panic-safe accessors for C ABI
//! functions.

use crate::ffi::error::{FfiError, FfiResult};
use crate::registry::{DeviceRegistry, ProfileRegistry};
use std::future::Future;
use std::sync::{Arc, OnceLock, RwLock};

/// Runtime state required by revolutionary mapping FFI domains.
#[derive(Clone)]
pub struct RevolutionaryRuntime {
    device_registry: DeviceRegistry,
    profile_registry: Arc<ProfileRegistry>,
}

impl RevolutionaryRuntime {
    /// Create a new runtime wrapper from shared registries.
    pub fn new(device_registry: DeviceRegistry, profile_registry: Arc<ProfileRegistry>) -> Self {
        Self {
            device_registry,
            profile_registry,
        }
    }

    /// Access the shared device registry.
    pub fn device_registry(&self) -> &DeviceRegistry {
        &self.device_registry
    }

    /// Access the shared profile registry.
    pub fn profile_registry(&self) -> &Arc<ProfileRegistry> {
        &self.profile_registry
    }
}

fn runtime_slot() -> &'static RwLock<Option<Arc<RevolutionaryRuntime>>> {
    static SLOT: OnceLock<RwLock<Option<Arc<RevolutionaryRuntime>>>> = OnceLock::new();
    SLOT.get_or_init(|| RwLock::new(None))
}

/// Register the shared runtime for FFI domains.
///
/// Subsequent calls replace the previous runtime. Callers should supply
/// the same instances used by the engine to keep state consistent.
pub fn set_revolutionary_runtime(runtime: RevolutionaryRuntime) {
    if let Ok(mut guard) = runtime_slot().write() {
        *guard = Some(Arc::new(runtime));
    }
}

/// Clear the shared runtime slot (primarily for tests).
pub fn clear_revolutionary_runtime() {
    if let Ok(mut guard) = runtime_slot().write() {
        guard.take();
    }
}

/// Execute a function with the shared runtime, returning an FFI-friendly error
/// if no runtime has been registered or the lock is poisoned.
pub fn with_revolutionary_runtime<F, T>(f: F) -> FfiResult<T>
where
    F: FnOnce(&RevolutionaryRuntime) -> FfiResult<T>,
{
    let guard = runtime_slot()
        .read()
        .map_err(|_| FfiError::internal("runtime lock poisoned"))?;

    let runtime = guard
        .as_ref()
        .ok_or_else(|| FfiError::internal("revolutionary runtime not initialized"))?;

    f(runtime)
}

/// Execute an async operation using the current Tokio runtime if one is
/// available, otherwise create a lightweight current-thread runtime.
pub fn block_on_ffi<Fut, T>(future: Fut) -> FfiResult<T>
where
    Fut: Future<Output = FfiResult<T>>,
{
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        return handle.block_on(future);
    }

    tokio::runtime::Runtime::new()
        .map_err(|e| FfiError::internal(format!("failed to create runtime: {e}")))?
        .block_on(future)
}
