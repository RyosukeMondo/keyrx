//! Profile management with hot-reload and thread-safe activation.
//!
//! This module provides the `ProfileManager` for creating, activating, and managing
//! Rhai configuration profiles with atomic hot-reload capabilities.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Instant, SystemTime};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::profile_compiler::{CompilationError, ProfileCompiler};
use typeshare::typeshare;

/// Maximum number of profiles allowed
const MAX_PROFILES: usize = 100;

/// File name for persisting active profile
const ACTIVE_PROFILE_FILE: &str = ".active";

/// Profile manager for CRUD operations and hot-reload.
pub struct ProfileManager {
    config_dir: PathBuf,
    active_profile: Arc<RwLock<Option<String>>>,
    profiles: HashMap<String, ProfileMetadata>,
    activation_lock: Arc<Mutex<()>>,
    compiler: ProfileCompiler,
}

/// Metadata for a single profile.
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileMetadata {
    pub name: String,
    #[typeshare(skip)]
    pub rhai_path: PathBuf,
    #[typeshare(skip)]
    pub krx_path: PathBuf,
    #[typeshare(skip)]
    pub modified_at: SystemTime,
    #[typeshare(serialized_as = "number")]
    pub layer_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[typeshare(skip)]
    pub activated_at: Option<SystemTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activated_by: Option<String>,
}

/// Template for creating new profiles.
#[typeshare]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileTemplate {
    /// Empty configuration with minimal valid syntax
    Blank,
    /// Simple A→B key remapping example
    SimpleRemap,
    /// CapsLock→Escape mapping
    CapslockEscape,
    /// Vim navigation with HJKL layer
    VimNavigation,
    /// Gaming-optimized profile
    Gaming,
}

