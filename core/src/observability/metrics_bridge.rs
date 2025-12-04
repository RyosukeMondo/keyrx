//! Metrics bridge for FFI export.
//!
//! This module provides a bridge between the metrics collection system and FFI,
//! enabling Flutter UI to access real-time performance metrics through both
//! callback-based and polling-based mechanisms.

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Callback function for metrics updates.
///
/// This is invoked periodically when metrics are updated.
pub type MetricsCallback = extern "C" fn(*const MetricsSnapshotFfi);

/// Callback function for threshold violations.
///
/// This is invoked when metrics exceed configured thresholds.
pub type ThresholdCallback = extern "C" fn(*const ThresholdViolation);

/// Operations that can be profiled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operation {
    /// Event processing latency
    EventProcess,
    /// Rule matching latency
    RuleMatch,
    /// Action execution latency
    ActionExecute,
    /// Driver read latency
    DriverRead,
    /// Driver write latency
    DriverWrite,
}

/// Type of threshold violation.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViolationType {
    /// Latency exceeded warning threshold
    LatencyWarning = 0,
    /// Latency exceeded error threshold
    LatencyError = 1,
    /// Memory exceeded warning threshold
    MemoryWarning = 2,
    /// Memory exceeded error threshold
    MemoryError = 3,
}

/// Threshold violation event for FFI.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ThresholdViolation {
    /// Unix timestamp in milliseconds
    pub timestamp: u64,
    /// Type of violation
    pub violation_type: ViolationType,
    /// Actual value that triggered the violation
    pub actual_value: u64,
    /// Threshold that was exceeded
    pub threshold_value: u64,
}

/// Configurable thresholds for metrics.
#[derive(Debug, Clone)]
pub struct MetricsThresholds {
    /// Latency warning threshold (microseconds)
    pub latency_warn_micros: u64,
    /// Latency error threshold (microseconds)
    pub latency_error_micros: u64,
    /// Memory warning threshold (bytes)
    pub memory_warn_bytes: u64,
    /// Memory error threshold (bytes)
    pub memory_error_bytes: u64,
}

impl Default for MetricsThresholds {
    fn default() -> Self {
        Self {
            // Default latency thresholds based on tech.md requirement
            latency_warn_micros: 50,   // Warn at 50us
            latency_error_micros: 100, // Error at 100us (tech.md limit)
            // Default memory thresholds (100MB warn, 500MB error)
            memory_warn_bytes: 100 * 1024 * 1024,
            memory_error_bytes: 500 * 1024 * 1024,
        }
    }
}

/// Trait for metrics collection implementations.
///
/// This trait abstracts the metrics collection system, allowing for
/// different implementations (e.g., no-op, full profiling, test doubles).
pub trait MetricsCollector: Send + Sync {
    /// Record latency for an operation.
    fn record_latency(&self, operation: Operation, micros: u64);

    /// Record memory usage in bytes.
    fn record_memory(&self, bytes: usize);

    /// Get a snapshot of current metrics.
    fn snapshot(&self) -> MetricsSnapshot;

    /// Reset all collected metrics.
    fn reset(&self);
}

/// Snapshot of metrics at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    /// Unix timestamp in milliseconds
    pub timestamp: u64,
    /// Event latency 50th percentile (microseconds)
    pub event_latency_p50: u64,
    /// Event latency 95th percentile (microseconds)
    pub event_latency_p95: u64,
    /// Event latency 99th percentile (microseconds)
    pub event_latency_p99: u64,
    /// Total events processed
    pub events_processed: u64,
    /// Total errors encountered
    pub errors_count: u64,
    /// Current memory usage (bytes)
    pub memory_used: u64,
}

impl Default for MetricsSnapshot {
    fn default() -> Self {
        Self {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            event_latency_p50: 0,
            event_latency_p95: 0,
            event_latency_p99: 0,
            events_processed: 0,
            errors_count: 0,
            memory_used: 0,
        }
    }
}

/// FFI-compatible metrics snapshot.
///
/// This struct uses C-compatible types for FFI export.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MetricsSnapshotFfi {
    /// Unix timestamp in milliseconds
    pub timestamp: u64,
    /// Event latency 50th percentile (microseconds)
    pub event_latency_p50: u64,
    /// Event latency 95th percentile (microseconds)
    pub event_latency_p95: u64,
    /// Event latency 99th percentile (microseconds)
    pub event_latency_p99: u64,
    /// Total events processed
    pub events_processed: u64,
    /// Total errors encountered
    pub errors_count: u64,
    /// Current memory usage (bytes)
    pub memory_used: u64,
}

