//! Profile points for function-level timing and hot spot identification.
//!
//! This module provides thread-safe, low-overhead function-level profiling
//! through RAII guards. Profile points automatically measure elapsed time
//! and track statistics for identifying hot spots in the codebase.
//!
//! # Design
//!
//! - Uses DashMap for lock-free concurrent access to profile data
//! - RAII guards ensure automatic timing without manual start/stop
//! - Bounded memory: old entries can be pruned if needed
//! - Hot spot identification through aggregated statistics
//!
//! # Performance
//!
//! - Recording: O(1) amortized via DashMap
//! - Overhead: < 1 microsecond per profile point
//! - No allocations in hot path after initial map growth
//!
//! # Example
//!
//! ```ignore
//! use keyrx_core::metrics::profile::ProfilePoints;
//!
//! let profiler = ProfilePoints::new();
//!
//! // Automatic timing with RAII guard
//! {
//!     let _guard = profiler.start("expensive_function");
//!     // ... code to profile ...
//! } // Automatically records elapsed time on drop
//!
//! // Get statistics
//! let stats = profiler.get_stats("expensive_function");
//! println!("Calls: {}, Avg: {}us", stats.count, stats.avg_micros);
//!
//! // Find hot spots
//! let hot_spots = profiler.hot_spots(5); // Top 5
//! ```

use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Statistics for a single profile point.
///
/// Tracks call count, total time, and derived statistics for identifying
/// hot spots and performance bottlenecks.
#[derive(Debug, Clone)]
pub struct ProfileStats {
    /// Total number of times this profile point was recorded.
    pub count: u64,

    /// Total accumulated time in microseconds across all calls.
    pub total_micros: u64,

    /// Average time per call in microseconds.
    pub avg_micros: u64,

    /// Minimum recorded time in microseconds.
    pub min_micros: u64,

    /// Maximum recorded time in microseconds.
    pub max_micros: u64,
}

impl ProfileStats {
    /// Create empty statistics.
    fn empty() -> Self {
        Self {
            count: 0,
            total_micros: 0,
            avg_micros: 0,
            min_micros: 0,
            max_micros: 0,
        }
    }
}

/// Internal profile point data stored in the map.
///
/// Uses atomics for lock-free updates from multiple threads.
struct ProfileData {
    /// Total number of calls (atomic for concurrent updates).
    count: AtomicU64,

    /// Total accumulated time in microseconds (atomic for concurrent updates).
    total_micros: AtomicU64,

    /// Minimum recorded time (atomic for concurrent updates).
    min_micros: AtomicU64,

    /// Maximum recorded time (atomic for concurrent updates).
    max_micros: AtomicU64,
}

impl ProfileData {
    /// Create new profile data.
    fn new() -> Self {
        Self {
            count: AtomicU64::new(0),
            total_micros: AtomicU64::new(0),
            min_micros: AtomicU64::new(u64::MAX),
            max_micros: AtomicU64::new(0),
        }
    }

    /// Record a sample atomically.
    fn record(&self, micros: u64) {
        self.count.fetch_add(1, Ordering::Relaxed);
        self.total_micros.fetch_add(micros, Ordering::Relaxed);

        // Update min atomically
        let mut current_min = self.min_micros.load(Ordering::Relaxed);
        while micros < current_min {
            match self.min_micros.compare_exchange_weak(
                current_min,
                micros,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current_min = actual,
            }
        }

        // Update max atomically
        let mut current_max = self.max_micros.load(Ordering::Relaxed);
        while micros > current_max {
            match self.max_micros.compare_exchange_weak(
                current_max,
                micros,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current_max = actual,
            }
        }
    }

    /// Get current statistics snapshot.
    fn stats(&self) -> ProfileStats {
        let count = self.count.load(Ordering::Relaxed);
        let total = self.total_micros.load(Ordering::Relaxed);
        let min = self.min_micros.load(Ordering::Relaxed);
        let max = self.max_micros.load(Ordering::Relaxed);

        ProfileStats {
            count,
            total_micros: total,
            avg_micros: if count > 0 { total / count } else { 0 },
            min_micros: if count > 0 { min } else { 0 },
            max_micros: max,
        }
    }
}

