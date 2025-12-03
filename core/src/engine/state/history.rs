//! State history tracking for debugging and replay.
//!
//! `StateHistory` maintains a bounded ring buffer of recent state changes,
//! enabling:
//! - Debugging by inspecting recent mutations and effects
//! - Replay of state changes for testing
//! - Analysis of state transition patterns
//! - Telemetry and diagnostics

use crate::engine::state::StateChange;
use std::collections::VecDeque;

/// Configuration for history tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Will be used in tasks 14+ for state persistence and debugging
pub struct HistoryConfig {
    /// Maximum number of state changes to keep.
    ///
    /// When capacity is reached, oldest changes are dropped.
    /// Default: 1000 changes.
    pub max_depth: usize,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self { max_depth: 1000 }
    }
}

#[allow(dead_code)] // Will be used in tasks 14+ for state persistence and debugging
impl HistoryConfig {
    /// Create a new history configuration.
    pub fn new(max_depth: usize) -> Self {
        Self { max_depth }
    }

    /// Create configuration with unlimited depth.
    ///
    /// WARNING: Unbounded history can consume arbitrary memory.
    /// Only use for testing or debugging scenarios.
    pub fn unlimited() -> Self {
        Self {
            max_depth: usize::MAX,
        }
    }
}

/// Ring buffer of recent state changes.
///
/// Maintains a bounded history of state changes for debugging and analysis.
/// When capacity is reached, oldest changes are automatically dropped.
///
/// # Examples
///
/// ```
/// use keyrx_core::engine::state::{StateHistory, HistoryConfig, StateChange, Mutation};
/// use keyrx_core::engine::KeyCode;
///
/// let mut history = StateHistory::new(HistoryConfig::new(100));
///
/// let change = StateChange::new(
///     Mutation::KeyDown { key: KeyCode::A, timestamp_us: 1000, is_repeat: false },
///     1,
///     1000
/// );
/// history.push(change.clone());
///
/// assert_eq!(history.len(), 1);
/// assert_eq!(history.recent(10).len(), 1);
/// ```
#[derive(Debug, Clone)]
#[allow(dead_code)] // Will be used in tasks 14+ for state persistence and debugging
pub struct StateHistory {
    /// Ring buffer of state changes.
    ///
    /// Oldest changes at front, newest at back.
    changes: VecDeque<StateChange>,

    /// Configuration for history tracking.
    config: HistoryConfig,

    /// Total number of changes ever recorded.
    ///
    /// Continues incrementing even when changes are dropped.
    /// Useful for detecting if history was truncated.
    total_changes: u64,
}

#[allow(dead_code)] // Will be used in tasks 14+ for state persistence and debugging
impl StateHistory {
    /// Create a new state history with the given configuration.
    pub fn new(config: HistoryConfig) -> Self {
        Self {
            changes: VecDeque::with_capacity(config.max_depth.min(1024)),
            config,
            total_changes: 0,
        }
    }

    /// Create a state history with a specific depth.
    pub fn with_depth(max_depth: usize) -> Self {
        Self::new(HistoryConfig::new(max_depth))
    }

    /// Push a new state change to the history.
    ///
    /// If capacity is reached, the oldest change is dropped.
    pub fn push(&mut self, change: StateChange) {
        // Drop oldest if at capacity
        if self.changes.len() >= self.config.max_depth {
            self.changes.pop_front();
        }

        self.changes.push_back(change);
        self.total_changes += 1;
    }

    /// Returns the number of changes currently in history.
    pub fn len(&self) -> usize {
        self.changes.len()
    }

    /// Returns true if history is empty.
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    /// Returns the total number of changes ever recorded.
    ///
    /// This includes changes that were dropped due to capacity limits.
    pub fn total_changes(&self) -> u64 {
        self.total_changes
    }

    /// Returns true if some changes were dropped due to capacity.
    pub fn has_truncated(&self) -> bool {
        self.total_changes > self.changes.len() as u64
    }

    /// Returns the most recent N changes.
    ///
    /// If N is larger than history size, returns all available changes.
    /// Changes are ordered from oldest to newest.
    pub fn recent(&self, count: usize) -> Vec<StateChange> {
        let start = self.changes.len().saturating_sub(count);
        self.changes.range(start..).cloned().collect()
    }

    /// Returns all changes in history.
    ///
    /// Changes are ordered from oldest to newest.
    pub fn all(&self) -> Vec<StateChange> {
        self.changes.iter().cloned().collect()
    }

