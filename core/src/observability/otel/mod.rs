//! OpenTelemetry observability utilities.

pub mod config;
pub mod layer;

pub use config::{OtelConfig, OtelConfigError};
pub use layer::{build_otel_layer, shutdown_tracer_provider, BoxedOtelLayer, OtelLayerError};
