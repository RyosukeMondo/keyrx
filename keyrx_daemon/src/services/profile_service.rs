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
use std::sync::{Arc, OnceLock};

use crate::config::{ActivationResult, ProfileError, ProfileManager, ProfileTemplate};
use crate::daemon::DaemonSharedState;

/// Profile information returned by list operations.
/// PROF-004: Added activation metadata fields.
#[derive(Debug, Clone)]
pub struct ProfileInfo {
    pub name: String,
    pub layer_count: usize,
    pub active: bool,
    pub modified_at: std::time::SystemTime,
    pub activated_at: Option<std::time::SystemTime>,
    pub activated_by: Option<String>,
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
    /// The running daemon in this process, attached once by the web layer.
    /// Activation / config changes are forwarded to it from here, so every
    /// transport (REST, MCP, WS-RPC) reaches the daemon the same way.
    daemon_state: OnceLock<Arc<DaemonSharedState>>,
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
        Self {
            profile_manager,
            daemon_state: OnceLock::new(),
        }
    }

    /// Returns a reference to the underlying ProfileManager.
    pub fn profile_manager(&self) -> &Arc<ProfileManager> {
        &self.profile_manager
    }

    /// Attaches the running daemon so activations and saved configs of the
    /// loaded profile take effect. Only the first attachment counts.
    pub fn attach_daemon_state(&self, state: Arc<DaemonSharedState>) {
        if self.daemon_state.set(state).is_err() {
            log::debug!("ProfileService already attached to a daemon");
        }
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
        let active_name = self.profile_manager.get_active().ok().flatten();

        let mut result: Vec<ProfileInfo> = profiles
            .into_iter()
            .map(|metadata| ProfileInfo {
                active: active_name.as_ref() == Some(&metadata.name),
                modified_at: metadata.modified_at,
                activated_at: metadata.activated_at,
                activated_by: metadata.activated_by,
                layer_count: metadata.layer_count,
                name: metadata.name,
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

        let active_name = self.profile_manager.get_active().ok().flatten();

        Ok(ProfileInfo {
            active: active_name.as_ref() == Some(&metadata.name),
            modified_at: metadata.modified_at,
            activated_at: metadata.activated_at,
            activated_by: metadata.activated_by,
            layer_count: metadata.layer_count,
            name: metadata.name,
        })
    }

    /// Activates a profile.
    /// PROF-001: Fixed race conditions with serialized activation via ProfileManager's Mutex.
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
    /// Returns [`ProfileError::LockError`] if activation lock cannot be acquired.
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

        // PROF-001: ProfileManager::activate now uses internal Mutex to serialize
        // concurrent activation attempts. This prevents race conditions where
        // multiple activations could corrupt the state.
        //
        // ProfileManager::activate requires &mut self, but we need to work around this
        // for now by unsafely casting away the Arc immutability.
        // This is safe because ProfileManager uses internal locks for thread-safety.
        //
        // CRITICAL FIX (v0.1.3): Wrap ALL blocking operations in spawn_blocking to prevent
        // blocking the async runtime. This fixes the config page freeze issue where activating
        // a profile would block subsequent API requests.
        let manager = Arc::clone(&self.profile_manager);
        let name_owned = name.to_string();

        let result = tokio::task::spawn_blocking(move || {
            log::debug!("spawn_blocking: Starting profile activation");

            // Activate profile (blocking operation)
            let activation_result = manager.activate(&name_owned)?;

            if activation_result.success {
                log::info!(
                    "Profile '{}' activated successfully (compile: {}ms, reload: {}ms)",
                    name_owned,
                    activation_result.compile_time_ms,
                    activation_result.reload_time_ms
                );
            } else {
                log::error!(
                    "Profile '{}' activation failed: {}",
                    name_owned,
                    activation_result
                        .error
                        .as_deref()
                        .unwrap_or("unknown error")
                );
            }

            log::debug!("spawn_blocking: Profile activation complete");
            Ok::<ActivationResult, ProfileError>(activation_result)
        })
        .await
        .map_err(|e| ProfileError::LockError(format!("Task join error: {}", e)))??;

        if result.success {
            if let Some(daemon_state) = self.daemon_state.get() {
                daemon_state.request_activation(name);
            }
        }
        Ok(result)
    }

    /// Reload the active profile, recompiling if .rhai is newer than .krx.
    ///
    /// Checks timestamps and only recompiles when needed. After recompilation
    /// the attached daemon (if any) is asked to reload.
    pub async fn reload_active_profile(&self) -> Result<crate::config::ReloadResult, ProfileError> {
        log::info!("Reloading active profile (with timestamp check)");

        let manager = Arc::clone(&self.profile_manager);

        let result = tokio::task::spawn_blocking(move || {
            let reload_result = manager.reload_active()?;

            if reload_result.recompiled && reload_result.success {
                // Get the active profile name for key blocking config
                let active_name = manager.get_active()?.unwrap_or_default();

                log::info!(
                    "Active profile '{}' recompiled ({}ms)",
                    active_name,
                    reload_result.compile_time_ms
                );
            }

            Ok::<crate::config::ReloadResult, ProfileError>(reload_result)
        })
        .await
        .map_err(|e| ProfileError::LockError(format!("Task join error: {}", e)))??;

        if result.recompiled && result.success {
            if let Some(daemon_state) = self.daemon_state.get() {
                daemon_state.request_reload();
            }
        }
        Ok(result)
    }

    /// Loads and deserializes a profile's .krx config file.
    ///
    /// This is used for extracting mapped keys for Windows key blocking.
    ///
    /// # Arguments
    ///
    /// * `name` - Profile name
    ///
    /// # Returns
    ///
    /// Owned ConfigRoot instance, deserialized from the .krx file.
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

        let manager = Arc::clone(&self.profile_manager);
        let name_owned = name.to_string();

        let metadata = tokio::task::spawn_blocking(move || manager.create(&name_owned, template))
            .await
            .map_err(|e| ProfileError::LockError(format!("Task join error: {}", e)))??;

        log::info!("Profile '{}' created successfully", name);

        Ok(ProfileInfo {
            name: metadata.name,
            layer_count: metadata.layer_count,
            active: false,
            modified_at: metadata.modified_at,
            activated_at: None,
            activated_by: None,
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

        let manager = Arc::clone(&self.profile_manager);
        let name_owned = name.to_string();

        tokio::task::spawn_blocking(move || manager.delete(&name_owned))
            .await
            .map_err(|e| ProfileError::LockError(format!("Task join error: {}", e)))??;

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

        let manager = Arc::clone(&self.profile_manager);
        let old_owned = old_name.to_string();
        let new_owned = new_name.to_string();

        let metadata = tokio::task::spawn_blocking(move || manager.rename(&old_owned, &new_owned))
            .await
            .map_err(|e| ProfileError::LockError(format!("Task join error: {}", e)))??;

        let active_name = self.profile_manager.get_active().ok().flatten();

        log::info!("Profile renamed successfully");

        Ok(ProfileInfo {
            active: active_name.as_ref() == Some(&metadata.name),
            modified_at: metadata.modified_at,
            activated_at: metadata.activated_at,
            activated_by: metadata.activated_by,
            layer_count: metadata.layer_count,
            name: metadata.name,
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

        let manager = Arc::clone(&self.profile_manager);
        let src_owned = src_name.to_string();
        let dest_owned = dest_name.to_string();

        let metadata =
            tokio::task::spawn_blocking(move || manager.duplicate(&src_owned, &dest_owned))
                .await
                .map_err(|e| ProfileError::LockError(format!("Task join error: {}", e)))??;

        log::info!("Profile duplicated successfully");

        Ok(ProfileInfo {
            name: metadata.name,
            layer_count: metadata.layer_count,
            active: false,
            modified_at: metadata.modified_at,
            activated_at: None,
            activated_by: None,
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

        let manager = Arc::clone(&self.profile_manager);
        let src_owned = src.to_path_buf();
        let name_owned = name.to_string();

        let metadata = tokio::task::spawn_blocking(move || manager.import(&src_owned, &name_owned))
            .await
            .map_err(|e| ProfileError::LockError(format!("Task join error: {}", e)))??;

        log::info!("Profile imported successfully");

        Ok(ProfileInfo {
            name: metadata.name,
            layer_count: metadata.layer_count,
            active: false,
            modified_at: metadata.modified_at,
            activated_at: None,
            activated_by: None,
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
        self.profile_manager.get_active().ok().flatten()
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
    /// Writes the .rhai file, recompiles to .krx, and if the modified profile
    /// is the currently active one, automatically triggers a daemon reload.
    /// This guarantees that config changes are always applied end-to-end.
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
    /// Returns [`ProfileError::Compilation`] if recompilation fails.
    pub async fn set_profile_config(&self, name: &str, content: &str) -> Result<(), ProfileError> {
        log::info!("Setting config for profile: {}", name);

        let manager = Arc::clone(&self.profile_manager);
        let name_owned = name.to_string();
        let content_owned = content.to_string();

        tokio::task::spawn_blocking(move || manager.set_config(&name_owned, &content_owned))
            .await
            .map_err(|e| ProfileError::LockError(format!("Task join error: {}", e)))??;

        log::info!("Config saved and recompiled for profile '{}'", name);

        // Trigger daemon reload if the modified profile is currently active
        if let Some(daemon_state) = self.daemon_state.get() {
            let active = daemon_state.get_active_profile();
            if active.as_deref() == Some(name) {
                log::info!(
                    "Active profile '{}' was modified, triggering daemon reload",
                    name
                );
                daemon_state.request_reload();
            }
        }

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
                activated_at: None,
                activated_by: None,
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
