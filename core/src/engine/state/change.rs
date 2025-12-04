//! State change events and effects.
//!
//! When mutations are applied to engine state, they produce `StateChange` events
//! that record what changed and any secondary effects. These events can be:
//! - Serialized for debugging and replay
//! - Emitted via FFI for UI updates
//! - Stored in history for undo/inspection
//! - Used for telemetry and analytics

use crate::engine::state::{LayerId, Mutation};
use crate::engine::KeyCode;
use serde::{Deserialize, Serialize};

/// A recorded state change with its effects.
///
/// Each mutation applied to `EngineState` produces a `StateChange` that captures:
/// - The mutation that was applied
/// - The state version after the change
/// - Any secondary effects triggered by the mutation
/// - Timestamp of when the change occurred
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, keyrx_ffi_macros::FfiMarshaler)]
#[ffi(strategy = "json")]
#[allow(dead_code)] // Will be used when EngineState is implemented
pub struct StateChange {
    /// The mutation that was applied.
    pub mutation: Mutation,

    /// State version after this change.
    ///
    /// Increments with each mutation. Enables:
    /// - Detecting stale state references
    /// - Ordering concurrent changes
    /// - Version-based caching invalidation
    pub version: u64,

    /// Timestamp when the change occurred (microseconds).
    ///
    /// Uses the same clock as input events for correlation.
    pub timestamp_us: u64,

    /// Secondary effects triggered by this mutation.
    ///
    /// Example: Releasing a modifier key may trigger:
    /// - DeactivateModifier effect
    /// - ClearPending effect (if pending decisions depend on it)
    pub effects: Vec<Effect>,
}

impl StateChange {
    /// Create a new state change.
    #[allow(dead_code)] // Will be used when EngineState is implemented
    pub fn new(mutation: Mutation, version: u64, timestamp_us: u64) -> Self {
        Self {
            mutation,
            version,
            timestamp_us,
            effects: Vec::new(),
        }
    }

    /// Create a state change with effects.
    #[allow(dead_code)] // Will be used when EngineState is implemented
    pub fn with_effects(
        mutation: Mutation,
        version: u64,
        timestamp_us: u64,
        effects: Vec<Effect>,
    ) -> Self {
        Self {
            mutation,
            version,
            timestamp_us,
            effects,
        }
    }

    /// Add an effect to this change.
    #[allow(dead_code)] // Will be used when EngineState is implemented
    pub fn add_effect(&mut self, effect: Effect) {
        self.effects.push(effect);
    }

    /// Returns true if this change has any effects.
    #[allow(dead_code)] // Will be used when EngineState is implemented
    pub fn has_effects(&self) -> bool {
        !self.effects.is_empty()
    }

    /// Returns the number of effects.
    #[allow(dead_code)] // Will be used when EngineState is implemented
    pub fn effect_count(&self) -> usize {
        self.effects.len()
    }
}

/// Secondary effects triggered by state mutations.
///
/// Effects represent automatic state synchronization that occurs when
/// mutations are applied. They ensure state invariants are maintained.
///
/// # Examples
///
/// - Releasing a modifier key triggers `ModifierDeactivated`
/// - Changing layers may trigger `PendingCleared` if decisions become invalid
/// - Clearing all keys triggers `AllModifiersCleared`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(dead_code)] // Will be used when EngineState is implemented
pub enum Effect {
    // === Key State Effects ===
    /// A key was automatically released due to state sync.
    KeyAutoReleased {
        key: KeyCode,
        reason: AutoReleaseReason,
    },

    // === Modifier Effects ===
    /// A modifier was automatically deactivated.
    ModifierDeactivated { modifier_id: u8 },

    /// All modifiers were cleared.
    AllModifiersCleared,

    /// A one-shot modifier was consumed.
    OneShotConsumed { modifier_id: u8 },

    // === Layer Effects ===
    /// A layer was automatically popped.
    LayerPopped { layer_id: LayerId },

    /// Multiple layers were cleared.
    LayersCleared { count: usize },

    // === Pending Decision Effects ===
    /// A pending decision was resolved.
    PendingResolved {
        decision_type: PendingDecisionType,
        resolution: PendingResolution,
    },

