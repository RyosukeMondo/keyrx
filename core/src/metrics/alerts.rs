//! Alerting for metrics thresholds.
//!
//! Provides configurable thresholds for latency, error rate, and memory usage
//! along with callback-based notifications when thresholds are exceeded. This
//! keeps alerting logic close to the metrics snapshots so dashboards and
//! exporters can react without polling external systems.

use super::snapshot::MetricsSnapshot;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

/// Alert severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AlertLevel {
    /// Warning threshold exceeded.
    Warning,
    /// Critical threshold exceeded.
    Critical,
}

/// Type of alert that was triggered.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AlertKind {
    /// p99 latency for an operation exceeded threshold.
    LatencyP99 { operation: String },
    /// Rolling error rate exceeded threshold.
    ErrorRate,
    /// Memory usage exceeded threshold.
    MemoryUsage,
    /// Potential memory leak detected by the monitor.
    MemoryLeak,
}

/// Alert event with contextual information.
#[derive(Debug, Clone)]
pub struct Alert {
    /// Timestamp in milliseconds when the alert was generated.
    pub timestamp_ms: u64,
    /// Severity of the alert.
    pub level: AlertLevel,
    /// What triggered the alert.
    pub kind: AlertKind,
    /// Observed metric value.
    pub actual: f64,
    /// Threshold that was exceeded.
    pub threshold: f64,
    /// Human-friendly message for logs/UX.
    pub message: String,
}

impl Alert {
    fn new(
        timestamp_ms: u64,
        level: AlertLevel,
        kind: AlertKind,
        actual: f64,
        threshold: f64,
        message: String,
    ) -> Self {
        Self {
            timestamp_ms,
            level,
            kind,
            actual,
            threshold,
            message,
        }
    }
}

/// Threshold configuration for alerting.
#[derive(Debug, Clone)]
pub struct AlertThresholds {
    /// Warning threshold for p99 latency (microseconds).
    pub latency_warn_p99: u64,
    /// Critical threshold for p99 latency (microseconds).
    pub latency_critical_p99: u64,
    /// Warning threshold for rolling error rate (errors per minute).
    pub error_rate_warn_per_minute: f64,
    /// Critical threshold for rolling error rate (errors per minute).
    pub error_rate_critical_per_minute: f64,
    /// Warning threshold for memory usage (bytes).
    pub memory_warn_bytes: usize,
    /// Critical threshold for memory usage (bytes).
    pub memory_critical_bytes: usize,
    /// Whether memory leak detection should emit alerts.
    pub leak_detection_enabled: bool,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            // Align defaults with the Flutter dashboard expectations.
            latency_warn_p99: 50,                     // 50µs warning
            latency_critical_p99: 100,                // 100µs critical
            error_rate_warn_per_minute: 10.0,         // 10 errors/min warn
            error_rate_critical_per_minute: 30.0,     // 30 errors/min critical
            memory_warn_bytes: 100 * 1024 * 1024,     // 100MB warn
            memory_critical_bytes: 500 * 1024 * 1024, // 500MB critical
            leak_detection_enabled: true,
        }
    }
}

/// Callback invoked when an alert is emitted.
pub type AlertCallback = Arc<dyn Fn(&Alert) + Send + Sync + 'static>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct AlertKey {
    level: AlertLevel,
    kind: AlertKind,
}

/// Alert manager that evaluates snapshots against thresholds and fires callbacks.
pub struct AlertManager {
    thresholds: RwLock<AlertThresholds>,
    callbacks: RwLock<Vec<AlertCallback>>,
    debounce: Duration,
    last_fired: Mutex<HashMap<AlertKey, u64>>,
}

impl AlertManager {
    /// Create a new alert manager with default debounce (5s) and thresholds.
    pub fn new(thresholds: AlertThresholds) -> Self {
        Self::with_debounce(thresholds, Duration::from_secs(5))
    }

    /// Create a new alert manager with custom debounce duration (primarily for tests).
    pub fn with_debounce(thresholds: AlertThresholds, debounce: Duration) -> Self {
        Self {
            thresholds: RwLock::new(thresholds),
            callbacks: RwLock::new(Vec::new()),
            debounce,
            last_fired: Mutex::new(HashMap::new()),
        }
    }

    /// Register a callback to be invoked when alerts fire.
    pub fn register_callback<F>(&self, callback: F)
    where
        F: Fn(&Alert) + Send + Sync + 'static,
    {
        if let Ok(mut guard) = self.callbacks.write() {
            guard.push(Arc::new(callback));
        }
    }

    /// Remove all registered callbacks.
    pub fn clear_callbacks(&self) {
        if let Ok(mut guard) = self.callbacks.write() {
            guard.clear();
        }
    }

