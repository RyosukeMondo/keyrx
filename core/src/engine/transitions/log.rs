//! State transition logging for debugging and replay.
//!
//! This module provides types for logging state transitions with complete
//! before/after state snapshots. This enables:
//! - Debugging state machine behavior
//! - Replay of transition sequences
//! - Analysis of state evolution
//! - Export of transition history
//!
//! # Feature Flag
//!
//! The logging functionality can be disabled at compile time using the
//! `transition-logging` feature flag. When disabled, all logging code is
//! removed at compile time with zero runtime overhead.
//!
//! To disable transition logging:
//! ```toml
//! [dependencies]
//! keyrx_core = { version = "...", default-features = false, features = [...] }
//! ```

use serde::{Deserialize, Serialize};

use crate::engine::state::snapshot::StateSnapshot;

use super::transition::{StateTransition, TransitionCategory};

/// A single entry in the transition log.
///
/// TransitionEntry captures a complete state transition with:
/// - The transition that occurred
/// - State before the transition
/// - State after the transition
/// - Timing information
///
/// This enables full reconstruction of state evolution and detailed
/// debugging of state machine behavior.
///
/// # Example
///
/// ```no_run
/// use keyrx_core::engine::transitions::log::TransitionEntry;
/// use keyrx_core::engine::transitions::transition::StateTransition;
/// use keyrx_core::engine::state::snapshot::StateSnapshot;
/// use keyrx_core::engine::KeyCode;
///
/// let entry = TransitionEntry {
///     transition: StateTransition::KeyPressed {
///         key: KeyCode::A,
///         timestamp: 1000,
///     },
///     state_before: StateSnapshot::empty(),
///     state_after: StateSnapshot::empty(),
///     wall_time_us: 1000000,
///     duration_ns: 5000,
/// };
///
/// // Serialize to JSON for export
/// let json = serde_json::to_string(&entry).unwrap();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionEntry {
    /// The state transition that occurred.
    pub transition: StateTransition,

    /// State snapshot before the transition was applied.
    pub state_before: StateSnapshot,

    /// State snapshot after the transition was applied.
    pub state_after: StateSnapshot,

    /// Wall clock time when the transition occurred (microseconds since epoch).
    ///
    /// This is the real-world time when the transition was recorded,
    /// distinct from any event timestamps embedded in the transition itself.
    pub wall_time_us: u64,

    /// Duration of the transition processing in nanoseconds.
    ///
    /// This measures how long it took to apply the transition and
    /// update the state, useful for performance analysis.
    pub duration_ns: u64,
}

impl TransitionEntry {
    /// Create a new transition entry.
    ///
    /// # Arguments
    ///
    /// * `transition` - The state transition that occurred
    /// * `state_before` - State snapshot before applying the transition
    /// * `state_after` - State snapshot after applying the transition
    /// * `wall_time_us` - Wall clock time (microseconds since epoch)
    /// * `duration_ns` - Processing duration in nanoseconds
    pub fn new(
        transition: StateTransition,
        state_before: StateSnapshot,
        state_after: StateSnapshot,
        wall_time_us: u64,
        duration_ns: u64,
    ) -> Self {
        Self {
            transition,
            state_before,
            state_after,
            wall_time_us,
            duration_ns,
        }
    }

    /// Get the category of this transition.
    #[inline]
    pub fn category(&self) -> TransitionCategory {
        self.transition.category()
    }

