//! Latency histogram for percentile tracking.
//!
//! This module provides a thread-safe, bounded-memory latency histogram using
//! hdrhistogram for accurate percentile calculation with O(1) recording and
//! O(log n) percentile queries.
//!
//! # Design
//!
//! - Uses HDR Histogram for memory-efficient storage of latency data
//! - Thread-safe through `Mutex` (acceptable for non-hot-path collection)
//! - Bounded memory: histogram has fixed size regardless of sample count
//! - Configurable thresholds for latency warnings
//!
//! # Threshold Violations
//!
//! When latencies exceed configured thresholds, warnings are logged via the
//! tracing infrastructure. This helps identify performance regressions.

use hdrhistogram::Histogram;
use std::sync::Mutex;
use tracing::warn;

/// Latency histogram for percentile tracking.
///
/// Records latency samples and provides percentile queries (p50, p95, p99).
/// Uses HDR Histogram for bounded memory and accurate percentile calculation.
///
/// # Thread Safety
///
/// This type is `Send + Sync` and can be shared across threads. Recording
/// operations take a lock, but the overhead is acceptable for metrics
/// collection (< 1 microsecond target).
///
/// # Memory
///
/// Memory usage is bounded regardless of sample count. The histogram uses
/// a logarithmic bucketing scheme that provides 3 significant digits of
/// precision while using minimal memory.
///
/// # Example
///
/// ```ignore
/// use keyrx_core::metrics::latency::LatencyHistogram;
///
/// let histogram = LatencyHistogram::new(1000); // 1ms threshold
///
/// // Record some latencies
/// histogram.record(150); // 150 microseconds
/// histogram.record(2500); // 2.5 milliseconds - exceeds threshold!
///
/// // Query percentiles
/// let p95 = histogram.percentile(95.0);
/// let p99 = histogram.percentile(99.0);
/// ```
pub struct LatencyHistogram {
    /// The underlying HDR histogram.
    ///
    /// Wrapped in Mutex for thread-safe access. The histogram tracks values
    /// from 1 microsecond to 1 hour (3,600,000,000 microseconds) with 3
    /// significant digits of precision.
    histogram: Mutex<Histogram<u64>>,

    /// Latency threshold in microseconds.
    ///
    /// If a recorded latency exceeds this threshold, a warning is logged
    /// via tracing. Set to 0 to disable threshold warnings.
    threshold_micros: u64,

    /// Name of this histogram for logging purposes.
    ///
    /// Used in warning messages to identify which operation exceeded thresholds.
    name: &'static str,
}

impl LatencyHistogram {
    /// Create a new latency histogram with threshold warnings.
    ///
    /// # Arguments
    ///
    /// * `threshold_micros` - Latency threshold in microseconds. Values exceeding
    ///   this will trigger warnings. Set to 0 to disable warnings.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Create histogram with 1ms threshold
    /// let histogram = LatencyHistogram::new(1000);
    /// ```
    pub fn new(threshold_micros: u64) -> Self {
        Self::with_name("unnamed", threshold_micros)
    }

    /// Create a new latency histogram with a name and threshold.
    ///
    /// # Arguments
    ///
    /// * `name` - Static string identifier for this histogram (used in logs)
    /// * `threshold_micros` - Latency threshold in microseconds
    ///
    /// # Example
    ///
    /// ```ignore
    /// let histogram = LatencyHistogram::with_name("event_process", 1000);
    /// ```
    #[allow(clippy::expect_used)] // Histogram creation with valid bounds cannot fail
    pub fn with_name(name: &'static str, threshold_micros: u64) -> Self {
        // Create histogram for 1us to 1 hour range with 3 significant digits
        // This gives us ~1% precision across the entire range while using
        // minimal memory (few KB).
        // The bounds are statically known to be valid, so expect is acceptable here.
        let histogram = Histogram::<u64>::new_with_bounds(1, 3_600_000_000, 3)
            .expect("Failed to create histogram with valid bounds");

        Self {
            histogram: Mutex::new(histogram),
            threshold_micros,
            name,
        }
    }

