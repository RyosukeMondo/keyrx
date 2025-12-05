//! Device bindings persistence for revolutionary mapping system.
//!
//! This module manages the persistent storage of device-to-profile assignments
//! and user labels. Bindings are stored in a JSON file with atomic writes to
//! prevent corruption.

use crate::config::config_dir;
use crate::identity::DeviceIdentity;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

use super::ProfileId;

/// Binding information for a single device
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeviceBinding {
    /// Optional profile assignment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_id: Option<ProfileId>,

    /// Whether remapping is enabled for this device
    #[serde(default = "default_remap_enabled")]
    pub remap_enabled: bool,
}

fn default_remap_enabled() -> bool {
    true
}

impl DeviceBinding {
    /// Create a new device binding with no profile assigned
    pub fn new() -> Self {
        Self {
            profile_id: None,
            remap_enabled: true,
        }
    }

    /// Create a binding with a specific profile
    pub fn with_profile(profile_id: ProfileId) -> Self {
        Self {
            profile_id: Some(profile_id),
            remap_enabled: true,
        }
    }

    /// Create a binding with remap disabled
    pub fn disabled() -> Self {
        Self {
            profile_id: None,
            remap_enabled: false,
        }
    }
}

impl Default for DeviceBinding {
    fn default() -> Self {
        Self::new()
    }
}

/// Error types for device bindings operations
#[derive(Debug, Error)]
pub enum DeviceBindingsError {
    /// I/O error during file operations
    #[error("I/O error for path {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Serialization/deserialization error
    #[error("Serialization error for path {path}: {source}")]
    Serialization {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    /// Corruption detected in bindings file
    #[error("Corrupt bindings file at {path}: {reason}")]
    Corruption { path: PathBuf, reason: String },
}

/// Persistent storage for device-to-profile bindings
///
/// Manages the mapping of device identities to their profile assignments
/// and configuration. Bindings are stored in a JSON file with atomic writes.
pub struct DeviceBindings {
    /// In-memory map of device bindings
    bindings: HashMap<DeviceIdentity, DeviceBinding>,

    /// Path to the bindings file
    file_path: PathBuf,
}

impl DeviceBindings {
    /// File name for device bindings storage
    pub const BINDINGS_FILE: &'static str = "device_bindings.json";

    /// Create a new DeviceBindings instance with default file path
    pub fn new() -> Self {
        Self::with_path(Self::default_path())
    }

    /// Create a DeviceBindings instance with a custom file path
    pub fn with_path(file_path: PathBuf) -> Self {
        Self {
            bindings: HashMap::new(),
            file_path,
        }
    }

    /// Get the default path for device bindings
    pub fn default_path() -> PathBuf {
        config_dir().join(Self::BINDINGS_FILE)
    }

    /// Load bindings from disk
    ///
    /// If the file doesn't exist, returns an empty bindings set.
    /// If the file is corrupted, creates a backup and returns empty bindings.
    pub fn load(&mut self) -> Result<(), DeviceBindingsError> {
        if !self.file_path.exists() {
            tracing::info!(
                service = "keyrx",
                event = "bindings_initialized",
                component = "registry",
                path = %self.file_path.display(),
                "No bindings file found, starting with empty bindings"
            );
            return Ok(());
        }

        let content = std::fs::read(&self.file_path).map_err(|source| DeviceBindingsError::Io {
            path: self.file_path.clone(),
            source,
        })?;

        match serde_json::from_slice::<HashMap<String, DeviceBinding>>(&content) {
            Ok(raw_bindings) => {
                // Convert string keys to DeviceIdentity
                self.bindings.clear();
                for (key, binding) in raw_bindings {
                    match DeviceIdentity::from_key(&key) {
                        Ok(identity) => {
                            self.bindings.insert(identity, binding);
                        }
                        Err(e) => {
                            tracing::warn!(
                                service = "keyrx",
                                event = "invalid_binding_key",
                                component = "registry",
                                key = %key,
                                error = %e,
                                "Skipping invalid device key in bindings file"
                            );
                        }
                    }
                }

                tracing::info!(
                    service = "keyrx",
                    event = "bindings_loaded",
                    component = "registry",
                    path = %self.file_path.display(),
                    count = self.bindings.len(),
                    "Device bindings loaded successfully"
                );

                Ok(())
            }
            Err(e) => {
                // File is corrupted, create backup and return empty
                self.create_backup()?;

                tracing::error!(
                    service = "keyrx",
                    event = "bindings_corruption",
                    component = "registry",
                    path = %self.file_path.display(),
                    error = %e,
                    "Corrupted bindings file, created backup and starting fresh"
                );

                self.bindings.clear();
                Ok(())
            }
        }
    }

