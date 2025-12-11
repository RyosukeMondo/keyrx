//! Profile resolution, queries, and cache management.
//!
//! This module handles:
//! - Profile lookup and retrieval
//! - Querying profiles by criteria (layout type, etc.)
//! - Cache management (invalidation, clearing)
//! - Profile validation

use super::{LayoutType, Profile, ProfileId, ProfileRegistry, ProfileRegistryError};
use std::sync::Arc;

/// Extension trait for ProfileRegistry resolution and query operations
pub trait ProfileRegistryResolution {
    /// Get a profile by ID (returns Arc for zero-copy sharing)
    fn get_profile(
        &self,
        profile_id: &str,
    ) -> impl std::future::Future<Output = Result<Arc<Profile>, ProfileRegistryError>> + Send;

    /// List all profile IDs
    fn list_profiles(&self) -> impl std::future::Future<Output = Vec<ProfileId>> + Send;

    /// Find profiles compatible with a given layout type
    fn find_compatible_profiles(
        &self,
        layout_type: &LayoutType,
    ) -> impl std::future::Future<Output = Vec<Arc<Profile>>> + Send;

    /// Validate a profile before saving
    fn validate_profile(&self, profile: &Profile) -> Result<(), ProfileRegistryError>;

    /// Invalidate the cache for a specific profile (useful after external modifications)
    fn invalidate_cache(&self, profile_id: &str) -> impl std::future::Future<Output = ()> + Send;

    /// Clear the entire cache
    fn clear_cache(&self) -> impl std::future::Future<Output = ()> + Send;

    /// Get the number of profiles in cache
    fn cache_size(&self) -> impl std::future::Future<Output = usize> + Send;
}

impl ProfileRegistryResolution for ProfileRegistry {
    /// Get a profile by ID (returns Arc for zero-copy sharing)
    async fn get_profile(&self, profile_id: &str) -> Result<Arc<Profile>, ProfileRegistryError> {
        use super::storage::ProfileRegistryStorage;

        // Try cache first
        {
            let cache = self.cache.read().await;
            if let Some(profile) = cache.get(profile_id) {
                return Ok(Arc::clone(profile));
            }
        }

        // Not in cache, try loading from disk
        let path = self.profile_file_path(profile_id);
        if !path.exists() {
            return Err(ProfileRegistryError::NotFound(profile_id.to_string()));
        }

        let profile = self.load_profile_from_file(&path)?;
        let arc_profile = Arc::new(profile);

        // Update cache
        let mut cache = self.cache.write().await;
        cache.insert(profile_id.to_string(), Arc::clone(&arc_profile));

        Ok(arc_profile)
    }

    /// List all profile IDs
    async fn list_profiles(&self) -> Vec<ProfileId> {
        let cache = self.cache.read().await;
        cache.keys().cloned().collect()
    }

    /// Find profiles compatible with a given layout type
    async fn find_compatible_profiles(&self, layout_type: &LayoutType) -> Vec<Arc<Profile>> {
        let cache = self.cache.read().await;
        cache
            .values()
            .filter(|profile| profile.is_compatible_with(layout_type))
            .map(Arc::clone)
            .collect()
    }

    /// Validate a profile before saving
    fn validate_profile(&self, profile: &Profile) -> Result<(), ProfileRegistryError> {
        // Check name is not empty
        if profile.name.trim().is_empty() {
            return Err(ProfileRegistryError::Validation(
                "Profile name cannot be empty".to_string(),
            ));
        }

        // Check ID is valid UUID format
        if uuid::Uuid::parse_str(&profile.id).is_err() {
            return Err(ProfileRegistryError::Validation(
                "Profile ID must be a valid UUID".to_string(),
            ));
        }

        // Validate timestamps are valid ISO 8601
        if chrono::DateTime::parse_from_rfc3339(&profile.created_at).is_err() {
            return Err(ProfileRegistryError::Validation(
                "Invalid created_at timestamp".to_string(),
            ));
        }

        if chrono::DateTime::parse_from_rfc3339(&profile.updated_at).is_err() {
            return Err(ProfileRegistryError::Validation(
                "Invalid updated_at timestamp".to_string(),
            ));
        }

        Ok(())
    }

    /// Invalidate the cache for a specific profile (useful after external modifications)
    async fn invalidate_cache(&self, profile_id: &str) {
        let mut cache = self.cache.write().await;
        cache.remove(profile_id);
    }

    /// Clear the entire cache
    async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Get the number of profiles in cache
    async fn cache_size(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drivers::keycodes::KeyCode;
    use crate::registry::profile::storage::ProfileRegistryStorage;
    use crate::registry::profile::{KeyAction, PhysicalPosition};
    use serial_test::serial;
    use tempfile::tempdir;

    #[tokio::test]
    #[serial]
    async fn test_list_profiles() {
        let temp = tempdir().unwrap();
        let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

        let profile1 = Profile::new("Profile 1", LayoutType::Matrix);
        let profile2 = Profile::new("Profile 2", LayoutType::Standard);

        registry.save_profile(&profile1).await.unwrap();
        registry.save_profile(&profile2).await.unwrap();

        let list = registry.list_profiles().await;
        assert_eq!(list.len(), 2);
        assert!(list.contains(&profile1.id));
        assert!(list.contains(&profile2.id));
    }

    #[tokio::test]
    #[serial]
    async fn test_find_compatible_profiles() {
        let temp = tempdir().unwrap();
        let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

        let matrix_profile = Profile::new("Matrix Profile", LayoutType::Matrix);
        let standard_profile = Profile::new("Standard Profile", LayoutType::Standard);

        registry.save_profile(&matrix_profile).await.unwrap();
        registry.save_profile(&standard_profile).await.unwrap();

        let compatible = registry.find_compatible_profiles(&LayoutType::Matrix).await;
        assert_eq!(compatible.len(), 1);
        assert_eq!(compatible[0].id, matrix_profile.id);
    }

    #[tokio::test]
    #[serial]
    async fn test_validation_empty_name() {
        let temp = tempdir().unwrap();
        let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

        let mut profile = Profile::new("Test", LayoutType::Matrix);
        profile.name = "".to_string();

        let result = registry.save_profile(&profile).await;
        assert!(matches!(result, Err(ProfileRegistryError::Validation(_))));
    }

    #[tokio::test]
    #[serial]
    async fn test_cache_invalidation() {
        let temp = tempdir().unwrap();
        let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

        let profile = Profile::new("Test", LayoutType::Matrix);
        registry.save_profile(&profile).await.unwrap();

        // Profile should be in cache
        assert_eq!(registry.cache_size().await, 1);

        // Invalidate cache
        registry.invalidate_cache(&profile.id).await;
        assert_eq!(registry.cache_size().await, 0);

        // Should still be able to load from disk
        let loaded = registry.get_profile(&profile.id).await.unwrap();
        assert_eq!(loaded.id, profile.id);
        assert_eq!(registry.cache_size().await, 1);
    }

    #[tokio::test]
    #[serial]
    async fn test_arc_sharing() {
        let temp = tempdir().unwrap();
        let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

        let profile = Profile::new("Test", LayoutType::Matrix);
        registry.save_profile(&profile).await.unwrap();

        // Get the same profile multiple times
        let arc1 = registry.get_profile(&profile.id).await.unwrap();
        let arc2 = registry.get_profile(&profile.id).await.unwrap();

        // They should point to the same allocation (Arc::ptr_eq)
        assert!(Arc::ptr_eq(&arc1, &arc2));
    }
}
