//! Integration tests for latency histogram accuracy.
//!
//! These tests verify the accuracy and correctness of latency percentile
//! calculations under various scenarios.

use keyrx_core::metrics::latency::LatencyHistogram;
use std::sync::Arc;
use std::thread;

#[test]
fn test_percentile_accuracy_with_known_distribution() {
    let histogram = LatencyHistogram::new(0);

    // Record 1000 evenly distributed samples from 1us to 1000us
    for i in 1..=1000 {
        histogram.record(i);
    }

    // Verify percentiles are accurate within HDR histogram precision (~1%)
    let p50 = histogram.percentile(50.0);
    assert!(p50 >= 490 && p50 <= 510, "p50={} should be ~500", p50);

    let p90 = histogram.percentile(90.0);
    assert!(p90 >= 890 && p90 <= 910, "p90={} should be ~900", p90);

    let p95 = histogram.percentile(95.0);
    assert!(p95 >= 940 && p95 <= 960, "p95={} should be ~950", p95);

    let p99 = histogram.percentile(99.0);
    assert!(p99 >= 980 && p99 <= 1000, "p99={} should be ~990", p99);

    let p999 = histogram.percentile(99.9);
    assert!(p999 >= 990 && p999 <= 1000, "p99.9={} should be ~999", p999);
}

#[test]
fn test_percentiles_with_skewed_distribution() {
    let histogram = LatencyHistogram::new(0);

    // Simulate real-world latency distribution:
    // - 90% fast operations (10-100us)
    // - 9% medium operations (100-1000us)
    // - 1% slow operations (1000-10000us)

    // 900 fast samples
    for _ in 0..900 {
        histogram.record(50); // Typical fast value
    }

    // 90 medium samples
    for _ in 0..90 {
        histogram.record(500); // Typical medium value
    }

    // 10 slow samples
    for _ in 0..10 {
        histogram.record(5000); // Typical slow value
    }

    // p50 should be in the fast range (50% of 1000 = sample 500, which is in the fast group)
    let p50 = histogram.percentile(50.0);
    assert!(p50 < 100, "p50={} should be in fast range", p50);

    // p90 should still be fast (90% of 1000 = sample 900, which is in the fast group)
    let p90 = histogram.percentile(90.0);
    assert!(p90 < 100, "p90={} should be in fast range", p90);

    // p95 should be in the medium range (95% of 1000 = sample 950, in medium group)
    let p95 = histogram.percentile(95.0);
    assert!(
        p95 >= 400 && p95 <= 600,
        "p95={} should be in medium range",
        p95
    );

    // p99.5 should be in the slow range (99.5% of 1000 = sample 995, in slow group)
    let p995 = histogram.percentile(99.5);
    assert!(p995 > 1000, "p99.5={} should be in slow range", p995);

    // Verify total count
    assert_eq!(histogram.count(), 1000);
}

#[test]
fn test_extreme_outliers_dont_skew_percentiles() {
    let histogram = LatencyHistogram::new(0);

    // Record 999 samples at 100us
    for _ in 0..999 {
        histogram.record(100);
    }

    // Record 1 extreme outlier at 1 hour (max value)
    histogram.record(3_600_000_000);

    // p50, p90, p95, p99 should all be close to 100us
    assert_eq!(histogram.percentile(50.0), 100);
    assert_eq!(histogram.percentile(90.0), 100);
    assert_eq!(histogram.percentile(95.0), 100);
    assert_eq!(histogram.percentile(99.0), 100);

    // Only p99.9 should capture the outlier
    let p999 = histogram.percentile(99.9);
    assert!(p999 > 1_000_000, "p99.9 should capture outlier");

    // Mean should be affected by outlier
    let mean = histogram.mean();
    assert!(mean > 100.0, "mean={} should be > 100 due to outlier", mean);
}

#[test]
fn test_small_sample_sizes() {
    let histogram = LatencyHistogram::new(0);

    // Single sample
    histogram.record(50);
    assert_eq!(histogram.percentile(50.0), 50);
    assert_eq!(histogram.percentile(95.0), 50);
    assert_eq!(histogram.percentile(99.0), 50);

    // Two samples
    histogram.reset();
    histogram.record(50);
    histogram.record(150);
    let p50 = histogram.percentile(50.0);
    assert!(p50 >= 50 && p50 <= 150);
}

