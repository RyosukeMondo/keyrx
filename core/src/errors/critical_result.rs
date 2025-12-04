//! Safe Result wrapper that prevents panics in critical paths.
//!
//! This module provides `CriticalResult<T>`, a wrapper around `Result<T, CriticalError>`
//! that does not expose panic-inducing methods like `unwrap()` or `expect()`.
//! All operations return safe alternatives with explicit fallback handling.
//!
//! # Design Philosophy
//!
//! - No `unwrap()`, `expect()`, or other panic-inducing methods
//! - All methods are marked with `#[must_use]` to prevent silent errors
//! - Fallback values are always provided or computed
//! - Errors are logged transparently when using fallback paths
//!
//! # Example
//!
//! ```ignore
//! use keyrx_core::errors::critical_result::CriticalResult;
//! use keyrx_core::errors::critical::CriticalError;
//!
//! fn risky_operation() -> CriticalResult<i32> {
//!     CriticalResult::ok(42)
//! }
//!
//! // Safe unwrapping with fallback
//! let value = risky_operation().unwrap_or_fallback(|err| {
//!     log::error!("Operation failed: {}, using fallback", err);
//!     0
//! });
//!
//! // Or use a default value
//! let value = risky_operation().unwrap_or_default();
//! ```

use super::critical::{CriticalError, FallbackAction};
use std::fmt;

/// A Result wrapper that prevents panics in critical paths.
///
/// This type wraps `Result<T, CriticalError>` and does not expose
/// any methods that can panic. All error handling is explicit and
/// provides safe fallback mechanisms.
#[must_use = "CriticalResult must be handled"]
#[derive(Debug, Clone)]
pub struct CriticalResult<T> {
    inner: Result<T, CriticalError>,
}

impl<T> CriticalResult<T> {
    /// Creates a successful CriticalResult.
    #[inline]
    pub fn ok(value: T) -> Self {
        Self { inner: Ok(value) }
    }

    /// Creates a failed CriticalResult.
    #[inline]
    pub fn err(error: CriticalError) -> Self {
        Self { inner: Err(error) }
    }

    /// Creates a CriticalResult from a standard Result.
    #[inline]
    pub fn from_result(result: Result<T, CriticalError>) -> Self {
        Self { inner: result }
    }

    /// Returns `true` if the result is `Ok`.
    #[inline]
    pub fn is_ok(&self) -> bool {
        self.inner.is_ok()
    }

    /// Returns `true` if the result is `Err`.
    #[inline]
    pub fn is_err(&self) -> bool {
        self.inner.is_err()
    }

    /// Converts to an `Option<T>`, discarding the error.
    #[inline]
    pub fn ok_option(self) -> Option<T> {
        self.inner.ok()
    }

    /// Converts to an `Option<CriticalError>`, discarding the success value.
    #[inline]
    pub fn err_option(self) -> Option<CriticalError> {
        self.inner.err()
    }

    /// Returns the contained value or a provided fallback.
    ///
    /// This is the primary safe unwrapping method. The fallback function
    /// receives the error and can perform logging or other side effects.
    #[inline]
    #[must_use = "the result is not used"]
    pub fn unwrap_or_fallback<F>(self, fallback: F) -> T
    where
        F: FnOnce(CriticalError) -> T,
    {
        match self.inner {
            Ok(value) => value,
            Err(err) => fallback(err),
        }
    }

    /// Returns the contained value or a provided default.
    #[inline]
    #[must_use = "the result is not used"]
    pub fn unwrap_or(self, default: T) -> T {
        self.inner.unwrap_or(default)
    }

    /// Returns the contained value or computes it from the error's fallback action.
    ///
    /// This method uses the error's `fallback_action()` to guide recovery,
    /// but requires the caller to interpret the action and produce a value.
    #[inline]
    #[must_use = "the result is not used"]
    pub fn unwrap_or_else<F>(self, f: F) -> T
    where
        F: FnOnce(CriticalError) -> T,
    {
        self.inner.unwrap_or_else(f)
    }

    /// Maps a `CriticalResult<T>` to `CriticalResult<U>` by applying a function.
    #[inline]
    #[must_use = "mapping has no effect without using the result"]
    pub fn map<U, F>(self, op: F) -> CriticalResult<U>
    where
        F: FnOnce(T) -> U,
    {
        CriticalResult {
            inner: self.inner.map(op),
        }
    }

    /// Maps a `CriticalResult<T>` by applying a function to the error.
    #[inline]
    #[must_use = "mapping has no effect without using the result"]
    pub fn map_err<F>(self, op: F) -> CriticalResult<T>
    where
        F: FnOnce(CriticalError) -> CriticalError,
    {
        CriticalResult {
            inner: self.inner.map_err(op),
        }
    }

