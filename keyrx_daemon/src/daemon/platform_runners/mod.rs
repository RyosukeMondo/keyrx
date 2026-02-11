//! Platform-specific daemon runners.
//!
//! This module contains the platform-specific implementations for running the daemon
//! with keyboard capture, web server, and IPC infrastructure.

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;
