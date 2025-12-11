//! Runtime configuration loading from TOML.
//!
//! This module provides functionality for loading and validating
//! configuration from TOML files, with support for CLI overrides.
//!
//! # Submodules
//!
//! - [`parsing`]: TOML parsing, file I/O, and schema validation
//! - [`validation`]: Configuration validation and value clamping
//!
//! # Example
//!
//! ```no_run
//! use keyrx_core::config::{Config, load_config, merge_cli_overrides};
//!
//! // Load from default path or specified path
//! let mut config = load_config(None);
//!
//! // Apply CLI overrides
//! merge_cli_overrides(&mut config, Some(150), Some(40), None);
//! ```

mod parsing;
mod validation;

use super::limits::{DEFAULT_EVENT_GAP_US, DEFAULT_REGRESSION_THRESHOLD_US, LATENCY_THRESHOLD_NS};
use super::migration::CURRENT_CONFIG_VERSION;
use super::timing::{DEFAULT_COMBO_TIMEOUT_MS, DEFAULT_HOLD_DELAY_MS, DEFAULT_TAP_TIMEOUT_MS};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub use parsing::load_config;
pub use validation::{merge_cli_overrides, validate_and_clamp};

// =============================================================================
// Configuration Structures
// =============================================================================

/// Root configuration structure.
///
/// This contains all configurable settings for KeyRx, organized into
/// logical sections.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct Config {
    /// Config format version for migration support.
    #[serde(default = "default_config_version")]
    pub version: u32,

    /// Timing configuration for tap-hold detection and combos.
    #[serde(default)]
    pub timing: TimingSection,

    /// UI-related configuration.
    #[serde(default)]
    pub ui: UiSection,

    /// Performance thresholds and limits.
    #[serde(default)]
    pub performance: PerformanceSection,

    /// Path configuration.
    #[serde(default)]
    pub paths: PathsSection,

    /// Scripting configuration.
    #[serde(default)]
    pub scripting: super::scripting::ScriptingSection,
}

fn default_config_version() -> u32 {
    CURRENT_CONFIG_VERSION
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: default_config_version(),
            timing: TimingSection::default(),
            ui: UiSection::default(),
            performance: PerformanceSection::default(),
            paths: PathsSection::default(),
            scripting: super::scripting::ScriptingSection::default(),
        }
    }
}

/// Timing configuration section.
///
/// Controls the behavior of tap-hold detection and combo key windows.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct TimingSection {
    /// Duration (ms) to distinguish tap from hold.
    ///
    /// Valid range: 50-1000 ms. Default: 200.
    #[serde(default = "default_tap_timeout")]
    pub tap_timeout_ms: u32,

    /// Window (ms) for detecting simultaneous keypresses as a combo.
    ///
    /// Valid range: 10-200 ms. Default: 50.
    #[serde(default = "default_combo_timeout")]
    pub combo_timeout_ms: u32,

    /// Delay (ms) before considering a key press as a hold.
    ///
    /// Valid range: 0-500 ms. Default: 0.
    #[serde(default = "default_hold_delay")]
    pub hold_delay_ms: u32,
}

fn default_tap_timeout() -> u32 {
    DEFAULT_TAP_TIMEOUT_MS
}

fn default_combo_timeout() -> u32 {
    DEFAULT_COMBO_TIMEOUT_MS
}

fn default_hold_delay() -> u32 {
    DEFAULT_HOLD_DELAY_MS
}

impl Default for TimingSection {
    fn default() -> Self {
        Self {
            tap_timeout_ms: default_tap_timeout(),
            combo_timeout_ms: default_combo_timeout(),
            hold_delay_ms: default_hold_delay(),
        }
    }
}

/// UI configuration section.
///
/// Controls UI behavior in the Flutter debugger and visualizer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct UiSection {
    /// Maximum number of events to keep in history.
    ///
    /// Valid range: 50-1000. Default: 300.
    #[serde(default = "default_max_events_history")]
    pub max_events_history: usize,

    /// Animation duration for UI transitions (ms).
    ///
    /// Valid range: 50-500 ms. Default: 150.
    #[serde(default = "default_animation_duration")]
    pub animation_duration_ms: u32,
}

fn default_max_events_history() -> usize {
    300
}

fn default_animation_duration() -> u32 {
    150
}

impl Default for UiSection {
    fn default() -> Self {
        Self {
            max_events_history: default_max_events_history(),
            animation_duration_ms: default_animation_duration(),
        }
    }
}

