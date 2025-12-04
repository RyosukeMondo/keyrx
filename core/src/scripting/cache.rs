//! Content-addressable cache for compiled Rhai ASTs.
//!
//! The cache uses a content hash (blake3) as the key and applies an
//! LRU eviction policy bounded by an estimated byte budget. This keeps
//! startup fast while preventing unbounded growth.

use blake3::Hasher;
use lru::LruCache;
use rhai::AST;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

/// Default maximum cache size in bytes (10 MiB).
pub const DEFAULT_MAX_SIZE_BYTES: usize = 10 * 1024 * 1024;

/// Default maximum number of cached entries.
pub const DEFAULT_MAX_ENTRIES: usize = 256;

/// Cache statistics snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheStats {
    /// Number of cache hits.
    pub hits: u64,
    /// Number of cache misses.
    pub misses: u64,
    /// Number of evicted entries.
    pub evictions: u64,
    /// Total cached bytes (estimated).
    pub size_bytes: usize,
    /// Maximum allowed cached bytes.
    pub max_size_bytes: usize,
    /// Current number of cached entries.
    pub entries: usize,
}

impl CacheStats {
    /// Calculate cache hit rate (0.0 to 100.0).
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }
}

#[derive(Debug)]
struct CacheEntry {
    ast: AST,
    size_bytes: usize,
    last_used: SystemTime,
}

#[derive(Debug)]
struct CacheIndex {
    entries: LruCache<String, CacheEntry>,
    size_bytes: usize,
    max_size_bytes: usize,
    hits: u64,
    misses: u64,
    evictions: u64,
}

impl CacheIndex {
    fn record_hit(&mut self) {
        self.hits = self.hits.saturating_add(1);
    }

    fn record_miss(&mut self) {
        self.misses = self.misses.saturating_add(1);
    }

    fn record_eviction(&mut self) {
        self.evictions = self.evictions.saturating_add(1);
    }

    fn replace_size_bytes(&mut self, previous: usize, new: usize) {
        self.size_bytes = self.size_bytes.saturating_sub(previous).saturating_add(new);
    }

    fn evict_until_within_budget(&mut self) {
        while self.size_bytes > self.max_size_bytes {
            if let Some((_key, evicted)) = self.entries.pop_lru() {
                self.size_bytes = self.size_bytes.saturating_sub(evicted.size_bytes);
                self.record_eviction();
            } else {
                break;
            }
        }
    }

    fn reset_after_poison(&mut self) {
        self.entries.clear();
        self.size_bytes = 0;
        self.hits = 0;
        self.misses = 0;
        self.evictions = 0;
    }
}

/// Content-addressable AST cache with LRU eviction.
#[derive(Debug)]
pub struct ScriptCache {
    cache_dir: PathBuf,
    index: Mutex<CacheIndex>,
}

impl ScriptCache {
    /// Create a cache with default limits.
    pub fn new(cache_dir: PathBuf) -> Self {
        Self::with_limits(cache_dir, DEFAULT_MAX_SIZE_BYTES, DEFAULT_MAX_ENTRIES)
    }

    /// Create a cache with custom limits.
    pub fn with_limits(cache_dir: PathBuf, max_size_bytes: usize, max_entries: usize) -> Self {
        let capped_bytes = if max_size_bytes == 0 {
            DEFAULT_MAX_SIZE_BYTES
        } else {
            max_size_bytes
        };

        let entry_capacity = NonZeroUsize::new(max_entries.max(1)).unwrap_or(NonZeroUsize::MIN);

        let index = CacheIndex {
            entries: LruCache::new(entry_capacity),
            size_bytes: 0,
            max_size_bytes: capped_bytes,
            hits: 0,
            misses: 0,
            evictions: 0,
        };

        Self {
            cache_dir,
            index: Mutex::new(index),
        }
    }

    /// Get the cache directory for persistence.
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Look up a script in the cache by content.
    ///
    /// Returns a cloned AST on cache hit, otherwise `None`.
    pub fn get(&self, script: &str) -> Option<AST> {
        let hash = cache_key(script);
        let mut index = self.lock_index();

        let result = {
            if let Some(entry) = index.entries.get_mut(&hash) {
                entry.last_used = SystemTime::now();
                Some(entry.ast.clone())
            } else {
                None
            }
        };

        if result.is_some() {
            index.record_hit();
        } else {
            index.record_miss();
        }

        result
    }

    /// Store a compiled AST keyed by its content hash.
    pub fn put(&self, script: &str, ast: &AST) {
        let hash = cache_key(script);
        let size_bytes = estimated_entry_size(script, ast);
        let mut index = self.lock_index();

        // Replace existing entry if present to refresh recency and size accounting
        if let Some(replaced) = index.entries.put(
            hash,
            CacheEntry {
                ast: ast.clone(),
                size_bytes,
                last_used: SystemTime::now(),
            },
        ) {
            index.replace_size_bytes(replaced.size_bytes, size_bytes);
        } else {
            index.size_bytes = index.size_bytes.saturating_add(size_bytes);
        }

        // Ensure we stay within the configured budget
        index.evict_until_within_budget();
    }

