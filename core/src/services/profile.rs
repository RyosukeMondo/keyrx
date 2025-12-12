//! Profile management service.
//!
//! This module provides the [`ProfileService`] for managing keymaps, hardware profiles,
//! and virtual layouts. It provides CRUD operations for all profile-related data.

use crate::config::models::{HardwareProfile, Keymap, VirtualLayout};
use crate::config::{ConfigManager, StorageError};
use thiserror::Error;

use super::traits::ProfileServiceTrait;

/// Errors that can occur during profile service operations.
///
/// This error type covers failures when managing profiles, keymaps,
/// and virtual layouts.
#[derive(Error, Debug)]
pub enum ProfileServiceError {
    /// Error from the underlying storage layer.
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    /// The requested resource was not found.
    ///
    /// The string contains the ID of the resource that was not found.
    #[error("Not found: {0}")]
    NotFound(String),
}

/// Service for managing profiles, keymaps, and virtual layouts.
///
/// The profile service provides CRUD operations for:
/// - **Virtual Layouts**: Define the logical key arrangement
/// - **Hardware Profiles**: Map physical keys to virtual layout positions
/// - **Keymaps**: Define key bindings and layers
///
/// # Dependency Injection
///
/// Use [`ProfileService::new`] to inject a custom [`ConfigManager`] for testing,
/// or [`ProfileService::with_defaults`] for production use.
///
/// # Example
///
/// ```no_run
/// use keyrx_core::services::{ProfileService, ProfileServiceTrait};
///
/// let service = ProfileService::with_defaults();
/// let keymaps = service.list_keymaps().expect("Failed to list keymaps");
/// for keymap in &keymaps {
///     println!("Keymap: {} ({})", keymap.name, keymap.id);
/// }
/// ```
pub struct ProfileService {
    config_manager: ConfigManager,
}

impl Default for ProfileService {
    fn default() -> Self {
        Self::with_defaults()
    }
}

impl ProfileService {
    /// Creates a new ProfileService with the provided ConfigManager.
    ///
    /// Use this constructor for dependency injection, allowing tests to
    /// inject mock or custom ConfigManager implementations.
    pub fn new(config_manager: ConfigManager) -> Self {
        Self { config_manager }
    }

    /// Creates a ProfileService with default dependencies.
    ///
    /// This is the convenience constructor for production use, creating
    /// a ConfigManager with default settings.
    pub fn with_defaults() -> Self {
        Self::new(ConfigManager::default())
    }
}

impl ProfileServiceTrait for ProfileService {
    fn list_virtual_layouts(&self) -> Result<Vec<VirtualLayout>, ProfileServiceError> {
        self.config_manager
            .load_virtual_layouts()
            .map(|m| m.into_values().collect())
            .map_err(Into::into)
    }

    fn save_virtual_layout(
        &self,
        layout: VirtualLayout,
    ) -> Result<VirtualLayout, ProfileServiceError> {
        self.config_manager.save_virtual_layout(&layout)?;
        Ok(layout)
    }

    fn delete_virtual_layout(&self, id: &str) -> Result<(), ProfileServiceError> {
        self.config_manager.delete_virtual_layout(id)?;
        Ok(())
    }

    fn list_hardware_profiles(&self) -> Result<Vec<HardwareProfile>, ProfileServiceError> {
        self.config_manager
            .load_hardware_profiles()
            .map(|m| m.into_values().collect())
            .map_err(Into::into)
    }

    fn save_hardware_profile(
        &self,
        profile: HardwareProfile,
    ) -> Result<HardwareProfile, ProfileServiceError> {
        self.config_manager.save_hardware_profile(&profile)?;
        Ok(profile)
    }

    fn delete_hardware_profile(&self, id: &str) -> Result<(), ProfileServiceError> {
        self.config_manager.delete_hardware_profile(id)?;
        Ok(())
    }

    fn list_keymaps(&self) -> Result<Vec<Keymap>, ProfileServiceError> {
        self.config_manager
            .load_keymaps()
            .map(|m| m.into_values().collect())
            .map_err(Into::into)
    }

    fn save_keymap(&self, keymap: Keymap) -> Result<Keymap, ProfileServiceError> {
        self.config_manager.save_keymap(&keymap)?;
        Ok(keymap)
    }

