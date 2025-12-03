//! Layer and modifier state management.

mod change;
mod error;
mod key_state;
mod keys;
mod layers;
mod modifiers;
mod mutation;

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[allow(unused_imports)] // Will be used when EngineState is implemented
pub use change::{
    AutoReleaseReason, Effect, HoldOutcome, PendingDecisionType, PendingResolution, StateChange,
};
#[allow(unused_imports)] // Will be used when EngineState is implemented
pub use error::{StateError, StateResult};
pub use key_state::KeyStateTracker;
#[allow(unused_imports)] // Will be used when EngineState is implemented
pub use keys::KeyState;
#[allow(unused_imports)] // Will be used when EngineState is implemented
pub use layers::LayerState;
pub use layers::{HoldAction, Layer, LayerAction, LayerId, LayerStack};
pub use modifiers::{
    Modifier, ModifierState, StandardModifier, StandardModifiers, VirtualModifiers,
};
#[allow(unused_imports)] // Will be used when EngineState is implemented
pub use mutation::Mutation;

/// A set of active modifiers (both physical and virtual).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModifierSet {
    /// Active modifier IDs (0-255 for custom modifiers).
    active: HashSet<u8>,
}

impl ModifierSet {
    /// Create empty modifier set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a modifier.
    pub fn add(&mut self, id: u8) {
        self.active.insert(id);
    }

    /// Remove a modifier.
    pub fn remove(&mut self, id: u8) {
        self.active.remove(&id);
    }

    /// Check if modifier is active.
    pub fn contains(&self, id: u8) -> bool {
        self.active.contains(&id)
    }

    /// Check if all specified modifiers are active.
    pub fn contains_all(&self, ids: &[u8]) -> bool {
        ids.iter().all(|id| self.active.contains(id))
    }

    /// Clear all modifiers.
    pub fn clear(&mut self) {
        self.active.clear();
    }

    /// Return active modifier IDs in deterministic order for telemetry/FFI.
    pub fn active_ids(&self) -> Vec<u8> {
        let mut ids: Vec<u8> = self.active.iter().copied().collect();
        ids.sort_unstable();
        ids
    }
}
