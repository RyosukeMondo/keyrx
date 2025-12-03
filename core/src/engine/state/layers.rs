use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::engine::KeyCode;
use crate::traits::LayerProvider;

pub type LayerId = u16;

/// A keyboard layer with its own key mappings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layer {
    /// Stable layer identifier.
    pub id: LayerId,
    /// Human-friendly layer name.
    pub name: String,
    /// Key mappings for this layer.
    pub mappings: HashMap<KeyCode, LayerAction>,
    /// If true, missing mappings fall through to lower layers.
    pub transparent: bool,
    /// Whether this layer is currently active (derived from the stack).
    pub active: bool,
    /// Priority hint for debug display (stack order wins at runtime).
    pub priority: i32,
}

impl Layer {
    /// Create a base layer (id 0) with default settings.
    pub fn base() -> Self {
        Self::with_id(0, "base")
    }

    /// Create a new layer with name and default id (0). Use `with_id` for stable ids.
    pub fn new(name: impl Into<String>) -> Self {
        Self::with_id(0, name)
    }

    /// Create a new layer with explicit id.
    pub fn with_id(id: LayerId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            mappings: HashMap::new(),
            transparent: false,
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

    /// Set a mapping for a key on this layer.
    pub fn set_mapping(&mut self, key: KeyCode, action: LayerAction) {
        self.mappings.insert(key, action);
    }
}

/// Action within a layer (superset of RemapAction).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayerAction {
    /// Simple remap.
    Remap(KeyCode),
    /// Block the key.
    Block,
    /// Tap-hold behavior.
    TapHold {
        tap: KeyCode,
        hold: HoldAction,
    },
    /// Layer control.
    LayerPush(LayerId),
    LayerPop,
    LayerToggle(LayerId),
    /// Modifier control.
    ModifierActivate(u8),
    ModifierDeactivate(u8),
    ModifierOneShot(u8),
    /// Explicitly pass through.
    Pass,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HoldAction {
    Key(KeyCode),
    Modifier(u8),
    Layer(LayerId),
}

/// Stack of active layers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerStack {
    /// All defined layers.
    layers: HashMap<LayerId, Layer>,
    /// Active layer IDs in priority order (last = highest).
    stack: Vec<LayerId>,
    /// Base layer ID (always present).
    base: LayerId,
}

impl Default for LayerStack {
    fn default() -> Self {
        Self::new()
    }
}

impl LayerStack {
    /// Create a new stack with a default base layer.
    pub fn new() -> Self {
        Self::with_base_layer(Layer::base())
    }

    /// Create a new stack with the provided base layer.
    pub fn with_base_layer(mut base_layer: Layer) -> Self {
        base_layer.activate();
        let base = base_layer.id;
        let mut layers = HashMap::new();
        layers.insert(base, base_layer);
        Self {
            layers,
            stack: vec![base],
            base,
        }
    }

    /// Define or replace a layer.
    ///
    /// If the layer is already active in the stack, the active flag is preserved.
    pub fn define_layer(&mut self, mut layer: Layer) {
        let id = layer.id;
        let was_active = self.is_active(id);
        if was_active {
            layer.activate();
        }

        self.layers.insert(id, layer);

        // Ensure base layer is always present and active.
        if !self.layers.contains_key(&self.base) {
            let mut base_layer = Layer::with_id(self.base, "base");
            base_layer.activate();
            self.layers.insert(self.base, base_layer);
            if !self.stack.contains(&self.base) {
                self.stack.insert(0, self.base);
            }
        }
    }

    /// Push a layer to the top of the stack (deduplicated).
    pub fn push(&mut self, layer_id: LayerId) -> bool {
        if layer_id == self.base {
            return false;
        }
        if !self.layers.contains_key(&layer_id) {
            return false;
        }

        if let Some(pos) = self.stack.iter().position(|&id| id == layer_id) {
            if pos == self.stack.len() - 1 {
                return false;
            }
            self.stack.remove(pos);
        }

        self.stack.push(layer_id);
        if let Some(layer) = self.layers.get_mut(&layer_id) {
            layer.activate();
        }
        true
    }

