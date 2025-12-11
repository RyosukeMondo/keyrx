//! Profile storage operations for persistent file I/O.
//!
//! This module handles all file-based persistence operations including:
//! - Loading profiles from disk
//! - Saving profiles with atomic writes
//! - Deleting profiles
//! - Loading all profiles from a directory

use super::resolution::ProfileRegistryResolution;
use super::{Profile, ProfileRegistry, ProfileRegistryError};
use std::path::Path;
use std::sync::Arc;

/// Extension trait for ProfileRegistry storage operations
pub trait ProfileRegistryStorage {
    /// Initialize the registry by loading all profiles from disk
    fn load_all_profiles(
        &self,
    ) -> impl std::future::Future<Output = Result<usize, ProfileRegistryError>> + Send;

    /// Load a profile from a file
    fn load_profile_from_file(&self, path: &Path) -> Result<Profile, ProfileRegistryError>;

    /// Save a profile to persistent storage with atomic write
    fn save_profile(
        &self,
        profile: &Profile,
    ) -> impl std::future::Future<Output = Result<(), ProfileRegistryError>> + Send;

    /// Delete a profile
    fn delete_profile(
        &self,
        profile_id: &str,
    ) -> impl std::future::Future<Output = Result<(), ProfileRegistryError>> + Send;
}

impl ProfileRegistryStorage for ProfileRegistry {
    /// Initialize the registry by loading all profiles from disk
    async fn load_all_profiles(&self) -> Result<usize, ProfileRegistryError> {
        // Ensure directory exists
        std::fs::create_dir_all(&self.profiles_dir).map_err(|source| ProfileRegistryError::Io {
            path: self.profiles_dir.clone(),
            source,
        })?;

        let mut count = 0;
        let entries =
            std::fs::read_dir(&self.profiles_dir).map_err(|source| ProfileRegistryError::Io {
                path: self.profiles_dir.clone(),
                source,
            })?;

        for entry in entries {
            let entry = entry.map_err(|source| ProfileRegistryError::Io {
                path: self.profiles_dir.clone(),
                source,
            })?;

            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match self.load_profile_from_file(&path) {
                    Ok(profile) => {
                        let mut cache = self.cache.write().await;
                        cache.insert(profile.id.clone(), Arc::new(profile));
                        count += 1;
                    }
                    Err(e) => {
                        tracing::warn!(
                            service = "keyrx",
                            event = "profile_load_failed",
                            component = "registry",
                            path = %path.display(),
                            error = %e,
                            "Failed to load profile, skipping"
                        );
                    }
                }
            }
        }

