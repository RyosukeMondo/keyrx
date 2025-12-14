//! Incremental code generation with SHA256 tracking
//!
//! Tracks hashes of contract files and generated outputs to skip regeneration
//! when nothing has changed, speeding up watch mode and development.

use crate::cache::{CacheEntry, CodegenCache};
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

/// Result of checking if regeneration is needed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegenerationStatus {
    /// File needs regeneration (contract changed or not in cache)
    Required,
    /// File can be skipped (hash matches cache)
    CanSkip,
}

/// Statistics for incremental generation
#[derive(Debug, Default, Clone)]
pub struct IncrementalStats {
    /// Number of files generated
    pub generated: usize,
    /// Number of files skipped
    pub skipped: usize,
    /// Time saved by skipping (estimated in milliseconds)
    pub time_saved_ms: u64,
}

impl IncrementalStats {
    /// Add a generated file to stats
    pub fn add_generated(&mut self) {
        self.generated += 1;
    }

    /// Add a skipped file to stats
    pub fn add_skipped(&mut self, estimated_time_ms: u64) {
        self.skipped += 1;
        self.time_saved_ms += estimated_time_ms;
    }

    /// Get total files processed
    pub fn total(&self) -> usize {
        self.generated + self.skipped
    }

    /// Get time saved in seconds
    pub fn time_saved_seconds(&self) -> f64 {
        self.time_saved_ms as f64 / 1000.0
    }
}

/// Incremental generation manager
pub struct IncrementalGenerator {
    cache: CodegenCache,
    force: bool,
}

impl IncrementalGenerator {
    /// Create a new incremental generator
    pub fn new(cache_path: &Path, force: bool) -> Result<Self> {
        let cache = if force {
            // Force mode: start with empty cache
            CodegenCache::new()
        } else {
            // Normal mode: load existing cache
            CodegenCache::load(cache_path)
                .with_context(|| format!("Failed to load cache from: {}", cache_path.display()))?
        };

        Ok(Self { cache, force })
    }

    /// Check if a contract needs regeneration
    pub fn needs_regeneration(
        &self,
        contract_path: &Path,
        contract_content: &str,
    ) -> Result<RegenerationStatus> {
        if self.force {
            return Ok(RegenerationStatus::Required);
        }

        // Calculate hash of current contract
        let current_hash = Self::hash_content(contract_content);

        // Get cached entry
        let contract_path_str = contract_path.to_string_lossy().to_string();
        let cached_entry = self.cache.get(&contract_path_str);

        match cached_entry {
            Some(entry) if entry.contract_hash == current_hash => {
                // Hash matches, can skip
                Ok(RegenerationStatus::CanSkip)
            }
            _ => {
                // No cache entry or hash mismatch, needs regeneration
                Ok(RegenerationStatus::Required)
            }
        }
    }

    /// Update cache after generating code for a contract
    pub fn update_cache(
        &mut self,
        contract_path: &Path,
        contract_content: &str,
        bindings_content: Option<&str>,
        models_content: Option<&str>,
    ) {
        let contract_hash = Self::hash_content(contract_content);
        let bindings_hash = bindings_content.map(Self::hash_content);
        let models_hash = models_content.map(Self::hash_content);

        let entry = CacheEntry {
            contract_hash,
            bindings_hash,
            models_hash,
            last_generated: chrono::Utc::now(),
        };

        let contract_path_str = contract_path.to_string_lossy().to_string();
        self.cache.update(contract_path_str, entry);
    }

    /// Save cache to disk
    pub fn save(&self, cache_path: &Path) -> Result<()> {
        self.cache.save(cache_path)
    }

    /// Calculate SHA256 hash of content
    fn hash_content(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Calculate SHA256 hash of a file
    pub fn hash_file(path: &Path) -> Result<String> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file for hashing: {}", path.display()))?;
        Ok(Self::hash_content(&content))
    }

