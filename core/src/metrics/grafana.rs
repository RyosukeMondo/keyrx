//! Grafana dashboard template for KeyRx metrics.
//!
//! This module builds a ready-to-import Grafana dashboard JSON document that
//! targets the Prometheus metrics emitted by `PrometheusExporter`. The
//! dashboard includes panel definitions and query templates for latency,
//! throughput, memory, error rates, and profiling data so operators can
//! visualize the system quickly without hand-authoring a dashboard.

use serde::Serialize;
use serde_json::{json, Value};

const DEFAULT_TITLE: &str = "KeyRx Metrics (Grafana)";
const DEFAULT_REFRESH: &str = "10s";

/// Grafana dashboard builder that emits JSON compatible with Prometheus data sources.
#[derive(Debug, Clone)]
pub struct GrafanaDashboard {
    namespace: String,
    datasource_uid: String,
    title: String,
    refresh: String,
}

impl GrafanaDashboard {
    /// Create a dashboard bound to a Prometheus data source UID and metric namespace.
    pub fn new(namespace: impl Into<String>, datasource_uid: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            datasource_uid: datasource_uid.into(),
            title: DEFAULT_TITLE.to_string(),
            refresh: DEFAULT_REFRESH.to_string(),
        }
    }

    /// Override the dashboard title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Override the refresh interval (Grafana duration strings, e.g., `30s`, `1m`).
    pub fn with_refresh(mut self, refresh: impl Into<String>) -> Self {
        self.refresh = refresh.into();
        self
    }

    /// Panel definitions with query templates for the dashboard.
    pub fn panels(&self) -> Vec<GrafanaPanel> {
        let op_selector = r#"operation=~"$operation""#.to_string();
        let profile_selector = r#"profile=~"$profile""#.to_string();
        let error_selector = r#"error_type=~"$error_type""#.to_string();

        vec![
            GrafanaPanel::timeseries(
                "Latency percentiles",
                "P50/P95/P99 latency per operation.",
                GridPos::new(0, 0),
                vec![
                    PanelQuery::new(
                        format!("{}{{{}}}", self.metric("latency_p50_us"), op_selector),
                        'A',
                        Some("p50 {{operation}}".to_string()),
                    ),
                    PanelQuery::new(
                        format!("{}{{{}}}", self.metric("latency_p95_us"), op_selector),
                        'B',
                        Some("p95 {{operation}}".to_string()),
                    ),
                    PanelQuery::new(
                        format!("{}{{{}}}", self.metric("latency_p99_us"), op_selector),
                        'C',
                        Some("p99 {{operation}}".to_string()),
                    ),
                ],
                Some("us"),
            ),
            GrafanaPanel::timeseries(
                "Throughput (5m)",
                "Latency sample throughput per operation over a 5m window.",
                GridPos::new(12, 0),
                vec![PanelQuery::new(
                    format!(
                        "increase({}[5m])",
                        self.namespaced_selector(&op_selector, "latency_count")
                    ),
                    'A',
                    Some("samples {{operation}}".to_string()),
                )],
                Some("none"),
            ),
            GrafanaPanel::timeseries(
                "Memory usage",
                "Current, peak, and baseline memory for the process.",
                GridPos::new(0, 8),
                vec![PanelQuery::new(
                    format!(
                        "{}{{state=~\"current|peak|baseline\"}}",
                        self.metric("memory_bytes")
                    ),
                    'A',
                    Some("{{state}}".to_string()),
                )],
                Some("bytes"),
            ),
            GrafanaPanel::timeseries(
                "Errors by type (5m)",
                "Error totals by type (5m increase).",
                GridPos::new(12, 8),
                vec![PanelQuery::new(
                    format!(
                        "increase({}[5m])",
                        self.namespaced_selector(&error_selector, "errors_by_type_total")
                    ),
                    'A',
                    Some("{{error_type}}".to_string()),
                )],
                Some("none"),
            ),
            GrafanaPanel::timeseries(
                "Error rate per minute",
                "Rolling error rate captured by the collector.",
                GridPos::new(0, 16),
                vec![PanelQuery::new(
                    self.metric("errors_rate_per_minute"),
                    'A',
                    Some("errors/min".to_string()),
                )],
                Some("none"),
            ),
            GrafanaPanel::timeseries(
                "Profile average (us)",
                "Average execution time by profile point.",
                GridPos::new(12, 16),
                vec![PanelQuery::new(
                    format!(
                        "{}{{{}}}",
                        self.metric("profile_avg_micros"),
                        profile_selector
                    ),
                    'A',
                    Some("{{profile}}".to_string()),
                )],
                Some("microseconds"),
            ),
        ]
    }

    /// Render the dashboard JSON as a serde value.
    pub fn to_value(&self) -> Value {
        let datasource = self.datasource_ref();
        let panels: Vec<Value> = self
            .panels()
            .into_iter()
            .enumerate()
            .map(|(idx, panel)| panel.to_value(idx as u32 + 1, &datasource))
            .collect();

        json!({
            "uid": format!("keyrx-{}", self.namespace),
            "title": self.title,
            "refresh": self.refresh,
            "schemaVersion": 39,
            "version": 1,
            "timezone": "browser",
            "tags": ["keyrx", "otel", "metrics"],
            "time": { "from": "now-6h", "to": "now" },
            "templating": { "list": self.templating() },
            "panels": panels,
        })
    }

    /// Render pretty-printed Grafana dashboard JSON.
    pub fn to_pretty_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.to_value())
    }

    fn metric(&self, suffix: &str) -> String {
        format!("{}_{}", self.namespace, suffix)
    }

    fn namespaced_selector(&self, selector: &str, metric: &str) -> String {
        format!("{}{{{}}}", self.metric(metric), selector)
    }

    fn datasource_ref(&self) -> Value {
        json!({ "type": "prometheus", "uid": self.datasource_uid })
    }

    fn templating(&self) -> Vec<Value> {
        let make_label_variable = |name: &str, metric: &str| {
            let query = format!("label_values({}, {name})", self.metric(metric));
            json!({
                "name": name,
                "type": "query",
                "hide": 0,
                "multi": true,
                "includeAll": true,
                "allValue": ".*",
                "query": query,
                "definition": query,
                "datasource": self.datasource_ref(),
                "refresh": 1,
                "sort": 1,
                "current": { "text": "All", "value": "$__all" },
            })
        };

        vec![
            make_label_variable("operation", "latency_p95_us"),
            make_label_variable("profile", "profile_avg_micros"),
            make_label_variable("error_type", "errors_by_type_total"),
        ]
    }
}