#[test]
fn test_statistics_accuracy() {
    let histogram = LatencyHistogram::new(0);

    // Record samples: 100, 200, 300, 400, 500
    for i in 1..=5 {
        histogram.record(i * 100);
    }

    // Verify min/max
    assert_eq!(histogram.min(), 100);
    assert_eq!(histogram.max(), 500);

    // Verify mean is ~300
    let mean = histogram.mean();
    assert!(
        mean >= 290.0 && mean <= 310.0,
        "mean={} should be ~300",
        mean
    );

    // Verify count
    assert_eq!(histogram.count(), 5);

    // Verify stddev is reasonable (should be ~141 for this distribution)
    let stddev = histogram.stddev();
    assert!(stddev > 100.0 && stddev < 200.0, "stddev={}", stddev);
}

#[test]
fn test_reset_clears_all_data() {
    let histogram = LatencyHistogram::new(0);

    // Record some data
    for i in 1..=100 {
        histogram.record(i * 10);
    }

    assert_eq!(histogram.count(), 100);
    assert!(histogram.percentile(50.0) > 0);

    // Reset
    histogram.reset();

    // Verify everything is cleared
    assert_eq!(histogram.count(), 0);
    assert_eq!(histogram.percentile(50.0), 0);
    assert_eq!(histogram.min(), 0);
    assert_eq!(histogram.max(), 0);
    assert_eq!(histogram.mean(), 0.0);
}

#[test]
fn test_concurrent_recording_accuracy() {
    let histogram = Arc::new(LatencyHistogram::new(0));
    let mut handles = vec![];

    // 10 threads each recording 100 samples (1-100us)
    for _ in 0..10 {
        let hist = Arc::clone(&histogram);
        let handle = thread::spawn(move || {
            for i in 1..=100 {
                hist.record(i);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Should have 1000 total samples
    assert_eq!(histogram.count(), 1000);

    // p50 should be around 50
    let p50 = histogram.percentile(50.0);
    assert!(p50 >= 40 && p50 <= 60, "p50={}", p50);

    // Min should be 1, max should be 100
    assert_eq!(histogram.min(), 1);
    assert!(histogram.max() >= 95 && histogram.max() <= 100);
}

#[test]
fn test_boundary_values() {
    let histogram = LatencyHistogram::new(0);

    // Test minimum boundary (1 microsecond)
    histogram.record(1);
    assert_eq!(histogram.min(), 1);
    assert_eq!(histogram.max(), 1);

    // Test very large value (close to max)
    histogram.reset();
    histogram.record(3_600_000_000); // 1 hour
    assert!(histogram.max() > 1_000_000_000);

    // Test values beyond max (should be saturated)
    histogram.reset();
    histogram.record(u64::MAX); // Should be clamped to histogram max
    assert_eq!(histogram.count(), 1);
}

#[test]
fn test_many_identical_samples() {
    let histogram = LatencyHistogram::new(0);

    // Record 10000 identical samples
    for _ in 0..10000 {
        histogram.record(500);
    }

    // All percentiles should be the same value
    assert_eq!(histogram.percentile(0.0), 500);
    assert_eq!(histogram.percentile(50.0), 500);
    assert_eq!(histogram.percentile(99.0), 500);
    assert_eq!(histogram.percentile(100.0), 500);

    assert_eq!(histogram.min(), 500);
    assert_eq!(histogram.max(), 500);
    assert_eq!(histogram.mean(), 500.0);
    assert_eq!(histogram.stddev(), 0.0); // No variance
}

#[test]
fn test_bimodal_distribution() {
    let histogram = LatencyHistogram::new(0);

    // Simulate bimodal distribution (e.g., cached vs uncached)
    // 500 fast samples at ~50us
    for _ in 0..500 {
        histogram.record(50);
    }

    // 500 slow samples at ~500us
    for _ in 0..500 {
        histogram.record(500);
    }

    // p25 should be in fast mode
    let p25 = histogram.percentile(25.0);
    assert!(p25 < 100, "p25={} should be in fast mode", p25);

    // p75 should be in slow mode
    let p75 = histogram.percentile(75.0);
    assert!(p75 > 400, "p75={} should be in slow mode", p75);

    // p50 should be somewhere in between or at one of the modes
    let p50 = histogram.percentile(50.0);
    assert!(p50 >= 50 && p50 <= 500, "p50={}", p50);

    // Mean should be around 275 (average of the two modes)
    let mean = histogram.mean();
    assert!(
        mean >= 250.0 && mean <= 300.0,
        "mean={} should be ~275",
        mean
    );
}
