//! Common types shared across platform-specific drivers.

use serde::Serialize;
use std::any::Any;
use std::fmt;
use std::path::PathBuf;

/// Extract a human-readable message from a panic payload.
///
/// This function handles the common case of extracting a message from
/// `std::panic::catch_unwind` results or panic hooks. It supports both
/// `&str` and `String` panic payloads, returning "Unknown panic" for
/// other types.
///
/// # Arguments
///
/// * `panic_info` - The panic payload from `catch_unwind` or a panic hook.
///
/// # Returns
///
/// A `String` containing the panic message.
pub fn extract_panic_message(panic_info: &(dyn Any + Send)) -> String {
    if let Some(s) = panic_info.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = panic_info.downcast_ref::<String>() {
        s.clone()
    } else {
        "Unknown panic".to_string()
    }
}

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

    /// Get the device name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the device path.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Get the vendor ID.
    pub fn vendor_id(&self) -> u16 {
        self.vendor_id
    }

    /// Get the product ID.
    pub fn product_id(&self) -> u16 {
        self.product_id
    }

    /// Check if this device is a keyboard.
    pub fn is_keyboard(&self) -> bool {
        self.is_keyboard
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

    #[test]
    fn device_info_accessors() {
        let info = DeviceInfo::new(
            PathBuf::from("/dev/input/event5"),
            "My Keyboard".to_string(),
            0xABCD,
            0x1234,
            true,
        );
        assert_eq!(info.name(), "My Keyboard");
        assert_eq!(info.path(), &PathBuf::from("/dev/input/event5"));
        assert_eq!(info.vendor_id(), 0xABCD);
        assert_eq!(info.product_id(), 0x1234);
        assert!(info.is_keyboard());
    }

    #[test]
    fn device_info_accessors_non_keyboard() {
        let info = DeviceInfo::new(
            PathBuf::from("/dev/input/event10"),
            "Mouse".to_string(),
            0x0001,
            0x0002,
            false,
        );
        assert_eq!(info.name(), "Mouse");
        assert!(!info.is_keyboard());
    }

    #[test]
    fn device_info_debug_output() {
        let info = DeviceInfo::new(
            PathBuf::from("/dev/input/event0"),
            "Test".to_string(),
            0,
            0,
            true,
        );
        let debug = format!("{:?}", info);
        assert!(debug.contains("DeviceInfo"));
        assert!(debug.contains("Test"));
    }

    #[test]
    fn extract_panic_message_str() {
        let payload: Box<dyn std::any::Any + Send> = Box::new("test panic message");
        assert_eq!(extract_panic_message(&*payload), "test panic message");
    }

    #[test]
    fn extract_panic_message_string() {
        let payload: Box<dyn std::any::Any + Send> = Box::new(String::from("owned panic"));
        assert_eq!(extract_panic_message(&*payload), "owned panic");
    }

    #[test]
    fn extract_panic_message_unknown() {
        let payload: Box<dyn std::any::Any + Send> = Box::new(42i32);
        assert_eq!(extract_panic_message(&*payload), "Unknown panic");
    }

    #[test]
    fn extract_panic_message_empty_str() {
        let payload: Box<dyn std::any::Any + Send> = Box::new("");
        assert_eq!(extract_panic_message(&*payload), "");
    }
}
