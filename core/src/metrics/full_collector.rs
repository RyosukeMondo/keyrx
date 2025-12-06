//! Full metrics collector implementation.
//!
//! This module provides a complete metrics collection implementation that combines
//! latency histograms, memory monitoring, and profile points into a unified collector.
//!
//! # Design
//!
//! The `FullMetricsCollector` aggregates three specialized components:
//! - **Latency Histograms**: Per-operation percentile tracking using HDR histograms
//! - **Memory Monitor**: Memory usage tracking with leak detection
//! - **Profile Points**: Function-level timing with hot spot identification
//!
//! # Thread Safety
//!
//! All components are thread-safe and use either lock-free data structures (DashMap,
//! atomics) or bounded locks (Mutex on histogram). The overall overhead target is
//! < 1 microsecond per recording operation.
//!
//! # Memory Usage
//!
//! Memory usage is bounded:
//! - 5 operation histograms: ~5KB each = ~25KB
//! - Memory monitor: ~800 bytes (100 samples * 8 bytes)
//! - Profile points: ~64 bytes per unique profile point
//!
//! Total overhead for typical applications: < 100KB
//!
//! # Example
//!
//! ```ignore
//! use keyrx_core::metrics::{MetricsCollector, Operation};
//! use keyrx_core::metrics::full_collector::FullMetricsCollector;
//!
//! let collector = FullMetricsCollector::new();
//!
//! // Record operation latency
//! collector.record_latency(Operation::EventProcess, 150);
//!
//! // Record memory usage
//! collector.record_memory(1024 * 1024);
//!
//! // Profile a function
//! {
//!     let _guard = collector.start_profile("expensive_function");
//!     // ... code to profile ...
//! }
//!
//! // Get snapshot
//! let snapshot = collector.snapshot();
//! ```

use super::collector::{MetricsCollector, ProfileGuard};
use super::latency::LatencyHistogram;
use super::memory::MemoryMonitor;
use super::operation::Operation;
use super::profile::ProfilePoints;
use super::snapshot::{LatencyStats, MemorySnapshot, MetricsSnapshot};
use std::collections::HashMap;
use std::sync::Arc;

/// Default latency threshold for warnings (1 millisecond).
const DEFAULT_LATENCY_THRESHOLD_MICROS: u64 = 1000;

/// Full-featured metrics collector.
///
/// Combines latency histograms, memory monitoring, and profile points into
/// a unified collector that implements the `MetricsCollector` trait.
///
/// # Thread Safety
///
/// This type is `Send + Sync` and can be safely shared across threads using
/// `Arc`. All internal components use thread-safe data structures.
///
/// # Performance
///
/// - `record_latency`: < 1 microsecond overhead
/// - `record_memory`: < 100 nanoseconds overhead
/// - `record_profile`: < 1 microsecond overhead
///
/// # Example
///
/// ```ignore
/// use std::sync::Arc;
/// use keyrx_core::metrics::full_collector::FullMetricsCollector;
///
/// let collector = Arc::new(FullMetricsCollector::new());
///
/// // Share across threads
/// let c1 = Arc::clone(&collector);
/// std::thread::spawn(move || {
///     c1.record_memory(1024);
/// });
/// ```
pub struct FullMetricsCollector {
    /// Per-operation latency histograms.
    ///
    /// Each operation type gets its own histogram for independent tracking
    /// of latency percentiles and statistics.
    latency_histograms: HashMap<Operation, LatencyHistogram>,

    /// Memory usage monitor with leak detection.
    memory_monitor: MemoryMonitor,

    /// Function-level profile points for hot spot identification.
    profile_points: Arc<ProfilePoints>,
}

impl FullMetricsCollector {
    /// Create a new full metrics collector with default thresholds.
    ///
    /// Uses 1ms latency threshold for all operations.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let collector = FullMetricsCollector::new();
    /// ```
    pub fn new() -> Self {
        Self::with_threshold(DEFAULT_LATENCY_THRESHOLD_MICROS)
    }

    /// Create a new full metrics collector with custom latency threshold.
    ///
    /// # Arguments
    ///
    /// * `threshold_micros` - Latency threshold in microseconds for all operations
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Use 500us threshold for more sensitive alerting
    /// let collector = FullMetricsCollector::with_threshold(500);
    /// ```
    pub fn with_threshold(threshold_micros: u64) -> Self {
        // Create histograms for each operation type
        let mut histograms = HashMap::new();
        for operation in Operation::all() {
            histograms.insert(
                *operation,
                LatencyHistogram::with_name(operation.name(), threshold_micros),
            );
        }

        Self {
            latency_histograms: histograms,
            memory_monitor: MemoryMonitor::new(),
            profile_points: Arc::new(ProfilePoints::new()),
        }
    }

