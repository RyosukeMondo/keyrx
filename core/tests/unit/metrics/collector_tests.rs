#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::print_stdout, clippy::print_stderr, clippy::field_reassign_with_default, clippy::useless_conversion, clippy::assertions_on_constants, clippy::manual_div_ceil, clippy::manual_strip, clippy::len_zero, clippy::redundant_closure, clippy::manual_range_contains, clippy::default_constructed_unit_structs, clippy::clone_on_copy, clippy::io_other_error, clippy::bool_assert_comparison, clippy::approx_constant, clippy::let_unit_value, clippy::while_let_on_iterator, clippy::await_holding_lock, clippy::unnecessary_cast, clippy::drop_non_drop, clippy::needless_range_loop, unused_imports, unused_variables, dead_code, unsafe_code, clippy::collapsible_if, clippy::bool_comparison, unexpected_cfgs)]
//! Integration tests for the full metrics collector.
//!
//! These tests verify that all metrics components work together correctly
//! and that the collector meets performance requirements.

use keyrx_core::metrics::{
    collector::MetricsCollector, full_collector::FullMetricsCollector, operation::Operation,
};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[test]
fn test_collector_initialization() {
    let collector = FullMetricsCollector::new();

    // Verify all operation histograms are created
    for operation in Operation::all() {
        assert!(collector.get_latency_histogram(*operation).is_some());
    }

    // Verify memory monitor exists
    let mem_stats = collector.get_memory_monitor().stats();
    assert_eq!(mem_stats.sample_count, 0);

    // Verify profile points exists
    assert_eq!(collector.get_profile_points().count(), 0);
}

#[test]
fn test_collector_with_custom_threshold() {
    let collector = FullMetricsCollector::with_threshold(500);

    // Record a latency below threshold (should not warn)
    collector.record_latency(Operation::EventProcess, 400);

    let histogram = collector
        .get_latency_histogram(Operation::EventProcess)
        .unwrap();
    assert_eq!(histogram.count(), 1);
}

#[test]
fn test_all_metrics_types_integrated() {
    let collector = FullMetricsCollector::new();

    // Record latency
    collector.record_latency(Operation::EventProcess, 150);
    collector.record_latency(Operation::RuleMatch, 50);
    collector.record_latency(Operation::ActionExecute, 200);

    // Record errors
    collector.record_error("io");
    collector.record_error("parser");

    // Record memory
    collector.record_memory(1024 * 1024);
    collector.record_memory(2048 * 1024);

    // Record profile
    collector.record_profile("test_function", 100);

    // Verify all metrics were recorded
    let event_hist = collector
        .get_latency_histogram(Operation::EventProcess)
        .unwrap();
    assert_eq!(event_hist.count(), 1);

    let mem_stats = collector.get_memory_monitor().stats();
    assert_eq!(mem_stats.sample_count, 2);

    let profile_stats = collector.get_profile_points().get_stats("test_function");
    assert_eq!(profile_stats.count, 1);
}

#[test]
fn test_snapshot_captures_all_metrics() {
    let collector = FullMetricsCollector::new();

    // Record various metrics
    collector.record_latency(Operation::EventProcess, 100);
    collector.record_latency(Operation::EventProcess, 200);
    collector.record_latency(Operation::RuleMatch, 50);
    collector.record_memory(1024 * 1024);
    collector.record_profile("func_a", 150);
    collector.record_error("io");
    collector.record_error("io");
    collector.record_error("parser");

    let snapshot = collector.snapshot();

    // Verify snapshot has timestamp
    assert!(snapshot.timestamp > 0);

    // Verify latency stats are present
    assert!(snapshot.latencies.contains_key("event_process"));
    assert!(snapshot.latencies.contains_key("rule_match"));

    let event_stats = &snapshot.latencies["event_process"];
    assert_eq!(event_stats.count, 2);
    assert_eq!(event_stats.min, 100);
    assert_eq!(event_stats.max, 200);

    // Verify memory stats
    assert_eq!(snapshot.memory.current, 1024 * 1024);

    // Verify profile stats
    assert!(snapshot.profiles.contains_key("func_a"));

    // Verify error stats
    assert_eq!(snapshot.errors.total, 3);
    assert_eq!(snapshot.errors.by_type.get("io"), Some(&2));
    assert_eq!(snapshot.errors.by_type.get("parser"), Some(&1));
}

