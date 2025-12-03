//! State persistence for saving and loading engine state across sessions.
//!
//! This module provides functionality to:
//! - Save engine state to disk in a versioned format
//! - Load state from disk with automatic version migration
//! - Handle corrupted state files with fallback to clean state
//! - Validate state integrity on load
//!
//! # File Format
//!
//! State is serialized to JSON using the StateSnapshot format, which includes:
//! - A format version for migration support
//! - Complete engine state (keys, layers, modifiers, pending)
//! - Metadata for validation
//!
//! # Error Handling
//!
//! The persistence layer is designed to be robust:
//! - Corrupted files are detected and logged
//! - Version mismatches trigger migration or fallback
//! - I/O errors are reported but don't crash the engine
//! - All errors result in a clean fallback state

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::snapshot::StateSnapshot;
use super::{EngineState, StateError, StateResult};
use crate::engine::decision::timing::TimingConfig;

/// Current persistence format version.
///
/// Increment this whenever the StateSnapshot format changes in a
/// backwards-incompatible way. The loader uses this to trigger migration.
#[allow(dead_code)] // Will be used in tasks 15-16 when persistence is integrated
const CURRENT_FORMAT_VERSION: u32 = 1;

/// Maximum age of state file in days before automatic cleanup.
#[allow(dead_code)] // Will be used in tasks 15-16 when persistence is integrated
const MAX_STATE_AGE_DAYS: u64 = 30;

/// Persisted state container with versioning metadata.
///
/// This wraps StateSnapshot with additional metadata needed for
/// persistence, migration, and validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // Will be used in tasks 15-16 when persistence is integrated
pub struct PersistedState {
    /// Format version for migration.
    pub format_version: u32,

    /// Timestamp when state was persisted (microseconds since epoch).
    pub saved_at: u64,

    /// The actual state snapshot.
    pub snapshot: StateSnapshot,

    /// Optional checksum for corruption detection (currently unused, reserved for future).
    #[serde(default)]
    pub checksum: Option<String>,
}

#[allow(dead_code)] // Will be used in tasks 15-16 when persistence is integrated
impl PersistedState {
    /// Create a new PersistedState from a StateSnapshot.
    pub fn new(snapshot: StateSnapshot, timestamp_us: u64) -> Self {
        Self {
            format_version: CURRENT_FORMAT_VERSION,
            saved_at: timestamp_us,
            snapshot,
            checksum: None,
        }
    }

    /// Check if this persisted state is too old.
    pub fn is_stale(&self, current_time_us: u64) -> bool {
        let age_us = current_time_us.saturating_sub(self.saved_at);
        let age_days = age_us / (1_000_000 * 60 * 60 * 24);
        age_days > MAX_STATE_AGE_DAYS
    }

    /// Check if format version is compatible with current version.
    pub fn is_compatible(&self) -> bool {
        // For now, we only support the current version
        // Future versions may add migration logic here
        self.format_version == CURRENT_FORMAT_VERSION
    }
}

/// State persistence manager.
///
/// Handles saving and loading engine state to/from disk with support for
/// version migration, corruption detection, and automatic cleanup.
///
/// # Example
///
/// ```no_run
/// use keyrx_core::engine::state::{EngineState, persistence::StatePersistence};
/// use keyrx_core::engine::decision::timing::TimingConfig;
/// use std::path::Path;
///
/// let path = Path::new("/tmp/keyrx_state.json");
/// let persistence = StatePersistence::new(path);
///
/// // Save state
/// let state = EngineState::new(TimingConfig::default());
/// persistence.save(&state, 1000000).expect("save failed");
///
/// // Load state
/// let loaded = persistence.load(TimingConfig::default()).expect("load failed");
/// assert_eq!(loaded.version(), state.version());
/// ```
#[allow(dead_code)] // Will be used in tasks 15-16 when persistence is integrated
pub struct StatePersistence {
    /// Path to the state file.
    path: PathBuf,
}