    /// Get the name of this transition.
    #[inline]
    pub fn name(&self) -> &'static str {
        self.transition.name()
    }

    /// Get the event timestamp embedded in the transition, if available.
    ///
    /// This is distinct from `wall_time_us` - the event timestamp comes from
    /// the input device or timing source, while wall_time_us is when we
    /// recorded the log entry.
    #[inline]
    pub fn event_timestamp(&self) -> Option<u64> {
        self.transition.timestamp()
    }

    /// Get the state version before the transition.
    #[inline]
    pub fn version_before(&self) -> u64 {
        self.state_before.version
    }

    /// Get the state version after the transition.
    #[inline]
    pub fn version_after(&self) -> u64 {
        self.state_after.version
    }

    /// Check if this transition changed the state version.
    ///
    /// Most transitions will increment the version, but some may not
    /// if they're no-ops or redundant.
    #[inline]
    pub fn changed_version(&self) -> bool {
        self.version_after() > self.version_before()
    }

    /// Get a summary of state changes between before and after.
    ///
    /// Returns a tuple of:
    /// - Number of keys changed (pressed or released)
    /// - Number of layers changed (pushed or popped)
    /// - Whether modifiers changed
    /// - Whether pending decisions changed
    pub fn state_diff_summary(&self) -> (usize, usize, bool, bool) {
        let keys_changed = if self.state_before.pressed_keys != self.state_after.pressed_keys {
            // Count how many keys were added or removed
            let before_keys: std::collections::HashSet<_> = self
                .state_before
                .pressed_keys
                .iter()
                .map(|pk| pk.key)
                .collect();
            let after_keys: std::collections::HashSet<_> = self
                .state_after
                .pressed_keys
                .iter()
                .map(|pk| pk.key)
                .collect();
            before_keys.symmetric_difference(&after_keys).count()
        } else {
            0
        };

        let layers_changed = if self.state_before.active_layers != self.state_after.active_layers {
            let before_layers: std::collections::HashSet<_> =
                self.state_before.active_layers.iter().collect();
            let after_layers: std::collections::HashSet<_> =
                self.state_after.active_layers.iter().collect();
            before_layers.symmetric_difference(&after_layers).count()
        } else {
            0
        };

        let modifiers_changed = self.state_before.standard_modifiers
            != self.state_after.standard_modifiers
            || self.state_before.virtual_modifiers != self.state_after.virtual_modifiers;

        let pending_changed = self.state_before.pending_count != self.state_after.pending_count;

        (
            keys_changed,
            layers_changed,
            modifiers_changed,
            pending_changed,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::KeyCode;

    #[test]
    fn test_new_entry() {
        let transition = StateTransition::KeyPressed {
            key: KeyCode::A,
            timestamp: 1000,
        };
        let state_before = StateSnapshot::empty();
        let state_after = StateSnapshot::empty();

        let entry = TransitionEntry::new(
            transition,
            state_before.clone(),
            state_after.clone(),
            1000000,
            5000,
        );

        assert_eq!(entry.wall_time_us, 1000000);
        assert_eq!(entry.duration_ns, 5000);
        assert_eq!(entry.name(), "KeyPressed");
        assert_eq!(entry.event_timestamp(), Some(1000));
    }

    #[test]
    fn test_category() {
        let entry = TransitionEntry::new(
            StateTransition::KeyPressed {
                key: KeyCode::A,
                timestamp: 1000,
            },
            StateSnapshot::empty(),
            StateSnapshot::empty(),
            1000000,
            5000,
        );

        assert_eq!(entry.category(), TransitionCategory::Engine);
    }

    #[test]
    fn test_version_tracking() {
        let mut state_before = StateSnapshot::empty();
        state_before.version = 5;

        let mut state_after = StateSnapshot::empty();
        state_after.version = 6;

        let entry = TransitionEntry::new(
            StateTransition::KeyPressed {
                key: KeyCode::A,
                timestamp: 1000,
            },
            state_before,
            state_after,
            1000000,
            5000,
        );

        assert_eq!(entry.version_before(), 5);
        assert_eq!(entry.version_after(), 6);
        assert!(entry.changed_version());
    }

    #[test]
    fn test_no_version_change() {
        let mut state = StateSnapshot::empty();
        state.version = 5;

        let entry = TransitionEntry::new(
            StateTransition::EngineReset,
            state.clone(),
            state,
            1000000,
            5000,
        );

        assert_eq!(entry.version_before(), 5);
        assert_eq!(entry.version_after(), 5);
        assert!(!entry.changed_version());
    }

    #[test]
    fn test_state_diff_summary_no_changes() {
        let state = StateSnapshot::empty();

        let entry = TransitionEntry::new(
            StateTransition::ConfigReloaded,
            state.clone(),
            state,
            1000000,
            5000,
        );

        let (keys, layers, mods, pending) = entry.state_diff_summary();
        assert_eq!(keys, 0);
        assert_eq!(layers, 0);
        assert!(!mods);
        assert!(!pending);
    }

    #[test]
    fn test_state_diff_summary_key_changes() {
        use crate::engine::state::snapshot::PressedKey;

        let state_before = StateSnapshot::empty();
        let state_after = StateSnapshot::with_keys(vec![PressedKey {
            key: KeyCode::A,
            pressed_at: 1000,
        }]);

        let entry = TransitionEntry::new(
            StateTransition::KeyPressed {
                key: KeyCode::A,
                timestamp: 1000,
            },
            state_before,
            state_after,
            1000000,
            5000,
        );

        let (keys, layers, mods, pending) = entry.state_diff_summary();
        assert_eq!(keys, 1);
        assert_eq!(layers, 0);
        assert!(!mods);
        assert!(!pending);
    }

    #[test]
    fn test_state_diff_summary_layer_changes() {
        let state_before = StateSnapshot::with_layers(vec![0]);
        let state_after = StateSnapshot::with_layers(vec![0, 1]);

        let entry = TransitionEntry::new(
            StateTransition::LayerPushed { layer: 1 },
            state_before,
            state_after,
            1000000,
            5000,
        );

        let (keys, layers, mods, pending) = entry.state_diff_summary();
        assert_eq!(keys, 0);
        assert_eq!(layers, 1);
        assert!(!mods);
        assert!(!pending);
    }

    #[test]
    fn test_state_diff_summary_pending_changes() {
        let state_before = StateSnapshot::empty();
        let mut state_after = StateSnapshot::empty();
        state_after.pending_count = 1;

        let entry = TransitionEntry::new(
            StateTransition::DecisionQueued {
                id: 1,
                kind: crate::engine::transitions::transition::DecisionKind::TapHold,
            },
            state_before,
            state_after,
            1000000,
            5000,
        );

        let (keys, layers, mods, pending) = entry.state_diff_summary();
        assert_eq!(keys, 0);
        assert_eq!(layers, 0);
        assert!(!mods);
        assert!(pending);
    }

    #[test]
    fn test_serialization() {
        let entry = TransitionEntry::new(
            StateTransition::KeyPressed {
                key: KeyCode::A,
                timestamp: 1000,
            },
            StateSnapshot::empty(),
            StateSnapshot::empty(),
            1000000,
            5000,
        );

        let json = serde_json::to_string(&entry).expect("serialize");
        assert!(json.contains("\"transition\""));
        assert!(json.contains("\"state_before\""));
        assert!(json.contains("\"state_after\""));
        assert!(json.contains("\"wall_time_us\":1000000"));
        assert!(json.contains("\"duration_ns\":5000"));

        let deserialized: TransitionEntry = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.name(), entry.name());
        assert_eq!(deserialized.wall_time_us, entry.wall_time_us);
        assert_eq!(deserialized.duration_ns, entry.duration_ns);
    }

    #[test]
    fn test_event_timestamp_none() {
        let entry = TransitionEntry::new(
            StateTransition::ConfigReloaded,
            StateSnapshot::empty(),
            StateSnapshot::empty(),
            1000000,
            5000,
        );

        assert_eq!(entry.event_timestamp(), None);
    }

    #[test]
    fn test_multiple_key_changes() {
        use crate::engine::state::snapshot::PressedKey;

        let state_before = StateSnapshot::with_keys(vec![
            PressedKey {
                key: KeyCode::A,
                pressed_at: 1000,
            },
            PressedKey {
                key: KeyCode::B,
                pressed_at: 1100,
            },
        ]);

        let state_after = StateSnapshot::with_keys(vec![
            PressedKey {
                key: KeyCode::B,
                pressed_at: 1100,
            },
            PressedKey {
                key: KeyCode::C,
                pressed_at: 1200,
            },
        ]);

        let entry = TransitionEntry::new(
            StateTransition::KeyPressed {
                key: KeyCode::C,
                timestamp: 1200,
            },
            state_before,
            state_after,
            1000000,
            5000,
        );

        let (keys, _, _, _) = entry.state_diff_summary();
        // A was removed, C was added = 2 changes
        assert_eq!(keys, 2);
    }
}

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
#[cfg(feature = "transition-logging")]
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