#[test]
fn test_reset_clears_all_collectors() {
    let collector = FullMetricsCollector::new();

    // Record data
    collector.record_latency(Operation::EventProcess, 100);
    collector.record_memory(1024);
    collector.record_profile("test", 50);
    collector.record_error("io");

    // Verify data exists
    assert!(
        collector
            .get_latency_histogram(Operation::EventProcess)
            .unwrap()
            .count()
            > 0
    );
    assert!(collector.get_memory_monitor().stats().sample_count > 0);
    assert!(collector.get_profile_points().count() > 0);

    // Reset
    collector.reset();

    // Verify all cleared
    assert_eq!(
        collector
            .get_latency_histogram(Operation::EventProcess)
            .unwrap()
            .count(),
        0
    );
    assert_eq!(collector.get_memory_monitor().stats().sample_count, 0);
    assert_eq!(collector.get_profile_points().count(), 0);
    let error_stats = collector.get_error_metrics().snapshot();
    assert_eq!(error_stats.total, 0);
    assert!(error_stats.by_type.is_empty());
}

#[test]
fn test_concurrent_multi_metric_recording() {
    let collector = Arc::new(FullMetricsCollector::new());
    let mut handles = vec![];

    // Spawn 4 threads, each recording different metrics
    for thread_id in 0..4 {
        let c = Arc::clone(&collector);
        let handle = thread::spawn(move || {
            for i in 0..100 {
                c.record_latency(Operation::EventProcess, (thread_id * 100 + i) as u64);
                c.record_memory(1024 * (thread_id * 100 + i));
                c.record_profile("concurrent_test", (thread_id * 10 + i) as u64);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Verify all metrics recorded correctly
    let hist = collector
        .get_latency_histogram(Operation::EventProcess)
        .unwrap();
    assert_eq!(hist.count(), 400);

    let mem_stats = collector.get_memory_monitor().stats();
    assert_eq!(mem_stats.sample_count, 400);

    let profile_stats = collector.get_profile_points().get_stats("concurrent_test");
    assert_eq!(profile_stats.count, 400);
}

#[test]
fn test_profile_guard_integration() {
    let collector = FullMetricsCollector::new();

    {
        let _guard = collector.start_profile("guarded_function");
        thread::sleep(Duration::from_micros(100));
    }

    let stats = collector.get_profile_points().get_stats("guarded_function");
    assert_eq!(stats.count, 1);
    assert!(stats.total_micros > 50); // Some time should be recorded
}

#[test]
fn test_multiple_operations_independent_tracking() {
    let collector = FullMetricsCollector::new();

    // Record different latencies for different operations
    collector.record_latency(Operation::EventProcess, 100);
    collector.record_latency(Operation::EventProcess, 200);

    collector.record_latency(Operation::RuleMatch, 50);
    collector.record_latency(Operation::RuleMatch, 75);

    collector.record_latency(Operation::ActionExecute, 300);

    // Verify independent tracking
    let event_hist = collector
        .get_latency_histogram(Operation::EventProcess)
        .unwrap();
    assert_eq!(event_hist.count(), 2);
    assert_eq!(event_hist.min(), 100);
    assert_eq!(event_hist.max(), 200);

    let rule_hist = collector
        .get_latency_histogram(Operation::RuleMatch)
        .unwrap();
    assert_eq!(rule_hist.count(), 2);
    assert_eq!(rule_hist.min(), 50);
    assert_eq!(rule_hist.max(), 75);

    let action_hist = collector
        .get_latency_histogram(Operation::ActionExecute)
        .unwrap();
    assert_eq!(action_hist.count(), 1);
    assert_eq!(action_hist.max(), 300);
}

#[test]
fn test_snapshot_json_serialization() {
    let collector = FullMetricsCollector::new();

    collector.record_latency(Operation::EventProcess, 150);
    collector.record_memory(1024 * 1024);
    collector.record_profile("test_func", 100);

    let snapshot = collector.snapshot();

    // Verify snapshot can be serialized to JSON
    let json = serde_json::to_string(&snapshot).expect("Failed to serialize");
    assert!(json.contains("event_process"));
    assert!(json.contains("timestamp"));

    // Verify deserialization
    let _deserialized: keyrx_core::metrics::snapshot::MetricsSnapshot =
        serde_json::from_str(&json).expect("Failed to deserialize");
}

#[test]
fn test_memory_leak_detection_integration() {
    let collector = FullMetricsCollector::new();

    // Record monotonically increasing memory
    for i in 0..30 {
        collector.record_memory(1024 * 1024 + (i * 15 * 1024));
    }

    let snapshot = collector.snapshot();
    assert!(snapshot.memory.has_potential_leak);
}

#[test]
fn test_latency_percentiles_in_snapshot() {
    let collector = FullMetricsCollector::new();

    // Record 100 samples
    for i in 1..=100 {
        collector.record_latency(Operation::EventProcess, i * 10);
    }

    let snapshot = collector.snapshot();
    let event_stats = &snapshot.latencies["event_process"];

    // Verify percentiles are reasonable
    assert!(event_stats.p50 >= 400 && event_stats.p50 <= 600);
    assert!(event_stats.p95 >= 900 && event_stats.p95 <= 1000);
    assert!(event_stats.p99 >= 950 && event_stats.p99 <= 1000);
    assert_eq!(event_stats.count, 100);
}

#[test]
fn test_default_constructor() {
    let collector = FullMetricsCollector::default();

    collector.record_latency(Operation::EventProcess, 100);
    let hist = collector
        .get_latency_histogram(Operation::EventProcess)
        .unwrap();
    assert_eq!(hist.count(), 1);
}

#[test]
fn test_realistic_application_simulation() {
    let collector = Arc::new(FullMetricsCollector::new());

    // Simulate realistic application with multiple threads
    let mut handles = vec![];

    // Event processing thread
    {
        let c = Arc::clone(&collector);
        handles.push(thread::spawn(move || {
            for _ in 0..50 {
                let _guard = c.start_profile("event_loop");
                c.record_latency(Operation::EventProcess, 150);
                thread::sleep(Duration::from_micros(10));
            }
        }));
    }

    // Rule matching thread
    {
        let c = Arc::clone(&collector);
        handles.push(thread::spawn(move || {
            for _ in 0..100 {
                c.record_latency(Operation::RuleMatch, 50);
                thread::sleep(Duration::from_micros(5));
            }
        }));
    }

    // Action execution thread
    {
        let c = Arc::clone(&collector);
        handles.push(thread::spawn(move || {
            for _ in 0..30 {
                c.record_latency(Operation::ActionExecute, 200);
                thread::sleep(Duration::from_micros(20));
            }
        }));
    }

    // Memory monitoring thread
    {
        let c = Arc::clone(&collector);
        handles.push(thread::spawn(move || {
            for i in 0..20 {
                c.record_memory(1024 * 1024 + (i * 1024));
                thread::sleep(Duration::from_micros(50));
            }
        }));
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Verify metrics were collected
    let snapshot = collector.snapshot();

    assert!(snapshot.latencies.contains_key("event_process"));
    assert!(snapshot.latencies.contains_key("rule_match"));
    assert!(snapshot.latencies.contains_key("action_execute"));

    assert_eq!(snapshot.latencies["event_process"].count, 50);
    assert_eq!(snapshot.latencies["rule_match"].count, 100);
    assert_eq!(snapshot.latencies["action_execute"].count, 30);

    // Verify memory was recorded (20 samples)
    assert!(snapshot.memory.current > 0);
    assert!(snapshot.profiles.contains_key("event_loop"));
}

#[test]
fn test_high_frequency_recording_performance() {
    let collector = FullMetricsCollector::new();

    // Measure overhead of 10000 metric recordings
    let start = std::time::Instant::now();
    for i in 0..10000 {
        collector.record_latency(Operation::EventProcess, (i % 1000) as u64);
    }
    let elapsed = start.elapsed();

    // Should complete in well under 100ms (target: < 1us per recording)
    assert!(
        elapsed.as_millis() < 100,
        "Recording took too long: {:?}",
        elapsed
    );

    let hist = collector
        .get_latency_histogram(Operation::EventProcess)
        .unwrap();
    assert_eq!(hist.count(), 10000);
}

#[test]
fn test_snapshot_consistency() {
    let collector = FullMetricsCollector::new();

    // Record metrics
    collector.record_latency(Operation::EventProcess, 100);
    collector.record_latency(Operation::EventProcess, 200);
    collector.record_memory(1024);

    // Take multiple snapshots
    let snapshot1 = collector.snapshot();
    thread::sleep(Duration::from_millis(1));
    let snapshot2 = collector.snapshot();

    // Timestamps should be different
    assert!(snapshot2.timestamp >= snapshot1.timestamp);

    // But metric values should be same (no new recordings)
    assert_eq!(
        snapshot1.latencies["event_process"].count,
        snapshot2.latencies["event_process"].count
    );
}

#[test]
fn test_all_operations_have_histograms() {
    let collector = FullMetricsCollector::new();

    // Verify every operation type has a histogram
    for operation in Operation::all() {
        let histogram = collector.get_latency_histogram(*operation);
        assert!(histogram.is_some(), "Missing histogram for {:?}", operation);
    }
}

#[test]
fn test_memory_stats_in_snapshot() {
    let collector = FullMetricsCollector::new();

    collector.record_memory(1024);
    collector.record_memory(2048);
    collector.record_memory(1536);

    let snapshot = collector.snapshot();

    assert_eq!(snapshot.memory.current, 1536);
    assert_eq!(snapshot.memory.peak, 2048);
    assert_eq!(snapshot.memory.baseline, 1024);
    assert_eq!(snapshot.memory.growth, 512);
    assert!(!snapshot.memory.has_potential_leak);
}

#[test]
fn test_profile_stats_in_snapshot() {
    let collector = FullMetricsCollector::new();

    collector.record_profile("func_a", 100);
    collector.record_profile("func_a", 200);
    collector.record_profile("func_b", 300);

    let snapshot = collector.snapshot();

    assert!(snapshot.profiles.contains_key("func_a"));
    assert!(snapshot.profiles.contains_key("func_b"));

    let func_a_stats = &snapshot.profiles["func_a"];
    assert_eq!(func_a_stats.count, 2);
    assert_eq!(func_a_stats.total_micros, 300);
}

#[test]
fn test_empty_snapshot() {
    let collector = FullMetricsCollector::new();

    let snapshot = collector.snapshot();

    // Should have latency entries for all operations (but with 0 count)
    for operation in Operation::all() {
        assert!(snapshot.latencies.contains_key(operation.name()));
        let stats = &snapshot.latencies[operation.name()];
        assert_eq!(stats.count, 0);
    }

    // Memory should be empty (check fields that exist)
    assert_eq!(snapshot.memory.current, 0);
    assert_eq!(snapshot.memory.peak, 0);
    assert_eq!(snapshot.memory.baseline, 0);

    // Profiles should be empty
    assert_eq!(snapshot.profiles.len(), 0);
}
