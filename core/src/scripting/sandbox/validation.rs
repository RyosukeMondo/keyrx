//! Input validation for script function parameters.
//!
//! This module provides a trait-based validation system for script function inputs.
//! Validators can be composed and reused across different functions.

use thiserror::Error;

/// Result type for validation operations.
pub type ValidationResult<T = ()> = Result<T, ValidationError>;

/// Validation error with detailed context.
///
/// Contains information about what validation failed and why, enabling
/// clear error messages for script users.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum ValidationError {
    /// Value is out of valid range.
    #[error("Value {value} is out of range [{min}, {max}]")]
    OutOfRange {
        value: String,
        min: String,
        max: String,
    },

    /// Value is of wrong type.
    #[error("Expected type {expected}, got {actual}")]
    WrongType { expected: String, actual: String },

    /// Value is invalid for the context.
    #[error("Invalid value for {context}: {reason}")]
    InvalidValue { context: String, reason: String },

    /// Required parameter is missing.
    #[error("Missing required parameter: {parameter}")]
    MissingParameter { parameter: String },

    /// Collection has invalid length.
    #[error("Collection length {actual} violates constraint: {constraint}")]
    InvalidLength { actual: usize, constraint: String },

    /// Pattern matching failed.
    #[error("Value does not match pattern: {pattern}")]
    PatternMismatch { pattern: String },

    /// Custom validation error with context.
    #[error("{message}")]
    Custom { message: String },
}

impl ValidationError {
    /// Create an out-of-range error.
    pub fn out_of_range<T: ToString>(value: T, min: T, max: T) -> Self {
        Self::OutOfRange {
            value: value.to_string(),
            min: min.to_string(),
            max: max.to_string(),
        }
    }

    /// Create a wrong-type error.
    pub fn wrong_type(expected: &str, actual: &str) -> Self {
        Self::WrongType {
            expected: expected.to_string(),
            actual: actual.to_string(),
        }
    }

    /// Create an invalid-value error with context.
    pub fn invalid_value(context: &str, reason: &str) -> Self {
        Self::InvalidValue {
            context: context.to_string(),
            reason: reason.to_string(),
        }
    }

    /// Create a missing-parameter error.
    pub fn missing_parameter(parameter: &str) -> Self {
        Self::MissingParameter {
            parameter: parameter.to_string(),
        }
    }

    /// Create an invalid-length error.
    pub fn invalid_length(actual: usize, constraint: &str) -> Self {
        Self::InvalidLength {
            actual,
            constraint: constraint.to_string(),
        }
    }

    /// Create a pattern-mismatch error.
    pub fn pattern_mismatch(pattern: &str) -> Self {
        Self::PatternMismatch {
            pattern: pattern.to_string(),
        }
    }

    /// Create a custom error with a message.
    pub fn custom(message: impl Into<String>) -> Self {
        Self::Custom {
            message: message.into(),
        }
    }
}

/// Trait for input validation.
///
/// Implementors define validation logic for specific types or constraints.
/// Validators should be composable and reusable.
///
/// # Examples
///
/// ```
/// use keyrx_core::scripting::sandbox::validation::{InputValidator, ValidationError};
///
/// struct PositiveValidator;
///
/// impl InputValidator<i32> for PositiveValidator {
///     fn validate(&self, value: &i32) -> Result<(), ValidationError> {
///         if *value > 0 {
///             Ok(())
///         } else {
///             Err(ValidationError::invalid_value(
///                 "positive integer",
///                 "value must be greater than 0"
///             ))
///         }
///     }
/// }
///
/// let validator = PositiveValidator;
/// assert!(validator.validate(&5).is_ok());
/// assert!(validator.validate(&-5).is_err());
/// ```
pub trait InputValidator<T: ?Sized> {
    /// Validate the input value.
    ///
    /// Returns `Ok(())` if validation passes, or a `ValidationError` describing
    /// why validation failed.
    fn validate(&self, value: &T) -> ValidationResult;

    /// Validate and return the value on success.
    ///
    /// This is a convenience method that validates and returns the value,
    /// useful for chaining validators.
    fn validate_and_return(&self, value: T) -> ValidationResult<T>
    where
        T: Sized,
    {
        self.validate(&value)?;
        Ok(value)
    }

    /// Combine this validator with another using AND logic.
    ///
    /// Both validators must pass for the combined validator to pass.
    fn and<V>(self, other: V) -> AndValidator<Self, V>
    where
        Self: Sized,
        V: InputValidator<T>,
    {
        AndValidator {
            first: self,
            second: other,
        }
    }

    /// Combine this validator with another using OR logic.
    ///
    /// At least one validator must pass for the combined validator to pass.
    fn or<V>(self, other: V) -> OrValidator<Self, V>
    where
        Self: Sized,
        V: InputValidator<T>,
    {
        OrValidator {
            first: self,
            second: other,
        }
    }
}

