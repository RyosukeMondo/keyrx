//! Profile resolution for the revolutionary mapping pipeline.
//!
//! The ProfileResolver provides fast, cached access to profiles for the event
//! processing pipeline. It wraps the ProfileRegistry and maintains an optimized
//! cache layer to meet the <100μs lookup target for hot paths.

use crate::registry::profile::{Profile, ProfileId, ProfileRegistry, ProfileRegistryError};
use crate::registry::ProfileRegistryResolution;
use std::sync::Arc;
#[cfg(debug_assertions)]
use std::time::Instant;
use thiserror::Error;

/// Error type for profile resolution
#[derive(Debug, Error)]
pub enum ProfileResolverError {
    #[error("Profile not found: {0}")]
    NotFound(ProfileId),

    #[error("Registry error: {0}")]
    RegistryError(#[from] ProfileRegistryError),
}

/// Resolves profile IDs to Profile instances with caching.
///
/// The ProfileResolver is optimized for the hot path in the event processing
/// pipeline. It wraps the ProfileRegistry and uses Arc<Profile> for zero-copy
/// sharing across the pipeline stages.
///
/// # Performance Characteristics
///
/// - Cold lookup (profile not in cache): <10ms (disk I/O + parse)
/// - Warm lookup (profile in cache): <100μs (Arc clone + HashMap lookup)
/// - Cache invalidation: O(1) removal from cache
///
/// The ProfileRegistry already maintains an internal cache using
/// Arc<RwLock<HashMap>>, so this resolver simply provides a convenient
/// interface for the pipeline with explicit invalidation semantics.
#[derive(Clone)]
pub struct ProfileResolver {
    /// Reference to the profile registry
    registry: Arc<ProfileRegistry>,
}

impl ProfileResolver {
    /// Create a new ProfileResolver wrapping the given registry.
    pub fn new(registry: Arc<ProfileRegistry>) -> Self {
        Self { registry }
    }

    /// Resolve a profile ID to a Profile instance.
    ///
    /// This method leverages the ProfileRegistry's internal cache:
    /// - First lookup hits the cache (warm path: <100μs)
    /// - Cache misses load from disk (cold path: <10ms)
    /// - Results are cached automatically for subsequent lookups
    ///
    /// Returns an Arc<Profile> for zero-copy sharing across pipeline stages.
    ///
    /// # Errors
    ///
    /// Returns `ProfileResolverError::NotFound` if the profile doesn't exist.
    /// Returns `ProfileResolverError::RegistryError` for I/O or serialization errors.
    ///
    /// # Performance
    ///
    /// This method meets the <100μs latency target for cached profiles.
    /// Cold loads may take up to 10ms but are rare (only on first access).
    pub async fn resolve(&self, profile_id: &str) -> Result<Arc<Profile>, ProfileResolverError> {
        #[cfg(debug_assertions)]
        let start = Instant::now();

        let profile = self
            .registry
            .get_profile(profile_id)
            .await
            .map_err(|e| match e {
                ProfileRegistryError::NotFound(id) => ProfileResolverError::NotFound(id),
                other => ProfileResolverError::RegistryError(other),
            })?;

        #[cfg(debug_assertions)]
        {
            let elapsed = start.elapsed();
            if elapsed.as_micros() > 100 {
                tracing::debug!(
                    service = "keyrx",
                    event = "slow_profile_lookup",
                    component = "profile_resolver",
                    profile_id = %profile_id,
                    latency_us = elapsed.as_micros(),
                    "Profile lookup exceeded 100μs target (likely cold load)"
                );
            }
        }

        Ok(profile)
    }

    /// Invalidate the cache for a specific profile.
    ///
    /// Call this when a profile has been updated externally or when you want
    /// to force a reload from disk on the next access.
    ///
    /// This is a fast operation (O(1) HashMap removal).
    pub async fn invalidate_cache(&self, profile_id: &str) {
        self.registry.invalidate_cache(profile_id).await;

        tracing::debug!(
            service = "keyrx",
            event = "profile_cache_invalidated",
            component = "profile_resolver",
            profile_id = %profile_id,
            "Profile cache entry invalidated"
        );
    }

    /// Invalidate all cached profiles.
    ///
    /// Use this when you need to force a complete cache refresh, for example
    /// after a bulk profile update or migration.
    pub async fn invalidate_all(&self) {
        self.registry.clear_cache().await;

        tracing::debug!(
            service = "keyrx",
            event = "profile_cache_cleared",
            component = "profile_resolver",
            "All profile cache entries cleared"
        );
    }

    /// Get cache statistics.
    ///
    /// Returns the number of profiles currently in the cache.
    /// Useful for monitoring and diagnostics.
    pub async fn cache_size(&self) -> usize {
        self.registry.cache_size().await
    }