    /// Calls `op` if the result is `Ok`, otherwise returns the `Err` value.
    #[inline]
    #[must_use = "flat mapping has no effect without using the result"]
    pub fn and_then<U, F>(self, op: F) -> CriticalResult<U>
    where
        F: FnOnce(T) -> CriticalResult<U>,
    {
        match self.inner {
            Ok(value) => op(value),
            Err(err) => CriticalResult::err(err),
        }
    }

    /// Calls `op` if the result is `Err`, otherwise returns the `Ok` value.
    #[inline]
    #[must_use = "recovery has no effect without using the result"]
    pub fn or_else<F>(self, op: F) -> CriticalResult<T>
    where
        F: FnOnce(CriticalError) -> CriticalResult<T>,
    {
        match self.inner {
            Ok(value) => CriticalResult::ok(value),
            Err(err) => op(err),
        }
    }

    /// Returns the error's fallback action if this is an error.
    #[inline]
    pub fn fallback_action(&self) -> Option<FallbackAction> {
        match &self.inner {
            Ok(_) => None,
            Err(err) => Some(err.fallback_action()),
        }
    }

    /// Returns whether the error is recoverable if this is an error.
    #[inline]
    pub fn is_recoverable(&self) -> Option<bool> {
        match &self.inner {
            Ok(_) => None,
            Err(err) => Some(err.is_recoverable()),
        }
    }

    /// Logs the error and returns a fallback value.
    ///
    /// This is a convenience method that logs the error at ERROR level
    /// and executes the fallback function.
    #[inline]
    #[must_use = "the result is not used"]
    pub fn unwrap_or_log<F>(self, context: &str, fallback: F) -> T
    where
        F: FnOnce(&CriticalError) -> T,
    {
        match self.inner {
            Ok(value) => value,
            Err(err) => {
                tracing::error!("{}: {} (fallback: {})", context, err, err.fallback_action());
                fallback(&err)
            }
        }
    }

    /// Converts to the underlying `Result<T, CriticalError>`.
    ///
    /// This method is provided for interop with code that expects
    /// standard Results, but should be used sparingly in critical paths.
    #[inline]
    pub fn into_result(self) -> Result<T, CriticalError> {
        self.inner
    }

    /// Inspects the contained value without consuming the result.
    #[inline]
    pub fn inspect<F>(self, f: F) -> Self
    where
        F: FnOnce(&T),
    {
        if let Ok(ref value) = self.inner {
            f(value);
        }
        self
    }

    /// Inspects the contained error without consuming the result.
    #[inline]
    pub fn inspect_err<F>(self, f: F) -> Self
    where
        F: FnOnce(&CriticalError),
    {
        if let Err(ref err) = self.inner {
            f(err);
        }
        self
    }
}

impl<T: Default> CriticalResult<T> {
    /// Returns the contained value or the default value of `T`.
    #[inline]
    #[must_use = "the result is not used"]
    pub fn unwrap_or_default(self) -> T {
        self.inner.unwrap_or_default()
    }
}

impl<T> From<Result<T, CriticalError>> for CriticalResult<T> {
    fn from(result: Result<T, CriticalError>) -> Self {
        Self::from_result(result)
    }
}

impl<T> From<CriticalResult<T>> for Result<T, CriticalError> {
    fn from(cr: CriticalResult<T>) -> Self {
        cr.inner
    }
}