impl From<MetricsSnapshot> for MetricsSnapshotFfi {
    fn from(snapshot: MetricsSnapshot) -> Self {
        Self {
            timestamp: snapshot.timestamp,
            event_latency_p50: snapshot.event_latency_p50,
            event_latency_p95: snapshot.event_latency_p95,
            event_latency_p99: snapshot.event_latency_p99,
            events_processed: snapshot.events_processed,
            errors_count: snapshot.errors_count,
            memory_used: snapshot.memory_used,
        }
    }
}

/// Bridge for sending metrics to FFI.
///
/// This bridge connects the metrics collector to Flutter via FFI,
/// supporting both callback-based real-time updates and polling-based access.
pub struct MetricsBridge {
    /// The metrics collector to read from
    collector: Arc<dyn MetricsCollector>,
    /// Optional callback for metrics updates
    callback: RwLock<Option<MetricsCallback>>,
    /// Optional callback for threshold violations
    threshold_callback: RwLock<Option<ThresholdCallback>>,
    /// Configured thresholds
    thresholds: RwLock<MetricsThresholds>,
    /// Update interval for callbacks (if enabled)
    update_interval: RwLock<Duration>,
    /// Flag to control background updates
    updates_enabled: Mutex<bool>,
}

impl MetricsBridge {
    /// Create a new metrics bridge with the given collector.
    pub fn new(collector: Arc<dyn MetricsCollector>) -> Self {
        Self {
            collector,
            callback: RwLock::new(None),
            threshold_callback: RwLock::new(None),
            thresholds: RwLock::new(MetricsThresholds::default()),
            update_interval: RwLock::new(Duration::from_secs(1)),
            updates_enabled: Mutex::new(false),
        }
    }

    /// Set the callback for metrics updates.
    ///
    /// The callback will be invoked periodically when background updates are enabled.
    pub fn set_callback(&self, callback: MetricsCallback) {
        if let Ok(mut guard) = self.callback.write() {
            *guard = Some(callback);
        } else {
            tracing::error!("Failed to acquire callback lock");
        }
    }

    /// Clear the callback, disabling metrics updates.
    pub fn clear_callback(&self) {
        if let Ok(mut guard) = self.callback.write() {
            *guard = None;
        } else {
            tracing::error!("Failed to acquire callback lock");
        }
    }

    /// Set the callback for threshold violations.
    ///
    /// The callback will be invoked when metrics exceed configured thresholds.
    pub fn set_threshold_callback(&self, callback: ThresholdCallback) {
        if let Ok(mut guard) = self.threshold_callback.write() {
            *guard = Some(callback);
        } else {
            tracing::error!("Failed to acquire threshold callback lock");
        }
    }

    /// Clear the threshold callback.
    pub fn clear_threshold_callback(&self) {
        if let Ok(mut guard) = self.threshold_callback.write() {
            *guard = None;
        } else {
            tracing::error!("Failed to acquire threshold callback lock");
        }
    }

    /// Set threshold values.
    ///
    /// # Arguments
    /// * `thresholds` - New threshold configuration
    pub fn set_thresholds(&self, thresholds: MetricsThresholds) {
        if let Ok(mut guard) = self.thresholds.write() {
            *guard = thresholds;
        } else {
            tracing::error!("Failed to acquire thresholds lock");
        }
    }

    /// Get current threshold values.
    pub fn get_thresholds(&self) -> MetricsThresholds {
        if let Ok(guard) = self.thresholds.read() {
            guard.clone()
        } else {
            tracing::error!("Failed to acquire thresholds lock");
            MetricsThresholds::default()
        }
    }

