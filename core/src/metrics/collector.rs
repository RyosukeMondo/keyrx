//! Metrics collector trait and RAII guard.
//!
//! This module defines the core abstraction for metrics collection in KeyRx.
//! The `MetricsCollector` trait allows for pluggable implementations ranging
//! from no-op collectors (zero overhead) to full profiling collectors.

use std::time::Instant;

use super::operation::Operation;

/// Trait for metrics collection implementations.
///
/// This trait abstracts the metrics collection system, allowing for
/// different implementations (e.g., no-op, full profiling, test doubles).
///
/// # Thread Safety
///
/// All implementations MUST be `Send + Sync` to allow usage from multiple
/// threads without synchronization overhead.
///
/// # Performance
///
/// Implementations should strive for < 1 microsecond overhead per recording.
/// Zero-allocation implementations are preferred for hot paths.
pub trait MetricsCollector: Send + Sync {
    /// Record latency for an operation in microseconds.
    ///
    /// # Arguments
    ///
    /// * `operation` - The type of operation being measured
    /// * `micros` - Duration in microseconds
    ///
    /// # Performance
    ///
    /// This method is called in hot paths and MUST be fast (< 1us overhead).
    fn record_latency(&self, operation: Operation, micros: u64);

    /// Record memory usage in bytes.
    ///
    /// # Arguments
    ///
    /// * `bytes` - Current memory usage in bytes
    fn record_memory(&self, bytes: usize);

    /// Start profiling a named code section.
    ///
    /// Returns an RAII guard that automatically records the elapsed time
    /// when dropped.
    ///
    /// # Arguments
    ///
    /// * `name` - Static string identifier for the profile point
    ///
    /// # Example
    ///
    /// ```ignore
    /// let _guard = collector.start_profile("expensive_function");
    /// // ... code to profile ...
    /// // Guard is dropped here, recording elapsed time automatically
    /// ```
    fn start_profile(&self, name: &'static str) -> ProfileGuard<'_>;

    /// Record a profile point manually.
    ///
    /// This is called automatically by `ProfileGuard` on drop, but can
    /// also be called manually if needed.
    ///
    /// # Arguments
    ///
    /// * `name` - Static string identifier for the profile point
    /// * `micros` - Duration in microseconds
    fn record_profile(&self, name: &'static str, micros: u64);

    /// Get a snapshot of current metrics.
    ///
    /// Returns a point-in-time view of all collected metrics.
    fn snapshot(&self) -> MetricsSnapshot;

    /// Reset all collected metrics.
    ///
    /// Clears all histograms, counters, and accumulated data.
    fn reset(&self);
}

/// RAII guard for automatic profiling.
///
/// When dropped, this guard automatically records the elapsed time
/// since its creation to the metrics collector.
pub struct ProfileGuard<'a> {
    collector: &'a dyn MetricsCollector,
    name: &'static str,
    start: Instant,
}

impl<'a> ProfileGuard<'a> {
    /// Create a new profile guard.
    ///
    /// This should typically be created via `MetricsCollector::start_profile`.
    pub fn new(collector: &'a dyn MetricsCollector, name: &'static str) -> Self {
        Self {
            collector,
            name,
            start: Instant::now(),
        }
    }
}

impl Drop for ProfileGuard<'_> {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed().as_micros() as u64;
        self.collector.record_profile(self.name, elapsed);
    }
}

/// Snapshot of metrics at a point in time.
///
/// This is a placeholder that will be replaced with the full implementation
/// in a later task (Task 7: MetricsSnapshot export).
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    /// Unix timestamp in milliseconds
    pub timestamp: u64,
    /// Placeholder for future implementation
    pub _placeholder: (),
}

impl MetricsSnapshot {
    /// Create an empty snapshot.
    pub fn empty() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        Self {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            _placeholder: (),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;

    // Test implementation that counts calls
    struct TestCollector {
        latency_calls: Arc<AtomicU64>,
        memory_calls: Arc<AtomicU64>,
        profile_calls: Arc<AtomicU64>,
    }

    impl TestCollector {
        fn new() -> Self {
            Self {
                latency_calls: Arc::new(AtomicU64::new(0)),
                memory_calls: Arc::new(AtomicU64::new(0)),
                profile_calls: Arc::new(AtomicU64::new(0)),
            }
        }
    }

    impl MetricsCollector for TestCollector {
        fn record_latency(&self, _operation: Operation, _micros: u64) {
            self.latency_calls.fetch_add(1, Ordering::Relaxed);
        }

        fn record_memory(&self, _bytes: usize) {
            self.memory_calls.fetch_add(1, Ordering::Relaxed);
        }

        fn start_profile(&self, name: &'static str) -> ProfileGuard<'_> {
            ProfileGuard::new(self, name)
        }

        fn record_profile(&self, _name: &'static str, _micros: u64) {
            self.profile_calls.fetch_add(1, Ordering::Relaxed);
        }

        fn snapshot(&self) -> MetricsSnapshot {
            MetricsSnapshot::empty()
        }

        fn reset(&self) {
            self.latency_calls.store(0, Ordering::Relaxed);
            self.memory_calls.store(0, Ordering::Relaxed);
            self.profile_calls.store(0, Ordering::Relaxed);
        }
    }

    #[test]
    fn test_record_latency() {
        let collector = TestCollector::new();
        collector.record_latency(Operation::EventProcess, 100);
        assert_eq!(collector.latency_calls.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_record_memory() {
        let collector = TestCollector::new();
        collector.record_memory(1024);
        assert_eq!(collector.memory_calls.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_profile_guard_auto_drop() {
        let collector = TestCollector::new();
        {
            let _guard = collector.start_profile("test_function");
            // Guard should record on drop
        }
        assert_eq!(collector.profile_calls.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_profile_guard_timing() {
        let collector = TestCollector::new();
        {
            let _guard = collector.start_profile("test_function");
            std::thread::sleep(std::time::Duration::from_micros(100));
        }
        // Just verify it was called, timing is non-deterministic in tests
        assert_eq!(collector.profile_calls.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_reset() {
        let collector = TestCollector::new();
        collector.record_latency(Operation::EventProcess, 100);
        collector.record_memory(1024);
        assert_eq!(collector.latency_calls.load(Ordering::Relaxed), 1);
        assert_eq!(collector.memory_calls.load(Ordering::Relaxed), 1);

        collector.reset();
        assert_eq!(collector.latency_calls.load(Ordering::Relaxed), 0);
        assert_eq!(collector.memory_calls.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_snapshot_has_timestamp() {
        let snapshot = MetricsSnapshot::empty();
        assert!(snapshot.timestamp > 0);
    }
}
