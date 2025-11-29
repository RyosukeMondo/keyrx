//! Layer and modifier state management.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

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

/// A keyboard layer with its own key mappings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    /// Layer name.
    pub name: String,
    /// Whether this layer is currently active.
    pub active: bool,
    /// Priority (higher = checked first).
    pub priority: i32,
}

impl Layer {
    /// Create a new layer.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            active: false,
            priority: 0,
        }
    }

    /// Activate this layer.
    pub fn activate(&mut self) {
        self.active = true;
    }

    /// Deactivate this layer.
    pub fn deactivate(&mut self) {
        self.active = false;
    }
}
