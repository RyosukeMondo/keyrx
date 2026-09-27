//! Persistence of the active-profile marker (`.active` file) across daemon restarts.
//!
//! PROF-004: the marker stores JSON activation metadata (name, timestamp, source) with a
//! fallback to the legacy plain-text format for files written before that change.

use std::fs;
use std::time::SystemTime;

use super::types::ProfileError;
use super::{ProfileManager, ACTIVE_PROFILE_FILE};

impl ProfileManager {
    /// Save the active profile name to persistent storage.
    /// PROF-004: Enhanced to store activation metadata (timestamp and source).
    ///
    /// This writes the profile name and metadata to a `.active` file in the config directory
    /// so it can be restored on daemon restart.
    pub(super) fn save_active_profile(&self, name: &str) -> Result<(), ProfileError> {
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
    pub(super) fn load_activation_metadata(
        &self,
        name: &str,
    ) -> (Option<SystemTime>, Option<String>) {
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
    pub(super) fn load_active_profile(&self) -> Option<String> {
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
                if self
                    .profiles
                    .read()
                    .expect("profiles RwLock poisoned")
                    .contains_key(&name)
                {
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
    pub(super) fn clear_active_profile_file(&self) {
        let active_file = self.config_dir.join(ACTIVE_PROFILE_FILE);
        if active_file.exists() {
            if let Err(e) = fs::remove_file(&active_file) {
                log::warn!("Failed to remove active profile file: {}", e);
            }
        }
    }
}