    /// Get the latency histogram for a specific operation.
    ///
    /// # Arguments
    ///
    /// * `operation` - The operation to get histogram for
    ///
    /// # Returns
    ///
    /// Reference to the latency histogram, or `None` if operation not found.
    pub fn get_latency_histogram(&self, operation: Operation) -> Option<&LatencyHistogram> {
        self.latency_histograms.get(&operation)
    }

    /// Get the memory monitor.
    ///
    /// Provides access to memory statistics and leak detection.
    pub fn get_memory_monitor(&self) -> &MemoryMonitor {
        &self.memory_monitor
    }

    /// Get the profile points tracker.
    ///
    /// Provides access to function-level profiling statistics and hot spots.
    pub fn get_profile_points(&self) -> Arc<ProfilePoints> {
        Arc::clone(&self.profile_points)
    }
}

impl Default for FullMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsCollector for FullMetricsCollector {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn record_latency(&self, operation: Operation, micros: u64) {
        if let Some(histogram) = self.latency_histograms.get(&operation) {
            histogram.record(micros);
        }
    }

    fn record_memory(&self, bytes: usize) {
        self.memory_monitor.record(bytes);
    }

    fn start_profile(&self, name: &'static str) -> ProfileGuard<'_> {
        ProfileGuard::new(self, name)
    }

    fn record_profile(&self, name: &'static str, micros: u64) {
        self.profile_points.record(name, micros);
    }

    fn snapshot(&self) -> MetricsSnapshot {
        // Collect latency statistics for each operation
        let mut latencies = HashMap::new();
        for (operation, histogram) in &self.latency_histograms {
            let stats = LatencyStats {
                count: histogram.count(),
                p50: histogram.percentile(50.0),
                p95: histogram.percentile(95.0),
                p99: histogram.percentile(99.0),
                mean: histogram.mean(),
                min: histogram.min(),
                max: histogram.max(),
            };
            latencies.insert(operation.name().to_string(), stats);
        }

        // Collect memory statistics
        let mem_stats = self.memory_monitor.stats();
        let memory = MemorySnapshot::new(
            mem_stats.current,
            mem_stats.peak,
            mem_stats.baseline,
            mem_stats.growth_from_baseline,
            self.memory_monitor.has_potential_leak(),
        );

        // Collect profile point statistics
        let mut profiles = HashMap::new();
        for (name, stats) in self.profile_points.all_stats() {
            profiles.insert(name.to_string(), stats.into());
        }

        MetricsSnapshot::new(latencies, memory, profiles)
    }

    fn reset(&self) {
        // Reset all histograms
        for histogram in self.latency_histograms.values() {
            histogram.reset();
        }

        // Reset memory monitor
        self.memory_monitor.reset();

        // Reset profile points
        self.profile_points.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_new_collector() {
        let collector = FullMetricsCollector::new();

        // Verify all operation histograms exist
        for operation in Operation::all() {
            assert!(collector.get_latency_histogram(*operation).is_some());
        }
    }

    #[test]
    fn test_with_threshold() {
        let collector = FullMetricsCollector::with_threshold(500);

        // Verify histograms exist
        assert!(collector
            .get_latency_histogram(Operation::EventProcess)
            .is_some());
    }

    #[test]
    fn test_record_latency() {
        let collector = FullMetricsCollector::new();

        collector.record_latency(Operation::EventProcess, 100);
        collector.record_latency(Operation::EventProcess, 200);
        collector.record_latency(Operation::EventProcess, 300);

        let histogram = collector
            .get_latency_histogram(Operation::EventProcess)
            .unwrap();

        assert_eq!(histogram.count(), 3);
        assert_eq!(histogram.min(), 100);
        assert_eq!(histogram.max(), 300);
    }

    #[test]
    fn test_record_memory() {
        let collector = FullMetricsCollector::new();

        collector.record_memory(1024);
        collector.record_memory(2048);

        let stats = collector.get_memory_monitor().stats();
        assert_eq!(stats.current, 2048);
        assert_eq!(stats.peak, 2048);
        assert_eq!(stats.baseline, 1024);
    }

    #[test]
    fn test_start_profile() {
        let collector = FullMetricsCollector::new();

        {
            let _guard = collector.start_profile("test_function");
            thread::sleep(std::time::Duration::from_micros(10));
        }

        let stats = collector.get_profile_points().get_stats("test_function");
        assert_eq!(stats.count, 1);
    }

    #[test]
    fn test_record_profile() {
        let collector = FullMetricsCollector::new();

        collector.record_profile("manual_profile", 150);
        collector.record_profile("manual_profile", 250);

        let stats = collector.get_profile_points().get_stats("manual_profile");
        assert_eq!(stats.count, 2);
        assert_eq!(stats.total_micros, 400);
    }

    #[test]
    fn test_snapshot() {
        let collector = FullMetricsCollector::new();

        let snapshot = collector.snapshot();
        assert!(snapshot.timestamp > 0);
    }

    #[test]
    fn test_reset() {
        let collector = FullMetricsCollector::new();

        // Record some data
        collector.record_latency(Operation::EventProcess, 100);
        collector.record_memory(1024);
        collector.record_profile("test", 50);

        // Reset
        collector.reset();

        // Verify everything is cleared
        let histogram = collector
            .get_latency_histogram(Operation::EventProcess)
            .unwrap();
        assert_eq!(histogram.count(), 0);

        let mem_stats = collector.get_memory_monitor().stats();
        assert_eq!(mem_stats.sample_count, 0);

        let prof_stats = collector.get_profile_points().get_stats("test");
        assert_eq!(prof_stats.count, 0);
    }

    #[test]
    fn test_multiple_operations() {
        let collector = FullMetricsCollector::new();

        // Record different operations
        collector.record_latency(Operation::EventProcess, 100);
        collector.record_latency(Operation::RuleMatch, 50);
        collector.record_latency(Operation::ActionExecute, 200);

        // Verify each has independent tracking
        let event_hist = collector
            .get_latency_histogram(Operation::EventProcess)
            .unwrap();
        let rule_hist = collector
            .get_latency_histogram(Operation::RuleMatch)
            .unwrap();
        let action_hist = collector
            .get_latency_histogram(Operation::ActionExecute)
            .unwrap();

        assert_eq!(event_hist.count(), 1);
        assert_eq!(rule_hist.count(), 1);
        assert_eq!(action_hist.count(), 1);
    }

    #[test]
    fn test_thread_safety() {
        let collector = Arc::new(FullMetricsCollector::new());
        let mut handles = vec![];

        // Spawn multiple threads recording different metrics
        for i in 0..4 {
            let c = Arc::clone(&collector);
            let handle = thread::spawn(move || {
                for j in 0..100 {
                    c.record_latency(Operation::EventProcess, (i * 100 + j) as u64);
                    c.record_memory(1024 * (i * 100 + j));
                    c.record_profile("concurrent_test", (i * 10 + j) as u64);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().expect("Thread panicked");
        }

        // Verify all data was recorded
        let histogram = collector
            .get_latency_histogram(Operation::EventProcess)
            .unwrap();
        assert_eq!(histogram.count(), 400);

        let mem_stats = collector.get_memory_monitor().stats();
        assert_eq!(mem_stats.sample_count, 400);

        let prof_stats = collector.get_profile_points().get_stats("concurrent_test");
        assert_eq!(prof_stats.count, 400);
    }

    #[test]
    fn test_get_components() {
        let collector = FullMetricsCollector::new();

        // Test getting components
        let _monitor = collector.get_memory_monitor();
        let _points = collector.get_profile_points();
        let _histogram = collector.get_latency_histogram(Operation::EventProcess);

        // All should be accessible
        assert!(collector
            .get_latency_histogram(Operation::EventProcess)
            .is_some());
    }

    #[test]
    fn test_default() {
        let collector = FullMetricsCollector::default();

        collector.record_latency(Operation::EventProcess, 100);
        let histogram = collector
            .get_latency_histogram(Operation::EventProcess)
            .unwrap();

        assert_eq!(histogram.count(), 1);
    }

    #[test]
    fn test_latency_percentiles() {
        let collector = FullMetricsCollector::new();

        // Record 100 samples
        for i in 1..=100 {
            collector.record_latency(Operation::EventProcess, i * 10);
        }

        let histogram = collector
            .get_latency_histogram(Operation::EventProcess)
            .unwrap();

        assert_eq!(histogram.count(), 100);

        let p50 = histogram.percentile(50.0);
        let p95 = histogram.percentile(95.0);
        let p99 = histogram.percentile(99.0);

        assert!(p50 >= 400 && p50 <= 600, "p50={}", p50);
        assert!(p95 >= 900 && p95 <= 1000, "p95={}", p95);
        assert!(p99 >= 950 && p99 <= 1000, "p99={}", p99);
    }

    #[test]
    fn test_memory_leak_detection() {
        let collector = FullMetricsCollector::new();

        // Record monotonically increasing memory
        for i in 0..30 {
            let mem = 1024 * 1024 + (i * 10 * 1024); // Growing by 10KB each
            collector.record_memory(mem);
        }

        // Check for potential leak
        let has_leak = collector.get_memory_monitor().has_potential_leak();
        assert!(has_leak);
    }

    #[test]
    fn test_profile_hot_spots() {
        let collector = FullMetricsCollector::new();

        // Record different functions with different times
        for _ in 0..10 {
            collector.record_profile("slow_function", 1000);
        }

        for _ in 0..5 {
            collector.record_profile("medium_function", 500);
        }

        collector.record_profile("fast_function", 100);

        let hot_spots = collector.get_profile_points().hot_spots(2);

        assert_eq!(hot_spots.len(), 2);
        assert_eq!(hot_spots[0].0, "slow_function");
        assert_eq!(hot_spots[0].1.total_micros, 10000);
    }
}
