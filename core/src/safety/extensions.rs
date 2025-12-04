//! Extension traits for safe unwrapping and error conversion.
//!
//! This module provides extension traits that make it easier to migrate from
//! panic-inducing methods like `unwrap()` and `expect()` to safe alternatives
//! with automatic logging and fallback handling.
//!
//! # Design Philosophy
//!
//! - Provide drop-in replacements for common panic patterns
//! - Automatically log when fallbacks are used
//! - Preserve context through structured logging
//! - Make migration from unsafe code easier
//!
//! # Example
//!
//! ```ignore
//! use keyrx_core::safety::extensions::{OptionExt, ResultExt};
//!
//! // Before: can panic
//! let value = some_option.unwrap();
//!
//! // After: safe with logging
//! let value = some_option.unwrap_or_log("operation context", 0);
//!
//! // Before: can panic
//! let result = some_result.unwrap();
//!
//! // After: converts to CriticalResult with context
//! let critical_result = some_result.map_critical("operation failed");
//! ```

use crate::errors::critical::CriticalError;
use crate::errors::critical_result::CriticalResult;

/// Extension trait for `Option<T>` providing safe unwrapping with logging.
///
/// This trait provides alternatives to `unwrap()` and `expect()` that log
/// when fallback values are used, making it easier to diagnose issues in
/// production without causing panics.
pub trait OptionExt<T> {
    /// Returns the contained value or a fallback, logging when the fallback is used.
    ///
    /// This is a safe alternative to `unwrap()` that logs at ERROR level when
    /// the Option is None and returns the provided fallback value.
    ///
    /// # Arguments
    ///
    /// * `context` - A description of the operation context for logging
    /// * `fallback` - The value to return if the Option is None
    ///
    /// # Example
    ///
    /// ```ignore
    /// let value = some_option.unwrap_or_log("parsing config", 42);
    /// // If None: logs "parsing config: Option was None, using fallback"
    /// // Returns: 42
    /// ```
    fn unwrap_or_log(self, context: &str, fallback: T) -> T;

    /// Returns the contained value or computes a fallback, logging when computed.
    ///
    /// Similar to `unwrap_or_log` but computes the fallback value lazily.
    ///
    /// # Arguments
    ///
    /// * `context` - A description of the operation context for logging
    /// * `fallback_fn` - A function to compute the fallback value
    ///
    /// # Example
    ///
    /// ```ignore
    /// let value = some_option.unwrap_or_log_else("loading device", || {
    ///     get_default_device()
    /// });
    /// ```
    fn unwrap_or_log_else<F>(self, context: &str, fallback_fn: F) -> T
    where
        F: FnOnce() -> T;

    /// Maps the value or logs and returns a fallback.
    ///
    /// This combines mapping and safe unwrapping in one operation.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let doubled = some_option.map_or_log("doubling value", 0, |x| x * 2);
    /// ```
    fn map_or_log<U, F>(self, context: &str, default: U, f: F) -> U
    where
        F: FnOnce(T) -> U;
}

impl<T> OptionExt<T> for Option<T> {
    fn unwrap_or_log(self, context: &str, fallback: T) -> T {
        match self {
            Some(value) => value,
            None => {
                tracing::error!("{}: Option was None, using fallback", context);
                fallback
            }
        }
    }

    fn unwrap_or_log_else<F>(self, context: &str, fallback_fn: F) -> T
    where
        F: FnOnce() -> T,
    {
        match self {
            Some(value) => value,
            None => {
                tracing::error!("{}: Option was None, computing fallback", context);
                fallback_fn()
            }
        }
    }

    fn map_or_log<U, F>(self, context: &str, default: U, f: F) -> U
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Some(value) => f(value),
            None => {
                tracing::error!("{}: Option was None during mapping, using default", context);
                default
            }
        }
    }
}

