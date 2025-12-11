//! Profile data model for revolutionary mapping system.
//!
//! This module defines the core data structures for storing and managing
//! key mapping profiles. Profiles are layout-aware and can be assigned to
//! specific devices based on their physical layout.
//!
//! # Submodules
//!
//! - **storage**: File I/O, persistence, and atomic write operations
//! - **resolution**: Profile resolution, queries, cache management, and validation

mod resolution;
mod storage;

use crate::config::config_dir;
use crate::drivers::keycodes::KeyCode;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
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
    pub(crate) profiles_dir: PathBuf,
    /// In-memory cache of loaded profiles (profile_id -> Profile)
    pub(crate) cache: Arc<RwLock<HashMap<ProfileId, Arc<Profile>>>>,
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
    pub(crate) fn default_profiles_dir() -> PathBuf {
        config_dir().join("profiles")
    }

    /// Get the file path for a profile
    pub(crate) fn profile_file_path(&self, profile_id: &str) -> PathBuf {
        self.profiles_dir.join(format!("{}.json", profile_id))
    }
}

impl Default for ProfileRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Re-export storage and resolution methods
pub use resolution::ProfileRegistryResolution;
pub use storage::ProfileRegistryStorage;

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
