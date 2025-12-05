//! Device identity module for unique device identification.
//!
//! This module provides the DeviceIdentity type which uniquely identifies
//! physical device instances using VID:PID:Serial triplets. This enables
//! per-device configuration, allowing users to have different mappings
//! and profiles for identical device models.

mod types;

pub use types::DeviceIdentity;

// Platform-specific serial extraction modules
#[cfg(windows)]
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;
