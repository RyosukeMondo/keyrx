//! Keymap cache trait and implementations for performance optimization.

use crate::drivers::keycodes::KeyCode;
use std::sync::atomic::{AtomicU64, Ordering};

/// Statistics tracked by the keymap cache.
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of successful cache hits.
    pub hits: u64,
    /// Number of cache misses.
    pub misses: u64,
    /// Current number of entries in the cache.
    pub size: usize,
    /// Maximum capacity of the cache.
    pub capacity: usize,
}

impl CacheStats {
    /// Calculate the cache hit rate as a percentage (0.0 to 100.0).
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }
}

/// Thread-safe tracker for cache statistics using atomic operations.
#[allow(dead_code)] // Used by LruKeymapCache implementation
#[derive(Debug)]
pub(crate) struct CacheStatsTracker {
    hits: AtomicU64,
    misses: AtomicU64,
}

#[allow(dead_code)] // Used by LruKeymapCache implementation
impl CacheStatsTracker {
    pub(crate) fn new() -> Self {
        Self {
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
        }
    }

    pub(crate) fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn snapshot(&self, size: usize, capacity: usize) -> CacheStats {
        CacheStats {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            size,
            capacity,
        }
    }
}

/// Platform-agnostic keymap cache interface.
///
/// Provides caching for scan code to KeyCode mappings with device-specific
/// invalidation support. All methods are thread-safe.
pub trait KeymapCache: Send + Sync {
    /// Retrieve a cached KeyCode for the given scan code and device.
    ///
    /// # Arguments
    /// * `scan_code` - The hardware scan code
    /// * `device_id` - Unique identifier for the input device
    ///
    /// # Returns
    /// * `Some(KeyCode)` if the mapping is cached
    /// * `None` if not found (cache miss)
    fn get(&self, scan_code: u32, device_id: &str) -> Option<KeyCode>;

    /// Insert a scan code to KeyCode mapping into the cache.
    ///
    /// # Arguments
    /// * `scan_code` - The hardware scan code
    /// * `device_id` - Unique identifier for the input device
    /// * `key` - The KeyCode to cache
    fn insert(&self, scan_code: u32, device_id: &str, key: KeyCode);

    /// Invalidate all cache entries for a specific device.
    ///
    /// Called when a device is removed or its keymap changes.
    ///
    /// # Arguments
    /// * `device_id` - Unique identifier for the device to invalidate
    fn invalidate_device(&self, device_id: &str);

    /// Clear all cache entries.
    fn clear(&self);

    /// Get current cache statistics.
    fn stats(&self) -> CacheStats;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_stats_hit_rate_calculation() {
        let stats = CacheStats {
            hits: 95,
            misses: 5,
            size: 50,
            capacity: 100,
        };
        assert_eq!(stats.hit_rate(), 95.0);

        let empty = CacheStats {
            hits: 0,
            misses: 0,
            size: 0,
            capacity: 100,
        };
        assert_eq!(empty.hit_rate(), 0.0);
    }

    #[test]
    fn cache_stats_tracker_basic() {
        let tracker = CacheStatsTracker::new();
        assert_eq!(tracker.hits.load(Ordering::Relaxed), 0);
        assert_eq!(tracker.misses.load(Ordering::Relaxed), 0);

        tracker.record_hit();
        tracker.record_hit();
        tracker.record_miss();

        let stats = tracker.snapshot(10, 100);
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.size, 10);
        assert_eq!(stats.capacity, 100);
    }
}