    /// Record a latency sample in microseconds.
    ///
    /// If the latency exceeds the configured threshold, a warning is logged.
    ///
    /// # Arguments
    ///
    /// * `micros` - Latency value in microseconds
    ///
    /// # Performance
    ///
    /// This operation takes a lock on the histogram but completes in < 1us
    /// on modern hardware. The recording itself is O(1) after acquiring the lock.
    ///
    /// # Example
    ///
    /// ```ignore
    /// histogram.record(150); // Record 150 microseconds
    /// histogram.record(2500); // Record 2.5ms - may trigger warning
    /// ```
    pub fn record(&self, micros: u64) {
        // Check threshold before acquiring lock for better performance
        if self.threshold_micros > 0 && micros > self.threshold_micros {
            warn!(
                target: "keyrx::metrics",
                histogram = self.name,
                latency_micros = micros,
                threshold_micros = self.threshold_micros,
                "Latency threshold exceeded"
            );
        }

        // Acquire lock and record sample
        // Saturate at histogram max to handle extreme outliers gracefully
        // If mutex is poisoned, skip recording rather than panic
        if let Ok(mut hist) = self.histogram.lock() {
            let _ = hist.record(micros.min(3_600_000_000));
        }
    }

    /// Get a percentile value from the histogram.
    ///
    /// # Arguments
    ///
    /// * `percentile` - Percentile to query (0.0 to 100.0)
    ///
    /// # Returns
    ///
    /// The latency value in microseconds at the requested percentile, or 0
    /// if the histogram is empty.
    ///
    /// # Performance
    ///
    /// This operation is O(log n) where n is the number of buckets (not samples).
    /// Since the bucket count is fixed, this is effectively O(1).
    ///
    /// # Example
    ///
    /// ```ignore
    /// let p50 = histogram.percentile(50.0); // Median
    /// let p95 = histogram.percentile(95.0); // 95th percentile
    /// let p99 = histogram.percentile(99.0); // 99th percentile
    /// ```
    pub fn percentile(&self, percentile: f64) -> u64 {
        self.histogram
            .lock()
            .map(|hist| hist.value_at_quantile(percentile / 100.0))
            .unwrap_or(0)
    }

    /// Get the mean (average) latency.
    ///
    /// # Returns
    ///
    /// The mean latency in microseconds, or 0.0 if the histogram is empty.
    pub fn mean(&self) -> f64 {
        self.histogram.lock().map(|hist| hist.mean()).unwrap_or(0.0)
    }

    /// Get the standard deviation of latencies.
    ///
    /// # Returns
    ///
    /// The standard deviation in microseconds.
    pub fn stddev(&self) -> f64 {
        self.histogram
            .lock()
            .map(|hist| hist.stdev())
            .unwrap_or(0.0)
    }

    /// Get the minimum recorded latency.
    ///
    /// # Returns
    ///
    /// The minimum latency in microseconds, or 0 if the histogram is empty.
    pub fn min(&self) -> u64 {
        self.histogram.lock().map(|hist| hist.min()).unwrap_or(0)
    }

    /// Get the maximum recorded latency.
    ///
    /// # Returns
    ///
    /// The maximum latency in microseconds, or 0 if the histogram is empty.
    pub fn max(&self) -> u64 {
        self.histogram.lock().map(|hist| hist.max()).unwrap_or(0)
    }

    /// Get the total number of recorded samples.
    ///
    /// # Returns
    ///
    /// The total count of samples recorded in this histogram.
    pub fn count(&self) -> u64 {
        self.histogram.lock().map(|hist| hist.len()).unwrap_or(0)
    }

    /// Reset the histogram, clearing all recorded samples.
    ///
    /// This is useful for starting fresh measurements or clearing data
    /// from a previous time window.
    pub fn reset(&self) {
        if let Ok(mut hist) = self.histogram.lock() {
            hist.reset();
        }
    }
}

// Implement Send + Sync to allow sharing across threads
// Safety: Mutex provides interior mutability and thread safety
#[allow(unsafe_code)]
unsafe impl Send for LatencyHistogram {}
#[allow(unsafe_code)]
unsafe impl Sync for LatencyHistogram {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_percentile() {
        let histogram = LatencyHistogram::new(0); // No threshold

