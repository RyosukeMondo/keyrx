//! Helper functions for Rhai scripting integration.

use crate::engine::KeyCode;
use crate::scripting::sandbox::validation::InputValidator;
use crate::scripting::sandbox::validators::{KeyCodeValidator, NonEmptyValidator};
use rhai::{EvalAltResult, Position};

/// Parse a key name string into a KeyCode, returning a Rhai-compatible error on failure.
///
/// This function provides a consistent error message format for all Rhai script
/// functions that need to parse key names. On failure, it logs a warning and
/// returns an appropriate `EvalAltResult` error.
///
/// Uses the InputValidator trait system for validation.
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
    // Validate non-empty first
    NonEmptyValidator.validate(key).map_err(|e| {
        Box::new(EvalAltResult::ErrorRuntime(
            format!("{}: {}", fn_name, e).into(),
            Position::NONE,
        ))
    })?;

    // Validate key code
    KeyCodeValidator.validate(key).map_err(|_| {
        tracing::warn!(
            service = "keyrx",
            event = "rhai_unknown_key",
            component = "scripting_helpers",
            function = fn_name,
            key = key,
            "Unknown key in Rhai function"
        );
        Box::new(EvalAltResult::ErrorRuntime(
            format!(
                "{}: Unknown key '{}'. See .spec-workflow/steering/tech.md (Key Naming & Aliases).",
                fn_name, key
            )
            .into(),
            Position::NONE,
        ))
    })?;

    // Parse the key (we know it's valid now)
    KeyCode::from_name(key).ok_or_else(|| {
        Box::new(EvalAltResult::ErrorRuntime(
            format!("{}: Failed to parse validated key '{}'", fn_name, key).into(),
            Position::NONE,
        ))
    })
}