#[cfg(feature = "transition-logging")]
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

#[cfg(feature = "transition-logging")]
impl Default for TransitionLog {
    /// Create a default transition log with capacity for 10,000 entries.
    fn default() -> Self {
        Self::new(10_000)
    }
}

// ============================================================================
// Zero-overhead stub implementation when logging is disabled
// ============================================================================

/// Zero-overhead stub for TransitionLog when logging is disabled.
///
/// When the `transition-logging` feature is disabled, this stub implementation
/// is used instead. All methods are no-ops and will be optimized away by the
/// compiler, resulting in zero runtime overhead.
#[cfg(not(feature = "transition-logging"))]
#[derive(Debug, Clone, Default)]
pub struct TransitionLog {
    // Zero-sized type - no memory overhead
    _marker: std::marker::PhantomData<()>,
}

#[cfg(not(feature = "transition-logging"))]
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

// ============================================================================
// Tests for feature flag behavior
// ============================================================================

#[cfg(test)]
mod feature_tests {
    use super::*;

    #[test]
    #[cfg(feature = "transition-logging")]
    fn test_transition_log_has_storage_when_enabled() {
        // When feature is enabled, TransitionLog should have actual storage
        let log = TransitionLog::new(100);
        assert_eq!(log.capacity(), 100);
        assert_eq!(log.len(), 0);
        assert!(std::mem::size_of_val(&log) > 0);
    }

