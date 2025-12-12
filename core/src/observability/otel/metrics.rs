//! OpenTelemetry metrics export helpers.
//!
//! This module wires KeyRx's internal metrics to OTEL by creating a meter
//! provider with an OTLP exporter and exposes a helper to record snapshots
//! into histogram/counter/gauge instruments. When OTEL is disabled or the
//! `otel-tracing` feature is not compiled, the builder returns `None` so
//! callers can skip exporting without additional branching.

use super::OtelConfig;
use crate::metrics::MetricsSnapshot;

/// Errors that can occur while building the OTEL metrics exporter.
#[derive(Debug, thiserror::Error)]
pub enum OtelMetricsError {
    /// Tokio runtime is required for the periodic reader/exporter workers.
    #[error("Tokio runtime not available for OTEL metrics exporter")]
    MissingRuntime,

    /// Failed to construct the OTEL metrics pipeline.
    #[cfg(feature = "otel-tracing")]
    #[error("Failed to build OTEL metrics exporter: {0}")]
    Exporter(#[from] opentelemetry_sdk::metrics::MetricError),
}

/// Handle for recording KeyRx metrics into OTEL instruments.
///
/// When the `otel-tracing` feature is disabled this is an empty struct that
/// performs no work, allowing call sites to be feature-agnostic.
pub struct OtelMetricsExporter {
    #[cfg(feature = "otel-tracing")]
    meter_provider: opentelemetry_sdk::metrics::SdkMeterProvider,
    #[cfg(feature = "otel-tracing")]
    latency_histogram: opentelemetry::metrics::Histogram<f64>,
    #[cfg(feature = "otel-tracing")]
    operation_counter: opentelemetry::metrics::Counter<u64>,
    #[cfg(feature = "otel-tracing")]
    memory_gauge: opentelemetry::metrics::Gauge<u64>,
    #[cfg(feature = "otel-tracing")]
    profile_histogram: opentelemetry::metrics::Histogram<f64>,
    #[cfg(feature = "otel-tracing")]
    last_counts: std::sync::Mutex<std::collections::HashMap<String, u64>>,
}

impl std::fmt::Debug for OtelMetricsExporter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OtelMetricsExporter").finish()
    }
}

impl OtelMetricsExporter {
    /// Record a metrics snapshot into OTEL instruments.
    ///
    /// - Latencies are recorded into a histogram with explicit buckets and an
    ///   `operation` attribute.
    /// - Operation counts are emitted as monotonic counters using deltas to
    ///   avoid double-counting across snapshots.
    /// - Memory readings are exported as gauges for current and peak usage.
    /// - Profile timings are recorded as histograms keyed by profile name.
    #[cfg(feature = "otel-tracing")]
    pub fn record_snapshot(&self, snapshot: &MetricsSnapshot) {
        use opentelemetry::KeyValue;

        let mut counts = self
            .last_counts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        for (operation, stats) in &snapshot.latencies {
            let attrs = [KeyValue::new("operation", operation.clone())];

            // Record representative percentiles to keep export payload light
            // while still populating histogram buckets.
            self.latency_histogram.record(stats.p50 as f64, &attrs);
            self.latency_histogram.record(stats.p95 as f64, &attrs);
            self.latency_histogram.record(stats.p99 as f64, &attrs);

            let previous = counts.get(operation).copied().unwrap_or_default();
            if stats.count > previous {
                self.operation_counter.add(stats.count - previous, &attrs);
            }
            counts.insert(operation.clone(), stats.count);
        }

        self.memory_gauge.record(
            snapshot.memory.current as u64,
            &[KeyValue::new("state", "current")],
        );
        self.memory_gauge.record(
            snapshot.memory.peak as u64,
            &[KeyValue::new("state", "peak")],
        );

        for (profile, stats) in &snapshot.profiles {
            let attrs = [KeyValue::new("profile", profile.clone())];
            self.profile_histogram
                .record(stats.avg_micros as f64, &attrs);
            self.profile_histogram
                .record(stats.max_micros as f64, &attrs);
        }
    }

    /// No-op stub when OTEL metrics are not compiled.
    #[cfg(not(feature = "otel-tracing"))]
    #[allow(unused_variables)]
    pub fn record_snapshot(&self, snapshot: &MetricsSnapshot) {}

    /// Flush and shut down the metrics provider.
    #[cfg(feature = "otel-tracing")]
    pub fn shutdown(&self) {
        let _ = self.meter_provider.shutdown();
    }