        Ok(count)
    }

    /// Load a profile from a file
    fn load_profile_from_file(&self, path: &Path) -> Result<Profile, ProfileRegistryError> {
        let data = std::fs::read_to_string(path).map_err(|source| ProfileRegistryError::Io {
            path: path.to_path_buf(),
            source,
        })?;

        let profile: Profile =
            serde_json::from_str(&data).map_err(|source| ProfileRegistryError::Serialization {
                path: path.to_path_buf(),
                source,
            })?;

        self.validate_profile(&profile)?;
        Ok(profile)
    }

    /// Save a profile to persistent storage with atomic write
    async fn save_profile(&self, profile: &Profile) -> Result<(), ProfileRegistryError> {
        self.validate_profile(profile)?;

        // Ensure directory exists
        std::fs::create_dir_all(&self.profiles_dir).map_err(|source| ProfileRegistryError::Io {
            path: self.profiles_dir.clone(),
            source,
        })?;

        let path = self.profile_file_path(&profile.id);

        // Serialize to JSON
        let serialized = serde_json::to_vec_pretty(profile).map_err(|source| {
            ProfileRegistryError::Serialization {
                path: path.clone(),
                source,
            }
        })?;

        // Atomic write: temp file + rename
        let tmp_path = path.with_extension("json.tmp");
        std::fs::write(&tmp_path, serialized).map_err(|source| ProfileRegistryError::Io {
            path: tmp_path.clone(),
            source,
        })?;

        std::fs::rename(&tmp_path, &path).map_err(|source| {
            // Best-effort cleanup of temp file
            let _ = std::fs::remove_file(&tmp_path);
            ProfileRegistryError::Io {
                path: path.clone(),
                source,
            }
        })?;

        // Update cache
        let mut cache = self.cache.write().await;
        cache.insert(profile.id.clone(), Arc::new(profile.clone()));

        tracing::info!(
            service = "keyrx",
            event = "profile_saved",
            component = "registry",
            profile_id = %profile.id,
            profile_name = %profile.name,
            "Profile saved successfully"
        );

        Ok(())
    }

    /// Delete a profile
    async fn delete_profile(&self, profile_id: &str) -> Result<(), ProfileRegistryError> {
        let path = self.profile_file_path(profile_id);

        if !path.exists() {
            return Err(ProfileRegistryError::NotFound(profile_id.to_string()));
        }

        std::fs::remove_file(&path).map_err(|source| ProfileRegistryError::Io {
            path: path.clone(),
            source,
        })?;

        // Remove from cache
        let mut cache = self.cache.write().await;
        cache.remove(profile_id);

        tracing::info!(
            service = "keyrx",
            event = "profile_deleted",
            component = "registry",
            profile_id = %profile_id,
            "Profile deleted successfully"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drivers::keycodes::KeyCode;
    use crate::registry::profile::{
        KeyAction, LayoutType, PhysicalPosition, ProfileRegistryResolution,
    };
    use serial_test::serial;
    use tempfile::tempdir;

    #[tokio::test]
    #[serial]
    async fn test_save_and_get_profile() {
        let temp = tempdir().unwrap();
        let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

        let mut profile = Profile::new("Test Profile", LayoutType::Matrix);
        profile.set_action(PhysicalPosition::new(0, 0), KeyAction::key(KeyCode::A));

        // Save profile
        registry.save_profile(&profile).await.unwrap();

        // Get profile back
        let loaded = registry.get_profile(&profile.id).await.unwrap();
        assert_eq!(loaded.id, profile.id);
        assert_eq!(loaded.name, profile.name);
        assert_eq!(loaded.layout_type, profile.layout_type);
        assert_eq!(loaded.mappings.len(), 1);
    }

    #[tokio::test]
    #[serial]
    async fn test_delete_profile() {
        let temp = tempdir().unwrap();
        let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

        let profile = Profile::new("Test Profile", LayoutType::Standard);
        registry.save_profile(&profile).await.unwrap();

        // Verify it exists
        assert!(registry.get_profile(&profile.id).await.is_ok());

        // Delete it
        registry.delete_profile(&profile.id).await.unwrap();

        // Verify it's gone
        assert!(matches!(
            registry.get_profile(&profile.id).await,
            Err(ProfileRegistryError::NotFound(_))
        ));
    }

    #[tokio::test]
    #[serial]
    async fn test_load_all_profiles() {
        let temp = tempdir().unwrap();
        let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

        // Save some profiles
        let profile1 = Profile::new("Profile 1", LayoutType::Matrix);
        let profile2 = Profile::new("Profile 2", LayoutType::Standard);

        registry.save_profile(&profile1).await.unwrap();
        registry.save_profile(&profile2).await.unwrap();

        // Clear cache
        registry.clear_cache().await;
        assert_eq!(registry.cache_size().await, 0);

        // Load all profiles
        let count = registry.load_all_profiles().await.unwrap();
        assert_eq!(count, 2);
        assert_eq!(registry.cache_size().await, 2);
    }

    #[tokio::test]
    #[serial]
    async fn test_atomic_write() {
        let temp = tempdir().unwrap();
        let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

        let profile = Profile::new("Test", LayoutType::Matrix);
        let profile_path = registry.profile_file_path(&profile.id);
        let tmp_path = profile_path.with_extension("json.tmp");

        registry.save_profile(&profile).await.unwrap();

        // Verify final file exists and temp file is cleaned up
        assert!(profile_path.exists());
        assert!(!tmp_path.exists());
    }

    #[tokio::test]
    #[serial]
    async fn test_corrupted_file_handling() {
        let temp = tempdir().unwrap();
        let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

        // Create a corrupted file
        let corrupted_path = temp.path().join("corrupted.json");
        std::fs::write(&corrupted_path, "not valid json").unwrap();

        // load_all_profiles should skip corrupted files and continue
        let count = registry.load_all_profiles().await.unwrap();
        assert_eq!(count, 0); // No valid profiles loaded
    }
}