/// Result of profile activation.
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivationResult {
    #[typeshare(serialized_as = "number")]
    pub compile_time_ms: u64,
    #[typeshare(serialized_as = "number")]
    pub reload_time_ms: u64,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Errors that can occur during profile operations.
#[derive(Debug, Error)]
pub enum ProfileError {
    #[error("Profile not found: {0}")]
    NotFound(String),

    #[error("Invalid profile name: {0}")]
    InvalidName(String),

    #[error("Compilation error: {0}")]
    Compilation(#[from] CompilationError),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Profile limit exceeded (max {MAX_PROFILES})")]
    ProfileLimitExceeded,

    #[error("Disk space exhausted")]
    DiskSpaceExhausted,

    #[error("Profile already exists: {0}")]
    AlreadyExists(String),

    #[error("Invalid template")]
    InvalidTemplate,

    #[error("Lock error: {0}")]
    LockError(String),

    #[error("Activation in progress for profile: {0}")]
    ActivationInProgress(String),

    #[error("Invalid metadata: {0}")]
    InvalidMetadata(String),
}

impl ProfileManager {
    /// Path to the profiles subdirectory.
    fn profiles_dir(&self) -> PathBuf {
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

        let mut manager = Self {
            config_dir,
            active_profile: Arc::new(RwLock::new(None)),
            profiles: HashMap::new(),
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
    pub fn scan_profiles(&mut self) -> Result<(), ProfileError> {
        let profiles_dir = self.profiles_dir();
        if !profiles_dir.exists() {
            return Ok(());
        }

        self.profiles.clear();

        for entry in fs::read_dir(&profiles_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("rhai") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    let metadata = self.load_profile_metadata(name)?;
                    self.profiles.insert(name.to_string(), metadata);
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

        // Try to read layer count from file (simple heuristic for now)
        let layer_count = Self::count_layers(&rhai_path)?;

        // PROF-004: Load activation metadata if this is the active profile
        let (activated_at, activated_by) = self.load_activation_metadata(name);

        Ok(ProfileMetadata {
            name: name.to_string(),
            rhai_path,
            krx_path,
            modified_at,
            layer_count,
            activated_at,
            activated_by,
        })
    }

    /// Count layers in a Rhai file (simple heuristic).
    fn count_layers(path: &Path) -> Result<usize, ProfileError> {
        let content = fs::read_to_string(path)?;
        let count = content.matches("layer(").count();
        Ok(count.max(1)) // At least one layer
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

    /// Create a new profile from a template.
    /// PROF-005: Enhanced duplicate name checking.
    pub fn create(
        &mut self,
        name: &str,
        template: ProfileTemplate,
    ) -> Result<ProfileMetadata, ProfileError> {
        Self::validate_name(name)?;

        if self.profiles.len() >= MAX_PROFILES {
            return Err(ProfileError::ProfileLimitExceeded);
        }

        // PROF-005: Check for duplicate in memory first
        if self.profiles.contains_key(name) {
            return Err(ProfileError::AlreadyExists(name.to_string()));
        }

        // PROF-005: Check for duplicate on disk (in case of desync)
        let rhai_path = self.rhai_path(name);
        if rhai_path.exists() {
            return Err(ProfileError::AlreadyExists(name.to_string()));
        }

        // Generate template content
        let content = match template {
            ProfileTemplate::Blank => Self::load_template("blank"),
            ProfileTemplate::SimpleRemap => Self::load_template("simple_remap"),
            ProfileTemplate::CapslockEscape => Self::load_template("capslock_escape"),
            ProfileTemplate::VimNavigation => Self::load_template("vim_navigation"),
            ProfileTemplate::Gaming => Self::load_template("gaming"),
        };

        fs::write(&rhai_path, content)?;

        let metadata = self.load_profile_metadata(name)?;
        self.profiles.insert(name.to_string(), metadata.clone());

        Ok(metadata)
    }

    /// Load template from embedded files.
    fn load_template(name: &str) -> String {
        match name {
            "blank" => include_str!("../../templates/blank.rhai"),
            "simple_remap" => include_str!("../../templates/simple_remap.rhai"),
            "capslock_escape" => include_str!("../../templates/capslock_escape.rhai"),
            "vim_navigation" => include_str!("../../templates/vim_navigation.rhai"),
            "gaming" => include_str!("../../templates/gaming.rhai"),
            _ => include_str!("../../templates/blank.rhai"),
        }
        .to_string()
    }

    /// Activate a profile with hot-reload.
    pub fn activate(&mut self, name: &str) -> Result<ActivationResult, ProfileError> {
        // Acquire activation lock to serialize concurrent activations
        let _lock = self.activation_lock.lock().map_err(|e| {
            ProfileError::LockError(format!("Failed to acquire activation lock: {}", e))
        })?;

        let start = Instant::now();

        // Get profile metadata
        let profile = self
            .profiles
            .get(name)
            .ok_or_else(|| ProfileError::NotFound(name.to_string()))?
            .clone();

        // Compile and reload
        let (compile_time, reload_time) = match self.compile_and_reload(name, &profile) {
            Ok(times) => times,
            Err((compile_time, e)) => {
                return Ok(ActivationResult {
                    compile_time_ms: compile_time,
                    reload_time_ms: 0,
                    success: false,
                    error: Some(e.to_string()),
                });
            }
        };

        log::info!(
            "Profile '{}' activated in {}ms (compile: {}ms, reload: {}ms)",
            name,
            start.elapsed().as_millis(),
            compile_time,
            reload_time
        );

        Ok(ActivationResult {
            compile_time_ms: compile_time,
            reload_time_ms: reload_time,
            success: true,
            error: None,
        })
    }

    /// Compile and reload a profile.
    /// PROF-003: Enhanced error handling with structured errors and context.
    fn compile_and_reload(
        &self,
        name: &str,
        profile: &ProfileMetadata,
    ) -> Result<(u64, u64), (u64, ProfileError)> {
        // PROF-003: Validate profile exists before attempting compilation
        if !profile.rhai_path.exists() {
            return Err((
                0,
                ProfileError::NotFound(format!(
                    "Profile '{}' source file not found at {:?}",
                    name, profile.rhai_path
                )),
            ));
        }

        // Compile .rhai → .krx with timeout
        let compile_result = self
            .compiler
            .compile_profile(&profile.rhai_path, &profile.krx_path);

        let compile_time = match compile_result {
            Ok(result) => result.compile_time_ms,
            Err(e) => {
                // PROF-003: Return structured compilation error with context
                log::error!("Compilation failed for profile '{}': {}", name, e);
                return Err((0, ProfileError::Compilation(e)));
            }
        };

        // Atomic swap
        let reload_start = Instant::now();
        *self.active_profile.write().map_err(|e| {
            (
                compile_time,
                ProfileError::LockError(format!(
                    "Failed to acquire write lock during activation of '{}': {}",
                    name, e
                )),
            )
        })? = Some(name.to_string());
        let reload_time = reload_start.elapsed().as_millis() as u64;

        // PROF-003: Persist active profile to disk with proper error handling
        if let Err(e) = self.save_active_profile(name) {
            log::warn!(
                "Failed to persist active profile '{}' (non-fatal): {}",
                name,
                e
            );
            // Don't fail the activation, but log the issue
        }

        Ok((compile_time, reload_time))
    }

    /// Delete a profile.
    pub fn delete(&mut self, name: &str) -> Result<(), ProfileError> {
        let profile = self
            .profiles
            .get(name)
            .ok_or_else(|| ProfileError::NotFound(name.to_string()))?
            .clone();

        // Check if this is the active profile
        let active = self
            .active_profile
            .read()
            .map_err(|e| ProfileError::LockError(format!("Failed to acquire read lock: {}", e)))?;
        if active.as_deref() == Some(name) {
            drop(active);
            *self.active_profile.write().map_err(|e| {
                ProfileError::LockError(format!("Failed to acquire write lock: {}", e))
            })? = None;
            // Clear persisted active profile since we're deleting it
            self.clear_active_profile_file();
        }

        // Delete both .rhai and .krx files
        if profile.rhai_path.exists() {
            fs::remove_file(&profile.rhai_path)?;
        }
        if profile.krx_path.exists() {
            fs::remove_file(&profile.krx_path)?;
        }

        self.profiles.remove(name);

        Ok(())
    }

    /// Duplicate a profile.
    pub fn duplicate(&mut self, src: &str, dest: &str) -> Result<ProfileMetadata, ProfileError> {
        Self::validate_name(dest)?;

        if self.profiles.len() >= MAX_PROFILES {
            return Err(ProfileError::ProfileLimitExceeded);
        }

        let src_profile = self
            .profiles
            .get(src)
            .ok_or_else(|| ProfileError::NotFound(src.to_string()))?
            .clone();

        let dest_rhai = self.rhai_path(dest);
        if dest_rhai.exists() {
            return Err(ProfileError::AlreadyExists(dest.to_string()));
        }

        fs::copy(&src_profile.rhai_path, &dest_rhai)?;

        let metadata = self.load_profile_metadata(dest)?;
        self.profiles.insert(dest.to_string(), metadata.clone());

        Ok(metadata)
    }

    /// Rename a profile.
    ///
    /// # Arguments
    /// * `old_name` - Current name of the profile
    /// * `new_name` - New name for the profile
    ///
    /// # Errors
    /// * `ProfileError::NotFound` - If the profile doesn't exist
    /// * `ProfileError::InvalidName` - If the new name is invalid
    /// * `ProfileError::AlreadyExists` - If a profile with the new name already exists
    /// * `ProfileError::IoError` - If file operations fail
    pub fn rename(
        &mut self,
        old_name: &str,
        new_name: &str,
    ) -> Result<ProfileMetadata, ProfileError> {
        // Validate new name
        Self::validate_name(new_name)?;

        // Check if source profile exists
        let old_profile = self
            .profiles
            .get(old_name)
            .ok_or_else(|| ProfileError::NotFound(old_name.to_string()))?
            .clone();

        // Check if destination already exists
        let new_rhai = self.rhai_path(new_name);
        if new_rhai.exists() {
            return Err(ProfileError::AlreadyExists(new_name.to_string()));
        }

        // Rename both .rhai and .krx files
        let new_krx = self.krx_path(new_name);

        fs::rename(&old_profile.rhai_path, &new_rhai)?;

        // Only rename .krx if it exists (might not exist if profile was never activated)
        if old_profile.krx_path.exists() {
            fs::rename(&old_profile.krx_path, &new_krx)?;
        }

        // Update active profile reference if renaming the active profile
        {
            let mut active = self.active_profile.write().map_err(|e| {
                ProfileError::LockError(format!("Failed to acquire write lock: {}", e))
            })?;
            if active.as_ref() == Some(&old_name.to_string()) {
                *active = Some(new_name.to_string());
            }
        }

        // Remove old entry and add new entry
        self.profiles.remove(old_name);
        let new_metadata = self.load_profile_metadata(new_name)?;
        self.profiles
            .insert(new_name.to_string(), new_metadata.clone());

        Ok(new_metadata)
    }

    /// Export a profile to a file.
    pub fn export(&self, name: &str, dest: &Path) -> Result<(), ProfileError> {
        let profile = self
            .profiles
            .get(name)
            .ok_or_else(|| ProfileError::NotFound(name.to_string()))?;

        fs::copy(&profile.rhai_path, dest)?;
        Ok(())
    }

    /// Import a profile from a file.
    pub fn import(&mut self, src: &Path, name: &str) -> Result<ProfileMetadata, ProfileError> {
        Self::validate_name(name)?;

        if self.profiles.len() >= MAX_PROFILES {
            return Err(ProfileError::ProfileLimitExceeded);
        }

        let dest_rhai = self.rhai_path(name);
        if dest_rhai.exists() {
            return Err(ProfileError::AlreadyExists(name.to_string()));
        }

        fs::copy(src, &dest_rhai)?;

        let metadata = self.load_profile_metadata(name)?;
        self.profiles.insert(name.to_string(), metadata.clone());

        Ok(metadata)
    }

    /// List all profiles.
    pub fn list(&self) -> Vec<&ProfileMetadata> {
        self.profiles.values().collect()
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

    /// Save the active profile name to persistent storage.
    /// PROF-004: Enhanced to store activation metadata (timestamp and source).
    ///
    /// This writes the profile name and metadata to a `.active` file in the config directory
    /// so it can be restored on daemon restart.
    fn save_active_profile(&self, name: &str) -> Result<(), ProfileError> {
        let active_file = self.config_dir.join(ACTIVE_PROFILE_FILE);

        // PROF-004: Store activation metadata as JSON
        let metadata = serde_json::json!({
            "name": name,
            "activated_at": SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            "activated_by": "user", // Default to user activation
        });

        let content = serde_json::to_string_pretty(&metadata)
            .map_err(|e| ProfileError::InvalidMetadata(e.to_string()))?;

        fs::write(&active_file, content).map_err(|e| {
            log::warn!(
                "Failed to persist active profile to {:?}: {}",
                active_file,
                e
            );
            ProfileError::IoError(e)
        })?;

        log::info!(
            "Persisted active profile '{}' with metadata to {:?}",
            name,
            active_file
        );
        Ok(())
    }

    /// Load activation metadata for a profile.
    /// PROF-004: Load activation timestamp and source from .active file.
    fn load_activation_metadata(&self, name: &str) -> (Option<SystemTime>, Option<String>) {
        let active_file = self.config_dir.join(ACTIVE_PROFILE_FILE);

        if !active_file.exists() {
            return (None, None);
        }

        match fs::read_to_string(&active_file) {
            Ok(content) => {
                // Try to parse as JSON first (new format)
                if let Ok(metadata) = serde_json::from_str::<serde_json::Value>(&content) {
                    let stored_name = metadata["name"].as_str();

                    // Only return metadata if this is the active profile
                    if stored_name == Some(name) {
                        let activated_at = metadata["activated_at"].as_u64().map(|secs| {
                            std::time::UNIX_EPOCH + std::time::Duration::from_secs(secs)
                        });
                        let activated_by = metadata["activated_by"].as_str().map(|s| s.to_string());

                        return (activated_at, activated_by);
                    }
                } else {
                    // Legacy format: just the profile name
                    let stored_name = content.trim();
                    if stored_name == name {
                        // Use file modification time as activation time
                        if let Ok(metadata) = active_file.metadata() {
                            if let Ok(modified) = metadata.modified() {
                                return (Some(modified), Some("user".to_string()));
                            }
                        }
                    }
                }
            }
            Err(e) => {
                log::warn!("Failed to read activation metadata: {}", e);
            }
        }

        (None, None)
    }

    /// Load the active profile name from persistent storage.
    ///
    /// This reads the `.active` file from the config directory.
    /// Returns None if the file doesn't exist or the profile no longer exists.
    fn load_active_profile(&self) -> Option<String> {
        let active_file = self.config_dir.join(ACTIVE_PROFILE_FILE);
        if !active_file.exists() {
            log::debug!(
                "No persisted active profile file found at {:?}",
                active_file
            );
            return None;
        }

        match fs::read_to_string(&active_file) {
            Ok(content) => {
                // PROF-004: Parse JSON metadata to extract profile name
                let name = match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(metadata) => {
                        // Extract name from JSON metadata
                        metadata
                            .get("name")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| {
                                // Fallback: treat entire content as plain text name (backward compat)
                                log::warn!(
                                    "Active profile file is not JSON, treating as plain text"
                                );
                                content.trim().to_string()
                            })
                    }
                    Err(_) => {
                        // Fallback: treat entire content as plain text name (backward compat)
                        log::warn!(
                            "Failed to parse active profile as JSON, treating as plain text"
                        );
                        content.trim().to_string()
                    }
                };

                // Verify the profile still exists
                if self.profiles.contains_key(&name) {
                    log::info!("Restored active profile '{}' from {:?}", name, active_file);
                    Some(name)
                } else {
                    log::warn!(
                        "Persisted active profile '{}' no longer exists, ignoring",
                        name
                    );
                    // Clean up stale file
                    let _ = fs::remove_file(&active_file);
                    None
                }
            }
            Err(e) => {
                log::warn!(
                    "Failed to read active profile from {:?}: {}",
                    active_file,
                    e
                );
                None
            }
        }
    }

    /// Clear the persisted active profile (used when deleting the active profile).
    fn clear_active_profile_file(&self) {
        let active_file = self.config_dir.join(ACTIVE_PROFILE_FILE);
        if active_file.exists() {
            if let Err(e) = fs::remove_file(&active_file) {
                log::warn!("Failed to remove active profile file: {}", e);
            }
        }
    }

    /// Get profile metadata by name.
    pub fn get(&self, name: &str) -> Option<&ProfileMetadata> {
        self.profiles.get(name)
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
        let profile = self
            .profiles
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
    /// let mut manager = ProfileManager::new(PathBuf::from("./config"))?;
    /// let new_config = r#"
    /// layer("base", #{
    ///     "KEY_A": simple("KEY_B"),
    /// });
    /// "#;
    /// manager.set_config("default", new_config)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_config(&mut self, name: &str, content: &str) -> Result<(), ProfileError> {
        let profile = self
            .profiles
            .get(name)
            .ok_or_else(|| ProfileError::NotFound(name.to_string()))?
            .clone();

        // Write to a temporary file first (atomic write pattern)
        let temp_path = profile.rhai_path.with_extension("rhai.tmp");
        fs::write(&temp_path, content)?;

        // Rename to final location (atomic on most filesystems)
        fs::rename(&temp_path, &profile.rhai_path)?;

        // Update metadata (modified time will have changed)
        let updated_metadata = self.load_profile_metadata(name)?;
        self.profiles.insert(name.to_string(), updated_metadata);

        Ok(())
    }

    // Test-only methods (available for integration tests)
    #[doc(hidden)]
    pub fn set_active_for_testing(&mut self, name: String) {
        // SAFETY: Test-only helper method - RwLock cannot be poisoned in test context
        #[allow(clippy::expect_used)]
        {
            *self
                .active_profile
                .write()
                .expect("Test helper: RwLock poisoned") = Some(name);
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

    #[doc(hidden)]
    pub fn count_layers_for_testing(path: &Path) -> Result<usize, ProfileError> {
        Self::count_layers(path)
    }
}
