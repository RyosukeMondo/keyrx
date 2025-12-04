//! Metrics collection overhead benchmarks.
//!
//! This benchmark suite verifies that metrics collection overhead meets
//! the < 1 microsecond target for all operations.

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use keyrx_core::metrics::{FullMetricsCollector, MetricsCollector, NoOpCollector, Operation};

/// Benchmark recording latency with FullMetricsCollector.
///
/// Target: < 1 microsecond per recording
fn benchmark_record_latency_full(c: &mut Criterion) {
    let collector = FullMetricsCollector::new();

    c.bench_function("full_collector_record_latency", |b| {
        b.iter(|| {
            collector.record_latency(black_box(Operation::EventProcess), black_box(150));
        })
    });
}

/// Benchmark recording latency with NoOpCollector.
///
/// Target: Should optimize to near-zero overhead
fn benchmark_record_latency_noop(c: &mut Criterion) {
    let collector = NoOpCollector::new();

    c.bench_function("noop_collector_record_latency", |b| {
        b.iter(|| {
            collector.record_latency(black_box(Operation::EventProcess), black_box(150));
        })
    });
}

/// Benchmark recording all operation types.
///
/// Target: < 1 microsecond per operation
fn benchmark_record_all_operations(c: &mut Criterion) {
    let collector = FullMetricsCollector::new();

    c.bench_function("full_collector_all_operations", |b| {
        b.iter(|| {
            for op in Operation::all() {
                collector.record_latency(black_box(*op), black_box(100));
            }
        })
    });
}

/// Benchmark recording memory usage.
///
/// Target: < 100 nanoseconds per recording
fn benchmark_record_memory(c: &mut Criterion) {
    let collector = FullMetricsCollector::new();

    c.bench_function("full_collector_record_memory", |b| {
        b.iter(|| {
            collector.record_memory(black_box(1024 * 1024));
        })
    });
}

/// Benchmark profile point recording via RAII guard.
///
/// Target: < 1 microsecond overhead (creation + drop)
fn benchmark_profile_guard(c: &mut Criterion) {
    let collector = FullMetricsCollector::new();

    c.bench_function("full_collector_profile_guard", |b| {
        b.iter(|| {
            let _guard = collector.start_profile(black_box("test_function"));
            // Guard drops here
        })
    });
}

/// Benchmark manual profile recording.
///
/// Target: < 1 microsecond per recording
fn benchmark_record_profile(c: &mut Criterion) {
    let collector = FullMetricsCollector::new();

    c.bench_function("full_collector_record_profile", |b| {
        b.iter(|| {
            collector.record_profile(black_box("test_function"), black_box(250));
        })
    });
}

/// Benchmark multiple profile points with different names.
///
/// Target: < 1 microsecond per unique profile point
fn benchmark_multiple_profile_points(c: &mut Criterion) {
    let collector = FullMetricsCollector::new();

    c.bench_function("full_collector_multiple_profiles", |b| {
        b.iter(|| {
            collector.record_profile(black_box("function_a"), black_box(100));
            collector.record_profile(black_box("function_b"), black_box(200));
            collector.record_profile(black_box("function_c"), black_box(300));
            collector.record_profile(black_box("function_a"), black_box(150));
        })
    });
}

/// Benchmark taking a snapshot.
///
/// This is not a hot path operation but should still be reasonably fast.
fn benchmark_snapshot(c: &mut Criterion) {
    let collector = FullMetricsCollector::new();

    // Pre-populate with some data
    for _ in 0..100 {
        collector.record_latency(Operation::EventProcess, 150);
        collector.record_latency(Operation::RuleMatch, 50);
        collector.record_memory(1024 * 1024);
        collector.record_profile("test_function", 250);
    }

    c.bench_function("full_collector_snapshot", |b| {
        b.iter(|| {
            black_box(collector.snapshot());
        })
    });
}