    #[test]
    #[cfg(not(feature = "transition-logging"))]
    fn test_transition_log_is_zero_sized_when_disabled() {
        // When feature is disabled, TransitionLog should be zero-sized
        let log = TransitionLog::new(100);
        assert_eq!(log.capacity(), 0);
        assert_eq!(log.len(), 0);
        // PhantomData is zero-sized
        assert_eq!(std::mem::size_of::<TransitionLog>(), 0);
    }

    #[test]
    #[cfg(not(feature = "transition-logging"))]
    fn test_stub_methods_are_no_ops() {
        use crate::engine::KeyCode;

        let mut log = TransitionLog::new(100);

        // Push should be a no-op
        let entry = TransitionEntry::new(
            StateTransition::KeyPressed {
                key: KeyCode::A,
                timestamp: 1000,
            },
            StateSnapshot::empty(),
            StateSnapshot::empty(),
            1000000,
            5000,
        );
        log.push(entry);

        // Log should remain empty
        assert!(log.is_empty());
        assert_eq!(log.len(), 0);
        assert!(log.last().is_none());
        assert_eq!(log.total_count(), 0);

        // Search methods should return empty
        assert!(log
            .search_by_category(TransitionCategory::Engine)
            .is_empty());
        assert!(log.search_by_name("KeyPressed").is_empty());
        assert!(log.search_by_time_range(0, 1000000).is_empty());
        assert!(log.search_version_changes().is_empty());

        // Export should return empty array
        assert_eq!(log.export_json().unwrap(), "[]");
        assert_eq!(log.export_json_pretty().unwrap(), "[]");

        // Statistics should be all zeros
        let (total, unique, total_dur, avg_dur) = log.statistics();
        assert_eq!(total, 0);
        assert_eq!(unique, 0);
        assert_eq!(total_dur, 0);
        assert_eq!(avg_dur, 0);
    }
}

#[cfg(all(test, feature = "transition-logging"))]
mod transition_log_tests {
    use super::*;
    use crate::engine::KeyCode;

    fn create_test_entry(key: KeyCode, timestamp: u64, wall_time_us: u64) -> TransitionEntry {
        TransitionEntry::new(
            StateTransition::KeyPressed { key, timestamp },
            StateSnapshot::empty(),
            StateSnapshot::empty(),
            wall_time_us,
            5000,
        )
    }

    #[test]
    fn test_new_log() {
        let log = TransitionLog::new(100);
        assert_eq!(log.capacity(), 100);
        assert_eq!(log.len(), 0);
        assert!(log.is_empty());
        assert_eq!(log.total_count(), 0);
        assert!(!log.has_wrapped());
    }

