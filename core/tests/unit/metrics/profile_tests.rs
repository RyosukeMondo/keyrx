//! Integration tests for profile points and hot spot identification.
//!
//! These tests verify the correctness of function-level profiling, statistics
//! aggregation, and hot spot detection.

use keyrx_core::metrics::profile::ProfilePoints;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[test]
fn test_manual_recording_accuracy() {
    let profiler = ProfilePoints::new();

    profiler.record("func_a", 100);
    profiler.record("func_a", 200);
    profiler.record("func_a", 300);

    let stats = profiler.get_stats("func_a");
    assert_eq!(stats.count, 3);
    assert_eq!(stats.total_micros, 600);
    assert_eq!(stats.avg_micros, 200);
    assert_eq!(stats.min_micros, 100);
    assert_eq!(stats.max_micros, 300);
}

#[test]
fn test_guard_automatic_timing() {
    let profiler = ProfilePoints::new();

    {
        let _guard = profiler.start("sleep_function");
        thread::sleep(Duration::from_micros(500));
    }

    let stats = profiler.get_stats("sleep_function");
    assert_eq!(stats.count, 1);
    // Sleep timing is approximate, verify it's in reasonable range
    assert!(stats.total_micros >= 400 && stats.total_micros < 5000);
}

#[test]
fn test_multiple_guards_same_function() {
    let profiler = ProfilePoints::new();

    // Simulate multiple calls to the same function
    for _ in 0..5 {
        let _guard = profiler.start("repeated_function");
        thread::sleep(Duration::from_micros(100));
    }

    let stats = profiler.get_stats("repeated_function");
    assert_eq!(stats.count, 5);
    assert!(stats.total_micros >= 400); // At least 500us total
}

#[test]
fn test_multiple_different_functions() {
    let profiler = ProfilePoints::new();

    profiler.record("fast_func", 10);
    profiler.record("medium_func", 100);
    profiler.record("slow_func", 1000);

    assert_eq!(profiler.count(), 3);

    let fast_stats = profiler.get_stats("fast_func");
    let medium_stats = profiler.get_stats("medium_func");
    let slow_stats = profiler.get_stats("slow_func");

    assert_eq!(fast_stats.total_micros, 10);
    assert_eq!(medium_stats.total_micros, 100);
    assert_eq!(slow_stats.total_micros, 1000);
}

#[test]
fn test_hot_spots_ordering() {
    let profiler = ProfilePoints::new();

    // Record different functions with varying total times
    profiler.record("func_c", 500); // Total: 500
    profiler.record("func_a", 1000); // Total: 2000
    profiler.record("func_a", 1000);
    profiler.record("func_b", 300); // Total: 900
    profiler.record("func_b", 300);
    profiler.record("func_b", 300);
    profiler.record("func_d", 100); // Total: 100

    let hot_spots = profiler.hot_spots(4);

    // Verify ordering by total time (descending)
    assert_eq!(hot_spots[0].0, "func_a"); // 2000us
    assert_eq!(hot_spots[0].1.total_micros, 2000);

    assert_eq!(hot_spots[1].0, "func_b"); // 900us
    assert_eq!(hot_spots[1].1.total_micros, 900);

    assert_eq!(hot_spots[2].0, "func_c"); // 500us
    assert_eq!(hot_spots[2].1.total_micros, 500);

    assert_eq!(hot_spots[3].0, "func_d"); // 100us
    assert_eq!(hot_spots[3].1.total_micros, 100);
}

#[test]
fn test_hot_spots_limiting() {
    let profiler = ProfilePoints::new();

    // Record many functions
    for i in 1..=10 {
        profiler.record("func_1", i * 100);
        profiler.record("func_2", i * 200);
        profiler.record("func_3", i * 300);
        profiler.record("func_4", i * 400);
        profiler.record("func_5", i * 500);
    }

    // Request only top 3
    let hot_spots = profiler.hot_spots(3);
    assert_eq!(hot_spots.len(), 3);

    // Top 3 should be func_5, func_4, func_3
    assert_eq!(hot_spots[0].0, "func_5");
    assert_eq!(hot_spots[1].0, "func_4");
    assert_eq!(hot_spots[2].0, "func_3");
}