/// Validator that combines two validators with AND logic.
pub struct AndValidator<A, B> {
    first: A,
    second: B,
}

impl<T, A, B> InputValidator<T> for AndValidator<A, B>
where
    A: InputValidator<T>,
    B: InputValidator<T>,
{
    fn validate(&self, value: &T) -> ValidationResult {
        self.first.validate(value)?;
        self.second.validate(value)
    }
}

/// Validator that combines two validators with OR logic.
pub struct OrValidator<A, B> {
    first: A,
    second: B,
}

impl<T, A, B> InputValidator<T> for OrValidator<A, B>
where
    A: InputValidator<T>,
    B: InputValidator<T>,
{
    fn validate(&self, value: &T) -> ValidationResult {
        match self.first.validate(value) {
            Ok(()) => Ok(()),
            Err(_) => self.second.validate(value),
        }
    }
}

/// Always-valid validator for testing or optional validation.
pub struct AlwaysValid;

impl<T> InputValidator<T> for AlwaysValid {
    fn validate(&self, _value: &T) -> ValidationResult {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestValidator;

    impl InputValidator<i32> for TestValidator {
        fn validate(&self, value: &i32) -> ValidationResult {
            if *value > 0 {
                Ok(())
            } else {
                Err(ValidationError::invalid_value(
                    "positive integer",
                    "value must be greater than 0",
                ))
            }
        }
    }

    struct RangeValidator {
        min: i32,
        max: i32,
    }

    impl InputValidator<i32> for RangeValidator {
        fn validate(&self, value: &i32) -> ValidationResult {
            if *value >= self.min && *value <= self.max {
                Ok(())
            } else {
                Err(ValidationError::out_of_range(*value, self.min, self.max))
            }
        }
    }

    #[test]
    fn test_validation_pass() {
        let validator = TestValidator;
        assert!(validator.validate(&5).is_ok());
    }

    #[test]
    fn test_validation_fail() {
        let validator = TestValidator;
        let result = validator.validate(&-5);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("positive integer"));
    }

    #[test]
    fn test_validate_and_return() {
        let validator = TestValidator;
        assert_eq!(validator.validate_and_return(5).unwrap(), 5);
        assert!(validator.validate_and_return(-5).is_err());
    }

    #[test]
    fn test_and_validator() {
        let positive = TestValidator;
        let range = RangeValidator { min: 0, max: 10 };
        let combined = positive.and(range);

        assert!(combined.validate(&5).is_ok());
        assert!(combined.validate(&-5).is_err());
        assert!(combined.validate(&15).is_err());
    }

    #[test]
    fn test_or_validator() {
        let range1 = RangeValidator { min: 0, max: 10 };
        let range2 = RangeValidator { min: 20, max: 30 };
        let combined = range1.or(range2);

        assert!(combined.validate(&5).is_ok());
        assert!(combined.validate(&25).is_ok());
        assert!(combined.validate(&15).is_err());
    }

    #[test]
    fn test_always_valid() {
        let validator = AlwaysValid;
        assert!(validator.validate(&42).is_ok());
        assert!(validator.validate(&-42).is_ok());
    }

    #[test]
    fn test_error_out_of_range() {
        let err = ValidationError::out_of_range(5, 0, 10);
        assert!(err.to_string().contains("5"));
        assert!(err.to_string().contains("0"));
        assert!(err.to_string().contains("10"));
    }

    #[test]
    fn test_error_wrong_type() {
        let err = ValidationError::wrong_type("String", "Integer");
        assert!(err.to_string().contains("String"));
        assert!(err.to_string().contains("Integer"));
    }

    #[test]
    fn test_error_invalid_value() {
        let err = ValidationError::invalid_value("key_code", "invalid code");
        assert!(err.to_string().contains("key_code"));
        assert!(err.to_string().contains("invalid code"));
    }

    #[test]
    fn test_error_missing_parameter() {
        let err = ValidationError::missing_parameter("required_param");
        assert!(err.to_string().contains("required_param"));
    }

    #[test]
    fn test_error_invalid_length() {
        let err = ValidationError::invalid_length(5, "max 3 items");
        assert!(err.to_string().contains("5"));
        assert!(err.to_string().contains("max 3 items"));
    }

    #[test]
    fn test_error_pattern_mismatch() {
        let err = ValidationError::pattern_mismatch("[a-z]+");
        assert!(err.to_string().contains("[a-z]+"));
    }

    #[test]
    fn test_error_custom() {
        let err = ValidationError::custom("custom error message");
        assert_eq!(err.to_string(), "custom error message");
    }

    #[test]
    fn test_error_equality() {
        let err1 = ValidationError::custom("test");
        let err2 = ValidationError::custom("test");
        assert_eq!(err1, err2);

        let err3 = ValidationError::custom("different");
        assert_ne!(err1, err3);
    }
}
