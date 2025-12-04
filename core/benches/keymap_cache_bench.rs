//! Keymap cache performance benchmarks.
//!
//! This benchmark suite validates the keymap cache performance improvements:
//! - Measures cache hit rate under realistic workloads
//! - Verifies latency improvements over uncached operations
//! - Tests concurrent access patterns

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use keyrx_core::drivers::common::cache::{KeymapCache, LruKeymapCache};
use keyrx_core::drivers::keycodes::KeyCode;
use std::sync::Arc;
use std::thread;

/// Benchmark cache hits vs. misses latency.
///
/// Measures the overhead of cache operations in the hot path.
fn benchmark_cache_hit_vs_miss(c: &mut Criterion) {
    let cache = LruKeymapCache::new(100).unwrap();

    // Pre-populate cache
    for i in 0..50 {
        cache.insert(i, "dev0", KeyCode::A);
    }

    let mut group = c.benchmark_group("cache_operations");

    group.bench_function("cache_hit", |b| {
        b.iter(|| {
            // Scan codes 0-49 are in cache
            black_box(cache.get(black_box(25), black_box("dev0")));
        })
    });

    group.bench_function("cache_miss", |b| {
        b.iter(|| {
            // Scan codes >= 50 are not in cache
            black_box(cache.get(black_box(99), black_box("dev0")));
        })
    });

    group.bench_function("cache_insert", |b| {
        let mut counter = 1000u32;
        b.iter(|| {
            counter = counter.wrapping_add(1);
            cache.insert(black_box(counter), black_box("dev0"), black_box(KeyCode::B));
        })
    });

    group.finish();
}

/// Benchmark cache hit rate under realistic workload.
///
/// Simulates typical key event patterns with temporal locality.
/// Target: > 90% hit rate for realistic key patterns.
fn benchmark_realistic_hit_rate(c: &mut Criterion) {
    let cache = LruKeymapCache::new(256).unwrap();

    c.bench_function("realistic_workload_with_locality", |b| {
        b.iter(|| {
            // Common keys typed repeatedly (high temporal locality)
            let common_keys = [30, 31, 32, 17, 18, 19, 45, 46, 47]; // a-s-d-w-e-r-x-c-v

            // Simulate 100 key events
            for _ in 0..100 {
                // 80% of events use common keys (Pareto principle)
                for _ in 0..80 {
                    let scan_code = common_keys[fastrand::usize(0..common_keys.len())];
                    if cache.get(scan_code, "keyboard0").is_none() {
                        cache.insert(scan_code, "keyboard0", KeyCode::A);
                    }
                }

                // 20% of events use random keys
                for _ in 0..20 {
                    let scan_code = fastrand::u32(0..120);
                    if cache.get(scan_code, "keyboard0").is_none() {
                        cache.insert(scan_code, "keyboard0", KeyCode::A);
                    }
                }
            }

            let stats = cache.stats();
            black_box(stats.hit_rate());
        })
    });
}

/// Benchmark cache with different capacity sizes.
///
/// Validates cache size vs. hit rate trade-offs.
fn benchmark_cache_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_size_scaling");

    for capacity in [16, 64, 256, 1024] {
        group.bench_with_input(
            BenchmarkId::from_parameter(capacity),
            &capacity,
            |b, &cap| {
                let cache = LruKeymapCache::new(cap).unwrap();

                b.iter(|| {
                    // Access pattern: 80% of traffic to 20% of keys
                    let hot_keys = cap / 5; // 20% of capacity

                    for _ in 0..100 {
                        let scan_code = if fastrand::u8(0..100) < 80 {
                            // 80% hot keys
                            fastrand::u32(0..hot_keys as u32)
                        } else {
                            // 20% cold keys
                            fastrand::u32(0..500)
                        };

                        if cache.get(scan_code, "dev0").is_none() {
                            cache.insert(scan_code, "dev0", KeyCode::A);
                        }
                    }
                })
            },
        );
    }

    group.finish();
}

/// Benchmark LRU eviction overhead.
///
/// Measures the cost of eviction when cache is at capacity.
fn benchmark_lru_eviction(c: &mut Criterion) {
    let cache = LruKeymapCache::new(64).unwrap();

    // Fill cache to capacity
    for i in 0..64 {
        cache.insert(i, "dev0", KeyCode::A);
    }

    c.bench_function("lru_eviction_at_capacity", |b| {
        let mut counter = 1000u32;
        b.iter(|| {
            counter = counter.wrapping_add(1);
            // This insert will trigger eviction
            cache.insert(black_box(counter), black_box("dev0"), black_box(KeyCode::B));
        })
    });
}

