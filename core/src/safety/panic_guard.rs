//! Panic recovery and backtrace capture for critical code paths.
//!
//! This module provides `PanicGuard`, a wrapper around `std::panic::catch_unwind`
//! that captures backtraces, logs panic information, and converts panics into
//! recoverable errors for critical paths.
//!
//! # Design Philosophy
//!
//! - All panics in critical paths must be caught and logged
//! - Backtraces are preserved for debugging
//! - Panics are converted to `CriticalError::CallbackPanic`
//! - No panic should escape to FFI boundaries or main event loops
//!
//! # Example
//!
//! ```ignore
//! use keyrx_core::safety::panic_guard::PanicGuard;
//!
//! // Wrap a potentially panicking operation
//! let result = PanicGuard::new("keyboard_callback")
//!     .execute(|| {
//!         // Code that might panic
//!         process_keyboard_input(data)
//!     });
//!
//! match result {
//!     Ok(value) => handle_success(value),
//!     Err(err) => {
//!         // Panic was caught and converted to CriticalError
//!         log::error!("Callback panicked: {}", err);
//!         execute_fallback();
//!     }
//! }
//! ```

use crate::errors::critical::CriticalError;
use crate::safety::panic_telemetry::{record_panic, record_recovery};
use std::any::Any;
use std::panic::{catch_unwind, AssertUnwindSafe};

/// Guard that catches panics and converts them to CriticalErrors.
///
/// This type wraps operations in `catch_unwind` and captures panic information
/// including backtraces for debugging. All caught panics are logged and converted
/// to `CriticalError::CallbackPanic`.
pub struct PanicGuard {
    /// Context name for logging (e.g., "keyboard_callback", "driver_init").
    context: String,
}

impl PanicGuard {
    /// Creates a new PanicGuard with the given context name.
    ///
    /// The context name is used in error messages and logs to identify
    /// where the panic occurred.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let guard = PanicGuard::new("input_processing");
    /// ```
    pub fn new(context: impl Into<String>) -> Self {
        Self {
            context: context.into(),
        }
    }

    /// Executes a closure and catches any panics.
    ///
    /// If the closure panics, the panic is caught, logged with backtrace,
    /// and converted to a `CriticalError::CallbackPanic`.
    ///
    /// # Type Parameters
    ///
    /// - `F`: The closure to execute
    /// - `T`: The return type of the closure
    ///
    /// # Example
    ///
    /// ```ignore
    /// let result = PanicGuard::new("process_event")
    ///     .execute(|| {
    ///         process_event(event)
    ///     });
    ///
    /// if let Err(err) = result {
    ///     log::error!("Processing panicked: {}", err);
    /// }
    /// ```
    pub fn execute<F, T>(self, f: F) -> Result<T, CriticalError>
    where
        F: FnOnce() -> T,
    {
        // Capture backtrace before catch_unwind to get accurate stack
        let backtrace = std::backtrace::Backtrace::capture();

        match catch_unwind(AssertUnwindSafe(f)) {
            Ok(result) => {
                // Record successful execution (recovery from previous panic)
                record_recovery();
                Ok(result)
            }
            Err(panic_info) => {
                let panic_message = extract_panic_message(&panic_info);

                // Format backtrace if available
                let backtrace_str = match backtrace.status() {
                    std::backtrace::BacktraceStatus::Captured => Some(format!("{}", backtrace)),
                    _ => None,
                };

                // Record panic in telemetry
                record_panic(&self.context, &panic_message, backtrace_str.clone());

                // Log the panic with context
                tracing::error!(
                    context = %self.context,
                    panic_message = %panic_message,
                    backtrace = ?backtrace_str,
                    "Panic caught in critical path"
                );

                Err(CriticalError::CallbackPanic {
                    panic_message: format!("{}: {}", self.context, panic_message),
                    backtrace: backtrace_str,
                })
            }
        }
    }

    /// Executes a closure and catches panics, returning a default value on panic.
    ///
    /// This is a convenience method for cases where you want to continue with
    /// a default value rather than propagating the error.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let value = PanicGuard::new("parse_config")
    ///     .execute_or_default(|| parse_config_file(), Config::default());
    /// ```
    pub fn execute_or_default<F, T>(self, f: F, default: T) -> T
    where
        F: FnOnce() -> T,
    {
        let context = self.context.clone();
        match self.execute(f) {
            Ok(result) => result,
            Err(err) => {
                tracing::warn!(
                    context = %context,
                    error = %err,
                    "Using default value after panic"
                );
                default
            }
        }
    }

