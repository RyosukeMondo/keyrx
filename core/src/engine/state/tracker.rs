//! Delta tracking for incremental state updates.
//!
//! The DeltaTracker records state changes as they occur and generates
//! StateDelta objects on demand. This enables efficient incremental
//! updates instead of sending full state snapshots.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use crate::engine::state::delta::{DeltaChange, StateDelta};

/// Tracks state changes and generates deltas on demand.
///
/// DeltaTracker maintains a version counter and a queue of pending changes.
/// When a delta is requested, it collects all pending changes into a
/// StateDelta and returns it, clearing the queue.
///
/// # Thread Safety
///
/// DeltaTracker is designed to be used from a single thread (the engine thread),
/// but uses atomic operations for the version counter to ensure correctness
/// if accessed from multiple threads during testing or debugging.
///
/// # Example
///
/// ```no_run
/// use keyrx_core::engine::state::tracker::DeltaTracker;
/// use keyrx_core::engine::state::delta::DeltaChange;
/// use keyrx_core::engine::KeyCode;
///
/// let tracker = DeltaTracker::new();
///
/// // Record some changes
/// tracker.record(DeltaChange::KeyPressed(KeyCode::A));
/// tracker.record(DeltaChange::LayerActivated(1));
///
/// // Take the delta (this clears the queue)
/// let delta = tracker.take_delta();
/// assert_eq!(delta.change_count(), 2);
///
/// // Next delta will have no changes
/// let delta = tracker.take_delta();
/// assert!(delta.is_empty());
/// ```
#[derive(Debug)]
pub struct DeltaTracker {
    /// Current state version.
    current_version: AtomicU64,
    /// Pending changes that haven't been sent yet.
    pending_changes: Mutex<Vec<DeltaChange>>,
}

impl DeltaTracker {
    /// Create a new delta tracker starting at version 0.
    pub fn new() -> Self {
        Self {
            current_version: AtomicU64::new(0),
            pending_changes: Mutex::new(Vec::new()),
        }
    }

    /// Create a new delta tracker with a specific starting version.
    pub fn with_version(version: u64) -> Self {
        Self {
            current_version: AtomicU64::new(version),
            pending_changes: Mutex::new(Vec::new()),
        }
    }

    /// Record a state change.
    ///
    /// The change is added to the pending queue and will be included
    /// in the next delta that is taken.
    ///
    /// If the mutex is poisoned, this method silently fails rather than panicking.
    pub fn record(&self, change: DeltaChange) {
        if let Ok(mut changes) = self.pending_changes.lock() {
            changes.push(change);
        }
    }

    /// Record multiple state changes at once.
    ///
    /// If the mutex is poisoned, this method silently fails rather than panicking.
    pub fn record_batch(&self, changes: Vec<DeltaChange>) {
        if let Ok(mut pending) = self.pending_changes.lock() {
            pending.extend(changes);
        }
    }

    /// Increment the version counter and return the new version.
    ///
    /// This should be called whenever the state changes.
    pub fn increment_version(&self) -> u64 {
        self.current_version.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// Get the current version without incrementing.
    pub fn current_version(&self) -> u64 {
        self.current_version.load(Ordering::SeqCst)
    }

    /// Take a delta containing all pending changes.
    ///
    /// This consumes all pending changes and returns a StateDelta.
    /// The from_version is the version before any changes, and
    /// to_version is the current version after all changes.
    ///
    /// After calling this, the pending changes queue is empty.
    ///
    /// If the mutex is poisoned, returns an empty delta.
    pub fn take_delta(&self) -> StateDelta {
        if let Ok(mut changes) = self.pending_changes.lock() {
            let from_version = if changes.is_empty() {
                self.current_version()
            } else {
                // If we have changes, from_version is current minus number of changes
                // This assumes each change incremented the version by 1
                self.current_version().saturating_sub(changes.len() as u64)
            };
            let to_version = self.current_version();

            let delta = StateDelta::new(from_version, to_version, changes.clone());
            changes.clear();
            delta
        } else {
            // Mutex poisoned, return empty delta
            let version = self.current_version();
            StateDelta::new(version, version, Vec::new())
        }
    }

    /// Peek at the pending changes without consuming them.
    ///
    /// This is useful for testing and debugging.
    ///
    /// If the mutex is poisoned, returns an empty vector.
    pub fn peek_changes(&self) -> Vec<DeltaChange> {
        self.pending_changes
            .lock()
            .map(|changes| changes.clone())
            .unwrap_or_default()
    }

    /// Get the number of pending changes.
    ///
    /// If the mutex is poisoned, returns 0.
    pub fn pending_count(&self) -> usize {
        self.pending_changes
            .lock()
            .map(|changes| changes.len())
            .unwrap_or(0)
    }

    /// Clear all pending changes without creating a delta.
    ///
    /// This is useful when a full state sync is performed and we want
    /// to discard any pending deltas.
    ///
    /// If the mutex is poisoned, this method does nothing.
    pub fn clear(&self) {
        if let Ok(mut changes) = self.pending_changes.lock() {
            changes.clear();
        }
    }

    /// Reset the tracker to a specific version and clear pending changes.
    ///
    /// This is useful when recovering from version mismatch by performing
    /// a full state sync.
    pub fn reset_to_version(&self, version: u64) {
        self.current_version.store(version, Ordering::SeqCst);
        self.clear();
    }
}

impl Default for DeltaTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::KeyCode;