/// Benchmark reset operation.
///
/// This is not a hot path operation but should still be reasonably fast.
fn benchmark_reset(c: &mut Criterion) {
    c.bench_function("full_collector_reset", |b| {
        b.iter_batched(
            || {
                let collector = FullMetricsCollector::new();
                // Pre-populate with data
                for _ in 0..100 {
                    collector.record_latency(Operation::EventProcess, 150);
                    collector.record_memory(1024 * 1024);
                    collector.record_profile("test_function", 250);
                }
                collector
            },
            |collector| {
                black_box(collector.reset());
            },
            BatchSize::SmallInput,
        )
    });
}

/// Benchmark concurrent latency recording from multiple threads.
///
/// Target: Should scale linearly with minimal contention
fn benchmark_concurrent_latency(c: &mut Criterion) {
    use std::sync::Arc;
    use std::thread;

    let collector = Arc::new(FullMetricsCollector::new());

    c.bench_function("full_collector_concurrent_latency", |b| {
        b.iter(|| {
            let threads: Vec<_> = (0..4)
                .map(|_| {
                    let c = collector.clone();
                    thread::spawn(move || {
                        for _ in 0..25 {
                            c.record_latency(Operation::EventProcess, 150);
                        }
                    })
                })
                .collect();

            for t in threads {
                t.join().unwrap();
            }
        })
    });
}

/// Benchmark concurrent memory recording from multiple threads.
///
/// Target: Should be fast with minimal contention (atomic operations)
fn benchmark_concurrent_memory(c: &mut Criterion) {
    use std::sync::Arc;
    use std::thread;

    let collector = Arc::new(FullMetricsCollector::new());

    c.bench_function("full_collector_concurrent_memory", |b| {
        b.iter(|| {
            let threads: Vec<_> = (0..4)
                .map(|_| {
                    let c = collector.clone();
                    thread::spawn(move || {
                        for _ in 0..25 {
                            c.record_memory(1024 * 1024);
                        }
                    })
                })
                .collect();

            for t in threads {
                t.join().unwrap();
            }
        })
    });
}

/// Benchmark concurrent profile recording from multiple threads.
///
/// Target: Should scale with minimal contention (DashMap is optimized for this)
fn benchmark_concurrent_profile(c: &mut Criterion) {
    use std::sync::Arc;
    use std::thread;

    let collector = Arc::new(FullMetricsCollector::new());

    c.bench_function("full_collector_concurrent_profile", |b| {
        b.iter(|| {
            let threads: Vec<_> = (0..4)
                .map(|i| {
                    let c = collector.clone();
                    thread::spawn(move || {
                        let name = match i {
                            0 => "function_a",
                            1 => "function_b",
                            2 => "function_c",
                            _ => "function_d",
                        };
                        for _ in 0..25 {
                            c.record_profile(name, 250);
                        }
                    })
                })
                .collect();

            for t in threads {
                t.join().unwrap();
            }
        })
    });
}

/// Benchmark realistic workload: mixed operations.
///
/// Target: Overall throughput should support high-frequency operations
fn benchmark_realistic_workload(c: &mut Criterion) {
    let collector = FullMetricsCollector::new();

    c.bench_function("full_collector_realistic_workload", |b| {
        b.iter(|| {
            // Simulate processing 10 events with full metrics
            for i in 0..10 {
                // Record event processing
                collector.record_latency(Operation::EventProcess, black_box(150 + i * 10));

                // Record rule matching
                collector.record_latency(Operation::RuleMatch, black_box(50 + i * 5));

                // Record action execution
                collector.record_latency(Operation::ActionExecute, black_box(100 + i * 8));

                // Profile a function every 3 events
                if i % 3 == 0 {
                    let _guard = collector.start_profile("process_combo");
                }

                // Record memory every 5 events
                if i % 5 == 0 {
                    collector.record_memory(black_box(1024 * 1024 * (10 + i as usize)));
                }
            }
        })
    });
}

criterion_group!(
    benches,
    benchmark_record_latency_full,
    benchmark_record_latency_noop,
    benchmark_record_all_operations,
    benchmark_record_memory,
    benchmark_profile_guard,
    benchmark_record_profile,
    benchmark_multiple_profile_points,
    benchmark_snapshot,
    benchmark_reset,
    benchmark_concurrent_latency,
    benchmark_concurrent_memory,
    benchmark_concurrent_profile,
    benchmark_realistic_workload,
);
criterion_main!(benches);
