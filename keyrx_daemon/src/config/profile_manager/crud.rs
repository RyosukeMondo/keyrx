//! Profile CRUD operations: create (from template), delete, duplicate, rename,
//! and file-based import/export.

use std::fs;
use std::path::Path;

use super::types::{ProfileError, ProfileMetadata, ProfileTemplate};
use super::{ProfileManager, MAX_PROFILES};

impl ProfileManager {
    /// Create a new profile from a template.
    /// PROF-005: Enhanced duplicate name checking.
    pub fn create(
        &self,
        name: &str,
        template: ProfileTemplate,
    ) -> Result<ProfileMetadata, ProfileError> {
        Self::validate_name(name)?;

        let mut profiles = self.profiles.write().map_err(|e| {
            ProfileError::LockError(format!("Failed to acquire profiles write lock: {}", e))
        })?;

        if profiles.len() >= MAX_PROFILES {
            return Err(ProfileError::ProfileLimitExceeded);
        }

        // PROF-005: Check for duplicate in memory first
        if profiles.contains_key(name) {
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
        profiles.insert(name.to_string(), metadata.clone());

        Ok(metadata)
    }

    /// Load template from embedded files.
    pub(super) fn load_template(name: &str) -> String {
        match name {
            "blank" => include_str!("../../../templates/blank.rhai"),
            "simple_remap" => include_str!("../../../templates/simple_remap.rhai"),
            "capslock_escape" => include_str!("../../../templates/capslock_escape.rhai"),
            "vim_navigation" => include_str!("../../../templates/vim_navigation.rhai"),
            "gaming" => include_str!("../../../templates/gaming.rhai"),
            _ => include_str!("../../../templates/blank.rhai"),
        }
        .to_string()
    }

    /// Delete a profile.
    pub fn delete(&self, name: &str) -> Result<(), ProfileError> {
        let profile = {
            let profiles = self.profiles.read().map_err(|e| {
                ProfileError::LockError(format!("Failed to acquire profiles read lock: {}", e))
            })?;
            profiles
                .get(name)
                .ok_or_else(|| ProfileError::NotFound(name.to_string()))?
                .clone()
        };

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

        self.profiles
            .write()
            .map_err(|e| {
                ProfileError::LockError(format!("Failed to acquire profiles write lock: {}", e))
            })?
            .remove(name);

        Ok(())
    }

    /// Duplicate a profile.
    pub fn duplicate(&self, src: &str, dest: &str) -> Result<ProfileMetadata, ProfileError> {
        Self::validate_name(dest)?;

        let mut profiles = self.profiles.write().map_err(|e| {
            ProfileError::LockError(format!("Failed to acquire profiles write lock: {}", e))
        })?;

        if profiles.len() >= MAX_PROFILES {
            return Err(ProfileError::ProfileLimitExceeded);
        }

        let src_profile = profiles
            .get(src)
            .ok_or_else(|| ProfileError::NotFound(src.to_string()))?
            .clone();

        let dest_rhai = self.rhai_path(dest);
        if dest_rhai.exists() {
            return Err(ProfileError::AlreadyExists(dest.to_string()));
        }

        fs::copy(&src_profile.rhai_path, &dest_rhai)?;

        let metadata = self.load_profile_metadata(dest)?;
        profiles.insert(dest.to_string(), metadata.clone());

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
    pub fn rename(&self, old_name: &str, new_name: &str) -> Result<ProfileMetadata, ProfileError> {
        // Validate new name
        Self::validate_name(new_name)?;

        let mut profiles = self.profiles.write().map_err(|e| {
            ProfileError::LockError(format!("Failed to acquire profiles write lock: {}", e))
        })?;

        // Check if source profile exists
        let old_profile = profiles
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
                // Keep the persisted .active in step, or a restart (and the
                // daemon's next reload) would look for the old name.
                if let Err(e) = self.save_active_profile(new_name) {
                    log::warn!("Failed to persist renamed active profile: {}", e);
                }
            }
        }

        // Remove old entry and add new entry
        profiles.remove(old_name);
        let new_metadata = self.load_profile_metadata(new_name)?;
        profiles.insert(new_name.to_string(), new_metadata.clone());

        Ok(new_metadata)
    }

    /// Export a profile to a file.
    pub fn export(&self, name: &str, dest: &Path) -> Result<(), ProfileError> {
        let profiles = self.profiles.read().expect("profiles RwLock poisoned");
        let profile = profiles
            .get(name)
            .ok_or_else(|| ProfileError::NotFound(name.to_string()))?;

        fs::copy(&profile.rhai_path, dest)?;
        Ok(())
    }

    /// Import a profile from a file.
    pub fn import(&self, src: &Path, name: &str) -> Result<ProfileMetadata, ProfileError> {
        Self::validate_name(name)?;

        let mut profiles = self.profiles.write().map_err(|e| {
            ProfileError::LockError(format!("Failed to acquire profiles write lock: {}", e))
        })?;

        if profiles.len() >= MAX_PROFILES {
            return Err(ProfileError::ProfileLimitExceeded);
        }

        let dest_rhai = self.rhai_path(name);
        if dest_rhai.exists() {
            return Err(ProfileError::AlreadyExists(name.to_string()));
        }

        fs::copy(src, &dest_rhai)?;

        let metadata = self.load_profile_metadata(name)?;
        profiles.insert(name.to_string(), metadata.clone());

        Ok(metadata)
    }
}
