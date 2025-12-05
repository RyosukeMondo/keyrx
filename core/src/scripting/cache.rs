//! Content-addressable cache for compiled Rhai ASTs.
//!
//! The cache uses a content hash (blake3) as the key and applies an
//! LRU eviction policy bounded by an estimated byte budget. This keeps
//! startup fast while preventing unbounded growth.

use blake3::Hasher;
use lru::LruCache;
use rhai::{Engine, AST};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;
use tracing::warn;

/// Default maximum cache size in bytes (10 MiB).
pub const DEFAULT_MAX_SIZE_BYTES: usize = 10 * 1024 * 1024;

/// Default maximum number of cached entries.
pub const DEFAULT_MAX_ENTRIES: usize = 256;

/// Cache format version for persisted ASTs.
const CACHE_FORMAT_VERSION: u32 = 1;

/// File extension used for persisted AST cache entries.
const CACHE_FILE_EXTENSION: &str = "rhaiast";

/// Version of the core crate used to ensure cache compatibility across releases.
const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");

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

#[derive(Debug, Serialize, Deserialize)]
struct PersistedAst {
    format_version: u32,
    core_version: String,
    hash: String,
    script: String,
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

        {
            let mut index = self.lock_index();
            if let Some(entry) = index.entries.get_mut(&hash) {
                entry.last_used = SystemTime::now();
                let ast = entry.ast.clone();
                index.record_hit();
                return Some(ast);
            }
        }

        if let Some((ast, size_bytes)) = self.load_from_disk(&hash) {
            let mut index = self.lock_index();
            self.upsert_entry(&mut index, hash, ast.clone(), size_bytes);
            index.record_hit();
            return Some(ast);
        }