    /// Check a snapshot for threshold violations and trigger callbacks.
    ///
    /// This is called automatically during metrics updates but can also
    /// be called manually for immediate checking.
    pub fn check_thresholds(&self, snapshot: &MetricsSnapshot) {
        let thresholds = if let Ok(guard) = self.thresholds.read() {
            guard.clone()
        } else {
            return;
        };

        let callback = if let Ok(guard) = self.threshold_callback.read() {
            *guard
        } else {
            return;
        };

        if let Some(cb) = callback {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;

            // Check latency thresholds (using p99 as representative)
            let latency = snapshot.event_latency_p99;
            if latency >= thresholds.latency_error_micros {
                let violation = ThresholdViolation {
                    timestamp,
                    violation_type: ViolationType::LatencyError,
                    actual_value: latency,
                    threshold_value: thresholds.latency_error_micros,
                };
                cb(&violation as *const ThresholdViolation);
                tracing::warn!(
                    service = "keyrx",
                    event = "threshold_violation",
                    component = "metrics_bridge",
                    violation_type = "latency_error",
                    actual = latency,
                    threshold = thresholds.latency_error_micros,
                    "Latency exceeded error threshold"
                );
            } else if latency >= thresholds.latency_warn_micros {
                let violation = ThresholdViolation {
                    timestamp,
                    violation_type: ViolationType::LatencyWarning,
                    actual_value: latency,
                    threshold_value: thresholds.latency_warn_micros,
                };
                cb(&violation as *const ThresholdViolation);
                tracing::warn!(
                    service = "keyrx",
                    event = "threshold_violation",
                    component = "metrics_bridge",
                    violation_type = "latency_warning",
                    actual = latency,
                    threshold = thresholds.latency_warn_micros,
                    "Latency exceeded warning threshold"
                );
            }

            // Check memory thresholds
            let memory = snapshot.memory_used;
            if memory >= thresholds.memory_error_bytes {
                let violation = ThresholdViolation {
                    timestamp,
                    violation_type: ViolationType::MemoryError,
                    actual_value: memory,
                    threshold_value: thresholds.memory_error_bytes,
                };
                cb(&violation as *const ThresholdViolation);
                tracing::warn!(
                    service = "keyrx",
                    event = "threshold_violation",
                    component = "metrics_bridge",
                    violation_type = "memory_error",
                    actual = memory,
                    threshold = thresholds.memory_error_bytes,
                    "Memory exceeded error threshold"
                );
            } else if memory >= thresholds.memory_warn_bytes {
                let violation = ThresholdViolation {
                    timestamp,
                    violation_type: ViolationType::MemoryWarning,
                    actual_value: memory,
                    threshold_value: thresholds.memory_warn_bytes,
                };
                cb(&violation as *const ThresholdViolation);
                tracing::warn!(
                    service = "keyrx",
                    event = "threshold_violation",
                    component = "metrics_bridge",
                    violation_type = "memory_warning",
                    actual = memory,
                    threshold = thresholds.memory_warn_bytes,
                    "Memory exceeded warning threshold"
                );
            }
        }
    }

    /// Set the update interval for callbacks.
    ///
    /// This only affects callback-based updates, not polling.
    pub fn set_interval(&self, interval: Duration) {
        if let Ok(mut guard) = self.update_interval.write() {
            *guard = interval;
        } else {
            tracing::error!("Failed to acquire interval lock");
        }
    }

    /// Get the current metrics snapshot.
    ///
    /// This can be called at any time for polling-based access.
    pub fn snapshot(&self) -> MetricsSnapshot {
        self.collector.snapshot()
    }

    /// Start background updates (if a callback is set).
    ///
    /// This spawns a background thread that periodically invokes the callback
    /// with updated metrics. The thread will run until `stop_updates` is called.
    ///
    /// # Note
    ///
    /// This is a placeholder for the actual implementation. In a real system,
    /// you would spawn a background thread here. For now, we just set a flag.
    pub fn start_updates(&self) {
        if let Ok(mut guard) = self.updates_enabled.lock() {
            *guard = true;
            tracing::debug!("Metrics bridge background updates enabled");
        } else {
            tracing::error!("Failed to acquire updates_enabled lock");
        }
        // TODO: Spawn background thread that periodically:
        // 1. Calls self.snapshot()
        // 2. Converts to MetricsSnapshotFfi
        // 3. Invokes callback if set
    }

    /// Stop background updates.
    ///
    /// This stops the background update thread if it's running.
    pub fn stop_updates(&self) {
        if let Ok(mut guard) = self.updates_enabled.lock() {
            *guard = false;
            tracing::debug!("Metrics bridge background updates disabled");
        } else {
            tracing::error!("Failed to acquire updates_enabled lock");
        }
    }

