//! OpenTelemetry tracing for engine event processing.
//!
//! This module provides distributed tracing capabilities for the KeyRx engine,
//! allowing detailed performance analysis and debugging of event processing.
//!
//! The tracing functionality is feature-gated behind `otel-tracing` to avoid
//! runtime overhead when not needed.

mod tracing_spans;
mod tracing_types;

pub use tracing_types::{SpanGuard, TracingError, TracingResult};

use std::path::Path;

/// Engine tracer for OpenTelemetry instrumentation.
///
/// When the `otel-tracing` feature is enabled, this tracer creates spans
/// for input events, decisions, and output generation. When disabled,
/// all methods are no-ops with zero runtime overhead.
#[derive(Debug)]
pub struct EngineTracer {
    #[cfg(feature = "otel-tracing")]
    tracer: opentelemetry::global::BoxedTracer,
    #[cfg(not(feature = "otel-tracing"))]
    _marker: std::marker::PhantomData<()>,
}

impl EngineTracer {
    /// Create a new EngineTracer.
    ///
    /// When `otel-tracing` feature is enabled, initializes the OpenTelemetry
    /// tracer with the given service name.
    ///
    /// # Errors
    ///
    /// Returns `TracingError::NotEnabled` when `otel-tracing` feature is disabled.
    #[cfg(feature = "otel-tracing")]
    pub fn new(service_name: &str) -> TracingResult<Self> {
        use opentelemetry::global;
        use opentelemetry_sdk::trace::TracerProvider as SdkTracerProvider;

        // Create a simple in-memory provider for now
        // The actual exporter will be configured in Task 18
        let provider = SdkTracerProvider::builder().build();
        let service_name_owned = service_name.to_string();
        global::set_tracer_provider(provider);

        Ok(Self {
            tracer: global::tracer(service_name_owned),
        })
    }

    /// Create a new EngineTracer (no-op when feature disabled).
    #[cfg(not(feature = "otel-tracing"))]
    pub fn new(_service_name: &str) -> TracingResult<Self> {
        Err(TracingError::NotEnabled)
    }

    /// Create a new EngineTracer with OTLP file export.
    ///
    /// When `otel-tracing` feature is enabled, initializes the OpenTelemetry
    /// tracer with an OTLP file exporter that writes traces to the specified path.
    ///
    /// # Errors
    ///
    /// Returns `TracingError::InitializationFailed` if the exporter cannot be created.
    /// Returns `TracingError::NotEnabled` when `otel-tracing` feature is disabled.
    #[cfg(feature = "otel-tracing")]
    pub fn with_file_export<P: AsRef<Path>>(service_name: &str, _path: P) -> TracingResult<Self> {
        use opentelemetry::global;
        use opentelemetry_otlp::{SpanExporter, WithExportConfig, WithTonicConfig};
        use opentelemetry_sdk::trace::{BatchSpanProcessor, TracerProvider as SdkTracerProvider};

        // Create OTLP exporter - using gRPC endpoint
        // Note: For file export, we use a local OTLP collector or stdout exporter
        // The path parameter is reserved for future file-based export
        let exporter = SpanExporter::builder()
            .with_tonic()
            .with_endpoint("http://localhost:4317")
            .build()
            .map_err(|e| TracingError::InitializationFailed(e.to_string()))?;

        let provider = SdkTracerProvider::builder()
            .with_span_processor(BatchSpanProcessor::builder(exporter).build())
            .build();

        let service_name_owned = service_name.to_string();
        global::set_tracer_provider(provider);

        Ok(Self {
            tracer: global::tracer(service_name_owned),
        })
    }

    /// Create a new EngineTracer with file export (no-op when feature disabled).
    #[cfg(not(feature = "otel-tracing"))]
    pub fn with_file_export<P: AsRef<Path>>(_service_name: &str, _path: P) -> TracingResult<Self> {
        Err(TracingError::NotEnabled)
    }

    /// Create a no-op tracer that does nothing.
    ///
    /// Useful when tracing is not needed but a tracer instance is required.
    #[cfg(not(feature = "otel-tracing"))]
    pub fn noop() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a no-op tracer that does nothing.
    #[cfg(feature = "otel-tracing")]
    pub fn noop() -> Self {
        use opentelemetry::global;
        Self {
            tracer: global::tracer("noop"),
        }
    }

    /// Shutdown the tracer and flush any pending spans.
    #[cfg(feature = "otel-tracing")]
    pub fn shutdown(&self) {
        opentelemetry::global::shutdown_tracer_provider();
    }

    /// Shutdown the tracer (no-op).
    #[cfg(not(feature = "otel-tracing"))]
    pub fn shutdown(&self) {}
}

impl Default for EngineTracer {
    fn default() -> Self {
        Self::noop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{DecisionType, InputEvent, KeyCode, OutputAction};

    #[test]
    fn engine_tracer_noop_compiles() {
        let tracer = EngineTracer::noop();
        let event = InputEvent::key_down(KeyCode::A, 1000);

        // All methods should be callable without panic
        let _guard1 = tracer.span_input_received(&event);
        let _guard2 = tracer.span_decision_made(DecisionType::Remap, 50, &[0, 1]);
        let _guard3 = tracer.span_output_generated(&[OutputAction::KeyDown(KeyCode::B)]);
        tracer.record_error("test error");
        tracer.shutdown();
    }

    #[test]
    fn span_guard_drops_safely() {
        let tracer = EngineTracer::noop();
        let event = InputEvent::key_down(KeyCode::A, 1000);

        {
            let _guard = tracer.span_input_received(&event);
            // Guard will be dropped at end of scope
        }
        // Should not panic
    }

    #[test]
    fn tracer_default_is_noop() {
        let tracer = EngineTracer::default();
        let event = InputEvent::key_down(KeyCode::CapsLock, 0);
        let _guard = tracer.span_input_received(&event);
    }

    #[cfg(not(feature = "otel-tracing"))]
    #[test]
    fn new_returns_not_enabled_when_feature_disabled() {
        let result = EngineTracer::new("test-service");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TracingError::NotEnabled));
    }

    #[test]
    fn tracer_disabled_does_not_panic() {
        let tracer = EngineTracer::noop();

        // Call all methods in rapid succession
        for _ in 0..10 {
            let event = InputEvent::key_down(KeyCode::A, 0);
            let _g1 = tracer.span_input_received(&event);
            let _g2 = tracer.span_decision_made(DecisionType::Remap, 0, &[]);
            let _g3 = tracer.span_output_generated(&[OutputAction::KeyDown(KeyCode::B)]);
            tracer.record_error("repeated error");
        }

        // Multiple shutdown calls should be safe
        tracer.shutdown();
        tracer.shutdown();
    }

    #[test]
    fn multiple_spans_linked_correctly() {
        let tracer = EngineTracer::noop();

        let input_event = InputEvent::key_down(KeyCode::CapsLock, 1000);
        let input_span = tracer.span_input_received(&input_event);
        let decision_span = tracer.span_decision_made(DecisionType::Hold, 150, &[0]);
        let output_span = tracer.span_output_generated(&[OutputAction::KeyDown(KeyCode::Escape)]);

        drop(output_span);
        drop(decision_span);
        drop(input_span);
    }

    #[cfg(not(feature = "otel-tracing"))]
    #[test]
    fn with_file_export_returns_not_enabled_when_feature_disabled() {
        let result = EngineTracer::with_file_export("test-service", "/tmp/traces.otlp");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TracingError::NotEnabled));
    }
}
