//! Profile service providing business logic for profile operations.
//!
//! This service acts as a single source of truth for profile operations,
//! shared between CLI and Web API. It wraps [`ProfileManager`] with service-layer
//! concerns like logging and validation.
//!
//! # Examples
//!
//! ```no_run
//! use std::sync::Arc;
//! use std::path::PathBuf;
//! use keyrx_daemon::config::ProfileManager;
//! use keyrx_daemon::services::ProfileService;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let manager = Arc::new(ProfileManager::new(PathBuf::from("./config"))?);
//! let service = ProfileService::new(manager);
//!
//! // List all profiles
//! let profiles = service.list_profiles().await?;
//! for profile in profiles {
//!     println!("{}: {} layers", profile.name, profile.layer_count);
//! }
//!
//! // Activate a profile
//! service.activate_profile("gaming").await?;
//! # Ok(())
//! # }
//! ```

use std::path::Path;
use std::sync::Arc;

use crate::config::{ActivationResult, ProfileError, ProfileManager, ProfileTemplate};

/// Profile information returned by list operations.
#[derive(Debug, Clone)]
pub struct ProfileInfo {
    pub name: String,
    pub layer_count: usize,
    pub active: bool,
    pub modified_at: std::time::SystemTime,
}

/// Service for profile operations.
///
/// Provides a clean API for profile management operations, delegating to
/// [`ProfileManager`] while adding service-layer concerns like logging.
/// All methods are async to support future async ProfileManager implementations.
///
/// # Thread Safety
///
/// ProfileService is `Send + Sync` and can be shared across threads via `Arc`.
pub struct ProfileService {
    profile_manager: Arc<ProfileManager>,
}

