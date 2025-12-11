//! Ring buffer implementation for transition logging.
//!
//! This module provides the `TransitionLog` struct which maintains a bounded
//! history of state transitions using a ring buffer for efficient memory usage.

use super::entry::TransitionEntry;
use crate::engine::transitions::transition::TransitionCategory;

/// A bounded ring buffer for storing transition history.
///
/// `TransitionLog` maintains a fixed-size history of state transitions using
/// a ring buffer. Once the buffer is full, new entries overwrite the oldest ones.
/// This ensures bounded memory usage while providing recent transition history
/// for debugging and analysis.
///
/// # Features
///
/// - **Ring buffer**: Fixed memory footprint, oldest entries are overwritten
/// - **Search**: Filter entries by category, transition name, or time range
/// - **Export**: Serialize to JSON for external analysis tools
/// - **Thread-safe**: Can be shared across threads (with proper synchronization)
///
/// # Feature Flag
///
/// This type is only available when the `transition-logging` feature is enabled.
/// When the feature is disabled, a zero-overhead stub implementation is used.
///
/// # Example
///
/// ```no_run
/// use keyrx_core::engine::transitions::log::{TransitionLog, TransitionEntry};
/// use keyrx_core::engine::transitions::transition::{StateTransition, TransitionCategory};
/// use keyrx_core::engine::state::snapshot::StateSnapshot;
/// use keyrx_core::engine::KeyCode;
///
/// // Create a log with capacity for 1000 entries
/// let mut log = TransitionLog::new(1000);
///
/// // Add an entry
/// let entry = TransitionEntry::new(
///     StateTransition::KeyPressed { key: KeyCode::A, timestamp: 1000 },
///     StateSnapshot::empty(),
///     StateSnapshot::empty(),
///     1000000,
///     5000,
/// );
/// log.push(entry);
///
/// // Search for engine-related transitions
/// let engine_transitions = log.search_by_category(TransitionCategory::Engine);
///
/// // Export to JSON
/// let json = log.export_json().unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct TransitionLog {
    /// Ring buffer of transition entries.
    entries: Vec<TransitionEntry>,

    /// Current write position in the ring buffer.
    write_pos: usize,

    /// Total number of entries ever added (wraps on overflow).
    total_count: u64,

    /// Maximum capacity of the log.
    capacity: usize,
}

impl TransitionLog {
    /// Create a new transition log with the specified capacity.
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of entries to store (must be > 0)
    ///
    /// # Panics
    ///
    /// Panics if capacity is 0.
    pub fn new(capacity: usize) -> Self {
        assert!(
            capacity > 0,
            "TransitionLog capacity must be greater than 0"
        );
        Self {
            entries: Vec::with_capacity(capacity),
            write_pos: 0,
            total_count: 0,
            capacity,
        }
    }

    /// Add a new entry to the log.
    ///
    /// If the log is full, this will overwrite the oldest entry.
    pub fn push(&mut self, entry: TransitionEntry) {
        if self.entries.len() < self.capacity {
            // Still filling initial capacity
            self.entries.push(entry);
            self.write_pos = self.entries.len() % self.capacity;
        } else {
            // Ring buffer is full, overwrite oldest
            self.entries[self.write_pos] = entry;
            self.write_pos = (self.write_pos + 1) % self.capacity;
        }
        self.total_count = self.total_count.wrapping_add(1);
    }

    /// Get the number of entries currently stored.
    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the log is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get the maximum capacity of the log.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get the total number of entries ever added.
    ///
    /// This count wraps on overflow but provides a sense of total activity.
    #[inline]
    pub fn total_count(&self) -> u64 {
        self.total_count
    }

    /// Check if the log has wrapped around (overwritten old entries).
    #[inline]
    pub fn has_wrapped(&self) -> bool {
        self.total_count > self.capacity as u64
    }