    /// Get a reference to the underlying registry.
    ///
    /// This allows access to other registry operations like save, delete, etc.
    /// when needed outside the hot path.
    pub fn registry(&self) -> &Arc<ProfileRegistry> {
        &self.registry
    }
}

impl std::fmt::Debug for ProfileResolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProfileResolver")
            .field("registry", &"ProfileRegistry")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drivers::keycodes::KeyCode;
    use crate::registry::profile::{LayoutType, PhysicalPosition};
    use crate::registry::ProfileRegistryStorage;
    use serial_test::serial;
    use tempfile::tempdir;

    #[tokio::test]
    #[serial]
    async fn test_resolve_existing_profile() {
        let temp = tempdir().unwrap();
        let registry = Arc::new(ProfileRegistry::with_directory(temp.path().to_path_buf()));
        let resolver = ProfileResolver::new(registry.clone());

        // Create and save a profile
        let profile = Profile::new("Test Profile", LayoutType::Matrix);
        let profile_id = profile.id.clone();
        registry.save_profile(&profile).await.unwrap();

        // Resolve should find it
        let resolved = resolver.resolve(&profile_id).await.unwrap();
        assert_eq!(resolved.id, profile_id);
        assert_eq!(resolved.name, "Test Profile");
        assert_eq!(resolved.layout_type, LayoutType::Matrix);
    }

    #[tokio::test]
    #[serial]
    async fn test_resolve_nonexistent_profile() {
        let temp = tempdir().unwrap();
        let registry = Arc::new(ProfileRegistry::with_directory(temp.path().to_path_buf()));
        let resolver = ProfileResolver::new(registry);

        // Try to resolve a profile that doesn't exist
        let result = resolver.resolve("nonexistent-id").await;
        assert!(matches!(result, Err(ProfileResolverError::NotFound(_))));
    }

    #[tokio::test]
    #[serial]
    async fn test_resolve_uses_cache() {
        let temp = tempdir().unwrap();
        let registry = Arc::new(ProfileRegistry::with_directory(temp.path().to_path_buf()));
        let resolver = ProfileResolver::new(registry.clone());

        // Create and save a profile
        let profile = Profile::new("Cached Profile", LayoutType::Standard);
        let profile_id = profile.id.clone();
        registry.save_profile(&profile).await.unwrap();

        // First resolve (cold - loads from disk)
        let resolved1 = resolver.resolve(&profile_id).await.unwrap();

        // Second resolve (warm - should hit cache)
        let resolved2 = resolver.resolve(&profile_id).await.unwrap();

        // Both should point to the same Arc allocation
        assert!(Arc::ptr_eq(&resolved1, &resolved2));
    }

    #[tokio::test]
    #[serial]
    async fn test_invalidate_cache_single_profile() {
        let temp = tempdir().unwrap();
        let registry = Arc::new(ProfileRegistry::with_directory(temp.path().to_path_buf()));
        let resolver = ProfileResolver::new(registry.clone());

        // Create and save a profile
        let profile = Profile::new("Test", LayoutType::Matrix);
        let profile_id = profile.id.clone();
        registry.save_profile(&profile).await.unwrap();

        // Load into cache
        let _resolved1 = resolver.resolve(&profile_id).await.unwrap();
        assert_eq!(resolver.cache_size().await, 1);

        // Invalidate
        resolver.invalidate_cache(&profile_id).await;
        assert_eq!(resolver.cache_size().await, 0);

        // Should still be able to resolve (loads from disk again)
        let resolved2 = resolver.resolve(&profile_id).await.unwrap();
        assert_eq!(resolved2.id, profile_id);
    }

    #[tokio::test]
    #[serial]
    async fn test_invalidate_all() {
        let temp = tempdir().unwrap();
        let registry = Arc::new(ProfileRegistry::with_directory(temp.path().to_path_buf()));
        let resolver = ProfileResolver::new(registry.clone());

        // Create and save multiple profiles
        let profile1 = Profile::new("Profile 1", LayoutType::Matrix);
        let profile2 = Profile::new("Profile 2", LayoutType::Standard);
        let id1 = profile1.id.clone();
        let id2 = profile2.id.clone();

        registry.save_profile(&profile1).await.unwrap();
        registry.save_profile(&profile2).await.unwrap();

        // Load both into cache
        let _p1 = resolver.resolve(&id1).await.unwrap();
        let _p2 = resolver.resolve(&id2).await.unwrap();
        assert_eq!(resolver.cache_size().await, 2);

        // Invalidate all
        resolver.invalidate_all().await;
        assert_eq!(resolver.cache_size().await, 0);

        // Both should still be resolvable
        let _p1_again = resolver.resolve(&id1).await.unwrap();
        let _p2_again = resolver.resolve(&id2).await.unwrap();
        assert_eq!(resolver.cache_size().await, 2);
    }

