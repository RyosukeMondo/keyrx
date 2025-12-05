//! Device discovery module.
//!
//! Contains shared types and helpers for keyboard discovery, profile
//! persistence, and registry lookup. Wiring into the engine/CLI/FFI is
//! implemented in downstream tasks.

pub mod registry;
pub mod session;
pub mod storage;
pub mod types;
pub mod watcher;

pub use registry::{DeviceRegistry, DiscoveryReason, RegistryEntry, RegistryStatus};
pub use session::{
    DiscoveryProgress, DiscoverySession, DiscoverySummary, DuplicateWarning, ExpectedPosition,
    SessionError, SessionStatus, SessionUpdate,
};
pub use storage::{
    default_profile_for, profile_path, read_profile, validate_schema, write_profile, StorageError,
};
pub use types::{
    default_schema_version, device_profiles_dir, DeviceId, DeviceProfile, PhysicalKey,
    ProfileSource, SCHEMA_VERSION,
};
pub use watcher::{
    DeviceEvent, DeviceEventReceiver, DeviceEventSender, DeviceState, DeviceWatchError,
    DeviceWatcher, WatcherResult,
};

#[cfg(test)]
pub(crate) mod test_utils {
    use std::sync::{Mutex, OnceLock};

    pub(crate) fn config_env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }
}
