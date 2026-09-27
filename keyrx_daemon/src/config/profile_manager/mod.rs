//! Profile management with hot-reload and thread-safe activation.
//!
//! This module provides the `ProfileManager` for creating, activating, and managing
//! Rhai configuration profiles with atomic hot-reload capabilities.
//!
//! Submodules split the implementation by responsibility:
//! - [`types`] — metadata, templates, results, and the shared error type
//! - [`crud`] — create/delete/duplicate/rename/import/export and templates
//! - [`activation`] — activate and reload the active profile with hot-reload
//! - [`persistence`] — the `.active` marker file across daemon restarts

mod activation;
mod crud;
mod persistence;
mod types;

pub use types::{ActivationResult, ProfileError, ProfileMetadata, ProfileTemplate, ReloadResult};

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError, RwLock};

use super::profile_compiler::ProfileCompiler;

/// Type alias for the profiles map protected by RwLock.
type ProfilesMap = RwLock<HashMap<String, ProfileMetadata>>;

/// Maximum number of profiles allowed
const MAX_PROFILES: usize = 100;

/// File name for persisting active profile
const ACTIVE_PROFILE_FILE: &str = ".active";

/// Profile manager for CRUD operations and hot-reload.
pub struct ProfileManager {
    config_dir: PathBuf,
    active_profile: Arc<RwLock<Option<String>>>,
    profiles: ProfilesMap,
    activation_lock: Arc<Mutex<()>>,
    compiler: ProfileCompiler,
}

impl ProfileManager {
    /// Path to the profiles subdirectory.
    pub fn profiles_dir(&self) -> PathBuf {
        self.config_dir.join("profiles")
    }

    /// Path to a profile's .rhai source file.
    fn rhai_path(&self, name: &str) -> PathBuf {
        self.profiles_dir().join(format!("{}.rhai", name))
    }

    /// Path to a profile's .krx compiled file.
    fn krx_path(&self, name: &str) -> PathBuf {
        self.profiles_dir().join(format!("{}.krx", name))
    }

    /// Create a new profile manager with the specified config directory.
    pub fn new(config_dir: PathBuf) -> Result<Self, ProfileError> {
        // Create config directory if it doesn't exist
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        // Create profiles subdirectory
        let profiles_dir = config_dir.join("profiles");
        if !profiles_dir.exists() {
            fs::create_dir_all(&profiles_dir)?;
        }

        let manager = Self {
            config_dir,
            active_profile: Arc::new(RwLock::new(None)),
            profiles: RwLock::new(HashMap::new()),
            activation_lock: Arc::new(Mutex::new(())),
            compiler: ProfileCompiler::new(),
        };

        // Scan for existing profiles
        manager.scan_profiles()?;

        // Restore persisted active profile if it exists
        if let Some(active_name) = manager.load_active_profile() {
            if let Ok(mut guard) = manager.active_profile.write() {
                *guard = Some(active_name);
            }
        }

        Ok(manager)
    }