/// Performance configuration section.
///
/// Controls performance thresholds and warnings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct PerformanceSection {
    /// Latency warning threshold in microseconds.
    ///
    /// Latencies above this trigger a warning. Default: 20000 (20ms).
    #[serde(default = "default_latency_warning")]
    pub latency_warning_us: u64,

    /// Latency caution threshold in microseconds.
    ///
    /// Latencies above this trigger a caution. Default: 10000 (10ms).
    #[serde(default = "default_latency_caution")]
    pub latency_caution_us: u64,

    /// Regression threshold in microseconds for benchmark comparisons.
    ///
    /// Valid range: 10-1000 μs. Default: 100.
    #[serde(default = "default_regression_threshold")]
    pub regression_threshold_us: u64,

    /// Benchmark latency threshold in nanoseconds.
    ///
    /// Used for pass/fail determination in benchmarks. Default: 1_000_000 (1ms).
    #[serde(default = "default_latency_threshold_ns")]
    pub latency_threshold_ns: u64,

    /// Default gap between simulated events in microseconds.
    ///
    /// Default: 1000 (1ms).
    #[serde(default = "default_event_gap")]
    pub event_gap_us: u64,
}

fn default_latency_warning() -> u64 {
    20_000
}

fn default_latency_caution() -> u64 {
    10_000
}

fn default_regression_threshold() -> u64 {
    DEFAULT_REGRESSION_THRESHOLD_US
}

fn default_latency_threshold_ns() -> u64 {
    LATENCY_THRESHOLD_NS
}

fn default_event_gap() -> u64 {
    DEFAULT_EVENT_GAP_US
}

impl Default for PerformanceSection {
    fn default() -> Self {
        Self {
            latency_warning_us: default_latency_warning(),
            latency_caution_us: default_latency_caution(),
            regression_threshold_us: default_regression_threshold(),
            latency_threshold_ns: default_latency_threshold_ns(),
            event_gap_us: default_event_gap(),
        }
    }
}

/// Paths configuration section.
///
/// Configures directories and file paths.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct PathsSection {
    /// Directory for user scripts.
    ///
    /// Relative to config directory if not absolute. Default: "scripts".
    #[serde(default = "default_scripts_dir")]
    pub scripts_dir: String,

    /// Directory for temporary files.
    ///
    /// Default: system temp directory.
    #[serde(default = "default_temp_dir")]
    pub temp_dir: String,
}

fn default_scripts_dir() -> String {
    "scripts".to_string()
}

fn default_temp_dir() -> String {
    std::env::temp_dir().to_string_lossy().into_owned()
}

impl Default for PathsSection {
    fn default() -> Self {
        Self {
            scripts_dir: default_scripts_dir(),
            temp_dir: default_temp_dir(),
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::limits::{
        DEFAULT_EVENT_GAP_US, DEFAULT_REGRESSION_THRESHOLD_US, LATENCY_THRESHOLD_NS,
    };
    use crate::config::migration::CURRENT_CONFIG_VERSION;
    use crate::config::timing::{
        DEFAULT_COMBO_TIMEOUT_MS, DEFAULT_HOLD_DELAY_MS, DEFAULT_TAP_TIMEOUT_MS,
    };
    use crate::discovery::test_utils::config_env_lock;
    use std::env;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn config_default_matches_constants() {
        let config = Config::default();

        assert_eq!(config.version, CURRENT_CONFIG_VERSION);
        assert_eq!(config.timing.tap_timeout_ms, DEFAULT_TAP_TIMEOUT_MS);
        assert_eq!(config.timing.combo_timeout_ms, DEFAULT_COMBO_TIMEOUT_MS);
        assert_eq!(config.timing.hold_delay_ms, DEFAULT_HOLD_DELAY_MS);
        assert_eq!(
            config.performance.regression_threshold_us,
            DEFAULT_REGRESSION_THRESHOLD_US
        );
        assert_eq!(
            config.performance.latency_threshold_ns,
            LATENCY_THRESHOLD_NS
        );
        assert_eq!(config.performance.event_gap_us, DEFAULT_EVENT_GAP_US);
    }

    #[test]
    fn config_default_ui_values() {
        let config = Config::default();

        assert_eq!(config.ui.max_events_history, 300);
        assert_eq!(config.ui.animation_duration_ms, 150);
    }

    #[test]
    fn config_default_performance_values() {
        let config = Config::default();

        assert_eq!(config.performance.latency_warning_us, 20_000);
        assert_eq!(config.performance.latency_caution_us, 10_000);
    }

    #[test]
    fn config_serde_roundtrip() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).expect("serialize");
        let decoded: Config = toml::from_str(&toml_str).expect("deserialize");
        assert_eq!(decoded, config);
    }

