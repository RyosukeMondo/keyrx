//! Observability module for KeyRx.
//!
//! This module provides structured logging, metrics collection, and FFI bridges
//! for exposing observability data to the Flutter UI.

pub mod bridge;
pub mod entry;
pub mod logger;
pub mod macros;
pub mod metrics_bridge;
pub mod otel;

// Re-export commonly used types
pub use bridge::LogBridge;
pub use entry::{LogEntry, LogLevel};
pub use logger::{LogError, OutputFormat, StructuredLogger};
pub use metrics_bridge::{
    MetricsBridge, MetricsCollector, MetricsSnapshot, MetricsSnapshotFfi, NoOpMetricsCollector,
    Operation,
};
pub use otel::{
    build_metrics_exporter, build_otel_layer, shutdown_tracer_provider, BoxedOtelLayer, OtelConfig,
    OtelConfigError, OtelLayerError, OtelMetricsError, OtelMetricsExporter,
};
