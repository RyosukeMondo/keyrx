//! Device discovery module.
//!
//! Contains shared types and helpers for keyboard discovery, profile
//! persistence, and registry lookup. Wiring into the engine/CLI/FFI is
//! implemented in downstream tasks.

pub mod storage;
pub mod types;

pub use storage::{
    default_profile_for, profile_path, read_profile, validate_schema, write_profile, StorageError,
};
pub use types::{
    default_schema_version, device_profiles_dir, DeviceId, DeviceProfile, PhysicalKey,
    ProfileSource, SCHEMA_VERSION,
};