    /// All pending decisions were cleared.
    PendingCleared { count: usize },

    /// Pending decisions were marked as interrupted.
    PendingInterrupted { count: usize },

    // === Timeout Effects ===
    /// A tap-hold decision timed out.
    TapHoldTimeout { key: KeyCode, outcome: HoldOutcome },

    /// A combo decision timed out.
    ComboTimeout { keys: Vec<KeyCode> },
}

/// Reason why a key was automatically released.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoReleaseReason {
    /// Layer changed while key was pressed.
    LayerChanged,
    /// All state was reset.
    StateReset,
    /// Modifier key released requires sync.
    ModifierSync,
}

/// Type of pending decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PendingDecisionType {
    /// Tap-hold decision.
    TapHold,
    /// Combo decision.
    Combo,
}

/// How a pending decision was resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PendingResolution {
    /// Decision was resolved as tap.
    Tap,
    /// Decision was resolved as hold.
    Hold,
    /// Decision was resolved as combo activated.
    ComboActivated,
    /// Decision was cancelled/interrupted.
    Cancelled,
    /// Decision timed out.
    Timeout,
}

/// Outcome of a tap-hold timeout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HoldOutcome {
    /// Hold action triggered.
    Hold,
    /// Tap action triggered (timeout before hold threshold).
    Tap,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_change_creation() {
        let mutation = Mutation::KeyDown {
            key: KeyCode::A,
            timestamp_us: 1000,
            is_repeat: false,
        };
        let change = StateChange::new(mutation.clone(), 42, 1000);

        assert_eq!(change.version, 42);
        assert_eq!(change.timestamp_us, 1000);
        assert_eq!(change.mutation, mutation);
        assert!(change.effects.is_empty());
    }

    #[test]
    fn state_change_with_effects() {
        let mutation = Mutation::ClearModifiers;
        let effects = vec![Effect::AllModifiersCleared];
        let change = StateChange::with_effects(mutation.clone(), 10, 2000, effects.clone());

        assert_eq!(change.version, 10);
        assert_eq!(change.mutation, mutation);
        assert_eq!(change.effects, effects);
        assert!(change.has_effects());
        assert_eq!(change.effect_count(), 1);
    }

    #[test]
    fn add_effect() {
        let mut change = StateChange::new(Mutation::PopLayer, 5, 3000);
        assert!(!change.has_effects());

        change.add_effect(Effect::LayerPopped { layer_id: 1 });
        assert!(change.has_effects());
        assert_eq!(change.effect_count(), 1);

        change.add_effect(Effect::PendingCleared { count: 2 });
        assert_eq!(change.effect_count(), 2);
    }

    #[test]
    fn state_change_serialization() {
        let mutation = Mutation::ActivateModifier { modifier_id: 5 };
        let change = StateChange::new(mutation, 100, 5000);

        let json = serde_json::to_string(&change).expect("serializes");
        let deserialized: StateChange = serde_json::from_str(&json).expect("deserializes");

        assert_eq!(change, deserialized);
    }

    #[test]
    fn effect_serialization() {
        let effect = Effect::ModifierDeactivated { modifier_id: 3 };

        let json = serde_json::to_string(&effect).expect("serializes");
        let deserialized: Effect = serde_json::from_str(&json).expect("deserializes");

        assert_eq!(effect, deserialized);
    }

    #[test]
    fn complex_effects() {
        let effects = vec![
            Effect::KeyAutoReleased {
                key: KeyCode::A,
                reason: AutoReleaseReason::LayerChanged,
            },
            Effect::PendingResolved {
                decision_type: PendingDecisionType::TapHold,
                resolution: PendingResolution::Hold,
            },
            Effect::TapHoldTimeout {
                key: KeyCode::Space,
                outcome: HoldOutcome::Hold,
            },
        ];

        let mutation = Mutation::PopLayer;
        let change = StateChange::with_effects(mutation, 50, 7000, effects);

        assert_eq!(change.effect_count(), 3);

        // Verify serialization works with complex effects
        let json = serde_json::to_string(&change).expect("serializes");
        let deserialized: StateChange = serde_json::from_str(&json).expect("deserializes");
        assert_eq!(change, deserialized);
    }
}
