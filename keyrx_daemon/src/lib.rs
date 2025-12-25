//! KeyRx daemon library.
//!
//! This library provides platform abstraction and event processing for the KeyRx daemon.

// test_utils must come first to make macros available to other modules
#[cfg(any(target_os = "linux", target_os = "windows"))]
#[macro_use]
pub mod test_utils;

pub mod config_loader;
#[cfg(any(target_os = "linux", target_os = "windows"))]
pub mod daemon;
pub mod device_manager;
pub mod platform;
pub mod processor;

#[cfg(feature = "web")]
pub mod web;
