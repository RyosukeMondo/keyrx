//! Profile data model for revolutionary mapping system.
//!
//! This module defines the core data structures for storing and managing
//! key mapping profiles. Profiles are layout-aware and can be assigned to
//! specific devices based on their physical layout.

use crate::config::config_dir;
use crate::drivers::keycodes::KeyCode;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

/// Unique identifier for a profile (UUID v4)
pub type ProfileId = String;

/// Physical position of a key on a device layout
///
/// Used to identify keys by their physical location (row, column)
/// rather than their scancode, enabling layout-aware remapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PhysicalPosition {
    /// Row index (0-based)
    pub row: u8,
    /// Column index (0-based)
    pub col: u8,
}

impl PhysicalPosition {
    /// Create a new physical position
    pub fn new(row: u8, col: u8) -> Self {
        Self { row, col }
    }

    /// Convert to a string key for serialization (format: "row,col")
    pub fn to_key(&self) -> String {
        format!("{},{}", self.row, self.col)
    }

    /// Parse from a string key (format: "row,col")
    pub fn from_key(key: &str) -> Option<Self> {
        let parts: Vec<&str> = key.split(',').collect();
        if parts.len() != 2 {
            return None;
        }
        let row = parts[0].parse().ok()?;
        let col = parts[1].parse().ok()?;
        Some(Self::new(row, col))
    }
}

impl fmt::Display for PhysicalPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{}", self.row, self.col)
    }
}

/// Layout type for device profiles
///
/// Defines the physical layout structure of a device, which determines
/// how keys are organized and mapped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LayoutType {
    /// Standard keyboard layout (ANSI/ISO)
    /// Keys are identified by standard positions
    Standard,

    /// Matrix layout (e.g., macro pads, Stream Deck)
    /// Keys are organized in a grid (row, col)
    Matrix,

    /// Split keyboard layout
    /// Two independent halves with separate coordinate systems
    Split,
}

impl LayoutType {
    /// Check if this layout type is compatible with another
    pub fn is_compatible_with(&self, other: &LayoutType) -> bool {
        self == other
    }
}

/// Action to perform when a key is pressed
///
/// Defines the behavior of a mapped key, supporting simple remaps,
/// chords, scripts, blocking, and passthrough.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum KeyAction {
    /// Remap to a single key
    Key {
        /// The output key to emit
        key: KeyCode,
    },

    /// Remap to a chord (multiple keys pressed simultaneously)
    Chord {
        /// Keys to press together
        keys: Vec<KeyCode>,
    },

    /// Execute a script/command
    Script {
        /// Script identifier or command to run
        script: String,
    },

    /// Block the key (no output)
    Block,

    /// Pass through unchanged (default behavior)
    Pass,
}

impl KeyAction {
    /// Create a simple key remap action
    pub fn key(key: KeyCode) -> Self {
        KeyAction::Key { key }
    }

    /// Create a chord action
    pub fn chord(keys: Vec<KeyCode>) -> Self {
        KeyAction::Chord { keys }
    }

    /// Create a script action
    pub fn script(script: impl Into<String>) -> Self {
        KeyAction::Script {
            script: script.into(),
        }
    }

    /// Create a block action
    pub fn block() -> Self {
        KeyAction::Block
    }

    /// Create a passthrough action
    pub fn pass() -> Self {
        KeyAction::Pass
    }
}

// Custom serialization/deserialization for HashMap<PhysicalPosition, KeyAction>
mod mappings_serde {
    use super::*;

    pub fn serialize<S>(
        map: &HashMap<PhysicalPosition, KeyAction>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let string_map: HashMap<String, &KeyAction> =
            map.iter().map(|(k, v)| (k.to_key(), v)).collect();
        string_map.serialize(serializer)
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<HashMap<PhysicalPosition, KeyAction>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string_map: HashMap<String, KeyAction> = HashMap::deserialize(deserializer)?;
        string_map
            .into_iter()
            .map(|(k, v)| {
                PhysicalPosition::from_key(&k)
                    .map(|pos| (pos, v))
                    .ok_or_else(|| serde::de::Error::custom(format!("Invalid position key: {}", k)))
            })
            .collect()
    }
}