    /// Save bindings to disk with atomic write
    ///
    /// Uses temp file + rename to ensure atomicity
    pub fn save(&self) -> Result<(), DeviceBindingsError> {
        // Ensure parent directory exists
        if let Some(parent) = self.file_path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| DeviceBindingsError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        // Convert DeviceIdentity keys to strings for JSON serialization
        let raw_bindings: HashMap<String, &DeviceBinding> = self
            .bindings
            .iter()
            .map(|(identity, binding)| (identity.to_key(), binding))
            .collect();

        // Serialize to JSON
        let serialized = serde_json::to_vec_pretty(&raw_bindings).map_err(|source| {
            DeviceBindingsError::Serialization {
                path: self.file_path.clone(),
                source,
            }
        })?;

        // Atomic write: temp file + rename
        let tmp_path = self.file_path.with_extension("json.tmp");
        std::fs::write(&tmp_path, serialized).map_err(|source| DeviceBindingsError::Io {
            path: tmp_path.clone(),
            source,
        })?;

        std::fs::rename(&tmp_path, &self.file_path).map_err(|source| {
            // Best-effort cleanup of temp file
            let _ = std::fs::remove_file(&tmp_path);
            DeviceBindingsError::Io {
                path: self.file_path.clone(),
                source,
            }
        })?;

        tracing::info!(
            service = "keyrx",
            event = "bindings_saved",
            component = "registry",
            path = %self.file_path.display(),
            count = self.bindings.len(),
            "Device bindings saved successfully"
        );

        Ok(())
    }

    /// Get the binding for a device
    pub fn get_binding(&self, device: &DeviceIdentity) -> Option<&DeviceBinding> {
        self.bindings.get(device)
    }

    /// Set the binding for a device
    pub fn set_binding(&mut self, device: DeviceIdentity, binding: DeviceBinding) {
        self.bindings.insert(device, binding);
    }

    /// Remove the binding for a device
    pub fn remove_binding(&mut self, device: &DeviceIdentity) -> Option<DeviceBinding> {
        self.bindings.remove(device)
    }

    /// Get all bindings
    pub fn all_bindings(&self) -> &HashMap<DeviceIdentity, DeviceBinding> {
        &self.bindings
    }

    /// Clear all bindings
    pub fn clear(&mut self) {
        self.bindings.clear();
    }

    /// Get the number of bindings
    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    /// Check if there are no bindings
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    /// Create a backup of the current bindings file
    fn create_backup(&self) -> Result<(), DeviceBindingsError> {
        if !self.file_path.exists() {
            return Ok(());
        }

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_path = self
            .file_path
            .with_file_name(format!("device_bindings.{}.bak", timestamp));

        std::fs::copy(&self.file_path, &backup_path).map_err(|source| DeviceBindingsError::Io {
            path: backup_path.clone(),
            source,
        })?;

        tracing::info!(
            service = "keyrx",
            event = "bindings_backup_created",
            component = "registry",
            backup_path = %backup_path.display(),
            "Created backup of corrupted bindings file"
        );

        Ok(())
    }
}

impl Default for DeviceBindings {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_device_binding_new() {
        let binding = DeviceBinding::new();
        assert_eq!(binding.profile_id, None);
        assert!(binding.remap_enabled);
    }

    #[test]
    fn test_device_binding_with_profile() {
        let profile_id = "test-profile-123".to_string();
        let binding = DeviceBinding::with_profile(profile_id.clone());
        assert_eq!(binding.profile_id, Some(profile_id));
        assert!(binding.remap_enabled);
    }

    #[test]
    fn test_device_binding_disabled() {
        let binding = DeviceBinding::disabled();
        assert_eq!(binding.profile_id, None);
        assert!(!binding.remap_enabled);
    }

    #[test]
    fn test_device_bindings_new() {
        let bindings = DeviceBindings::new();
        assert!(bindings.is_empty());
    }

