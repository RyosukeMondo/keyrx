//! Integration tests for memory tracking accuracy and leak detection.
//!
//! These tests verify the correctness of memory monitoring, baseline tracking,
//! peak detection, and leak detection heuristics.

use keyrx_core::metrics::memory::MemoryMonitor;
use std::sync::Arc;
use std::thread;

#[test]
fn test_baseline_captured_on_first_sample() {
    let monitor = MemoryMonitor::new();
    let stats = monitor.stats();
    assert_eq!(stats.baseline, 0);

    monitor.record(1024 * 1024); // 1 MB

    let stats = monitor.stats();
    assert_eq!(stats.baseline, 1024 * 1024);
    assert_eq!(stats.current, 1024 * 1024);
    assert_eq!(stats.peak, 1024 * 1024);
    assert_eq!(stats.growth_from_baseline, 0);
}

#[test]
fn test_peak_tracking() {
    let monitor = MemoryMonitor::new();

    // Record increasing values
    monitor.record(1024);
    monitor.record(2048);
    monitor.record(4096);
    monitor.record(8192);

    let stats = monitor.stats();
    assert_eq!(stats.peak, 8192);
    assert_eq!(stats.current, 8192);

    // Record a lower value - peak should remain
    monitor.record(2048);
    let stats = monitor.stats();
    assert_eq!(stats.peak, 8192);
    assert_eq!(stats.current, 2048);
}

#[test]
fn test_growth_from_baseline() {
    let monitor = MemoryMonitor::new();

    monitor.record(1024); // Baseline
    let stats = monitor.stats();
    assert_eq!(stats.growth_from_baseline, 0);

    monitor.record(2048);
    let stats = monitor.stats();
    assert_eq!(stats.growth_from_baseline, 1024);

    monitor.record(4096);
    let stats = monitor.stats();
    assert_eq!(stats.growth_from_baseline, 3072);

    // Decrease below baseline
    monitor.record(512);
    let stats = monitor.stats();
    assert_eq!(stats.growth_from_baseline, 0); // Saturating sub
}

#[test]
fn test_no_leak_with_stable_memory() {
    let monitor = MemoryMonitor::new();

    // Record stable memory for many samples
    for _ in 0..100 {
        monitor.record(1024 * 1024);
    }

    assert!(!monitor.has_potential_leak());
}

#[test]
fn test_no_leak_with_fluctuating_memory() {
    let monitor = MemoryMonitor::new();

    // Simulate realistic memory patterns with fluctuation
    for i in 0..100 {
        let base = 1024 * 1024;
        let variation = if i % 2 == 0 {
            100 * 1024
        } else {
            -50 * 1024_i64
        };
        let mem = (base as i64 + variation) as usize;
        monitor.record(mem);
    }

    assert!(!monitor.has_potential_leak());
}

#[test]
fn test_no_leak_with_periodic_spikes() {
    let monitor = MemoryMonitor::new();

    // Simulate periodic GC-like patterns
    for i in 0..100 {
        let mem = if i % 10 == 0 {
            2048 * 1024 // Spike every 10 samples
        } else {
            1024 * 1024 // Normal
        };
        monitor.record(mem);
    }

    assert!(!monitor.has_potential_leak());
}

#[test]
fn test_leak_detected_with_monotonic_growth() {
    let monitor = MemoryMonitor::new();

    // Record continuously growing memory above threshold
    // Leak threshold is 10KB per sample for 20 consecutive samples
    for i in 0..30 {
        let mem = 1024 * 1024 + (i * 15 * 1024); // Growing by 15KB each
        monitor.record(mem);
    }

    assert!(monitor.has_potential_leak());
}

#[test]
fn test_no_leak_with_growth_below_threshold() {
    let monitor = MemoryMonitor::new();

    // Record slow growth below leak threshold (10KB/sample)
    for i in 0..30 {
        let mem = 1024 * 1024 + (i * 5 * 1024); // Growing by 5KB each
        monitor.record(mem);
    }

    assert!(!monitor.has_potential_leak());
}

#[test]
fn test_leak_detection_requires_consecutive_growth() {
    let monitor = MemoryMonitor::new();

    // Record growth interrupted by decreases
    for i in 0..30 {
        let mem = if i % 5 == 0 {
            1024 * 1024 // Reset every 5 samples
        } else {
            1024 * 1024 + (i * 15 * 1024)
        };
        monitor.record(mem);
    }

    // Should not detect leak because growth is not consecutive
    assert!(!monitor.has_potential_leak());
}

#[test]
fn test_leak_detection_needs_enough_samples() {
    let monitor = MemoryMonitor::new();

    // Record only a few samples with high growth
    for i in 0..10 {
        let mem = 1024 * 1024 + (i * 50 * 1024);
        monitor.record(mem);
    }

    // Should not flag leak yet - not enough samples
    assert!(!monitor.has_potential_leak());
}