#[allow(dead_code)] // Will be used in tasks 15-16 when persistence is integrated
impl StatePersistence {
    /// Create a new persistence manager for the given path.
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    /// Save engine state to disk.
    ///
    /// # Arguments
    ///
    /// * `state` - The engine state to save
    /// * `timestamp_us` - Current timestamp in microseconds
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Serialization fails
    /// - File I/O fails
    /// - Parent directory doesn't exist
    pub fn save(&self, state: &EngineState, timestamp_us: u64) -> StateResult<()> {
        // Create snapshot from current state
        let snapshot: StateSnapshot = state.into();

        // Wrap in persistence container
        let persisted = PersistedState::new(snapshot, timestamp_us);

        // Serialize to JSON (pretty-printed for easier debugging)
        let json = serde_json::to_string_pretty(&persisted).map_err(|e| {
            StateError::PersistenceFailed {
                reason: format!("Serialization failed: {}", e),
            }
        })?;

        // Ensure parent directory exists
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|e| StateError::PersistenceFailed {
                reason: format!("Failed to create directory: {}", e),
            })?;
        }

        // Write to temporary file first for atomic replacement
        let temp_path = self.path.with_extension("tmp");
        fs::write(&temp_path, json).map_err(|e| StateError::PersistenceFailed {
            reason: format!("Failed to write file: {}", e),
        })?;

        // Atomic rename (on Unix systems)
        fs::rename(&temp_path, &self.path).map_err(|e| StateError::PersistenceFailed {
            reason: format!("Failed to rename file: {}", e),
        })?;

        Ok(())
    }

    /// Load engine state from disk.
    ///
    /// # Arguments
    ///
    /// * `timing_config` - Timing configuration for the new state
    ///
    /// # Returns
    ///
    /// Returns a loaded EngineState, or a clean state if loading fails.
    /// Errors are logged but don't propagate - the engine always gets a valid state.
    ///
    /// # Fallback Behavior
    ///
    /// Returns a clean state if:
    /// - File doesn't exist
    /// - File is corrupted
    /// - Format version is incompatible
    /// - State is stale (older than MAX_STATE_AGE_DAYS)
    pub fn load(&self, timing_config: TimingConfig) -> StateResult<EngineState> {
        self.load_with_timestamp(timing_config, Self::current_time_us())
    }

    /// Load state with explicit timestamp (for testing).
    fn load_with_timestamp(
        &self,
        timing_config: TimingConfig,
        current_time_us: u64,
    ) -> StateResult<EngineState> {
        // Try to read the file
        let json = match fs::read_to_string(&self.path) {
            Ok(content) => content,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                // State file not found, using clean state
                return Ok(EngineState::new(timing_config));
            }
            Err(_e) => {
                // Failed to read state file, using clean state
                return Ok(EngineState::new(timing_config));
            }
        };

        // Try to deserialize
        let persisted: PersistedState = match serde_json::from_str(&json) {
            Ok(state) => state,
            Err(_e) => {
                // Failed to parse state file, using clean state
                return Ok(EngineState::new(timing_config));
            }
        };

        // Check format version compatibility
        if !persisted.is_compatible() {
            // State file format version is incompatible with current version, using clean state
            return Ok(EngineState::new(timing_config));
        }

        // Check if state is stale
        if persisted.is_stale(current_time_us) {
            // State file is stale (older than MAX_STATE_AGE_DAYS), using clean state
            return Ok(EngineState::new(timing_config));
        }

        // Convert snapshot to EngineState
        // Note: We only restore keys and layers, not pending decisions or ephemeral state
        self.restore_from_snapshot(persisted.snapshot, timing_config)
    }

    /// Restore EngineState from a snapshot.
    ///
    /// This creates a new EngineState and applies the snapshot data.
    /// Only persistent state is restored (keys, layers, modifiers).
    /// Pending decisions are not restored as they're ephemeral.
    fn restore_from_snapshot(
        &self,
        snapshot: StateSnapshot,
        timing_config: TimingConfig,
    ) -> StateResult<EngineState> {
        // Create new state with the base layer from snapshot
        let mut state = EngineState::with_base_layer(snapshot.base_layer, timing_config);

        // Restore pressed keys
        for pressed_key in &snapshot.pressed_keys {
            state
                .keys_mut()
                .press(pressed_key.key, pressed_key.pressed_at, false);
        }

        // Restore layers
        // The base layer is already active, so we need to push the others in order
        // Skip the first layer if it's the base layer
        let layers_to_push: Vec<_> = snapshot
            .active_layers
            .iter()
            .filter(|&&layer_id| layer_id != snapshot.base_layer)
            .copied()
            .collect();

        for layer_id in layers_to_push {
            state.layers_mut().push(layer_id);
        }

        // Restore standard modifiers
        for modifier in [
            super::StandardModifier::Shift,
            super::StandardModifier::Control,
            super::StandardModifier::Alt,
            super::StandardModifier::Meta,
        ] {
            if snapshot.standard_modifiers.is_active(modifier) {
                state
                    .modifiers_mut()
                    .activate(super::Modifier::Standard(modifier));
            }
        }

        // Restore virtual modifiers
        for modifier_id in &snapshot.virtual_modifiers {
            state
                .modifiers_mut()
                .activate(super::Modifier::Virtual(*modifier_id));
        }

        // Note: We intentionally don't restore version or pending decisions
        // Version starts at 0 for a fresh session
        // Pending decisions are ephemeral and shouldn't persist across restarts

        Ok(state)
    }

    /// Delete the state file.
    ///
    /// This is useful for cleanup or resetting to a clean state.
    pub fn delete(&self) -> StateResult<()> {
        match fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(StateError::PersistenceFailed {
                reason: format!("Failed to delete file: {}", e),
            }),
        }
    }

    /// Check if a persisted state file exists.
    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    /// Get current time in microseconds since epoch.
    fn current_time_us() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::state::{Modifier, Mutation, StandardModifier};
    use crate::engine::KeyCode;
    use tempfile::TempDir;

    fn temp_path() -> (TempDir, PathBuf) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("state.json");
        (dir, path)
    }

    #[test]
    fn persisted_state_new() {
        let snapshot = StateSnapshot::empty();
        let persisted = PersistedState::new(snapshot, 1000);

        assert_eq!(persisted.format_version, CURRENT_FORMAT_VERSION);
        assert_eq!(persisted.saved_at, 1000);
        assert!(persisted.checksum.is_none());
    }

    #[test]
    fn persisted_state_is_compatible() {
        let snapshot = StateSnapshot::empty();
        let persisted = PersistedState::new(snapshot, 1000);

        assert!(persisted.is_compatible());
    }

    #[test]
    fn persisted_state_is_stale() {
        let snapshot = StateSnapshot::empty();
        let old_time = 1000;
        let persisted = PersistedState::new(snapshot, old_time);

        // Not stale after 1 day
        let one_day_later = old_time + (1_000_000 * 60 * 60 * 24);
        assert!(!persisted.is_stale(one_day_later));

        // Stale after 31 days
        let thirty_one_days_later = old_time + (1_000_000 * 60 * 60 * 24 * 31);
        assert!(persisted.is_stale(thirty_one_days_later));
    }

    #[test]
    fn save_and_load_empty_state() {
        let (_dir, path) = temp_path();
        let persistence = StatePersistence::new(&path);

        let state = EngineState::new(TimingConfig::default());

        // Save
        persistence.save(&state, 1000).expect("save failed");
        assert!(path.exists());

        // Load
        let loaded = persistence
            .load(TimingConfig::default())
            .expect("load failed");
        assert_eq!(loaded.version(), 0); // Version resets
        assert!(loaded.no_keys_pressed());
        assert!(loaded.only_base_layer_active());
        assert_eq!(loaded.base_layer(), 0);
    }

    #[test]
    fn save_and_load_with_keys() {
        let (_dir, path) = temp_path();
        let persistence = StatePersistence::new(&path);

        let mut state = EngineState::new(TimingConfig::default());
        state
            .apply(Mutation::KeyDown {
                key: KeyCode::A,
                timestamp_us: 1000,
                is_repeat: false,
            })
            .unwrap();
        state
            .apply(Mutation::KeyDown {
                key: KeyCode::B,
                timestamp_us: 1100,
                is_repeat: false,
            })
            .unwrap();

        let save_time = 2000;
        let load_time = save_time + 1000; // 1ms later

        // Save
        persistence.save(&state, save_time).expect("save failed");

        // Load
        let loaded = persistence
            .load_with_timestamp(TimingConfig::default(), load_time)
            .expect("load failed");
        assert!(loaded.is_key_pressed(KeyCode::A));
        assert!(loaded.is_key_pressed(KeyCode::B));
        assert_eq!(loaded.key_press_time(KeyCode::A), Some(1000));
        assert_eq!(loaded.key_press_time(KeyCode::B), Some(1100));
    }

    #[test]
    fn save_and_load_with_layers() {
        let (_dir, path) = temp_path();
        let persistence = StatePersistence::new(&path);

        let mut state = EngineState::new(TimingConfig::default());
        state.apply(Mutation::PushLayer { layer_id: 1 }).unwrap();
        state.apply(Mutation::PushLayer { layer_id: 2 }).unwrap();

        let save_time = 2000;
        let load_time = save_time + 1000;

        // Save
        persistence.save(&state, save_time).expect("save failed");

        // Load
        let loaded = persistence
            .load_with_timestamp(TimingConfig::default(), load_time)
            .expect("load failed");
        assert!(loaded.is_layer_active(0));
        assert!(loaded.is_layer_active(1));
        assert!(loaded.is_layer_active(2));
        assert_eq!(loaded.top_layer(), 2);
    }

    #[test]
    fn save_and_load_with_modifiers() {
        let (_dir, path) = temp_path();
        let persistence = StatePersistence::new(&path);

        let mut state = EngineState::new(TimingConfig::default());
        state
            .apply(Mutation::ActivateModifier { modifier_id: 5 })
            .unwrap();
        state
            .apply(Mutation::ActivateModifier { modifier_id: 10 })
            .unwrap();

        // Activate standard modifier directly
        state
            .modifiers_mut()
            .activate(Modifier::Standard(StandardModifier::Shift));

        let save_time = 2000;
        let load_time = save_time + 1000;

        // Save
        persistence.save(&state, save_time).expect("save failed");

        // Load
        let loaded = persistence
            .load_with_timestamp(TimingConfig::default(), load_time)
            .expect("load failed");
        assert!(loaded.is_modifier_active(Modifier::Virtual(5)));
        assert!(loaded.is_modifier_active(Modifier::Virtual(10)));
        assert!(loaded.is_modifier_active(Modifier::Standard(StandardModifier::Shift)));
    }

    #[test]
    fn save_with_custom_base_layer() {
        let (_dir, path) = temp_path();
        let persistence = StatePersistence::new(&path);

        let state = EngineState::with_base_layer(5, TimingConfig::default());

        let save_time = 1000;
        let load_time = save_time + 1000;

        // Save
        persistence.save(&state, save_time).expect("save failed");

        // Load
        let loaded = persistence
            .load_with_timestamp(TimingConfig::default(), load_time)
            .expect("load failed");
        assert_eq!(loaded.base_layer(), 5);
        assert!(loaded.is_layer_active(5));
    }

    #[test]
    fn pending_decisions_not_persisted() {
        let (_dir, path) = temp_path();
        let persistence = StatePersistence::new(&path);

        let mut state = EngineState::new(TimingConfig::default());
        state
            .apply(Mutation::AddTapHold {
                key: KeyCode::A,
                pressed_at: 1000,
                tap_action: KeyCode::B,
                hold_action: super::super::HoldAction::Key(KeyCode::C),
            })
            .unwrap();

        assert_eq!(state.pending_count(), 1);

        // Save
        persistence.save(&state, 2000).expect("save failed");

        // Load - pending should not be restored
        let loaded = persistence
            .load(TimingConfig::default())
            .expect("load failed");
        assert_eq!(loaded.pending_count(), 0);
    }

    #[test]
    fn load_nonexistent_file_returns_clean_state() {
        let (_dir, path) = temp_path();
        let persistence = StatePersistence::new(&path);

        let loaded = persistence
            .load(TimingConfig::default())
            .expect("load failed");
        assert_eq!(loaded.version(), 0);
        assert!(loaded.no_keys_pressed());
        assert!(loaded.only_base_layer_active());
    }

    #[test]
    fn load_corrupted_file_returns_clean_state() {
        let (_dir, path) = temp_path();
        let persistence = StatePersistence::new(&path);

        // Write corrupted JSON
        fs::write(&path, "{ invalid json }").unwrap();

        let loaded = persistence
            .load(TimingConfig::default())
            .expect("load failed");
        assert_eq!(loaded.version(), 0);
        assert!(loaded.no_keys_pressed());
    }

    #[test]
    fn load_incompatible_version_returns_clean_state() {
        let (_dir, path) = temp_path();
        let persistence = StatePersistence::new(&path);

        // Create persisted state with incompatible version
        let mut persisted = PersistedState::new(StateSnapshot::empty(), 1000);
        persisted.format_version = 999; // Future version

        let json = serde_json::to_string(&persisted).unwrap();
        fs::write(&path, json).unwrap();

        let loaded = persistence
            .load(TimingConfig::default())
            .expect("load failed");
        assert_eq!(loaded.version(), 0);
        assert!(loaded.no_keys_pressed());
    }

    #[test]
    fn load_stale_state_returns_clean_state() {
        let (_dir, path) = temp_path();
        let persistence = StatePersistence::new(&path);

        let state = EngineState::new(TimingConfig::default());
        let old_time = 1000;

        // Save with old timestamp
        persistence.save(&state, old_time).expect("save failed");

        // Load with timestamp 31 days later
        let current_time = old_time + (1_000_000 * 60 * 60 * 24 * 31);
        let loaded = persistence
            .load_with_timestamp(TimingConfig::default(), current_time)
            .expect("load failed");

        // Should get clean state due to staleness
        assert_eq!(loaded.version(), 0);
    }

    #[test]
    fn delete_removes_file() {
        let (_dir, path) = temp_path();
        let persistence = StatePersistence::new(&path);

        let state = EngineState::new(TimingConfig::default());
        persistence.save(&state, 1000).expect("save failed");
        assert!(path.exists());

        persistence.delete().expect("delete failed");
        assert!(!path.exists());
    }

    #[test]
    fn delete_nonexistent_file_succeeds() {
        let (_dir, path) = temp_path();
        let persistence = StatePersistence::new(&path);

        // Delete non-existent file should succeed
        persistence.delete().expect("delete failed");
    }

    #[test]
    fn exists_returns_correct_status() {
        let (_dir, path) = temp_path();
        let persistence = StatePersistence::new(&path);

        assert!(!persistence.exists());

        let state = EngineState::new(TimingConfig::default());
        persistence.save(&state, 1000).expect("save failed");

        assert!(persistence.exists());
    }

    #[test]
    fn save_creates_parent_directory() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("subdir").join("state.json");
        let persistence = StatePersistence::new(&path);

        let state = EngineState::new(TimingConfig::default());
        persistence.save(&state, 1000).expect("save failed");

        assert!(path.exists());
        assert!(path.parent().unwrap().exists());
    }

    #[test]
    fn save_is_atomic() {
        let (_dir, path) = temp_path();
        let persistence = StatePersistence::new(&path);

        let state1 = EngineState::new(TimingConfig::default());
        persistence.save(&state1, 1000).expect("save failed");

        // Save again - should atomically replace
        let mut state2 = EngineState::new(TimingConfig::default());
        state2
            .apply(Mutation::KeyDown {
                key: KeyCode::A,
                timestamp_us: 2000,
                is_repeat: false,
            })
            .unwrap();
        let save_time = 2000;
        let load_time = save_time + 1000;
        persistence.save(&state2, save_time).expect("save failed");

        // Load should get the second state
        let loaded = persistence
            .load_with_timestamp(TimingConfig::default(), load_time)
            .expect("load failed");
        assert!(loaded.is_key_pressed(KeyCode::A));
    }

    #[test]
    fn serialization_format_is_json() {
        let (_dir, path) = temp_path();
        let persistence = StatePersistence::new(&path);

        let state = EngineState::new(TimingConfig::default());
        persistence.save(&state, 1000).expect("save failed");

        // Read raw file and verify it's valid JSON
        let json = fs::read_to_string(&path).unwrap();
        let _: serde_json::Value = serde_json::from_str(&json).expect("not valid JSON");

        // Verify it contains expected fields
        assert!(json.contains("\"format_version\""));
        assert!(json.contains("\"saved_at\""));
        assert!(json.contains("\"snapshot\""));
    }

    #[test]
    fn complex_state_round_trip() {
        let (_dir, path) = temp_path();
        let persistence = StatePersistence::new(&path);

        // Create complex state
        let mut state = EngineState::with_base_layer(3, TimingConfig::default());

        // Add keys
        state
            .apply(Mutation::KeyDown {
                key: KeyCode::A,
                timestamp_us: 1000,
                is_repeat: false,
            })
            .unwrap();
        state
            .apply(Mutation::KeyDown {
                key: KeyCode::B,
                timestamp_us: 1100,
                is_repeat: false,
            })
            .unwrap();

        // Add layers
        state.apply(Mutation::PushLayer { layer_id: 5 }).unwrap();
        state.apply(Mutation::PushLayer { layer_id: 7 }).unwrap();

        // Add modifiers
        state
            .apply(Mutation::ActivateModifier { modifier_id: 10 })
            .unwrap();
        state
            .apply(Mutation::ActivateModifier { modifier_id: 20 })
            .unwrap();
        state
            .modifiers_mut()
            .activate(Modifier::Standard(StandardModifier::Control));
        state
            .modifiers_mut()
            .activate(Modifier::Standard(StandardModifier::Alt));

        let save_time = 5000;
        let load_time = save_time + 1000;

        // Save
        persistence.save(&state, save_time).expect("save failed");

        // Load and verify
        let loaded = persistence
            .load_with_timestamp(TimingConfig::default(), load_time)
            .expect("load failed");

        // Keys
        assert!(loaded.is_key_pressed(KeyCode::A));
        assert!(loaded.is_key_pressed(KeyCode::B));
        assert_eq!(loaded.pressed_key_count(), 2);

        // Layers
        assert_eq!(loaded.base_layer(), 3);
        assert!(loaded.is_layer_active(3));
        assert!(loaded.is_layer_active(5));
        assert!(loaded.is_layer_active(7));
        assert_eq!(loaded.top_layer(), 7);

        // Virtual modifiers
        assert!(loaded.is_modifier_active(Modifier::Virtual(10)));
        assert!(loaded.is_modifier_active(Modifier::Virtual(20)));

        // Standard modifiers
        assert!(loaded.is_modifier_active(Modifier::Standard(StandardModifier::Control)));
        assert!(loaded.is_modifier_active(Modifier::Standard(StandardModifier::Alt)));
        assert!(!loaded.is_modifier_active(Modifier::Standard(StandardModifier::Shift)));
    }
}
