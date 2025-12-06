//! Metrics collector trait and RAII guard.
//!
//! This module defines the core abstraction for metrics collection in KeyRx.
//! The `MetricsCollector` trait allows for pluggable implementations ranging
//! from no-op collectors (zero overhead) to full profiling collectors.

use std::any::Any;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{Duration, Instant};

#[cfg(feature = "otel-tracing")]
use opentelemetry::metrics::{Counter, Histogram, UpDownCounter};
#[cfg(feature = "otel-tracing")]
use opentelemetry::KeyValue;

use super::operation::Operation;
use super::snapshot::MetricsSnapshot;

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
pub trait MetricsCollector: Send + Sync + Any {
    /// Downcast support for accessing concrete collectors (e.g., OTEL).
    fn as_any(&self) -> &dyn Any;
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

    /// Record an error occurrence for observability.
    ///
    /// # Arguments
    ///
    /// * `error_type` - Logical error category (e.g., "io", "script", "config")
    ///
    /// # Default
    ///
    /// Implementations may override to track error counters and rates.
    fn record_error(&self, _error_type: &str) {}

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

/// OpenTelemetry-backed metrics collector for key events, latency, and gauges.
///
/// This collector wires the KeyRx event stream into OTEL counters, histograms,
/// and gauges to satisfy the metrics collection requirements:
/// - Counters for key processing and errors (Req 1.1, 1.4)
/// - Histograms for processing and script execution latency (Req 1.2)
/// - Gauges for active sessions/devices (Req 1.3)
///
/// When the `otel-tracing` feature is disabled, methods degrade to no-ops while
/// still tracking the last gauge values to keep call sites feature-agnostic.
#[derive(Debug)]
pub struct OtelMetricsCollector {
    #[cfg(feature = "otel-tracing")]
    key_events_total: Counter<u64>,
    #[cfg(feature = "otel-tracing")]
    errors_total: Counter<u64>,
    #[cfg(feature = "otel-tracing")]
    processing_latency: Histogram<f64>,
    #[cfg(feature = "otel-tracing")]
    script_execution_time: Histogram<f64>,
    #[cfg(feature = "otel-tracing")]
    active_sessions: UpDownCounter<i64>,
    #[cfg(feature = "otel-tracing")]
    active_devices: UpDownCounter<i64>,
    session_value: AtomicI64,
    device_value: AtomicI64,
}

impl OtelMetricsCollector {
    /// Create a new OTEL metrics collector using the global meter provider.
    ///
    /// Instruments are prefixed with `keyrx.*` and use microsecond buckets to
    /// align with latency requirements.
    pub fn new() -> Self {
        #[cfg(feature = "otel-tracing")]
        {
            const LATENCY_BUCKETS_US: &[f64] = &[
                50.0,
                100.0,
                250.0,
                500.0,
                1_000.0,
                2_500.0,
                5_000.0,
                10_000.0,
                25_000.0,
                50_000.0,
                100_000.0,
                250_000.0,
                500_000.0,
                1_000_000.0,
            ];

            let meter = opentelemetry::global::meter("keyrx.metrics");

            let key_events_total = meter
                .u64_counter("keyrx.key_events.total")
                .with_description("Total key events processed by action and key code")
                .with_unit("1")
                .build();

            let errors_total = meter
                .u64_counter("keyrx.errors.total")
                .with_description("Total errors encountered by type")
                .with_unit("1")
                .build();

            let processing_latency = meter
                .f64_histogram("keyrx.processing.latency.us")
                .with_description("Event processing latency in microseconds")
                .with_unit("us")
                .with_boundaries(LATENCY_BUCKETS_US.to_vec())
                .build();

            let script_execution_time = meter
                .f64_histogram("keyrx.script.execution.us")
                .with_description("Script execution time in microseconds")
                .with_unit("us")
                .with_boundaries(LATENCY_BUCKETS_US.to_vec())
                .build();

            let active_sessions = meter
                .i64_up_down_counter("keyrx.sessions.active")
                .with_description("Active session count")
                .with_unit("1")
                .build();

            let active_devices = meter
                .i64_up_down_counter("keyrx.devices.active")
                .with_description("Active device count")
                .with_unit("1")
                .build();

            return Self {
                key_events_total,
                errors_total,
                processing_latency,
                script_execution_time,
                active_sessions,
                active_devices,
                session_value: AtomicI64::new(0),
                device_value: AtomicI64::new(0),
            };
        }

        #[cfg(not(feature = "otel-tracing"))]
        Self {
            session_value: AtomicI64::new(0),
            device_value: AtomicI64::new(0),
        }
    }

    /// Record a processed key event with key code and action attributes.
    pub fn record_key_event(&self, key_code: u32, action: &str) {
        #[cfg(feature = "otel-tracing")]
        {
            self.key_events_total.add(
                1,
                &[
                    KeyValue::new("key_code", key_code as i64),
                    KeyValue::new("action", action.to_string()),
                ],
            );
        }
        let _ = (key_code, action);
    }

    /// Record event processing latency in microseconds.
    pub fn record_processing_latency(&self, duration: Duration) {
        #[cfg(feature = "otel-tracing")]
        {
            let micros = duration.as_secs_f64() * 1_000_000.0;
            self.processing_latency
                .record(micros, &[KeyValue::new("unit", "us")]);
        }
        let _ = duration;
    }

