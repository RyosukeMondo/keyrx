use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::path::PathBuf;

/// Schema version for serialized device profiles.
pub const SCHEMA_VERSION: u8 = 1;

/// Default schema version for serde deserialization.
pub fn default_schema_version() -> u8 {
    SCHEMA_VERSION
}

/// Identifier for a physical device, typically USB vendor/product IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeviceId {
    pub vendor_id: u16,
    pub product_id: u16,
}

impl DeviceId {
    /// Create a new DeviceId.
    pub fn new(vendor_id: u16, product_id: u16) -> Self {
        Self {
            vendor_id,
            product_id,
        }
    }

    /// Filename-safe identifier in lowercase hex (`vvvv_pppp.json`).
    pub fn to_filename(&self) -> String {
        format!("{:04x}_{:04x}.json", self.vendor_id, self.product_id)
    }
}

impl fmt::Display for DeviceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04x}:{:04x}", self.vendor_id, self.product_id)
    }
}

/// Origin of the profile (discovered on device, default, or migrated).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ProfileSource {
    Discovered,
    #[default]
    Default,
    Migrated,
}

/// Physical key metadata captured during discovery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhysicalKey {
    pub scan_code: u16,
    pub row: u8,
    pub col: u8,
    #[serde(default)]
    pub alias: Option<String>,
}

impl PhysicalKey {
    pub fn new(scan_code: u16, row: u8, col: u8) -> Self {
        Self {
            scan_code,
            row,
            col,
            alias: None,
        }
    }
}

/// Per-device profile describing the physical layout and aliases.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeviceProfile {
    #[serde(default = "default_schema_version")]
    pub schema_version: u8,
    pub vendor_id: u16,
    pub product_id: u16,
    #[serde(default)]
    pub name: Option<String>,
    pub discovered_at: chrono::DateTime<chrono::Utc>,
    pub rows: u8,
    pub cols_per_row: Vec<u8>,
    /// scan_code -> PhysicalKey
    #[serde(default)]
    pub keymap: HashMap<u16, PhysicalKey>,
    /// alias -> scan_code
    #[serde(default)]
    pub aliases: HashMap<String, u16>,
    #[serde(default)]
    pub source: ProfileSource,
}

/// Resolve the device profiles directory.
///
/// Preference order:
/// 1. `$XDG_CONFIG_HOME/keyrx/devices`
/// 2. `$HOME/.config/keyrx/devices`
/// 3. `.config/keyrx/devices` relative to CWD (last-resort fallback)
pub fn device_profiles_dir() -> PathBuf {
    if let Ok(xdg) = env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(xdg).join("keyrx").join("devices");
    }

    if let Ok(home) = env::var("HOME") {
        return PathBuf::from(home)
            .join(".config")
            .join("keyrx")
            .join("devices");
    }

    PathBuf::from(".")
        .join(".config")
        .join("keyrx")
        .join("devices")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use serial_test::serial;
    use std::env;
    use tempfile::tempdir;

    #[test]
    fn device_id_display_and_filename() {
        let id = DeviceId::new(0x1234, 0xABCD);
        assert_eq!(id.to_string(), "1234:abcd");
        assert_eq!(id.to_filename(), "1234_abcd.json");
    }

    #[test]
    fn profile_defaults_apply_on_deserialize() {
        let json_value = json!({
            "vendor_id": 4660,
            "product_id": 43981,
            "discovered_at": "2025-01-01T00:00:00Z",
            "rows": 1,
            "cols_per_row": [1],
            "keymap": {}
        });
        let profile: DeviceProfile = serde_json::from_value(json_value).unwrap();
        assert_eq!(profile.schema_version, SCHEMA_VERSION);
        assert_eq!(profile.source, ProfileSource::Default);
        assert!(profile.aliases.is_empty());
        assert!(profile.keymap.is_empty());
        assert!(profile.name.is_none());
    }

    #[test]
    #[serial]
    fn device_profiles_dir_prefers_xdg_config_home() {
        let temp = tempdir().unwrap();
        let prev_xdg = env::var("XDG_CONFIG_HOME").ok();
        let prev_home = env::var("HOME").ok();

        env::set_var("XDG_CONFIG_HOME", temp.path());
        env::remove_var("HOME");

        let path = device_profiles_dir();
        assert!(path.starts_with(temp.path()));
        assert!(path.ends_with(PathBuf::from("keyrx").join("devices")));

        match prev_xdg {
            Some(val) => env::set_var("XDG_CONFIG_HOME", val),
            None => env::remove_var("XDG_CONFIG_HOME"),
        }
        if let Some(home) = prev_home {
            env::set_var("HOME", home);
        }
    }

    #[test]
    #[serial]
    fn device_profiles_dir_falls_back_to_home() {
        let temp = tempdir().unwrap();
        let prev_xdg = env::var("XDG_CONFIG_HOME").ok();
        let prev_home = env::var("HOME").ok();

        env::remove_var("XDG_CONFIG_HOME");
        env::set_var("HOME", temp.path());

        let path = device_profiles_dir();
        assert!(path.starts_with(temp.path()));
        assert!(path.ends_with(PathBuf::from(".config").join("keyrx").join("devices")));

        if let Some(xdg) = prev_xdg {
            env::set_var("XDG_CONFIG_HOME", xdg);
        }
        match prev_home {
            Some(val) => env::set_var("HOME", val),
            None => env::remove_var("HOME"),
        }
    }
}
