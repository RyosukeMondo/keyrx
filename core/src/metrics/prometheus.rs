//! Prometheus exposition for metrics snapshots.
//!
//! This module converts `MetricsSnapshot` data into the Prometheus text
//! exposition format so it can be returned from a `/metrics` HTTP endpoint.
//! No HTTP server is included here; callers supply the transport and simply
//! write the rendered body with the accompanying content type.

use super::snapshot::MetricsSnapshot;
use std::collections::BTreeMap;
use std::fmt::Write as FmtWrite;

/// Content type for Prometheus text exposition.
pub const CONTENT_TYPE: &str = "text/plain; version=0.0.4";

/// Renderer that exports `MetricsSnapshot` values as Prometheus text.
#[derive(Debug, Clone)]
pub struct PrometheusExporter {
    namespace: String,
    default_labels: BTreeMap<String, String>,
}

impl PrometheusExporter {
    /// Create a new exporter with an optional namespace and default labels.
    ///
    /// The namespace prefixes every metric name (e.g., `keyrx_latency_count`).
    /// Default labels are applied to every sample and may be overridden by
    /// per-sample labels with the same key.
    pub fn new(
        namespace: impl Into<String>,
        default_labels: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        let mut labels = BTreeMap::new();
        for (k, v) in default_labels {
            labels.insert(k.into(), v.into());
        }

        Self {
            namespace: namespace.into(),
            default_labels: labels,
        }
    }

    /// Render the provided snapshot to Prometheus text exposition.
    ///
    /// The returned string can be served directly from an HTTP handler.
    pub fn render(&self, snapshot: &MetricsSnapshot) -> String {
        let mut buf = String::new();
        self.render_latency_metrics(&mut buf, snapshot);
        self.render_memory_metrics(&mut buf, snapshot);
        self.render_error_metrics(&mut buf, snapshot);
        self.render_profile_metrics(&mut buf, snapshot);
        buf
    }

    fn render_latency_metrics(&self, buf: &mut String, snapshot: &MetricsSnapshot) {
        let count_name = format!("{}_latency_count", self.namespace);
        let p50_name = format!("{}_latency_p50_us", self.namespace);
        let p95_name = format!("{}_latency_p95_us", self.namespace);
        let p99_name = format!("{}_latency_p99_us", self.namespace);
        let mean_name = format!("{}_latency_mean_us", self.namespace);
        let min_name = format!("{}_latency_min_us", self.namespace);
        let max_name = format!("{}_latency_max_us", self.namespace);

        self.write_header(
            buf,
            &count_name,
            "counter",
            "Total latency samples recorded by operation",
        );
        self.write_header(
            buf,
            &p50_name,
            "gauge",
            "50th percentile latency in microseconds by operation",
        );
        self.write_header(
            buf,
            &p95_name,
            "gauge",
            "95th percentile latency in microseconds by operation",
        );
        self.write_header(
            buf,
            &p99_name,
            "gauge",
            "99th percentile latency in microseconds by operation",
        );
        self.write_header(
            buf,
            &mean_name,
            "gauge",
            "Mean latency in microseconds by operation",
        );
        self.write_header(
            buf,
            &min_name,
            "gauge",
            "Minimum latency in microseconds by operation",
        );
        self.write_header(
            buf,
            &max_name,
            "gauge",
            "Maximum latency in microseconds by operation",
        );

        for (operation, stats) in &snapshot.latencies {
            let labels = vec![("operation".to_string(), operation.clone())];
            self.write_sample(buf, &count_name, &labels, stats.count);
            self.write_sample(buf, &p50_name, &labels, stats.p50);
            self.write_sample(buf, &p95_name, &labels, stats.p95);
            self.write_sample(buf, &p99_name, &labels, stats.p99);
            self.write_sample(buf, &mean_name, &labels, stats.mean);
            self.write_sample(buf, &min_name, &labels, stats.min);
            self.write_sample(buf, &max_name, &labels, stats.max);
        }

        if !snapshot.latencies.is_empty() {
            buf.push('\n');
        }
    }