/// A key mapping profile
///
/// Profiles define how keys are remapped for devices with a specific layout.
/// They are layout-aware and can be assigned to multiple devices with the
/// same layout type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    /// Unique identifier (UUID v4)
    pub id: ProfileId,

    /// Human-readable profile name
    pub name: String,

    /// Layout type this profile is designed for
    pub layout_type: LayoutType,

    /// Key mappings: physical position → action
    /// Only contains entries for remapped keys (sparse map)
    #[serde(default, with = "mappings_serde")]
    pub mappings: HashMap<PhysicalPosition, KeyAction>,

    /// Creation timestamp (ISO 8601)
    pub created_at: String,

    /// Last modification timestamp (ISO 8601)
    pub updated_at: String,
}

impl Profile {
    /// Create a new profile with the given name and layout type
    pub fn new(name: impl Into<String>, layout_type: LayoutType) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            layout_type,
            mappings: HashMap::new(),
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// Get the action for a physical position
    pub fn get_action(&self, pos: &PhysicalPosition) -> Option<&KeyAction> {
        self.mappings.get(pos)
    }

    /// Set the action for a physical position
    pub fn set_action(&mut self, pos: PhysicalPosition, action: KeyAction) {
        self.mappings.insert(pos, action);
        self.touch();
    }

    /// Remove the mapping for a physical position
    pub fn remove_action(&mut self, pos: &PhysicalPosition) -> Option<KeyAction> {
        let result = self.mappings.remove(pos);
        if result.is_some() {
            self.touch();
        }
        result
    }

    /// Clear all mappings
    pub fn clear_mappings(&mut self) {
        self.mappings.clear();
        self.touch();
    }