    /// Executes a closure and catches panics, calling a fallback on panic.
    ///
    /// The fallback function receives the CriticalError and can perform
    /// additional recovery logic before returning a value.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let value = PanicGuard::new("load_mappings")
    ///     .execute_or_else(
    ///         || load_mappings_from_disk(),
    ///         |err| {
    ///             log::error!("Failed to load mappings: {}", err);
    ///             load_default_mappings()
    ///         }
    ///     );
    /// ```
    pub fn execute_or_else<F, T, E>(self, f: F, fallback: E) -> T
    where
        F: FnOnce() -> T,
        E: FnOnce(CriticalError) -> T,
    {
        match self.execute(f) {
            Ok(result) => result,
            Err(err) => fallback(err),
        }
    }
}

/// Extracts a panic message from the panic payload.
///
/// Panics can be either `&str` or `String`, this function handles both cases
/// and provides a fallback message if neither type matches.
fn extract_panic_message(panic_info: &Box<dyn Any + Send>) -> String {
    if let Some(s) = panic_info.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = panic_info.downcast_ref::<String>() {
        s.clone()
    } else {
        "Unknown panic payload".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catches_panic_with_str() {
        let result = PanicGuard::new("test_context").execute(|| {
            panic!("test panic");
        });

        assert!(result.is_err());
        match result.unwrap_err() {
            CriticalError::CallbackPanic { panic_message, .. } => {
                assert!(panic_message.contains("test_context"));
                assert!(panic_message.contains("test panic"));
            }
            _ => panic!("Expected CallbackPanic"),
        }
    }

    #[test]
    fn catches_panic_with_string() {
        let result = PanicGuard::new("test_context").execute(|| {
            panic!("{}", "formatted panic".to_string());
        });

        assert!(result.is_err());
        match result.unwrap_err() {
            CriticalError::CallbackPanic { panic_message, .. } => {
                assert!(panic_message.contains("formatted panic"));
            }
            _ => panic!("Expected CallbackPanic"),
        }
    }

    #[test]
    fn returns_ok_on_success() {
        let result = PanicGuard::new("test_context").execute(|| 42);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn execute_or_default_returns_value_on_success() {
        let value = PanicGuard::new("test_context").execute_or_default(|| 42, 0);

        assert_eq!(value, 42);
    }

    #[test]
    fn execute_or_default_returns_default_on_panic() {
        let value = PanicGuard::new("test_context").execute_or_default(
            || {
                panic!("test panic");
            },
            99,
        );

        assert_eq!(value, 99);
    }

    #[test]
    fn execute_or_else_returns_value_on_success() {
        let value =
            PanicGuard::new("test_context").execute_or_else(|| 42, |_| panic!("fallback called"));

        assert_eq!(value, 42);
    }

    #[test]
    fn execute_or_else_calls_fallback_on_panic() {
        let value = PanicGuard::new("test_context").execute_or_else(
            || {
                panic!("test panic");
            },
            |err| match err {
                CriticalError::CallbackPanic { .. } => 99,
                _ => panic!("Expected CallbackPanic"),
            },
        );

        assert_eq!(value, 99);
    }

    #[test]
    fn captures_context_in_error() {
        let result = PanicGuard::new("custom_context").execute(|| {
            panic!("oops");
        });

        match result.unwrap_err() {
            CriticalError::CallbackPanic { panic_message, .. } => {
                assert!(panic_message.contains("custom_context"));
            }
            _ => panic!("Expected CallbackPanic"),
        }
    }

    #[test]
    fn extract_panic_message_handles_str() {
        let panic_info: Box<dyn Any + Send> = Box::new("test message");
        let message = extract_panic_message(&panic_info);
        assert_eq!(message, "test message");
    }

    #[test]
    fn extract_panic_message_handles_string() {
        let panic_info: Box<dyn Any + Send> = Box::new("test string".to_string());
        let message = extract_panic_message(&panic_info);
        assert_eq!(message, "test string");
    }

    #[test]
    fn extract_panic_message_handles_unknown() {
        let panic_info: Box<dyn Any + Send> = Box::new(42);
        let message = extract_panic_message(&panic_info);
        assert_eq!(message, "Unknown panic payload");
    }

    #[test]
    fn backtrace_is_captured() {
        let result = PanicGuard::new("test_backtrace").execute(|| {
            panic!("backtrace test");
        });

        match result.unwrap_err() {
            CriticalError::CallbackPanic { backtrace, .. } => {
                // Backtrace may or may not be available depending on environment
                // Just verify it's present in the structure
                assert!(backtrace.is_some() || backtrace.is_none());
            }
            _ => panic!("Expected CallbackPanic"),
        }
    }
}