    fn render_memory_metrics(&self, buf: &mut String, snapshot: &MetricsSnapshot) {
        let memory_name = format!("{}_memory_bytes", self.namespace);
        let leak_name = format!("{}_memory_leak_detected", self.namespace);

        self.write_header(buf, &memory_name, "gauge", "Memory usage in bytes");
        self.write_header(
            buf,
            &leak_name,
            "gauge",
            "Potential leak indicator (1 when suspected, otherwise 0)",
        );

        self.write_sample(
            buf,
            &memory_name,
            &[("state".to_string(), "current".to_string())],
            snapshot.memory.current as u64,
        );
        self.write_sample(
            buf,
            &memory_name,
            &[("state".to_string(), "peak".to_string())],
            snapshot.memory.peak as u64,
        );
        self.write_sample(
            buf,
            &memory_name,
            &[("state".to_string(), "baseline".to_string())],
            snapshot.memory.baseline as u64,
        );
        self.write_sample(
            buf,
            &memory_name,
            &[("state".to_string(), "growth".to_string())],
            snapshot.memory.growth as u64,
        );
        self.write_sample(
            buf,
            &leak_name,
            &[],
            if snapshot.memory.has_potential_leak {
                1u64
            } else {
                0u64
            },
        );

        buf.push('\n');
    }

    fn render_error_metrics(&self, buf: &mut String, snapshot: &MetricsSnapshot) {
        let total_name = format!("{}_errors_total", self.namespace);
        let by_type_name = format!("{}_errors_by_type_total", self.namespace);
        let rate_name = format!("{}_errors_rate_per_minute", self.namespace);

        self.write_header(buf, &total_name, "counter", "Total errors recorded");
        self.write_header(
            buf,
            &by_type_name,
            "counter",
            "Total errors recorded by error type",
        );
        self.write_header(
            buf,
            &rate_name,
            "gauge",
            "Rolling errors per minute over the last window",
        );

        self.write_sample(buf, &total_name, &[], snapshot.errors.total);
        self.write_sample(buf, &rate_name, &[], snapshot.errors.rate_per_minute);

        for (error_type, count) in &snapshot.errors.by_type {
            let labels = vec![("error_type".to_string(), error_type.clone())];
            self.write_sample(buf, &by_type_name, &labels, *count);
        }

        buf.push('\n');
    }

    fn render_profile_metrics(&self, buf: &mut String, snapshot: &MetricsSnapshot) {
        let count_name = format!("{}_profile_count_total", self.namespace);
        let avg_name = format!("{}_profile_avg_micros", self.namespace);
        let max_name = format!("{}_profile_max_micros", self.namespace);
        let min_name = format!("{}_profile_min_micros", self.namespace);
        let total_name = format!("{}_profile_total_micros", self.namespace);

        self.write_header(
            buf,
            &count_name,
            "counter",
            "Total times a profile point executed",
        );
        self.write_header(
            buf,
            &avg_name,
            "gauge",
            "Average execution time in microseconds for a profile point",
        );
        self.write_header(
            buf,
            &max_name,
            "gauge",
            "Maximum execution time in microseconds for a profile point",
        );
        self.write_header(
            buf,
            &min_name,
            "gauge",
            "Minimum execution time in microseconds for a profile point",
        );
        self.write_header(
            buf,
            &total_name,
            "counter",
            "Total accumulated execution time in microseconds for a profile point",
        );

        for (profile, stats) in &snapshot.profiles {
            let labels = vec![("profile".to_string(), profile.clone())];
            self.write_sample(buf, &count_name, &labels, stats.count);
            self.write_sample(buf, &avg_name, &labels, stats.avg_micros);
            self.write_sample(buf, &max_name, &labels, stats.max_micros);
            self.write_sample(buf, &min_name, &labels, stats.min_micros);
            self.write_sample(buf, &total_name, &labels, stats.total_micros);
        }
    }

    fn write_header(&self, buf: &mut String, name: &str, metric_type: &str, help: &str) {
        let _ = writeln!(buf, "# HELP {name} {help}");
        let _ = writeln!(buf, "# TYPE {name} {metric_type}");
    }

