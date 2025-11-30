//! OS-specific input drivers.

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
