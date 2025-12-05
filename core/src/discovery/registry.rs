//! Device profile registry with on-disk persistence.
//!
//! This module loads per-device profiles, falls back to defaults when missing or
//! invalid, and exposes a `discover_needed` signal so callers can prompt users
//! to run discovery without blocking engine startup.

use crate::discovery::storage::{
    default_profile_for, profile_path, read_profile, write_profile, StorageError,
};
use crate::discovery::types::{DeviceId, DeviceProfile};
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;

/// Reason why discovery should be prompted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscoveryReason {
    /// No profile exists yet for this device.
    MissingProfile,
    /// The on-disk profile failed to parse.
    ParseError,
    /// The on-disk profile failed schema validation.
    ValidationError,
    /// The on-disk profile schema is outdated.
    SchemaMismatch { expected: u8, found: u8 },
    /// Other IO errors (permission denied, etc.).
    IoError(String),
}

/// Status of a registry lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryStatus {
    /// Profile is ready for use.
    Ready,
    /// Discovery should be triggered before trusting the profile.
    NeedsDiscovery(DiscoveryReason),
}

/// Result of loading a device profile.
#[derive(Debug, Clone)]
pub struct RegistryEntry {
    pub device_id: DeviceId,
    pub profile: DeviceProfile,
    pub path: PathBuf,
    pub status: RegistryStatus,
}

impl RegistryEntry {
    /// True when discovery should be offered to the user.
    pub fn discover_needed(&self) -> bool {
        matches!(self.status, RegistryStatus::NeedsDiscovery(_))
    }
}

/// In-memory registry with lazy loading and on-disk persistence.
#[derive(Debug, Default)]
pub struct DeviceRegistry {
    cache: HashMap<DeviceId, DeviceProfile>,
}

impl DeviceRegistry {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Load a profile from cache or disk, falling back to a default profile on error.
    ///
    /// The returned entry always contains a usable profile, but `discover_needed`
    /// will be true when the profile was missing, corrupt, or schema-mismatched.
    pub fn load_or_default(&mut self, device_id: DeviceId) -> RegistryEntry {
        if let Some(profile) = self.cache.get(&device_id) {
            return RegistryEntry {
                device_id,
                profile: profile.clone(),
                path: profile_path(device_id),
                status: RegistryStatus::Ready,
            };
        }

        let path = profile_path(device_id);
        match read_profile(device_id) {
            Ok(profile) => {
                self.cache.insert(device_id, profile.clone());
                RegistryEntry {
                    device_id,
                    profile,
                    path,
                    status: RegistryStatus::Ready,
                }
            }
            Err(err) => {
                let status = RegistryStatus::NeedsDiscovery(DiscoveryReason::from(&err));
                RegistryEntry {
                    device_id,
                    profile: default_profile_for(device_id),
                    path,
                    status,
                }
            }
        }
    }

    /// Persist a profile and update the in-memory cache.
    pub fn save_profile(&mut self, profile: DeviceProfile) -> Result<PathBuf, StorageError> {
        let path = write_profile(&profile)?;
        let device_id = DeviceId::new(profile.vendor_id, profile.product_id);
        self.cache.insert(device_id, profile);
        Ok(path)
    }
}

