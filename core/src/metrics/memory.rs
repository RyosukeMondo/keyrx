//! Memory monitoring with leak detection.
//!
//! This module provides memory usage tracking with bounded sampling and leak
//! detection heuristics. All operations use atomic primitives for thread safety
//! without locks.
//!
//! # Architecture
//!
//! Memory monitoring tracks three key metrics:
//! - **Current**: Latest sampled memory usage
//! - **Peak**: Maximum observed memory usage
//! - **Baseline**: Initial memory usage at startup
//!
//! # Leak Detection
//!
//! The leak detector uses a simple heuristic:
//! - If memory grows monotonically over a threshold period
//! - And growth exceeds a configurable rate
//! - Then a potential leak is flagged
//!
//! # Performance
//!
//! - All operations are lock-free using atomics
//! - Sample buffer is bounded (ring buffer)
//! - Memory overhead is O(1) regardless of runtime
//!
//! # Example
//!
//! ```ignore
//! use keyrx_core::metrics::memory::MemoryMonitor;
//!
//! let monitor = MemoryMonitor::new();
//!
//! // Record current memory usage
//! monitor.record(1024 * 1024); // 1 MB
//!
//! // Check for leaks
//! if monitor.has_potential_leak() {
//!     eprintln!("Warning: Potential memory leak detected");
//! }
//!
//! // Get statistics
//! let stats = monitor.stats();
//! println!("Current: {} bytes", stats.current);
//! println!("Peak: {} bytes", stats.peak);
//! println!("Growth: {} bytes", stats.growth_from_baseline);
//! ```

use std::sync::atomic::{AtomicUsize, Ordering};

/// Maximum number of samples to keep for leak detection.
///
/// This bounds memory usage to a fixed size. A ring buffer of 100 samples
/// provides enough history for leak detection while keeping overhead minimal.
const MAX_SAMPLES: usize = 100;

/// Minimum growth rate (bytes per sample) to consider a potential leak.
///
/// If memory grows by more than 10KB per sample consistently, flag as potential leak.
const LEAK_GROWTH_THRESHOLD: usize = 10 * 1024;

/// Number of consecutive growth samples needed to flag a leak.
///
/// Requiring 20 consecutive samples (20% of buffer) reduces false positives
/// from temporary allocations.
const LEAK_CONSECUTIVE_THRESHOLD: usize = 20;

/// Memory usage monitor with leak detection.
///
/// Tracks current, peak, and baseline memory usage using lock-free atomics.
/// Maintains a bounded sample buffer for leak detection heuristics.
///
/// # Thread Safety
///
/// All methods are safe to call concurrently from multiple threads.
/// Uses `Ordering::Relaxed` for performance since exact ordering isn't
/// critical for metrics.
pub struct MemoryMonitor {
    /// Current memory usage in bytes.
    current: AtomicUsize,

    /// Peak memory usage in bytes.
    peak: AtomicUsize,

    /// Baseline memory usage (first sample) in bytes.
    baseline: AtomicUsize,

    /// Ring buffer of recent samples for leak detection.
    ///
    /// Each sample is a memory usage value in bytes.
    /// We use a fixed-size array to avoid allocations.
    samples: [AtomicUsize; MAX_SAMPLES],

    /// Current write position in the ring buffer.
    sample_index: AtomicUsize,

    /// Total number of samples recorded (may exceed MAX_SAMPLES).
    sample_count: AtomicUsize,
}

impl MemoryMonitor {
    /// Create a new memory monitor.
    ///
    /// All metrics are initialized to zero. The baseline will be set
    /// on the first call to `record()`.
    pub fn new() -> Self {
        Self {
            current: AtomicUsize::new(0),
            peak: AtomicUsize::new(0),
            baseline: AtomicUsize::new(0),
            samples: std::array::from_fn(|_| AtomicUsize::new(0)),
            sample_index: AtomicUsize::new(0),
            sample_count: AtomicUsize::new(0),
        }
    }

