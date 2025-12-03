//! KeyRx Core - The Ultimate Input Remapping Engine
//!
//! This crate provides the core functionality for KeyRx:
//! - Async event loop for input processing
//! - Rhai scripting integration
//! - Layer and modifier state management
//! - OS-specific input drivers
//! - FFI exports for Flutter integration

pub mod cli;
pub mod config;
pub mod discovery;
pub mod drivers;
pub mod engine;
pub mod error;
pub mod ffi;
pub mod mocks;
pub mod scripting;
pub mod traits;
pub mod uat;
pub mod validation;

// Re-export commonly used types
pub use discovery::{
    default_schema_version, device_profiles_dir, DeviceId, DeviceProfile, DeviceRegistry,
    DiscoveryProgress, DiscoveryReason, DiscoverySession, DiscoverySummary, DuplicateWarning,
    ExpectedPosition, PhysicalKey, ProfileSource, RegistryEntry, RegistryStatus, SessionError,
    SessionStatus, SessionUpdate, SCHEMA_VERSION,
};
pub use engine::{Engine, InputEvent, KeyCode, Layer, ModifierSet, OutputAction};
pub use error::KeyRxError;
pub use mocks::{MockInput, MockRuntime, MockState};
pub use scripting::RhaiRuntime;
pub use traits::{InputSource, ScriptRuntime, StateStore};