    /// Returns an iterator over all changes.
    ///
    /// Changes are ordered from oldest to newest.
    pub fn iter(&self) -> impl Iterator<Item = &StateChange> {
        self.changes.iter()
    }

    /// Returns the most recent change, if any.
    pub fn last(&self) -> Option<&StateChange> {
        self.changes.back()
    }

    /// Returns the oldest change in history, if any.
    pub fn first(&self) -> Option<&StateChange> {
        self.changes.front()
    }

    /// Clear all history.
    ///
    /// Note: This resets `len()` but not `total_changes()`.
    pub fn clear(&mut self) {
        self.changes.clear();
    }

    /// Returns the configured maximum depth.
    pub fn max_depth(&self) -> usize {
        self.config.max_depth
    }

    /// Filter changes by a predicate.
    ///
    /// Returns all changes matching the predicate, in chronological order.
    pub fn filter<F>(&self, predicate: F) -> Vec<StateChange>
    where
        F: Fn(&StateChange) -> bool,
    {
        self.changes
            .iter()
            .filter(|change| predicate(change))
            .cloned()
            .collect()
    }

    /// Find changes within a version range (inclusive).
    ///
    /// Returns all changes with versions in [min_version, max_version].
    pub fn version_range(&self, min_version: u64, max_version: u64) -> Vec<StateChange> {
        self.filter(|change| change.version >= min_version && change.version <= max_version)
    }

    /// Find changes within a timestamp range (inclusive).
    ///
    /// Returns all changes with timestamps in [min_timestamp_us, max_timestamp_us].
    pub fn timestamp_range(
        &self,
        min_timestamp_us: u64,
        max_timestamp_us: u64,
    ) -> Vec<StateChange> {
        self.filter(|change| {
            change.timestamp_us >= min_timestamp_us && change.timestamp_us <= max_timestamp_us
        })
    }

    /// Resize the history to a new maximum depth.
    ///
    /// If the new depth is smaller than current size, oldest changes are dropped.
    pub fn resize(&mut self, new_max_depth: usize) {
        self.config.max_depth = new_max_depth;

        // Drop oldest changes if necessary
        while self.changes.len() > new_max_depth {
            self.changes.pop_front();
        }

        // Shrink capacity if significantly over-allocated
        if self.changes.capacity() > new_max_depth * 2 {
            self.changes.shrink_to_fit();
        }
    }
}