impl<T: fmt::Display> fmt::Display for CriticalResult<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.inner {
            Ok(value) => write!(f, "Ok({})", value),
            Err(err) => write!(f, "Err({})", err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ok_result_is_ok() {
        let result: CriticalResult<i32> = CriticalResult::ok(42);
        assert!(result.is_ok());
        assert!(!result.is_err());
    }

    #[test]
    fn err_result_is_err() {
        let result: CriticalResult<i32> = CriticalResult::err(CriticalError::ProcessingFailed {
            reason: "test".to_string(),
            cause: None,
        });
        assert!(result.is_err());
        assert!(!result.is_ok());
    }

    #[test]
    fn unwrap_or_fallback_with_ok() {
        let result = CriticalResult::ok(42);
        let value = result.unwrap_or_fallback(|_| 0);
        assert_eq!(value, 42);
    }

    #[test]
    fn unwrap_or_fallback_with_err() {
        let result: CriticalResult<i32> = CriticalResult::err(CriticalError::ProcessingFailed {
            reason: "test".to_string(),
            cause: None,
        });
        let value = result.unwrap_or_fallback(|_| 0);
        assert_eq!(value, 0);
    }

    #[test]
    fn unwrap_or_with_default() {
        let result: CriticalResult<i32> = CriticalResult::err(CriticalError::ProcessingFailed {
            reason: "test".to_string(),
            cause: None,
        });
        let value = result.unwrap_or(99);
        assert_eq!(value, 99);
    }

    #[test]
    fn unwrap_or_default_works() {
        let result: CriticalResult<i32> = CriticalResult::err(CriticalError::ProcessingFailed {
            reason: "test".to_string(),
            cause: None,
        });
        let value = result.unwrap_or_default();
        assert_eq!(value, 0);
    }

    #[test]
    fn map_transforms_ok() {
        let result = CriticalResult::ok(42);
        let mapped = result.map(|x| x * 2);
        assert_eq!(mapped.unwrap_or(0), 84);
    }

    #[test]
    fn map_preserves_err() {
        let result: CriticalResult<i32> = CriticalResult::err(CriticalError::ProcessingFailed {
            reason: "test".to_string(),
            cause: None,
        });
        let mapped = result.map(|x| x * 2);
        assert!(mapped.is_err());
    }

    #[test]
    fn and_then_chains_ok() {
        let result = CriticalResult::ok(42);
        let chained = result.and_then(|x| CriticalResult::ok(x * 2));
        assert_eq!(chained.unwrap_or(0), 84);
    }

    #[test]
    fn and_then_propagates_err() {
        let result: CriticalResult<i32> = CriticalResult::err(CriticalError::ProcessingFailed {
            reason: "test".to_string(),
            cause: None,
        });
        let chained = result.and_then(|x| CriticalResult::ok(x * 2));
        assert!(chained.is_err());
    }

    #[test]
    fn or_else_recovers_from_err() {
        let result: CriticalResult<i32> = CriticalResult::err(CriticalError::ProcessingFailed {
            reason: "test".to_string(),
            cause: None,
        });
        let recovered = result.or_else(|_| CriticalResult::ok(99));
        assert_eq!(recovered.unwrap_or(0), 99);
    }

    #[test]
    fn fallback_action_returns_some_on_err() {
        let result: CriticalResult<i32> = CriticalResult::err(CriticalError::ProcessingFailed {
            reason: "test".to_string(),
            cause: None,
        });
        assert!(result.fallback_action().is_some());
    }

    #[test]
    fn fallback_action_returns_none_on_ok() {
        let result = CriticalResult::ok(42);
        assert!(result.fallback_action().is_none());
    }

    #[test]
    fn is_recoverable_on_recoverable_error() {
        let result: CriticalResult<i32> = CriticalResult::err(CriticalError::ProcessingFailed {
            reason: "test".to_string(),
            cause: None,
        });
        assert_eq!(result.is_recoverable(), Some(true));
    }

    #[test]
    fn is_recoverable_on_non_recoverable_error() {
        let result: CriticalResult<i32> = CriticalResult::err(CriticalError::DriverInitFailed {
            reason: "test".to_string(),
            cause: None,
        });
        assert_eq!(result.is_recoverable(), Some(false));
    }

    #[test]
    fn from_result_conversion() {
        let std_result: Result<i32, CriticalError> = Ok(42);
        let critical_result = CriticalResult::from_result(std_result);
        assert!(critical_result.is_ok());
    }

    #[test]
    fn into_result_conversion() {
        let critical_result = CriticalResult::ok(42);
        let std_result: Result<i32, CriticalError> = critical_result.into_result();
        assert_eq!(std_result.unwrap(), 42);
    }

    #[test]
    fn inspect_called_on_ok() {
        let mut inspected = false;
        let result = CriticalResult::ok(42);
        result.inspect(|_| inspected = true);
        assert!(inspected);
    }

    #[test]
    fn inspect_not_called_on_err() {
        let mut inspected = false;
        let result: CriticalResult<i32> = CriticalResult::err(CriticalError::ProcessingFailed {
            reason: "test".to_string(),
            cause: None,
        });
        result.inspect(|_| inspected = true);
        assert!(!inspected);
    }

    #[test]
    fn inspect_err_called_on_err() {
        let mut inspected = false;
        let result: CriticalResult<i32> = CriticalResult::err(CriticalError::ProcessingFailed {
            reason: "test".to_string(),
            cause: None,
        });
        result.inspect_err(|_| inspected = true);
        assert!(inspected);
    }

    #[test]
    fn inspect_err_not_called_on_ok() {
        let mut inspected = false;
        let result = CriticalResult::ok(42);
        result.inspect_err(|_| inspected = true);
        assert!(!inspected);
    }
}
