//! KeyRx daemon library.
//!
//! This library provides platform abstraction and event processing for the KeyRx daemon.

pub mod config_loader;
#[cfg(feature = "linux")]
pub mod daemon;
pub mod device_manager;
pub mod platform;
pub mod processor;

#[cfg(feature = "linux")]
pub mod test_utils;

#[cfg(feature = "web")]
pub mod web;