    /// Pop the top-most non-base layer.
    pub fn pop(&mut self) -> Option<LayerId> {
        if self.stack.len() <= 1 {
            return None;
        }

        let id = self.stack.pop()?;
        if let Some(layer) = self.layers.get_mut(&id) {
            layer.deactivate();
        }
        Some(id)
    }

    /// Toggle a layer on/off.
    pub fn toggle(&mut self, layer_id: LayerId) -> bool {
        if layer_id == self.base || !self.layers.contains_key(&layer_id) {
            return false;
        }

        if let Some(pos) = self.stack.iter().position(|&id| id == layer_id) {
            self.stack.remove(pos);
            if let Some(layer) = self.layers.get_mut(&layer_id) {
                layer.deactivate();
            }
            true
        } else {
            self.stack.push(layer_id);
            if let Some(layer) = self.layers.get_mut(&layer_id) {
                layer.activate();
            }
            true
        }
    }

    /// Check if a layer is active.
    pub fn is_active(&self, layer_id: LayerId) -> bool {
        self.stack.contains(&layer_id)
    }

    /// Number of defined layers (including base).
    pub fn len(&self) -> usize {
        self.layers.len()
    }

    /// True when only the base layer is present.
    pub fn is_empty(&self) -> bool {
        self.layers.len() <= 1
    }

    /// Find a layer ID by name (case-insensitive).
    pub fn layer_id_by_name(&self, name: &str) -> Option<LayerId> {
        let needle = name.to_ascii_lowercase();
        self.layers
            .iter()
            .find_map(|(&id, layer)| (layer.name.to_ascii_lowercase() == needle).then_some(id))
    }

    /// Define or update a layer by name, returning its stable ID.
    ///
    /// If the layer already exists, the name/transparency are updated while
    /// preserving mappings and activation state.
    pub fn define_or_update_named(&mut self, name: &str, transparent: bool) -> LayerId {
        if let Some(id) = self.layer_id_by_name(name) {
            if let Some(layer) = self.layers.get_mut(&id) {
                layer.name = name.to_string();
                layer.transparent = transparent;
            }
            return id;
        }

        let id = self.next_layer_id();
        let mut layer = Layer::with_id(id, name.to_string());
        layer.transparent = transparent;
        self.define_layer(layer);
        id
    }

    /// Set a mapping on a specific layer by ID.
    pub fn set_mapping_for_layer(
        &mut self,
        layer_id: LayerId,
        key: KeyCode,
        action: LayerAction,
    ) -> bool {
        if let Some(layer) = self.layers.get_mut(&layer_id) {
            layer.set_mapping(key, action);
            true
        } else {
            false
        }
    }

    /// Check if a layer is active by name (case-insensitive).
    pub fn is_active_by_name(&self, name: &str) -> bool {
        self.layer_id_by_name(name)
            .map(|id| self.is_active(id))
            .unwrap_or(false)
    }

    fn next_layer_id(&self) -> LayerId {
        self.layers
            .keys()
            .copied()
            .max()
            .map(|max| max.saturating_add(1))
            .unwrap_or(0)
    }

    /// Look up action for a key, checking layers top to bottom.
    pub fn lookup(&self, key: KeyCode) -> Option<&LayerAction> {
        for id in self.stack.iter().rev() {
            if let Some(layer) = self.layers.get(id) {
                if let Some(action) = layer.mappings.get(&key) {
                    return Some(action);
                }
                if !layer.transparent {
                    return None;
                }
            }
        }
        None
    }

    /// Get active layer names for debugging in top-to-bottom order.
    pub fn active_layers(&self) -> Vec<&str> {
        self.stack
            .iter()
            .rev()
            .filter_map(|id| self.layers.get(id))
            .map(|layer| layer.name.as_str())
            .collect()
    }

