//! KeyRx daemon library.
//!
//! This library provides platform abstraction and event processing for the KeyRx daemon.

// test_utils must come first to make macros available to other modules
#[cfg(any(target_os = "linux", target_os = "windows"))]
#[macro_use]
pub mod test_utils;

pub mod auth;
pub mod cli;
pub mod config;
pub mod config_loader;
#[cfg(any(target_os = "linux", target_os = "windows"))]
pub mod daemon;
pub mod device_manager;
pub mod error;
pub mod ipc;
pub mod macro_recorder;
pub mod platform;
pub mod processor;
pub mod services;
pub mod test_mode;
pub mod validation;
pub mod web;
