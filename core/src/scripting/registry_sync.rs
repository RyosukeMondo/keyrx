//! Registry synchronization for the Rhai scripting runtime.
//!
//! This module handles synchronizing the layer and modifier views with
//! the underlying registry after pending operations have been applied.

use super::builtins::{LayerView, ModifierView};
use crate::scripting::RemapRegistry;

/// Handles synchronization of views with the registry.
///
/// After pending operations are applied to the registry, the layer and
/// modifier views need to be updated to reflect the new state. This struct
/// encapsulates that synchronization logic.
pub struct RegistrySyncer {
    layer_view: LayerView,
    modifier_view: ModifierView,
}

impl RegistrySyncer {
    /// Create a new syncer with the given views.
    pub fn new(layer_view: LayerView, modifier_view: ModifierView) -> Self {
        Self {
            layer_view,
            modifier_view,
        }
    }

    /// Synchronize both views from the registry.
    pub fn sync_from_registry(&mut self, registry: &RemapRegistry) {
        self.sync_layer_view(registry);
        self.sync_modifier_view(registry);
    }

    /// Synchronize the layer view from the registry.
    fn sync_layer_view(&mut self, registry: &RemapRegistry) {
        if let Ok(mut view) = self.layer_view.lock() {
            *view = registry.layers().clone();
        }
    }

    /// Synchronize the modifier view from the registry.
    fn sync_modifier_view(&mut self, registry: &RemapRegistry) {
        if let Ok(mut view) = self.modifier_view.lock() {
            view.sync_from_registry(registry);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::LayerStack;
    use crate::scripting::builtins::ModifierPreview;
    use std::sync::{Arc, Mutex};

    #[test]
    fn syncer_updates_layer_view() {
        let layer_view: LayerView = Arc::new(Mutex::new(LayerStack::new()));
        let modifier_view: ModifierView = Arc::new(Mutex::new(ModifierPreview::new()));
        let mut syncer = RegistrySyncer::new(layer_view.clone(), modifier_view);

        let mut registry = RemapRegistry::new();
        registry.define_layer("test", false).unwrap();
        registry.push_layer("test").unwrap();

        syncer.sync_from_registry(&registry);

        let view = layer_view.lock().unwrap();
        assert!(view.is_active(1)); // Layer 1 should be active (base is 0)
    }

    #[test]
    fn syncer_updates_modifier_view() {
        let layer_view: LayerView = Arc::new(Mutex::new(LayerStack::new()));
        let modifier_view: ModifierView = Arc::new(Mutex::new(ModifierPreview::new()));
        let mut syncer = RegistrySyncer::new(layer_view, modifier_view.clone());

        let mut registry = RemapRegistry::new();
        registry.define_modifier_with_id("hyper", None).unwrap();

        syncer.sync_from_registry(&registry);

        let view = modifier_view.lock().unwrap();
        assert!(view.id_for("hyper", "test").is_ok());
    }
}
