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
use std::time::{Duration, Instant, SystemTime};
use tracing::warn;

/// Default maximum cache size in bytes (10 MiB).
pub const DEFAULT_MAX_SIZE_BYTES: usize = 10 * 1024 * 1024;

/// Default maximum number of cached entries.
pub const DEFAULT_MAX_ENTRIES: usize = 256;

/// Cache format version for persisted ASTs.
pub const CACHE_FORMAT_VERSION: u32 = 1;

/// File extension used for persisted AST cache entries.
pub const CACHE_FILE_EXTENSION: &str = "rhaiast";

/// Version of the core crate used to ensure cache compatibility across releases.
pub const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");

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
    /// Total estimated microseconds saved on startup from cache hits.
    pub startup_micros_saved: u64,
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
    compile_micros: Option<u64>,
}

#[derive(Debug)]
struct CacheIndex {
    entries: LruCache<String, CacheEntry>,
    size_bytes: usize,
    max_size_bytes: usize,
    hits: u64,
    misses: u64,
    evictions: u64,
    startup_micros_saved: u64,
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

    fn record_saved_time(&mut self, load_duration: Duration, compile_micros: Option<u64>) {
        if let Some(compile_micros) = compile_micros {
            let load_micros = load_duration.as_micros() as u64;
            let saved = compile_micros.saturating_sub(load_micros);
            self.startup_micros_saved = self.startup_micros_saved.saturating_add(saved);
        }
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
        self.startup_micros_saved = 0;
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PersistedAst {
    pub format_version: u32,
    pub core_version: String,
    pub hash: String,
    pub script: String,
    #[serde(default)]
    pub compile_micros: Option<u64>,
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
            startup_micros_saved: 0,
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
        let lookup_start = Instant::now();

        {
            let mut index = self.lock_index();
            if let Some(entry) = index.entries.get_mut(&hash) {
                entry.last_used = SystemTime::now();
                let ast = entry.ast.clone();
                let compile_micros = entry.compile_micros;
                index.record_hit();
                index.record_saved_time(lookup_start.elapsed(), compile_micros);
                return Some(ast);
            }
        }

        let disk_start = Instant::now();
        if let Some((ast, size_bytes, compile_micros)) = self.load_from_disk(&hash) {
            let mut index = self.lock_index();
            self.upsert_entry(&mut index, hash, ast.clone(), size_bytes, compile_micros);
            index.record_hit();
            index.record_saved_time(disk_start.elapsed(), compile_micros);
            return Some(ast);
        }

        let mut index = self.lock_index();
        index.record_miss();
        None
    }

    /// Store a compiled AST keyed by its content hash.
    pub fn put(&self, script: &str, ast: &AST, compile_micros: Option<u64>) {
        let hash = cache_key(script);

        let (serialized, size_bytes) = match serialize_entry(&hash, script, ast, compile_micros) {
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
            self.upsert_entry(
                &mut index,
                hash.clone(),
                ast.clone(),
                size_bytes,
                compile_micros,
            );
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
        index.startup_micros_saved = 0;

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
            startup_micros_saved: index.startup_micros_saved,
        }
    }

    fn upsert_entry(
        &self,
        index: &mut CacheIndex,
        hash: String,
        ast: AST,
        size_bytes: usize,
        compile_micros: Option<u64>,
    ) {
        let compile_micros =
            compile_micros.or_else(|| index.entries.peek(&hash).and_then(|e| e.compile_micros));
        if let Some(replaced) = index.entries.put(
            hash,
            CacheEntry {
                ast,
                size_bytes,
                last_used: SystemTime::now(),
                compile_micros,
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

    fn load_from_disk(&self, hash: &str) -> Option<(AST, usize, Option<u64>)> {
        let path = self.cache_file_path(hash);
        let bytes = fs::read(&path).ok()?;

        match serde_json::from_slice::<PersistedAst>(&bytes) {
            Ok(entry)
                if entry.format_version == CACHE_FORMAT_VERSION
                    && entry.core_version == CORE_VERSION
                    && entry.hash == hash =>
            {
                let compile_start = Instant::now();
                match Engine::new().compile(&entry.script) {
                    Ok(ast) => {
                        let compile_micros = entry
                            .compile_micros
                            .or_else(|| Some(compile_start.elapsed().as_micros() as u64));
                        Some((ast, bytes.len(), compile_micros))
                    }
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

pub fn serialize_entry(
    hash: &str,
    script: &str,
    _ast: &AST,
    compile_micros: Option<u64>,
) -> Result<Vec<u8>, serde_json::Error> {
    let entry = PersistedAst {
        format_version: CACHE_FORMAT_VERSION,
        core_version: CORE_VERSION.to_string(),
        hash: hash.to_string(),
        script: script.to_string(),
        compile_micros,
    };

    serde_json::to_vec(&entry)
}