    /// Scan the profiles directory for .rhai files.
    pub fn scan_profiles(&self) -> Result<(), ProfileError> {
        let profiles_dir = self.profiles_dir();
        if !profiles_dir.exists() {
            return Ok(());
        }

        let mut profiles = self.profiles.write().map_err(|e| {
            ProfileError::LockError(format!("Failed to acquire profiles write lock: {}", e))
        })?;
        profiles.clear();

        for entry in fs::read_dir(&profiles_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("rhai") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    let metadata = self.load_profile_metadata(name)?;
                    profiles.insert(name.to_string(), metadata);
                }
            }
        }

        Ok(())
    }

    /// Load metadata for a profile by name.
    /// PROF-004: Load activation metadata from .active file.
    fn load_profile_metadata(&self, name: &str) -> Result<ProfileMetadata, ProfileError> {
        let rhai_path = self.rhai_path(name);
        let krx_path = self.krx_path(name);

        if !rhai_path.exists() {
            return Err(ProfileError::NotFound(name.to_string()));
        }

        let modified_at = rhai_path.metadata()?.modified()?;

        let stats = ProfileStats::of_compiled(&krx_path);

        // PROF-004: Load activation metadata if this is the active profile
        let (activated_at, activated_by) = self.load_activation_metadata(name);

        Ok(ProfileMetadata {
            name: name.to_string(),
            rhai_path,
            krx_path,
            modified_at,
            layer_count: stats.layers,
            device_count: stats.devices,
            key_count: stats.keys,
            activated_at,
            activated_by,
        })
    }

    /// Validate profile name.
    /// PROF-002: Enhanced validation with strict regex-like rules.
    pub fn validate_name(name: &str) -> Result<(), ProfileError> {
        if name.is_empty() {
            return Err(ProfileError::InvalidName(
                "Name cannot be empty".to_string(),
            ));
        }

        if name.len() > 64 {
            return Err(ProfileError::InvalidName(format!(
                "Name too long (max 64 chars, got {})",
                name.len()
            )));
        }

        // Allow only alphanumeric, dash, underscore (^[a-zA-Z0-9_-]{1,64}$)
        if !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(ProfileError::InvalidName(
                "Name can only contain ASCII alphanumeric characters, dashes, and underscores"
                    .to_string(),
            ));
        }

        // Reject names starting with dash or underscore
        if name.starts_with('-') || name.starts_with('_') {
            return Err(ProfileError::InvalidName(
                "Name cannot start with dash or underscore".to_string(),
            ));
        }

        Ok(())
    }

    /// List all profiles.
    pub fn list(&self) -> Vec<ProfileMetadata> {
        self.profiles
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .values()
            .cloned()
            .collect()
    }

    /// Get the currently active profile name.
    ///
    /// # Errors
    ///
    /// Returns `ProfileError::LockError` if the RwLock is poisoned.
    pub fn get_active(&self) -> Result<Option<String>, ProfileError> {
        self.active_profile
            .read()
            .map(|guard| guard.clone())
            .map_err(|e| ProfileError::LockError(format!("Failed to acquire read lock: {}", e)))
    }

    /// Get profile metadata by name.
    pub fn get(&self, name: &str) -> Option<ProfileMetadata> {
        self.profiles
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(name)
            .cloned()
    }

    /// Get the configuration content (.rhai file) for a profile.
    ///
    /// # Arguments
    ///
    /// * `name` - Profile name
    ///
    /// # Returns
    ///
    /// The content of the .rhai configuration file as a String.
    ///
    /// # Errors
    ///
    /// Returns `ProfileError::NotFound` if the profile doesn't exist.
    /// Returns `ProfileError::IoError` if the file cannot be read.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::path::PathBuf;
    /// # use keyrx_daemon::config::ProfileManager;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let manager = ProfileManager::new(PathBuf::from("./config"))?;
    /// let config = manager.get_config("default")?;
    /// println!("Config content:\n{}", config);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_config(&self, name: &str) -> Result<String, ProfileError> {
        let profiles = self.profiles.read().unwrap_or_else(PoisonError::into_inner);
        let profile = profiles
            .get(name)
            .ok_or_else(|| ProfileError::NotFound(name.to_string()))?;

        fs::read_to_string(&profile.rhai_path).map_err(ProfileError::IoError)
    }

    /// Set the configuration content (.rhai file) for a profile.
    ///
    /// This method writes the configuration content to the profile's .rhai file.
    /// It does NOT automatically recompile or activate the profile.
    ///
    /// # Arguments
    ///
    /// * `name` - Profile name
    /// * `content` - The new configuration content to write
    ///
    /// # Errors
    ///
    /// Returns `ProfileError::NotFound` if the profile doesn't exist.
    /// Returns `ProfileError::IoError` if the file cannot be written.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::path::PathBuf;
    /// # use keyrx_daemon::config::ProfileManager;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let manager = ProfileManager::new(PathBuf::from("./config"))?;
    /// let new_config = r#"
    /// layer("base", #{
    ///     "KEY_A": simple("KEY_B"),
    /// });
    /// "#;
    /// manager.set_config("default", new_config)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_config(&self, name: &str, content: &str) -> Result<(), ProfileError> {
        let profile = {
            let profiles = self.profiles.read().unwrap_or_else(PoisonError::into_inner);
            profiles
                .get(name)
                .ok_or_else(|| ProfileError::NotFound(name.to_string()))?
                .clone()
        };

        // Write to a temporary file first (atomic write pattern)
        let temp_path = profile.rhai_path.with_extension("rhai.tmp");
        fs::write(&temp_path, content)?;

        // Rename to final location (atomic on most filesystems)
        fs::rename(&temp_path, &profile.rhai_path)?;

        // Recompile .rhai → .krx so the daemon can reload the updated config
        let compile_result = self
            .compiler
            .compile_profile(&profile.rhai_path, &profile.krx_path);
        if let Err(e) = compile_result {
            log::error!(
                "Failed to compile profile '{}' after config update: {}",
                name,
                e
            );
            return Err(ProfileError::Compilation(e));
        }
        log::info!("Recompiled profile '{}' after config update", name);

        // Update metadata (modified time will have changed)
        let updated_metadata = self.load_profile_metadata(name)?;
        self.profiles
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(name.to_string(), updated_metadata);

        Ok(())
    }

    // Test-only methods (available for integration tests)
    #[doc(hidden)]
    pub fn set_active_for_testing(&self, name: String) {
        // SAFETY: Test-only helper method - RwLock cannot be poisoned in test context
        #[allow(clippy::expect_used)]
        {
            *self
                .active_profile
                .write()
                .unwrap_or_else(PoisonError::into_inner) = Some(name);
        }
    }

    #[doc(hidden)]
    pub fn load_profile_metadata_for_testing(
        &self,
        name: &str,
    ) -> Result<ProfileMetadata, ProfileError> {
        self.load_profile_metadata(name)
    }

    #[doc(hidden)]
    pub fn load_template_for_testing(name: &str) -> String {
        Self::load_template(name)
    }
}

/// Counts shown in the profile list, read from the compiled `.krx` (what the
/// daemon would run). A profile that was never compiled reports one (base)
/// layer and no devices or keys.
struct ProfileStats {
    layers: usize,
    devices: usize,
    keys: usize,
}

impl ProfileStats {
    fn of_compiled(krx_path: &Path) -> Self {
        use keyrx_core::config::KeyMapping;

        let Ok(config) = crate::config_loader::load_config(krx_path) else {
            return Self {
                layers: 1,
                devices: 0,
                keys: 0,
            };
        };
        let keys = config
            .devices
            .iter()
            .flat_map(|d| &d.mappings)
            .map(|m| match m {
                KeyMapping::Base(_) => 1,
                KeyMapping::Conditional { mappings, .. } => mappings.len(),
            })
            .sum();
        let mut layers: Vec<u8> = config
            .devices
            .iter()
            .flat_map(crate::daemon::remapping_state::layer_modifiers)
            .collect();
        layers.sort_unstable();
        layers.dedup();
        Self {
            layers: 1 + layers.len(), // base + one per layer modifier
            devices: config.devices.len(),
            keys,
        }
    }
}
