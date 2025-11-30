//! Device discovery module.
//!
//! Contains shared types and helpers for keyboard discovery, profile
//! persistence, and registry lookup. Wiring into the engine/CLI/FFI is
//! implemented in downstream tasks.

pub mod types;

pub use types::{
    device_profiles_dir, default_schema_version, DeviceId, DeviceProfile, PhysicalKey,
    ProfileSource, SCHEMA_VERSION,
};
