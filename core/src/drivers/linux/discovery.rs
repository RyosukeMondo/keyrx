//! Linux keyboard device discovery.
//!
//! This module provides functionality for scanning and enumerating keyboard
//! devices available on the system via the evdev interface.

use crate::drivers::DeviceInfo;
use crate::errors::KeyrxError;
use crate::keyrx_err;
use std::path::Path;

use super::device_info::try_get_keyboard_info;

/// List all keyboard devices available on the system.
///
/// Scans `/dev/input/event*` devices and returns information about those
/// that have keyboard capability (EV_KEY with standard keyboard keys).
///
/// # Returns
///
/// A sorted vector of `DeviceInfo` structures for all detected keyboards.
///
/// # Errors
///
/// Returns an error if:
/// - The `/dev/input` directory cannot be read
/// - Device enumeration fails due to permission issues
///
/// # Example
///
/// ```ignore
/// let keyboards = list_keyboards()?;
/// for kb in keyboards {
///     println!("Found keyboard: {} at {}", kb.name, kb.path.display());
/// }
/// ```
pub fn list_keyboards() -> Result<Vec<DeviceInfo>, KeyrxError> {
    use crate::errors::driver::DRIVER_DEVICE_NOT_FOUND;

    let input_dir = Path::new("/dev/input");

    if !input_dir.exists() {
        return Err(keyrx_err!(
            DRIVER_DEVICE_NOT_FOUND,
            device = "/dev/input".to_string(),
            reason =
                "Directory not found. Ensure you are running on a Linux system with evdev support"
                    .to_string()
        ));
    }

    let entries = std::fs::read_dir(input_dir).map_err(|e| {
        keyrx_err!(
            DRIVER_DEVICE_NOT_FOUND,
            device = "/dev/input".to_string(),
            error = e.to_string()
        )
    })?;
    let mut keyboards: Vec<DeviceInfo> = entries
        .flatten()
        .filter_map(|entry| try_get_keyboard_info(&entry.path()))
        .collect();

    keyboards.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(keyboards)
}
