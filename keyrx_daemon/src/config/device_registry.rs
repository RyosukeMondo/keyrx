//! Device Registry - Persistent device metadata storage with atomic writes
//!
//! This module provides the DeviceRegistry component for managing device metadata
//! with atomic write operations and comprehensive input validation.

use crate::error::RegistryError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use typeshare::typeshare;

/// Device scope determines whether configuration applies globally or per-device
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceScope {
    /// Configuration applies only to this specific device
    DeviceSpecific,
    /// Configuration applies globally to all devices
    Global,
}

/// Device metadata entry
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeviceEntry {
    /// Unique device identifier (max 256 chars)
    pub id: String,
    /// User-friendly name (max 64 chars)
    pub name: String,
    /// Serial number if available
    pub serial: Option<String>,
    /// Scope for configuration application
    pub scope: DeviceScope,
    /// Associated layout name (max 32 chars)
    pub layout: Option<String>,
    /// Last seen timestamp (Unix seconds)
    #[typeshare(serialized_as = "number")]
    pub last_seen: u64,
}

/// Validation error types for device registry operations
#[derive(Debug)]
pub enum DeviceValidationError {
    /// Device not found in registry
    DeviceNotFound(String),
    /// Invalid device name (too long or invalid characters)
    InvalidName(String),
    /// Invalid device ID (too long)
    InvalidDeviceId(String),
}

impl std::fmt::Display for DeviceValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceValidationError::DeviceNotFound(id) => write!(f, "Device not found: {}", id),
            DeviceValidationError::InvalidName(msg) => write!(f, "Invalid device name: {}", msg),
            DeviceValidationError::InvalidDeviceId(msg) => write!(f, "Invalid device ID: {}", msg),
        }
    }
}

impl std::error::Error for DeviceValidationError {}

/// Device registry with persistent storage
pub struct DeviceRegistry {
    devices: HashMap<String, DeviceEntry>,
    path: PathBuf,
}

impl DeviceRegistry {
    /// Create a new empty registry with the given path
    pub fn new(path: PathBuf) -> Self {
        Self {
            devices: HashMap::new(),
            path,
        }
    }

    /// Load registry from disk with automatic recovery from corruption
    ///
    /// If the registry file is corrupted, creates an empty registry and saves it.
    /// If file does not exist, returns an empty registry.
    ///
    /// # Errors
    ///
    /// Returns `RegistryError::FailedToLoad` if the file cannot be read (e.g., permission denied).
    /// Returns `RegistryError::IOError` if the recovered registry cannot be saved.
    pub fn load(path: &Path) -> Result<Self, RegistryError> {
        match std::fs::read_to_string(path) {
            Ok(contents) => match serde_json::from_str(&contents) {
                Ok(devices) => {
                    log::debug!("Loaded device registry from {:?}", path);
                    Ok(Self {
                        devices,
                        path: path.to_path_buf(),
                    })
                }
                Err(e) => {
                    log::warn!(
                        "Corrupted registry at {:?}: {}. Creating empty registry.",
                        path,
                        e
                    );
                    let empty = Self::new(path.to_path_buf());
                    empty.save()?;
                    Ok(empty)
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                log::info!("No registry file found, creating new registry");
                let empty = Self::new(path.to_path_buf());
                empty.save()?;
                Ok(empty)
            }
            Err(e) => Err(RegistryError::FailedToLoad(e.kind())),
        }
    }

    /// Save registry to disk with atomic write
    ///
    /// Uses write-to-temp-then-rename pattern to prevent corruption.
    /// Creates parent directory if it doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns `RegistryError::CorruptedRegistry` if serialization fails.
    /// Returns `RegistryError::IOError` if parent directory creation, file write, or rename fails.
    pub fn save(&self) -> Result<(), RegistryError> {
        // Ensure parent directory exists
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| RegistryError::IOError(e.kind()))?;
        }

        let tmp_path = self.path.with_extension("tmp");

        let json = serde_json::to_string_pretty(&self.devices)
            .map_err(|e| RegistryError::CorruptedRegistry(e.to_string()))?;

        std::fs::write(&tmp_path, json).map_err(|e| RegistryError::IOError(e.kind()))?;