    /// Trigger an immediate callback with current metrics.
    ///
    /// This can be used for on-demand updates without waiting for the interval.
    /// This also checks thresholds and triggers threshold callbacks if needed.
    pub fn trigger_callback(&self) {
        let snapshot = self.snapshot();

        // Check thresholds first
        self.check_thresholds(&snapshot);

        // Then trigger the regular metrics callback if set
        if let Ok(guard) = self.callback.read() {
            if let Some(callback) = *guard {
                let ffi_snapshot = MetricsSnapshotFfi::from(snapshot);
                callback(&ffi_snapshot as *const MetricsSnapshotFfi);
            }
        } else {
            tracing::error!("Failed to acquire callback lock for trigger");
        }
    }
}

/// No-op metrics collector for production builds.
///
/// This implementation has zero overhead as all methods are no-ops.
pub struct NoOpMetricsCollector;

impl MetricsCollector for NoOpMetricsCollector {
    fn record_latency(&self, _operation: Operation, _micros: u64) {
        // No-op
    }

    fn record_memory(&self, _bytes: usize) {
        // No-op
    }

    fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot::default()
    }

    fn reset(&self) {
        // No-op
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_snapshot_default() {
        let snapshot = MetricsSnapshot::default();
        assert_eq!(snapshot.event_latency_p50, 0);
        assert_eq!(snapshot.event_latency_p95, 0);
        assert_eq!(snapshot.event_latency_p99, 0);
        assert_eq!(snapshot.events_processed, 0);
        assert_eq!(snapshot.errors_count, 0);
        assert_eq!(snapshot.memory_used, 0);
        assert!(snapshot.timestamp > 0);
    }

    #[test]
    fn test_ffi_conversion() {
        let snapshot = MetricsSnapshot {
            timestamp: 1234567890,
            event_latency_p50: 100,
            event_latency_p95: 500,
            event_latency_p99: 1000,
            events_processed: 42,
            errors_count: 3,
            memory_used: 1024 * 1024,
        };

        let ffi_snapshot = MetricsSnapshotFfi::from(snapshot.clone());
        assert_eq!(ffi_snapshot.timestamp, snapshot.timestamp);
        assert_eq!(ffi_snapshot.event_latency_p50, snapshot.event_latency_p50);
        assert_eq!(ffi_snapshot.event_latency_p95, snapshot.event_latency_p95);
        assert_eq!(ffi_snapshot.event_latency_p99, snapshot.event_latency_p99);
        assert_eq!(ffi_snapshot.events_processed, snapshot.events_processed);
        assert_eq!(ffi_snapshot.errors_count, snapshot.errors_count);
        assert_eq!(ffi_snapshot.memory_used, snapshot.memory_used);
    }

    #[test]
    fn test_noop_collector() {
        let collector = NoOpMetricsCollector;
        collector.record_latency(Operation::EventProcess, 100);
        collector.record_memory(1024);
        let snapshot = collector.snapshot();
        assert_eq!(snapshot.events_processed, 0);
        collector.reset();
    }

    #[test]
    fn test_metrics_bridge_creation() {
        let collector: Arc<dyn MetricsCollector> = Arc::new(NoOpMetricsCollector);
        let bridge = MetricsBridge::new(collector);
        let snapshot = bridge.snapshot();
        assert!(snapshot.timestamp > 0);
    }

    #[test]
    fn test_metrics_bridge_interval() {
        let collector: Arc<dyn MetricsCollector> = Arc::new(NoOpMetricsCollector);
        let bridge = MetricsBridge::new(collector);
        bridge.set_interval(Duration::from_millis(500));
        let guard = bridge.update_interval.read();
        if let Ok(g) = guard {
            assert_eq!(*g, Duration::from_millis(500));
        } else {
            panic!("Failed to acquire lock in test");
        }
    }

    #[test]
    fn test_metrics_bridge_updates() {
        let collector: Arc<dyn MetricsCollector> = Arc::new(NoOpMetricsCollector);
        let bridge = MetricsBridge::new(collector);

        {
            let guard = bridge.updates_enabled.lock();
            if let Ok(g) = guard {
                assert!(!*g);
            }
        }
        bridge.start_updates();
        {
            let guard = bridge.updates_enabled.lock();
            if let Ok(g) = guard {
                assert!(*g);
            }
        }
        bridge.stop_updates();
        {
            let guard = bridge.updates_enabled.lock();
            if let Ok(g) = guard {
                assert!(!*g);
            }
        }
    }
}