/// Simple position representation for Grafana grid layout.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct GridPos {
    pub h: u16,
    pub w: u16,
    pub x: u16,
    pub y: u16,
}

impl GridPos {
    fn new(x: u16, y: u16) -> Self {
        Self { h: 8, w: 12, x, y }
    }
}

/// Panel definition used by the dashboard builder.
#[derive(Debug, Clone)]
pub struct GrafanaPanel {
    pub title: String,
    pub description: String,
    pub targets: Vec<PanelQuery>,
    pub panel_type: PanelType,
    pub grid_pos: GridPos,
    pub unit: Option<&'static str>,
}

impl GrafanaPanel {
    fn timeseries(
        title: impl Into<String>,
        description: impl Into<String>,
        grid_pos: GridPos,
        targets: Vec<PanelQuery>,
        unit: Option<&'static str>,
    ) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
            targets,
            panel_type: PanelType::Timeseries,
            grid_pos,
            unit,
        }
    }

    fn to_value(&self, id: u32, datasource: &Value) -> Value {
        let targets: Vec<Value> = self
            .targets
            .iter()
            .map(|target| target.to_value(datasource))
            .collect();

        json!({
            "id": id,
            "title": self.title,
            "description": self.description,
            "type": self.panel_type.as_str(),
            "gridPos": {
                "h": self.grid_pos.h,
                "w": self.grid_pos.w,
                "x": self.grid_pos.x,
                "y": self.grid_pos.y,
            },
            "targets": targets,
            "fieldConfig": {
                "defaults": {
                    "unit": self.unit.unwrap_or("short"),
                },
                "overrides": [],
            },
            "options": {
                "legend": { "displayMode": "list", "placement": "right" },
                "tooltip": { "mode": "single" },
            },
        })
    }
}

/// Query target for a panel.
#[derive(Debug, Clone)]
pub struct PanelQuery {
    pub expr: String,
    pub ref_id: char,
    pub legend: Option<String>,
}

impl PanelQuery {
    pub fn new(expr: impl Into<String>, ref_id: char, legend: Option<String>) -> Self {
        Self {
            expr: expr.into(),
            ref_id,
            legend,
        }
    }

    fn to_value(&self, datasource: &Value) -> Value {
        json!({
            "datasource": datasource,
            "expr": self.expr,
            "refId": self.ref_id.to_string(),
            "legendFormat": self.legend.clone().unwrap_or_default(),
            "range": true,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PanelType {
    Timeseries,
}

impl PanelType {
    fn as_str(self) -> &'static str {
        match self {
            PanelType::Timeseries => "timeseries",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_panels_with_namespace() {
        let dashboard = GrafanaDashboard::new("keyrx", "prometheus");
        let panels = dashboard.panels();

        assert!(!panels.is_empty(), "panels should be populated");
        assert!(panels.iter().any(|panel| panel
            .targets
            .iter()
            .any(|target| target.expr.contains("keyrx_latency_p95_us"))));
    }

    #[test]
    fn renders_dashboard_value() {
        let dashboard = GrafanaDashboard::new("keyrx", "prometheus");
        let value = dashboard.to_value();

        let templating = value
            .get("templating")
            .and_then(|v| v.get("list"))
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        assert_eq!(templating.len(), 3);

        let panels = value.get("panels").and_then(|v| v.as_array()).unwrap();
        assert_eq!(panels.len(), 6);
        assert!(serde_json::to_string(&value)
            .unwrap()
            .contains("keyrx_errors_rate_per_minute"));
    }

    #[test]
    fn produces_pretty_json() {
        let dashboard = GrafanaDashboard::new("keyrx", "prometheus").with_title("Custom");
        let json = dashboard.to_pretty_json().expect("JSON should render");
        assert!(json.contains("Custom"));
        assert!(json.contains("panels"));
    }
}