impl From<&StorageError> for DiscoveryReason {
    fn from(err: &StorageError) -> Self {
        match err {
            StorageError::Io { source, .. } => {
                if source.kind() == io::ErrorKind::NotFound {
                    DiscoveryReason::MissingProfile
                } else {
                    DiscoveryReason::IoError(source.to_string())
                }
            }
            StorageError::Parse { .. } => DiscoveryReason::ParseError,
            StorageError::SchemaMismatch {
                expected, found, ..
            } => DiscoveryReason::SchemaMismatch {
                expected: *expected,
                found: *found,
            },
            StorageError::Validation { .. } => DiscoveryReason::ValidationError,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::test_utils::config_env_lock;
    use crate::discovery::types::SCHEMA_VERSION;
    use serial_test::serial;
    use std::env;
    use std::fs;
    use tempfile::tempdir;

    fn with_temp_config<F: FnOnce()>(f: F) {
        let _guard = config_env_lock().lock().unwrap();
        let temp = tempdir().unwrap();
        let prev_xdg = env::var("XDG_CONFIG_HOME").ok();
        let prev_home = env::var("HOME").ok();

        env::set_var("XDG_CONFIG_HOME", temp.path());
        env::remove_var("HOME");
        f();

        if let Some(xdg) = prev_xdg {
            env::set_var("XDG_CONFIG_HOME", xdg);
        } else {
            env::remove_var("XDG_CONFIG_HOME");
        }
        if let Some(home) = prev_home {
            env::set_var("HOME", home);
        } else {
            env::remove_var("HOME");
        }
    }

    #[test]
    #[serial]
    fn loads_existing_profile_as_ready() {
        with_temp_config(|| {
            let mut registry = DeviceRegistry::new();
            let device_id = DeviceId::new(0x1234, 0x5678);
            let mut profile = default_profile_for(device_id);
            profile.rows = 1;
            registry.save_profile(profile.clone()).unwrap();

            let entry = registry.load_or_default(device_id);
            assert!(!entry.discover_needed());
            assert_eq!(entry.status, RegistryStatus::Ready);
            assert_eq!(entry.profile.vendor_id, profile.vendor_id);
            assert_eq!(entry.path, profile_path(device_id));
        });
    }

    #[test]
    #[serial]
    fn missing_profile_sets_discover_needed() {
        with_temp_config(|| {
            let mut registry = DeviceRegistry::new();
            let device_id = DeviceId::new(0x1111, 0x2222);
            let entry = registry.load_or_default(device_id);

            assert!(entry.discover_needed());
            assert_eq!(
                entry.status,
                RegistryStatus::NeedsDiscovery(DiscoveryReason::MissingProfile)
            );
            assert_eq!(entry.profile.vendor_id, device_id.vendor_id);
            assert_eq!(entry.profile.product_id, device_id.product_id);
        });
    }

    #[test]
    #[serial]
    fn schema_mismatch_signals_discovery() {
        with_temp_config(|| {
            let mut registry = DeviceRegistry::new();
            let device_id = DeviceId::new(0x0A0A, 0x0B0B);
            let path = profile_path(device_id);
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir).unwrap();
            }
            let bad_profile = format!(
                r#"{{
                    "schema_version": {},
                    "vendor_id": {},
                    "product_id": {},
                    "discovered_at": "2025-01-01T00:00:00Z",
                    "rows": 0,
                    "cols_per_row": [],
                    "keymap": {{}},
                    "aliases": {{}},
                    "source": "Default"
                }}"#,
                SCHEMA_VERSION.saturating_sub(1),
                device_id.vendor_id,
                device_id.product_id
            );
            std::fs::write(&path, bad_profile).unwrap();

            let entry = registry.load_or_default(device_id);
            assert!(entry.discover_needed());
            assert!(matches!(
                entry.status,
                RegistryStatus::NeedsDiscovery(DiscoveryReason::SchemaMismatch { .. })
            ));
        });
    }

    #[test]
    #[serial]
    fn validation_error_signals_discovery() {
        with_temp_config(|| {
            let mut registry = DeviceRegistry::new();
            let device_id = DeviceId::new(0xDEAD, 0xBEEF);
            let path = profile_path(device_id);
            if let Some(dir) = path.parent() {
                fs::create_dir_all(dir).unwrap();
            }
            let invalid_profile = r#"
                {
                    "schema_version": 1,
                    "vendor_id": 57005,
                    "product_id": 48879,
                    "discovered_at": "2025-01-01T00:00:00Z",
                    "rows": "invalid",
                    "cols_per_row": [1],
                    "keymap": {},
                    "aliases": {},
                    "source": "Default"
                }
            "#;
            fs::write(&path, invalid_profile).unwrap();

            let entry = registry.load_or_default(device_id);

            assert!(entry.discover_needed());
            assert_eq!(
                entry.status,
                RegistryStatus::NeedsDiscovery(DiscoveryReason::ValidationError)
            );
        });
    }
}