/// Extension trait for `Result<T, E>` providing conversion to `CriticalResult`.
///
/// This trait makes it easier to convert standard Results into CriticalResults
/// with automatic error conversion and context preservation.
pub trait ResultExt<T, E> {
    /// Converts a Result to a CriticalResult with context.
    ///
    /// This maps arbitrary errors to `CriticalError::ProcessingFailed` with
    /// the provided context message. It's the primary migration path from
    /// standard Results to CriticalResults.
    ///
    /// # Arguments
    ///
    /// * `context` - A description of what operation failed
    ///
    /// # Example
    ///
    /// ```ignore
    /// fn parse_config(data: &str) -> Result<Config, ParseError> {
    ///     // ... parsing logic
    /// }
    ///
    /// let critical_result = parse_config(data)
    ///     .map_critical("parsing configuration");
    /// // If error: creates ProcessingFailed with "parsing configuration: <error>"
    /// ```
    fn map_critical(self, context: &str) -> CriticalResult<T>
    where
        E: std::fmt::Display;

    /// Converts a Result to a CriticalResult with a custom error mapper.
    ///
    /// This gives more control over how errors are converted to CriticalError,
    /// allowing specific error types to map to appropriate CriticalError variants.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let result = some_operation()
    ///     .map_critical_with(|err| match err {
    ///         MyError::NotFound => CriticalError::DiscoveryFailed {
    ///             reason: "resource not found".to_string(),
    ///             cause: Some(err.to_string()),
    ///         },
    ///         MyError::Permission => CriticalError::DriverInitFailed {
    ///             reason: "permission denied".to_string(),
    ///             cause: Some(err.to_string()),
    ///         },
    ///     });
    /// ```
    fn map_critical_with<F>(self, mapper: F) -> CriticalResult<T>
    where
        F: FnOnce(E) -> CriticalError;

    /// Unwraps or logs the error and returns a fallback.
    ///
    /// This is similar to `OptionExt::unwrap_or_log` but for Results,
    /// logging the actual error message.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let value = some_result.unwrap_or_log("loading config", Config::default());
    /// // If Err: logs "loading config: <error message>, using fallback"
    /// ```
    fn unwrap_or_log(self, context: &str, fallback: T) -> T
    where
        E: std::fmt::Display;

    /// Unwraps or logs the error and computes a fallback.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = parse_config().unwrap_or_log_else("parsing config", |err| {
    ///     eprintln!("Parse failed: {}", err);
    ///     Config::default()
    /// });
    /// ```
    fn unwrap_or_log_else<F>(self, context: &str, fallback_fn: F) -> T
    where
        E: std::fmt::Display,
        F: FnOnce(E) -> T;
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    fn map_critical(self, context: &str) -> CriticalResult<T>
    where
        E: std::fmt::Display,
    {
        match self {
            Ok(value) => CriticalResult::ok(value),
            Err(err) => CriticalResult::err(CriticalError::ProcessingFailed {
                reason: context.to_string(),
                cause: Some(err.to_string()),
            }),
        }
    }

    fn map_critical_with<F>(self, mapper: F) -> CriticalResult<T>
    where
        F: FnOnce(E) -> CriticalError,
    {
        match self {
            Ok(value) => CriticalResult::ok(value),
            Err(err) => CriticalResult::err(mapper(err)),
        }
    }

    fn unwrap_or_log(self, context: &str, fallback: T) -> T
    where
        E: std::fmt::Display,
    {
        match self {
            Ok(value) => value,
            Err(err) => {
                tracing::error!("{}: {}, using fallback", context, err);
                fallback
            }
        }
    }

    fn unwrap_or_log_else<F>(self, context: &str, fallback_fn: F) -> T
    where
        E: std::fmt::Display,
        F: FnOnce(E) -> T,
    {
        match self {
            Ok(value) => value,
            Err(err) => {
                tracing::error!("{}: {}, computing fallback", context, err);
                fallback_fn(err)
            }
        }
    }
}

/// Extension trait for `Result<T, CriticalError>` to convert to `CriticalResult`.
///
/// This provides a convenient way to convert standard Results with CriticalError
/// into the CriticalResult wrapper.
pub trait CriticalResultExt<T> {
    /// Converts a Result<T, CriticalError> to a CriticalResult<T>.
    ///
    /// # Example
    ///
    /// ```ignore
    /// fn risky_op() -> Result<i32, CriticalError> {
    ///     // ... operation
    /// }
    ///
    /// let critical: CriticalResult<i32> = risky_op().into_critical();
    /// ```
    fn into_critical(self) -> CriticalResult<T>;
}

