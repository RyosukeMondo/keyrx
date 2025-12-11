//! Registry module for revolutionary mapping system.
//!
//! This module provides the core data structures and registries for managing
//! device identities, profiles, and bindings in the revolutionary mapping system.

pub mod bindings;
pub mod device;
pub mod profile;

// Re-export profile extension traits for convenient usage
pub use profile::{ProfileRegistryResolution, ProfileRegistryStorage};

// Re-export commonly used types
pub use bindings::{DeviceBinding, DeviceBindings, DeviceBindingsError};
pub use device::{DeviceEvent, DeviceRegistry, DeviceRegistryError, DeviceState};
pub use profile::{
    KeyAction, LayoutType, PhysicalPosition, Profile, ProfileId, ProfileRegistry,
    ProfileRegistryError,
};
