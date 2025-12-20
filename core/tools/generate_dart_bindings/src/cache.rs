//! Persistent cache for incremental code generation
//!
//! Stores SHA256 hashes of contract files and Rust sources to skip regeneration
//! when nothing has changed.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Cache file format version for forward compatibility
const CACHE_VERSION: u32 = 1;

/// Cache entry containing hashes for a single contract
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CacheEntry {
    /// SHA256 hash of the contract JSON content
    pub contract_hash: String,
    /// SHA256 hash of the generated bindings file (if it exists)
    pub bindings_hash: Option<String>,
    /// SHA256 hash of the generated models file (if it exists)
    pub models_hash: Option<String>,
    /// Timestamp of last generation (for debugging)
    pub last_generated: chrono::DateTime<chrono::Utc>,
}

/// Persistent cache storage for code generation
#[derive(Debug, Serialize, Deserialize)]
pub struct CodegenCache {
    /// Cache format version
    version: u32,
    /// Map from contract file path to cache entry
    entries: HashMap<String, CacheEntry>,
}

impl Default for CodegenCache {
    fn default() -> Self {
        Self::new()
    }
}

impl CodegenCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self {
            version: CACHE_VERSION,
            entries: HashMap::new(),
        }
    }

    /// Load cache from disk, creating new one if file doesn't exist or is corrupted
    pub fn load(cache_path: &Path) -> Result<Self> {
        if !cache_path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(cache_path)
            .with_context(|| format!("Failed to read cache file: {}", cache_path.display()))?;

        let cache: CodegenCache = serde_json::from_str(&content).unwrap_or_else(|e| {
            #[allow(clippy::print_stderr)]
            {
                eprintln!(
                    "Warning: Failed to parse cache file ({}), starting fresh",
                    e
                );
            }
            Self::new()
        });

        // Check version compatibility
        if cache.version != CACHE_VERSION {
            #[allow(clippy::print_stderr)]
            {
                eprintln!(
                    "Warning: Cache version mismatch (expected {}, got {}), starting fresh",
                    CACHE_VERSION, cache.version
                );
            }
            return Ok(Self::new());
        }

        Ok(cache)
    }

    /// Save cache to disk
    pub fn save(&self, cache_path: &Path) -> Result<()> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create cache directory: {}", parent.display())
            })?;
        }

        let content =
            serde_json::to_string_pretty(self).with_context(|| "Failed to serialize cache")?;

        fs::write(cache_path, content)
            .with_context(|| format!("Failed to write cache file: {}", cache_path.display()))?;

        Ok(())
    }

    /// Get cache entry for a contract file
    pub fn get(&self, contract_path: &str) -> Option<&CacheEntry> {
        self.entries.get(contract_path)
    }

    /// Update cache entry for a contract file
    pub fn update(&mut self, contract_path: String, entry: CacheEntry) {
        self.entries.insert(contract_path, entry);
    }

    /// Remove cache entry for a contract file
    pub fn remove(&mut self, contract_path: &str) {
        self.entries.remove(contract_path);
    }

    /// Clear all cache entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Get number of cached entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get default cache file path
    pub fn default_path() -> PathBuf {
        PathBuf::from(".cache/codegen/hashes.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_new_cache() {
        let cache = CodegenCache::new();
        assert_eq!(cache.version, CACHE_VERSION);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_operations() {
        let mut cache = CodegenCache::new();

        let entry = CacheEntry {
            contract_hash: "abc123".to_string(),
            bindings_hash: Some("def456".to_string()),
            models_hash: Some("ghi789".to_string()),
            last_generated: chrono::Utc::now(),
        };

        // Insert entry
        cache.update("test_contract.json".to_string(), entry.clone());
        assert_eq!(cache.len(), 1);

        // Retrieve entry
        let retrieved = cache.get("test_contract.json");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().contract_hash, "abc123");

        // Remove entry
        cache.remove("test_contract.json");
        assert!(cache.is_empty());
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("test_cache.json");

        let mut cache = CodegenCache::new();
        let entry = CacheEntry {
            contract_hash: "test_hash".to_string(),
            bindings_hash: None,
            models_hash: None,
            last_generated: chrono::Utc::now(),
        };
        cache.update("test.json".to_string(), entry);

        // Save cache
        cache.save(&cache_path).unwrap();
        assert!(cache_path.exists());

        // Load cache
        let loaded = CodegenCache::load(&cache_path).unwrap();
        assert_eq!(loaded.len(), 1);
        assert!(loaded.get("test.json").is_some());
    }

    #[test]
    fn test_load_nonexistent_cache() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("nonexistent.json");

        let cache = CodegenCache::load(&cache_path).unwrap();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_load_corrupted_cache() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("corrupted.json");

        // Write invalid JSON
        fs::write(&cache_path, "{ invalid json }").unwrap();

        // Should return new cache instead of failing
        let cache = CodegenCache::load(&cache_path).unwrap();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_clear_cache() {
        let mut cache = CodegenCache::new();
        cache.update(
            "test1.json".to_string(),
            CacheEntry {
                contract_hash: "hash1".to_string(),
                bindings_hash: None,
                models_hash: None,
                last_generated: chrono::Utc::now(),
            },
        );
        cache.update(
            "test2.json".to_string(),
            CacheEntry {
                contract_hash: "hash2".to_string(),
                bindings_hash: None,
                models_hash: None,
                last_generated: chrono::Utc::now(),
            },
        );

        assert_eq!(cache.len(), 2);
        cache.clear();
        assert!(cache.is_empty());
    }
}