        std::fs::rename(&tmp_path, &self.path).map_err(|e| RegistryError::IOError(e.kind()))?;

        log::debug!("Saved device registry to {:?}", self.path);
        Ok(())
    }

    /// Rename a device
    ///
    /// Validates that name is ≤64 chars and contains only valid characters
    pub fn rename(&mut self, id: &str, name: &str) -> Result<(), DeviceValidationError> {
        validate_device_name(name)?;

        let device = self
            .devices
            .get_mut(id)
            .ok_or_else(|| DeviceValidationError::DeviceNotFound(id.to_string()))?;

        device.name = name.to_string();
        Ok(())
    }

    /// Set device scope
    pub fn set_scope(&mut self, id: &str, scope: DeviceScope) -> Result<(), DeviceValidationError> {
        let device = self
            .devices
            .get_mut(id)
            .ok_or_else(|| DeviceValidationError::DeviceNotFound(id.to_string()))?;

        device.scope = scope;
        Ok(())
    }

    /// Set device layout
    ///
    /// Validates that layout name is ≤32 chars
    pub fn set_layout(&mut self, id: &str, layout: &str) -> Result<(), DeviceValidationError> {
        validate_layout_name(layout)?;

        let device = self
            .devices
            .get_mut(id)
            .ok_or_else(|| DeviceValidationError::DeviceNotFound(id.to_string()))?;

        device.layout = Some(layout.to_string());
        Ok(())
    }

    /// Remove device from registry
    pub fn forget(&mut self, id: &str) -> Result<DeviceEntry, DeviceValidationError> {
        self.devices
            .remove(id)
            .ok_or_else(|| DeviceValidationError::DeviceNotFound(id.to_string()))
    }

    /// List all devices
    pub fn list(&self) -> Vec<&DeviceEntry> {
        self.devices.values().collect()
    }

    /// Get device by ID
    pub fn get(&self, id: &str) -> Option<&DeviceEntry> {
        self.devices.get(id)
    }

    /// Update last_seen timestamp for a device
    pub fn update_last_seen(&mut self, id: &str) -> Result<(), DeviceValidationError> {
        let device = self
            .devices
            .get_mut(id)
            .ok_or_else(|| DeviceValidationError::DeviceNotFound(id.to_string()))?;

        device.last_seen = current_timestamp();
        Ok(())
    }

    /// Register a new device or update existing one
    pub fn register(&mut self, entry: DeviceEntry) -> Result<(), DeviceValidationError> {
        validate_device_id(&entry.id)?;
        validate_device_name(&entry.name)?;

        if let Some(layout) = &entry.layout {
            validate_layout_name(layout)?;
        }

        self.devices.insert(entry.id.clone(), entry);
        Ok(())
    }
}

/// Validate device name: ≤64 chars, alphanumeric + space/dash/underscore only
fn validate_device_name(name: &str) -> Result<(), DeviceValidationError> {
    if name.is_empty() {
        return Err(DeviceValidationError::InvalidName(
            "Device name cannot be empty".to_string(),
        ));
    }

    if name.len() > 64 {
        return Err(DeviceValidationError::InvalidName(format!(
            "Device name too long: {} chars (max 64)",
            name.len()
        )));
    }

    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == ' ' || c == '-' || c == '_')
    {
        return Err(DeviceValidationError::InvalidName(
            "Device name contains invalid characters (only alphanumeric, space, dash, underscore allowed)".to_string()
        ));
    }

    Ok(())
}

/// Validate device ID: ≤256 chars
fn validate_device_id(id: &str) -> Result<(), DeviceValidationError> {
    if id.is_empty() {
        return Err(DeviceValidationError::InvalidDeviceId(
            "Device ID cannot be empty".to_string(),
        ));
    }

    if id.len() > 256 {
        return Err(DeviceValidationError::InvalidDeviceId(format!(
            "Device ID too long: {} chars (max 256)",
            id.len()
        )));
    }

    Ok(())
}

/// Validate layout name: ≤32 chars
fn validate_layout_name(layout: &str) -> Result<(), DeviceValidationError> {
    if layout.len() > 32 {
        return Err(DeviceValidationError::InvalidName(format!(
            "Layout name too long: {} chars (max 32)",
            layout.len()
        )));
    }

    Ok(())
}

