//! State mutation operations for the engine.
//!
//! Mutations represent explicit, atomic state change operations. Each mutation
//! can be applied to an `EngineState` to produce a `StateChange` event.

use crate::engine::state::{HoldAction, LayerAction, LayerId};
use crate::engine::KeyCode;
use serde::{Deserialize, Serialize};

/// An atomic state change operation.
///
/// Mutations are the only way to modify engine state. Each mutation applied
/// to an `EngineState` produces a `StateChange` event that can be recorded,
/// serialized, or used for debugging.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(dead_code)] // Will be used when EngineState is implemented
pub enum Mutation {
    // === Key State Mutations ===
    /// Record a key press with timestamp.
    KeyDown {
        key: KeyCode,
        timestamp_us: u64,
        is_repeat: bool,
    },

    /// Record a key release with timestamp.
    KeyUp { key: KeyCode, timestamp_us: u64 },

    // === Layer State Mutations ===
    /// Push a layer to the top of the layer stack.
    PushLayer { layer_id: LayerId },

    /// Pop the topmost non-base layer from the stack.
    PopLayer,

    /// Toggle a layer on/off.
    ToggleLayer { layer_id: LayerId },

    // === Modifier State Mutations ===
    /// Activate a modifier (standard or virtual).
    ActivateModifier { modifier_id: u8 },

    /// Deactivate a modifier.
    DeactivateModifier { modifier_id: u8 },

    /// Arm a one-shot modifier (applies to next event).
    ArmOneShotModifier { modifier_id: u8 },

    /// Clear all modifiers.
    ClearModifiers,

    // === Pending Decision Mutations ===
    /// Add a tap-hold pending decision.
    AddTapHold {
        key: KeyCode,
        pressed_at: u64,
        tap_action: KeyCode,
        hold_action: HoldAction,
    },

    /// Add a combo pending decision.
    AddCombo {
        keys: Vec<KeyCode>,
        started_at: u64,
        action: LayerAction,
    },

    /// Mark tap-hold decisions as interrupted (for permissive_hold).
    MarkInterrupted { by_key: KeyCode },

    /// Clear all pending decisions.
    ClearPending,

    // === Batch Mutations ===
    /// Execute multiple mutations atomically.
    ///
    /// If any mutation fails, all changes are rolled back.
    Batch { mutations: Vec<Mutation> },
}

impl Mutation {
    /// Returns true if this is a key state mutation.
    #[allow(dead_code)] // Will be used when EngineState is implemented
    pub fn is_key_mutation(&self) -> bool {
        matches!(self, Mutation::KeyDown { .. } | Mutation::KeyUp { .. })
    }

    /// Returns true if this is a layer mutation.
    #[allow(dead_code)] // Will be used when EngineState is implemented
    pub fn is_layer_mutation(&self) -> bool {
        matches!(
            self,
            Mutation::PushLayer { .. } | Mutation::PopLayer | Mutation::ToggleLayer { .. }
        )
    }

    /// Returns true if this is a modifier mutation.
    #[allow(dead_code)] // Will be used when EngineState is implemented
    pub fn is_modifier_mutation(&self) -> bool {
        matches!(
            self,
            Mutation::ActivateModifier { .. }
                | Mutation::DeactivateModifier { .. }
                | Mutation::ArmOneShotModifier { .. }
                | Mutation::ClearModifiers
        )
    }

    /// Returns true if this is a pending decision mutation.
    #[allow(dead_code)] // Will be used when EngineState is implemented
    pub fn is_pending_mutation(&self) -> bool {
        matches!(
            self,
            Mutation::AddTapHold { .. }
                | Mutation::AddCombo { .. }
                | Mutation::MarkInterrupted { .. }
                | Mutation::ClearPending
        )
    }

    /// Returns the timestamp associated with this mutation, if any.
    #[allow(dead_code)] // Will be used when EngineState is implemented
    pub fn timestamp(&self) -> Option<u64> {
        match self {
            Mutation::KeyDown { timestamp_us, .. } => Some(*timestamp_us),
            Mutation::KeyUp { timestamp_us, .. } => Some(*timestamp_us),
            Mutation::AddTapHold { pressed_at, .. } => Some(*pressed_at),
            Mutation::AddCombo { started_at, .. } => Some(*started_at),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mutation_classification() {
        let key_down = Mutation::KeyDown {
            key: KeyCode::A,
            timestamp_us: 1000,
            is_repeat: false,
        };
        assert!(key_down.is_key_mutation());
        assert!(!key_down.is_layer_mutation());
        assert_eq!(key_down.timestamp(), Some(1000));

        let push_layer = Mutation::PushLayer { layer_id: 1 };
        assert!(!push_layer.is_key_mutation());
        assert!(push_layer.is_layer_mutation());
        assert_eq!(push_layer.timestamp(), None);

        let activate_mod = Mutation::ActivateModifier { modifier_id: 5 };
        assert!(activate_mod.is_modifier_mutation());

        let add_tap_hold = Mutation::AddTapHold {
            key: KeyCode::A,
            pressed_at: 2000,
            tap_action: KeyCode::B,
            hold_action: HoldAction::Key(KeyCode::C),
        };
        assert!(add_tap_hold.is_pending_mutation());
        assert_eq!(add_tap_hold.timestamp(), Some(2000));
    }

    #[test]
    fn mutation_is_cloneable_and_serializable() {
        let mutation = Mutation::KeyDown {
            key: KeyCode::Space,
            timestamp_us: 5000,
            is_repeat: false,
        };
        let cloned = mutation.clone();
        assert_eq!(mutation, cloned);

        let json = serde_json::to_string(&mutation).expect("serializes");
        let deserialized: Mutation = serde_json::from_str(&json).expect("deserializes");
        assert_eq!(mutation, deserialized);
    }

    #[test]
    fn batch_mutation() {
        let batch = Mutation::Batch {
            mutations: vec![
                Mutation::KeyDown {
                    key: KeyCode::A,
                    timestamp_us: 1000,
                    is_repeat: false,
                },
                Mutation::ActivateModifier { modifier_id: 1 },
            ],
        };
        assert!(!batch.is_key_mutation());
        assert!(!batch.is_layer_mutation());
    }
}
