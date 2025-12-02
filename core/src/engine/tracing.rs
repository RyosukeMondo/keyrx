//! OpenTelemetry tracing for engine event processing.
//!
//! This module provides distributed tracing capabilities for the KeyRx engine,
//! allowing detailed performance analysis and debugging of event processing.
//!
//! The tracing functionality is feature-gated behind `otel-tracing` to avoid
//! runtime overhead when not needed.

use crate::engine::{DecisionType, InputEvent, OutputAction};

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
    _context: opentelemetry::Context,
    #[cfg(not(feature = "otel-tracing"))]
    _marker: std::marker::PhantomData<()>,
}

impl SpanGuard {
    #[cfg(feature = "otel-tracing")]
    fn new(context: opentelemetry::Context) -> Self {
        Self { _context: context }
    }

    #[cfg(not(feature = "otel-tracing"))]
    fn new_noop() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

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

    /// Start a span for an input event received.
    ///
    /// Records attributes: key, pressed, timestamp, device_id, is_repeat, is_synthetic
    #[cfg(feature = "otel-tracing")]
    pub fn span_input_received(&self, event: &InputEvent) -> SpanGuard {
        use opentelemetry::trace::{TraceContextExt, Tracer};
        use opentelemetry::{Context, KeyValue};

        let span = self
            .tracer
            .span_builder("input_received")
            .with_attributes([
                KeyValue::new("key", format!("{:?}", event.key)),
                KeyValue::new("pressed", event.pressed),
                KeyValue::new("timestamp_us", event.timestamp_us as i64),
                KeyValue::new("device_id", event.device_id.clone().unwrap_or_default()),
                KeyValue::new("is_repeat", event.is_repeat),
                KeyValue::new("is_synthetic", event.is_synthetic),
                KeyValue::new("scan_code", event.scan_code as i64),
            ])
            .start(&self.tracer);

        let context = Context::current().with_span(span);
        SpanGuard::new(context)
    }

    /// Start a span for an input event received (no-op).
    #[cfg(not(feature = "otel-tracing"))]
    pub fn span_input_received(&self, _event: &InputEvent) -> SpanGuard {
        SpanGuard::new_noop()
    }

    /// Start a span for decision made by the engine.
    ///
    /// Records attributes: decision_type, latency_us, active_layers
    #[cfg(feature = "otel-tracing")]
    pub fn span_decision_made(
        &self,
        decision: DecisionType,
        latency_us: u64,
        active_layers: &[u32],
    ) -> SpanGuard {
        use opentelemetry::trace::{TraceContextExt, Tracer};
        use opentelemetry::{Context, KeyValue};

        let span = self
            .tracer
            .span_builder("decision_made")
            .with_attributes([
                KeyValue::new("decision_type", format!("{:?}", decision)),
                KeyValue::new("latency_us", latency_us as i64),
                KeyValue::new("active_layers", format!("{:?}", active_layers)),
                KeyValue::new("layer_count", active_layers.len() as i64),
            ])
            .start(&self.tracer);

        let context = Context::current().with_span(span);
        SpanGuard::new(context)
    }

    /// Start a span for decision made (no-op).
    #[cfg(not(feature = "otel-tracing"))]
    pub fn span_decision_made(
        &self,
        _decision: DecisionType,
        _latency_us: u64,
        _active_layers: &[u32],
    ) -> SpanGuard {
        SpanGuard::new_noop()
    }

    /// Start a span for output generation.
    ///
    /// Records attributes: action_count, actions (serialized)
    #[cfg(feature = "otel-tracing")]
    pub fn span_output_generated(&self, actions: &[OutputAction]) -> SpanGuard {
        use opentelemetry::trace::{TraceContextExt, Tracer};
        use opentelemetry::{Context, KeyValue};

        // Serialize first few actions for debugging (avoid huge spans)
        let actions_preview: Vec<String> =
            actions.iter().take(5).map(|a| format!("{:?}", a)).collect();

        let span = self
            .tracer
            .span_builder("output_generated")
            .with_attributes([
                KeyValue::new("action_count", actions.len() as i64),
                KeyValue::new("actions", format!("{:?}", actions_preview)),
            ])
            .start(&self.tracer);

        let context = Context::current().with_span(span);
        SpanGuard::new(context)
    }

    /// Start a span for output generation (no-op).
    #[cfg(not(feature = "otel-tracing"))]
    pub fn span_output_generated(&self, _actions: &[OutputAction]) -> SpanGuard {
        SpanGuard::new_noop()
    }

    /// Record an error event on the current trace.
    #[cfg(feature = "otel-tracing")]
    pub fn record_error(&self, error: &str) {
        use opentelemetry::trace::{Span, Status, Tracer};

        let mut span = self.tracer.start("error");
        span.set_status(Status::error(error.to_string()));
    }

    /// Record an error event (no-op).
    #[cfg(not(feature = "otel-tracing"))]
    pub fn record_error(&self, _error: &str) {}

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
    use crate::engine::KeyCode;

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

    #[cfg(not(feature = "otel-tracing"))]
    #[test]
    fn new_returns_not_enabled_when_feature_disabled() {
        let result = EngineTracer::new("test-service");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TracingError::NotEnabled));
    }

    #[test]
    fn span_methods_accept_correct_types() {
        let tracer = EngineTracer::noop();

        // Test InputEvent
        let event = InputEvent {
            key: KeyCode::A,
            pressed: true,
            timestamp_us: 1000,
            device_id: Some("test-device".to_string()),
            is_repeat: false,
            is_synthetic: false,
            scan_code: 30,
        };
        let _g1 = tracer.span_input_received(&event);

        // Test DecisionType variants
        let _g2 = tracer.span_decision_made(DecisionType::PassThrough, 100, &[]);
        let _g3 = tracer.span_decision_made(DecisionType::Remap, 50, &[0]);
        let _g4 = tracer.span_decision_made(DecisionType::Block, 25, &[0, 1, 2]);
        let _g5 = tracer.span_decision_made(DecisionType::Tap, 200, &[0]);
        let _g6 = tracer.span_decision_made(DecisionType::Hold, 500, &[1]);

        // Test OutputAction variants
        let _g7 = tracer.span_output_generated(&[]);
        let _g8 = tracer.span_output_generated(&[OutputAction::Block]);
        let _g9 = tracer.span_output_generated(&[
            OutputAction::KeyDown(KeyCode::Escape),
            OutputAction::KeyUp(KeyCode::Escape),
        ]);
    }
}
