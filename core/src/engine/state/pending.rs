//! Pending decision state management.
//!
//! Tracks tap-hold and combo decisions awaiting resolution. Provides efficient
//! storage and lookup for pending decisions with timeout tracking.

use crate::engine::decision::pending::{
    DecisionQueue, DecisionResolution, PendingDecision, PendingDecisionState,
};
use crate::engine::state::{HoldAction, LayerAction};
use crate::engine::{InputEvent, KeyCode, TimingConfig};

/// Unified pending decision state component.
///
/// Manages all pending decisions (tap-hold and combos) with efficient
/// resolution checking and timeout handling. This is a wrapper around
/// DecisionQueue that provides a unified state component interface.
#[derive(Debug)]
#[allow(dead_code)] // Will be used when EngineState is implemented
pub struct PendingState {
    queue: DecisionQueue,
}

#[allow(dead_code)] // Will be used when EngineState is implemented
impl PendingState {
    /// Maximum number of pending decisions to track.
    pub const MAX_PENDING: usize = DecisionQueue::MAX_PENDING;

    /// Create a new empty pending state.
    pub fn new(config: TimingConfig) -> Self {
        Self {
            queue: DecisionQueue::new(config),
        }
    }

    /// Create with default timing configuration.
    pub fn default_config() -> Self {
        Self::new(TimingConfig::default())
    }

    // === Add Methods ===

    /// Add a new tap-hold pending decision.
    ///
    /// Returns `(added, eager_resolution)`:
    /// - `added`: true if decision was added successfully
    /// - `eager_resolution`: Some if eager tap is configured
    ///
    /// Returns `(false, None)` if queue is full.
    pub fn add_tap_hold(
        &mut self,
        key: KeyCode,
        pressed_at: u64,
        tap_action: KeyCode,
        hold_action: HoldAction,
    ) -> (bool, Option<DecisionResolution>) {
        self.queue
            .add_tap_hold(key, pressed_at, tap_action, hold_action)
    }

    /// Add a new combo pending decision.
    ///
    /// Returns false if:
    /// - Queue is full
    /// - Keys array is invalid (too few/many keys)
    /// - Too few unique keys after deduplication
    pub fn add_combo(&mut self, keys: &[KeyCode], started_at: u64, action: LayerAction) -> bool {
        self.queue.add_combo(keys, started_at, action)
    }

    // === Resolve Methods ===

    /// Check for resolution based on a new input event.
    ///
    /// Returns resolutions triggered by the event. Removes resolved
    /// decisions from the pending queue.
    pub fn resolve_on_event(&mut self, event: &InputEvent) -> Vec<DecisionResolution> {
        self.queue.check_event(event)
    }

    /// Check for timeout-based resolutions.
    ///
    /// Returns resolutions for decisions that have exceeded their deadlines.
    pub fn resolve_on_timeout(&mut self, now_us: u64) -> Vec<DecisionResolution> {
        self.queue.check_timeouts(now_us)
    }

    /// Mark tap-hold decisions as interrupted (for permissive_hold).
    ///
    /// All tap-hold decisions except the one for `by_key` are marked
    /// as interrupted, which may trigger hold resolution on release.
    pub fn mark_interrupted(&mut self, by_key: KeyCode) {
        self.queue.mark_interrupted(by_key);
    }

    // === Clear Methods ===

    /// Clear all pending decisions.
    ///
    /// Returns the count of decisions that were cleared.
    pub fn clear(&mut self) -> usize {
        let count = self.len();
        self.queue.clear();
        count
    }

    // === Query Methods ===

    /// Get the number of pending decisions.
    pub fn len(&self) -> usize {
        self.queue.pending().len()
    }

    /// Check if there are no pending decisions.
    pub fn is_empty(&self) -> bool {
        self.queue.pending().is_empty()
    }

    /// Inspect pending decisions for debugging/telemetry.
    pub fn pending(&self) -> &[PendingDecision] {
        self.queue.pending()
    }