    fn delete_keymap(&self, id: &str) -> Result<(), ProfileServiceError> {
        self.config_manager.delete_keymap(id)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::models::LayoutType;
    use std::collections::HashMap;
    use tempfile::tempdir;

    fn test_layout(id: &str) -> VirtualLayout {
        VirtualLayout {
            id: id.into(),
            name: format!("Layout {}", id),
            layout_type: LayoutType::Matrix,
            keys: vec![],
        }
    }

    fn test_hardware_profile(id: &str) -> HardwareProfile {
        HardwareProfile {
            id: id.into(),
            vendor_id: 0x1234,
            product_id: 0x5678,
            name: Some(format!("Profile {}", id)),
            virtual_layout_id: "layout-1".into(),
            wiring: HashMap::new(),
        }
    }

    fn test_keymap(id: &str) -> Keymap {
        Keymap {
            id: id.into(),
            name: format!("Keymap {}", id),
            virtual_layout_id: "layout-1".into(),
            layers: vec![],
            combos: vec![],
        }
    }

    #[test]
    fn service_new_creates_with_config_manager() {
        let dir = tempdir().unwrap();
        let config_manager = ConfigManager::new(dir.path());
        let service = ProfileService::new(config_manager);

        // Verify service works by calling list (should be empty initially)
        let layouts = service.list_virtual_layouts().unwrap();
        assert!(layouts.is_empty());
    }

    #[test]
    fn service_default_creates_with_defaults() {
        // Just verify Default impl works (delegates to with_defaults)
        let _service = ProfileService::default();
    }

    // ========== Virtual Layout Tests ==========

    #[test]
    fn list_virtual_layouts_returns_empty_initially() {
        let dir = tempdir().unwrap();
        let service = ProfileService::new(ConfigManager::new(dir.path()));

        let layouts = service.list_virtual_layouts().unwrap();
        assert!(layouts.is_empty());
    }

    #[test]
    fn save_and_list_virtual_layout() {
        let dir = tempdir().unwrap();
        let service = ProfileService::new(ConfigManager::new(dir.path()));

        let layout = test_layout("test-layout");
        let saved = service.save_virtual_layout(layout.clone()).unwrap();
        assert_eq!(saved.id, "test-layout");

        let layouts = service.list_virtual_layouts().unwrap();
        assert_eq!(layouts.len(), 1);
        assert_eq!(layouts[0].id, "test-layout");
    }

    #[test]
    fn save_virtual_layout_updates_existing() {
        let dir = tempdir().unwrap();
        let service = ProfileService::new(ConfigManager::new(dir.path()));

        // Save initial layout
        let mut layout = test_layout("test-layout");
        service.save_virtual_layout(layout.clone()).unwrap();

        // Update and save again
        layout.name = "Updated Name".into();
        service.save_virtual_layout(layout.clone()).unwrap();

        // Verify only one layout exists with updated name
        let layouts = service.list_virtual_layouts().unwrap();
        assert_eq!(layouts.len(), 1);
        assert_eq!(layouts[0].name, "Updated Name");
    }

    #[test]
    fn delete_virtual_layout_removes_it() {
        let dir = tempdir().unwrap();
        let service = ProfileService::new(ConfigManager::new(dir.path()));

        let layout = test_layout("to-delete");
        service.save_virtual_layout(layout).unwrap();
        assert_eq!(service.list_virtual_layouts().unwrap().len(), 1);

        service.delete_virtual_layout("to-delete").unwrap();
        assert!(service.list_virtual_layouts().unwrap().is_empty());
    }

    // ========== Hardware Profile Tests ==========

    #[test]
    fn list_hardware_profiles_returns_empty_initially() {
        let dir = tempdir().unwrap();
        let service = ProfileService::new(ConfigManager::new(dir.path()));

        let profiles = service.list_hardware_profiles().unwrap();
        assert!(profiles.is_empty());
    }

    #[test]
    fn save_and_list_hardware_profile() {
        let dir = tempdir().unwrap();
        let service = ProfileService::new(ConfigManager::new(dir.path()));

        let profile = test_hardware_profile("test-profile");
        let saved = service.save_hardware_profile(profile.clone()).unwrap();
        assert_eq!(saved.id, "test-profile");

        let profiles = service.list_hardware_profiles().unwrap();
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].id, "test-profile");
    }

    #[test]
    fn delete_hardware_profile_removes_it() {
        let dir = tempdir().unwrap();
        let service = ProfileService::new(ConfigManager::new(dir.path()));

        let profile = test_hardware_profile("to-delete");
        service.save_hardware_profile(profile).unwrap();
        assert_eq!(service.list_hardware_profiles().unwrap().len(), 1);

        service.delete_hardware_profile("to-delete").unwrap();
        assert!(service.list_hardware_profiles().unwrap().is_empty());
    }

    // ========== Keymap Tests ==========

    #[test]
    fn list_keymaps_returns_empty_initially() {
        let dir = tempdir().unwrap();
        let service = ProfileService::new(ConfigManager::new(dir.path()));

        let keymaps = service.list_keymaps().unwrap();
        assert!(keymaps.is_empty());
    }

    #[test]
    fn save_and_list_keymap() {
        let dir = tempdir().unwrap();
        let service = ProfileService::new(ConfigManager::new(dir.path()));

        let keymap = test_keymap("test-keymap");
        let saved = service.save_keymap(keymap.clone()).unwrap();
        assert_eq!(saved.id, "test-keymap");

        let keymaps = service.list_keymaps().unwrap();
        assert_eq!(keymaps.len(), 1);
        assert_eq!(keymaps[0].id, "test-keymap");
    }

    #[test]
    fn delete_keymap_removes_it() {
        let dir = tempdir().unwrap();
        let service = ProfileService::new(ConfigManager::new(dir.path()));

        let keymap = test_keymap("to-delete");
        service.save_keymap(keymap).unwrap();
        assert_eq!(service.list_keymaps().unwrap().len(), 1);

        service.delete_keymap("to-delete").unwrap();
        assert!(service.list_keymaps().unwrap().is_empty());
    }

    // ========== Error Type Tests ==========

    #[test]
    fn profile_service_error_display() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let storage_err =
            ProfileServiceError::Storage(StorageError::ReadFile("test.json".into(), io_err));
        assert!(storage_err.to_string().contains("Storage error"));

        let not_found_err = ProfileServiceError::NotFound("item".into());
        assert!(not_found_err.to_string().contains("Not found"));
    }

    #[test]
    fn profile_service_error_from_storage_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let storage_err = StorageError::ReadFile("test.json".into(), io_err);
        let profile_err: ProfileServiceError = storage_err.into();
        assert!(matches!(profile_err, ProfileServiceError::Storage(_)));
    }
}
