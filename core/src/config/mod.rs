//! Centralized configuration module for KeyRx.
//!
//! This module provides all configuration constants and runtime configuration loading
//! for the KeyRx engine. Constants are organized into logical submodules:
//!
//! - [`timing`]: Timing constants for tap detection, combos, and holds
//! - [`keys`]: Platform-specific key codes (evdev for Linux, VK for Windows)
//! - [`paths`]: File system paths for config, devices, and scripts
//! - [`limits`]: Capacity and threshold limits
//! - [`exit_codes`]: CLI exit code definitions
//! - [`scripting`]: Script execution security settings
//! - [`loader`]: Runtime configuration loading from TOML

pub mod exit_codes;
mod keys;
mod limits;
mod loader;
mod migration;
mod paths;
pub mod scripting;
mod timing;

pub use exit_codes::*;
pub use keys::*;
pub use limits::*;
pub use loader::*;
pub use migration::*;
pub use paths::*;
pub use scripting::*;
pub use timing::*;
