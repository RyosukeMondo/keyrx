//! Mock state store for testing.

use crate::engine::{Layer, ModifierSet};
use crate::traits::StateStore;
use std::collections::HashMap;

/// Mock state store for testing.
pub struct MockState {
    layers: HashMap<String, Layer>,
    modifiers: ModifierSet,
}

impl MockState {
    /// Create a new mock state.
    pub fn new() -> Self {
        Self {
            layers: HashMap::new(),
            modifiers: ModifierSet::new(),
        }
    }
}

impl Default for MockState {
    fn default() -> Self {
        Self::new()
    }
}

impl StateStore for MockState {
    fn get_layer(&self, name: &str) -> Option<&Layer> {
        self.layers.get(name)
    }

    fn get_layer_mut(&mut self, name: &str) -> Option<&mut Layer> {
        self.layers.get_mut(name)
    }

    fn create_layer(&mut self, name: &str) -> &mut Layer {
        self.layers
            .entry(name.to_string())
            .or_insert_with(|| Layer::new(name))
    }

    fn active_modifiers(&self) -> &ModifierSet {
        &self.modifiers
    }

    fn set_active_modifiers(&mut self, mods: ModifierSet) {
        self.modifiers = mods;
    }

    fn active_layers(&self) -> Vec<&str> {
        self.layers
            .values()
            .filter(|l| l.active)
            .map(|l| l.name.as_str())
            .collect()
    }
}
