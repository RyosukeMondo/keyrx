use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::{Hash, Hasher};

/// Unique identifier for a physical device instance.
///
/// Unlike the old DeviceId which only used VID:PID, DeviceIdentity includes
/// serial number to distinguish between multiple identical devices.
/// This enables per-device configuration and mapping profiles.
///
/// Note: Equality and hashing are based only on VID:PID:Serial,
/// ignoring user_label to maintain consistent HashMap behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceIdentity {
    /// USB Vendor ID (e.g., 0x046D for Logitech)
    pub vendor_id: u16,
    /// USB Product ID (e.g., 0xC52B for specific device model)
    pub product_id: u16,
    /// Device serial number extracted from USB descriptors or generated
    /// Falls back to synthetic ID based on physical port if unavailable
    pub serial_number: String,
    /// Optional user-assigned label for easier identification in UI
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_label: Option<String>,
}

impl DeviceIdentity {
    /// Create a new DeviceIdentity.
    pub fn new(vendor_id: u16, product_id: u16, serial_number: String) -> Self {
        Self {
            vendor_id,
            product_id,
            serial_number,
            user_label: None,
        }
    }

    /// Create with a user label.
    pub fn with_label(
        vendor_id: u16,
        product_id: u16,
        serial_number: String,
        user_label: String,
    ) -> Self {
        Self {
            vendor_id,
            product_id,
            serial_number,
            user_label: Some(user_label),
        }
    }

    /// Convert to a string key suitable for HashMap keys and file storage.
    ///
    /// Format: `{vendor_id:04x}:{product_id:04x}:{serial_number}`
    /// Example: `046d:c52b:ABC123456`
    pub fn to_key(&self) -> String {
        format!(
            "{:04x}:{:04x}:{}",
            self.vendor_id, self.product_id, self.serial_number
        )
    }

    /// Parse a DeviceIdentity from a key string.
    ///
    /// Expected format: `{vendor_id:04x}:{product_id:04x}:{serial_number}`
    ///
    /// # Errors
    /// Returns an error if the format is invalid or hex parsing fails.
    pub fn from_key(key: &str) -> Result<Self, String> {
        let parts: Vec<&str> = key.split(':').collect();
        if parts.len() < 3 {
            return Err(format!(
                "Invalid key format: expected 'VID:PID:SERIAL', got '{}'",
                key
            ));
        }

        let vendor_id = u16::from_str_radix(parts[0], 16)
            .map_err(|e| format!("Invalid vendor_id '{}': {}", parts[0], e))?;

        let product_id = u16::from_str_radix(parts[1], 16)
            .map_err(|e| format!("Invalid product_id '{}': {}", parts[1], e))?;

        // Serial number is everything after the second colon
        // (in case serial contains colons)
        let serial_number = parts[2..].join(":");

        Ok(Self::new(vendor_id, product_id, serial_number))
    }

    /// Get a display name for the device.
    /// Uses user label if set, otherwise falls back to VID:PID.
    pub fn display_name(&self) -> String {
        self.user_label
            .as_ref()
            .cloned()
            .unwrap_or_else(|| format!("{:04x}:{:04x}", self.vendor_id, self.product_id))
    }
}

impl Hash for DeviceIdentity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash based on VID:PID:Serial only, ignoring user_label
        // This ensures consistent hashing even when label changes
        self.vendor_id.hash(state);
        self.product_id.hash(state);
        self.serial_number.hash(state);
    }
}

impl PartialEq for DeviceIdentity {
    fn eq(&self, other: &Self) -> bool {
        // Equality based on VID:PID:Serial only, ignoring user_label
        // This ensures consistent equality even when label changes
        self.vendor_id == other.vendor_id
            && self.product_id == other.product_id
            && self.serial_number == other.serial_number
    }
}

impl Eq for DeviceIdentity {}