    /// Update thresholds at runtime.
    pub fn update_thresholds(&self, thresholds: AlertThresholds) {
        if let Ok(mut guard) = self.thresholds.write() {
            *guard = thresholds;
        }
    }

    /// Get the currently configured thresholds.
    pub fn thresholds(&self) -> AlertThresholds {
        self.thresholds
            .read()
            .map(|guard| guard.clone())
            .unwrap_or_else(|_| AlertThresholds::default())
    }

    /// Evaluate a snapshot and emit alerts, returning the emitted list.
    pub fn evaluate_and_emit(&self, snapshot: &MetricsSnapshot) -> Vec<Alert> {
        let alerts = self.evaluate(snapshot);
        if alerts.is_empty() {
            return alerts;
        }

        let mut deduped = Vec::with_capacity(alerts.len());
        if let Ok(mut fired) = self.last_fired.lock() {
            for alert in alerts {
                let key = AlertKey {
                    level: alert.level,
                    kind: alert.kind.clone(),
                };
                let should_fire = fired
                    .get(&key)
                    .map(|last| {
                        snapshot.timestamp.saturating_sub(*last) >= self.debounce.as_millis() as u64
                    })
                    .unwrap_or(true);

                if should_fire {
                    fired.insert(key, snapshot.timestamp);
                    deduped.push(alert);
                }
            }
        }

        if let Ok(callbacks) = self.callbacks.read() {
            for alert in &deduped {
                for callback in callbacks.iter() {
                    callback(alert);
                }
            }
        }

        deduped
    }

    /// Evaluate thresholds without firing callbacks.
    pub fn evaluate(&self, snapshot: &MetricsSnapshot) -> Vec<Alert> {
        let thresholds = self.thresholds();
        let timestamp = snapshot.timestamp;
        let mut alerts = Vec::new();

        self.check_latency_alerts(snapshot, &thresholds, timestamp, &mut alerts);
        self.check_error_rate_alerts(snapshot, &thresholds, timestamp, &mut alerts);
        self.check_memory_alerts(snapshot, &thresholds, timestamp, &mut alerts);

        alerts
    }

    fn check_latency_alerts(
        &self,
        snapshot: &MetricsSnapshot,
        thresholds: &AlertThresholds,
        timestamp: u64,
        alerts: &mut Vec<Alert>,
    ) {
        for (operation, stats) in &snapshot.latencies {
            if stats.count == 0 {
                continue;
            }

            if stats.p99 >= thresholds.latency_critical_p99 {
                alerts.push(Alert::new(
                    timestamp,
                    AlertLevel::Critical,
                    AlertKind::LatencyP99 {
                        operation: operation.clone(),
                    },
                    stats.p99 as f64,
                    thresholds.latency_critical_p99 as f64,
                    format!(
                        "p99 latency for {operation} is {}µs (critical {}µs)",
                        stats.p99, thresholds.latency_critical_p99
                    ),
                ));
            } else if stats.p99 >= thresholds.latency_warn_p99 {
                alerts.push(Alert::new(
                    timestamp,
                    AlertLevel::Warning,
                    AlertKind::LatencyP99 {
                        operation: operation.clone(),
                    },
                    stats.p99 as f64,
                    thresholds.latency_warn_p99 as f64,
                    format!(
                        "p99 latency for {operation} is {}µs (warn {}µs)",
                        stats.p99, thresholds.latency_warn_p99
                    ),
                ));
            }
        }
    }

    fn check_error_rate_alerts(
        &self,
        snapshot: &MetricsSnapshot,
        thresholds: &AlertThresholds,
        timestamp: u64,
        alerts: &mut Vec<Alert>,
    ) {
        let rate = snapshot.errors.rate_per_minute;
        if rate >= thresholds.error_rate_critical_per_minute {
            alerts.push(Alert::new(
                timestamp,
                AlertLevel::Critical,
                AlertKind::ErrorRate,
                rate,
                thresholds.error_rate_critical_per_minute,
                format!(
                    "Error rate {rate:.1} errs/min exceeds critical {:.1}",
                    thresholds.error_rate_critical_per_minute
                ),
            ));
        } else if rate >= thresholds.error_rate_warn_per_minute {
            alerts.push(Alert::new(
                timestamp,
                AlertLevel::Warning,
                AlertKind::ErrorRate,
                rate,
                thresholds.error_rate_warn_per_minute,
                format!(
                    "Error rate {rate:.1} errs/min exceeds warning {:.1}",
                    thresholds.error_rate_warn_per_minute
                ),
            ));
        }
    }

