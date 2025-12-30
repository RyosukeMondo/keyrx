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

/// Maximum number of profiles allowed
const MAX_PROFILES: usize = 100;

/// Maximum profile name length
const MAX_PROFILE_NAME_LEN: usize = 32;

/// Profile manager for CRUD operations and hot-reload.
pub struct ProfileManager {
    config_dir: PathBuf,
    active_profile: Arc<RwLock<Option<String>>>,
    profiles: HashMap<String, ProfileMetadata>,
    activation_lock: Arc<Mutex<()>>,
    compiler: ProfileCompiler,
}

/// Metadata for a single profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileMetadata {
    pub name: String,
    pub rhai_path: PathBuf,
    pub krx_path: PathBuf,
    pub modified_at: SystemTime,
    pub layer_count: usize,
}

/// Template for creating new profiles.
#[derive(Debug, Clone, Copy)]
pub enum ProfileTemplate {
    /// Empty configuration with just base layer
    Blank,
    /// QMK-style layer system example
    QmkLayers,
}

/// Result of profile activation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivationResult {
    pub compile_time_ms: u64,
    pub reload_time_ms: u64,
    pub success: bool,
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
}

impl ProfileManager {
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

        Ok(manager)
    }

    /// Scan the profiles directory for .rhai files.
    pub fn scan_profiles(&mut self) -> Result<(), ProfileError> {
        let profiles_dir = self.config_dir.join("profiles");
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
    fn load_profile_metadata(&self, name: &str) -> Result<ProfileMetadata, ProfileError> {
        let rhai_path = self
            .config_dir
            .join("profiles")
            .join(format!("{}.rhai", name));
        let krx_path = self
            .config_dir
            .join("profiles")
            .join(format!("{}.krx", name));

        if !rhai_path.exists() {
            return Err(ProfileError::NotFound(name.to_string()));
        }

        let modified_at = rhai_path.metadata()?.modified()?;

        // Try to read layer count from file (simple heuristic for now)
        let layer_count = Self::count_layers(&rhai_path)?;

        Ok(ProfileMetadata {
            name: name.to_string(),
            rhai_path,
            krx_path,
            modified_at,
            layer_count,
        })
    }

    /// Count layers in a Rhai file (simple heuristic).
    fn count_layers(path: &Path) -> Result<usize, ProfileError> {
        let content = fs::read_to_string(path)?;
        let count = content.matches("layer(").count();
        Ok(count.max(1)) // At least one layer
    }

    /// Validate profile name.
    pub fn validate_name(name: &str) -> Result<(), ProfileError> {
        if name.is_empty() {
            return Err(ProfileError::InvalidName(
                "Name cannot be empty".to_string(),
            ));
        }

        if name.len() > MAX_PROFILE_NAME_LEN {
            return Err(ProfileError::InvalidName(format!(
                "Name too long (max {} chars)",
                MAX_PROFILE_NAME_LEN
            )));
        }

        // Allow alphanumeric, dash, underscore
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(ProfileError::InvalidName(
                "Name can only contain alphanumeric characters, dashes, and underscores"
                    .to_string(),
            ));
        }

        Ok(())
    }

    /// Create a new profile from a template.
    pub fn create(
        &mut self,
        name: &str,
        template: ProfileTemplate,
    ) -> Result<ProfileMetadata, ProfileError> {
        Self::validate_name(name)?;

        if self.profiles.len() >= MAX_PROFILES {
            return Err(ProfileError::ProfileLimitExceeded);
        }

        let rhai_path = self
            .config_dir
            .join("profiles")
            .join(format!("{}.rhai", name));
        if rhai_path.exists() {
            return Err(ProfileError::AlreadyExists(name.to_string()));
        }

        // Generate template content
        let content = match template {
            ProfileTemplate::Blank => Self::generate_blank_template(),
            ProfileTemplate::QmkLayers => Self::generate_qmk_template(),
        };

        fs::write(&rhai_path, content)?;

        let metadata = self.load_profile_metadata(name)?;
        self.profiles.insert(name.to_string(), metadata.clone());

        Ok(metadata)
    }

    /// Generate blank template.
    fn generate_blank_template() -> String {
        r#"// KeyRx2 Configuration
// Base layer - passthrough by default
layer("base", #{
    // Add your key mappings here
});
"#
        .to_string()
    }

    /// Generate QMK-style template.
    fn generate_qmk_template() -> String {
        r#"// KeyRx2 Configuration - QMK-style layers
// Base layer
layer("base", #{
    // Example: Space as layer toggle
    "KEY_SPACE": tap_hold("KEY_SPACE", layer_toggle("lower"), 200),
});

