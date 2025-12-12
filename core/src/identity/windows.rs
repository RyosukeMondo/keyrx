//! Windows-specific device serial number extraction.
//!
//! This module extracts serial numbers from Windows device paths using:
//! 1. USB iSerial descriptor via HidD_GetSerialNumberString (preferred)
//! 2. Device Instance ID from device path (fallback)
//!
//! Windows device paths have the format:
//! \\?\HID#VID_046D&PID_C52B#7&12345678&0&0000#{...}
//! The Instance ID is: 7&12345678&0&0000

// #[cfg(windows)] implied by parent module
#![allow(unsafe_code)]

use anyhow::{Context, Result};
use std::mem;
use windows::Win32::Devices::HumanInterfaceDevice::HidD_GetSerialNumberString;
use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
};

/// Extract serial number from a Windows device path.
///
/// Attempts to read the USB iSerial descriptor first. If that fails
/// (e.g., for devices without iSerial or port-bound devices),
/// falls back to parsing the Instance ID from the device path.
///
/// # Arguments
/// * `device_path` - Raw Input device path (e.g., `\\?\HID#VID_046D&PID_C52B#7&12345678&0&0000#{...}`)
///
/// # Returns
/// Serial number string or error if extraction fails.
///
/// # Examples
/// ```ignore
/// let serial = extract_serial_number(r"\\?\HID#VID_046D&PID_C52B#InstanceID#{...}")?;
/// ```
pub fn extract_serial_number(device_path: &str) -> Result<String> {
    // Try to read iSerial descriptor first (preferred method)
    if let Ok(serial) = read_iserial_descriptor(device_path) {
        if !serial.is_empty() && serial != "0" {
            tracing::debug!(
                device_path,
                serial,
                method = "iSerial",
                "Extracted serial number"
            );
            return Ok(serial);
        }
    }

    // Fallback to Instance ID from device path
    let instance_id = parse_instance_id_from_path(device_path)?;
    tracing::debug!(
        device_path,
        serial = instance_id,
        method = "InstanceID",
        "Extracted serial number from Instance ID"
    );
    Ok(instance_id)
}

/// Parse the Instance ID from a Windows device path.
///
/// The Instance ID is the portion between the second and third `#` in paths like:
/// `\\?\HID#VID_046D&PID_C52B#7&12345678&0&0000#{...}`
///                              ^^^^^^^^^^^^^^^^^
///
/// For port-bound devices (e.g., PS/2 keyboards), this provides a stable
/// synthetic identifier based on the physical port.
///
/// # Errors
/// Returns error if the path format is invalid or Instance ID cannot be extracted.
fn parse_instance_id_from_path(device_path: &str) -> Result<String> {
    // Remove \\?\ prefix if present
    let path = device_path.strip_prefix(r"\\?\").unwrap_or(device_path);

    // Split by '#' and extract the third segment (index 2)
    let parts: Vec<&str> = path.split('#').collect();
    if parts.len() < 3 {
        anyhow::bail!(
            "Invalid device path format: expected at least 3 '#' separated parts, got {}",
            parts.len()
        );
    }

    let instance_id = parts[2];
    if instance_id.is_empty() {
        anyhow::bail!("Empty Instance ID in device path");
    }

    Ok(instance_id.to_string())
}

/// Read the iSerial USB descriptor using HidD_GetSerialNumberString.
///
/// Opens the device handle and queries the serial number string descriptor.
/// This is the preferred method as it reads the actual USB serial number
/// that manufacturers assign to devices.
///
/// # Errors
/// Returns error if:
/// - Device handle cannot be opened
/// - HID API call fails
/// - Serial string cannot be converted to UTF-16
///
/// # Safety
/// Uses unsafe Win32 API calls with proper handle management.
fn read_iserial_descriptor(device_path: &str) -> Result<String> {
    // Convert path to wide string for Win32 API
    let wide_path: Vec<u16> = device_path.encode_utf16().chain(Some(0)).collect();

    // Open device handle for HID query
    let handle = unsafe {
        CreateFileW(
            windows::core::PCWSTR(wide_path.as_ptr()),
            0, // No access (query only)
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            Default::default(),
            None,
        )?
    };

    // Ensure handle is valid and will be closed
    if handle == INVALID_HANDLE_VALUE || handle.is_invalid() {
        anyhow::bail!("Failed to open device handle");
    }

    // RAII guard to ensure handle is closed
    let _guard = HandleGuard(handle);

    // Buffer for serial string (max 126 UTF-16 chars as per USB spec)
    const MAX_SERIAL_LEN: usize = 126;
    let mut buffer: [u16; MAX_SERIAL_LEN] = [0; MAX_SERIAL_LEN];

    // Query serial number string descriptor
    let success = unsafe {
        HidD_GetSerialNumberString(
            handle,
            buffer.as_mut_ptr() as *mut _,
            (buffer.len() * mem::size_of::<u16>()) as u32,
        )
    };

    if !success.as_bool() {
        anyhow::bail!("HidD_GetSerialNumberString failed");
    }

    // Find null terminator and convert to String
    let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
    let serial = String::from_utf16(&buffer[..len])
        .context("Failed to convert serial number from UTF-16")?;

    Ok(serial)
}

/// RAII guard to ensure Windows HANDLE is closed.
struct HandleGuard(HANDLE);

impl Drop for HandleGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_instance_id_basic() {
        let path = r"\\?\HID#VID_046D&PID_C52B#7&12345678&0&0000#{...}";
        let instance_id = parse_instance_id_from_path(path).unwrap();
        assert_eq!(instance_id, "7&12345678&0&0000");
    }

    #[test]
    fn test_parse_instance_id_without_prefix() {
        let path = r"HID#VID_046D&PID_C52B#InstanceID123#{guid}";
        let instance_id = parse_instance_id_from_path(path).unwrap();
        assert_eq!(instance_id, "InstanceID123");
    }

    #[test]
    fn test_parse_instance_id_complex() {
        // Real-world example from PS/2 keyboard
        let path = r"\\?\ACPI#PNP0303#4&1234abcd&0#{884b96c3-56ef-11d1-bc8c-00a0c91405dd}";
        let instance_id = parse_instance_id_from_path(path).unwrap();
        assert_eq!(instance_id, "4&1234abcd&0");
    }

    #[test]
    fn test_parse_instance_id_invalid_format() {
        let path = r"\\?\InvalidPath";
        assert!(parse_instance_id_from_path(path).is_err());
    }

    #[test]
    fn test_parse_instance_id_empty_instance() {
        let path = r"\\?\HID#VID_046D&PID_C52B##{guid}";
        assert!(parse_instance_id_from_path(path).is_err());
    }

    #[test]
    fn test_parse_instance_id_usb_device() {
        // USB device with serial in path
        let path = r"\\?\HID#VID_1234&PID_5678#ABC123456#{...}";
        let instance_id = parse_instance_id_from_path(path).unwrap();
        assert_eq!(instance_id, "ABC123456");
    }

    // Note: Integration tests with real HID devices are in tests/identity_tests.rs
    // to avoid requiring actual hardware in unit tests.
}