/// Benchmark device invalidation performance.
///
/// Measures cost of invalidating all entries for a device.
fn benchmark_device_invalidation(c: &mut Criterion) {
    let mut group = c.benchmark_group("device_invalidation");

    for num_devices in [1, 4, 16] {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_devices),
            &num_devices,
            |b, &num_devs| {
                b.iter_batched(
                    || {
                        let cache = LruKeymapCache::new(256).unwrap();
                        // Populate cache with entries from multiple devices
                        for dev_id in 0..num_devs {
                            for scan_code in 0..64 {
                                cache.insert(scan_code, &format!("dev{}", dev_id), KeyCode::A);
                            }
                        }
                        cache
                    },
                    |cache| {
                        // Invalidate one device
                        black_box(cache.invalidate_device("dev0"));
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );
    }

    group.finish();
}

/// Benchmark concurrent cache access from multiple threads.
///
/// Validates thread-safety overhead and contention characteristics.
fn benchmark_concurrent_access(c: &mut Criterion) {
    let cache = Arc::new(LruKeymapCache::new(256).unwrap());

    // Pre-populate with some data
    for i in 0..128 {
        cache.insert(i, "dev0", KeyCode::A);
    }

    let mut group = c.benchmark_group("concurrent_access");

    for num_threads in [2, 4, 8] {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_threads),
            &num_threads,
            |b, &threads| {
                b.iter(|| {
                    let handles: Vec<_> = (0..threads)
                        .map(|_| {
                            let cache_clone = cache.clone();
                            thread::spawn(move || {
                                // Each thread does mixed read/write operations
                                for _ in 0..25 {
                                    let scan_code = fastrand::u32(0..200);
                                    if cache_clone.get(scan_code, "dev0").is_none() {
                                        cache_clone.insert(scan_code, "dev0", KeyCode::A);
                                    }
                                }
                            })
                        })
                        .collect();

                    for handle in handles {
                        handle.join().unwrap();
                    }
                })
            },
        );
    }

    group.finish();
}

/// Benchmark cache stats collection overhead.
///
/// Verifies that stats tracking has minimal impact on hot path.
fn benchmark_stats_overhead(c: &mut Criterion) {
    let cache = LruKeymapCache::new(100).unwrap();

    // Pre-populate cache
    for i in 0..50 {
        cache.insert(i, "dev0", KeyCode::A);
    }

    c.bench_function("cache_with_stats_tracking", |b| {
        b.iter(|| {
            // Simulate 100 lookups with stats tracking
            for i in 0..100 {
                black_box(cache.get(black_box(i % 75), black_box("dev0")));
            }
            // Check stats periodically
            black_box(cache.stats());
        })
    });
}

/// Benchmark multi-device workload.
///
/// Simulates multiple input devices with device-specific caching.
fn benchmark_multi_device_workload(c: &mut Criterion) {
    let cache = LruKeymapCache::new(512).unwrap();

    c.bench_function("multi_device_workload", |b| {
        b.iter(|| {
            // Simulate 3 active devices
            let devices = ["keyboard0", "keyboard1", "numpad0"];

            for _ in 0..100 {
                let device = devices[fastrand::usize(0..devices.len())];
                let scan_code = fastrand::u32(0..120);

                if cache.get(scan_code, device).is_none() {
                    cache.insert(scan_code, device, KeyCode::A);
                }
            }
        })
    });
}

/// Benchmark cache clear operation.
///
/// Measures cost of clearing entire cache (infrequent operation).
fn benchmark_cache_clear(c: &mut Criterion) {
    c.bench_function("cache_clear", |b| {
        b.iter_batched(
            || {
                let cache = LruKeymapCache::new(256).unwrap();
                // Populate cache
                for i in 0..256 {
                    cache.insert(i, "dev0", KeyCode::A);
                }
                cache
            },
            |cache| {
                black_box(cache.clear());
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

/// Benchmark worst-case: high miss rate scenario.
///
/// Tests cache behavior when hit rate is low (many unique keys).
fn benchmark_worst_case_miss_rate(c: &mut Criterion) {
    let cache = LruKeymapCache::new(64).unwrap();

    c.bench_function("worst_case_low_hit_rate", |b| {
        let mut counter = 0u32;
        b.iter(|| {
            // Access 1000 unique scan codes with only 64 capacity
            // This creates high eviction and low hit rate
            for _ in 0..1000 {
                counter = counter.wrapping_add(1);
                if cache.get(counter, "dev0").is_none() {
                    cache.insert(counter, "dev0", KeyCode::A);
                }
            }
        })
    });
}

/// Benchmark: Compare cached vs uncached lookup simulation.
///
/// Simulates the latency difference between cached and uncached operations.
/// Uncached operation simulates syscall overhead (~1-5µs).
fn benchmark_cached_vs_uncached_latency(c: &mut Criterion) {
    let cache = LruKeymapCache::new(100).unwrap();

    // Pre-populate cache
    for i in 0..50 {
        cache.insert(i, "dev0", KeyCode::A);
    }

    let mut group = c.benchmark_group("cached_vs_uncached");

    group.bench_function("cached_lookup", |b| {
        b.iter(|| {
            // Fast cache lookup
            black_box(cache.get(black_box(25), black_box("dev0")));
        })
    });

    group.bench_function("simulated_uncached_lookup", |b| {
        b.iter(|| {
            // Simulate syscall overhead with sleep
            // Real evdev/MapVirtualKey calls take 1-5µs
            std::thread::sleep(std::time::Duration::from_nanos(1000)); // 1µs
            black_box(KeyCode::A);
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_cache_hit_vs_miss,
    benchmark_realistic_hit_rate,
    benchmark_cache_sizes,
    benchmark_lru_eviction,
    benchmark_device_invalidation,
    benchmark_concurrent_access,
    benchmark_stats_overhead,
    benchmark_multi_device_workload,
    benchmark_cache_clear,
    benchmark_worst_case_miss_rate,
    benchmark_cached_vs_uncached_latency,
);
criterion_main!(benches);
