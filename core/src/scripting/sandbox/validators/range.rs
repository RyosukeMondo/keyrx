//! Range validators for numeric types.
//!
//! These validators check that numeric values fall within acceptable ranges.

use crate::scripting::sandbox::validation::{InputValidator, ValidationError, ValidationResult};
use std::cmp::PartialOrd;
use std::fmt::Display;

/// Validates that a value is within an inclusive range [min, max].
///
/// # Examples
///
/// ```
/// use keyrx_core::scripting::sandbox::validation::InputValidator;
/// use keyrx_core::scripting::sandbox::validators::RangeValidator;
///
/// let validator = RangeValidator::new(0, 100);
/// assert!(validator.validate(&50).is_ok());
/// assert!(validator.validate(&0).is_ok());
/// assert!(validator.validate(&100).is_ok());
/// assert!(validator.validate(&-1).is_err());
/// assert!(validator.validate(&101).is_err());
/// ```
#[derive(Debug, Clone, Copy)]
pub struct RangeValidator<T> {
    min: T,
    max: T,
}

impl<T> RangeValidator<T> {
    /// Create a new range validator with inclusive bounds.
    pub const fn new(min: T, max: T) -> Self {
        Self { min, max }
    }
}

impl<T> InputValidator<T> for RangeValidator<T>
where
    T: PartialOrd + Display + Copy,
{
    fn validate(&self, value: &T) -> ValidationResult {
        if *value >= self.min && *value <= self.max {
            Ok(())
        } else {
            Err(ValidationError::out_of_range(*value, self.min, self.max))
        }
    }
}

/// Validates that a value is non-negative (>= 0).
///
/// This is a specialized range validator for common non-negative checks.
///
/// # Examples
///
/// ```
/// use keyrx_core::scripting::sandbox::validation::InputValidator;
/// use keyrx_core::scripting::sandbox::validators::NonNegativeValidator;
///
/// let validator = NonNegativeValidator;
/// assert!(validator.validate(&0).is_ok());
/// assert!(validator.validate(&42).is_ok());
/// assert!(validator.validate(&-1).is_err());
/// ```
#[derive(Debug, Clone, Copy)]
pub struct NonNegativeValidator;

impl InputValidator<i64> for NonNegativeValidator {
    fn validate(&self, value: &i64) -> ValidationResult {
        if *value >= 0 {
            Ok(())
        } else {
            Err(ValidationError::invalid_value(
                "non-negative integer",
                &format!("value must be >= 0, got {}", value),
            ))
        }
    }
}

impl InputValidator<i32> for NonNegativeValidator {
    fn validate(&self, value: &i32) -> ValidationResult {
        if *value >= 0 {
            Ok(())
        } else {
            Err(ValidationError::invalid_value(
                "non-negative integer",
                &format!("value must be >= 0, got {}", value),
            ))
        }
    }
}

impl InputValidator<f64> for NonNegativeValidator {
    fn validate(&self, value: &f64) -> ValidationResult {
        if *value >= 0.0 {
            Ok(())
        } else {
            Err(ValidationError::invalid_value(
                "non-negative number",
                &format!("value must be >= 0, got {}", value),
            ))
        }
    }
}

/// Validates that a value is positive (> 0).
///
/// This is a specialized range validator for common positive checks.
///
/// # Examples
///
/// ```
/// use keyrx_core::scripting::sandbox::validation::InputValidator;
/// use keyrx_core::scripting::sandbox::validators::PositiveValidator;
///
/// let validator = PositiveValidator;
/// assert!(validator.validate(&1).is_ok());
/// assert!(validator.validate(&42).is_ok());
/// assert!(validator.validate(&0).is_err());
/// assert!(validator.validate(&-1).is_err());
/// ```
#[derive(Debug, Clone, Copy)]
pub struct PositiveValidator;

impl InputValidator<i64> for PositiveValidator {
    fn validate(&self, value: &i64) -> ValidationResult {
        if *value > 0 {
            Ok(())
        } else {
            Err(ValidationError::invalid_value(
                "positive integer",
                &format!("value must be > 0, got {}", value),
            ))
        }
    }
}

impl InputValidator<i32> for PositiveValidator {
    fn validate(&self, value: &i32) -> ValidationResult {
        if *value > 0 {
            Ok(())
        } else {
            Err(ValidationError::invalid_value(
                "positive integer",
                &format!("value must be > 0, got {}", value),
            ))
        }
    }
}

impl InputValidator<f64> for PositiveValidator {
    fn validate(&self, value: &f64) -> ValidationResult {
        if *value > 0.0 {
            Ok(())
        } else {
            Err(ValidationError::invalid_value(
                "positive number",
                &format!("value must be > 0, got {}", value),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_validator_i64() {
        let validator = RangeValidator::new(0i64, 100i64);
        assert!(validator.validate(&0).is_ok());
        assert!(validator.validate(&50).is_ok());
        assert!(validator.validate(&100).is_ok());
        assert!(validator.validate(&-1).is_err());
        assert!(validator.validate(&101).is_err());
    }

    #[test]
    fn range_validator_f64() {
        let validator = RangeValidator::new(0.0, 1.0);
        assert!(validator.validate(&0.0).is_ok());
        assert!(validator.validate(&0.5).is_ok());
        assert!(validator.validate(&1.0).is_ok());
        assert!(validator.validate(&-0.1).is_err());
        assert!(validator.validate(&1.1).is_err());
    }

    #[test]
    fn non_negative_validator_i64() {
        let validator = NonNegativeValidator;
        assert!(validator.validate(&0i64).is_ok());
        assert!(validator.validate(&42i64).is_ok());
        assert!(validator.validate(&-1i64).is_err());
    }

    #[test]
    fn non_negative_validator_i32() {
        let validator = NonNegativeValidator;
        assert!(validator.validate(&0i32).is_ok());
        assert!(validator.validate(&42i32).is_ok());
        assert!(validator.validate(&-1i32).is_err());
    }

    #[test]
    fn non_negative_validator_f64() {
        let validator = NonNegativeValidator;
        assert!(validator.validate(&0.0).is_ok());
        assert!(validator.validate(&42.5).is_ok());
        assert!(validator.validate(&-0.1).is_err());
    }

    #[test]
    fn positive_validator_i64() {
        let validator = PositiveValidator;
        assert!(validator.validate(&1i64).is_ok());
        assert!(validator.validate(&42i64).is_ok());
        assert!(validator.validate(&0i64).is_err());
        assert!(validator.validate(&-1i64).is_err());
    }

    #[test]
    fn positive_validator_i32() {
        let validator = PositiveValidator;
        assert!(validator.validate(&1i32).is_ok());
        assert!(validator.validate(&42i32).is_ok());
        assert!(validator.validate(&0i32).is_err());
        assert!(validator.validate(&-1i32).is_err());
    }

    #[test]
    fn positive_validator_f64() {
        let validator = PositiveValidator;
        assert!(validator.validate(&0.1).is_ok());
        assert!(validator.validate(&42.5).is_ok());
        assert!(validator.validate(&0.0).is_err());
        assert!(validator.validate(&-0.1).is_err());
    }

    #[test]
    fn range_error_message() {
        let validator = RangeValidator::new(0, 100);
        let err = validator.validate(&150).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("150"));
        assert!(msg.contains("0"));
        assert!(msg.contains("100"));
    }

    #[test]
    fn positive_error_message() {
        let validator = PositiveValidator;
        let err = validator.validate(&-5i64).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("positive"));
        assert!(msg.contains("-5"));
    }
}
