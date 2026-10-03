//! Device Registry - Persistent device metadata storage with atomic writes
//!
//! This module provides the DeviceRegistry component for managing device metadata
//! with atomic write operations and comprehensive input validation.

use crate::error::RegistryError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use typeshare::typeshare;

/// Legacy device scope enum - kept for backward compatibility with old registry files
/// This allows old registry files with the "scope" field to be loaded without errors
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum DeviceScopeLegacy {
    DeviceSpecific,
    Global,
}

/// Device metadata entry
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeviceEntry {
    /// Unique device identifier (max 256 chars)
    pub id: String,
    /// User-friendly name (max 64 chars)
    pub name: String,
    /// Serial number if available
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub serial: Option<String>,
    /// Associated layout name (max 32 chars)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<String>,
    /// Last seen timestamp (Unix seconds)
    #[typeshare(serialized_as = "number")]
    pub last_seen: u64,
    /// Scope field for backward compatibility (ignored during serialization, accepted during deserialization)
    #[serde(skip_serializing, default)]
    #[typeshare(skip)]
    #[allow(dead_code)]
    scope: Option<DeviceScopeLegacy>,
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

impl DeviceEntry {
    /// Create a new device entry
    pub fn new(
        id: String,
        name: String,
        serial: Option<String>,
        layout: Option<String>,
        last_seen: u64,
    ) -> Self {
        Self {
            id,
            name,
            serial,
            layout,
            last_seen,
            scope: None,
        }
    }
}

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

        let json = serde_json::to_string_pretty(&self.devices)
            .map_err(|e| RegistryError::CorruptedRegistry(e.to_string()))?;

        crate::config::atomic_file::write_atomic(&self.path, json.as_bytes())
            .map_err(|e| RegistryError::IOError(e.kind()))?;

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

    /// Registers `id` if unknown, named after `default_name` made valid
    /// (invalid characters become '-', cut to 64 chars). The web UI edits
    /// connected devices, which need not have been registered before.
    pub fn ensure_registered(
        &mut self,
        id: &str,
        default_name: &str,
    ) -> Result<(), DeviceValidationError> {
        if self.devices.contains_key(id) {
            return Ok(());
        }
        let mut name: String = default_name
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' {
                    c
                } else {
                    '-'
                }
            })
            .take(64)
            .collect();
        if name.trim().is_empty() {
            name = "Keyboard".to_string();
        }
        let last_seen = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or_default();
        self.register(DeviceEntry::new(
            id.to_string(),
            name,
            None,
            None,
            last_seen,
        ))
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
#[path = "device_registry_tests.rs"]
mod tests;