// Lower layer - symbols and numbers
layer("lower", #{
    "KEY_A": simple("KEY_1"),
    "KEY_S": simple("KEY_2"),
    "KEY_D": simple("KEY_3"),
});
"#
        .to_string()
    }

    /// Activate a profile with hot-reload.
    pub fn activate(&mut self, name: &str) -> Result<ActivationResult, ProfileError> {
        // Acquire activation lock to serialize concurrent activations
        let _lock = self.activation_lock.lock().unwrap();

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
    fn compile_and_reload(
        &self,
        name: &str,
        profile: &ProfileMetadata,
    ) -> Result<(u64, u64), (u64, ProfileError)> {
        // Compile .rhai → .krx with timeout
        let compile_result = self
            .compiler
            .compile_profile(&profile.rhai_path, &profile.krx_path);

        let compile_time = match compile_result {
            Ok(result) => result.compile_time_ms,
            Err(e) => {
                // Return compilation time as 0 for errors
                return Err((0, e.into()));
            }
        };

        // Atomic swap
        let reload_start = Instant::now();
        *self.active_profile.write().unwrap() = Some(name.to_string());
        let reload_time = reload_start.elapsed().as_millis() as u64;

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
        let active = self.active_profile.read().unwrap();
        if active.as_deref() == Some(name) {
            drop(active);
            *self.active_profile.write().unwrap() = None;
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

        let dest_rhai = self
            .config_dir
            .join("profiles")
            .join(format!("{}.rhai", dest));
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
        let new_rhai = self
            .config_dir
            .join("profiles")
            .join(format!("{}.rhai", new_name));
        if new_rhai.exists() {
            return Err(ProfileError::AlreadyExists(new_name.to_string()));
        }

        // Rename both .rhai and .krx files
        let new_krx = self
            .config_dir
            .join("profiles")
            .join(format!("{}.krx", new_name));

        fs::rename(&old_profile.rhai_path, &new_rhai)?;

        // Only rename .krx if it exists (might not exist if profile was never activated)
        if old_profile.krx_path.exists() {
            fs::rename(&old_profile.krx_path, &new_krx)?;
        }

        // Update active profile reference if renaming the active profile
        {
            let mut active = self.active_profile.write().unwrap();
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

        let dest_rhai = self
            .config_dir
            .join("profiles")
            .join(format!("{}.rhai", name));
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
    pub fn get_active(&self) -> Option<String> {
        self.active_profile.read().unwrap().clone()
    }

    /// Get profile metadata by name.
    pub fn get(&self, name: &str) -> Option<&ProfileMetadata> {
        self.profiles.get(name)
    }

    // Test-only methods (available for integration tests)
    #[doc(hidden)]
    pub fn set_active_for_testing(&mut self, name: String) {
        *self.active_profile.write().unwrap() = Some(name);
    }

    #[doc(hidden)]
    pub fn load_profile_metadata_for_testing(
        &self,
        name: &str,
    ) -> Result<ProfileMetadata, ProfileError> {
        self.load_profile_metadata(name)
    }

    #[doc(hidden)]
    pub fn generate_blank_template_for_testing() -> String {
        Self::generate_blank_template()
    }

    #[doc(hidden)]
    pub fn generate_qmk_template_for_testing() -> String {
        Self::generate_qmk_template()
    }

    #[doc(hidden)]
    pub fn count_layers_for_testing(path: &Path) -> Result<usize, ProfileError> {
        Self::count_layers(path)
    }
}
