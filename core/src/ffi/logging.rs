use crate::ffi::domains::engine::global_event_registry;
use crate::ffi::events::EventType;
use serde::Serialize;
use std::fmt;
use tracing::field::{Field, Visit};
use tracing::{Event, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;

/// JSON payload structure for diagnostics logs
#[derive(Debug, Serialize)]
struct DiagnosticsLogPayload {
    level: String,
    message: String,
    target: String,
    fields: std::collections::HashMap<String, String>,
}

/// A tracing layer that forwards events to the FFI event registry.
pub struct FfiLoggingLayer;

impl<S> Layer<S> for FfiLoggingLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut visitor = JsonVisitor::default();
        event.record(&mut visitor);

        let payload = DiagnosticsLogPayload {
            level: event.metadata().level().to_string(),
            message: visitor.message,
            target: event.metadata().target().to_string(),
            fields: visitor.fields,
        };

        // Invoke the callback if registered.
        // We use the global event registry directly.
        // Note: This might be chatty, so we might want to filter by level or target in the future.
        global_event_registry().invoke(EventType::DiagnosticsLog, &payload);
    }
}

#[derive(Default)]
struct JsonVisitor {
    message: String,
    fields: std::collections::HashMap<String, String>,
}

impl Visit for JsonVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
        } else {
            self.fields
                .insert(field.name().to_string(), format!("{:?}", value));
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        } else {
            self.fields
                .insert(field.name().to_string(), value.to_string());
        }
    }
}
