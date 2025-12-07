use crate::drivers::DeviceInfo;
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

/// Heuristic classifier for determining device class based on identifiers and names.
pub struct DeviceClassifier;

impl DeviceClassifier {
    /// Classify a device using vendor/product identifiers and common naming hints.
    pub fn classify(device: &DeviceInfo) -> DeviceClass {
        let name = device.name().to_lowercase();
        let vid = device.vendor_id();
        let pid = device.product_id();

        if is_virtual(&name, vid, pid) {
            return DeviceClass::VirtualKeyboard;
        }

        if is_laptop(&name) {
            return DeviceClass::LaptopKeyboard;
        }

        if is_mechanical(&name, vid, pid) {
            return DeviceClass::MechanicalKeyboard;
        }

        if is_membrane(&name) {
            return DeviceClass::MembraneKeyboard;
        }

        DeviceClass::Unknown
    }
}

const MECHANICAL_KEYWORDS: &[&str] = &[
    "mechanical",
    "gateron",
    "kailh",
    "cherry",
    "optical switch",
    "hall effect",
    "hot-swap",
    "hotswap",
    "linear switch",
    "tactile switch",
    "buckling spring",
];

const MEMBRANE_KEYWORDS: &[&str] = &[
    "membrane",
    "rubber dome",
    "office keyboard",
    "quiet touch",
    "soft touch",
    "scissor",
];

const LAPTOP_KEYWORDS: &[&str] = &[
    "laptop",
    "notebook",
    "thinkpad",
    "macbook",
    "internal keyboard",
];

const VIRTUAL_KEYWORDS: &[&str] = &["virtual", "simulated", "vkeyboard", "software keyboard"];

// A handful of common mechanical vendor IDs to bias detection when names are generic.
const MECHANICAL_VENDORS: &[u16] = &[
    0x1532, // Razer
    0x1b1c, // Corsair
    0x1038, // SteelSeries
    0x048d, // Ducky/Holtek OEM boards
    0x3434, // Glorious
];

fn is_virtual(name: &str, vid: u16, pid: u16) -> bool {
    vid == 0 && pid == 0 || contains_any(name, VIRTUAL_KEYWORDS)
}

fn is_laptop(name: &str) -> bool {
    contains_any(name, LAPTOP_KEYWORDS)
}

fn is_mechanical(name: &str, vid: u16, _pid: u16) -> bool {
    contains_any(name, MECHANICAL_KEYWORDS) || MECHANICAL_VENDORS.contains(&vid)
}

fn is_membrane(name: &str) -> bool {
    contains_any(name, MEMBRANE_KEYWORDS)
}

fn contains_any(name: &str, keywords: &[&str]) -> bool {
    keywords.iter().any(|kw| name.contains(kw))
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
    fn classifies_virtual_from_zero_ids() {
        let info = device("Virtual Keyboard", 0, 0);
        assert_eq!(
            DeviceClassifier::classify(&info),
            DeviceClass::VirtualKeyboard
        );
    }

    #[test]
    fn classifies_virtual_from_keywords() {
        let info = device("Simulated VKeyboard", 0x1111, 0x2222);
        assert_eq!(
            DeviceClassifier::classify(&info),
            DeviceClass::VirtualKeyboard
        );
    }

    #[test]
    fn classifies_laptop() {
        let info = device("ThinkPad Laptop Keyboard", 0x3333, 0x4444);
        assert_eq!(
            DeviceClassifier::classify(&info),
            DeviceClass::LaptopKeyboard
        );
    }

    #[test]
    fn classifies_mechanical_from_keywords() {
        let info = device("Gateron Optical Mechanical Keyboard", 0x1234, 0x5678);
        assert_eq!(
            DeviceClassifier::classify(&info),
            DeviceClass::MechanicalKeyboard
        );
    }

    #[test]
    fn classifies_mechanical_from_vendor_bias() {
        let info = device("Gaming Keyboard", 0x1b1c, 0x0001);
        assert_eq!(
            DeviceClassifier::classify(&info),
            DeviceClass::MechanicalKeyboard
        );
    }

    #[test]
    fn classifies_membrane_from_keywords() {
        let info = device("Office Keyboard (Rubber Dome)", 0x9999, 0x0001);
        assert_eq!(
            DeviceClassifier::classify(&info),
            DeviceClass::MembraneKeyboard
        );
    }

    #[test]
    fn falls_back_to_unknown() {
        let info = device("USB Input Device", 0xAAAA, 0xBBBB);
        assert_eq!(DeviceClassifier::classify(&info), DeviceClass::Unknown);
    }
}
