//! Mock state store for testing.

use crate::engine::{Layer, ModifierSet};
use crate::traits::StateStore;
use std::collections::HashMap;

/// Represents a state change recorded by MockState.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateChange {
    /// A layer was created.
    LayerCreated(String),
    /// A layer was activated.
    LayerActivated(String),
    /// A layer was deactivated.
    LayerDeactivated(String),
    /// Modifiers were set.
    ModifiersSet(Vec<u8>),
}

/// Mock state store for testing.
pub struct MockState {
    layers: HashMap<String, Layer>,
    modifiers: ModifierSet,
    /// History of state changes for verification.
    history: Vec<StateChange>,
}

impl MockState {
    /// Create a new mock state.
    pub fn new() -> Self {
        Self {
            layers: HashMap::new(),
            modifiers: ModifierSet::new(),
            history: Vec::new(),
        }
    }

    /// Get the history of all state changes.
    ///
    /// Useful for verifying state mutations occurred in the expected order.
    pub fn state_history(&self) -> &[StateChange] {
        &self.history
    }

    /// Clear the state history.
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Assert that a layer was activated at some point.
    ///
    /// # Panics
    /// Panics if the layer was never activated.
    pub fn assert_layer_activated(&self, name: &str) {
        let was_activated = self
            .history
            .iter()
            .any(|change| matches!(change, StateChange::LayerActivated(n) if n == name));
        assert!(
            was_activated,
            "Expected layer '{}' to be activated, but it was not. History: {:?}",
            name, self.history
        );
    }

    /// Assert that a layer was deactivated at some point.
    ///
    /// # Panics
    /// Panics if the layer was never deactivated.
    pub fn assert_layer_deactivated(&self, name: &str) {
        let was_deactivated = self
            .history
            .iter()
            .any(|change| matches!(change, StateChange::LayerDeactivated(n) if n == name));
        assert!(
            was_deactivated,
            "Expected layer '{}' to be deactivated, but it was not. History: {:?}",
            name, self.history
        );
    }

    /// Assert that a layer was created at some point.
    ///
    /// # Panics
    /// Panics if the layer was never created.
    pub fn assert_layer_created(&self, name: &str) {
        let was_created = self
            .history
            .iter()
            .any(|change| matches!(change, StateChange::LayerCreated(n) if n == name));
        assert!(
            was_created,
            "Expected layer '{}' to be created, but it was not. History: {:?}",
            name, self.history
        );
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
        // Track the current state before returning mutable reference
        // Since we can't track changes through the mutable reference directly,
        // callers should use MockState-specific methods if tracking is needed
        self.layers.get_mut(name)
    }

    fn create_layer(&mut self, name: &str) -> &mut Layer {
        if !self.layers.contains_key(name) {
            self.history
                .push(StateChange::LayerCreated(name.to_string()));
            self.layers.insert(name.to_string(), Layer::new(name));
        }
        self.layers.get_mut(name).unwrap()
    }

    fn active_modifiers(&self) -> &ModifierSet {
        &self.modifiers
    }

    fn set_active_modifiers(&mut self, mods: ModifierSet) {
        // Extract active modifier IDs for history tracking
        let active_ids: Vec<u8> = (0..=255u8).filter(|&id| mods.contains(id)).collect();
        self.history.push(StateChange::ModifiersSet(active_ids));
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

/// Extension trait for MockState to track layer activation through mutable operations.
impl MockState {
    /// Activate a layer and track the change.
    ///
    /// Unlike `get_layer_mut().activate()`, this method properly records the state change.
    pub fn activate_layer(&mut self, name: &str) -> bool {
        if let Some(layer) = self.layers.get_mut(name) {
            let was_active = layer.active;
            layer.activate();
            let is_active = layer.active;
            // Track after we're done with the mutable borrow
            if !was_active && is_active {
                self.history
                    .push(StateChange::LayerActivated(name.to_string()));
            }
            true
        } else {
            false
        }
    }

    /// Deactivate a layer and track the change.
    ///
    /// Unlike `get_layer_mut().deactivate()`, this method properly records the state change.
    pub fn deactivate_layer(&mut self, name: &str) -> bool {
        if let Some(layer) = self.layers.get_mut(name) {
            let was_active = layer.active;
            layer.deactivate();
            let is_active = layer.active;
            // Track after we're done with the mutable borrow
            if was_active && !is_active {
                self.history
                    .push(StateChange::LayerDeactivated(name.to_string()));
            }
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_history_tracks_layer_creation() {
        let mut state = MockState::new();
        state.create_layer("test_layer");

        assert_eq!(state.state_history().len(), 1);
        assert_eq!(
            state.state_history()[0],
            StateChange::LayerCreated("test_layer".to_string())
        );
    }

    #[test]
    fn test_state_history_tracks_modifier_changes() {
        let mut state = MockState::new();
        let mut mods = ModifierSet::new();
        mods.add(1);
        mods.add(5);
        state.set_active_modifiers(mods);

        assert_eq!(state.state_history().len(), 1);
        match &state.state_history()[0] {
            StateChange::ModifiersSet(ids) => {
                assert!(ids.contains(&1));
                assert!(ids.contains(&5));
            }
            _ => panic!("Expected ModifiersSet"),
        }
    }

    #[test]
    fn test_activate_layer_tracks_change() {
        let mut state = MockState::new();
        state.create_layer("nav");
        state.activate_layer("nav");

        assert!(state
            .state_history()
            .contains(&StateChange::LayerActivated("nav".to_string())));
    }

    #[test]
    fn test_deactivate_layer_tracks_change() {
        let mut state = MockState::new();
        state.create_layer("nav");
        state.activate_layer("nav");
        state.deactivate_layer("nav");

        assert!(state
            .state_history()
            .contains(&StateChange::LayerDeactivated("nav".to_string())));
    }

    #[test]
    fn test_assert_layer_activated_passes() {
        let mut state = MockState::new();
        state.create_layer("test");
        state.activate_layer("test");

        state.assert_layer_activated("test"); // Should not panic
    }

    #[test]
    #[should_panic(expected = "Expected layer 'missing' to be activated")]
    fn test_assert_layer_activated_fails() {
        let state = MockState::new();
        state.assert_layer_activated("missing");
    }

    #[test]
    fn test_assert_layer_deactivated_passes() {
        let mut state = MockState::new();
        state.create_layer("test");
        state.activate_layer("test");
        state.deactivate_layer("test");

        state.assert_layer_deactivated("test"); // Should not panic
    }

    #[test]
    fn test_assert_layer_created_passes() {
        let mut state = MockState::new();
        state.create_layer("test");

        state.assert_layer_created("test"); // Should not panic
    }

    #[test]
    fn test_clear_history() {
        let mut state = MockState::new();
        state.create_layer("test");
        assert!(!state.state_history().is_empty());

        state.clear_history();
        assert!(state.state_history().is_empty());
    }

    #[test]
    fn test_duplicate_activation_not_tracked() {
        let mut state = MockState::new();
        state.create_layer("test");
        state.activate_layer("test");
        state.activate_layer("test"); // Already active

        // Should only have one activation
        let activations: Vec<_> = state
            .state_history()
            .iter()
            .filter(|c| matches!(c, StateChange::LayerActivated(_)))
            .collect();
        assert_eq!(activations.len(), 1);
    }
}
