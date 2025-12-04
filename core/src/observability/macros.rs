//! Convenience macros for structured logging.
//!
//! This module provides ergonomic macros that wrap the tracing crate's functionality
//! with consistent field naming and patterns used throughout KeyRx.
//!
//! # Examples
//!
//! ```rust
//! use keyrx_core::log_event;
//! use tracing::Level;
//!
//! // Log an event with structured fields
//! log_event!(Level::INFO, "key_processed",
//!     key_code = 65,
//!     latency_us = 150,
//! );
//! ```
//!
//! ```rust
//! use keyrx_core::log_error;
//!
//! let result: Result<(), std::io::Error> = Err(std::io::Error::new(
//!     std::io::ErrorKind::NotFound,
//!     "file not found"
//! ));
//!
//! if let Err(e) = result {
//!     log_error!(e, "Failed to open config file");
//! }
//! ```
//!
//! ```rust
//! use keyrx_core::timed_span;
//!
//! let _span = timed_span!("processing_keys", batch_size = 10).entered();
//! // ... do work ...
//! // Span timing is automatically recorded when dropped
//! ```

/// Log an event with structured context.
///
/// This macro wraps `tracing::event!` to ensure consistent field naming
/// across the codebase. All events include an "event" field describing
/// what happened.
///
/// # Arguments
///
/// * `$level` - The log level (e.g., `Level::INFO`, `Level::ERROR`)
/// * `$event` - A string literal describing the event (e.g., "key_processed")
/// * `$($field)*` - Optional key-value pairs for additional context
///
/// # Examples
///
/// ```rust
/// use keyrx_core::log_event;
/// use tracing::Level;
///
/// // Simple event
/// log_event!(Level::INFO, "engine_started");
///
/// // Event with context fields
/// log_event!(Level::DEBUG, "key_mapped",
///     from_key = "a",
///     to_key = "b",
/// );
///
/// // Event with computed values
/// let latency = 42;
/// log_event!(Level::INFO, "request_completed",
///     latency_us = latency,
///     status = "success",
/// );
/// ```
#[macro_export]
macro_rules! log_event {
    ($level:expr, $event:literal) => {
        tracing::event!($level, event = $event);
    };
    ($level:expr, $event:literal, $($field:tt)*) => {
        tracing::event!($level, event = $event, $($field)*);
    };
}

/// Log an error with structured context.
///
/// This macro standardizes error logging by always including the error
/// message, a context description, and optional additional fields.
///
/// # Arguments
///
/// * `$err` - The error value (must implement `Display`)
/// * `$context` - A string literal describing where/why the error occurred
/// * `$($field)*` - Optional key-value pairs for additional context
///
/// # Examples
///
/// ```rust
/// use keyrx_core::log_error;
/// use std::io;
///
/// let err = io::Error::new(io::ErrorKind::NotFound, "config.toml");
///
/// // Simple error log
/// log_error!(err, "Failed to load configuration");
///
/// // Error with additional context
/// log_error!(err, "Failed to load configuration",
///     config_path = "/etc/keyrx/config.toml",
///     attempted_reload = true,
/// );
/// ```
///
/// # Output Format
///
/// The generated log will always include:
/// - `error`: The error's Display representation
/// - `context`: The provided context string
/// - Any additional fields provided
/// - Message: "Error occurred"
#[macro_export]
macro_rules! log_error {
    ($err:expr, $context:literal) => {
        tracing::error!(
            error = %$err,
            context = $context,
            "Error occurred"
        );
    };
    ($err:expr, $context:literal, $($field:tt)*) => {
        tracing::error!(
            error = %$err,
            context = $context,
            $($field)*
            "Error occurred"
        );
    };
}

/// Create a timed span for performance tracking.
///
/// This macro wraps `tracing::info_span!` to create spans that automatically
/// track execution time. When the span is entered and later dropped, the
/// duration is recorded in the logs.
///
/// # Arguments
///
/// * `$name` - A string literal naming the span (e.g., "process_batch")
/// * `$($field)*` - Optional key-value pairs for span context
///
/// # Examples
///
/// ```rust
/// use keyrx_core::timed_span;
///
/// fn process_keys(keys: &[u32]) {
///     let span = timed_span!("process_keys", count = keys.len());
///     let _guard = span.entered();
///
///     // ... process keys ...
///
///     // Span timing is automatically recorded when _guard is dropped
/// }
/// ```
///
/// ```rust
/// use keyrx_core::timed_span;
///
/// // Simple span without fields
/// let _span = timed_span!("database_query").entered();
/// // ... do work ...
/// ```
///
/// # Performance Tracking
///
/// To see span timings in logs, configure the logger with span events enabled:
///
/// ```rust,no_run
/// use keyrx_core::observability::StructuredLogger;
///
/// StructuredLogger::new()
///     .with_span_events(true)
///     .init()
///     .expect("Failed to initialize logger");
/// ```
#[macro_export]
macro_rules! timed_span {
    ($name:literal) => {
        tracing::info_span!($name)
    };
    ($name:literal, $($field:tt)*) => {
        tracing::info_span!($name, $($field)*)
    };
}

#[cfg(test)]
mod tests {
    use tracing::Level;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    // Helper to capture log output for testing
    fn init_test_logger() {
        let _ = tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer().with_test_writer())
            .try_init();
    }

    #[test]
    fn test_log_event_simple() {
        init_test_logger();
        log_event!(Level::INFO, "test_event");
    }

    #[test]
    fn test_log_event_with_fields() {
        init_test_logger();
        log_event!(
            Level::DEBUG,
            "key_processed",
            key_code = 65,
            latency_us = 150,
        );
    }

    #[test]
    fn test_log_error_simple() {
        init_test_logger();
        let err = std::io::Error::new(std::io::ErrorKind::NotFound, "test file");
        log_error!(err, "Failed to open file");
    }

    #[test]
    fn test_log_error_with_fields() {
        init_test_logger();
        let err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "config.toml");
        log_error!(
            err,
            "Failed to read configuration",
            path = "/etc/keyrx/config.toml",
            retry_count = 3,
        );
    }

    #[test]
    fn test_timed_span_simple() {
        init_test_logger();
        let span = timed_span!("test_operation");
        let _guard = span.entered();
        // Simulate some work
        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    #[test]
    fn test_timed_span_with_fields() {
        init_test_logger();
        let span = timed_span!("batch_processing", batch_size = 100, priority = "high");
        let _guard = span.entered();
        // Simulate some work
        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    #[test]
    fn test_nested_spans() {
        init_test_logger();
        let outer = timed_span!("outer_operation");
        let _outer_guard = outer.entered();

        log_event!(Level::INFO, "outer_started");

        {
            let inner = timed_span!("inner_operation", step = 1);
            let _inner_guard = inner.entered();
            log_event!(Level::DEBUG, "inner_processing");
        }

        log_event!(Level::INFO, "outer_completed");
    }

    #[test]
    fn test_error_with_result() {
        init_test_logger();

        fn failing_operation() -> Result<(), std::io::Error> {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "operation failed",
            ))
        }

        if let Err(e) = failing_operation() {
            log_error!(e, "Operation failed in test");
        }
    }

    #[test]
    fn test_event_with_different_levels() {
        init_test_logger();

        log_event!(Level::TRACE, "trace_event");
        log_event!(Level::DEBUG, "debug_event");
        log_event!(Level::INFO, "info_event");
        log_event!(Level::WARN, "warn_event");
        log_event!(Level::ERROR, "error_event");
    }
}