    /// Record a memory usage sample.
    ///
    /// Updates current, peak, and baseline metrics. Adds sample to
    /// ring buffer for leak detection.
    ///
    /// # Arguments
    ///
    /// * `bytes` - Current memory usage in bytes
    ///
    /// # Performance
    ///
    /// This method is designed for low overhead:
    /// - 4 atomic stores (relaxed ordering)
    /// - No allocations
    /// - No locks
    /// - Typical overhead: < 100ns
    pub fn record(&self, bytes: usize) {
        // Update current
        self.current.store(bytes, Ordering::Relaxed);

        // Update peak (fetch_max would be ideal but not stable)
        let mut current_peak = self.peak.load(Ordering::Relaxed);
        while bytes > current_peak {
            match self.peak.compare_exchange_weak(
                current_peak,
                bytes,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current_peak = actual,
            }
        }

        // Set baseline on first sample
        let count = self.sample_count.load(Ordering::Relaxed);
        if count == 0 {
            self.baseline.store(bytes, Ordering::Relaxed);
        }

        // Add to ring buffer
        let index = self.sample_index.fetch_add(1, Ordering::Relaxed) % MAX_SAMPLES;
        self.samples[index].store(bytes, Ordering::Relaxed);

        // Increment total count
        self.sample_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Check if there's a potential memory leak.
    ///
    /// Uses a simple heuristic: if memory has grown consistently
    /// for LEAK_CONSECUTIVE_THRESHOLD samples, flag as potential leak.
    ///
    /// # Returns
    ///
    /// `true` if a potential leak is detected, `false` otherwise.
    ///
    /// # Algorithm
    ///
    /// 1. Check if we have enough samples
    /// 2. Walk backwards through the ring buffer
    /// 3. Count consecutive samples where memory increased
    /// 4. If count exceeds threshold, return true
    ///
    /// # Performance
    ///
    /// - O(n) where n = min(sample_count, MAX_SAMPLES)
    /// - Typically < 1us for 100 samples
    /// - No allocations
    pub fn has_potential_leak(&self) -> bool {
        let count = self.sample_count.load(Ordering::Relaxed);
        if count < LEAK_CONSECUTIVE_THRESHOLD {
            return false;
        }

        // Walk backwards through samples, counting consecutive growth
        let mut consecutive_growth = 0;
        let mut prev_sample = self.current.load(Ordering::Relaxed);

        // Start from most recent sample and walk backwards
        let samples_to_check = count.min(MAX_SAMPLES);
        let current_index = self.sample_index.load(Ordering::Relaxed);

        for i in 1..samples_to_check {
            let index = (current_index + MAX_SAMPLES - i) % MAX_SAMPLES;
            let sample = self.samples[index].load(Ordering::Relaxed);

            if sample < prev_sample && (prev_sample - sample) >= LEAK_GROWTH_THRESHOLD {
                consecutive_growth += 1;
                if consecutive_growth >= LEAK_CONSECUTIVE_THRESHOLD {
                    return true;
                }
            } else {
                consecutive_growth = 0;
            }

            prev_sample = sample;
        }

        false
    }

    /// Get current memory statistics.
    ///
    /// Returns a snapshot of current, peak, and baseline memory usage,
    /// along with derived metrics like growth from baseline.
    ///
    /// # Returns
    ///
    /// A `MemoryStats` struct containing all metrics.
    pub fn stats(&self) -> MemoryStats {
        let current = self.current.load(Ordering::Relaxed);
        let peak = self.peak.load(Ordering::Relaxed);
        let baseline = self.baseline.load(Ordering::Relaxed);

        MemoryStats {
            current,
            peak,
            baseline,
            growth_from_baseline: current.saturating_sub(baseline),
            sample_count: self.sample_count.load(Ordering::Relaxed),
        }
    }

    /// Reset all metrics.
    ///
    /// Clears current, peak, baseline, and all samples.
    /// The next call to `record()` will set a new baseline.
    pub fn reset(&self) {
        self.current.store(0, Ordering::Relaxed);
        self.peak.store(0, Ordering::Relaxed);
        self.baseline.store(0, Ordering::Relaxed);
        self.sample_index.store(0, Ordering::Relaxed);
        self.sample_count.store(0, Ordering::Relaxed);

        // Clear all samples
        for sample in &self.samples {
            sample.store(0, Ordering::Relaxed);
        }
    }
}

impl Default for MemoryMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory usage statistics at a point in time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryStats {
    /// Current memory usage in bytes.
    pub current: usize,

    /// Peak memory usage in bytes.
    pub peak: usize,

    /// Baseline memory usage (first sample) in bytes.
    pub baseline: usize,

    /// Growth from baseline in bytes (current - baseline).
    pub growth_from_baseline: usize,