    #[test]
    #[should_panic(expected = "TransitionLog capacity must be greater than 0")]
    fn test_zero_capacity_panics() {
        TransitionLog::new(0);
    }

    #[test]
    fn test_push_single_entry() {
        let mut log = TransitionLog::new(10);
        let entry = create_test_entry(KeyCode::A, 1000, 1000000);

        log.push(entry);

        assert_eq!(log.len(), 1);
        assert!(!log.is_empty());
        assert_eq!(log.total_count(), 1);
        assert!(!log.has_wrapped());
    }

    #[test]
    fn test_push_multiple_entries() {
        let mut log = TransitionLog::new(10);

        for i in 0..5 {
            let entry = create_test_entry(KeyCode::A, i * 100, i * 100000);
            log.push(entry);
        }

        assert_eq!(log.len(), 5);
        assert_eq!(log.total_count(), 5);
        assert!(!log.has_wrapped());
    }

    #[test]
    fn test_ring_buffer_wrap() {
        let mut log = TransitionLog::new(3);

        // Add 5 entries to a capacity-3 log
        for i in 0..5 {
            let entry = create_test_entry(KeyCode::A, i * 100, i * 100000);
            log.push(entry);
        }

        assert_eq!(log.len(), 3); // Still only 3 entries
        assert_eq!(log.total_count(), 5); // But 5 total added
        assert!(log.has_wrapped());
    }

    #[test]
    fn test_iter_order_before_wrap() {
        let mut log = TransitionLog::new(10);

        for i in 0..5 {
            let entry = create_test_entry(KeyCode::A, i * 100, i * 100000);
            log.push(entry);
        }

        let entries: Vec<_> = log.iter().collect();
        assert_eq!(entries.len(), 5);

        // Check chronological order
        for i in 0..5 {
            assert_eq!(entries[i].wall_time_us, i as u64 * 100000);
        }
    }

