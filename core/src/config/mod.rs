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
//! - [`loader`]: Runtime configuration loading from TOML

// Note: Allow unused until submodule constants are implemented (tasks 2-6, 13)
#![allow(unused_imports)]

pub mod exit_codes;
mod keys;
mod limits;
mod loader;
mod paths;
mod timing;

pub use exit_codes::*;
pub use keys::*;
pub use limits::*;
pub use loader::*;
pub use paths::*;
pub use timing::*;