    #[tokio::test]
    #[serial]
    async fn test_cache_size() {
        let temp = tempdir().unwrap();
        let registry = Arc::new(ProfileRegistry::with_directory(temp.path().to_path_buf()));
        let resolver = ProfileResolver::new(registry.clone());

        assert_eq!(resolver.cache_size().await, 0);

        // Add profiles
        let profile1 = Profile::new("P1", LayoutType::Matrix);
        let profile2 = Profile::new("P2", LayoutType::Standard);
        let profile3 = Profile::new("P3", LayoutType::Split);

        registry.save_profile(&profile1).await.unwrap();
        registry.save_profile(&profile2).await.unwrap();
        registry.save_profile(&profile3).await.unwrap();

        // Note: ProfileRegistry automatically caches profiles on save,
        // so cache size is already 3
        assert_eq!(resolver.cache_size().await, 3);

        // Clear cache
        resolver.invalidate_all().await;
        assert_eq!(resolver.cache_size().await, 0);

        // Resolve them to load into cache
        let _p1 = resolver.resolve(&profile1.id).await.unwrap();
        assert_eq!(resolver.cache_size().await, 1);

        let _p2 = resolver.resolve(&profile2.id).await.unwrap();
        assert_eq!(resolver.cache_size().await, 2);

        let _p3 = resolver.resolve(&profile3.id).await.unwrap();
        assert_eq!(resolver.cache_size().await, 3);
    }

    #[tokio::test]
    #[serial]
    async fn test_concurrent_resolution() {
        let temp = tempdir().unwrap();
        let registry = Arc::new(ProfileRegistry::with_directory(temp.path().to_path_buf()));
        let resolver = ProfileResolver::new(registry.clone());

        // Create a profile
        let profile = Profile::new("Concurrent Test", LayoutType::Matrix);
        let profile_id = profile.id.clone();
        registry.save_profile(&profile).await.unwrap();

        // Spawn multiple concurrent resolve operations
        let mut handles = vec![];
        for _ in 0..20 {
            let resolver_clone = resolver.clone();
            let id_clone = profile_id.clone();
            let handle = tokio::spawn(async move { resolver_clone.resolve(&id_clone).await });
            handles.push(handle);
        }

        // All should succeed and return the same profile
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
            let resolved = result.unwrap();
            assert_eq!(resolved.id, profile_id);
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_zero_copy_sharing() {
        let temp = tempdir().unwrap();
        let registry = Arc::new(ProfileRegistry::with_directory(temp.path().to_path_buf()));
        let resolver = ProfileResolver::new(registry.clone());

        // Create a profile
        let mut profile = Profile::new("Zero Copy Test", LayoutType::Matrix);
        profile.set_action(
            PhysicalPosition::new(0, 0),
            crate::registry::profile::KeyAction::key(KeyCode::A),
        );
        let profile_id = profile.id.clone();
        registry.save_profile(&profile).await.unwrap();

        // Resolve multiple times
        let arc1 = resolver.resolve(&profile_id).await.unwrap();
        let arc2 = resolver.resolve(&profile_id).await.unwrap();
        let arc3 = resolver.resolve(&profile_id).await.unwrap();

        // All should point to the same allocation (zero-copy)
        assert!(Arc::ptr_eq(&arc1, &arc2));
        assert!(Arc::ptr_eq(&arc2, &arc3));
    }

    #[tokio::test]
    #[serial]
    async fn test_registry_access() {
        let temp = tempdir().unwrap();
        let registry = Arc::new(ProfileRegistry::with_directory(temp.path().to_path_buf()));
        let resolver = ProfileResolver::new(registry.clone());

        // Access registry through resolver
        let profile = Profile::new("Test", LayoutType::Matrix);
        resolver.registry().save_profile(&profile).await.unwrap();

        // Should be able to resolve it
        let resolved = resolver.resolve(&profile.id).await.unwrap();
        assert_eq!(resolved.id, profile.id);
    }

    #[tokio::test]
    #[serial]
    async fn test_clone() {
        let temp = tempdir().unwrap();
        let registry = Arc::new(ProfileRegistry::with_directory(temp.path().to_path_buf()));
        let resolver1 = ProfileResolver::new(registry.clone());
        let resolver2 = resolver1.clone();

        // Both resolvers should work independently
        let profile = Profile::new("Clone Test", LayoutType::Standard);
        let profile_id = profile.id.clone();
        registry.save_profile(&profile).await.unwrap();

        let resolved1 = resolver1.resolve(&profile_id).await.unwrap();
        let resolved2 = resolver2.resolve(&profile_id).await.unwrap();

        // Should share the same cache (same Arc)
        assert!(Arc::ptr_eq(&resolved1, &resolved2));
    }
}