    /// Get active layer IDs in priority order (last = highest).
    pub fn active_layer_ids(&self) -> Vec<u32> {
        self.stack.iter().map(|&id| id as u32).collect()
    }
}

impl LayerProvider for LayerStack {
    fn active_layer(&self) -> LayerId {
        *self.stack.last().unwrap_or(&self.base)
    }

    fn active_layer_ids(&self) -> Vec<LayerId> {
        self.stack.clone()
    }

    fn push(&mut self, layer_id: LayerId) -> bool {
        LayerStack::push(self, layer_id)
    }

    fn pop(&mut self) -> Option<LayerId> {
        LayerStack::pop(self)
    }

    fn toggle(&mut self, layer_id: LayerId) -> bool {
        LayerStack::toggle(self, layer_id)
    }

    fn is_active(&self, layer_id: LayerId) -> bool {
        LayerStack::is_active(self, layer_id)
    }

    fn lookup(&self, key: KeyCode) -> Option<&LayerAction> {
        LayerStack::lookup(self, key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::KeyCode;

    #[test]
    fn push_adds_layer_and_respects_priority() {
        let mut stack = LayerStack::new();
        let mut nav = Layer::with_id(1, "nav");
        nav.set_mapping(KeyCode::A, LayerAction::Remap(KeyCode::B));
        stack.define_layer(nav);

        let mut fn_layer = Layer::with_id(2, "fn");
        fn_layer.set_mapping(KeyCode::A, LayerAction::Block);
        stack.define_layer(fn_layer);

        assert!(stack.push(1));
        assert!(stack.push(2));

        let action = stack.lookup(KeyCode::A);
        assert_eq!(action, Some(&LayerAction::Block));
        assert!(stack.is_active(1));
        assert!(stack.is_active(2));
    }

    #[test]
    fn pop_deactivates_top_non_base_layer() {
        let mut stack = LayerStack::new();
        let nav = Layer::with_id(1, "nav");
        stack.define_layer(nav);
        assert!(stack.push(1));

        let popped = stack.pop();
        assert_eq!(popped, Some(1));
        assert!(!stack.is_active(1));
        // Base layer remains.
        assert!(stack.is_active(0));
        assert!(stack.pop().is_none());
    }

    #[test]
    fn toggle_toggles_activation_state() {
        let mut stack = LayerStack::new();
        stack.define_layer(Layer::with_id(1, "nav"));

        assert!(stack.toggle(1));
        assert!(stack.is_active(1));
        assert!(stack.toggle(1));
        assert!(!stack.is_active(1));
    }

    #[test]
    fn lookup_respects_transparency() {
        let mut stack = LayerStack::new();

        let mut base = Layer::with_id(0, "base");
        base.set_mapping(KeyCode::A, LayerAction::Remap(KeyCode::B));
        stack.define_layer(base);

        let mut overlay = Layer::with_id(1, "overlay");
        overlay.transparent = true;
        stack.define_layer(overlay);
        stack.push(1);

        assert_eq!(
            stack.lookup(KeyCode::A),
            Some(&LayerAction::Remap(KeyCode::B))
        );
    }

    #[test]
    fn opaque_layer_without_mapping_blocks_lookup() {
        let mut stack = LayerStack::new();
        let mut opaque = Layer::with_id(1, "opaque");
        opaque.transparent = false;
        stack.define_layer(opaque);
        stack.push(1);

        assert_eq!(stack.lookup(KeyCode::A), None);
    }

    #[test]
    fn push_moves_existing_layer_to_top() {
        let mut stack = LayerStack::new();
        stack.define_layer(Layer::with_id(1, "nav"));
        stack.define_layer(Layer::with_id(2, "fn"));
        assert!(stack.push(1));
        assert!(stack.push(2));

        // Re-pushing layer 1 should move it above layer 2.
        assert!(stack.push(1));
        let names = stack.active_layers();
        assert_eq!(names.first(), Some(&"nav"));
    }
}
