//! Windows keyboard device enumeration.
//!
//! This module provides functionality for listing keyboard devices on Windows.

#![allow(unsafe_code)]

use crate::drivers::DeviceInfo;
use crate::errors::KeyrxError;
use regex::Regex;
use std::mem::size_of;
use std::path::PathBuf;
use std::sync::OnceLock;
use windows::Win32::UI::Input::{
    GetRawInputDeviceInfoW, GetRawInputDeviceList, RAWINPUTDEVICELIST, RIDI_DEVICENAME,
    RIM_TYPEKEYBOARD,
};

static DEVICE_PATH_REGEX: OnceLock<Regex> = OnceLock::new();

/// List all keyboard devices available on the system.
///
/// On Windows, this uses the Raw Input API to enumerate all connected keyboard devices.
/// It parses the device path to extract Vendor ID and Product ID.
///
/// # Errors
///
/// Returns an error if the Raw Input API fails to return the device list.
pub fn list_keyboards() -> Result<Vec<DeviceInfo>, KeyrxError> {
    let mut device_count = 0;
    // First call to get the number of devices
    unsafe {
        if GetRawInputDeviceList(None, &mut device_count, size_of::<RAWINPUTDEVICELIST>() as u32)
            == u32::MAX
        {
            return Err(KeyrxError::from(anyhow::anyhow!(
                "Failed to get raw input device list count"
            )));
        }
    }

    if device_count == 0 {
        return Ok(vec![]);
    }

    let mut devices = vec![RAWINPUTDEVICELIST::default(); device_count as usize];
    // Second call to get the actual list
    unsafe {
        if GetRawInputDeviceList(
            Some(devices.as_mut_ptr()),
            &mut device_count,
            size_of::<RAWINPUTDEVICELIST>() as u32,
        ) == u32::MAX
        {
            return Err(KeyrxError::from(anyhow::anyhow!(
                "Failed to get raw input device list"
            )));
        }
    }

    let mut result = Vec::new();
    let re = DEVICE_PATH_REGEX.get_or_init(|| {
        Regex::new(r"(?i)VID_([0-9A-F]{4})&PID_([0-9A-F]{4})").expect("Invalid regex")
    });

    for device in devices {
        if device.dwType != RIM_TYPEKEYBOARD {
            continue;
        }

        let mut name_len = 0;
        // First call to get the length of the device name
        unsafe {
            GetRawInputDeviceInfoW(device.hDevice, RIDI_DEVICENAME, None, &mut name_len);
        }

        if name_len == 0 {
            continue;
        }

        let mut name_buffer = vec![0u16; name_len as usize];
        // Second call to get the device name
        let bytes_copied = unsafe {
            GetRawInputDeviceInfoW(
                device.hDevice,
                RIDI_DEVICENAME,
                Some(name_buffer.as_mut_ptr() as *mut _),
                &mut name_len,
            )
        };

        if bytes_copied == u32::MAX || bytes_copied == 0 {
            continue;
        }

        let name_string = String::from_utf16_lossy(&name_buffer[..bytes_copied as usize]);
        // Remove null terminator if present
        let name_clean = name_string.trim_matches(char::from(0)).to_string();

        let mut vendor_id = 0;
        let mut product_id = 0;

        if let Some(caps) = re.captures(&name_clean) {
            if let (Some(vid), Some(pid)) = (caps.get(1), caps.get(2)) {
                vendor_id = u16::from_str_radix(vid.as_str(), 16).unwrap_or(0);
                product_id = u16::from_str_radix(pid.as_str(), 16).unwrap_or(0);
            }
        }

        // Provide a friendly name based on IDs
        let name = if vendor_id != 0 || product_id != 0 {
            format!("HID Keyboard ({:04X}:{:04X})", vendor_id, product_id)
        } else {
            "HID Keyboard (Generic)".to_string()
        };

        // Create device info
        // Note: Windows Raw Input devices are separate from the Global Hook,
        // but our InputSource currently relies on the Global Hook.
        // This enumeration is mainly for identifying what's connected.
        result.push(DeviceInfo::new(
            PathBuf::from(name_clean),
            name,
            vendor_id,
            product_id,
            true,
        ));
    }

    // Sort by name for consistent ordering
    result.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_keyboards_runs_without_panic() {
        // We can't assert much about the result since it depends on the host,
        // but it shouldn't panic.
        let _ = list_keyboards();
    }
}