    #[test]
    fn new_tracker_starts_at_version_zero() {
        let tracker = DeltaTracker::new();
        assert_eq!(tracker.current_version(), 0);
        assert_eq!(tracker.pending_count(), 0);
    }

    #[test]
    fn with_version_sets_initial_version() {
        let tracker = DeltaTracker::with_version(42);
        assert_eq!(tracker.current_version(), 42);
        assert_eq!(tracker.pending_count(), 0);
    }

    #[test]
    fn record_adds_change() {
        let tracker = DeltaTracker::new();
        tracker.record(DeltaChange::KeyPressed(KeyCode::A));

        assert_eq!(tracker.pending_count(), 1);
        let changes = tracker.peek_changes();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0], DeltaChange::KeyPressed(KeyCode::A));
    }

    #[test]
    fn record_batch_adds_multiple_changes() {
        let tracker = DeltaTracker::new();
        let changes = vec![
            DeltaChange::KeyPressed(KeyCode::A),
            DeltaChange::KeyPressed(KeyCode::B),
            DeltaChange::LayerActivated(1),
        ];

        tracker.record_batch(changes.clone());
        assert_eq!(tracker.pending_count(), 3);

        let pending = tracker.peek_changes();
        assert_eq!(pending, changes);
    }

    #[test]
    fn increment_version_increases_counter() {
        let tracker = DeltaTracker::new();
        assert_eq!(tracker.current_version(), 0);

        let v1 = tracker.increment_version();
        assert_eq!(v1, 1);
        assert_eq!(tracker.current_version(), 1);

        let v2 = tracker.increment_version();
        assert_eq!(v2, 2);
        assert_eq!(tracker.current_version(), 2);
    }

    #[test]
    fn take_delta_with_no_changes() {
        let tracker = DeltaTracker::new();
        let delta = tracker.take_delta();

        assert!(delta.is_empty());
        assert_eq!(delta.from_version, 0);
        assert_eq!(delta.to_version, 0);
        assert_eq!(tracker.pending_count(), 0);
    }

    #[test]
    fn take_delta_with_changes() {
        let tracker = DeltaTracker::new();

        // Record changes and increment version
        tracker.record(DeltaChange::KeyPressed(KeyCode::A));
        tracker.increment_version();

        tracker.record(DeltaChange::LayerActivated(1));
        tracker.increment_version();

        let delta = tracker.take_delta();

        assert_eq!(delta.change_count(), 2);
        assert_eq!(delta.from_version, 0);
        assert_eq!(delta.to_version, 2);
        assert_eq!(tracker.pending_count(), 0);
    }

    #[test]
    fn take_delta_clears_pending_changes() {
        let tracker = DeltaTracker::new();

        tracker.record(DeltaChange::KeyPressed(KeyCode::A));
        tracker.increment_version();

        assert_eq!(tracker.pending_count(), 1);

        let _delta = tracker.take_delta();
        assert_eq!(tracker.pending_count(), 0);

        // Next delta should be empty
        let delta2 = tracker.take_delta();
        assert!(delta2.is_empty());
    }

    #[test]
    fn multiple_take_delta_cycles() {
        let tracker = DeltaTracker::new();

        // First cycle
        tracker.record(DeltaChange::KeyPressed(KeyCode::A));
        tracker.increment_version();
        let delta1 = tracker.take_delta();
        assert_eq!(delta1.change_count(), 1);
        assert_eq!(delta1.to_version, 1);

        // Second cycle
        tracker.record(DeltaChange::KeyReleased(KeyCode::A));
        tracker.increment_version();
        let delta2 = tracker.take_delta();
        assert_eq!(delta2.change_count(), 1);
        assert_eq!(delta2.from_version, 1);
        assert_eq!(delta2.to_version, 2);

        // Third cycle with no changes
        let delta3 = tracker.take_delta();
        assert!(delta3.is_empty());
        assert_eq!(delta3.from_version, 2);
        assert_eq!(delta3.to_version, 2);
    }

    #[test]
    fn clear_removes_all_pending() {
        let tracker = DeltaTracker::new();

        tracker.record(DeltaChange::KeyPressed(KeyCode::A));
        tracker.record(DeltaChange::LayerActivated(1));
        assert_eq!(tracker.pending_count(), 2);

        tracker.clear();
        assert_eq!(tracker.pending_count(), 0);
    }

    #[test]
    fn reset_to_version_updates_version_and_clears() {
        let tracker = DeltaTracker::with_version(10);
        tracker.increment_version();
        tracker.increment_version();
        assert_eq!(tracker.current_version(), 12);

        tracker.record(DeltaChange::KeyPressed(KeyCode::A));
        assert_eq!(tracker.pending_count(), 1);

        tracker.reset_to_version(20);
        assert_eq!(tracker.current_version(), 20);
        assert_eq!(tracker.pending_count(), 0);
    }

    #[test]
    fn peek_changes_does_not_consume() {
        let tracker = DeltaTracker::new();

        tracker.record(DeltaChange::KeyPressed(KeyCode::A));
        tracker.record(DeltaChange::KeyPressed(KeyCode::B));

        let peeked = tracker.peek_changes();
        assert_eq!(peeked.len(), 2);

        // Pending count should still be 2
        assert_eq!(tracker.pending_count(), 2);

        // Taking delta should still have both changes
        tracker.increment_version();
        tracker.increment_version();
        let delta = tracker.take_delta();
        assert_eq!(delta.change_count(), 2);
    }

    #[test]
    fn default_creates_new_tracker() {
        let tracker = DeltaTracker::default();
        assert_eq!(tracker.current_version(), 0);
        assert_eq!(tracker.pending_count(), 0);
    }

    #[test]
    fn complex_scenario() {
        let tracker = DeltaTracker::with_version(100);

        // Simulate multiple state changes
        tracker.record(DeltaChange::KeyPressed(KeyCode::A));
        tracker.increment_version(); // v101

        tracker.record(DeltaChange::KeyPressed(KeyCode::B));
        tracker.increment_version(); // v102

        tracker.record(DeltaChange::LayerActivated(1));
        tracker.increment_version(); // v103

        tracker.record(DeltaChange::ModifierChanged {
            id: 5,
            active: true,
        });
        tracker.increment_version(); // v104

        // Take delta
        let delta = tracker.take_delta();
        assert_eq!(delta.from_version, 100);
        assert_eq!(delta.to_version, 104);
        assert_eq!(delta.change_count(), 4);

        // Continue with more changes
        tracker.record(DeltaChange::KeyReleased(KeyCode::A));
        tracker.increment_version(); // v105

        let delta2 = tracker.take_delta();
        assert_eq!(delta2.from_version, 104);
        assert_eq!(delta2.to_version, 105);
        assert_eq!(delta2.change_count(), 1);
    }

    #[test]
    fn version_wraps_correctly() {
        let tracker = DeltaTracker::with_version(u64::MAX - 2);

        tracker.increment_version(); // u64::MAX - 1
        tracker.increment_version(); // u64::MAX
        tracker.increment_version(); // wraps to 0

        assert_eq!(tracker.current_version(), 0);
    }
}
