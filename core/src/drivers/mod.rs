//! OS-specific input drivers.
//!
//! This module provides platform-agnostic abstractions for keyboard input capture
//! and output injection. The [`PlatformInput`] type alias resolves to the
//! appropriate driver for the current operating system.

mod common;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

pub use common::DeviceInfo;

#[cfg(target_os = "windows")]
pub use windows::WindowsInput;

#[cfg(target_os = "linux")]
pub use linux::LinuxInput;

/// Platform-specific input driver.
///
/// This type alias resolves to:
/// - [`LinuxInput`] on Linux (using evdev/uinput)
/// - [`WindowsInput`] on Windows (using WH_KEYBOARD_LL hooks)
#[cfg(target_os = "linux")]
pub type PlatformInput = LinuxInput;

/// Platform-specific input driver.
///
/// This type alias resolves to:
/// - [`LinuxInput`] on Linux (using evdev/uinput)
/// - [`WindowsInput`] on Windows (using WH_KEYBOARD_LL hooks)
#[cfg(target_os = "windows")]
pub type PlatformInput = WindowsInput;

/// List all available keyboard devices on the current platform.
///
/// Returns a vector of [`DeviceInfo`] structs describing each detected keyboard.
/// On Linux, this scans `/dev/input/event*` devices for keyboard capability.
/// On Windows, this returns information about attached HID keyboards.
///
/// # Errors
///
/// Returns an error if device enumeration fails (e.g., permission denied).
#[cfg(target_os = "linux")]
pub fn list_keyboards() -> anyhow::Result<Vec<DeviceInfo>> {
    linux::list_keyboards()
}

/// List all available keyboard devices on the current platform.
///
/// Returns a vector of [`DeviceInfo`] structs describing each detected keyboard.
/// On Linux, this scans `/dev/input/event*` devices for keyboard capability.
/// On Windows, this returns information about attached HID keyboards.
///
/// # Errors
///
/// Returns an error if device enumeration fails (e.g., permission denied).
#[cfg(target_os = "windows")]
pub fn list_keyboards() -> anyhow::Result<Vec<DeviceInfo>> {
    windows::list_keyboards()
}
