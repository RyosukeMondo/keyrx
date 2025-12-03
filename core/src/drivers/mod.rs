//! OS-specific input drivers.
//!
//! This module provides platform-agnostic abstractions for keyboard input capture
//! and output injection. The [`PlatformInput`] type alias resolves to the
//! appropriate driver for the current operating system.

pub mod bypass;
pub mod common;
pub mod emergency_exit;
mod injector;
pub mod keycodes;

pub use bypass::BypassController;

pub use injector::{InjectedKey, KeyInjector, MockKeyInjector};

#[cfg(all(target_os = "windows", feature = "windows-driver"))]
pub mod windows;

#[cfg(all(target_os = "linux", feature = "linux-driver"))]
pub mod linux;

pub use common::DeviceInfo;

#[cfg(all(target_os = "windows", feature = "windows-driver"))]
pub use windows::WindowsInput;

#[cfg(all(target_os = "linux", feature = "linux-driver"))]
pub use linux::LinuxInput;

/// Platform-specific input driver.
///
/// This type alias resolves to:
/// - [`LinuxInput`] on Linux (using evdev/uinput)
/// - `WindowsInput` on Windows (using WH_KEYBOARD_LL hooks)
#[cfg(all(target_os = "linux", feature = "linux-driver"))]
pub type PlatformInput = LinuxInput;

/// Platform-specific input driver.
///
/// This type alias resolves to:
/// - `LinuxInput` on Linux (using evdev/uinput)
/// - [`WindowsInput`] on Windows (using WH_KEYBOARD_LL hooks)
#[cfg(all(target_os = "windows", feature = "windows-driver"))]
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
#[cfg(all(target_os = "linux", feature = "linux-driver"))]
pub fn list_keyboards() -> Result<Vec<DeviceInfo>, crate::errors::KeyrxError> {
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
#[cfg(all(target_os = "windows", feature = "windows-driver"))]
pub fn list_keyboards() -> Result<Vec<DeviceInfo>, crate::errors::KeyrxError> {
    windows::list_keyboards()
}

/// Fallback implementation when no platform driver is available.
///
/// Returns an error indicating that no platform driver is compiled in.
#[cfg(not(any(
    all(target_os = "linux", feature = "linux-driver"),
    all(target_os = "windows", feature = "windows-driver")
)))]
pub fn list_keyboards() -> Result<Vec<DeviceInfo>, crate::errors::KeyrxError> {
    use crate::errors::driver::DRIVER_INITIALIZATION_FAILED;
    use crate::keyrx_err;

    Err(keyrx_err!(
        DRIVER_INITIALIZATION_FAILED,
        reason =
            "No platform driver compiled in. Enable 'linux-driver' or 'windows-driver' feature."
                .to_string()
    ))
}

/// Fallback type alias when no platform driver is available.
///
/// This uses MockKeyInjector as a placeholder when no real driver is compiled in.
#[cfg(not(any(
    all(target_os = "linux", feature = "linux-driver"),
    all(target_os = "windows", feature = "windows-driver")
)))]
pub type PlatformInput = crate::mocks::MockInput;
