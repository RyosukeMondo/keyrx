//! Validation configuration module.
//!
//! Loads validation thresholds from `~/.config/keyrx/validation.toml`
//! with sensible defaults when the file is missing or malformed.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Validation configuration - all thresholds centralized here.
/// Loaded from ~/.config/keyrx/validation.toml or uses defaults.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ValidationConfig {
    /// Maximum errors to report before stopping (prevents flood)
    pub max_errors: usize,
    /// Maximum suggestions for invalid key names
    pub max_suggestions: usize,
    /// Levenshtein distance threshold for "similar" keys
    pub similarity_threshold: usize,
    /// Number of blocked keys before warning user
    pub blocked_keys_warning_threshold: usize,
    /// Maximum depth for circular remap detection (A→B→C→...→A)
    pub max_cycle_depth: usize,
    /// Tap timeout range for warnings [min, max] in ms
    pub tap_timeout_warn_range: (u32, u32),
    /// Combo timeout range for warnings [min, max] in ms
    pub combo_timeout_warn_range: (u32, u32),
    /// Debounce delay for Flutter UI validation (ms)
    pub ui_validation_debounce_ms: u32,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            max_errors: 20,
            max_suggestions: 5,
            similarity_threshold: 3,
            blocked_keys_warning_threshold: 10,
            max_cycle_depth: 10,
            tap_timeout_warn_range: (50, 500),
            combo_timeout_warn_range: (10, 100),
            ui_validation_debounce_ms: 500,
        }
    }
}

impl ValidationConfig {
    /// Load from default config path or return defaults.
    pub fn load() -> Self {
        Self::load_from_path(config_path()).unwrap_or_default()
    }

    /// Load from specific path (for testing).
    /// Returns None if file doesn't exist or is malformed.
    pub fn load_from_path(path: impl AsRef<Path>) -> Option<Self> {
        let content = std::fs::read_to_string(path).ok()?;
        toml::from_str(&content).ok()
    }
}

/// Returns the default config file path: ~/.config/keyrx/validation.toml
fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("keyrx")
        .join("validation.toml")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values_are_sensible() {
        let config = ValidationConfig::default();
        assert_eq!(config.max_errors, 20);
        assert_eq!(config.max_suggestions, 5);
        assert_eq!(config.similarity_threshold, 3);
        assert_eq!(config.blocked_keys_warning_threshold, 10);
        assert_eq!(config.max_cycle_depth, 10);
        assert_eq!(config.tap_timeout_warn_range, (50, 500));
        assert_eq!(config.combo_timeout_warn_range, (10, 100));
        assert_eq!(config.ui_validation_debounce_ms, 500);
    }

    #[test]
    fn load_returns_defaults_when_no_file() {
        let config = ValidationConfig::load_from_path("/nonexistent/path.toml");
        assert!(config.is_none());
    }

    #[test]
    fn load_parses_valid_toml() {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("validation.toml");
        let mut file = std::fs::File::create(&path).unwrap();
        writeln!(file, "max_errors = 50").unwrap();
        writeln!(file, "max_suggestions = 10").unwrap();

        let config = ValidationConfig::load_from_path(&path).unwrap();
        assert_eq!(config.max_errors, 50);
        assert_eq!(config.max_suggestions, 10);
        // Other fields should be defaults
        assert_eq!(config.similarity_threshold, 3);
    }

    #[test]
    fn malformed_toml_returns_none() {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.toml");
        let mut file = std::fs::File::create(&path).unwrap();
        writeln!(file, "this is not valid toml {{{{").unwrap();

        let config = ValidationConfig::load_from_path(&path);
        assert!(config.is_none());
    }

    #[test]
    fn config_path_is_valid() {
        let path = config_path();
        assert!(path.to_string_lossy().contains("keyrx"));
        assert!(path.to_string_lossy().contains("validation.toml"));
    }
}