#[test]
fn test_reset_clears_all_metrics() {
    let monitor = MemoryMonitor::new();

    // Record some data
    for i in 0..50 {
        monitor.record(1024 * (i + 1));
    }

    let stats = monitor.stats();
    assert!(stats.current > 0);
    assert!(stats.peak > 0);
    assert!(stats.sample_count > 0);

    // Reset
    monitor.reset();

    let stats = monitor.stats();
    assert_eq!(stats.current, 0);
    assert_eq!(stats.peak, 0);
    assert_eq!(stats.baseline, 0);
    assert_eq!(stats.sample_count, 0);
    assert_eq!(stats.growth_from_baseline, 0);

    // Next record should set new baseline
    monitor.record(2048);
    let stats = monitor.stats();
    assert_eq!(stats.baseline, 2048);
}

#[test]
fn test_ring_buffer_wrapping() {
    let monitor = MemoryMonitor::new();

    // Record more than buffer size (100) to force wrapping
    for i in 0..150 {
        monitor.record(1024 * (i + 1));
    }

    let stats = monitor.stats();
    assert_eq!(stats.sample_count, 150);
    assert_eq!(stats.current, 1024 * 150);

    // Ring buffer should wrap, but stats should be correct
    assert!(stats.peak >= 1024 * 150);
}

#[test]
fn test_concurrent_recording() {
    let monitor = Arc::new(MemoryMonitor::new());
    let mut handles = vec![];

    // 10 threads each recording 100 samples
    for thread_id in 0..10 {
        let m = Arc::clone(&monitor);
        let handle = thread::spawn(move || {
            for i in 0..100 {
                m.record(1024 * (thread_id * 100 + i + 1));
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    let stats = monitor.stats();
    assert_eq!(stats.sample_count, 1000);
    assert!(stats.peak > 0);
    assert!(stats.current > 0);
}

#[test]
fn test_zero_memory_recording() {
    let monitor = MemoryMonitor::new();

    monitor.record(0);
    let stats = monitor.stats();
    assert_eq!(stats.current, 0);
    assert_eq!(stats.peak, 0);
    assert_eq!(stats.baseline, 0);
}

#[test]
fn test_large_memory_values() {
    let monitor = MemoryMonitor::new();

    // Record very large values (multi-GB)
    let large_value = 10 * 1024 * 1024 * 1024; // 10 GB
    monitor.record(large_value);

    let stats = monitor.stats();
    assert_eq!(stats.current, large_value);
    assert_eq!(stats.peak, large_value);
}

#[test]
fn test_memory_decrease_doesnt_affect_peak() {
    let monitor = MemoryMonitor::new();

    monitor.record(5 * 1024 * 1024); // 5 MB
    monitor.record(10 * 1024 * 1024); // 10 MB - peak
    monitor.record(3 * 1024 * 1024); // 3 MB - decrease

    let stats = monitor.stats();
    assert_eq!(stats.current, 3 * 1024 * 1024);
    assert_eq!(stats.peak, 10 * 1024 * 1024);
}

#[test]
fn test_sample_count_accuracy() {
    let monitor = MemoryMonitor::new();

    for i in 1..=1000 {
        monitor.record(i * 1024);
    }

    let stats = monitor.stats();
    assert_eq!(stats.sample_count, 1000);
}

#[test]
fn test_leak_detection_after_reset() {
    let monitor = MemoryMonitor::new();

    // Create a leak pattern
    for i in 0..30 {
        monitor.record(1024 * 1024 + (i * 15 * 1024));
    }
    assert!(monitor.has_potential_leak());

    // Reset
    monitor.reset();
    assert!(!monitor.has_potential_leak());

    // Record stable memory - should not leak
    for _ in 0..30 {
        monitor.record(1024 * 1024);
    }
    assert!(!monitor.has_potential_leak());
}

#[test]
fn test_gradual_growth_then_stabilization() {
    let monitor = MemoryMonitor::new();

    // Gradual growth phase
    for i in 0..10 {
        monitor.record(1024 * 1024 + (i * 20 * 1024));
    }

    // Stabilization phase
    for _ in 0..50 {
        monitor.record(1024 * 1024 + (10 * 20 * 1024));
    }

    // Should not detect leak because recent samples are stable
    assert!(!monitor.has_potential_leak());
}

#[test]
fn test_memory_stats_equality() {
    use keyrx_core::metrics::memory::MemoryStats;

    let stats1 = MemoryStats {
        current: 1024,
        peak: 2048,
        baseline: 512,
        growth_from_baseline: 512,
        sample_count: 10,
    };

    let stats2 = MemoryStats {
        current: 1024,
        peak: 2048,
        baseline: 512,
        growth_from_baseline: 512,
        sample_count: 10,
    };

    assert_eq!(stats1, stats2);
}

#[test]
fn test_rapid_growth_detection() {
    let monitor = MemoryMonitor::new();

    // Very rapid growth - 100KB per sample
    for i in 0..25 {
        let mem = 1024 * 1024 + (i * 100 * 1024);
        monitor.record(mem);
    }

    // Should definitely detect this as a potential leak
    assert!(monitor.has_potential_leak());
}
