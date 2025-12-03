//! Tracing type definitions for engine instrumentation.
//!
//! This module contains error types and span guard types used by
//! the `EngineTracer` for OpenTelemetry tracing operations.

/// Result type for tracing operations.
pub type TracingResult<T> = Result<T, TracingError>;

/// Error type for tracing operations.
#[derive(Debug)]
pub enum TracingError {
    /// Failed to initialize the tracer provider.
    InitializationFailed(String),
    /// Failed to export traces.
    ExportFailed(String),
    /// Tracer is not enabled.
    NotEnabled,
}

impl std::fmt::Display for TracingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TracingError::InitializationFailed(msg) => {
                write!(f, "Tracer initialization failed: {}", msg)
            }
            TracingError::ExportFailed(msg) => write!(f, "Trace export failed: {}", msg),
            TracingError::NotEnabled => write!(f, "OpenTelemetry tracing is not enabled"),
        }
    }
}

impl std::error::Error for TracingError {}

/// Span guard that ends the span when dropped.
///
/// This provides RAII-style span lifecycle management.
#[derive(Debug)]
pub struct SpanGuard {
    #[cfg(feature = "otel-tracing")]
    pub(super) _context: opentelemetry::Context,
    #[cfg(not(feature = "otel-tracing"))]
    pub(super) _marker: std::marker::PhantomData<()>,
}

impl SpanGuard {
    #[cfg(feature = "otel-tracing")]
    pub(super) fn new(context: opentelemetry::Context) -> Self {
        Self { _context: context }
    }

    #[cfg(not(feature = "otel-tracing"))]
    pub(super) fn new_noop() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }

    #[cfg(feature = "otel-tracing")]
    pub(super) fn new_noop() -> Self {
        Self {
            _context: opentelemetry::Context::current(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracing_error_display() {
        let err = TracingError::NotEnabled;
        let display = format!("{}", err);
        assert!(display.contains("not enabled"));

        let err = TracingError::InitializationFailed("test".to_string());
        let display = format!("{}", err);
        assert!(display.contains("initialization failed"));
        assert!(display.contains("test"));

        let err = TracingError::ExportFailed("export test".to_string());
        let display = format!("{}", err);
        assert!(display.contains("export failed"));
    }

    #[test]
    fn tracing_error_implements_std_error() {
        fn assert_error<E: std::error::Error>(_: &E) {}

        let err1 = TracingError::NotEnabled;
        let err2 = TracingError::InitializationFailed("test".to_string());
        let err3 = TracingError::ExportFailed("test".to_string());

        assert_error(&err1);
        assert_error(&err2);
        assert_error(&err3);
    }

    #[test]
    fn tracing_error_debug_formatting() {
        let err1 = TracingError::NotEnabled;
        let err2 = TracingError::InitializationFailed("init error".to_string());
        let err3 = TracingError::ExportFailed("export error".to_string());

        let debug1 = format!("{:?}", err1);
        let debug2 = format!("{:?}", err2);
        let debug3 = format!("{:?}", err3);

        assert!(debug1.contains("NotEnabled"));
        assert!(debug2.contains("InitializationFailed"));
        assert!(debug2.contains("init error"));
        assert!(debug3.contains("ExportFailed"));
        assert!(debug3.contains("export error"));
    }

    #[test]
    fn span_guard_drops_safely() {
        let guard = SpanGuard::new_noop();
        drop(guard);
        // Should not panic
    }
}
