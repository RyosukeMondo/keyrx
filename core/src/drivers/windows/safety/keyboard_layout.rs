//! Keyboard layout utilities for Windows.
//!
//! This module provides safe wrappers for keyboard layout related Windows APIs,
//! specifically for converting between scan codes and virtual keys.

#[cfg(target_os = "windows")]
use windows::Win32::UI::Input::KeyboardAndMouse::{MapVirtualKeyW, MAPVK_VSC_TO_VK};

/// Converts a hardware scan code to a Windows Virtual Key (VK) code.
///
/// This uses the system's current keyboard layout.
///
/// # Arguments
/// * `scan_code` - The hardware scan code to convert.
///
/// # Returns
/// * The corresponding Virtual Key code, or 0 if translation is not possible.
#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
pub fn scan_code_to_vk(scan_code: u16) -> u16 {
    // SAFETY: MapVirtualKeyW is a read-only Windows API call that queries the
    // keyboard layout. It does not modify system state and is safe to call.
    // We cast u16 scan_code to u32 as required by the API.
    // MAPVK_VSC_TO_VK (1) requests translation from scan code to VK.
    unsafe { MapVirtualKeyW(scan_code as u32, MAPVK_VSC_TO_VK) as u16 }
}

/// Dummy implementation for non-Windows platforms to satisfy the compiler
/// if this module is included elsewhere (though typically guarded by cfg).
#[cfg(not(target_os = "windows"))]
pub fn scan_code_to_vk(_scan_code: u16) -> u16 {
    0
}