#[test]
fn test_all_stats() {
    let profiler = ProfilePoints::new();

    profiler.record("func_a", 100);
    profiler.record("func_b", 200);
    profiler.record("func_c", 300);

    let all = profiler.all_stats();
    assert_eq!(all.len(), 3);

    let total: u64 = all.iter().map(|(_, s)| s.total_micros).sum();
    assert_eq!(total, 600);
}

#[test]
fn test_empty_profiler() {
    let profiler = ProfilePoints::new();

    assert_eq!(profiler.count(), 0);

    let stats = profiler.get_stats("nonexistent");
    assert_eq!(stats.count, 0);
    assert_eq!(stats.total_micros, 0);
    assert_eq!(stats.avg_micros, 0);
    assert_eq!(stats.min_micros, 0);
    assert_eq!(stats.max_micros, 0);

    let hot_spots = profiler.hot_spots(10);
    assert_eq!(hot_spots.len(), 0);

    let all = profiler.all_stats();
    assert_eq!(all.len(), 0);
}

#[test]
fn test_reset_clears_all_data() {
    let profiler = ProfilePoints::new();

    profiler.record("func_a", 100);
    profiler.record("func_b", 200);
    profiler.record("func_c", 300);

    assert_eq!(profiler.count(), 3);

    profiler.reset();

    assert_eq!(profiler.count(), 0);
    let stats = profiler.get_stats("func_a");
    assert_eq!(stats.count, 0);
}

#[test]
fn test_min_max_tracking() {
    let profiler = ProfilePoints::new();

    profiler.record("variable_func", 50);
    profiler.record("variable_func", 500);
    profiler.record("variable_func", 5000);
    profiler.record("variable_func", 100);
    profiler.record("variable_func", 1000);

    let stats = profiler.get_stats("variable_func");
    assert_eq!(stats.count, 5);
    assert_eq!(stats.min_micros, 50);
    assert_eq!(stats.max_micros, 5000);
    assert_eq!(stats.total_micros, 6650);
    assert_eq!(stats.avg_micros, 1330);
}