    /// Get an iterator over all entries in chronological order.
    ///
    /// Returns entries from oldest to newest.
    pub fn iter(&self) -> Box<dyn Iterator<Item = &TransitionEntry> + '_> {
        if self.entries.len() < self.capacity {
            // Not full yet, entries are in order
            Box::new(self.entries.iter())
        } else {
            // Ring buffer wrapped, need to iterate starting from write_pos
            let (older, newer) = self.entries.split_at(self.write_pos);
            Box::new(newer.iter().chain(older.iter()))
        }
    }

    /// Get the most recent entry, if any.
    pub fn last(&self) -> Option<&TransitionEntry> {
        if self.entries.is_empty() {
            None
        } else if self.entries.len() < self.capacity {
            self.entries.last()
        } else {
            // Ring buffer wrapped, last entry is before write_pos
            let last_idx = if self.write_pos == 0 {
                self.capacity - 1
            } else {
                self.write_pos - 1
            };
            Some(&self.entries[last_idx])
        }
    }

    /// Clear all entries from the log.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.write_pos = 0;
    }

    /// Search for entries matching a specific transition category.
    pub fn search_by_category(&self, category: TransitionCategory) -> Vec<&TransitionEntry> {
        self.iter()
            .filter(|entry| entry.category() == category)
            .collect()
    }

    /// Search for entries matching a specific transition name.
    pub fn search_by_name(&self, name: &str) -> Vec<&TransitionEntry> {
        self.iter().filter(|entry| entry.name() == name).collect()
    }

    /// Search for entries within a wall time range.
    ///
    /// # Arguments
    ///
    /// * `start_us` - Start of time range (microseconds since epoch), inclusive
    /// * `end_us` - End of time range (microseconds since epoch), inclusive
    pub fn search_by_time_range(&self, start_us: u64, end_us: u64) -> Vec<&TransitionEntry> {
        self.iter()
            .filter(|entry| entry.wall_time_us >= start_us && entry.wall_time_us <= end_us)
            .collect()
    }

    /// Search for entries that changed the state version.
    pub fn search_version_changes(&self) -> Vec<&TransitionEntry> {
        self.iter()
            .filter(|entry| entry.changed_version())
            .collect()
    }

    /// Search for entries by custom predicate.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use keyrx_core::engine::transitions::log::TransitionLog;
    /// # let log = TransitionLog::new(100);
    /// // Find all slow transitions (> 1ms)
    /// let slow = log.search(|entry| entry.duration_ns > 1_000_000);
    /// ```
    pub fn search<F>(&self, predicate: F) -> Vec<&TransitionEntry>
    where
        F: Fn(&TransitionEntry) -> bool,
    {
        self.iter().filter(|entry| predicate(entry)).collect()
    }

    /// Export all entries to JSON.
    ///
    /// Returns a JSON string containing all entries in chronological order.
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        let entries: Vec<&TransitionEntry> = self.iter().collect();
        serde_json::to_string(&entries)
    }

    /// Export all entries to pretty-printed JSON.
    ///
    /// Returns a formatted JSON string with indentation for human readability.
    pub fn export_json_pretty(&self) -> Result<String, serde_json::Error> {
        let entries: Vec<&TransitionEntry> = self.iter().collect();
        serde_json::to_string_pretty(&entries)
    }

    /// Export filtered entries to JSON.
    ///
    /// # Arguments
    ///
    /// * `entries` - The entries to export (typically from a search result)
    pub fn export_entries_json(entries: &[&TransitionEntry]) -> Result<String, serde_json::Error> {
        serde_json::to_string(entries)
    }

    /// Export filtered entries to pretty-printed JSON.
    pub fn export_entries_json_pretty(
        entries: &[&TransitionEntry],
    ) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(entries)
    }

    /// Get statistics about the log contents.
    ///
    /// Returns a tuple of:
    /// - Total entries currently stored
    /// - Number of unique transition names
    /// - Total processing time (sum of all durations)
    /// - Average processing time per entry
    pub fn statistics(&self) -> (usize, usize, u64, u64) {
        let total = self.len();
        if total == 0 {
            return (0, 0, 0, 0);
        }

        let mut names = std::collections::HashSet::new();
        let mut total_duration = 0u64;

        for entry in self.iter() {
            names.insert(entry.name());
            total_duration = total_duration.saturating_add(entry.duration_ns);
        }

        let avg_duration = total_duration / total as u64;

        (total, names.len(), total_duration, avg_duration)
    }
}

impl Default for TransitionLog {
    /// Create a default transition log with capacity for 10,000 entries.
    fn default() -> Self {
        Self::new(10_000)
    }
}
