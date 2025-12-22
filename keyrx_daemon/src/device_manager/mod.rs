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

#[cfg(feature = "linux")]
mod linux;

#[cfg(feature = "linux")]
pub use linux::{enumerate_keyboards, match_device, DeviceManager, ManagedDevice};

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
#[derive(Debug, Clone)]
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
}
