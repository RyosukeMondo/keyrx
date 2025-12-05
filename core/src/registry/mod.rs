//! Registry module for revolutionary mapping system.
//!
//! This module provides the core data structures and registries for managing
//! device identities, profiles, and bindings in the revolutionary mapping system.

pub mod device;
pub mod profile;

// Re-export commonly used types
pub use device::{DeviceEvent, DeviceRegistry, DeviceRegistryError, DeviceState};
pub use profile::{KeyAction, LayoutType, PhysicalPosition, Profile, ProfileId};