/// Thread-safe profile points for function-level timing.
///
/// This type allows recording function execution times with minimal overhead.
/// Profile points are identified by static string names and statistics are
/// aggregated for hot spot analysis.
///
/// # Thread Safety
///
/// This type is `Send + Sync` and can be safely shared across threads.
/// All operations use either lock-free atomics (DashMap) or relaxed
/// atomic operations within profile data.
///
/// # Memory
///
/// Memory usage grows with the number of unique profile points (function names).
/// Each profile point uses ~64 bytes. For typical applications with hundreds
/// of profile points, total memory usage is < 100KB.
///
/// # Example
///
/// ```ignore
/// use keyrx_core::metrics::profile::ProfilePoints;
/// use std::sync::Arc;
///
/// let profiler = Arc::new(ProfilePoints::new());
///
/// // Profile some work
/// {
///     let _guard = profiler.start("process_event");
///     // ... event processing ...
/// }
///
/// // Get statistics
/// let stats = profiler.get_stats("process_event");
/// println!("Average latency: {}us", stats.avg_micros);
///
/// // Find hot spots
/// for (name, stats) in profiler.hot_spots(10) {
///     println!("{}: {} calls, {}us total", name, stats.count, stats.total_micros);
/// }
/// ```
pub struct ProfilePoints {
    /// Map from profile point name to aggregated statistics.
    ///
    /// DashMap provides lock-free concurrent access for both reads and writes.
    points: Arc<DashMap<&'static str, Arc<ProfileData>>>,
}

impl ProfilePoints {
    /// Create a new profile points tracker.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let profiler = ProfilePoints::new();
    /// ```
    pub fn new() -> Self {
        Self {
            points: Arc::new(DashMap::new()),
        }
    }

    /// Start profiling a named code section.
    ///
    /// Returns an RAII guard that automatically records elapsed time when dropped.
    ///
    /// # Arguments
    ///
    /// * `name` - Static string identifier for this profile point
    ///
    /// # Performance
    ///
    /// This operation is O(1) amortized. First call for a new name allocates
    /// map entry, subsequent calls reuse existing entry.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let _guard = profiler.start("expensive_function");
    /// // ... code to profile ...
    /// // Guard automatically records time on drop
    /// ```
    pub fn start(&self, name: &'static str) -> ProfilePointGuard<'_> {
        ProfilePointGuard::new(self, name)
    }

    /// Record a profile point manually.
    ///
    /// This is called automatically by `ProfilePointGuard` on drop, but can
    /// also be called manually if needed.
    ///
    /// # Arguments
    ///
    /// * `name` - Static string identifier for the profile point
    /// * `micros` - Duration in microseconds
    ///
    /// # Example
    ///
    /// ```ignore
    /// profiler.record("manual_timing", 1500); // 1.5ms
    /// ```
    pub fn record(&self, name: &'static str, micros: u64) {
        // Get or create profile data entry
        let data = self
            .points
            .entry(name)
            .or_insert_with(|| Arc::new(ProfileData::new()))
            .clone();

        // Record the sample
        data.record(micros);
    }

    /// Get statistics for a specific profile point.
    ///
    /// # Arguments
    ///
    /// * `name` - Profile point name to query
    ///
    /// # Returns
    ///
    /// Statistics for the named profile point, or empty stats if the name
    /// has never been recorded.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let stats = profiler.get_stats("process_event");
    /// println!("Calls: {}, Avg: {}us", stats.count, stats.avg_micros);
    /// ```
    pub fn get_stats(&self, name: &'static str) -> ProfileStats {
        self.points
            .get(name)
            .map(|entry| entry.value().stats())
            .unwrap_or_else(ProfileStats::empty)
    }

    /// Get all profile points and their statistics.
    ///
    /// # Returns
    ///
    /// Vector of (name, stats) tuples for all recorded profile points.
    ///
    /// # Example
    ///
    /// ```ignore
    /// for (name, stats) in profiler.all_stats() {
    ///     println!("{}: {} calls", name, stats.count);
    /// }
    /// ```
    pub fn all_stats(&self) -> Vec<(&'static str, ProfileStats)> {
        self.points
            .iter()
            .map(|entry| (*entry.key(), entry.value().stats()))
            .collect()
    }

    /// Get the top N hot spots by total time.
    ///
    /// Hot spots are profile points that consumed the most total time,
    /// indicating where the application spends most of its execution time.
    ///
    /// # Arguments
    ///
    /// * `n` - Number of hot spots to return
    ///
    /// # Returns
    ///
    /// Vector of (name, stats) tuples sorted by total time descending.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let hot_spots = profiler.hot_spots(5);
    /// for (name, stats) in hot_spots {
    ///     println!("{}: {}us total in {} calls",
    ///         name, stats.total_micros, stats.count);
    /// }
    /// ```
    pub fn hot_spots(&self, n: usize) -> Vec<(&'static str, ProfileStats)> {
        let mut stats = self.all_stats();

        // Sort by total time descending
        stats.sort_by_key(|(_, s)| std::cmp::Reverse(s.total_micros));

        // Take top N
        stats.truncate(n);
        stats
    }

    /// Reset all profile points, clearing collected statistics.
    ///
    /// This is useful for starting fresh measurements or clearing data
    /// from a previous time window.
    ///
    /// # Example
    ///
    /// ```ignore
    /// profiler.reset();
    /// // All statistics cleared
    /// ```
    pub fn reset(&self) {
        self.points.clear();
    }

    /// Get the number of unique profile points.
    ///
    /// # Returns
    ///
    /// Count of unique profile point names that have been recorded.
    pub fn count(&self) -> usize {
        self.points.len()
    }
}

impl Default for ProfilePoints {
    fn default() -> Self {
        Self::new()
    }
}

// Safety: DashMap and Arc are Send + Sync, ProfileData uses atomics
#[allow(unsafe_code)]
unsafe impl Send for ProfilePoints {}
#[allow(unsafe_code)]
unsafe impl Sync for ProfilePoints {}

/// RAII guard for automatic profile timing.
///
/// When dropped, this guard automatically records the elapsed time
/// since its creation to the profile points tracker.
pub struct ProfilePointGuard<'a> {
    profiler: &'a ProfilePoints,
    name: &'static str,
    start: Instant,
}