    #[test]
    fn config_partial_toml_uses_defaults() {
        let toml_str = r#"
[timing]
tap_timeout_ms = 175

[ui]
max_events_history = 500
"#;
        let config: Config = toml::from_str(toml_str).expect("parse");

        assert_eq!(config.timing.tap_timeout_ms, 175);
        assert_eq!(config.timing.combo_timeout_ms, DEFAULT_COMBO_TIMEOUT_MS);
        assert_eq!(config.timing.hold_delay_ms, DEFAULT_HOLD_DELAY_MS);
        assert_eq!(config.ui.max_events_history, 500);
        assert_eq!(config.ui.animation_duration_ms, 150);
        assert_eq!(
            config.performance.regression_threshold_us,
            DEFAULT_REGRESSION_THRESHOLD_US
        );
    }

    #[test]
    fn load_config_returns_defaults_when_file_not_found() {
        let temp = tempdir().unwrap();
        let nonexistent = temp.path().join("nonexistent.toml");

        let config = load_config(Some(&nonexistent));
        assert_eq!(config, Config::default());
    }

    #[test]
    fn load_config_from_file() {
        let temp = tempdir().unwrap();
        let config_path = temp.path().join("config.toml");

        let toml_content = r#"
[timing]
tap_timeout_ms = 180
combo_timeout_ms = 60

[performance]
latency_warning_us = 25000
latency_caution_us = 12000
"#;
        fs::write(&config_path, toml_content).unwrap();

        let config = load_config(Some(&config_path));

        assert_eq!(config.timing.tap_timeout_ms, 180);
        assert_eq!(config.timing.combo_timeout_ms, 60);
        assert_eq!(config.timing.hold_delay_ms, DEFAULT_HOLD_DELAY_MS);
        assert_eq!(config.performance.latency_warning_us, 25000);
        assert_eq!(config.performance.latency_caution_us, 12000);
    }

    #[test]
    fn load_config_migrates_unversioned_file() {
        let temp = tempdir().unwrap();
        let config_path = temp.path().join("config.toml");

        let toml_content = r#"
[timing]
tap_timeout_ms = 180
"#;
        fs::write(&config_path, toml_content).unwrap();

        let config = load_config(Some(&config_path));
        assert_eq!(config.version, CURRENT_CONFIG_VERSION);

        let rewritten = fs::read_to_string(&config_path).unwrap();
        assert!(
            rewritten.contains("version"),
            "version field should be written during migration"
        );
    }

    #[test]
    fn load_config_handles_invalid_toml() {
        let temp = tempdir().unwrap();
        let config_path = temp.path().join("config.toml");

        fs::write(&config_path, "this is not valid [toml").unwrap();

        let config = load_config(Some(&config_path));
        assert_eq!(config, Config::default());
    }

    #[test]
    fn load_config_schema_validation_failure_uses_default() {
        let temp = tempdir().unwrap();
        let config_path = temp.path().join("config.toml");

        let toml_content = r#"
[timing]
tap_timeout_ms = "fast"
"#;
        fs::write(&config_path, toml_content).unwrap();

        let config = load_config(Some(&config_path));

        assert_eq!(config, Config::default());
    }

    #[test]
    fn validate_clamps_tap_timeout_low() {
        let mut config = Config::default();
        config.timing.tap_timeout_ms = 10;

        validate_and_clamp(&mut config);

        assert_eq!(config.timing.tap_timeout_ms, 50);
    }

    #[test]
    fn validate_clamps_tap_timeout_high() {
        let mut config = Config::default();
        config.timing.tap_timeout_ms = 2000;

        validate_and_clamp(&mut config);

        assert_eq!(config.timing.tap_timeout_ms, 1000);
    }

    #[test]
    fn validate_clamps_combo_timeout() {
        let mut config = Config::default();
        config.timing.combo_timeout_ms = 5;

        validate_and_clamp(&mut config);

        assert_eq!(config.timing.combo_timeout_ms, 10);
    }

