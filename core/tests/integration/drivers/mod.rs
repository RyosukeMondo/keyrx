//! Integration tests for driver safety and error handling.
//!
//! This module contains integration tests for the driver safety infrastructure,
//! including error handling, recovery mechanisms, and platform-specific safety wrappers.

pub mod error_recovery;

#[cfg(target_os = "windows")]
pub mod windows_safety;

#[cfg(target_os = "linux")]
pub mod linux_safety;
