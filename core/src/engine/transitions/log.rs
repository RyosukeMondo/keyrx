//! State transition logging for debugging and replay.
//!
//! This module provides types for logging state transitions with complete
//! before/after state snapshots. This enables:
//! - Debugging state machine behavior
//! - Replay of transition sequences
//! - Analysis of state evolution
//! - Export of transition history

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