    /// Update the updated_at timestamp to current time
    pub fn touch(&mut self) {
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    /// Check if this profile is compatible with a given layout type
    pub fn is_compatible_with(&self, layout_type: &LayoutType) -> bool {
        self.layout_type.is_compatible_with(layout_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physical_position_new() {
        let pos = PhysicalPosition::new(2, 5);
        assert_eq!(pos.row, 2);
        assert_eq!(pos.col, 5);
    }

    #[test]
    fn test_physical_position_hash_eq() {
        let pos1 = PhysicalPosition::new(1, 2);
        let pos2 = PhysicalPosition::new(1, 2);
        let pos3 = PhysicalPosition::new(2, 1);

        assert_eq!(pos1, pos2);
        assert_ne!(pos1, pos3);

        // Test HashMap usage
        let mut map = HashMap::new();
        map.insert(pos1, "test");
        assert_eq!(map.get(&pos2), Some(&"test"));
        assert_eq!(map.get(&pos3), None);
    }

    #[test]
    fn test_layout_type_compatibility() {
        assert!(LayoutType::Standard.is_compatible_with(&LayoutType::Standard));
        assert!(LayoutType::Matrix.is_compatible_with(&LayoutType::Matrix));
        assert!(!LayoutType::Standard.is_compatible_with(&LayoutType::Matrix));
    }

    #[test]
    fn test_key_action_constructors() {
        let key = KeyAction::key(KeyCode::A);
        assert_eq!(key, KeyAction::Key { key: KeyCode::A });

        let chord = KeyAction::chord(vec![KeyCode::LeftCtrl, KeyCode::C]);
        assert_eq!(
            chord,
            KeyAction::Chord {
                keys: vec![KeyCode::LeftCtrl, KeyCode::C]
            }
        );

        let script = KeyAction::script("launch.sh");
        assert_eq!(
            script,
            KeyAction::Script {
                script: "launch.sh".to_string()
            }
        );

        assert_eq!(KeyAction::block(), KeyAction::Block);
        assert_eq!(KeyAction::pass(), KeyAction::Pass);
    }

    #[test]
    fn test_profile_new() {
        let profile = Profile::new("Test Profile", LayoutType::Matrix);

        assert!(!profile.id.is_empty());
        assert_eq!(profile.name, "Test Profile");
        assert_eq!(profile.layout_type, LayoutType::Matrix);
        assert!(profile.mappings.is_empty());
        assert!(!profile.created_at.is_empty());
        assert_eq!(profile.created_at, profile.updated_at);
    }

    #[test]
    fn test_profile_mappings() {
        let mut profile = Profile::new("Test", LayoutType::Standard);
        let pos = PhysicalPosition::new(0, 0);

        // Initially no mapping
        assert_eq!(profile.get_action(&pos), None);

        // Set a mapping
        profile.set_action(pos, KeyAction::key(KeyCode::A));
        assert_eq!(
            profile.get_action(&pos),
            Some(&KeyAction::Key { key: KeyCode::A })
        );

        // Update mapping
        profile.set_action(pos, KeyAction::key(KeyCode::B));
        assert_eq!(
            profile.get_action(&pos),
            Some(&KeyAction::Key { key: KeyCode::B })
        );

        // Remove mapping
        let removed = profile.remove_action(&pos);
        assert_eq!(removed, Some(KeyAction::Key { key: KeyCode::B }));
        assert_eq!(profile.get_action(&pos), None);
    }

    #[test]
    fn test_profile_clear_mappings() {
        let mut profile = Profile::new("Test", LayoutType::Matrix);

        profile.set_action(PhysicalPosition::new(0, 0), KeyAction::key(KeyCode::A));
        profile.set_action(PhysicalPosition::new(0, 1), KeyAction::key(KeyCode::B));
        assert_eq!(profile.mappings.len(), 2);

        profile.clear_mappings();
        assert!(profile.mappings.is_empty());
    }

    #[test]
    fn test_profile_touch() {
        let mut profile = Profile::new("Test", LayoutType::Standard);
        let original_time = profile.updated_at.clone();

        // Sleep a bit to ensure timestamp changes
        std::thread::sleep(std::time::Duration::from_millis(10));

        profile.touch();
        assert_ne!(profile.updated_at, original_time);
    }

    #[test]
    fn test_profile_compatibility() {
        let profile = Profile::new("Test", LayoutType::Matrix);

        assert!(profile.is_compatible_with(&LayoutType::Matrix));
        assert!(!profile.is_compatible_with(&LayoutType::Standard));
    }

    #[test]
    fn test_serialization() {
        let mut profile = Profile::new("Test Profile", LayoutType::Matrix);
        profile.set_action(PhysicalPosition::new(0, 0), KeyAction::key(KeyCode::A));
        profile.set_action(
            PhysicalPosition::new(1, 2),
            KeyAction::chord(vec![KeyCode::LeftCtrl, KeyCode::C]),
        );

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&profile).unwrap();

        // Deserialize back
        let deserialized: Profile = serde_json::from_str(&json).unwrap();

        assert_eq!(profile.id, deserialized.id);
        assert_eq!(profile.name, deserialized.name);
        assert_eq!(profile.layout_type, deserialized.layout_type);
        assert_eq!(profile.mappings, deserialized.mappings);
    }
}

// =============================================================================
// ProfileRegistry
// =============================================================================

/// Errors that can occur during ProfileRegistry operations
#[derive(Debug, Error)]
pub enum ProfileRegistryError {
    #[error("IO error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to serialize/deserialize profile at {path}: {source}")]
    Serialization {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("Profile not found: {0}")]
    NotFound(ProfileId),

    #[error("Invalid profile: {0}")]
    Validation(String),

    #[error("Profile name already exists: {0}")]
    DuplicateName(String),
}

/// ProfileRegistry manages persistent storage and in-memory caching of profiles
///
/// Provides CRUD operations for profiles with:
/// - In-memory cache for fast access
/// - Atomic writes to prevent corruption
/// - Validation before save
/// - Thread-safe concurrent access
pub struct ProfileRegistry {
    /// Path to the profiles directory
    profiles_dir: PathBuf,
    /// In-memory cache of loaded profiles (profile_id -> Profile)
    cache: Arc<RwLock<HashMap<ProfileId, Arc<Profile>>>>,
}

impl ProfileRegistry {
    /// Create a new ProfileRegistry with default storage location
    pub fn new() -> Self {
        Self::with_directory(Self::default_profiles_dir())
    }

    /// Create a new ProfileRegistry with a custom storage directory
    pub fn with_directory(profiles_dir: PathBuf) -> Self {
        Self {
            profiles_dir,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get the default profiles directory
    fn default_profiles_dir() -> PathBuf {
        config_dir().join("profiles")
    }

    /// Initialize the registry by loading all profiles from disk
    pub async fn load_all_profiles(&self) -> Result<usize, ProfileRegistryError> {
        // Ensure directory exists
        std::fs::create_dir_all(&self.profiles_dir).map_err(|source| ProfileRegistryError::Io {
            path: self.profiles_dir.clone(),
            source,
        })?;

        let mut count = 0;
        let entries =
            std::fs::read_dir(&self.profiles_dir).map_err(|source| ProfileRegistryError::Io {
                path: self.profiles_dir.clone(),
                source,
            })?;

        for entry in entries {
            let entry = entry.map_err(|source| ProfileRegistryError::Io {
                path: self.profiles_dir.clone(),
                source,
            })?;

            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match self.load_profile_from_file(&path) {
                    Ok(profile) => {
                        let mut cache = self.cache.write().await;
                        cache.insert(profile.id.clone(), Arc::new(profile));
                        count += 1;
                    }
                    Err(e) => {
                        tracing::warn!(
                            service = "keyrx",
                            event = "profile_load_failed",
                            component = "registry",
                            path = %path.display(),
                            error = %e,
                            "Failed to load profile, skipping"
                        );
                    }
                }
            }
        }

        Ok(count)
    }

    /// Load a profile from a file
    fn load_profile_from_file(&self, path: &Path) -> Result<Profile, ProfileRegistryError> {
        let data = std::fs::read_to_string(path).map_err(|source| ProfileRegistryError::Io {
            path: path.to_path_buf(),
            source,
        })?;

        let profile: Profile =
            serde_json::from_str(&data).map_err(|source| ProfileRegistryError::Serialization {
                path: path.to_path_buf(),
                source,
            })?;

        self.validate_profile(&profile)?;
        Ok(profile)
    }

    /// Save a profile to persistent storage with atomic write
    pub async fn save_profile(&self, profile: &Profile) -> Result<(), ProfileRegistryError> {
        self.validate_profile(profile)?;

        // Ensure directory exists
        std::fs::create_dir_all(&self.profiles_dir).map_err(|source| ProfileRegistryError::Io {
            path: self.profiles_dir.clone(),
            source,
        })?;

        let path = self.profile_file_path(&profile.id);

        // Serialize to JSON
        let serialized = serde_json::to_vec_pretty(profile).map_err(|source| {
            ProfileRegistryError::Serialization {
                path: path.clone(),
                source,
            }
        })?;

        // Atomic write: temp file + rename
        let tmp_path = path.with_extension("json.tmp");
        std::fs::write(&tmp_path, serialized).map_err(|source| ProfileRegistryError::Io {
            path: tmp_path.clone(),
            source,
        })?;

        std::fs::rename(&tmp_path, &path).map_err(|source| {
            // Best-effort cleanup of temp file
            let _ = std::fs::remove_file(&tmp_path);
            ProfileRegistryError::Io {
                path: path.clone(),
                source,
            }
        })?;

        // Update cache
        let mut cache = self.cache.write().await;
        cache.insert(profile.id.clone(), Arc::new(profile.clone()));

        tracing::info!(
            service = "keyrx",
            event = "profile_saved",
            component = "registry",
            profile_id = %profile.id,
            profile_name = %profile.name,
            "Profile saved successfully"
        );

        Ok(())
    }

    /// Get a profile by ID (returns Arc for zero-copy sharing)
    pub async fn get_profile(
        &self,
        profile_id: &str,
    ) -> Result<Arc<Profile>, ProfileRegistryError> {
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

    /// Delete a profile
    pub async fn delete_profile(&self, profile_id: &str) -> Result<(), ProfileRegistryError> {
        let path = self.profile_file_path(profile_id);

        if !path.exists() {
            return Err(ProfileRegistryError::NotFound(profile_id.to_string()));
        }

        std::fs::remove_file(&path).map_err(|source| ProfileRegistryError::Io {
            path: path.clone(),
            source,
        })?;

        // Remove from cache
        let mut cache = self.cache.write().await;
        cache.remove(profile_id);

        tracing::info!(
            service = "keyrx",
            event = "profile_deleted",
            component = "registry",
            profile_id = %profile_id,
            "Profile deleted successfully"
        );

        Ok(())
    }

    /// List all profile IDs
    pub async fn list_profiles(&self) -> Vec<ProfileId> {
        let cache = self.cache.read().await;
        cache.keys().cloned().collect()
    }

    /// Find profiles compatible with a given layout type
    pub async fn find_compatible_profiles(&self, layout_type: &LayoutType) -> Vec<Arc<Profile>> {
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

    /// Get the file path for a profile
    fn profile_file_path(&self, profile_id: &str) -> PathBuf {
        self.profiles_dir.join(format!("{}.json", profile_id))
    }

    /// Invalidate the cache for a specific profile (useful after external modifications)
    pub async fn invalidate_cache(&self, profile_id: &str) {
        let mut cache = self.cache.write().await;
        cache.remove(profile_id);
    }

    /// Clear the entire cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Get the number of profiles in cache
    pub async fn cache_size(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }
}

impl Default for ProfileRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod registry_tests {
    use super::*;
    use serial_test::serial;
    use tempfile::tempdir;

    #[tokio::test]
    #[serial]
    async fn test_save_and_get_profile() {
        let temp = tempdir().unwrap();
        let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

        let mut profile = Profile::new("Test Profile", LayoutType::Matrix);
        profile.set_action(PhysicalPosition::new(0, 0), KeyAction::key(KeyCode::A));

        // Save profile
        registry.save_profile(&profile).await.unwrap();

        // Get profile back
        let loaded = registry.get_profile(&profile.id).await.unwrap();
        assert_eq!(loaded.id, profile.id);
        assert_eq!(loaded.name, profile.name);
        assert_eq!(loaded.layout_type, profile.layout_type);
        assert_eq!(loaded.mappings.len(), 1);
    }

    #[tokio::test]
    #[serial]
    async fn test_delete_profile() {
        let temp = tempdir().unwrap();
        let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

        let profile = Profile::new("Test Profile", LayoutType::Standard);
        registry.save_profile(&profile).await.unwrap();

        // Verify it exists
        assert!(registry.get_profile(&profile.id).await.is_ok());

        // Delete it
        registry.delete_profile(&profile.id).await.unwrap();

        // Verify it's gone
        assert!(matches!(
            registry.get_profile(&profile.id).await,
            Err(ProfileRegistryError::NotFound(_))
        ));
    }

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
    async fn test_load_all_profiles() {
        let temp = tempdir().unwrap();
        let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

        // Save some profiles
        let profile1 = Profile::new("Profile 1", LayoutType::Matrix);
        let profile2 = Profile::new("Profile 2", LayoutType::Standard);

        registry.save_profile(&profile1).await.unwrap();
        registry.save_profile(&profile2).await.unwrap();

        // Clear cache
        registry.clear_cache().await;
        assert_eq!(registry.cache_size().await, 0);

        // Load all profiles
        let count = registry.load_all_profiles().await.unwrap();
        assert_eq!(count, 2);
        assert_eq!(registry.cache_size().await, 2);
    }

    #[tokio::test]
    #[serial]
    async fn test_atomic_write() {
        let temp = tempdir().unwrap();
        let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

        let profile = Profile::new("Test", LayoutType::Matrix);
        let profile_path = registry.profile_file_path(&profile.id);
        let tmp_path = profile_path.with_extension("json.tmp");

        registry.save_profile(&profile).await.unwrap();

        // Verify final file exists and temp file is cleaned up
        assert!(profile_path.exists());
        assert!(!tmp_path.exists());
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
    async fn test_corrupted_file_handling() {
        let temp = tempdir().unwrap();
        let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

        // Create a corrupted file
        let corrupted_path = temp.path().join("corrupted.json");
        std::fs::write(&corrupted_path, "not valid json").unwrap();

        // load_all_profiles should skip corrupted files and continue
        let count = registry.load_all_profiles().await.unwrap();
        assert_eq!(count, 0); // No valid profiles loaded
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