    fn write_sample<V: Into<PrometheusValue>>(
        &self,
        buf: &mut String,
        name: &str,
        labels: &[(String, String)],
        value: V,
    ) {
        let mut merged = self.default_labels.clone();
        for (k, v) in labels {
            merged.insert(k.clone(), v.clone());
        }

        let label_text = if merged.is_empty() {
            String::new()
        } else {
            let mut text = String::from("{");
            for (idx, (k, v)) in merged.into_iter().enumerate() {
                if idx > 0 {
                    text.push(',');
                }
                let _ = write!(text, r#"{k}="{}""#, escape_label_value(&v));
            }
            text.push('}');
            text
        };

        let rendered: String = value.into().into();
        let _ = writeln!(buf, "{name}{label_text} {rendered}");
    }
}

/// Prometheus sample value representation that preserves integer formatting.
enum PrometheusValue {
    Int(u64),
    Float(f64),
}

impl From<u64> for PrometheusValue {
    fn from(value: u64) -> Self {
        PrometheusValue::Int(value)
    }
}

impl From<usize> for PrometheusValue {
    fn from(value: usize) -> Self {
        PrometheusValue::Int(value as u64)
    }
}

impl From<i64> for PrometheusValue {
    fn from(value: i64) -> Self {
        PrometheusValue::Int(value as u64)
    }
}

impl From<f64> for PrometheusValue {
    fn from(value: f64) -> Self {
        PrometheusValue::Float(value)
    }
}

impl From<PrometheusValue> for String {
    fn from(value: PrometheusValue) -> Self {
        match value {
            PrometheusValue::Int(v) => v.to_string(),
            PrometheusValue::Float(v) => format!("{v}"),
        }
    }
}

fn escape_label_value(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '"' => escaped.push_str("\\\""),
            other => escaped.push(other),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::errors::ErrorSnapshot;
    use crate::metrics::snapshot::{LatencyStats, MemorySnapshot, ProfileSnapshot};
    use std::collections::HashMap;

    fn sample_snapshot() -> MetricsSnapshot {
        let mut latencies = HashMap::new();
        latencies.insert(
            "event_process".to_string(),
            LatencyStats {
                count: 3,
                p50: 10,
                p95: 20,
                p99: 30,
                mean: 15.0,
                min: 5,
                max: 50,
            },
        );

        let mut profiles = HashMap::new();
        profiles.insert(
            "dispatch".to_string(),
            ProfileSnapshot {
                count: 4,
                total_micros: 80,
                avg_micros: 20,
                min_micros: 10,
                max_micros: 30,
            },
        );

        let mut error_types = HashMap::new();
        error_types.insert("io".to_string(), 2);

        MetricsSnapshot::new(
            latencies,
            MemorySnapshot::new(100, 150, 80, 20, false),
            ErrorSnapshot {
                total: 2,
                rate_per_minute: 1.5,
                by_type: error_types,
            },
            profiles,
        )
    }

    #[test]
    fn renders_metrics_with_namespace_and_labels() {
        let exporter = PrometheusExporter::new("keyrx", [("service", "test")]);
        let output = exporter.render(&sample_snapshot());

        assert!(output.contains("# TYPE keyrx_latency_count counter"));
        assert!(
            output.contains(r#"keyrx_latency_p95_us{operation="event_process",service="test"} 20"#)
        );
        let memory_line = match output
            .lines()
            .find(|line| line.contains("keyrx_memory_bytes") && line.contains("current"))
        {
            Some(line) => line,
            None => panic!("memory gauge not rendered"),
        };
        assert!(memory_line.contains(r#"state="current""#));
        assert!(memory_line.contains(r#"service="test""#));
        assert!(memory_line.ends_with("100"));
        assert!(output.contains(r#"keyrx_errors_total{service="test"} 2"#));
        assert!(
            output.contains(r#"keyrx_profile_total_micros{profile="dispatch",service="test"} 80"#)
        );
    }

    #[test]
    fn escapes_label_values() {
        let mut snapshot = sample_snapshot();
        snapshot
            .errors
            .by_type
            .insert("io\"path\nunstable".to_string(), 1);

        let exporter = PrometheusExporter::new("keyrx", std::iter::empty::<(String, String)>());
        let output = exporter.render(&snapshot);

        assert!(
            output.contains(r#"io\"path\nunstable""#),
            "label value should be escaped"
        );
    }

    #[test]
    fn exposes_content_type() {
        assert_eq!(CONTENT_TYPE, "text/plain; version=0.0.4");
    }
}