    /// Clear the cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_hash_content() {
        let content1 = "test content";
        let content2 = "test content";
        let content3 = "different content";

        let hash1 = IncrementalGenerator::hash_content(content1);
        let hash2 = IncrementalGenerator::hash_content(content2);
        let hash3 = IncrementalGenerator::hash_content(content3);

        // Same content should produce same hash
        assert_eq!(hash1, hash2);
        // Different content should produce different hash
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_hash_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        fs::write(&file_path, "test content").unwrap();

        let hash = IncrementalGenerator::hash_file(&file_path).unwrap();
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // SHA256 produces 64 hex chars
    }

    #[test]
    fn test_needs_regeneration_no_cache() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("cache.json");

        let generator = IncrementalGenerator::new(&cache_path, false).unwrap();

        let contract_path = temp_dir.path().join("test.json");
        let status = generator
            .needs_regeneration(&contract_path, "test content")
            .unwrap();

        assert_eq!(status, RegenerationStatus::Required);
    }

    #[test]
    fn test_needs_regeneration_with_cache() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("cache.json");
        let contract_path = temp_dir.path().join("test.json");

        // First run: generate and cache
        let mut generator = IncrementalGenerator::new(&cache_path, false).unwrap();
        let content = "test content";

        generator.update_cache(&contract_path, content, None, None);
        generator.save(&cache_path).unwrap();

        // Second run: should skip
        let generator2 = IncrementalGenerator::new(&cache_path, false).unwrap();
        let status = generator2.needs_regeneration(&contract_path, content).unwrap();

        assert_eq!(status, RegenerationStatus::CanSkip);
    }

    #[test]
    fn test_needs_regeneration_content_changed() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("cache.json");
        let contract_path = temp_dir.path().join("test.json");

        // First run: generate and cache
        let mut generator = IncrementalGenerator::new(&cache_path, false).unwrap();
        generator.update_cache(&contract_path, "old content", None, None);
        generator.save(&cache_path).unwrap();

        // Second run with changed content: should regenerate
        let generator2 = IncrementalGenerator::new(&cache_path, false).unwrap();
        let status = generator2
            .needs_regeneration(&contract_path, "new content")
            .unwrap();

        assert_eq!(status, RegenerationStatus::Required);
    }

    #[test]
    fn test_force_mode() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("cache.json");
        let contract_path = temp_dir.path().join("test.json");

        // Create cache
        let mut generator = IncrementalGenerator::new(&cache_path, false).unwrap();
        generator.update_cache(&contract_path, "test content", None, None);
        generator.save(&cache_path).unwrap();

        // Force mode: should always regenerate even if cached
        let generator_force = IncrementalGenerator::new(&cache_path, true).unwrap();
        let status = generator_force
            .needs_regeneration(&contract_path, "test content")
            .unwrap();

        assert_eq!(status, RegenerationStatus::Required);
    }

    #[test]
    fn test_incremental_stats() {
        let mut stats = IncrementalStats::default();

        assert_eq!(stats.generated, 0);
        assert_eq!(stats.skipped, 0);
        assert_eq!(stats.total(), 0);

        stats.add_generated();
        stats.add_generated();
        stats.add_skipped(100);
        stats.add_skipped(200);

        assert_eq!(stats.generated, 2);
        assert_eq!(stats.skipped, 2);
        assert_eq!(stats.total(), 4);
        assert_eq!(stats.time_saved_ms, 300);
        assert_eq!(stats.time_saved_seconds(), 0.3);
    }

    #[test]
    fn test_clear_cache() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("cache.json");

        let mut generator = IncrementalGenerator::new(&cache_path, false).unwrap();
        generator.update_cache(
            &temp_dir.path().join("test.json"),
            "test content",
            None,
            None,
        );

        assert_eq!(generator.cache_size(), 1);

        generator.clear_cache();
        assert_eq!(generator.cache_size(), 0);
    }
}
