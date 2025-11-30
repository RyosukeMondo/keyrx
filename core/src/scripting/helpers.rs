//! Helper functions for Rhai scripting integration.

use crate::engine::KeyCode;
use rhai::{EvalAltResult, Position};

/// Parse a key name string into a KeyCode, returning a Rhai-compatible error on failure.
///
/// This function provides a consistent error message format for all Rhai script
/// functions that need to parse key names. On failure, it logs a warning and
/// returns an appropriate `EvalAltResult` error.
///
/// # Arguments
///
/// * `key` - The key name to parse (e.g., "a", "LeftCtrl", "Space")
/// * `fn_name` - The name of the calling function for error messages (e.g., "remap", "block")
///
/// # Returns
///
/// * `Ok(KeyCode)` - The parsed key code
/// * `Err(Box<EvalAltResult>)` - A Rhai runtime error with a helpful message
///
/// # Example
///
/// ```ignore
/// let key_code = parse_key_or_error("LeftCtrl", "block")?;
/// ```
pub fn parse_key_or_error(key: &str, fn_name: &str) -> Result<KeyCode, Box<EvalAltResult>> {
    match KeyCode::from_name(key) {
        Some(k) => Ok(k),
        None => {
            tracing::warn!(
                service = "keyrx",
                event = "rhai_unknown_key",
                component = "scripting_helpers",
                function = fn_name,
                key = key,
                "Unknown key in Rhai function"
            );
            Err(Box::new(EvalAltResult::ErrorRuntime(
                format!(
                    "Unknown key '{}'. See .spec-workflow/steering/tech.md (Key Naming & Aliases).",
                    key
                )
                .into(),
                Position::NONE,
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_key_or_error_valid_key() {
        let result = parse_key_or_error("a", "test");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), KeyCode::A);
    }

    #[test]
    fn parse_key_or_error_valid_key_uppercase() {
        let result = parse_key_or_error("A", "test");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), KeyCode::A);
    }

    #[test]
    fn parse_key_or_error_modifier() {
        let result = parse_key_or_error("LeftCtrl", "block");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), KeyCode::LeftCtrl);
    }

    #[test]
    fn parse_key_or_error_space() {
        let result = parse_key_or_error("Space", "pass");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), KeyCode::Space);
    }

    #[test]
    fn parse_key_or_error_invalid_key() {
        let result = parse_key_or_error("InvalidKey123", "remap");
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_str = err.to_string();
        assert!(err_str.contains("InvalidKey123"));
        assert!(err_str.contains(".spec-workflow/steering/tech.md (Key Naming & Aliases)."));
    }

    #[test]
    fn parse_key_or_error_empty_string() {
        let result = parse_key_or_error("", "block");
        assert!(result.is_err());
    }
}