    #[test]
    fn validate_clamps_hold_delay() {
        let mut config = Config::default();
        config.timing.hold_delay_ms = 1000;

        validate_and_clamp(&mut config);

        assert_eq!(config.timing.hold_delay_ms, 500);
    }

    #[test]
    fn validate_clamps_max_events_history() {
        let mut config = Config::default();
        config.ui.max_events_history = 10;

        validate_and_clamp(&mut config);

        assert_eq!(config.ui.max_events_history, 50);
    }

    #[test]
    fn validate_adjusts_caution_when_greater_than_warning() {
        let mut config = Config::default();
        config.performance.latency_warning_us = 10000;
        config.performance.latency_caution_us = 15000;

        validate_and_clamp(&mut config);

        assert!(config.performance.latency_caution_us < config.performance.latency_warning_us);
        assert_eq!(config.performance.latency_caution_us, 5000);
    }

    #[test]
    fn merge_cli_overrides_applies_values() {
        let mut config = Config::default();

        merge_cli_overrides(&mut config, Some(150), Some(40), Some(10));

        assert_eq!(config.timing.tap_timeout_ms, 150);
        assert_eq!(config.timing.combo_timeout_ms, 40);
        assert_eq!(config.timing.hold_delay_ms, 10);
    }

    #[test]
    fn merge_cli_overrides_none_leaves_unchanged() {
        let mut config = Config::default();
        let original_tap = config.timing.tap_timeout_ms;
        let original_combo = config.timing.combo_timeout_ms;

        merge_cli_overrides(&mut config, None, None, Some(5));

        assert_eq!(config.timing.tap_timeout_ms, original_tap);
        assert_eq!(config.timing.combo_timeout_ms, original_combo);
        assert_eq!(config.timing.hold_delay_ms, 5);
    }

    #[test]
    fn merge_cli_overrides_validates() {
        let mut config = Config::default();

        // Pass invalid values - should be clamped
        merge_cli_overrides(&mut config, Some(10000), None, None);

        assert_eq!(config.timing.tap_timeout_ms, 1000); // clamped to max
    }

    #[test]
    fn load_config_uses_default_path_when_none() {
        // Use shared lock for XDG_CONFIG_HOME modification
        let _guard = config_env_lock().lock().unwrap();

        // Create a temp directory and set it as XDG_CONFIG_HOME
        let temp = tempdir().unwrap();
        let prev_xdg = env::var("XDG_CONFIG_HOME").ok();
        let prev_home = env::var("HOME").ok();

        env::set_var("XDG_CONFIG_HOME", temp.path());
        // Remove HOME to ensure XDG_CONFIG_HOME is used
        env::remove_var("HOME");

        // Create keyrx config directory
        let keyrx_dir = temp.path().join("keyrx");
        fs::create_dir_all(&keyrx_dir).unwrap();

        // Write config file
        let config_path = keyrx_dir.join("config.toml");
        let toml_content = r#"
[timing]
tap_timeout_ms = 222
"#;
        fs::write(&config_path, toml_content).unwrap();

        // Load without specifying path
        let config = load_config(None);

        assert_eq!(config.timing.tap_timeout_ms, 222);

        // Restore environment
        match prev_xdg {
            Some(val) => env::set_var("XDG_CONFIG_HOME", val),
            None => env::remove_var("XDG_CONFIG_HOME"),
        }
        match prev_home {
            Some(val) => env::set_var("HOME", val),
            None => env::remove_var("HOME"),
        }
    }

    #[test]
    fn config_includes_scripting_section() {
        use crate::scripting::sandbox::ScriptMode;

        let config = Config::default();
        assert_eq!(config.scripting.mode, ScriptMode::Standard);
    }

    #[test]
    fn config_scripting_from_toml() {
        use crate::scripting::sandbox::ScriptMode;

        let toml_str = r#"
[scripting]
mode = "Safe"
"#;
        let config: Config = toml::from_str(toml_str).expect("parse");
        assert_eq!(config.scripting.mode, ScriptMode::Safe);
    }

    #[test]
    fn config_scripting_partial_toml_uses_defaults() {
        use crate::scripting::sandbox::ScriptMode;

        let toml_str = r#"
[timing]
tap_timeout_ms = 175
"#;
        let config: Config = toml::from_str(toml_str).expect("parse");
        assert_eq!(config.scripting.mode, ScriptMode::Standard);
    }
}