        let mut index = self.lock_index();
        index.record_miss();
        None
    }

    /// Store a compiled AST keyed by its content hash.
    pub fn put(&self, script: &str, ast: &AST) {
        let hash = cache_key(script);

        let (serialized, size_bytes) = match serialize_entry(&hash, script, ast) {
            Ok(bytes) => {
                let size = bytes.len();
                (Some(bytes), size)
            }
            Err(error) => {
                warn!(
                    %hash,
                    error = ?error,
                    "failed to serialize AST cache entry; falling back to in-memory only"
                );
                (None, estimated_entry_size(script, ast))
            }
        };

        {
            let mut index = self.lock_index();
            self.upsert_entry(&mut index, hash.clone(), ast.clone(), size_bytes);
            index.evict_until_within_budget();
        }

        if let Some(bytes) = serialized {
            if let Err(error) = self.persist_entry(&hash, &bytes) {
                warn!(%hash, error = ?error, "failed to persist AST cache entry");
            }
        }
    }

    /// Clear all cache entries and statistics.
    pub fn clear(&self) {
        let mut index = self.lock_index();
        index.entries.clear();
        index.size_bytes = 0;
        index.hits = 0;
        index.misses = 0;
        index.evictions = 0;

        if let Err(error) = fs::remove_dir_all(&self.cache_dir) {
            if error.kind() != io::ErrorKind::NotFound {
                warn!(
                    error = ?error,
                    path = %self.cache_dir.display(),
                    "failed to remove persisted cache directory"
                );
            }
        }
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

    fn upsert_entry(&self, index: &mut CacheIndex, hash: String, ast: AST, size_bytes: usize) {
        if let Some(replaced) = index.entries.put(
            hash,
            CacheEntry {
                ast,
                size_bytes,
                last_used: SystemTime::now(),
            },
        ) {
            index.replace_size_bytes(replaced.size_bytes, size_bytes);
        } else {
            index.size_bytes = index.size_bytes.saturating_add(size_bytes);
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

    fn cache_file_path(&self, hash: &str) -> PathBuf {
        self.cache_dir
            .join(format!("{hash}.{CACHE_FILE_EXTENSION}"))
    }

    fn persist_entry(&self, hash: &str, bytes: &[u8]) -> io::Result<()> {
        fs::create_dir_all(&self.cache_dir)?;
        let path = self.cache_file_path(hash);
        let tmp_path = path.with_extension("tmp");
        fs::write(&tmp_path, bytes)?;
        fs::rename(&tmp_path, &path)?;
        Ok(())
    }

    fn load_from_disk(&self, hash: &str) -> Option<(AST, usize)> {
        let path = self.cache_file_path(hash);
        let bytes = fs::read(&path).ok()?;

        match serde_json::from_slice::<PersistedAst>(&bytes) {
            Ok(entry)
                if entry.format_version == CACHE_FORMAT_VERSION
                    && entry.core_version == CORE_VERSION
                    && entry.hash == hash =>
            {
                match Engine::new().compile(&entry.script) {
                    Ok(ast) => Some((ast, bytes.len())),
                    Err(error) => {
                        warn!(%hash, error = %error, "failed to deserialize cached AST source; discarding entry");
                        self.discard_entry(&path);
                        None
                    }
                }
            }
            Ok(_) => {
                self.discard_entry(&path);
                None
            }
            Err(error) => {
                warn!(%hash, error = ?error, "failed to deserialize cached AST; discarding entry");
                self.discard_entry(&path);
                None
            }
        }
    }

    fn discard_entry(&self, path: &Path) {
        if let Err(error) = fs::remove_file(path) {
            if error.kind() != io::ErrorKind::NotFound {
                warn!(error = ?error, path = %path.display(), "failed to remove invalid cache entry");
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

fn serialize_entry(hash: &str, script: &str, _ast: &AST) -> Result<Vec<u8>, serde_json::Error> {
    let entry = PersistedAst {
        format_version: CACHE_FORMAT_VERSION,
        core_version: CORE_VERSION.to_string(),
        hash: hash.to_string(),
        script: script.to_string(),
    };

    serde_json::to_vec(&entry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rhai::Engine;
    use std::fs;
    use tempfile::tempdir;

    fn make_cache(max_size_bytes: usize) -> (ScriptCache, tempfile::TempDir) {
        let dir = tempdir().expect("tempdir");
        let cache = ScriptCache::with_limits(dir.path().to_path_buf(), max_size_bytes, 8);
        (cache, dir)
    }

    fn compile_ast(script: &str) -> AST {
        Engine::new().compile(script).expect("compile")
    }

    fn serialized_len(script: &str, ast: &AST) -> usize {
        serialize_entry(&cache_key(script), script, ast)
            .expect("serialize")
            .len()
    }

    #[test]
    fn cache_hit_returns_ast() {
        let (cache, _dir) = make_cache(1024);
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
        let (cache, _dir) = make_cache(1024);
        assert!(cache.get("not cached").is_none());

        let stats = cache.stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn lru_eviction_respects_byte_budget() {
        let script_a = "let a = 11;";
        let script_b = "let b = 22;";
        let script_c = "let c = 33;";

        let ast_a = compile_ast(script_a);
        let ast_b = compile_ast(script_b);
        let ast_c = compile_ast(script_c);

        let size_a = serialized_len(script_a, &ast_a);
        let size_b = serialized_len(script_b, &ast_b);
        let budget = size_a + size_b;
        let (cache, _dir) = make_cache(budget);

        cache.put(script_a, &ast_a);
        cache.put(script_b, &ast_b);
        cache.put(script_c, &ast_c);

        let hash_a = cache_key(script_a);
        let hash_b = cache_key(script_b);
        let hash_c = cache_key(script_c);

        let index = cache.index.lock().expect("index lock");
        let present = usize::from(index.entries.contains(&hash_a))
            + usize::from(index.entries.contains(&hash_b))
            + usize::from(index.entries.contains(&hash_c));
        assert!(present <= 2);
        assert!(index.evictions >= 1);
        assert!(index.size_bytes <= budget);
    }

    #[test]
    fn cache_updates_recency_on_get() {
        let script_a = "let a = 100;";
        let script_b = "let b = 200;";
        let script_c = "let c = 300;";

        let ast_a = compile_ast(script_a);
        let ast_b = compile_ast(script_b);
        let ast_c = compile_ast(script_c);
        let large_script = "let really_big_value = 1234567890;";
        let large_ast = compile_ast(large_script);

        let budget = serialized_len(script_a, &ast_a)
            + serialized_len(script_b, &ast_b)
            + serialized_len(large_script, &large_ast) / 2;

        let (cache, _dir) = make_cache(budget);

        cache.put(script_a, &ast_a);
        cache.put(script_b, &ast_b);
        cache.put(script_c, &ast_c);

        // Access A to make it most recent
        assert!(cache.get(script_a).is_some());

        // Force eviction by exceeding byte budget with a larger script
        cache.put(large_script, &large_ast);

        let hash_a = cache_key(script_a);
        let hash_b = cache_key(script_b);
        let hash_c = cache_key(script_c);

        let index = cache.index.lock().expect("index lock");
        let present = usize::from(index.entries.contains(&hash_a))
            + usize::from(index.entries.contains(&hash_b))
            + usize::from(index.entries.contains(&hash_c));

        assert!(index.entries.contains(&hash_a));
        assert!(present <= 2);
        assert!(index.evictions >= 1);
    }

    #[test]
    fn cache_clear_resets_state() {
        let (cache, dir) = make_cache(256);
        let script = "let x = 42;";
        cache.put(script, &compile_ast(script));
        assert!(cache.get(script).is_some());

        cache.clear();
        assert!(cache.get(script).is_none());

        let stats = cache.stats();
        assert_eq!(stats.entries, 0);
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 1); // miss after clear
        assert!(!dir
            .path()
            .join(format!("{}.{}", cache_key(script), CACHE_FILE_EXTENSION))
            .exists());
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
        let (cache, _dir) = make_cache(256);
        let script = "let x = 1;";
        cache.put(script, &compile_ast(script));

        // Intentionally poison the mutex in the same thread to avoid Send requirements
        let _ = std::panic::catch_unwind(|| {
            if let Ok(_guard) = cache.index.lock() {
                panic!("intentional poison");
            }
        });

        cache.put(script, &compile_ast(script));
        assert!(cache.get(script).is_some());
    }

    #[test]
    fn cache_persists_ast_to_disk() {
        let dir = tempdir().expect("tempdir");
        let script = "let load_me = 10 + 5;";
        let ast = compile_ast(script);

        let cache = ScriptCache::with_limits(dir.path().to_path_buf(), 4096, 8);
        cache.put(script, &ast);

        let reloaded = ScriptCache::with_limits(dir.path().to_path_buf(), 4096, 8);
        let retrieved = reloaded.get(script);
        assert!(retrieved.is_some());

        let stats = reloaded.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
    }

    #[test]
    fn incompatible_cache_version_is_ignored() {
        let dir = tempdir().expect("tempdir");
        let script = "let stale = 99;";

        let entry = PersistedAst {
            format_version: CACHE_FORMAT_VERSION + 1,
            core_version: CORE_VERSION.to_string(),
            hash: cache_key(script),
            script: script.to_string(),
        };

        let encoded = serde_json::to_vec(&entry).expect("encode");
        let path = dir
            .path()
            .join(format!("{}.{}", cache_key(script), CACHE_FILE_EXTENSION));
        fs::create_dir_all(dir.path()).expect("mkdirs");
        fs::write(&path, encoded).expect("write");

        let cache = ScriptCache::with_limits(dir.path().to_path_buf(), 4096, 8);
        assert!(cache.get(script).is_none());
        assert_eq!(cache.stats().misses, 1);
        assert!(!path.exists());
    }
}