    #[test]
    fn test_iter_order_after_wrap() {
        let mut log = TransitionLog::new(3);

        // Add 5 entries (indices 0-4)
        for i in 0..5 {
            let entry = create_test_entry(KeyCode::A, i * 100, i * 100000);
            log.push(entry);
        }

        // Log should contain entries 2, 3, 4 in chronological order
        let entries: Vec<_> = log.iter().collect();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].wall_time_us, 200000); // entry 2
        assert_eq!(entries[1].wall_time_us, 300000); // entry 3
        assert_eq!(entries[2].wall_time_us, 400000); // entry 4
    }

    #[test]
    fn test_last_entry() {
        let mut log = TransitionLog::new(10);

        assert!(log.last().is_none());

        log.push(create_test_entry(KeyCode::A, 100, 100000));
        assert_eq!(log.last().unwrap().wall_time_us, 100000);

        log.push(create_test_entry(KeyCode::B, 200, 200000));
        assert_eq!(log.last().unwrap().wall_time_us, 200000);
    }

    #[test]
    fn test_last_entry_after_wrap() {
        let mut log = TransitionLog::new(3);

        for i in 0..5 {
            log.push(create_test_entry(KeyCode::A, i * 100, i * 100000));
        }

        // Last entry should be entry 4
        assert_eq!(log.last().unwrap().wall_time_us, 400000);
    }

    #[test]
    fn test_clear() {
        let mut log = TransitionLog::new(10);

        for i in 0..5 {
            log.push(create_test_entry(KeyCode::A, i * 100, i * 100000));
        }

        log.clear();

        assert_eq!(log.len(), 0);
        assert!(log.is_empty());
        assert!(log.last().is_none());
        // Note: total_count is NOT reset by clear
    }

    #[test]
    fn test_search_by_category() {
        let mut log = TransitionLog::new(10);

        log.push(create_test_entry(KeyCode::A, 100, 100000));
        log.push(TransitionEntry::new(
            StateTransition::ConfigReloaded,
            StateSnapshot::empty(),
            StateSnapshot::empty(),
            200000,
            5000,
        ));
        log.push(create_test_entry(KeyCode::B, 300, 300000));

        let engine_entries = log.search_by_category(TransitionCategory::Engine);
        assert_eq!(engine_entries.len(), 2); // 2 KeyPressed events

        let system_entries = log.search_by_category(TransitionCategory::System);
        assert_eq!(system_entries.len(), 1); // 1 ConfigReloaded event
    }

    #[test]
    fn test_search_by_name() {
        let mut log = TransitionLog::new(10);

        log.push(create_test_entry(KeyCode::A, 100, 100000));
        log.push(TransitionEntry::new(
            StateTransition::ConfigReloaded,
            StateSnapshot::empty(),
            StateSnapshot::empty(),
            200000,
            5000,
        ));
        log.push(create_test_entry(KeyCode::B, 300, 300000));

        let key_pressed = log.search_by_name("KeyPressed");
        assert_eq!(key_pressed.len(), 2);

        let config_reloaded = log.search_by_name("ConfigReloaded");
        assert_eq!(config_reloaded.len(), 1);
    }

    #[test]
    fn test_search_by_time_range() {
        let mut log = TransitionLog::new(10);

        for i in 0..5 {
            log.push(create_test_entry(KeyCode::A, i * 100, i * 100000));
        }

        // Search for entries between 150000 and 350000
        let results = log.search_by_time_range(150000, 350000);
        assert_eq!(results.len(), 2); // entries at 200000 and 300000
        assert_eq!(results[0].wall_time_us, 200000);
        assert_eq!(results[1].wall_time_us, 300000);
    }

    #[test]
    fn test_search_version_changes() {
        let mut log = TransitionLog::new(10);

        // Entry with version change
        let mut state_before1 = StateSnapshot::empty();
        state_before1.version = 1;
        let mut state_after1 = StateSnapshot::empty();
        state_after1.version = 2;

        log.push(TransitionEntry::new(
            StateTransition::KeyPressed {
                key: KeyCode::A,
                timestamp: 100,
            },
            state_before1,
            state_after1,
            100000,
            5000,
        ));

        // Entry without version change
        let state = StateSnapshot::empty();
        log.push(TransitionEntry::new(
            StateTransition::EngineReset,
            state.clone(),
            state,
            200000,
            5000,
        ));

        let version_changes = log.search_version_changes();
        assert_eq!(version_changes.len(), 1);
        assert_eq!(version_changes[0].wall_time_us, 100000);
    }

    #[test]
    fn test_search_custom_predicate() {
        let mut log = TransitionLog::new(10);

        log.push(TransitionEntry::new(
            StateTransition::KeyPressed {
                key: KeyCode::A,
                timestamp: 100,
            },
            StateSnapshot::empty(),
            StateSnapshot::empty(),
            100000,
            500_000, // 0.5ms
        ));

        log.push(TransitionEntry::new(
            StateTransition::KeyPressed {
                key: KeyCode::B,
                timestamp: 200,
            },
            StateSnapshot::empty(),
            StateSnapshot::empty(),
            200000,
            2_000_000, // 2ms (slow)
        ));

        // Find slow transitions (> 1ms)
        let slow = log.search(|entry| entry.duration_ns > 1_000_000);
        assert_eq!(slow.len(), 1);
        assert_eq!(slow[0].wall_time_us, 200000);
    }

    #[test]
    fn test_export_json() {
        let mut log = TransitionLog::new(10);

        log.push(create_test_entry(KeyCode::A, 100, 100000));
        log.push(create_test_entry(KeyCode::B, 200, 200000));

        let json = log.export_json().expect("export failed");
        assert!(json.contains("KeyPressed"));
        assert!(json.contains("100000"));
        assert!(json.contains("200000"));

        // Verify it's valid JSON
        let parsed: Vec<TransitionEntry> = serde_json::from_str(&json).expect("invalid JSON");
        assert_eq!(parsed.len(), 2);
    }

    #[test]
    fn test_export_json_pretty() {
        let mut log = TransitionLog::new(10);

        log.push(create_test_entry(KeyCode::A, 100, 100000));

        let json = log.export_json_pretty().expect("export failed");
        assert!(json.contains('\n')); // Pretty printing adds newlines
        assert!(json.contains("  ")); // Pretty printing adds indentation
    }

    #[test]
    fn test_export_entries_json() {
        let mut log = TransitionLog::new(10);

        log.push(create_test_entry(KeyCode::A, 100, 100000));
        log.push(create_test_entry(KeyCode::B, 200, 200000));
        log.push(create_test_entry(KeyCode::C, 300, 300000));

        // Search for specific entries
        let filtered = log.search_by_time_range(150000, 300000);
        assert_eq!(filtered.len(), 2);

        // Export only filtered entries
        let json = TransitionLog::export_entries_json(&filtered).expect("export failed");

        let parsed: Vec<TransitionEntry> = serde_json::from_str(&json).expect("invalid JSON");
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].wall_time_us, 200000);
        assert_eq!(parsed[1].wall_time_us, 300000);
    }

    #[test]
    fn test_statistics_empty() {
        let log = TransitionLog::new(10);

        let (total, unique_names, total_duration, avg_duration) = log.statistics();
        assert_eq!(total, 0);
        assert_eq!(unique_names, 0);
        assert_eq!(total_duration, 0);
        assert_eq!(avg_duration, 0);
    }

    #[test]
    fn test_statistics_with_data() {
        let mut log = TransitionLog::new(10);

        log.push(TransitionEntry::new(
            StateTransition::KeyPressed {
                key: KeyCode::A,
                timestamp: 100,
            },
            StateSnapshot::empty(),
            StateSnapshot::empty(),
            100000,
            1000, // 1000ns
        ));

        log.push(TransitionEntry::new(
            StateTransition::KeyPressed {
                key: KeyCode::B,
                timestamp: 200,
            },
            StateSnapshot::empty(),
            StateSnapshot::empty(),
            200000,
            3000, // 3000ns
        ));

        log.push(TransitionEntry::new(
            StateTransition::LayerPushed { layer: 1 },
            StateSnapshot::empty(),
            StateSnapshot::empty(),
            300000,
            2000, // 2000ns
        ));

        let (total, unique_names, total_duration, avg_duration) = log.statistics();
        assert_eq!(total, 3);
        assert_eq!(unique_names, 2); // KeyPressed and LayerPushed
        assert_eq!(total_duration, 6000); // 1000 + 3000 + 2000
        assert_eq!(avg_duration, 2000); // 6000 / 3
    }

    #[test]
    fn test_default() {
        let log = TransitionLog::default();
        assert_eq!(log.capacity(), 10_000);
        assert!(log.is_empty());
    }

    #[test]
    fn test_ring_buffer_full_cycle() {
        let mut log = TransitionLog::new(3);

        // Fill the buffer completely
        for i in 0..3 {
            log.push(create_test_entry(KeyCode::A, i * 100, i * 100000));
        }

        assert_eq!(log.len(), 3);
        assert_eq!(log.total_count(), 3);
        assert!(!log.has_wrapped());

        // Add one more to trigger wrap
        log.push(create_test_entry(KeyCode::B, 300, 300000));

        assert_eq!(log.len(), 3);
        assert_eq!(log.total_count(), 4);
        assert!(log.has_wrapped());

        // Verify oldest entry was overwritten
        let entries: Vec<_> = log.iter().collect();
        assert_eq!(entries[0].wall_time_us, 100000); // entry 1 (entry 0 was overwritten)
        assert_eq!(entries[1].wall_time_us, 200000); // entry 2
        assert_eq!(entries[2].wall_time_us, 300000); // entry 3
    }

    #[test]
    fn test_multiple_wraps() {
        let mut log = TransitionLog::new(3);

        // Add many entries (multiple wraps)
        for i in 0..10 {
            log.push(create_test_entry(KeyCode::A, i * 100, i * 100000));
        }

        assert_eq!(log.len(), 3);
        assert_eq!(log.total_count(), 10);
        assert!(log.has_wrapped());

        // Should contain the last 3 entries (7, 8, 9)
        let entries: Vec<_> = log.iter().collect();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].wall_time_us, 700000);
        assert_eq!(entries[1].wall_time_us, 800000);
        assert_eq!(entries[2].wall_time_us, 900000);
    }
}
