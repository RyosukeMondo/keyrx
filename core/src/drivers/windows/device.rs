//! Windows keyboard device enumeration.
//!
//! This module provides functionality for listing keyboard devices on Windows.

use crate::drivers::DeviceInfo;
use anyhow::Result;
use std::path::PathBuf;

/// List all keyboard devices available on the system.
///
/// On Windows, this returns a single entry representing the system keyboard.
/// Windows uses a global low-level keyboard hook (WH_KEYBOARD_LL) which
/// intercepts all keyboard input regardless of which physical device generated it.
///
/// # Errors
///
/// This function currently always succeeds on Windows.
pub fn list_keyboards() -> Result<Vec<DeviceInfo>> {
    // Windows uses a global keyboard hook that captures all keyboard input.
    // We return a single "virtual" device representing the system keyboard.
    // Full HID device enumeration could be added later for device-specific handling.
    Ok(vec![DeviceInfo::new(
        PathBuf::from("\\\\?\\HID#System#Keyboard"),
        "System Keyboard (Global Hook)".to_string(),
        0, // Vendor ID not available via global hook
        0, // Product ID not available via global hook
        true,
    )])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_keyboards_returns_system_keyboard() {
        let keyboards = list_keyboards().unwrap();
        assert_eq!(keyboards.len(), 1);
        assert!(keyboards[0].is_keyboard());
        assert!(keyboards[0].name().contains("System Keyboard"));
    }
}
