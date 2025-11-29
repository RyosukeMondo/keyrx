//! State store trait for layer and modifier management.

use crate::engine::{Layer, ModifierSet};

/// Trait for engine state storage.
///
/// Implementations:
/// - `InMemoryState`: Production in-memory state
/// - `MockState`: Test mock for state simulation
pub trait StateStore {
    /// Get a layer by name.
    fn get_layer(&self, name: &str) -> Option<&Layer>;

    /// Get mutable layer by name.
    fn get_layer_mut(&mut self, name: &str) -> Option<&mut Layer>;

    /// Create a new layer.
    fn create_layer(&mut self, name: &str) -> &mut Layer;

    /// Get currently active modifiers.
    fn active_modifiers(&self) -> &ModifierSet;

    /// Set active modifiers.
    fn set_active_modifiers(&mut self, mods: ModifierSet);

    /// Get list of active layer names.
    fn active_layers(&self) -> Vec<&str>;
}
