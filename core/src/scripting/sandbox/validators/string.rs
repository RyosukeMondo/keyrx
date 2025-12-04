//! String validators.
//!
//! These validators check string properties like length and patterns.

use crate::scripting::sandbox::validation::{InputValidator, ValidationError, ValidationResult};
use regex::Regex;

/// Validates that a string is non-empty.
///
/// # Examples
///
/// ```
/// use keyrx_core::scripting::sandbox::validation::InputValidator;
/// use keyrx_core::scripting::sandbox::validators::NonEmptyValidator;
///
/// let validator = NonEmptyValidator;
/// assert!(validator.validate("hello").is_ok());
/// assert!(validator.validate("a").is_ok());
/// assert!(validator.validate("").is_err());
/// ```
#[derive(Debug, Clone, Copy)]
pub struct NonEmptyValidator;

impl InputValidator<str> for NonEmptyValidator {
    fn validate(&self, value: &str) -> ValidationResult {
        if value.is_empty() {
            Err(ValidationError::invalid_value(
                "string",
                "string must not be empty",
            ))
        } else {
            Ok(())
        }
    }
}

impl InputValidator<String> for NonEmptyValidator {
    fn validate(&self, value: &String) -> ValidationResult {
        self.validate(value.as_str())
    }
}

/// Validates that a string length is within bounds.
///
/// # Examples
///
/// ```
/// use keyrx_core::scripting::sandbox::validation::InputValidator;
/// use keyrx_core::scripting::sandbox::validators::LengthValidator;
///
/// let validator = LengthValidator::new(1, 10);
/// assert!(validator.validate("hello").is_ok());
/// assert!(validator.validate("a").is_ok());
/// assert!(validator.validate("").is_err());
/// assert!(validator.validate("this is too long").is_err());
/// ```
#[derive(Debug, Clone, Copy)]
pub struct LengthValidator {
    min: usize,
    max: usize,
}

impl LengthValidator {
    /// Create a new length validator with inclusive bounds.
    pub const fn new(min: usize, max: usize) -> Self {
        Self { min, max }
    }

    /// Create a validator that checks minimum length only.
    pub const fn min(min: usize) -> Self {
        Self {
            min,
            max: usize::MAX,
        }
    }

    /// Create a validator that checks maximum length only.
    pub const fn max(max: usize) -> Self {
        Self { min: 0, max }
    }
}

impl InputValidator<str> for LengthValidator {
    fn validate(&self, value: &str) -> ValidationResult {
        let len = value.len();
        if len < self.min || len > self.max {
            Err(ValidationError::invalid_length(
                len,
                &format!("length must be between {} and {}", self.min, self.max),
            ))
        } else {
            Ok(())
        }
    }
}

impl InputValidator<String> for LengthValidator {
    fn validate(&self, value: &String) -> ValidationResult {
        self.validate(value.as_str())
    }
}

/// Validates that a string matches a regex pattern.
///
/// # Examples
///
/// ```
/// use keyrx_core::scripting::sandbox::validation::InputValidator;
/// use keyrx_core::scripting::sandbox::validators::PatternValidator;
///
/// let validator = PatternValidator::new(r"^[a-z]+$").unwrap();
/// assert!(validator.validate("hello").is_ok());
/// assert!(validator.validate("world").is_ok());
/// assert!(validator.validate("Hello").is_err());
/// assert!(validator.validate("hello123").is_err());
/// ```
#[derive(Debug, Clone)]
pub struct PatternValidator {
    pattern: Regex,
    pattern_str: String,
}

impl PatternValidator {
    /// Create a new pattern validator from a regex string.
    ///
    /// Returns an error if the regex is invalid.
    pub fn new(pattern: &str) -> Result<Self, regex::Error> {
        Ok(Self {
            pattern: Regex::new(pattern)?,
            pattern_str: pattern.to_string(),
        })
    }
}

impl InputValidator<str> for PatternValidator {
    fn validate(&self, value: &str) -> ValidationResult {
        if self.pattern.is_match(value) {
            Ok(())
        } else {
            Err(ValidationError::pattern_mismatch(&self.pattern_str))
        }
    }
}

impl InputValidator<String> for PatternValidator {
    fn validate(&self, value: &String) -> ValidationResult {
        self.validate(value.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_empty_validator() {
        let validator = NonEmptyValidator;
        assert!(validator.validate("hello").is_ok());
        assert!(validator.validate("a").is_ok());
        assert!(validator.validate(" ").is_ok());
        assert!(validator.validate("").is_err());
    }

    #[test]
    fn non_empty_string_type() {
        let validator = NonEmptyValidator;
        let s = String::from("hello");
        assert!(validator.validate(&s).is_ok());

        let s = String::from("");
        assert!(validator.validate(&s).is_err());
    }

    #[test]
    fn length_validator() {
        let validator = LengthValidator::new(1, 10);
        assert!(validator.validate("a").is_ok());
        assert!(validator.validate("hello").is_ok());
        assert!(validator.validate("1234567890").is_ok());
        assert!(validator.validate("").is_err());
        assert!(validator.validate("12345678901").is_err());
    }

    #[test]
    fn length_validator_min() {
        let validator = LengthValidator::min(3);
        assert!(validator.validate("abc").is_ok());
        assert!(validator.validate("abcdefghijklmnop").is_ok());
        assert!(validator.validate("ab").is_err());
        assert!(validator.validate("").is_err());
    }

    #[test]
    fn length_validator_max() {
        let validator = LengthValidator::max(5);
        assert!(validator.validate("").is_ok());
        assert!(validator.validate("hello").is_ok());
        assert!(validator.validate("hi").is_ok());
        assert!(validator.validate("toolong").is_err());
    }

    #[test]
    fn pattern_validator() {
        let validator = PatternValidator::new(r"^[a-z]+$").unwrap();
        assert!(validator.validate("hello").is_ok());
        assert!(validator.validate("world").is_ok());
        assert!(validator.validate("abc").is_ok());
        assert!(validator.validate("Hello").is_err());
        assert!(validator.validate("hello123").is_err());
        assert!(validator.validate("").is_err());
    }

    #[test]
    fn pattern_validator_complex() {
        let validator = PatternValidator::new(r"^\d{3}-\d{4}$").unwrap();
        assert!(validator.validate("123-4567").is_ok());
        assert!(validator.validate("000-0000").is_ok());
        assert!(validator.validate("123-456").is_err());
        assert!(validator.validate("abc-defg").is_err());
    }

    #[test]
    fn pattern_validator_error_message() {
        let validator = PatternValidator::new(r"^[a-z]+$").unwrap();
        let err = validator.validate("Hello123").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("pattern"));
    }

    #[test]
    fn length_error_message() {
        let validator = LengthValidator::new(1, 10);
        let err = validator.validate("").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("0"));
        assert!(msg.contains("length"));
    }
}