#[test]
fn test_concurrent_recording_same_function() {
    let profiler = Arc::new(ProfilePoints::new());
    let mut handles = vec![];

    // 10 threads recording to the same function
    for _ in 0..10 {
        let p = Arc::clone(&profiler);
        let handle = thread::spawn(move || {
            for i in 1..=100 {
                p.record("shared_func", i);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    let stats = profiler.get_stats("shared_func");
    assert_eq!(stats.count, 1000);
    // Each thread records 1+2+...+100 = 5050
    // 10 threads = 50500 total
    assert_eq!(stats.total_micros, 50500);
}

#[test]
fn test_concurrent_recording_different_functions() {
    let profiler = Arc::new(ProfilePoints::new());
    let mut handles = vec![];

    // 5 threads, each recording to its own function
    for thread_id in 0..5 {
        let p = Arc::clone(&profiler);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                p.record("func_1", (thread_id + 1) as u64 * 100);
                p.record("func_2", (thread_id + 1) as u64 * 200);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Should have multiple functions
    assert!(profiler.count() >= 2);

    // Each function should have been called from all threads
    let stats1 = profiler.get_stats("func_1");
    let stats2 = profiler.get_stats("func_2");
    assert_eq!(stats1.count, 500); // 5 threads * 100 iterations
    assert_eq!(stats2.count, 500);
}

#[test]
fn test_average_calculation() {
    let profiler = ProfilePoints::new();

    // Record 5 values: 100, 200, 300, 400, 500 (avg = 300)
    for i in 1..=5 {
        profiler.record("avg_test", i * 100);
    }

    let stats = profiler.get_stats("avg_test");
    assert_eq!(stats.count, 5);
    assert_eq!(stats.total_micros, 1500);
    assert_eq!(stats.avg_micros, 300);
}

#[test]
fn test_nested_profiling() {
    let profiler = ProfilePoints::new();

    {
        let _outer = profiler.start("outer_function");
        thread::sleep(Duration::from_micros(100));

        {
            let _inner = profiler.start("inner_function");
            thread::sleep(Duration::from_micros(50));
        }

        thread::sleep(Duration::from_micros(100));
    }

    let outer_stats = profiler.get_stats("outer_function");
    let inner_stats = profiler.get_stats("inner_function");

    assert_eq!(outer_stats.count, 1);
    assert_eq!(inner_stats.count, 1);

    // Outer should be >= inner (plus overhead)
    assert!(outer_stats.total_micros >= inner_stats.total_micros);

    // Verify reasonable timing
    assert!(inner_stats.total_micros >= 40 && inner_stats.total_micros < 5000);
    assert!(outer_stats.total_micros >= 200 && outer_stats.total_micros < 10000);
}

#[test]
fn test_zero_overhead_verification() {
    let profiler = ProfilePoints::new();

    // Record many very fast operations
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        profiler.record("fast_op", 1);
    }
    let elapsed = start.elapsed();

    // 1000 recordings should complete in well under 10ms
    assert!(elapsed.as_millis() < 10, "Took too long: {:?}", elapsed);

    let stats = profiler.get_stats("fast_op");
    assert_eq!(stats.count, 1000);
}

#[test]
fn test_large_timing_values() {
    let profiler = ProfilePoints::new();

    // Record very large timing values (seconds)
    profiler.record("slow_op", 1_000_000); // 1 second
    profiler.record("slow_op", 5_000_000); // 5 seconds

    let stats = profiler.get_stats("slow_op");
    assert_eq!(stats.count, 2);
    assert_eq!(stats.total_micros, 6_000_000);
    assert_eq!(stats.avg_micros, 3_000_000);
    assert_eq!(stats.min_micros, 1_000_000);
    assert_eq!(stats.max_micros, 5_000_000);
}

#[test]
fn test_hot_spots_with_equal_times() {
    let profiler = ProfilePoints::new();

    // Record multiple functions with same total time
    profiler.record("func_a", 1000);
    profiler.record("func_b", 1000);
    profiler.record("func_c", 1000);

    let hot_spots = profiler.hot_spots(10);
    assert_eq!(hot_spots.len(), 3);

    // All should have same total time
    for (_, stats) in &hot_spots {
        assert_eq!(stats.total_micros, 1000);
    }
}

#[test]
fn test_guard_drops_on_panic_recovery() {
    let profiler = Arc::new(ProfilePoints::new());

    // Simulate panic and recovery using AssertUnwindSafe
    let profiler_clone = Arc::clone(&profiler);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard = profiler_clone.start("panic_test");
        panic!("Test panic");
    }));

    assert!(result.is_err());

    // Guard should have dropped and recorded timing despite panic
    let stats = profiler.get_stats("panic_test");
    assert_eq!(stats.count, 1);
}

#[test]
fn test_realistic_workload_simulation() {
    let profiler = ProfilePoints::new();

    // Simulate realistic application workload
    for _ in 0..100 {
        profiler.record("event_process", 150);
        profiler.record("rule_match", 50);
        profiler.record("action_execute", 200);
    }

    // Add some outliers
    profiler.record("event_process", 5000);
    profiler.record("action_execute", 10000);

    let hot_spots = profiler.hot_spots(3);

    // action_execute should be hottest (20000 + 200*100 = 30000)
    assert_eq!(hot_spots[0].0, "action_execute");
    assert!(hot_spots[0].1.total_micros > 25000);

    // event_process should be second (15000 + 150*100 = 20000)
    assert_eq!(hot_spots[1].0, "event_process");

    // rule_match should be third (5000)
    assert_eq!(hot_spots[2].0, "rule_match");
}
