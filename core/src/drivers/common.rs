//! Common types shared across platform-specific drivers.

use serde::Serialize;
use std::fmt;
use std::path::PathBuf;

/// Information about an input device.
///
/// This struct provides a platform-agnostic representation of keyboard devices
/// that can be used for device discovery and selection.
#[derive(Debug, Clone, Serialize)]
pub struct DeviceInfo {
    /// Path to the device (e.g., `/dev/input/event0` on Linux).
    pub path: PathBuf,
    /// Human-readable device name.
    pub name: String,
    /// USB vendor ID (0 if not available).
    pub vendor_id: u16,
    /// USB product ID (0 if not available).
    pub product_id: u16,
    /// Whether this device is detected as a keyboard.
    pub is_keyboard: bool,
}

impl DeviceInfo {
    /// Create a new DeviceInfo with all fields specified.
    pub fn new(
        path: PathBuf,
        name: String,
        vendor_id: u16,
        product_id: u16,
        is_keyboard: bool,
    ) -> Self {
        Self {
            path,
            name,
            vendor_id,
            product_id,
            is_keyboard,
        }
    }
}

impl fmt::Display for DeviceInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({:04x}:{:04x}) at {}",
            self.name,
            self.vendor_id,
            self.product_id,
            self.path.display()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_info_display() {
        let info = DeviceInfo::new(
            PathBuf::from("/dev/input/event0"),
            "Test Keyboard".to_string(),
            0x1234,
            0x5678,
            true,
        );
        assert_eq!(
            info.to_string(),
            "Test Keyboard (1234:5678) at /dev/input/event0"
        );
    }

    #[test]
    fn device_info_display_zero_ids() {
        let info = DeviceInfo::new(
            PathBuf::from("/dev/input/event1"),
            "Generic Device".to_string(),
            0,
            0,
            false,
        );
        assert_eq!(
            info.to_string(),
            "Generic Device (0000:0000) at /dev/input/event1"
        );
    }

    #[test]
    fn device_info_serializes_to_json() {
        let info = DeviceInfo::new(
            PathBuf::from("/dev/input/event0"),
            "Test Keyboard".to_string(),
            0x1234,
            0x5678,
            true,
        );
        let json = serde_json::to_string(&info).expect("serialization failed");
        assert!(json.contains("\"name\":\"Test Keyboard\""));
        assert!(json.contains("\"vendor_id\":4660")); // 0x1234 = 4660
        assert!(json.contains("\"product_id\":22136")); // 0x5678 = 22136
        assert!(json.contains("\"is_keyboard\":true"));
    }

    #[test]
    fn device_info_clone() {
        let info = DeviceInfo::new(
            PathBuf::from("/dev/input/event0"),
            "Test Keyboard".to_string(),
            0x1234,
            0x5678,
            true,
        );
        let cloned = info.clone();
        assert_eq!(cloned.path, info.path);
        assert_eq!(cloned.name, info.name);
        assert_eq!(cloned.vendor_id, info.vendor_id);
        assert_eq!(cloned.product_id, info.product_id);
        assert_eq!(cloned.is_keyboard, info.is_keyboard);
    }
}
