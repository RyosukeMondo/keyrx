//! OTLP exporter for metrics snapshots.
//!
//! This module provides a thin wrapper around the OTEL metrics pipeline so
//! callers can push `MetricsSnapshot` data to an OTLP collector. It performs
//! basic configuration validation, supports batch export intervals, and
//! degrades to a no-op when OTEL is disabled or not compiled.

use super::snapshot::MetricsSnapshot;
use crate::observability::otel::{
    build_metrics_exporter, OtelConfig, OtelConfigError, OtelMetricsError, OtelMetricsExporter,
};
use std::fmt;
use std::time::Duration;

/// Configuration for exporting metrics to an OTLP collector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OtlpExporterConfig {
    /// Whether OTLP export is enabled.
    pub enabled: bool,
    /// OTLP endpoint, e.g. `http://localhost:4317`.
    pub endpoint: String,
    /// Logical service name used in OTEL resources.
    pub service_name: String,
    /// Maximum metrics queued before a flush.
    pub batch_size: usize,
    /// Interval between periodic exports.
    pub export_interval: Duration,
}

impl Default for OtlpExporterConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            endpoint: OtelConfig::DEFAULT_ENDPOINT.to_string(),
            service_name: OtelConfig::DEFAULT_SERVICE_NAME.to_string(),
            batch_size: OtelConfig::DEFAULT_BATCH_SIZE,
            export_interval: Duration::from_secs(OtelConfig::DEFAULT_EXPORT_INTERVAL_SECS),
        }
    }
}

impl OtlpExporterConfig {
    fn to_otel_config(&self) -> OtelConfig {
        OtelConfig {
            enabled: self.enabled,
            endpoint: self.endpoint.clone(),
            service_name: self.service_name.clone(),
            batch_size: self.batch_size,
            export_interval: self.export_interval,
        }
    }

    /// Validate the configuration before building an exporter.
    pub fn validate(&self) -> Result<(), OtelConfigError> {
        self.to_otel_config().validate()
    }
}

/// Errors building or using the OTLP exporter.
#[derive(Debug, thiserror::Error)]
pub enum OtlpExporterError {
    /// Invalid configuration supplied.
    #[error(transparent)]
    Config(#[from] OtelConfigError),

    /// Failed to construct the underlying OTEL exporter.
    #[error(transparent)]
    Exporter(#[from] OtelMetricsError),
}

/// OTLP exporter that records snapshots into OTEL instruments for batch export.
pub struct OtlpExporter {
    inner: OtelMetricsExporter,
}

impl fmt::Debug for OtlpExporter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OtlpExporter").finish()
    }
}

impl OtlpExporter {
    /// Export a snapshot into OTEL instruments.
    ///
    /// Export occurs asynchronously via the SDK's periodic reader configured
    /// when the exporter was built. When OTEL is not compiled this is a no-op.
    pub fn export(&self, snapshot: &MetricsSnapshot) {
        self.inner.record_snapshot(snapshot);
    }

    /// Flush and shut down the OTEL meter provider.
    ///
    /// Safe to call multiple times; no-ops when OTEL is not compiled.
    pub fn shutdown(&self) {
        self.inner.shutdown();
    }
}

/// Build an OTLP exporter using the provided configuration.
///
/// Returns `Ok(None)` when OTEL export is disabled or not compiled.
pub fn build_otlp_exporter(
    config: &OtlpExporterConfig,
) -> Result<Option<OtlpExporter>, OtlpExporterError> {
    config.validate()?;

    let otel_config = config.to_otel_config();
    let exporter = build_metrics_exporter(&otel_config)?;

    Ok(exporter.map(|inner| OtlpExporter { inner }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_exporter_returns_none() {
        let config = OtlpExporterConfig::default();
        let exporter = build_otlp_exporter(&config).expect("config should be valid");
        assert!(exporter.is_none());
    }

    #[test]
    fn rejects_invalid_endpoint() {
        let mut config = OtlpExporterConfig::default();
        config.enabled = true;
        config.endpoint = "localhost:4317".to_string(); // missing scheme

        let err = build_otlp_exporter(&config).expect_err("endpoint should be rejected");
        assert!(matches!(
            err,
            OtlpExporterError::Config(OtelConfigError::InvalidEndpoint(_))
        ));
    }
}