    /// No-op shutdown when OTEL metrics are not compiled.
    #[cfg(not(feature = "otel-tracing"))]
    pub fn shutdown(&self) {}
}

/// Build an OTEL metrics exporter using the provided configuration.
///
/// Returns `Ok(None)` when OTEL is disabled or the feature is not compiled.
pub fn build_metrics_exporter(
    config: &OtelConfig,
) -> Result<Option<OtelMetricsExporter>, OtelMetricsError> {
    if !config.enabled {
        return Ok(None);
    }

    #[cfg(not(feature = "otel-tracing"))]
    {
        let _ = config;
        Ok(None)
    }

    #[cfg(feature = "otel-tracing")]
    {
        use opentelemetry::metrics::MeterProvider as _;
        use opentelemetry::KeyValue;
        use opentelemetry_otlp::WithExportConfig;
        use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider, Temporality};
        use opentelemetry_sdk::resource::Resource;

        // Periodic reader spawns background tasks; ensure Tokio runtime exists.
        if tokio::runtime::Handle::try_current().is_err() {
            return Err(OtelMetricsError::MissingRuntime);
        }

        let exporter = opentelemetry_otlp::MetricExporter::builder()
            .with_tonic()
            .with_temporality(Temporality::Cumulative)
            .with_endpoint(config.endpoint.clone())
            .build()?;

        let reader = PeriodicReader::builder(exporter, opentelemetry_sdk::runtime::Tokio)
            .with_interval(config.export_interval)
            .build();

        let meter_provider = SdkMeterProvider::builder()
            .with_resource(Resource::new(vec![KeyValue::new(
                "service.name",
                config.service_name.clone(),
            )]))
            .with_reader(reader)
            .build();

        let meter = meter_provider.meter("keyrx");

        // Explicit microsecond buckets to satisfy latency histogram requirement.
        const LATENCY_BUCKETS: &[f64] = &[
            50.0,        // 50us
            100.0,       // 0.1ms
            250.0,       // 0.25ms
            500.0,       // 0.5ms
            1_000.0,     // 1ms
            2_500.0,     // 2.5ms
            5_000.0,     // 5ms
            10_000.0,    // 10ms
            25_000.0,    // 25ms
            50_000.0,    // 50ms
            100_000.0,   // 100ms
            250_000.0,   // 250ms
            500_000.0,   // 500ms
            1_000_000.0, // 1s
        ];

        let latency_histogram = meter
            .f64_histogram("keyrx.latency.us")
            .with_description("Operation latency in microseconds")
            .with_unit("us")
            .with_boundaries(LATENCY_BUCKETS.to_vec())
            .build();

        let operation_counter = meter
            .u64_counter("keyrx.operations.total")
            .with_description("Total recorded operations by type")
            .build();

        let memory_gauge = meter
            .u64_gauge("keyrx.memory.bytes")
            .with_description("Memory usage readings")
            .with_unit("By")
            .build();

        let profile_histogram = meter
            .f64_histogram("keyrx.profile.micros")
            .with_description("Profiled section timings in microseconds")
            .with_unit("us")
            .with_boundaries(LATENCY_BUCKETS.to_vec())
            .build();

        // Install globally so downstream callers can fetch a meter.
        opentelemetry::global::set_meter_provider(meter_provider.clone());

        Ok(Some(OtelMetricsExporter {
            meter_provider,
            latency_histogram,
            operation_counter,
            memory_gauge,
            profile_histogram,
            last_counts: std::sync::Mutex::new(std::collections::HashMap::new()),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn base_config(enabled: bool) -> OtelConfig {
        OtelConfig {
            enabled,
            endpoint: OtelConfig::DEFAULT_ENDPOINT.to_string(),
            service_name: OtelConfig::DEFAULT_SERVICE_NAME.to_string(),
            batch_size: OtelConfig::DEFAULT_BATCH_SIZE,
            export_interval: Duration::from_secs(OtelConfig::DEFAULT_EXPORT_INTERVAL_SECS),
        }
    }

    #[test]
    fn returns_none_when_disabled() {
        let config = base_config(false);
        let exporter = build_metrics_exporter(&config).expect("should not error when disabled");
        assert!(exporter.is_none());
    }

    #[cfg(feature = "otel-tracing")]
    #[test]
    fn errors_without_runtime_when_enabled() {
        let config = base_config(true);
        let err = build_metrics_exporter(&config).expect_err("runtime missing should error");
        assert!(matches!(err, OtelMetricsError::MissingRuntime));
    }
}