    /// Total number of samples recorded.
    pub sample_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_monitor_has_zero_stats() {
        let monitor = MemoryMonitor::new();
        let stats = monitor.stats();

        assert_eq!(stats.current, 0);
        assert_eq!(stats.peak, 0);
        assert_eq!(stats.baseline, 0);
        assert_eq!(stats.growth_from_baseline, 0);
        assert_eq!(stats.sample_count, 0);
    }

    #[test]
    fn test_record_sets_baseline_on_first_sample() {
        let monitor = MemoryMonitor::new();
        monitor.record(1024);

        let stats = monitor.stats();
        assert_eq!(stats.baseline, 1024);
        assert_eq!(stats.current, 1024);
        assert_eq!(stats.peak, 1024);
        assert_eq!(stats.sample_count, 1);
    }

    #[test]
    fn test_record_updates_peak() {
        let monitor = MemoryMonitor::new();
        monitor.record(1024);
        monitor.record(2048);
        monitor.record(1536);

        let stats = monitor.stats();
        assert_eq!(stats.current, 1536);
        assert_eq!(stats.peak, 2048);
        assert_eq!(stats.sample_count, 3);
    }

    #[test]
    fn test_growth_from_baseline() {
        let monitor = MemoryMonitor::new();
        monitor.record(1024);
        monitor.record(2048);

        let stats = monitor.stats();
        assert_eq!(stats.baseline, 1024);
        assert_eq!(stats.current, 2048);
        assert_eq!(stats.growth_from_baseline, 1024);
    }

    #[test]
    fn test_no_leak_with_stable_memory() {
        let monitor = MemoryMonitor::new();

        // Record stable memory usage
        for _ in 0..50 {
            monitor.record(1024 * 1024); // 1 MB constant
        }

        assert!(!monitor.has_potential_leak());
    }

    #[test]
    fn test_no_leak_with_fluctuating_memory() {
        let monitor = MemoryMonitor::new();

        // Record fluctuating memory
        for i in 0..50 {
            let mem = if i % 2 == 0 { 1024 * 1024 } else { 2048 * 1024 };
            monitor.record(mem);
        }

        assert!(!monitor.has_potential_leak());
    }

    #[test]
    fn test_leak_detected_with_monotonic_growth() {
        let monitor = MemoryMonitor::new();

        // Record monotonically increasing memory above threshold
        for i in 0..30 {
            let mem = 1024 * 1024 + (i * LEAK_GROWTH_THRESHOLD);
            monitor.record(mem);
        }

        assert!(monitor.has_potential_leak());
    }

    #[test]
    fn test_no_leak_with_small_growth() {
        let monitor = MemoryMonitor::new();

        // Record growth below threshold
        for i in 0..30 {
            let mem = 1024 * 1024 + (i * 100); // Small growth
            monitor.record(mem);
        }

        assert!(!monitor.has_potential_leak());
    }

    #[test]
    fn test_reset_clears_all_metrics() {
        let monitor = MemoryMonitor::new();
        monitor.record(1024);
        monitor.record(2048);

        monitor.reset();

        let stats = monitor.stats();
        assert_eq!(stats.current, 0);
        assert_eq!(stats.peak, 0);
        assert_eq!(stats.baseline, 0);
        assert_eq!(stats.sample_count, 0);
    }

    #[test]
    fn test_ring_buffer_wraps_correctly() {
        let monitor = MemoryMonitor::new();

        // Record more samples than buffer size
        for i in 0..(MAX_SAMPLES + 50) {
            monitor.record(1024 * i);
        }

        let stats = monitor.stats();
        assert_eq!(stats.sample_count, MAX_SAMPLES + 50);
    }

    #[test]
    fn test_concurrent_recording() {
        use std::sync::Arc;
        use std::thread;

        let monitor = Arc::new(MemoryMonitor::new());
        let mut handles = vec![];

        // Spawn multiple threads recording simultaneously
        for i in 0..4 {
            let m = Arc::clone(&monitor);
            let handle = thread::spawn(move || {
                for j in 0..100 {
                    m.record(1024 * (i * 100 + j));
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let stats = monitor.stats();
        assert_eq!(stats.sample_count, 400);
        assert!(stats.peak > 0);
    }

    #[test]
    fn test_memory_stats_equality() {
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
}
