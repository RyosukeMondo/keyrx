//! OpenTelemetry tracing layer wiring.
//!
//! This module creates a batching OTLP tracer and exposes it as a `tracing`
//! layer that can be composed with existing subscribers. When OTEL is disabled
//! or the `otel-tracing` feature is not compiled, the builder returns `None`
//! so callers can skip attaching the layer without special-casing.

use super::OtelConfig;
use tracing_subscriber::layer::Layer;
use tracing_subscriber::Registry;

/// Boxed OTEL layer type to keep subscriber composition ergonomic.
pub type BoxedOtelLayer = Box<dyn Layer<Registry> + Send + Sync + 'static>;

/// Errors that can occur while building the OTEL tracing layer.
#[derive(Debug, thiserror::Error)]
pub enum OtelLayerError {
    #[cfg(feature = "otel-tracing")]
    #[error("Failed to install OTEL tracer: {0}")]
    Install(#[from] opentelemetry::trace::TraceError),

    #[error("Tokio runtime not available for OTEL batch exporter")]
    MissingRuntime,
}

/// Build an OTEL tracing layer using the provided configuration.
///
/// Returns `Ok(None)` when OTEL is disabled or the `otel-tracing` feature is
/// not compiled. Callers can attach the returned layer to their subscriber:
///
/// ```
/// # use keyrx_core::observability::otel::{build_otel_layer, OtelConfig};
/// # use tracing_subscriber::layer::SubscriberExt;
/// # use tracing_subscriber::util::SubscriberInitExt;
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = OtelConfig {
///     enabled: true,
///     endpoint: OtelConfig::DEFAULT_ENDPOINT.to_string(),
///     service_name: OtelConfig::DEFAULT_SERVICE_NAME.to_string(),
///     batch_size: OtelConfig::DEFAULT_BATCH_SIZE,
///     export_interval: std::time::Duration::from_secs(
///         OtelConfig::DEFAULT_EXPORT_INTERVAL_SECS,
///     ),
/// };
///
/// if let Some(layer) = build_otel_layer(&config)? {
///     tracing_subscriber::registry().with(layer).try_init()?;
/// }
/// # Ok(()) }
/// ```
pub fn build_otel_layer(config: &OtelConfig) -> Result<Option<BoxedOtelLayer>, OtelLayerError> {
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
        use opentelemetry::global;
        use opentelemetry::trace::TracerProvider as _;
        use opentelemetry::KeyValue;
        use opentelemetry_otlp::WithExportConfig;
        use opentelemetry_sdk::resource::Resource;
        use opentelemetry_sdk::trace::{BatchConfigBuilder, BatchSpanProcessor, TracerProvider};
        use tracing_opentelemetry::OpenTelemetryLayer;

        // Batch exporter requires an active Tokio runtime to spawn workers.
        if tokio::runtime::Handle::try_current().is_err() {
            return Err(OtelLayerError::MissingRuntime);
        }

        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint(config.endpoint.clone())
            .build()?;

        let batch_config = BatchConfigBuilder::default()
            .with_max_queue_size(config.batch_size.saturating_mul(2))
            .with_max_export_batch_size(config.batch_size)
            .with_scheduled_delay(config.export_interval)
            .with_max_export_timeout(config.export_interval)
            .build();

        let tracer_provider = TracerProvider::builder()
            .with_span_processor(
                BatchSpanProcessor::builder(exporter, opentelemetry_sdk::runtime::Tokio)
                    .with_batch_config(batch_config)
                    .build(),
            )
            .with_resource(Resource::new(vec![KeyValue::new(
                "service.name",
                config.service_name.clone(),
            )]))
            .build();

        let tracer = tracer_provider.tracer(config.service_name.clone());
        // Install globally so shutdown can flush outstanding spans.
        global::set_tracer_provider(tracer_provider);

        let layer: OpenTelemetryLayer<Registry, _> =
            tracing_opentelemetry::layer().with_tracer(tracer);

        Ok(Some(Box::new(layer)))
    }
}

/// Flush spans and shut down the tracer provider if OTEL is enabled.
pub fn shutdown_tracer_provider() {
    #[cfg(feature = "otel-tracing")]
    opentelemetry::global::shutdown_tracer_provider();
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
        let layer = build_otel_layer(&config).expect("should not error when disabled");
        assert!(layer.is_none());
    }

    #[cfg(feature = "otel-tracing")]
    #[tokio::test(flavor = "current_thread")]
    async fn builds_layer_when_enabled() {
        let mut config = base_config(true);
        config.export_interval = Duration::from_millis(50);
        let layer = build_otel_layer(&config).expect("layer should build");
        assert!(layer.is_some());
    }
}
