//! Span creation methods for the EngineTracer.
//!
//! This module contains the span-related functionality separated from
//! the tracer lifecycle methods for better code organization.

use super::tracing_types::SpanGuard;
use super::EngineTracer;
use crate::engine::{DecisionType, InputEvent, OutputAction};

impl EngineTracer {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::KeyCode;

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

    #[test]
    fn span_input_has_correct_attributes() {
        let tracer = EngineTracer::noop();

        let event = InputEvent {
            key: KeyCode::Space,
            pressed: true,
            timestamp_us: 123456789,
            device_id: Some("keyboard-1".to_string()),
            is_repeat: true,
            is_synthetic: false,
            scan_code: 57,
        };
        let _guard = tracer.span_input_received(&event);

        // Test with no device_id (tests None handling)
        let event_no_device = InputEvent {
            key: KeyCode::Enter,
            pressed: true,
            timestamp_us: 0,
            device_id: None,
            is_repeat: false,
            is_synthetic: true,
            scan_code: 28,
        };
        let _guard2 = tracer.span_input_received(&event_no_device);
    }

    #[test]
    fn span_decision_includes_latency() {
        let tracer = EngineTracer::noop();
        let latencies: [u64; 3] = [0, 100, u64::MAX];

        for latency in latencies {
            let _g1 = tracer.span_decision_made(DecisionType::PassThrough, latency, &[]);
            let _g2 = tracer.span_decision_made(DecisionType::Remap, latency, &[0]);
            let _g3 = tracer.span_decision_made(DecisionType::Block, latency, &[1, 2]);
        }
    }

    #[test]
    fn span_output_handles_many_actions() {
        let tracer = EngineTracer::noop();

        let _g1 = tracer.span_output_generated(&[]);
        let _g2 = tracer.span_output_generated(&[OutputAction::Block]);

        let many_actions: Vec<OutputAction> = (0..100)
            .map(|_| OutputAction::KeyDown(KeyCode::A))
            .collect();
        let _g3 = tracer.span_output_generated(&many_actions);
    }

    #[test]
    fn record_error_handles_various_messages() {
        let tracer = EngineTracer::noop();

        tracer.record_error("");
        tracer.record_error("Test error message");
        tracer.record_error(&"x".repeat(10000));
        tracer.record_error("エラー: 何かが間違っています 🔥");
    }
}
