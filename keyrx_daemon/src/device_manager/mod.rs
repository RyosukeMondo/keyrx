//! Device discovery and management for keyboard input devices.
//!
//! This module provides functionality for discovering available keyboard devices,
//! matching them against configuration patterns, and managing device lifecycle.
//!
//! # Overview
//!
//! The device management system consists of several components:
//!
//! - [`KeyboardInfo`]: Information about a discovered keyboard device
//! - [`enumerate_keyboards`]: Discovers available keyboard devices
//! - [`match_device`]: Matches devices against configuration patterns
//! - [`DeviceManager`]: Manages multiple devices and matches them to configurations
//! - [`ManagedDevice`]: A device paired with its configuration and runtime state

use crate::platform::DeviceError;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
mod linux_enum;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
pub use linux::{DeviceManager, ManagedDevice, RefreshResult};
#[cfg(target_os = "linux")]
pub use linux_enum::enumerate_keyboards;
#[cfg(target_os = "windows")]
pub use windows::{enumerate_keyboards, DeviceManager, ManagedDevice, RefreshResult};

/// Matches a device against a `device_start` pattern: THE shared glob rule
/// (`keyrx_core::runtime::device_pattern`) over the device's name, serial and
/// physical path.
pub fn match_device(device: &KeyboardInfo, pattern: &str) -> bool {
    keyrx_core::runtime::device_pattern::matches_any(&device.identities(), pattern)
}

/// Errors that can occur during device discovery.
#[derive(Debug, thiserror::Error)]
pub enum DiscoveryError {
    /// No keyboard devices were found on the system.
    #[error("no keyboard devices found")]
    NoDevicesFound,

    /// Failed to access a device during enumeration.
    #[error("failed to access device: {0}")]
    AccessError(#[from] DeviceError),

    /// I/O error during device enumeration.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Information about a discovered keyboard device.
///
/// Two `KeyboardInfo` instances are considered equal if they have the same path,
/// which is the unique identifier for a device node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyboardInfo {
    /// Path to the device node (e.g., `/dev/input/event0`).
    pub path: std::path::PathBuf,
    /// Human-readable device name.
    pub name: String,
    /// Serial number if available.
    pub serial: Option<String>,
    /// Physical location identifier if available.
    pub phys: Option<String>,
}

impl KeyboardInfo {
    /// Returns a unique device ID for this keyboard.
    ///
    /// The ID is generated from the serial number if available, otherwise
    /// falls back to a path-based identifier for stability.
    #[must_use]
    /// The strings device patterns are matched against: name, serial and
    /// physical path (when known).
    pub fn identities(&self) -> Vec<&str> {
        let mut ids = vec![self.name.as_str()];
        ids.extend(self.serial.as_deref());
        ids.extend(self.phys.as_deref());
        ids
    }

    pub fn device_id(&self) -> String {
        if let Some(ref serial) = self.serial {
            if !serial.is_empty() {
                return format!("serial-{}", serial);
            }
        }
        // Fallback to path-based ID
        format!("path-{}", self.path.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discovery_error_display() {
        let err = DiscoveryError::NoDevicesFound;
        assert_eq!(err.to_string(), "no keyboard devices found");
    }

    #[test]
    fn test_keyboard_info_debug() {
        let info = KeyboardInfo {
            path: std::path::PathBuf::from("/dev/input/event0"),
            name: "Test Keyboard".to_string(),
            serial: Some("ABC123".to_string()),
            phys: Some("usb-0000:00:14.0-1/input0".to_string()),
        };
        let debug_str = format!("{:?}", info);
        assert!(debug_str.contains("Test Keyboard"));
        assert!(debug_str.contains("ABC123"));
    }

    #[test]
    fn test_keyboard_info_clone() {
        let info = KeyboardInfo {
            path: std::path::PathBuf::from("/dev/input/event0"),
            name: "Test Keyboard".to_string(),
            serial: None,
            phys: None,
        };
        let cloned = info.clone();
        assert_eq!(cloned.name, info.name);
        assert_eq!(cloned.path, info.path);
    }

    #[test]
    fn test_keyboard_info_equality_same_path() {
        let info1 = KeyboardInfo {
            path: std::path::PathBuf::from("/dev/input/event0"),
            name: "Keyboard A".to_string(),
            serial: Some("SN1".to_string()),
            phys: None,
        };
        let info2 = KeyboardInfo {
            path: std::path::PathBuf::from("/dev/input/event0"),
            name: "Keyboard A".to_string(),
            serial: Some("SN1".to_string()),
            phys: None,
        };
        assert_eq!(info1, info2);
    }

    #[test]
    fn test_keyboard_info_inequality_different_path() {
        let info1 = KeyboardInfo {
            path: std::path::PathBuf::from("/dev/input/event0"),
            name: "Keyboard A".to_string(),
            serial: None,
            phys: None,
        };
        let info2 = KeyboardInfo {
            path: std::path::PathBuf::from("/dev/input/event1"),
            name: "Keyboard A".to_string(),
            serial: None,
            phys: None,
        };
        assert_ne!(info1, info2);
    }

    #[test]
    fn test_keyboard_info_equality_all_fields_matter() {
        // All fields contribute to equality per derive
        let info1 = KeyboardInfo {
            path: std::path::PathBuf::from("/dev/input/event0"),
            name: "Keyboard A".to_string(),
            serial: Some("SN1".to_string()),
            phys: Some("usb-1".to_string()),
        };
        let info2 = KeyboardInfo {
            path: std::path::PathBuf::from("/dev/input/event0"),
            name: "Keyboard B".to_string(), // Different name
            serial: Some("SN1".to_string()),
            phys: Some("usb-1".to_string()),
        };
        assert_ne!(info1, info2);
    }
}
