//! Zero-overhead stub implementation when logging is disabled.
//!
//! When the `transition-logging` feature is disabled, this stub implementation
//! is used instead. All methods are no-ops and will be optimized away by the
//! compiler, resulting in zero runtime overhead.

use super::entry::TransitionEntry;
use crate::engine::transitions::transition::TransitionCategory;

/// Zero-overhead stub for TransitionLog when logging is disabled.
///
/// When the `transition-logging` feature is disabled, this stub implementation
/// is used instead. All methods are no-ops and will be optimized away by the
/// compiler, resulting in zero runtime overhead.
#[derive(Debug, Clone, Default)]
pub struct TransitionLog {
    // Zero-sized type - no memory overhead
    _marker: std::marker::PhantomData<()>,
}

impl TransitionLog {
    /// Create a new transition log (no-op when logging is disabled).
    #[inline(always)]
    pub fn new(_capacity: usize) -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }

    /// Add a new entry to the log (no-op when logging is disabled).
    #[inline(always)]
    pub fn push(&mut self, _entry: TransitionEntry) {
        // No-op: compiler will optimize this away
    }

    /// Get the number of entries currently stored (always 0 when disabled).
    #[inline(always)]
    pub fn len(&self) -> usize {
        0
    }

    /// Check if the log is empty (always true when disabled).
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        true
    }

    /// Get the maximum capacity of the log (always 0 when disabled).
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        0
    }

    /// Get the total number of entries ever added (always 0 when disabled).
    #[inline(always)]
    pub fn total_count(&self) -> u64 {
        0
    }

    /// Check if the log has wrapped around (always false when disabled).
    #[inline(always)]
    pub fn has_wrapped(&self) -> bool {
        false
    }

    /// Get an iterator over all entries (always empty when disabled).
    #[inline(always)]
    pub fn iter(&self) -> Box<dyn Iterator<Item = &TransitionEntry> + '_> {
        Box::new(std::iter::empty())
    }

    /// Get the most recent entry (always None when disabled).
    #[inline(always)]
    pub fn last(&self) -> Option<&TransitionEntry> {
        None
    }

    /// Clear all entries from the log (no-op when disabled).
    #[inline(always)]
    pub fn clear(&mut self) {
        // No-op: compiler will optimize this away
    }

    /// Search for entries matching a specific transition category (always empty when disabled).
    #[inline(always)]
    pub fn search_by_category(&self, _category: TransitionCategory) -> Vec<&TransitionEntry> {
        Vec::new()
    }

    /// Search for entries matching a specific transition name (always empty when disabled).
    #[inline(always)]
    pub fn search_by_name(&self, _name: &str) -> Vec<&TransitionEntry> {
        Vec::new()
    }

    /// Search for entries within a wall time range (always empty when disabled).
    #[inline(always)]
    pub fn search_by_time_range(&self, _start_us: u64, _end_us: u64) -> Vec<&TransitionEntry> {
        Vec::new()
    }

    /// Search for entries that changed the state version (always empty when disabled).
    #[inline(always)]
    pub fn search_version_changes(&self) -> Vec<&TransitionEntry> {
        Vec::new()
    }

    /// Search for entries by custom predicate (always empty when disabled).
    #[inline(always)]
    pub fn search<F>(&self, _predicate: F) -> Vec<&TransitionEntry>
    where
        F: Fn(&TransitionEntry) -> bool,
    {
        Vec::new()
    }

    /// Export all entries to JSON (returns empty array when disabled).
    #[inline(always)]
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        Ok("[]".to_string())
    }

    /// Export all entries to pretty-printed JSON (returns empty array when disabled).
    #[inline(always)]
    pub fn export_json_pretty(&self) -> Result<String, serde_json::Error> {
        Ok("[]".to_string())
    }

    /// Export filtered entries to JSON (returns empty array when disabled).
    #[inline(always)]
    pub fn export_entries_json(_entries: &[&TransitionEntry]) -> Result<String, serde_json::Error> {
        Ok("[]".to_string())
    }

    /// Export filtered entries to pretty-printed JSON (returns empty array when disabled).
    #[inline(always)]
    pub fn export_entries_json_pretty(
        _entries: &[&TransitionEntry],
    ) -> Result<String, serde_json::Error> {
        Ok("[]".to_string())
    }

    /// Get statistics about the log contents (always zeros when disabled).
    #[inline(always)]
    pub fn statistics(&self) -> (usize, usize, u64, u64) {
        (0, 0, 0, 0)
    }
}
