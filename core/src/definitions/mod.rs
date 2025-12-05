//! Device definitions module for revolutionary mapping system.
//!
//! This module provides data structures and utilities for loading and managing
//! device layout definitions from TOML files. Device definitions describe the
//! physical layout of input devices, enabling layout-aware key remapping.

pub mod types;

// Re-export commonly used types
pub use types::{DeviceDefinition, LayoutDefinition, VisualMetadata};
