//! Signal handling abstraction for daemon lifecycle management.
//!
//! This module provides a platform-agnostic interface for signal handling,
//! with platform-specific implementations in the linux and windows submodules.

// Platform-specific implementations
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
pub use linux::{install_signal_handlers, SignalHandler};
#[cfg(target_os = "windows")]
pub use windows::{install_signal_handlers, SignalHandler};