    /// Record script execution time in microseconds.
    pub fn record_script_execution_time(&self, duration: Duration) {
        #[cfg(feature = "otel-tracing")]
        {
            let micros = duration.as_secs_f64() * 1_000_000.0;
            self.script_execution_time
                .record(micros, &[KeyValue::new("unit", "us")]);
        }
        let _ = duration;
    }

    /// Record an error occurrence with error type attribute.
    pub fn record_error(&self, error_type: &str) {
        #[cfg(feature = "otel-tracing")]
        {
            self.errors_total
                .add(1, &[KeyValue::new("error_type", error_type.to_string())]);
        }
        let _ = error_type;
    }

    /// Set the current active session gauge value.
    pub fn set_active_sessions(&self, count: i64) {
        let previous = self.session_value.swap(count, Ordering::Relaxed);
        #[cfg(feature = "otel-tracing")]
        {
            let delta = count - previous;
            if delta != 0 {
                self.active_sessions
                    .add(delta, &[KeyValue::new("state", "session")]);
            }
        }
        let _ = previous;
    }

    /// Set the current active device gauge value.
    pub fn set_active_devices(&self, count: i64) {
        let previous = self.device_value.swap(count, Ordering::Relaxed);
        #[cfg(feature = "otel-tracing")]
        {
            let delta = count - previous;
            if delta != 0 {
                self.active_devices
                    .add(delta, &[KeyValue::new("state", "device")]);
            }
        }
        let _ = previous;
    }
}

impl Default for OtelMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl OtelMetricsCollector {
    fn current_sessions(&self) -> i64 {
        self.session_value.load(Ordering::Relaxed)
    }

    fn current_devices(&self) -> i64 {
        self.device_value.load(Ordering::Relaxed)
    }
}

impl MetricsCollector for OtelMetricsCollector {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn record_latency(&self, operation: Operation, micros: u64) {
        match operation {
            Operation::EventProcess => {
                self.record_processing_latency(Duration::from_micros(micros));
            }
            Operation::RuleMatch | Operation::ActionExecute => {
                self.record_script_execution_time(Duration::from_micros(micros));
            }
            _ => {}
        }
    }

    fn record_memory(&self, _bytes: usize) {}

    fn start_profile(&self, name: &'static str) -> ProfileGuard<'_> {
        ProfileGuard::new(self, name)
    }

    fn record_profile(&self, _name: &'static str, _micros: u64) {}

    fn record_error(&self, error_type: &str) {
        OtelMetricsCollector::record_error(self, error_type);
    }

    fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot::empty()
    }

    fn reset(&self) {
        self.set_active_sessions(0);
        self.set_active_devices(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    // Test implementation that counts calls
    struct TestCollector {
        latency_calls: Arc<AtomicU64>,
        memory_calls: Arc<AtomicU64>,
        profile_calls: Arc<AtomicU64>,
        error_calls: Arc<AtomicU64>,
    }

    impl TestCollector {
        fn new() -> Self {
            Self {
                latency_calls: Arc::new(AtomicU64::new(0)),
                memory_calls: Arc::new(AtomicU64::new(0)),
                profile_calls: Arc::new(AtomicU64::new(0)),
                error_calls: Arc::new(AtomicU64::new(0)),
            }
        }
    }

    impl MetricsCollector for TestCollector {
        fn as_any(&self) -> &dyn Any {
            self
        }

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

        fn record_error(&self, _error_type: &str) {
            self.error_calls.fetch_add(1, Ordering::Relaxed);
        }

        fn snapshot(&self) -> MetricsSnapshot {
            MetricsSnapshot::empty()
        }

        fn reset(&self) {
            self.latency_calls.store(0, Ordering::Relaxed);
            self.memory_calls.store(0, Ordering::Relaxed);
            self.profile_calls.store(0, Ordering::Relaxed);
            self.error_calls.store(0, Ordering::Relaxed);
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
    fn otel_collector_tracks_gauges_locally() {
        let collector = OtelMetricsCollector::new();

        collector.set_active_sessions(2);
        collector.set_active_sessions(5);
        assert_eq!(collector.current_sessions(), 5);

        collector.set_active_devices(1);
        collector.set_active_devices(0);
        assert_eq!(collector.current_devices(), 0);
    }

    #[test]
    fn otel_collector_accepts_event_and_error_calls() {
        let collector = OtelMetricsCollector::new();

        // These calls should be no-ops without OTEL enabled but must not panic.
        collector.record_key_event(42, "press");
        collector.record_processing_latency(Duration::from_micros(250));
        collector.record_script_execution_time(Duration::from_micros(750));
        collector.record_error("test_error");
    }

    #[test]
    fn test_reset() {
        let collector = TestCollector::new();
        collector.record_latency(Operation::EventProcess, 100);
        collector.record_memory(1024);
        assert_eq!(collector.latency_calls.load(Ordering::Relaxed), 1);
        assert_eq!(collector.memory_calls.load(Ordering::Relaxed), 1);
        collector.record_error("test");
        assert_eq!(collector.error_calls.load(Ordering::Relaxed), 1);

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
