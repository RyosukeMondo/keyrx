//! Layer and modifier state management.

mod key_state;
mod layers;
mod modifiers;

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub use key_state::KeyStateTracker;
pub use layers::{HoldAction, Layer, LayerAction, LayerId, LayerStack};
pub use modifiers::{
    Modifier, ModifierState, StandardModifier, StandardModifiers, VirtualModifiers,
};

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
}