    #[test]
    fn test_set_and_get_binding() {
        let mut bindings = DeviceBindings::new();
        let device = DeviceIdentity::new(0x046D, 0xC52B, "ABC123".to_string());
        let binding = DeviceBinding::with_profile("profile-1".to_string());

        bindings.set_binding(device.clone(), binding.clone());

        let retrieved = bindings.get_binding(&device);
        assert_eq!(retrieved, Some(&binding));
    }

    #[test]
    fn test_remove_binding() {
        let mut bindings = DeviceBindings::new();
        let device = DeviceIdentity::new(0x046D, 0xC52B, "ABC123".to_string());
        let binding = DeviceBinding::new();

        bindings.set_binding(device.clone(), binding.clone());
        assert_eq!(bindings.len(), 1);

        let removed = bindings.remove_binding(&device);
        assert_eq!(removed, Some(binding));
        assert_eq!(bindings.len(), 0);
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test_bindings.json");

        // Create and populate bindings
        let mut bindings = DeviceBindings::with_path(file_path.clone());
        let device1 = DeviceIdentity::new(0x046D, 0xC52B, "ABC123".to_string());
        let device2 = DeviceIdentity::new(0x1234, 0x5678, "XYZ789".to_string());

        bindings.set_binding(
            device1.clone(),
            DeviceBinding::with_profile("profile-1".to_string()),
        );
        bindings.set_binding(device2.clone(), DeviceBinding::disabled());

        // Save
        bindings.save().unwrap();
        assert!(file_path.exists());

        // Load into new instance
        let mut loaded = DeviceBindings::with_path(file_path);
        loaded.load().unwrap();

        assert_eq!(loaded.len(), 2);
        assert_eq!(
            loaded.get_binding(&device1).unwrap().profile_id,
            Some("profile-1".to_string())
        );
        assert!(!loaded.get_binding(&device2).unwrap().remap_enabled);
    }

    #[test]
    fn test_load_nonexistent_file() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("nonexistent.json");

        let mut bindings = DeviceBindings::with_path(file_path);
        let result = bindings.load();

        assert!(result.is_ok());
        assert!(bindings.is_empty());
    }

    #[test]
    fn test_load_corrupted_file() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("device_bindings.json");

        // Write invalid JSON
        std::fs::write(&file_path, b"{ this is not valid json }").unwrap();

        let mut bindings = DeviceBindings::with_path(file_path.clone());
        let result = bindings.load();

        // Should succeed but with empty bindings
        assert!(result.is_ok());
        assert!(bindings.is_empty());

        // Backup should be created with pattern device_bindings.*.bak
        let backup_files: Vec<_> = std::fs::read_dir(temp_dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                name.starts_with("device_bindings.") && name.ends_with(".bak")
            })
            .collect();

        assert_eq!(backup_files.len(), 1);
    }

    #[test]
    fn test_atomic_write() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("atomic.json");

        let mut bindings = DeviceBindings::with_path(file_path.clone());
        let device = DeviceIdentity::new(0x046D, 0xC52B, "ABC123".to_string());
        bindings.set_binding(device, DeviceBinding::new());

        bindings.save().unwrap();

        // Verify temp file was cleaned up
        let tmp_path = file_path.with_extension("json.tmp");
        assert!(!tmp_path.exists());

        // Verify final file exists
        assert!(file_path.exists());
    }

    #[test]
    fn test_clear_bindings() {
        let mut bindings = DeviceBindings::new();
        let device = DeviceIdentity::new(0x046D, 0xC52B, "ABC123".to_string());
        bindings.set_binding(device, DeviceBinding::new());

        assert_eq!(bindings.len(), 1);

        bindings.clear();
        assert!(bindings.is_empty());
    }

    #[test]
    fn test_serialization_format() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("format.json");

        let mut bindings = DeviceBindings::with_path(file_path.clone());
        let device = DeviceIdentity::new(0x046D, 0xC52B, "ABC123".to_string());
        bindings.set_binding(
            device,
            DeviceBinding::with_profile("test-profile".to_string()),
        );

        bindings.save().unwrap();

        // Read and verify JSON structure
        let content = std::fs::read_to_string(&file_path).unwrap();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();

        assert!(json.is_object());
        assert!(json.get("046d:c52b:ABC123").is_some());

        let binding_json = &json["046d:c52b:ABC123"];
        assert_eq!(binding_json["profile_id"], "test-profile");
        assert_eq!(binding_json["remap_enabled"], true);
    }
}