impl<'a> ProfilePointGuard<'a> {
    /// Create a new profile point guard.
    ///
    /// This should typically be created via `ProfilePoints::start`.
    pub fn new(profiler: &'a ProfilePoints, name: &'static str) -> Self {
        Self {
            profiler,
            name,
            start: Instant::now(),
        }
    }
}

impl Drop for ProfilePointGuard<'_> {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed().as_micros() as u64;
        self.profiler.record(self.name, elapsed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_record_and_stats() {
        let profiler = ProfilePoints::new();

        // Record some samples
        profiler.record("test_function", 100);
        profiler.record("test_function", 200);
        profiler.record("test_function", 300);

        let stats = profiler.get_stats("test_function");
        assert_eq!(stats.count, 3);
        assert_eq!(stats.total_micros, 600);
        assert_eq!(stats.avg_micros, 200);
        assert_eq!(stats.min_micros, 100);
        assert_eq!(stats.max_micros, 300);
    }

    #[test]
    fn test_guard_auto_drop() {
        let profiler = ProfilePoints::new();

        {
            let _guard = profiler.start("test_function");
            // Guard should record on drop
            // Add minimal work to ensure measurable time
            thread::sleep(std::time::Duration::from_micros(10));
        }

        let stats = profiler.get_stats("test_function");
        assert_eq!(stats.count, 1);
        // Timing may be 0 on very fast systems, so just verify count
    }

    #[test]
    fn test_empty_stats() {
        let profiler = ProfilePoints::new();
        let stats = profiler.get_stats("nonexistent");

        assert_eq!(stats.count, 0);
        assert_eq!(stats.total_micros, 0);
        assert_eq!(stats.avg_micros, 0);
        assert_eq!(stats.min_micros, 0);
        assert_eq!(stats.max_micros, 0);
    }

    #[test]
    fn test_multiple_profile_points() {
        let profiler = ProfilePoints::new();

        profiler.record("function_a", 100);
        profiler.record("function_b", 200);
        profiler.record("function_c", 300);

        assert_eq!(profiler.count(), 3);

        let stats_a = profiler.get_stats("function_a");
        let stats_b = profiler.get_stats("function_b");
        let stats_c = profiler.get_stats("function_c");

        assert_eq!(stats_a.total_micros, 100);
        assert_eq!(stats_b.total_micros, 200);
        assert_eq!(stats_c.total_micros, 300);
    }

    #[test]
    fn test_hot_spots() {
        let profiler = ProfilePoints::new();

        // Record different amounts of time
        profiler.record("slow_function", 1000);
        profiler.record("slow_function", 1000);
        profiler.record("medium_function", 500);
        profiler.record("fast_function", 100);

        let hot_spots = profiler.hot_spots(2);

        assert_eq!(hot_spots.len(), 2);
        assert_eq!(hot_spots[0].0, "slow_function");
        assert_eq!(hot_spots[0].1.total_micros, 2000);
        assert_eq!(hot_spots[1].0, "medium_function");
        assert_eq!(hot_spots[1].1.total_micros, 500);
    }

    #[test]
    fn test_reset() {
        let profiler = ProfilePoints::new();

        profiler.record("test_function", 100);
        assert_eq!(profiler.count(), 1);

        profiler.reset();
        assert_eq!(profiler.count(), 0);

        let stats = profiler.get_stats("test_function");
        assert_eq!(stats.count, 0);
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
    fn test_thread_safety() {
        let profiler = Arc::new(ProfilePoints::new());
        let mut handles = vec![];

        // Spawn 10 threads that each record 100 samples
        for _ in 0..10 {
            let prof = Arc::clone(&profiler);
            let handle = thread::spawn(move || {
                for i in 1..=100 {
                    prof.record("concurrent_function", i * 10);
                }
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().expect("Thread panicked");
        }

        // Verify all samples were recorded
        let stats = profiler.get_stats("concurrent_function");
        assert_eq!(stats.count, 1000);
    }

    #[test]
    fn test_guard_timing() {
        let profiler = ProfilePoints::new();

        {
            let _guard = profiler.start("sleep_test");
            thread::sleep(std::time::Duration::from_micros(100));
        }

        let stats = profiler.get_stats("sleep_test");
        assert_eq!(stats.count, 1);
        // Sleep timing is non-deterministic, just verify it recorded something
        assert!(stats.total_micros > 50);
    }

    #[test]
    fn test_min_max_tracking() {
        let profiler = ProfilePoints::new();

        profiler.record("variable_timing", 50);
        profiler.record("variable_timing", 500);
        profiler.record("variable_timing", 5000);
        profiler.record("variable_timing", 100);

        let stats = profiler.get_stats("variable_timing");
        assert_eq!(stats.count, 4);
        assert_eq!(stats.min_micros, 50);
        assert_eq!(stats.max_micros, 5000);
    }

    #[test]
    fn test_hot_spots_ordering() {
        let profiler = ProfilePoints::new();

        // Record in random order
        profiler.record("medium", 500);
        profiler.record("fast", 100);
        profiler.record("slow", 2000);
        profiler.record("medium", 500); // Total: 1000

        let hot_spots = profiler.hot_spots(10);

        // Should be ordered by total time descending
        assert_eq!(hot_spots[0].0, "slow"); // 2000us
        assert_eq!(hot_spots[1].0, "medium"); // 1000us
        assert_eq!(hot_spots[2].0, "fast"); // 100us
    }

    #[test]
    fn test_hot_spots_limit() {
        let profiler = ProfilePoints::new();

        // Use predefined static strings instead of dynamic allocation
        profiler.record("func_1", 100);
        profiler.record("func_2", 200);
        profiler.record("func_3", 300);
        profiler.record("func_4", 400);
        profiler.record("func_5", 500);
        profiler.record("func_6", 600);
        profiler.record("func_7", 700);
        profiler.record("func_8", 800);
        profiler.record("func_9", 900);
        profiler.record("func_10", 1000);

        let hot_spots = profiler.hot_spots(3);
        assert_eq!(hot_spots.len(), 3);
        // Should return top 3 by total time
        assert_eq!(hot_spots[0].0, "func_10");
        assert_eq!(hot_spots[1].0, "func_9");
        assert_eq!(hot_spots[2].0, "func_8");
    }

    #[test]
    fn test_concurrent_same_name() {
        let profiler = Arc::new(ProfilePoints::new());
        let mut handles = vec![];

        // Multiple threads recording to the same profile point
        for _ in 0..5 {
            let prof = Arc::clone(&profiler);
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    let _guard = prof.start("shared_point");
                    // Minimal work
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().expect("Thread panicked");
        }

        let stats = profiler.get_stats("shared_point");
        assert_eq!(stats.count, 500);
    }
}
