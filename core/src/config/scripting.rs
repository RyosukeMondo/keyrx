//! Scripting configuration.
//!
//! This module provides configuration for script execution security and behavior.

use crate::scripting::sandbox::ScriptMode;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Scripting configuration section.
///
/// Controls script execution mode and security settings.
///
/// # Examples
///
/// ```
/// use keyrx_core::config::scripting::ScriptingSection;
/// use keyrx_core::scripting::sandbox::ScriptMode;
///
/// let config = ScriptingSection::default();
/// assert_eq!(config.mode, ScriptMode::Standard);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct ScriptingSection {
    /// Script execution mode controlling available functions.
    ///
    /// - `safe`: Only safe functions (no side effects)
    /// - `standard`: Safe + standard functions (keyboard operations) - **default**
    /// - `full`: All functions including system interaction
    ///
    /// Default: `standard`
    #[serde(default)]
    pub mode: ScriptMode,
}

impl Default for ScriptingSection {
    fn default() -> Self {
        Self {
            mode: ScriptMode::Standard,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_script_mode() {
        let config = ScriptingSection::default();
        assert_eq!(config.mode, ScriptMode::Standard);
    }

    #[test]
    fn test_scripting_section_serde_roundtrip() {
        let config = ScriptingSection {
            mode: ScriptMode::Full,
        };
        let toml_str = toml::to_string(&config).expect("serialize");
        let decoded: ScriptingSection = toml::from_str(&toml_str).expect("deserialize");
        assert_eq!(decoded, config);
    }

    #[test]
    fn test_scripting_section_from_toml_safe() {
        let toml_str = r#"mode = "Safe""#;
        let config: ScriptingSection = toml::from_str(toml_str).expect("parse");
        assert_eq!(config.mode, ScriptMode::Safe);
    }

    #[test]
    fn test_scripting_section_from_toml_standard() {
        let toml_str = r#"mode = "Standard""#;
        let config: ScriptingSection = toml::from_str(toml_str).expect("parse");
        assert_eq!(config.mode, ScriptMode::Standard);
    }

    #[test]
    fn test_scripting_section_from_toml_full() {
        let toml_str = r#"mode = "Full""#;
        let config: ScriptingSection = toml::from_str(toml_str).expect("parse");
        assert_eq!(config.mode, ScriptMode::Full);
    }

    #[test]
    fn test_scripting_section_partial_toml_uses_default() {
        let toml_str = r#""#;
        let config: ScriptingSection = toml::from_str(toml_str).expect("parse");
        assert_eq!(config.mode, ScriptMode::Standard);
    }
}
