//! Configuration management module
//!
//! This module provides components for managing device metadata,
//! profiles, layouts, and configuration generation.

pub mod device_registry;
pub mod layout_manager;
pub mod profile_manager;
pub mod rhai_generator;

pub use device_registry::{DeviceEntry, DeviceRegistry, DeviceScope, RegistryError};
pub use layout_manager::{KeyboardLayout, LayoutError, LayoutManager, LayoutSource};
pub use profile_manager::{
    ActivationResult, ProfileError, ProfileManager, ProfileMetadata, ProfileTemplate,
};
pub use rhai_generator::{GeneratorError, KeyAction, LayerMode, MacroStep, RhaiGenerator};
