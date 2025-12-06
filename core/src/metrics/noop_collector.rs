//! Zero-overhead no-op metrics collector.
//!
//! This module provides a `NoOpCollector` that implements the `MetricsCollector`
//! trait with zero runtime overhead. All methods are marked `#[inline(always)]`
//! and do nothing, allowing the compiler to completely eliminate them in optimized
//! builds.
//!
//! # Use Cases
//!
//! - Production builds where metrics overhead is unacceptable
//! - Release mode where profiling is disabled
//! - Testing scenarios where metrics are not relevant
//!
//! # Performance
//!
//! In release builds with optimizations enabled, all methods compile to zero
//! instructions. The null object pattern ensures type safety without runtime cost.

use super::collector::{MetricsCollector, ProfileGuard};
use super::operation::Operation;
use super::snapshot::MetricsSnapshot;

/// Zero-overhead no-op metrics collector.
///
/// This collector implements the `MetricsCollector` trait but does nothing.
/// All methods are inlined and compile to no-ops in optimized builds.
///
/// # Example
///
/// ```ignore
/// use keyrx_core::metrics::noop_collector::NoOpCollector;
/// use keyrx_core::metrics::collector::MetricsCollector;
///
/// let collector = NoOpCollector::new();
/// collector.record_latency(Operation::EventProcess, 100); // Compiles to nothing
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct NoOpCollector;

impl NoOpCollector {
    /// Create a new no-op collector.
    ///
    /// This is a zero-cost operation as the struct is zero-sized.
    #[inline(always)]
    pub const fn new() -> Self {
        Self
    }
}

impl MetricsCollector for NoOpCollector {
    #[inline(always)]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    /// Record latency - no-op implementation.
    ///
    /// This compiles to zero instructions in release builds.
    #[inline(always)]
    fn record_latency(&self, _operation: Operation, _micros: u64) {
        // Intentionally empty - compiles to nothing
    }

    /// Record memory usage - no-op implementation.
    ///
    /// This compiles to zero instructions in release builds.
    #[inline(always)]
    fn record_memory(&self, _bytes: usize) {
        // Intentionally empty - compiles to nothing
    }

    /// Start profiling - returns a no-op guard.
    ///
    /// The guard creation may have minimal overhead, but the drop
    /// implementation will compile to nothing.
    #[inline(always)]
    fn start_profile(&self, name: &'static str) -> ProfileGuard<'_> {
        ProfileGuard::new(self, name)
    }

    /// Record profile point - no-op implementation.
    ///
    /// This compiles to zero instructions in release builds.
    #[inline(always)]
    fn record_profile(&self, _name: &'static str, _micros: u64) {
        // Intentionally empty - compiles to nothing
    }

    /// Record error - no-op implementation.
    #[inline(always)]
    fn record_error(&self, _error_type: &str) {
        // Intentionally empty - compiles to nothing
    }

    /// Get empty snapshot.
    ///
    /// Returns a minimal snapshot with just a timestamp.
    #[inline(always)]
    fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot::empty()
    }

    /// Reset metrics - no-op implementation.
    ///
    /// This compiles to zero instructions in release builds.
    #[inline(always)]
    fn reset(&self) {
        // Intentionally empty - compiles to nothing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noop_collector_is_zero_sized() {
        assert_eq!(std::mem::size_of::<NoOpCollector>(), 0);
    }

    #[test]
    fn test_noop_collector_default() {
        let collector = NoOpCollector::default();
        collector.record_latency(Operation::EventProcess, 100);
        collector.record_memory(1024);
    }

    #[test]
    fn test_noop_collector_new() {
        let collector = NoOpCollector::new();
        collector.record_latency(Operation::DriverRead, 50);
    }

    #[test]
    fn test_profile_guard_works() {
        let collector = NoOpCollector::new();
        {
            let _guard = collector.start_profile("test_function");
            // Guard should drop without panicking
        }
    }

    #[test]
    fn test_record_profile_manual() {
        let collector = NoOpCollector::new();
        collector.record_profile("manual_profile", 123);
    }

    #[test]
    fn test_snapshot_returns_empty() {
        let collector = NoOpCollector::new();
        let snapshot = collector.snapshot();
        assert!(snapshot.timestamp > 0);
    }

    #[test]
    fn test_reset_does_nothing() {
        let collector = NoOpCollector::new();
        collector.record_latency(Operation::EventProcess, 100);
        collector.reset();
        // No state to verify, just ensure it doesn't panic
    }

    #[test]
    fn test_clone() {
        let collector = NoOpCollector::new();
        let _cloned = collector.clone();
    }

    #[test]
    fn test_copy() {
        let collector = NoOpCollector::new();
        let _copied = collector;
        // Can still use original because it's Copy
        collector.record_latency(Operation::EventProcess, 100);
    }

    #[test]
    fn test_const_new() {
        const COLLECTOR: NoOpCollector = NoOpCollector::new();
        COLLECTOR.record_latency(Operation::EventProcess, 100);
    }

    #[test]
    fn test_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<NoOpCollector>();
        assert_sync::<NoOpCollector>();
    }
}