/// Get current Unix timestamp in seconds
fn current_timestamp() -> u64 {
    // SAFETY: SystemTime::now() is always after UNIX_EPOCH on modern systems
    #[allow(clippy::expect_used)]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("System time is before UNIX_EPOCH")
            .as_secs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_device(id: &str, name: &str) -> DeviceEntry {
        DeviceEntry {
            id: id.to_string(),
            name: name.to_string(),
            serial: None,
            scope: DeviceScope::Global,
            layout: None,
            last_seen: current_timestamp(),
        }
    }

    #[test]
    fn test_new_registry() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("registry.json");
        let registry = DeviceRegistry::new(path.clone());

        assert_eq!(registry.list().len(), 0);
        assert_eq!(registry.path, path);
    }

    #[test]
    fn test_register_device() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("registry.json");
        let mut registry = DeviceRegistry::new(path);

        let device = create_test_device("dev1", "Test Device");
        registry.register(device).unwrap();

        assert_eq!(registry.list().len(), 1);
        assert!(registry.get("dev1").is_some());
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("registry.json");

        let mut registry = DeviceRegistry::new(path.clone());
        let device = create_test_device("dev1", "Test Device");
        registry.register(device).unwrap();
        registry.save().unwrap();

        let loaded = DeviceRegistry::load(&path).unwrap();
        assert_eq!(loaded.list().len(), 1);

        let loaded_device = loaded.get("dev1").unwrap();
        assert_eq!(loaded_device.name, "Test Device");
    }

    #[test]
    fn test_atomic_write_no_corruption() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("registry.json");

        let mut registry = DeviceRegistry::new(path.clone());
        let device = create_test_device("dev1", "Device1");
        registry.register(device).unwrap();
        registry.save().unwrap();

        let tmp_path = path.with_extension("tmp");
        assert!(
            !tmp_path.exists(),
            "Temp file should be removed after atomic rename"
        );
        assert!(path.exists(), "Registry file should exist");
    }

    #[test]
    fn test_rename_device() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("registry.json");
        let mut registry = DeviceRegistry::new(path);

        let device = create_test_device("dev1", "Old Name");
        registry.register(device).unwrap();

        registry.rename("dev1", "New Name").unwrap();
        assert_eq!(registry.get("dev1").unwrap().name, "New Name");
    }

    #[test]
    fn test_rename_nonexistent_device() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("registry.json");
        let mut registry = DeviceRegistry::new(path);

        let result = registry.rename("nonexistent", "New Name");
        assert!(matches!(
            result,
            Err(DeviceValidationError::DeviceNotFound(_))
        ));
    }

    #[test]
    fn test_set_scope() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("registry.json");
        let mut registry = DeviceRegistry::new(path);

        let device = create_test_device("dev1", "Device");
        registry.register(device).unwrap();

        registry
            .set_scope("dev1", DeviceScope::DeviceSpecific)
            .unwrap();
        assert_eq!(
            registry.get("dev1").unwrap().scope,
            DeviceScope::DeviceSpecific
        );
    }

    #[test]
    fn test_set_layout() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("registry.json");
        let mut registry = DeviceRegistry::new(path);

        let device = create_test_device("dev1", "Device");
        registry.register(device).unwrap();

        registry.set_layout("dev1", "ansi_104").unwrap();
        assert_eq!(
            registry.get("dev1").unwrap().layout,
            Some("ansi_104".to_string())
        );
    }

    #[test]
    fn test_forget_device() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("registry.json");
        let mut registry = DeviceRegistry::new(path);

        let device = create_test_device("dev1", "Device");
        registry.register(device).unwrap();

        let removed = registry.forget("dev1").unwrap();
        assert_eq!(removed.id, "dev1");
        assert!(registry.get("dev1").is_none());
    }

    #[test]
    fn test_update_last_seen() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("registry.json");
        let mut registry = DeviceRegistry::new(path);

        let mut device = create_test_device("dev1", "Device");
        device.last_seen = 1000;
        registry.register(device).unwrap();

        registry.update_last_seen("dev1").unwrap();

        let updated = registry.get("dev1").unwrap();
        assert!(updated.last_seen > 1000);
    }

    #[test]
    fn test_validate_device_name_too_long() {
        let long_name = "a".repeat(65);
        let result = validate_device_name(&long_name);
        assert!(matches!(result, Err(DeviceValidationError::InvalidName(_))));
    }

    #[test]
    fn test_validate_device_name_invalid_chars() {
        let result = validate_device_name("Device@#$");
        assert!(matches!(result, Err(DeviceValidationError::InvalidName(_))));
    }

    #[test]
    fn test_validate_device_name_valid() {
        assert!(validate_device_name("My Device-123_test").is_ok());
    }

    #[test]
    fn test_validate_device_id_too_long() {
        let long_id = "a".repeat(257);
        let result = validate_device_id(&long_id);
        assert!(matches!(
            result,
            Err(DeviceValidationError::InvalidDeviceId(_))
        ));
    }

    #[test]
    fn test_validate_layout_name_too_long() {
        let long_layout = "a".repeat(33);
        let result = validate_layout_name(&long_layout);
        assert!(matches!(result, Err(DeviceValidationError::InvalidName(_))));
    }

    #[test]
    fn test_corrupted_registry_file_recovery() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("registry.json");

        fs::write(&path, "{ invalid json ").unwrap();

        // Load should recover by creating empty registry
        let registry = DeviceRegistry::load(&path).unwrap();
        assert_eq!(registry.list().len(), 0);

        // Verify the file was fixed
        let contents = fs::read_to_string(&path).unwrap();
        assert!(contents.contains("{}") || contents.contains("{ }"));
    }

    #[test]
    fn test_load_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("nonexistent.json");

        let registry = DeviceRegistry::load(&path).unwrap();
        assert_eq!(registry.list().len(), 0);
    }

    #[test]
    fn test_list_devices() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("registry.json");
        let mut registry = DeviceRegistry::new(path);

        registry
            .register(create_test_device("dev1", "Device 1"))
            .unwrap();
        registry
            .register(create_test_device("dev2", "Device 2"))
            .unwrap();

        let list = registry.list();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_empty_device_name() {
        let result = validate_device_name("");
        assert!(matches!(result, Err(DeviceValidationError::InvalidName(_))));
    }

    #[test]
    fn test_empty_device_id() {
        let result = validate_device_id("");
        assert!(matches!(
            result,
            Err(DeviceValidationError::InvalidDeviceId(_))
        ));
    }

    #[test]
    #[cfg(unix)] // Permission-based tests only work on Unix
    fn test_write_protected_directory() {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        // Test that save returns proper error when directory is not writable
        let temp_dir = TempDir::new().unwrap();
        let protected_dir = temp_dir.path().join("protected");
        fs::create_dir(&protected_dir).unwrap();

        // Make directory read-only (remove write permission)
        let mut perms = fs::metadata(&protected_dir).unwrap().permissions();
        perms.set_mode(0o444); // r--r--r--
        fs::set_permissions(&protected_dir, perms).unwrap();

        let path = protected_dir.join("registry.json");
        let registry = DeviceRegistry::new(path);
        let result = registry.save();

        // Cleanup: restore permissions before TempDir cleanup
        let mut perms = fs::metadata(&protected_dir).unwrap().permissions();
        perms.set_mode(0o755);
        let _ = fs::set_permissions(&protected_dir, perms);

        assert!(matches!(result, Err(RegistryError::IOError(_))));
    }

    #[test]
    fn test_creates_parent_directory() {
        // Test that save creates parent directory if it doesn't exist
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir
            .path()
            .join("nonexistent_dir")
            .join("registry.json");

        let registry = DeviceRegistry::new(path.clone());
        let result = registry.save();

        assert!(result.is_ok());
        assert!(path.exists());
        assert!(path.parent().unwrap().exists());
    }

    #[test]
    fn test_recovery_creates_valid_empty_registry() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("registry.json");

        // Write corrupted data
        fs::write(&path, "[this is not a hashmap]").unwrap();

        // Load should recover and create empty registry
        let registry = DeviceRegistry::load(&path).unwrap();
        assert_eq!(registry.list().len(), 0);

        // Verify we can add devices to the recovered registry
        let mut registry = registry;
        let device = create_test_device("dev1", "Test Device");
        registry.register(device).unwrap();
        assert_eq!(registry.list().len(), 1);
    }
}