    /// Clear all cache entries and statistics.
    pub fn clear(&self) {
        let mut index = self.lock_index();
        index.entries.clear();
        index.size_bytes = 0;
        index.hits = 0;
        index.misses = 0;
        index.evictions = 0;
    }

    /// Get a snapshot of cache statistics.
    pub fn stats(&self) -> CacheStats {
        let index = self.lock_index();
        CacheStats {
            hits: index.hits,
            misses: index.misses,
            evictions: index.evictions,
            size_bytes: index.size_bytes,
            max_size_bytes: index.max_size_bytes,
            entries: index.entries.len(),
        }
    }

    fn lock_index(&self) -> std::sync::MutexGuard<'_, CacheIndex> {
        match self.index.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                let mut guard = poisoned.into_inner();
                guard.reset_after_poison();
                guard
            }
        }
    }
}

/// Create a stable content hash for the given script.
pub fn cache_key(script: &str) -> String {
    let mut hasher = Hasher::new();
    hasher.update(script.as_bytes());
    hasher.finalize().to_hex().to_string()
}

fn estimated_entry_size(script: &str, _ast: &AST) -> usize {
    // Use script length as a conservative placeholder until serialized AST sizing is available.
    script.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rhai::Engine;

    fn make_cache(max_size_bytes: usize) -> ScriptCache {
        ScriptCache::with_limits(PathBuf::from(".keyrx_cache/scripts"), max_size_bytes, 8)
    }

    fn compile_ast(script: &str) -> AST {
        Engine::new().compile(script).expect("compile")
    }

    #[test]
    fn cache_hit_returns_ast() {
        let cache = make_cache(1024);
        let script = "let a = 1; a + 1;";
        let ast = compile_ast(script);

        cache.put(script, &ast);

        assert!(cache.get(script).is_some());

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.entries, 1);
    }

    #[test]
    fn cache_miss_records_stat() {
        let cache = make_cache(1024);
        assert!(cache.get("not cached").is_none());

        let stats = cache.stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn lru_eviction_respects_byte_budget() {
        let budget = 24;
        let cache = make_cache(budget);

        // Scripts are ~12 bytes each; inserting three should exceed the budget.
        let script_a = "let a = 11;";
        let script_b = "let b = 22;";
        let script_c = "let c = 33;";

        cache.put(script_a, &compile_ast(script_a));
        cache.put(script_b, &compile_ast(script_b));
        cache.put(script_c, &compile_ast(script_c));

        // Oldest entry should be evicted to stay within budget
        assert!(cache.get(script_a).is_none());
        assert!(cache.get(script_b).is_some());
        assert!(cache.get(script_c).is_some());

        let stats = cache.stats();
        assert_eq!(stats.evictions, 1);
        assert!(stats.size_bytes <= budget);
    }

    #[test]
    fn cache_updates_recency_on_get() {
        let cache = make_cache(48);

        let script_a = "let a = 100;";
        let script_b = "let b = 200;";
        let script_c = "let c = 300;";

        cache.put(script_a, &compile_ast(script_a));
        cache.put(script_b, &compile_ast(script_b));
        cache.put(script_c, &compile_ast(script_c));

        // Access A to make it most recent
        assert!(cache.get(script_a).is_some());

        // Force eviction by exceeding byte budget with a larger script
        let large_script = "let really_big_value = 1234567890;";
        cache.put(large_script, &compile_ast(large_script));

        // The least recently used should now be B
        assert!(cache.get(script_b).is_none());
        assert!(cache.get(script_a).is_some());
    }

    #[test]
    fn cache_clear_resets_state() {
        let cache = make_cache(256);
        let script = "let x = 42;";
        cache.put(script, &compile_ast(script));
        assert!(cache.get(script).is_some());

        cache.clear();
        assert!(cache.get(script).is_none());

        let stats = cache.stats();
        assert_eq!(stats.entries, 0);
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 1); // miss after clear
    }

    #[test]
    fn cache_key_is_content_addressable() {
        let script_a1 = "let x = 1 + 2;";
        let script_a2 = "let x = 1 + 2;";
        let script_b = "let x = 3 + 4;";

        let hash_a1 = cache_key(script_a1);
        let hash_a2 = cache_key(script_a2);
        let hash_b = cache_key(script_b);

        assert_eq!(hash_a1, hash_a2);
        assert_ne!(hash_a1, hash_b);
    }

    #[test]
    fn cache_survives_poisoning() {
        let cache = make_cache(256);
        let script = "let x = 1;";
        cache.put(script, &compile_ast(script));

        // Intentionally poison the mutex in the same thread to avoid Send requirements
        let _ = std::panic::catch_unwind(|| {
            if let Ok(_guard) = cache.index.lock() {
                panic!("intentional poison");
            }
        });

        // Should recover with cleared cache
        assert!(cache.get(script).is_none());
        assert_eq!(cache.stats().entries, 0);
    }
}