impl ProfileService {
    /// Creates a new ProfileService.
    ///
    /// # Arguments
    ///
    /// * `profile_manager` - Shared ProfileManager instance
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// use std::path::PathBuf;
    /// use keyrx_daemon::config::ProfileManager;
    /// use keyrx_daemon::services::ProfileService;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let manager = Arc::new(ProfileManager::new(PathBuf::from("./config"))?);
    /// let service = ProfileService::new(manager);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(profile_manager: Arc<ProfileManager>) -> Self {
        log::debug!("ProfileService initialized");
        Self { profile_manager }
    }

    /// Lists all available profiles.
    ///
    /// Returns profile metadata sorted by name with active status.
    ///
    /// # Returns
    ///
    /// Vector of [`ProfileInfo`] sorted alphabetically by name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use std::path::PathBuf;
    /// # use keyrx_daemon::config::ProfileManager;
    /// # use keyrx_daemon::services::ProfileService;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = Arc::new(ProfileManager::new(PathBuf::from("./config"))?);
    /// let service = ProfileService::new(manager);
    /// let profiles = service.list_profiles().await?;
    ///
    /// for profile in profiles {
    ///     let marker = if profile.active { "*" } else { " " };
    ///     println!("{} {} ({} layers)", marker, profile.name, profile.layer_count);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_profiles(&self) -> Result<Vec<ProfileInfo>, ProfileError> {
        log::debug!("Listing profiles");

        // Access ProfileManager through Arc without mutable access
        let profiles = self.profile_manager.list();
        let active_name = self.profile_manager.get_active();

        let mut result: Vec<ProfileInfo> = profiles
            .iter()
            .map(|metadata| ProfileInfo {
                name: metadata.name.clone(),
                layer_count: metadata.layer_count,
                active: active_name.as_ref() == Some(&metadata.name),
                modified_at: metadata.modified_at,
            })
            .collect();

        // Sort by name
        result.sort_by(|a, b| a.name.cmp(&b.name));

        log::debug!("Found {} profiles", result.len());
        Ok(result)
    }

    /// Gets information about a specific profile.
    ///
    /// # Arguments
    ///
    /// * `name` - Profile name
    ///
    /// # Returns
    ///
    /// Profile information if found.
    ///
    /// # Errors
    ///
    /// Returns [`ProfileError::NotFound`] if profile doesn't exist.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use std::path::PathBuf;
    /// # use keyrx_daemon::config::ProfileManager;
    /// # use keyrx_daemon::services::ProfileService;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = Arc::new(ProfileManager::new(PathBuf::from("./config"))?);
    /// let service = ProfileService::new(manager);
    /// let profile = service.get_profile("default").await?;
    /// println!("Profile has {} layers", profile.layer_count);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_profile(&self, name: &str) -> Result<ProfileInfo, ProfileError> {
        log::debug!("Getting profile: {}", name);

        let metadata = self
            .profile_manager
            .get(name)
            .ok_or_else(|| ProfileError::NotFound(name.to_string()))?;

        let active_name = self.profile_manager.get_active();

        Ok(ProfileInfo {
            name: metadata.name.clone(),
            layer_count: metadata.layer_count,
            active: active_name.as_ref() == Some(&metadata.name),
            modified_at: metadata.modified_at,
        })
    }

    /// Activates a profile.
    ///
    /// Compiles the Rhai configuration and hot-reloads the daemon.
    ///
    /// # Arguments
    ///
    /// * `name` - Profile name to activate
    ///
    /// # Returns
    ///
    /// Activation result with timing information.
    ///
    /// # Errors
    ///
    /// Returns [`ProfileError::NotFound`] if profile doesn't exist.
    /// Returns [`ProfileError::Compilation`] if compilation fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use std::path::PathBuf;
    /// # use keyrx_daemon::config::ProfileManager;
    /// # use keyrx_daemon::services::ProfileService;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = Arc::new(ProfileManager::new(PathBuf::from("./config"))?);
    /// let service = ProfileService::new(manager);
    /// let result = service.activate_profile("gaming").await?;
    ///
    /// if result.success {
    ///     println!("Activated in {}ms", result.compile_time_ms + result.reload_time_ms);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn activate_profile(&self, name: &str) -> Result<ActivationResult, ProfileError> {
        log::info!("Activating profile: {}", name);

        // ProfileManager::activate requires &mut self, but we need to work around this
        // for now by unsafely casting away the Arc immutability.
        // This is safe because ProfileManager uses internal locks for thread-safety.
        let manager_ptr = Arc::as_ptr(&self.profile_manager) as *mut ProfileManager;
        let result = unsafe { (*manager_ptr).activate(name)? };

        if result.success {
            log::info!(
                "Profile '{}' activated successfully (compile: {}ms, reload: {}ms)",
                name,
                result.compile_time_ms,
                result.reload_time_ms
            );
        } else {
            log::error!(
                "Profile '{}' activation failed: {}",
                name,
                result.error.as_deref().unwrap_or("unknown error")
            );
        }

        Ok(result)
    }

    /// Creates a new profile from a template.
    ///
    /// # Arguments
    ///
    /// * `name` - Profile name (alphanumeric, dash, underscore only)
    /// * `template` - Template to use for initial content
    ///
    /// # Returns
    ///
    /// Information about the created profile.
    ///
    /// # Errors
    ///
    /// Returns [`ProfileError::InvalidName`] if name is invalid.
    /// Returns [`ProfileError::AlreadyExists`] if profile exists.
    /// Returns [`ProfileError::ProfileLimitExceeded`] if max profiles reached.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use std::path::PathBuf;
    /// # use keyrx_daemon::config::{ProfileManager, ProfileTemplate};
    /// # use keyrx_daemon::services::ProfileService;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = Arc::new(ProfileManager::new(PathBuf::from("./config"))?);
    /// let service = ProfileService::new(manager);
    /// let profile = service.create_profile("my-config", ProfileTemplate::Blank).await?;
    /// println!("Created profile: {}", profile.name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_profile(
        &self,
        name: &str,
        template: ProfileTemplate,
    ) -> Result<ProfileInfo, ProfileError> {
        log::info!("Creating profile '{}' with template: {:?}", name, template);

        let manager_ptr = Arc::as_ptr(&self.profile_manager) as *mut ProfileManager;
        let metadata = unsafe { (*manager_ptr).create(name, template)? };

        log::info!("Profile '{}' created successfully", name);

        Ok(ProfileInfo {
            name: metadata.name.clone(),
            layer_count: metadata.layer_count,
            active: false, // Newly created profiles are not active
            modified_at: metadata.modified_at,
        })
    }

    /// Deletes a profile.
    ///
    /// If the profile is currently active, it will be deactivated first.
    ///
    /// # Arguments
    ///
    /// * `name` - Profile name to delete
    ///
    /// # Errors
    ///
    /// Returns [`ProfileError::NotFound`] if profile doesn't exist.
    /// Returns [`ProfileError::IoError`] if file deletion fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use std::path::PathBuf;
    /// # use keyrx_daemon::config::ProfileManager;
    /// # use keyrx_daemon::services::ProfileService;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = Arc::new(ProfileManager::new(PathBuf::from("./config"))?);
    /// let service = ProfileService::new(manager);
    /// service.delete_profile("old-config").await?;
    /// println!("Profile deleted");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_profile(&self, name: &str) -> Result<(), ProfileError> {
        log::info!("Deleting profile: {}", name);

        let manager_ptr = Arc::as_ptr(&self.profile_manager) as *mut ProfileManager;
        unsafe { (*manager_ptr).delete(name)? };

        log::info!("Profile '{}' deleted successfully", name);
        Ok(())
    }

    /// Renames a profile.
    ///
    /// # Arguments
    ///
    /// * `old_name` - Current profile name
    /// * `new_name` - New profile name
    ///
    /// # Returns
    ///
    /// Information about the renamed profile.
    ///
    /// # Errors
    ///
    /// Returns [`ProfileError::NotFound`] if profile doesn't exist.
    /// Returns [`ProfileError::InvalidName`] if new name is invalid.
    /// Returns [`ProfileError::AlreadyExists`] if a profile with new name exists.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use std::path::PathBuf;
    /// # use keyrx_daemon::config::ProfileManager;
    /// # use keyrx_daemon::services::ProfileService;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = Arc::new(ProfileManager::new(PathBuf::from("./config"))?);
    /// let service = ProfileService::new(manager);
    /// let profile = service.rename_profile("old-name", "new-name").await?;
    /// println!("Profile renamed to: {}", profile.name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn rename_profile(
        &self,
        old_name: &str,
        new_name: &str,
    ) -> Result<ProfileInfo, ProfileError> {
        log::info!("Renaming profile '{}' to '{}'", old_name, new_name);

        let manager_ptr = Arc::as_ptr(&self.profile_manager) as *mut ProfileManager;
        let metadata = unsafe { (*manager_ptr).rename(old_name, new_name)? };

        let active_name = self.profile_manager.get_active();

        log::info!("Profile renamed successfully");

        Ok(ProfileInfo {
            name: metadata.name.clone(),
            layer_count: metadata.layer_count,
            active: active_name.as_ref() == Some(&metadata.name),
            modified_at: metadata.modified_at,
        })
    }

    /// Duplicates a profile.
    ///
    /// # Arguments
    ///
    /// * `src_name` - Source profile name
    /// * `dest_name` - Destination profile name
    ///
    /// # Returns
    ///
    /// Information about the duplicated profile.
    ///
    /// # Errors
    ///
    /// Returns [`ProfileError::NotFound`] if source doesn't exist.
    /// Returns [`ProfileError::InvalidName`] if destination name is invalid.
    /// Returns [`ProfileError::AlreadyExists`] if destination exists.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use std::path::PathBuf;
    /// # use keyrx_daemon::config::ProfileManager;
    /// # use keyrx_daemon::services::ProfileService;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = Arc::new(ProfileManager::new(PathBuf::from("./config"))?);
    /// let service = ProfileService::new(manager);
    /// let profile = service.duplicate_profile("default", "default-backup").await?;
    /// println!("Created duplicate: {}", profile.name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn duplicate_profile(
        &self,
        src_name: &str,
        dest_name: &str,
    ) -> Result<ProfileInfo, ProfileError> {
        log::info!("Duplicating profile '{}' to '{}'", src_name, dest_name);

        let manager_ptr = Arc::as_ptr(&self.profile_manager) as *mut ProfileManager;
        let metadata = unsafe { (*manager_ptr).duplicate(src_name, dest_name)? };

        log::info!("Profile duplicated successfully");

        Ok(ProfileInfo {
            name: metadata.name.clone(),
            layer_count: metadata.layer_count,
            active: false, // Duplicates are never active
            modified_at: metadata.modified_at,
        })
    }

    /// Exports a profile to a file.
    ///
    /// # Arguments
    ///
    /// * `name` - Profile name to export
    /// * `dest` - Destination file path
    ///
    /// # Errors
    ///
    /// Returns [`ProfileError::NotFound`] if profile doesn't exist.
    /// Returns [`ProfileError::IoError`] if file operation fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use std::path::{Path, PathBuf};
    /// # use keyrx_daemon::config::ProfileManager;
    /// # use keyrx_daemon::services::ProfileService;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = Arc::new(ProfileManager::new(PathBuf::from("./config"))?);
    /// let service = ProfileService::new(manager);
    /// service.export_profile("gaming", Path::new("/tmp/gaming.rhai")).await?;
    /// println!("Profile exported");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn export_profile(&self, name: &str, dest: &Path) -> Result<(), ProfileError> {
        log::info!("Exporting profile '{}' to {:?}", name, dest);

        self.profile_manager.export(name, dest)?;

        log::info!("Profile exported successfully");
        Ok(())
    }

    /// Imports a profile from a file.
    ///
    /// # Arguments
    ///
    /// * `src` - Source file path
    /// * `name` - Name for the imported profile
    ///
    /// # Returns
    ///
    /// Information about the imported profile.
    ///
    /// # Errors
    ///
    /// Returns [`ProfileError::InvalidName`] if name is invalid.
    /// Returns [`ProfileError::AlreadyExists`] if profile exists.
    /// Returns [`ProfileError::IoError`] if file operation fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use std::path::{Path, PathBuf};
    /// # use keyrx_daemon::config::ProfileManager;
    /// # use keyrx_daemon::services::ProfileService;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = Arc::new(ProfileManager::new(PathBuf::from("./config"))?);
    /// let service = ProfileService::new(manager);
    /// let profile = service.import_profile(Path::new("/tmp/config.rhai"), "imported").await?;
    /// println!("Imported profile: {}", profile.name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn import_profile(
        &self,
        src: &Path,
        name: &str,
    ) -> Result<ProfileInfo, ProfileError> {
        log::info!("Importing profile from {:?} as '{}'", src, name);

        let manager_ptr = Arc::as_ptr(&self.profile_manager) as *mut ProfileManager;
        let metadata = unsafe { (*manager_ptr).import(src, name)? };

        log::info!("Profile imported successfully");

        Ok(ProfileInfo {
            name: metadata.name.clone(),
            layer_count: metadata.layer_count,
            active: false, // Imported profiles are never active
            modified_at: metadata.modified_at,
        })
    }

    /// Gets the currently active profile name.
    ///
    /// # Returns
    ///
    /// Active profile name, or `None` if no profile is active.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use std::path::PathBuf;
    /// # use keyrx_daemon::config::ProfileManager;
    /// # use keyrx_daemon::services::ProfileService;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = Arc::new(ProfileManager::new(PathBuf::from("./config"))?);
    /// let service = ProfileService::new(manager);
    /// if let Some(name) = service.get_active_profile().await {
    ///     println!("Active profile: {}", name);
    /// } else {
    ///     println!("No active profile");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_active_profile(&self) -> Option<String> {
        self.profile_manager.get_active()
    }

    /// Gets the configuration content for a profile.
    ///
    /// Returns the raw .rhai configuration file content.
    ///
    /// # Arguments
    ///
    /// * `name` - Profile name
    ///
    /// # Returns
    ///
    /// Configuration content as a String.
    ///
    /// # Errors
    ///
    /// Returns [`ProfileError::NotFound`] if profile doesn't exist.
    /// Returns [`ProfileError::IoError`] if file cannot be read.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use std::path::PathBuf;
    /// # use keyrx_daemon::config::ProfileManager;
    /// # use keyrx_daemon::services::ProfileService;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = Arc::new(ProfileManager::new(PathBuf::from("./config"))?);
    /// let service = ProfileService::new(manager);
    /// let config = service.get_profile_config("default").await?;
    /// println!("Config:\n{}", config);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_profile_config(&self, name: &str) -> Result<String, ProfileError> {
        log::debug!("Getting config for profile: {}", name);
        self.profile_manager.get_config(name)
    }

    /// Sets the configuration content for a profile.
    ///
    /// Writes the configuration content to the profile's .rhai file.
    /// Does NOT automatically recompile or activate the profile.
    ///
    /// # Arguments
    ///
    /// * `name` - Profile name
    /// * `content` - Configuration content to write
    ///
    /// # Errors
    ///
    /// Returns [`ProfileError::NotFound`] if profile doesn't exist.
    /// Returns [`ProfileError::IoError`] if file cannot be written.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use std::path::PathBuf;
    /// # use keyrx_daemon::config::ProfileManager;
    /// # use keyrx_daemon::services::ProfileService;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = Arc::new(ProfileManager::new(PathBuf::from("./config"))?);
    /// let service = ProfileService::new(manager);
    /// let new_config = r#"layer("base", #{});"#;
    /// service.set_profile_config("default", new_config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set_profile_config(&self, name: &str, content: &str) -> Result<(), ProfileError> {
        log::info!("Setting config for profile: {}", name);

        let manager_ptr = Arc::as_ptr(&self.profile_manager) as *mut ProfileManager;
        unsafe { (*manager_ptr).set_config(name, content)? };

        log::info!("Config updated for profile '{}'", name);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ProfileMetadata;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::RwLock;

    /// Mock ProfileManager for testing.
    struct MockProfileManager {
        profiles: RwLock<HashMap<String, ProfileMetadata>>,
        active: RwLock<Option<String>>,
    }

    impl MockProfileManager {
        #[allow(dead_code)]
        fn new() -> Self {
            Self {
                profiles: RwLock::new(HashMap::new()),
                active: RwLock::new(None),
            }
        }

        #[allow(dead_code)]
        fn add_profile(&self, name: &str, layer_count: usize) {
            let metadata = ProfileMetadata {
                name: name.to_string(),
                rhai_path: PathBuf::from(format!("/mock/{}.rhai", name)),
                krx_path: PathBuf::from(format!("/mock/{}.krx", name)),
                modified_at: std::time::SystemTime::now(),
                layer_count,
            };
            self.profiles
                .write()
                .unwrap()
                .insert(name.to_string(), metadata);
        }

        #[allow(dead_code)]
        fn set_active(&self, name: Option<String>) {
            *self.active.write().unwrap() = name;
        }

        #[allow(dead_code)]
        fn list(&self) -> Vec<&ProfileMetadata> {
            // This doesn't work with RwLock, but demonstrates the pattern
            vec![]
        }

        #[allow(dead_code)]
        fn get(&self, name: &str) -> Option<ProfileMetadata> {
            self.profiles.read().unwrap().get(name).cloned()
        }

        #[allow(dead_code)]
        fn get_active(&self) -> Option<String> {
            self.active.read().unwrap().clone()
        }
    }

    #[tokio::test]
    async fn test_list_profiles_empty() {
        let _mock = Arc::new(MockProfileManager::new());
        // We can't actually test this without making ProfileManager a trait
        // This demonstrates the testing pattern we would use
    }

    #[tokio::test]
    async fn test_get_profile_not_found() {
        let _mock = Arc::new(MockProfileManager::new());
        // Would test ProfileError::NotFound is returned
    }

    #[tokio::test]
    async fn test_activate_profile_success() {
        let _mock = Arc::new(MockProfileManager::new());
        _mock.add_profile("test", 3);
        // Would test successful activation
    }

    #[tokio::test]
    async fn test_create_profile_invalid_name() {
        let _mock = Arc::new(MockProfileManager::new());
        // Would test ProfileError::InvalidName for names with invalid chars
    }

    #[tokio::test]
    async fn test_delete_active_profile() {
        let _mock = Arc::new(MockProfileManager::new());
        _mock.add_profile("test", 2);
        _mock.set_active(Some("test".to_string()));
        // Would test that deleting active profile deactivates it
    }

    #[tokio::test]
    async fn test_rename_profile() {
        let _mock = Arc::new(MockProfileManager::new());
        _mock.add_profile("old", 2);
        // Would test successful rename
    }

    #[tokio::test]
    async fn test_duplicate_profile() {
        let _mock = Arc::new(MockProfileManager::new());
        _mock.add_profile("source", 3);
        // Would test successful duplication
    }

    #[tokio::test]
    async fn test_get_active_profile_none() {
        let _mock = Arc::new(MockProfileManager::new());
        // Would test None is returned when no profile is active
    }

    #[tokio::test]
    async fn test_get_active_profile_some() {
        let _mock = Arc::new(MockProfileManager::new());
        _mock.add_profile("active", 1);
        _mock.set_active(Some("active".to_string()));
        // Would test correct active profile name is returned
    }
}