    /// Create a serializable snapshot of pending decisions.
    pub fn snapshot(&self) -> Vec<PendingDecisionState> {
        self.queue.snapshot()
    }
}

impl Default for PendingState {
    fn default() -> Self {
        Self::default_config()
    }
}

impl Clone for PendingState {
    fn clone(&self) -> Self {
        // DecisionQueue doesn't implement Clone, so we recreate with default state
        // In practice, cloning engine state should be rare and done via snapshots
        Self::new(TimingConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_state_is_empty() {
        let state = PendingState::default_config();
        assert_eq!(state.len(), 0);
        assert!(state.is_empty());
    }

    #[test]
    fn add_tap_hold_without_eager_tap() {
        let mut state = PendingState::default_config();
        let (added, eager) =
            state.add_tap_hold(KeyCode::A, 0, KeyCode::B, HoldAction::Key(KeyCode::C));

        assert!(added);
        assert!(eager.is_none());
        assert_eq!(state.len(), 1);
    }

    #[test]
    fn add_tap_hold_with_eager_tap() {
        let mut config = TimingConfig::default();
        config.eager_tap = true;
        let mut state = PendingState::new(config);

        let (added, eager) =
            state.add_tap_hold(KeyCode::A, 0, KeyCode::B, HoldAction::Key(KeyCode::C));

        assert!(added);
        assert_eq!(
            eager,
            Some(DecisionResolution::Tap {
                key: KeyCode::B,
                was_eager: true
            })
        );
        assert_eq!(state.len(), 1);
    }

    #[test]
    fn add_tap_hold_respects_capacity() {
        let mut state = PendingState::default_config();

        // Fill to capacity
        for _ in 0..PendingState::MAX_PENDING {
            let (added, _) =
                state.add_tap_hold(KeyCode::A, 0, KeyCode::B, HoldAction::Key(KeyCode::C));
            assert!(added);
        }

        // Next add should fail
        let (added, _) = state.add_tap_hold(KeyCode::Z, 0, KeyCode::Y, HoldAction::Key(KeyCode::X));
        assert!(!added);
        assert_eq!(state.len(), PendingState::MAX_PENDING);
    }

    #[test]
    fn add_combo_validates_keys() {
        let mut state = PendingState::default_config();

        // Too few keys
        assert!(!state.add_combo(&[KeyCode::A], 0, LayerAction::Block));

        // Valid combo
        assert!(state.add_combo(&[KeyCode::A, KeyCode::B], 0, LayerAction::Block));
        assert_eq!(state.len(), 1);
    }

    #[test]
    fn resolve_tap_on_quick_release() {
        let mut state = PendingState::default_config();
        state.add_tap_hold(KeyCode::A, 0, KeyCode::B, HoldAction::Key(KeyCode::C));

        let event = InputEvent::key_up(KeyCode::A, 50_000); // 50ms
        let resolutions = state.resolve_on_event(&event);

        assert_eq!(resolutions.len(), 1);
        assert_eq!(
            resolutions[0],
            DecisionResolution::Tap {
                key: KeyCode::B,
                was_eager: false
            }
        );
        assert!(state.is_empty());
    }

    #[test]
    fn resolve_hold_on_timeout() {
        let mut state = PendingState::default_config();
        state.add_tap_hold(KeyCode::A, 0, KeyCode::B, HoldAction::Key(KeyCode::C));

        let resolutions = state.resolve_on_timeout(250_000); // 250ms

        assert_eq!(resolutions.len(), 1);
        assert_eq!(
            resolutions[0],
            DecisionResolution::Hold {
                key: KeyCode::A,
                action: HoldAction::Key(KeyCode::C),
                from_eager: false
            }
        );
        assert!(state.is_empty());
    }

    #[test]
    fn resolve_combo_on_all_keys_pressed() {
        let mut state = PendingState::default_config();
        state.add_combo(
            &[KeyCode::A, KeyCode::B],
            0,
            LayerAction::Remap(KeyCode::Escape),
        );

        // Press first key
        let res1 = state.resolve_on_event(&InputEvent::key_down(KeyCode::A, 10_000));
        assert!(res1.is_empty());
        assert_eq!(state.len(), 1);

        // Press second key - should trigger
        let res2 = state.resolve_on_event(&InputEvent::key_down(KeyCode::B, 20_000));
        assert_eq!(res2.len(), 1);
        assert_eq!(
            res2[0],
            DecisionResolution::ComboTriggered(LayerAction::Remap(KeyCode::Escape))
        );
        assert!(state.is_empty());
    }

    #[test]
    fn resolve_combo_timeout_returns_matched_keys() {
        let mut state = PendingState::default_config();
        state.add_combo(
            &[KeyCode::A, KeyCode::B],
            0,
            LayerAction::Remap(KeyCode::Escape),
        );

        // Press one key
        state.resolve_on_event(&InputEvent::key_down(KeyCode::A, 10_000));

        // Timeout
        let resolutions = state.resolve_on_timeout(100_000);
        assert_eq!(resolutions.len(), 1);
        match &resolutions[0] {
            DecisionResolution::ComboTimeout(keys) => {
                assert_eq!(keys.len(), 1);
                assert!(keys.contains(&KeyCode::A));
            }
            _ => panic!("Expected ComboTimeout"),
        }
        assert!(state.is_empty());
    }

    #[test]
    fn mark_interrupted_affects_tap_hold() {
        let mut config = TimingConfig::default();
        config.permissive_hold = true;
        let mut state = PendingState::new(config);

        state.add_tap_hold(KeyCode::A, 0, KeyCode::B, HoldAction::Key(KeyCode::C));

        // Mark interrupted by another key
        state.mark_interrupted(KeyCode::Z);

        // Release before deadline should resolve as hold due to interruption
        let event = InputEvent::key_up(KeyCode::A, 50_000);
        let resolutions = state.resolve_on_event(&event);

        assert_eq!(resolutions.len(), 1);
        assert_eq!(
            resolutions[0],
            DecisionResolution::Hold {
                key: KeyCode::A,
                action: HoldAction::Key(KeyCode::C),
                from_eager: false
            }
        );
    }

    #[test]
    fn clear_removes_all() {
        let mut state = PendingState::default_config();
        state.add_tap_hold(KeyCode::A, 0, KeyCode::B, HoldAction::Key(KeyCode::C));
        state.add_combo(&[KeyCode::D, KeyCode::E], 0, LayerAction::Block);

        assert_eq!(state.len(), 2);

        let count = state.clear();
        assert_eq!(count, 2);
        assert!(state.is_empty());
    }

    #[test]
    fn snapshot_captures_state() {
        let mut state = PendingState::default_config();
        state.add_tap_hold(
            KeyCode::CapsLock,
            10,
            KeyCode::Escape,
            HoldAction::Modifier(2),
        );

        let snapshot = state.snapshot();
        assert_eq!(snapshot.len(), 1);

        match &snapshot[0] {
            PendingDecisionState::TapHold {
                key,
                pressed_at,
                tap_action,
                hold_action,
                ..
            } => {
                assert_eq!(*key, KeyCode::CapsLock);
                assert_eq!(*pressed_at, 10);
                assert_eq!(*tap_action, KeyCode::Escape);
                assert_eq!(*hold_action, HoldAction::Modifier(2));
            }
            _ => panic!("Expected TapHold"),
        }

        // Verify serializable
        serde_json::to_string(&snapshot[0]).expect("serializes");
    }

    #[test]
    fn default_creates_valid_state() {
        let state = PendingState::default();
        assert!(state.is_empty());
    }

    #[test]
    fn clone_creates_empty_state() {
        let mut state = PendingState::default_config();
        state.add_tap_hold(KeyCode::A, 0, KeyCode::B, HoldAction::Key(KeyCode::C));

        let cloned = state.clone();
        // Clone creates a new empty state
        assert!(cloned.is_empty());
    }
}
