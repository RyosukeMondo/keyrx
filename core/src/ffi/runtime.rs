//! Shared runtime access for revolutionary mapping FFI domains.
//!
//! Provides a thread-safe slot for sharing live `DeviceRegistry` and
//! `ProfileRegistry` instances between the engine and FFI exports. This
//! avoids duplicating state while keeping panic-safe accessors for C ABI
//! functions.

use crate::definitions::DeviceDefinitionLibrary;
use crate::ffi::error::{FfiError, FfiResult};
use crate::registry::{DeviceRegistry, ProfileRegistry};
use std::future::Future;
use std::sync::{Arc, Mutex, OnceLock, RwLock};
use tracing::{error, info, warn};

/// Runtime state required by revolutionary mapping FFI domains.
#[derive(Clone)]
pub struct RevolutionaryRuntime {
    device_registry: DeviceRegistry,
    profile_registry: Arc<ProfileRegistry>,
    device_definitions: Arc<DeviceDefinitionLibrary>,
    rhai_runtime: Arc<Mutex<crate::scripting::RhaiRuntime>>,
}

impl RevolutionaryRuntime {
    /// Create a new runtime wrapper from shared registries.
    pub fn new(
        device_registry: DeviceRegistry,
        profile_registry: Arc<ProfileRegistry>,
        device_definitions: Arc<DeviceDefinitionLibrary>,
        rhai_runtime: Arc<Mutex<crate::scripting::RhaiRuntime>>,
    ) -> Self {
        Self {
            device_registry,
            profile_registry,
            device_definitions,
            rhai_runtime,
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

    /// Access the shared scripting runtime.
    pub fn rhai_runtime(&self) -> &Arc<Mutex<crate::scripting::RhaiRuntime>> {
        &self.rhai_runtime
    }

    /// Access the shared device definition library.
    pub fn device_definitions(&self) -> &Arc<DeviceDefinitionLibrary> {
        &self.device_definitions
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
pub fn set_revolutionary_runtime(runtime: RevolutionaryRuntime) -> FfiResult<()> {
    let mut guard = runtime_slot().write().map_err(|_| {
        error!(
            service = "keyrx",
            component = "ffi_runtime",
            event = "runtime_lock_poisoned",
            "Failed to acquire runtime slot for initialization"
        );
        FfiError::internal("runtime lock poisoned")
    })?;

    if guard.is_some() {
        warn!(
            service = "keyrx",
            component = "ffi_runtime",
            event = "runtime_replaced",
            "Replacing existing revolutionary runtime instance"
        );
    }

    *guard = Some(Arc::new(runtime));

    info!(
        service = "keyrx",
        component = "ffi_runtime",
        event = "runtime_initialized",
        "Revolutionary runtime initialized"
    );

    Ok(())
}

/// Clear the shared runtime slot (primarily for tests).
pub fn clear_revolutionary_runtime() -> FfiResult<()> {
    let mut guard = runtime_slot().write().map_err(|_| {
        error!(
            service = "keyrx",
            component = "ffi_runtime",
            event = "runtime_lock_poisoned",
            "Failed to acquire runtime slot for clearing"
        );
        FfiError::internal("runtime lock poisoned")
    })?;

    if guard.take().is_some() {
        info!(
            service = "keyrx",
            component = "ffi_runtime",
            event = "runtime_cleared",
            "Revolutionary runtime cleared"
        );
    } else {
        warn!(
            service = "keyrx",
            component = "ffi_runtime",
            event = "runtime_clear_noop",
            "clear_revolutionary_runtime called with no runtime set"
        );
    }

    Ok(())
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

/// Get a clone of the current revolutionary runtime if initialized.
pub fn get_revolutionary_runtime() -> Option<Arc<RevolutionaryRuntime>> {
    runtime_slot().read().ok().and_then(|g| g.clone())
}

/// Guard that clears the shared runtime when dropped.
pub struct RevolutionaryRuntimeGuard;

impl RevolutionaryRuntimeGuard {
    /// Install the given runtime and return a guard that will clear it on drop.
    pub fn install(runtime: RevolutionaryRuntime) -> FfiResult<Self> {
        set_revolutionary_runtime(runtime)?;
        Ok(Self)
    }
}

impl Drop for RevolutionaryRuntimeGuard {
    fn drop(&mut self) {
        if let Err(err) = clear_revolutionary_runtime() {
            error!(
                service = "keyrx",
                component = "ffi_runtime",
                event = "runtime_clear_failed",
                error = %err,
                "Failed to clear revolutionary runtime"
            );
        }
    }
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