    fn check_memory_alerts(
        &self,
        snapshot: &MetricsSnapshot,
        thresholds: &AlertThresholds,
        timestamp: u64,
        alerts: &mut Vec<Alert>,
    ) {
        let memory = snapshot.memory.current as f64;
        if memory >= thresholds.memory_critical_bytes as f64 {
            alerts.push(Alert::new(
                timestamp,
                AlertLevel::Critical,
                AlertKind::MemoryUsage,
                memory,
                thresholds.memory_critical_bytes as f64,
                format!(
                    "Memory usage {} bytes exceeds critical {} bytes",
                    snapshot.memory.current, thresholds.memory_critical_bytes
                ),
            ));
        } else if memory >= thresholds.memory_warn_bytes as f64 {
            alerts.push(Alert::new(
                timestamp,
                AlertLevel::Warning,
                AlertKind::MemoryUsage,
                memory,
                thresholds.memory_warn_bytes as f64,
                format!(
                    "Memory usage {} bytes exceeds warning {} bytes",
                    snapshot.memory.current, thresholds.memory_warn_bytes
                ),
            ));
        }

        if thresholds.leak_detection_enabled && snapshot.memory.has_potential_leak {
            alerts.push(Alert::new(
                timestamp,
                AlertLevel::Warning,
                AlertKind::MemoryLeak,
                1.0,
                1.0,
                "Potential memory leak detected from rolling samples".to_string(),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::{ErrorSnapshot, LatencyStats, MemorySnapshot};

    fn snapshot_with_latency(
        latency_p99: u64,
        error_rate: f64,
        memory: MemorySnapshot,
        timestamp: u64,
    ) -> MetricsSnapshot {
        let mut latencies = HashMap::new();
        latencies.insert(
            "event_process".to_string(),
            LatencyStats {
                count: 10,
                p50: latency_p99 / 2,
                p95: latency_p99 - 10,
                p99: latency_p99,
                mean: latency_p99 as f64 / 2.0,
                min: 10,
                max: latency_p99,
            },
        );

        MetricsSnapshot {
            timestamp,
            latencies,
            memory,
            errors: ErrorSnapshot {
                total: 10,
                rate_per_minute: error_rate,
                by_type: HashMap::new(),
            },
            profiles: HashMap::new(),
        }
    }

    #[test]
    fn emits_alerts_for_latency_error_rate_and_memory() {
        let manager =
            AlertManager::with_debounce(AlertThresholds::default(), Duration::from_millis(0));

        let snapshot = snapshot_with_latency(
            250,
            40.0,
            MemorySnapshot::new(600 * 1024 * 1024, 700 * 1024 * 1024, 512, 0, true),
            1_000,
        );

        let alerts = manager.evaluate_and_emit(&snapshot);
        assert!(alerts
            .iter()
            .any(|a| matches!(a.kind, AlertKind::LatencyP99 { .. })
                && a.level == AlertLevel::Critical));
        assert!(alerts
            .iter()
            .any(|a| matches!(a.kind, AlertKind::ErrorRate) && a.level == AlertLevel::Critical));
        assert!(alerts
            .iter()
            .any(|a| matches!(a.kind, AlertKind::MemoryUsage)));
        assert!(alerts
            .iter()
            .any(|a| matches!(a.kind, AlertKind::MemoryLeak)));
    }

    #[test]
    fn debounces_repeated_alerts() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let thresholds = AlertThresholds {
            latency_warn_p99: 100,
            latency_critical_p99: 200,
            error_rate_warn_per_minute: 1.0,
            error_rate_critical_per_minute: 5.0,
            memory_warn_bytes: 1024,
            memory_critical_bytes: 2048,
            leak_detection_enabled: false,
        };

        let manager = AlertManager::with_debounce(thresholds, Duration::from_millis(10));
        let fired = Arc::new(AtomicUsize::new(0));
        let fired_clone = Arc::clone(&fired);

        manager.register_callback(move |_| {
            fired_clone.fetch_add(1, Ordering::Relaxed);
        });

        let base_snapshot =
            snapshot_with_latency(150, 2.0, MemorySnapshot::new(512, 512, 0, 0, false), 1_000);

        manager.evaluate_and_emit(&base_snapshot);
        manager.evaluate_and_emit(&base_snapshot);

        assert_eq!(fired.load(Ordering::Relaxed), 2);

        let later_snapshot =
            snapshot_with_latency(150, 2.0, MemorySnapshot::new(512, 512, 0, 0, false), 1_020);

        manager.evaluate_and_emit(&later_snapshot);
        assert_eq!(fired.load(Ordering::Relaxed), 4);
    }
}
