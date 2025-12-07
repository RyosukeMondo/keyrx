use crate::drivers::{list_keyboards, DeviceInfo};
use crate::errors::KeyrxError;
use serde::{Deserialize, Serialize};

/// High-level classification of a keyboard device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceClass {
    MechanicalKeyboard,
    MembraneKeyboard,
    LaptopKeyboard,
    VirtualKeyboard,
    Unknown,
}

/// Hardware fingerprint extracted from a device enumeration result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HardwareInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub vendor_name: Option<String>,
    pub product_name: Option<String>,
    pub device_class: DeviceClass,
}

impl HardwareInfo {
    /// Build a HardwareInfo record from a DeviceInfo.
    pub fn from_device_info(device: &DeviceInfo) -> Self {
        let device_class = infer_device_class(device);

        Self {
            vendor_id: device.vendor_id(),
            product_id: device.product_id(),
            vendor_name: None,
            product_name: Some(device.name().to_string()).filter(|s| !s.is_empty()),
            device_class,
        }
    }
}

/// Detects connected devices and produces coarse-grained hardware metadata.
pub struct HardwareDetector;

impl HardwareDetector {
    /// Extract hardware metadata for a single device.
    pub fn detect(device: &DeviceInfo) -> HardwareInfo {
        HardwareInfo::from_device_info(device)
    }

    /// Enumerate all keyboards and produce hardware metadata for each.
    ///
    /// Uses the platform-specific driver list to gather devices.
    pub fn detect_all() -> Result<Vec<HardwareInfo>, KeyrxError> {
        Self::detect_with(list_keyboards)
    }

    /// Enumerate devices using a custom listing function (useful for tests).
    pub fn detect_with(
        list_devices: impl Fn() -> Result<Vec<DeviceInfo>, KeyrxError>,
    ) -> Result<Vec<HardwareInfo>, KeyrxError> {
        let devices = list_devices()?;
        Ok(devices.iter().map(Self::detect).collect())
    }
}

fn infer_device_class(device: &DeviceInfo) -> DeviceClass {
    let name = device.name().to_lowercase();
    let vendor_id = device.vendor_id();
    let product_id = device.product_id();

    if name.contains("virtual") || name.contains("simulated") || name.contains("vkeyboard") {
        return DeviceClass::VirtualKeyboard;
    }

    if vendor_id == 0 && product_id == 0 {
        return DeviceClass::VirtualKeyboard;
    }

    if name.contains("laptop")
        || name.contains("notebook")
        || name.contains("thinkpad")
        || name.contains("macbook")
        || name.contains("internal keyboard")
    {
        return DeviceClass::LaptopKeyboard;
    }

    if name.contains("mechanical")
        || name.contains("cherry")
        || name.contains("gateron")
        || name.contains("kailh")
        || name.contains("optical switch")
        || name.contains("hall effect")
    {
        return DeviceClass::MechanicalKeyboard;
    }

    if name.contains("membrane") || name.contains("rubber dome") {
        return DeviceClass::MembraneKeyboard;
    }

    DeviceClass::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn device(name: &str, vendor_id: u16, product_id: u16) -> DeviceInfo {
        DeviceInfo::new(
            PathBuf::from("/dev/input/event0"),
            name.to_string(),
            vendor_id,
            product_id,
            true,
        )
    }

    #[test]
    fn infers_mechanical_from_keywords() {
        let dev = device("Gateron Optical Mechanical Keyboard", 0x1234, 0x5678);
        let info = HardwareDetector::detect(&dev);
        assert_eq!(info.device_class, DeviceClass::MechanicalKeyboard);
    }

    #[test]
    fn infers_membrane_from_keywords() {
        let dev = device("Office Keyboard (Rubber Dome)", 0x1111, 0x2222);
        let info = HardwareDetector::detect(&dev);
        assert_eq!(info.device_class, DeviceClass::MembraneKeyboard);
    }

    #[test]
    fn infers_laptop_from_keywords() {
        let dev = device("ThinkPad Laptop Keyboard", 0x3333, 0x4444);
        let info = HardwareDetector::detect(&dev);
        assert_eq!(info.device_class, DeviceClass::LaptopKeyboard);
    }

    #[test]
    fn infers_virtual_from_zero_ids() {
        let dev = device("Virtual Keyboard", 0, 0);
        let info = HardwareDetector::detect(&dev);
        assert_eq!(info.device_class, DeviceClass::VirtualKeyboard);
    }

    #[test]
    fn detect_with_allows_custom_listing() {
        let devices = vec![
            device("Custom Keyboard A", 0xAAAA, 0x0001),
            device("Custom Keyboard B", 0xBBBB, 0x0002),
        ];

        let result = HardwareDetector::detect_with(|| Ok(devices.clone())).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].vendor_id, 0xAAAA);
        assert_eq!(result[1].product_id, 0x0002);
    }
}
