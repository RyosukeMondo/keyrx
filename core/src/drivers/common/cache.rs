//! Keymap cache trait and implementations for performance optimization.

use crate::drivers::keycodes::KeyCode;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

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

/// LRU-based keymap cache implementation.
///
/// Uses a least-recently-used eviction policy to keep memory usage bounded.
/// Thread-safe via internal Mutex.
pub struct LruKeymapCache {
    cache: Mutex<LruCache<(u32, String), KeyCode>>,
    stats: CacheStatsTracker,
    capacity: NonZeroUsize,
}

impl LruKeymapCache {
    /// Create a new LRU cache with the specified capacity.
    ///
    /// # Arguments
    /// * `capacity` - Maximum number of entries to cache
    ///
    /// # Returns
    /// * `Some(cache)` if capacity > 0
    /// * `None` if capacity is 0
    pub fn new(capacity: usize) -> Option<Self> {
        let capacity = NonZeroUsize::new(capacity)?;
        Some(Self {
            cache: Mutex::new(LruCache::new(capacity)),
            stats: CacheStatsTracker::new(),
            capacity,
        })
    }
}

impl KeymapCache for LruKeymapCache {
    fn get(&self, scan_code: u32, device_id: &str) -> Option<KeyCode> {
        let key = (scan_code, device_id.to_string());

        // Handle poisoned mutex by clearing cache and returning None
        let mut cache = match self.cache.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                // Cache corrupted - clear and rebuild
                let mut guard = poisoned.into_inner();
                guard.clear();
                guard
            }
        };

        match cache.get(&key) {
            Some(&keycode) => {
                self.stats.record_hit();
                Some(keycode)
            }
            None => {
                self.stats.record_miss();
                None
            }
        }
    }

    fn insert(&self, scan_code: u32, device_id: &str, key: KeyCode) {
        let cache_key = (scan_code, device_id.to_string());

        let mut cache = match self.cache.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                let mut guard = poisoned.into_inner();
                guard.clear();
                guard
            }
        };

        cache.put(cache_key, key);
    }

    fn invalidate_device(&self, device_id: &str) {
        let mut cache = match self.cache.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                let mut guard = poisoned.into_inner();
                guard.clear();
                guard
            }
        };

        // Collect keys to remove (can't modify while iterating)
        let keys_to_remove: Vec<_> = cache
            .iter()
            .filter_map(|(key, _)| {
                if key.1 == device_id {
                    Some(key.clone())
                } else {
                    None
                }
            })
            .collect();

        // Remove all matching entries
        for key in keys_to_remove {
            cache.pop(&key);
        }
    }

    fn clear(&self) {
        let mut cache = match self.cache.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        cache.clear();
    }

    fn stats(&self) -> CacheStats {
        let cache = match self.cache.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        let size = cache.len();
        drop(cache);

        self.stats.snapshot(size, self.capacity.get())
    }
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

    #[test]
    fn lru_cache_basic_operations() {
        let cache = LruKeymapCache::new(10).unwrap();

        // Initially empty
        assert_eq!(cache.get(30, "dev0"), None);
        let stats = cache.stats();
        assert_eq!(stats.size, 0);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hits, 0);

        // Insert and retrieve
        cache.insert(30, "dev0", KeyCode::A);
        assert_eq!(cache.get(30, "dev0"), Some(KeyCode::A));

        let stats = cache.stats();
        assert_eq!(stats.size, 1);
        assert_eq!(stats.hits, 1);
    }

    #[test]
    fn lru_cache_eviction() {
        let cache = LruKeymapCache::new(3).unwrap();

        // Fill cache to capacity
        cache.insert(1, "dev0", KeyCode::A);
        cache.insert(2, "dev0", KeyCode::B);
        cache.insert(3, "dev0", KeyCode::C);
        assert_eq!(cache.stats().size, 3);

        // Add one more - should evict LRU (key 1)
        cache.insert(4, "dev0", KeyCode::D);
        assert_eq!(cache.stats().size, 3);

        // Key 1 should be evicted
        assert_eq!(cache.get(1, "dev0"), None);
        // Keys 2, 3, 4 should still be present
        assert_eq!(cache.get(2, "dev0"), Some(KeyCode::B));
        assert_eq!(cache.get(3, "dev0"), Some(KeyCode::C));
        assert_eq!(cache.get(4, "dev0"), Some(KeyCode::D));
    }

    #[test]
    fn lru_cache_device_invalidation() {
        let cache = LruKeymapCache::new(10).unwrap();

        // Add entries for multiple devices
        cache.insert(1, "dev0", KeyCode::A);
        cache.insert(2, "dev0", KeyCode::B);
        cache.insert(1, "dev1", KeyCode::C);
        cache.insert(2, "dev1", KeyCode::D);

        assert_eq!(cache.stats().size, 4);

        // Invalidate dev0
        cache.invalidate_device("dev0");

        // dev0 entries should be gone
        assert_eq!(cache.get(1, "dev0"), None);
        assert_eq!(cache.get(2, "dev0"), None);

        // dev1 entries should remain
        assert_eq!(cache.get(1, "dev1"), Some(KeyCode::C));
        assert_eq!(cache.get(2, "dev1"), Some(KeyCode::D));

        assert_eq!(cache.stats().size, 2);
    }

    #[test]
    fn lru_cache_clear() {
        let cache = LruKeymapCache::new(10).unwrap();

        cache.insert(1, "dev0", KeyCode::A);
        cache.insert(2, "dev0", KeyCode::B);
        assert_eq!(cache.stats().size, 2);

        cache.clear();

        assert_eq!(cache.stats().size, 0);
        assert_eq!(cache.get(1, "dev0"), None);
        assert_eq!(cache.get(2, "dev0"), None);
    }

    #[test]
    fn lru_cache_hit_rate() {
        let cache = LruKeymapCache::new(10).unwrap();

        // 10 inserts
        for i in 0..10 {
            cache.insert(i, "dev0", KeyCode::A);
        }

        // 10 hits
        for i in 0..10 {
            cache.get(i, "dev0");
        }

        // 5 misses
        for i in 10..15 {
            cache.get(i, "dev0");
        }

        let stats = cache.stats();
        assert_eq!(stats.hits, 10);
        assert_eq!(stats.misses, 5);
        assert_eq!(stats.hit_rate(), 200.0 / 3.0); // 10/15 * 100 = 66.666...
    }

    #[test]
    fn lru_cache_zero_capacity_returns_none() {
        assert!(LruKeymapCache::new(0).is_none());
    }
}