impl<T> CriticalResultExt<T> for Result<T, CriticalError> {
    fn into_critical(self) -> CriticalResult<T> {
        CriticalResult::from_result(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn option_unwrap_or_log_with_some() {
        let opt = Some(42);
        let value = opt.unwrap_or_log("test context", 0);
        assert_eq!(value, 42);
    }

    #[test]
    fn option_unwrap_or_log_with_none() {
        let opt: Option<i32> = None;
        let value = opt.unwrap_or_log("test context", 99);
        assert_eq!(value, 99);
    }

    #[test]
    fn option_unwrap_or_log_else_with_some() {
        let opt = Some(42);
        let value = opt.unwrap_or_log_else("test context", || 0);
        assert_eq!(value, 42);
    }

    #[test]
    fn option_unwrap_or_log_else_with_none() {
        let opt: Option<i32> = None;
        let mut called = false;
        let value = opt.unwrap_or_log_else("test context", || {
            called = true;
            99
        });
        assert_eq!(value, 99);
        assert!(called);
    }

    #[test]
    fn option_map_or_log_with_some() {
        let opt = Some(21);
        let value = opt.map_or_log("test context", 0, |x| x * 2);
        assert_eq!(value, 42);
    }

    #[test]
    fn option_map_or_log_with_none() {
        let opt: Option<i32> = None;
        let value = opt.map_or_log("test context", 99, |x| x * 2);
        assert_eq!(value, 99);
    }

    #[test]
    fn result_map_critical_with_ok() {
        let result: Result<i32, String> = Ok(42);
        let critical = result.map_critical("test operation");
        assert!(critical.is_ok());
        assert_eq!(critical.unwrap_or(0), 42);
    }

    #[test]
    fn result_map_critical_with_err() {
        let result: Result<i32, String> = Err("test error".to_string());
        let critical = result.map_critical("test operation");
        assert!(critical.is_err());
    }

    #[test]
    fn result_map_critical_with_custom_mapper() {
        let result: Result<i32, String> = Err("not found".to_string());
        let critical = result.map_critical_with(|err| CriticalError::DiscoveryFailed {
            reason: err,
            cause: None,
        });
        assert!(critical.is_err());
        matches!(
            critical.err_option(),
            Some(CriticalError::DiscoveryFailed { .. })
        );
    }

    #[test]
    fn result_unwrap_or_log_with_ok() {
        let result: Result<i32, String> = Ok(42);
        let value = result.unwrap_or_log("test context", 0);
        assert_eq!(value, 42);
    }

    #[test]
    fn result_unwrap_or_log_with_err() {
        let result: Result<i32, String> = Err("test error".to_string());
        let value = result.unwrap_or_log("test context", 99);
        assert_eq!(value, 99);
    }

    #[test]
    fn result_unwrap_or_log_else_with_ok() {
        let result: Result<i32, String> = Ok(42);
        let value = result.unwrap_or_log_else("test context", |_| 0);
        assert_eq!(value, 42);
    }

    #[test]
    fn result_unwrap_or_log_else_with_err() {
        let result: Result<i32, String> = Err("test error".to_string());
        let mut called = false;
        let value = result.unwrap_or_log_else("test context", |err| {
            called = true;
            assert_eq!(err, "test error");
            99
        });
        assert_eq!(value, 99);
        assert!(called);
    }

    #[test]
    fn critical_result_ext_conversion() {
        let result: Result<i32, CriticalError> = Ok(42);
        let critical = result.into_critical();
        assert!(critical.is_ok());
        assert_eq!(critical.unwrap_or(0), 42);
    }

    #[test]
    fn critical_result_ext_with_error() {
        let result: Result<i32, CriticalError> = Err(CriticalError::ProcessingFailed {
            reason: "test".to_string(),
            cause: None,
        });
        let critical = result.into_critical();
        assert!(critical.is_err());
    }

    #[test]
    fn chained_operations() {
        // Test chaining multiple operations
        let result: Result<i32, String> = Ok(10);
        let value = result
            .map_critical("parsing number")
            .map(|x| x * 2)
            .map(|x| x + 1)
            .unwrap_or(0);
        assert_eq!(value, 21);
    }

    #[test]
    fn chained_with_error() {
        let result: Result<i32, String> = Err("parse failed".to_string());
        let value = result
            .map_critical("parsing number")
            .map(|x| x * 2)
            .unwrap_or_log("final unwrap", |_| 99);
        assert_eq!(value, 99);
    }
}