impl fmt::Display for DeviceIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(label) = &self.user_label {
            write!(
                f,
                "{} ({:04x}:{:04x}:{})",
                label, self.vendor_id, self.product_id, self.serial_number
            )
        } else {
            write!(
                f,
                "{:04x}:{:04x}:{}",
                self.vendor_id, self.product_id, self.serial_number
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_to_key_format() {
        let id = DeviceIdentity::new(0x046D, 0xC52B, "ABC123".to_string());
        assert_eq!(id.to_key(), "046d:c52b:ABC123");
    }

    #[test]
    fn test_from_key_valid() {
        let key = "046d:c52b:ABC123";
        let id = DeviceIdentity::from_key(key).unwrap();
        assert_eq!(id.vendor_id, 0x046D);
        assert_eq!(id.product_id, 0xC52B);
        assert_eq!(id.serial_number, "ABC123");
    }

    #[test]
    fn test_from_key_with_colon_in_serial() {
        let key = "046d:c52b:ABC:123:XYZ";
        let id = DeviceIdentity::from_key(key).unwrap();
        assert_eq!(id.serial_number, "ABC:123:XYZ");
    }

    #[test]
    fn test_from_key_invalid_format() {
        assert!(DeviceIdentity::from_key("046d:c52b").is_err());
        assert!(DeviceIdentity::from_key("invalid").is_err());
    }

    #[test]
    fn test_from_key_invalid_hex() {
        assert!(DeviceIdentity::from_key("GGGG:c52b:ABC123").is_err());
        assert!(DeviceIdentity::from_key("046d:ZZZZ:ABC123").is_err());
    }

    #[test]
    fn test_roundtrip_key_conversion() {
        let original = DeviceIdentity::new(0x1234, 0xABCD, "SERIAL123".to_string());
        let key = original.to_key();
        let restored = DeviceIdentity::from_key(&key).unwrap();
        assert_eq!(original.vendor_id, restored.vendor_id);
        assert_eq!(original.product_id, restored.product_id);
        assert_eq!(original.serial_number, restored.serial_number);
    }

    #[test]
    fn test_hash_implementation() {
        let id1 = DeviceIdentity::new(0x046D, 0xC52B, "ABC123".to_string());
        let id2 = DeviceIdentity::new(0x046D, 0xC52B, "ABC123".to_string());
        let id3 = DeviceIdentity::new(0x046D, 0xC52B, "XYZ789".to_string());

        // Same devices should have same hash
        let mut map = HashMap::new();
        map.insert(id1.clone(), "device1");
        assert_eq!(map.get(&id2), Some(&"device1"));

        // Different serial should have different hash
        map.insert(id3.clone(), "device3");
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_hash_ignores_user_label() {
        let id1 = DeviceIdentity::with_label(
            0x046D,
            0xC52B,
            "ABC123".to_string(),
            "My Device".to_string(),
        );
        let id2 = DeviceIdentity::with_label(
            0x046D,
            0xC52B,
            "ABC123".to_string(),
            "Different Label".to_string(),
        );

        // Same device with different labels should hash the same
        let mut map = HashMap::new();
        map.insert(id1.clone(), "value1");
        assert_eq!(map.get(&id2), Some(&"value1"));
    }

    #[test]
    fn test_eq_implementation() {
        let id1 = DeviceIdentity::new(0x046D, 0xC52B, "ABC123".to_string());
        let id2 = DeviceIdentity::new(0x046D, 0xC52B, "ABC123".to_string());
        let id3 = DeviceIdentity::new(0x046D, 0xC52B, "XYZ789".to_string());

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_display_without_label() {
        let id = DeviceIdentity::new(0x046D, 0xC52B, "ABC123".to_string());
        assert_eq!(id.to_string(), "046d:c52b:ABC123");
    }

    #[test]
    fn test_display_with_label() {
        let id = DeviceIdentity::with_label(
            0x046D,
            0xC52B,
            "ABC123".to_string(),
            "My Keyboard".to_string(),
        );
        assert_eq!(id.to_string(), "My Keyboard (046d:c52b:ABC123)");
    }

    #[test]
    fn test_display_name() {
        let id1 = DeviceIdentity::new(0x046D, 0xC52B, "ABC123".to_string());
        assert_eq!(id1.display_name(), "046d:c52b");

        let id2 = DeviceIdentity::with_label(
            0x046D,
            0xC52B,
            "ABC123".to_string(),
            "My Device".to_string(),
        );
        assert_eq!(id2.display_name(), "My Device");
    }

    #[test]
    fn test_serialization() {
        let id = DeviceIdentity::with_label(
            0x046D,
            0xC52B,
            "ABC123".to_string(),
            "Test Device".to_string(),
        );

        let json = serde_json::to_string(&id).unwrap();
        let deserialized: DeviceIdentity = serde_json::from_str(&json).unwrap();

        assert_eq!(id, deserialized);
        assert_eq!(id.user_label, deserialized.user_label);
    }

    #[test]
    fn test_serialization_without_label() {
        let id = DeviceIdentity::new(0x046D, 0xC52B, "ABC123".to_string());

        let json = serde_json::to_string(&id).unwrap();
        // user_label should be omitted when None
        assert!(!json.contains("user_label"));

        let deserialized: DeviceIdentity = serde_json::from_str(&json).unwrap();
        assert_eq!(id, deserialized);
        assert_eq!(id.user_label, None);
    }
}
