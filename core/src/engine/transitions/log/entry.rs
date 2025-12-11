//! TransitionEntry type for capturing state transitions with snapshots.
//!
//! This module provides the `TransitionEntry` struct which captures a complete
//! state transition with before/after snapshots for debugging and replay.

use serde::{Deserialize, Serialize};

use crate::engine::state::snapshot::StateSnapshot;
use crate::engine::transitions::transition::{StateTransition, TransitionCategory};

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