impl Default for StateHistory {
    fn default() -> Self {
        Self::new(HistoryConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::state::Mutation;
    use crate::engine::KeyCode;

    fn make_change(version: u64, timestamp_us: u64) -> StateChange {
        StateChange::new(
            Mutation::KeyDown {
                key: KeyCode::A,
                timestamp_us,
                is_repeat: false,
            },
            version,
            timestamp_us,
        )
    }

    #[test]
    fn new_history_is_empty() {
        let history = StateHistory::new(HistoryConfig::new(100));
        assert_eq!(history.len(), 0);
        assert!(history.is_empty());
        assert_eq!(history.total_changes(), 0);
        assert!(!history.has_truncated());
    }

    #[test]
    fn push_single_change() {
        let mut history = StateHistory::new(HistoryConfig::new(100));
        let change = make_change(1, 1000);

        history.push(change.clone());

        assert_eq!(history.len(), 1);
        assert_eq!(history.total_changes(), 1);
        assert_eq!(history.last(), Some(&change));
        assert_eq!(history.first(), Some(&change));
    }

    #[test]
    fn push_multiple_changes() {
        let mut history = StateHistory::new(HistoryConfig::new(100));

        for i in 0..10 {
            history.push(make_change(i, i * 1000));
        }

        assert_eq!(history.len(), 10);
        assert_eq!(history.total_changes(), 10);
        assert_eq!(history.first().unwrap().version, 0);
        assert_eq!(history.last().unwrap().version, 9);
    }

    #[test]
    fn ring_buffer_drops_oldest() {
        let mut history = StateHistory::new(HistoryConfig::new(5));

        // Push more than capacity
        for i in 0..10 {
            history.push(make_change(i, i * 1000));
        }

        assert_eq!(history.len(), 5);
        assert_eq!(history.total_changes(), 10);
        assert!(history.has_truncated());

        // Should contain changes 5-9 (oldest 0-4 were dropped)
        assert_eq!(history.first().unwrap().version, 5);
        assert_eq!(history.last().unwrap().version, 9);
    }

    #[test]
    fn recent_returns_last_n() {
        let mut history = StateHistory::new(HistoryConfig::new(100));

        for i in 0..20 {
            history.push(make_change(i, i * 1000));
        }

        let recent = history.recent(5);
        assert_eq!(recent.len(), 5);
        assert_eq!(recent[0].version, 15);
        assert_eq!(recent[4].version, 19);
    }

    #[test]
    fn recent_handles_overflow() {
        let mut history = StateHistory::new(HistoryConfig::new(100));

        for i in 0..5 {
            history.push(make_change(i, i * 1000));
        }

        let recent = history.recent(10);
        assert_eq!(recent.len(), 5); // Only 5 available
    }

    #[test]
    fn all_returns_everything() {
        let mut history = StateHistory::new(HistoryConfig::new(100));

        for i in 0..10 {
            history.push(make_change(i, i * 1000));
        }

        let all = history.all();
        assert_eq!(all.len(), 10);
        assert_eq!(all[0].version, 0);
        assert_eq!(all[9].version, 9);
    }

    #[test]
    fn iter_works() {
        let mut history = StateHistory::new(HistoryConfig::new(100));

        for i in 0..5 {
            history.push(make_change(i, i * 1000));
        }

        let versions: Vec<u64> = history.iter().map(|c| c.version).collect();
        assert_eq!(versions, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn clear_removes_all() {
        let mut history = StateHistory::new(HistoryConfig::new(100));

        for i in 0..10 {
            history.push(make_change(i, i * 1000));
        }

        assert_eq!(history.len(), 10);
        assert_eq!(history.total_changes(), 10);

        history.clear();

        assert_eq!(history.len(), 0);
        assert!(history.is_empty());
        // total_changes is not reset
        assert_eq!(history.total_changes(), 10);
    }

    #[test]
    fn filter_works() {
        let mut history = StateHistory::new(HistoryConfig::new(100));

        for i in 0..10 {
            history.push(make_change(i, i * 1000));
        }

        // Filter even versions
        let even = history.filter(|c| c.version % 2 == 0);
        assert_eq!(even.len(), 5);
        assert_eq!(even[0].version, 0);
        assert_eq!(even[4].version, 8);
    }

    #[test]
    fn version_range_works() {
        let mut history = StateHistory::new(HistoryConfig::new(100));

        for i in 0..10 {
            history.push(make_change(i, i * 1000));
        }

        let range = history.version_range(3, 7);
        assert_eq!(range.len(), 5);
        assert_eq!(range[0].version, 3);
        assert_eq!(range[4].version, 7);
    }

    #[test]
    fn timestamp_range_works() {
        let mut history = StateHistory::new(HistoryConfig::new(100));

        for i in 0..10 {
            history.push(make_change(i, i * 1000));
        }

        let range = history.timestamp_range(2000, 6000);
        assert_eq!(range.len(), 5);
        assert_eq!(range[0].timestamp_us, 2000);
        assert_eq!(range[4].timestamp_us, 6000);
    }

    #[test]
    fn resize_to_smaller() {
        let mut history = StateHistory::new(HistoryConfig::new(10));

        for i in 0..10 {
            history.push(make_change(i, i * 1000));
        }

        assert_eq!(history.len(), 10);

        history.resize(5);

        assert_eq!(history.len(), 5);
        assert_eq!(history.max_depth(), 5);
        // Should keep newest 5
        assert_eq!(history.first().unwrap().version, 5);
        assert_eq!(history.last().unwrap().version, 9);
    }

    #[test]
    fn resize_to_larger() {
        let mut history = StateHistory::new(HistoryConfig::new(5));

        for i in 0..10 {
            history.push(make_change(i, i * 1000));
        }

        assert_eq!(history.len(), 5);

        history.resize(20);

        assert_eq!(history.len(), 5); // Doesn't restore dropped changes
        assert_eq!(history.max_depth(), 20);

        // Can now add more
        for i in 10..15 {
            history.push(make_change(i, i * 1000));
        }

        assert_eq!(history.len(), 10);
    }

    #[test]
    fn default_config() {
        let config = HistoryConfig::default();
        assert_eq!(config.max_depth, 1000);
    }

    #[test]
    fn unlimited_config() {
        let config = HistoryConfig::unlimited();
        assert_eq!(config.max_depth, usize::MAX);
    }

    #[test]
    fn with_depth_constructor() {
        let history = StateHistory::with_depth(42);
        assert_eq!(history.max_depth(), 42);
    }
}
