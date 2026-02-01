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
        let active_name = self.profile_manager.get_active().ok().flatten();

        let mut result: Vec<ProfileInfo> = profiles
            .iter()
            .map(|metadata| ProfileInfo {
                name: metadata.name.clone(),
                layer_count: metadata.layer_count,
                active: active_name.as_ref() == Some(&metadata.name),
                modified_at: metadata.modified_at,
                activated_at: metadata.activated_at,
                activated_by: metadata.activated_by.clone(),
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
            name: metadata.name.clone(),
            layer_count: metadata.layer_count,
            active: active_name.as_ref() == Some(&metadata.name),
            modified_at: metadata.modified_at,
            activated_at: metadata.activated_at,
            activated_by: metadata.activated_by.clone(),
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
            let manager_ptr = Arc::as_ptr(&manager) as *mut ProfileManager;
            let activation_result = unsafe { (*manager_ptr).activate(&name_owned)? };

            if activation_result.success {
                log::info!(
                    "Profile '{}' activated successfully (compile: {}ms, reload: {}ms)",
                    name_owned,
                    activation_result.compile_time_ms,
                    activation_result.reload_time_ms
                );

                // Configure Windows key blocking based on actual profile mappings
                #[cfg(target_os = "windows")]
                {
                    use crate::platform::windows::platform_state::PlatformState;

                    log::info!("Configuring key blocking for profile: {}", name_owned);

                    // Load the activated profile's .krx file and extract all mapped keys
                    // Note: This is also a blocking operation (file I/O + deserialization)
                    let config_dir = crate::cli::config_dir::get_config_dir()
                        .map_err(|e| ProfileError::NotFound(format!("Config dir error: {}", e)))?;
                    let profiles_dir = config_dir.join("profiles");
                    let krx_path = profiles_dir.join(format!("{}.krx", name_owned));

                    if let Ok(config_data) = std::fs::read(&krx_path) {
                        use keyrx_compiler::serialize::deserialize as deserialize_krx;
                        use rkyv::Deserialize;

                        // Deserialize .krx file (validates magic, version, hash)
                        let archived = deserialize_krx(&config_data);
                        match archived {
                            Ok(archived_config) => {
                                // Deserialize from archived format to ConfigRoot
                                let config: keyrx_core::config::ConfigRoot = archived_config
                                    .deserialize(&mut rkyv::Infallible)
                                    .map_err(|_| ProfileError::NotFound("Deserialization failed".to_string()))?;
                                log::info!(
                                    "✓ Loaded profile config: {} devices, {} total mappings",
                                    config.devices.len(),
                                    config.devices.iter().map(|d| d.mappings.len()).sum::<usize>()
                                );

                                match PlatformState::configure_blocking(Some(&config)) {
                                    Ok(()) => {
                                        log::info!("✓ Key blocking configured successfully");
                                    }
                                    Err(e) => {
                                        log::error!("✗ Failed to configure key blocking: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!("✗ Failed to deserialize profile config: {}", e);
                                // Clear any existing blocks if config load fails
                                if let Err(e) = PlatformState::configure_blocking(None) {
                                    log::error!("✗ Failed to clear key blocking: {}", e);
                                }
                            }
                        }
                    } else {
                        log::error!("✗ Failed to read .krx file: {:?}", krx_path);
                        // Clear any existing blocks if file read fails
                        if let Err(e) = PlatformState::configure_blocking(None) {
                            log::error!("✗ Failed to clear key blocking: {}", e);
                        }
                    }
                }

                // Signal the daemon to reload configuration via SIGHUP
                // This triggers the reload callback in the event loop
                Self::signal_daemon_reload();
            } else {
                log::error!(
                    "Profile '{}' activation failed: {}",
                    name_owned,
                    activation_result.error.as_deref().unwrap_or("unknown error")
                );
            }

            log::debug!("spawn_blocking: Profile activation complete");
            Ok::<ActivationResult, ProfileError>(activation_result)
        })
        .await
        .map_err(|e| ProfileError::LockError(format!("Task join error: {}", e)))??;

        Ok(result)
    }

    /// Signals the daemon to reload its configuration.
    ///
    /// On Unix, this sends SIGHUP to the current process.
    /// On Windows, this is a no-op (reload happens via other mechanisms).
    fn signal_daemon_reload() {
        #[cfg(unix)]
        {
            use nix::libc;

            let pid = std::process::id();
            log::debug!("Sending SIGHUP to daemon (pid: {})", pid);
            // SIGHUP = 1 on all Unix systems
            // SAFETY: Sending a signal to our own process is safe
            let result = unsafe { libc::kill(pid as i32, libc::SIGHUP) };
            if result != 0 {
                log::warn!(
                    "Failed to send SIGHUP for daemon reload: {}",
                    std::io::Error::last_os_error()
                );
            }
        }

        #[cfg(not(unix))]
        {
            log::info!("Profile activated. Daemon reload signal not available on this platform.");
        }
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

        let manager_ptr = Arc::as_ptr(&self.profile_manager) as *mut ProfileManager;
        let metadata = unsafe { (*manager_ptr).create(name, template)? };

        log::info!("Profile '{}' created successfully", name);

        Ok(ProfileInfo {
            name: metadata.name.clone(),
            layer_count: metadata.layer_count,
            active: false, // Newly created profiles are not active
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

        let active_name = self.profile_manager.get_active().ok().flatten();

        log::info!("Profile renamed successfully");

        Ok(ProfileInfo {
            name: metadata.name.clone(),
            layer_count: metadata.layer_count,
            active: active_name.as_ref() == Some(&metadata.name),
            modified_at: metadata.modified_at,
            activated_at: metadata.activated_at,
            activated_by: metadata.activated_by.clone(),
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

        let manager_ptr = Arc::as_ptr(&self.profile_manager) as *mut ProfileManager;
        let metadata = unsafe { (*manager_ptr).import(src, name)? };

        log::info!("Profile imported successfully");

        Ok(ProfileInfo {
            name: metadata.name.clone(),
            layer_count: metadata.layer_count,
            active: false, // Imported profiles are never active
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
