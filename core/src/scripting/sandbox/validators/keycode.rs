//! Validator for KeyCode strings.
//!
//! Validates that a string can be parsed into a valid KeyCode.

use crate::drivers::keycodes::KeyCode;
use crate::scripting::sandbox::validation::{InputValidator, ValidationError, ValidationResult};
use std::str::FromStr;

/// Validates that a string represents a valid KeyCode.
///
/// This validator checks that the input string can be successfully parsed
/// into a [`KeyCode`] using the `FromStr` implementation.
///
/// # Examples
///
/// ```
/// use keyrx_core::scripting::sandbox::validation::InputValidator;
/// use keyrx_core::scripting::sandbox::validators::KeyCodeValidator;
///
/// let validator = KeyCodeValidator;
/// assert!(validator.validate("A").is_ok());
/// assert!(validator.validate("shift").is_ok());
/// assert!(validator.validate("CapsLock").is_ok());
/// assert!(validator.validate("InvalidKey123").is_err());
/// ```
#[derive(Debug, Clone, Copy)]
pub struct KeyCodeValidator;

impl InputValidator<str> for KeyCodeValidator {
    fn validate(&self, value: &str) -> ValidationResult {
        KeyCode::from_str(value).map_err(|_| {
            ValidationError::invalid_value(
                "key_code",
                &format!("'{}' is not a valid KeyCode", value),
            )
        })?;
        Ok(())
    }
}

impl InputValidator<String> for KeyCodeValidator {
    fn validate(&self, value: &String) -> ValidationResult {
        self.validate(value.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_keycodes() {
        let validator = KeyCodeValidator;

        // Basic keys
        assert!(validator.validate("A").is_ok());
        assert!(validator.validate("a").is_ok());
        assert!(validator.validate("Z").is_ok());

        // Modifiers
        assert!(validator.validate("shift").is_ok());
        assert!(validator.validate("ctrl").is_ok());
        assert!(validator.validate("alt").is_ok());

        // Special keys
        assert!(validator.validate("CapsLock").is_ok());
        assert!(validator.validate("caps").is_ok());
        assert!(validator.validate("Escape").is_ok());
        assert!(validator.validate("esc").is_ok());

        // Function keys
        assert!(validator.validate("F1").is_ok());
        assert!(validator.validate("F12").is_ok());

        // Number keys
        assert!(validator.validate("1").is_ok());
        assert!(validator.validate("0").is_ok());
    }

    #[test]
    fn invalid_keycodes() {
        let validator = KeyCodeValidator;

        assert!(validator.validate("InvalidKey123").is_err());
        assert!(validator.validate("NotAKey").is_err());
        assert!(validator.validate("").is_err());
        assert!(validator.validate("F99").is_err());
    }

    #[test]
    fn error_message() {
        let validator = KeyCodeValidator;
        let err = validator.validate("InvalidKey").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("InvalidKey"));
        assert!(msg.contains("key_code"));
    }

    #[test]
    fn string_type() {
        let validator = KeyCodeValidator;
        let s = String::from("A");
        assert!(validator.validate(&s).is_ok());

        let s = String::from("InvalidKey");
        assert!(validator.validate(&s).is_err());
    }
}
