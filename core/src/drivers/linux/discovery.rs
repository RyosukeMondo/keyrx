//! Linux keyboard device discovery.
//!
//! This module provides functionality for scanning and enumerating keyboard
//! devices available on the system via the evdev interface.

use crate::drivers::DeviceInfo;
use anyhow::{bail, Context, Result};
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
pub fn list_keyboards() -> Result<Vec<DeviceInfo>> {
    let input_dir = Path::new("/dev/input");

    if !input_dir.exists() {
        bail!(
            "/dev/input directory not found\n\n\
             Remediation:\n  \
             1. Ensure you are running on a Linux system with evdev support\n  \
             2. Check if the input subsystem is loaded"
        );
    }

    let entries = std::fs::read_dir(input_dir).context("Failed to read /dev/input directory")?;
    let mut keyboards: Vec<DeviceInfo> = entries
        .flatten()
        .filter_map(|entry| try_get_keyboard_info(&entry.path()))
        .collect();

    keyboards.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(keyboards)
}
