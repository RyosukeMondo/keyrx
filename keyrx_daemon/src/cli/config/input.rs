//! Layer 1: Input parsing and validation.

use crate::config::rhai_generator::MacroStep;
use crate::error::CliError;
use std::path::PathBuf;

/// Determines the config directory from various sources.
pub fn determine_config_dir(config_dir: Option<PathBuf>) -> PathBuf {
    config_dir
        .or_else(|| std::env::var("KEYRX_CONFIG_DIR").ok().map(PathBuf::from))
        .unwrap_or_else(|| {
            let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
            path.push("keyrx");
            path
        })
}

/// Parses a macro sequence from string format.
///
/// Format: "press:VK_A,wait:50,release:VK_A"
pub fn parse_macro_sequence(sequence: &str) -> Result<Vec<MacroStep>, CliError> {
    let mut steps = Vec::new();

    for part in sequence.split(',') {
        let part = part.trim();
        if let Some(key) = part.strip_prefix("press:") {
            steps.push(MacroStep::Press(key.to_string()));
        } else if let Some(key) = part.strip_prefix("release:") {
            steps.push(MacroStep::Release(key.to_string()));
        } else if let Some(ms) = part.strip_prefix("wait:") {
            let ms = ms.parse::<u16>().map_err(|_| CliError::InvalidArguments {
                reason: format!("Invalid wait time: {}", ms),
            })?;
            steps.push(MacroStep::Wait(ms));
        } else {
            return Err(CliError::InvalidArguments {
                reason: format!("Invalid macro step: {}", part),
            });
        }
    }

    Ok(steps)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_macro_sequence_valid() {
        let result = parse_macro_sequence("press:VK_A,wait:50,release:VK_A");
        assert!(result.is_ok());
        let steps = result.unwrap();
        assert_eq!(steps.len(), 3);
    }

    #[test]
    fn test_parse_macro_sequence_invalid() {
        let result = parse_macro_sequence("invalid:VK_A");
        assert!(result.is_err());
    }
}