        // Record some samples
        for i in 1..=100 {
            histogram.record(i * 10);
        }

        // Verify percentiles are in expected ranges
        let p50 = histogram.percentile(50.0);
        let p95 = histogram.percentile(95.0);
        let p99 = histogram.percentile(99.0);

        assert!(p50 >= 400 && p50 <= 600, "p50={}", p50);
        assert!(p95 >= 900 && p95 <= 1000, "p95={}", p95);
        assert!(p99 >= 950 && p99 <= 1000, "p99={}", p99);
    }

    #[test]
    fn test_threshold_warning() {
        // This test verifies the warning path is exercised
        // Actual warning logging is tested via tracing subscriber
        let histogram = LatencyHistogram::with_name("test", 100);

        histogram.record(50); // Below threshold
        histogram.record(150); // Above threshold - logs warning

        assert_eq!(histogram.count(), 2);
    }

    #[test]
    fn test_empty_histogram() {
        let histogram = LatencyHistogram::new(0);

        // Empty histogram should return 0 for all queries
        assert_eq!(histogram.percentile(50.0), 0);
        assert_eq!(histogram.min(), 0);
        assert_eq!(histogram.max(), 0);
        assert_eq!(histogram.count(), 0);
    }

    #[test]
    fn test_reset() {
        let histogram = LatencyHistogram::new(0);

        histogram.record(100);
        histogram.record(200);
        assert_eq!(histogram.count(), 2);

        histogram.reset();
        assert_eq!(histogram.count(), 0);
        assert_eq!(histogram.percentile(50.0), 0);
    }

    #[test]
    fn test_statistics() {
        let histogram = LatencyHistogram::new(0);

        // Record uniform samples
        for i in 1..=10 {
            histogram.record(i * 100);
        }

        assert_eq!(histogram.count(), 10);
        assert_eq!(histogram.min(), 100);
        assert_eq!(histogram.max(), 1000);

        let mean = histogram.mean();
        assert!(mean >= 500.0 && mean <= 600.0, "mean={}", mean);

        let stddev = histogram.stddev();
        assert!(stddev > 0.0, "stddev={}", stddev);
    }

    #[test]
    fn test_extreme_values() {
        let histogram = LatencyHistogram::new(0);

        // Record very small and very large values
        histogram.record(1); // Minimum boundary
        histogram.record(1_000_000); // 1 second
        histogram.record(3_600_000_000); // 1 hour (max)

        assert_eq!(histogram.count(), 3);
        assert_eq!(histogram.min(), 1);
        assert!(histogram.max() > 1_000_000);
    }

    #[test]
    fn test_thread_safety() {
        use std::sync::Arc;
        use std::thread;

        let histogram = Arc::new(LatencyHistogram::new(0));
        let mut handles = vec![];

        // Spawn 10 threads that each record 100 samples
        for _ in 0..10 {
            let hist = Arc::clone(&histogram);
            let handle = thread::spawn(move || {
                for i in 1..=100 {
                    hist.record(i * 10);
                }
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().expect("Thread panicked");
        }

        // Verify all samples were recorded
        assert_eq!(histogram.count(), 1000);
    }

    #[test]
    fn test_with_name() {
        let histogram = LatencyHistogram::with_name("test_operation", 500);

        histogram.record(100);
        histogram.record(600); // Exceeds threshold

        assert_eq!(histogram.count(), 2);
    }

    #[test]
    fn test_percentile_accuracy() {
        let histogram = LatencyHistogram::new(0);

        // Record 1000 samples: 1, 2, 3, ..., 1000
        for i in 1..=1000 {
            histogram.record(i);
        }

        // p50 should be around 500
        let p50 = histogram.percentile(50.0);
        assert!(p50 >= 490 && p50 <= 510, "p50={}", p50);

        // p95 should be around 950
        let p95 = histogram.percentile(95.0);
        assert!(p95 >= 940 && p95 <= 960, "p95={}", p95);

        // p99 should be around 990
        let p99 = histogram.percentile(99.0);
        assert!(p99 >= 980 && p99 <= 1000, "p99={}", p99);
    }
}
