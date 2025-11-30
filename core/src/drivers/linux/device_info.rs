//! Linux keyboard device information and detection.
//!
//! This module provides functionality for querying evdev devices and
//! determining which ones are keyboards based on their capabilities.

use crate::drivers::DeviceInfo;
use std::path::Path;
use tracing::debug;

/// Attempts to open a device path and return keyboard info if it's a keyboard.
///
/// This function checks if a device has keyboard capabilities by verifying
/// it supports essential keyboard keys (A, Enter, Space).
///
/// # Arguments
///
/// * `path` - Path to the evdev device to check
///
/// # Returns
///
/// - `Some(DeviceInfo)` if the device is a keyboard
/// - `None` if:
///   - The file is not an event device
///   - The device cannot be opened
///   - The device lacks keyboard capabilities
pub fn try_get_keyboard_info(path: &Path) -> Option<DeviceInfo> {
    let file_name = path.file_name().and_then(|n| n.to_str())?;
    if !file_name.starts_with("event") {
        return None;
    }

    let device = match evdev::Device::open(path) {
        Ok(d) => d,
        Err(e) => {
            debug!(
                service = "keyrx",
                event = "device_open_failed",
                component = "linux_device_info",
                path = %path.display(),
                error = %e,
                "Could not open device for detection"
            );
            return None;
        }
    };

    let has_keyboard_keys = device
        .supported_keys()
        .map(|keys| {
            keys.contains(evdev::Key::KEY_A)
                && keys.contains(evdev::Key::KEY_ENTER)
                && keys.contains(evdev::Key::KEY_SPACE)
        })
        .unwrap_or(false);

    if !has_keyboard_keys {
        return None;
    }

    let name = device.name().unwrap_or("Unknown Device").to_string();
    let input_id = device.input_id();
    Some(DeviceInfo::new(
        path.to_path_buf(),
        name,
        input_id.vendor(),
        input_id.product(),
        true,
    ))
}
