//! Layer state tracking for the unified engine state.
//!
//! This module provides both the legacy LayerStack for backward compatibility and
//! the new LayerState component which tracks active layers and their priorities.
//! LayerState is designed to be part of the unified EngineState.

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

/// Unified layer state component for the engine.
///
/// This tracks which layers are currently active and provides efficient
/// operations for layer management. It's designed to be part of EngineState.
///
/// Key differences from LayerStack:
/// - Focused on state tracking, not layer definitions
/// - Cleaner separation of concerns
/// - More efficient for state queries
#[derive(Debug, Clone)]
#[allow(dead_code)] // Will be used when EngineState is implemented in Phase 3
pub struct LayerState {
    /// Active layer IDs in priority order (last = highest priority).
    /// The first element is always the base layer.
    stack: Vec<LayerId>,
    /// Base layer ID (always present in stack).
    base: LayerId,
}

impl Default for LayerState {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)] // Will be used when EngineState is implemented in Phase 3
impl LayerState {
    /// Create a new LayerState with base layer (id 0).
    pub fn new() -> Self {
        Self::with_base(0)
    }

    /// Create a new LayerState with a specific base layer ID.
    pub fn with_base(base: LayerId) -> Self {
        Self {
            stack: vec![base],
            base,
        }
    }

    /// Push a layer to the top of the stack.
    ///
    /// If the layer is already in the stack, it's moved to the top.
    /// The base layer cannot be pushed (it's always at the bottom).
    ///
    /// # Arguments
    /// * `layer_id` - The layer to activate
    ///
    /// # Returns
    /// `true` if the stack changed, `false` otherwise
    pub fn push(&mut self, layer_id: LayerId) -> bool {
        // Cannot push the base layer
        if layer_id == self.base {
            return false;
        }

        // If already at the top, no change needed
        if self.stack.last() == Some(&layer_id) {
            return false;
        }

        // Remove from current position if present
        if let Some(pos) = self.stack.iter().position(|&id| id == layer_id) {
            self.stack.remove(pos);
        }

        // Add to top
        self.stack.push(layer_id);
        true
    }

    /// Pop the top-most non-base layer from the stack.
    ///
    /// # Returns
    /// The layer ID that was popped, or None if only the base layer remains
    pub fn pop(&mut self) -> Option<LayerId> {
        if self.stack.len() <= 1 {
            return None;
        }

        self.stack.pop()
    }

    /// Toggle a layer on/off.
    ///
    /// If the layer is active, it's removed. If inactive, it's pushed to the top.
    /// The base layer cannot be toggled.
    ///
    /// # Arguments
    /// * `layer_id` - The layer to toggle
    ///
    /// # Returns
    /// `true` if the operation succeeded, `false` if the layer is the base
    pub fn toggle(&mut self, layer_id: LayerId) -> bool {
        if layer_id == self.base {
            return false;
        }

        if let Some(pos) = self.stack.iter().position(|&id| id == layer_id) {
            // Layer is active - remove it
            self.stack.remove(pos);
        } else {
            // Layer is inactive - push it
            self.stack.push(layer_id);
        }
        true
    }

    /// Check if a layer is currently active.
    ///
    /// # Arguments
    /// * `layer_id` - The layer to check
    ///
    /// # Returns
    /// `true` if the layer is in the active stack
    #[inline]
    pub fn is_active(&self, layer_id: LayerId) -> bool {
        self.stack.contains(&layer_id)
    }

    /// Get the top-most active layer ID.
    ///
    /// # Returns
    /// The ID of the highest-priority active layer
    #[inline]
    pub fn top_layer(&self) -> LayerId {
        *self.stack.last().unwrap_or(&self.base)
    }

    /// Get all active layer IDs in priority order (first = lowest priority).
    ///
    /// # Returns
    /// A slice of active layer IDs with the base layer first
    #[inline]
    pub fn active_layers(&self) -> &[LayerId] {
        &self.stack
    }

    /// Get the number of active layers (including base).
    #[inline]
    pub fn len(&self) -> usize {
        self.stack.len()
    }

    /// Check if only the base layer is active.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.stack.len() <= 1
    }

    /// Clear all non-base layers, leaving only the base layer active.
    pub fn clear(&mut self) {
        self.stack.truncate(1);
    }

    /// Get the base layer ID.
    #[inline]
    pub fn base_layer(&self) -> LayerId {
        self.base
    }

    /// Get a copy of the active layer IDs as a vector.
    ///
    /// This is useful when you need ownership of the layer list.
    pub fn active_layers_vec(&self) -> Vec<LayerId> {
        self.stack.clone()
    }
}

/// Stack of active layers (legacy implementation).
///
/// This is the original LayerStack implementation maintained for backward
/// compatibility. New code should prefer the unified LayerState component.
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

    /// Get the active layer names in priority order (base first).
    pub fn active_layer_names(&self) -> Vec<String> {
        self.stack
            .iter()
            .filter_map(|id| self.layers.get(id).map(|layer| layer.name.clone()))
            .collect()
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

    // LayerState tests
    mod layer_state_tests {
        use super::*;

        #[test]
        fn new_layer_state_has_base_only() {
            let state = LayerState::new();
            assert_eq!(state.len(), 1);
            assert!(state.is_empty());
            assert_eq!(state.top_layer(), 0);
            assert!(state.is_active(0));
        }

        #[test]
        fn with_base_creates_correct_state() {
            let state = LayerState::with_base(5);
            assert_eq!(state.base_layer(), 5);
            assert_eq!(state.top_layer(), 5);
            assert!(state.is_active(5));
        }

        #[test]
        fn push_adds_layer_to_top() {
            let mut state = LayerState::new();
            assert!(state.push(1));
            assert_eq!(state.len(), 2);
            assert_eq!(state.top_layer(), 1);
            assert!(state.is_active(1));
        }

        #[test]
        fn push_base_layer_returns_false() {
            let mut state = LayerState::new();
            assert!(!state.push(0));
            assert_eq!(state.len(), 1);
        }

        #[test]
        fn push_existing_layer_moves_to_top() {
            let mut state = LayerState::new();
            state.push(1);
            state.push(2);
            assert!(state.push(1));

            let layers = state.active_layers();
            assert_eq!(layers, &[0, 2, 1]);
            assert_eq!(state.top_layer(), 1);
        }

        #[test]
        fn push_already_at_top_returns_false() {
            let mut state = LayerState::new();
            state.push(1);
            assert!(!state.push(1));
            assert_eq!(state.len(), 2);
        }

        #[test]
        fn pop_removes_top_layer() {
            let mut state = LayerState::new();
            state.push(1);
            state.push(2);

            assert_eq!(state.pop(), Some(2));
            assert_eq!(state.top_layer(), 1);
            assert!(!state.is_active(2));
        }

        #[test]
        fn pop_with_only_base_returns_none() {
            let mut state = LayerState::new();
            assert_eq!(state.pop(), None);
            assert_eq!(state.len(), 1);
            assert!(state.is_active(0));
        }

        #[test]
        fn toggle_activates_inactive_layer() {
            let mut state = LayerState::new();
            assert!(state.toggle(1));
            assert!(state.is_active(1));
            assert_eq!(state.top_layer(), 1);
        }

        #[test]
        fn toggle_deactivates_active_layer() {
            let mut state = LayerState::new();
            state.push(1);
            assert!(state.toggle(1));
            assert!(!state.is_active(1));
            assert_eq!(state.top_layer(), 0);
        }

        #[test]
        fn toggle_base_layer_returns_false() {
            let mut state = LayerState::new();
            assert!(!state.toggle(0));
            assert!(state.is_active(0));
        }

        #[test]
        fn is_active_checks_presence_in_stack() {
            let mut state = LayerState::new();
            state.push(1);
            state.push(2);

            assert!(state.is_active(0));
            assert!(state.is_active(1));
            assert!(state.is_active(2));
            assert!(!state.is_active(3));
        }

        #[test]
        fn active_layers_returns_slice() {
            let mut state = LayerState::new();
            state.push(1);
            state.push(2);

            let layers = state.active_layers();
            assert_eq!(layers, &[0, 1, 2]);
        }

        #[test]
        fn clear_removes_all_non_base_layers() {
            let mut state = LayerState::new();
            state.push(1);
            state.push(2);
            state.push(3);

            state.clear();
            assert_eq!(state.len(), 1);
            assert!(state.is_empty());
            assert_eq!(state.top_layer(), 0);
            assert!(state.is_active(0));
            assert!(!state.is_active(1));
        }

        #[test]
        fn active_layers_vec_returns_owned_copy() {
            let mut state = LayerState::new();
            state.push(1);
            state.push(2);

            let vec = state.active_layers_vec();
            assert_eq!(vec, vec![0, 1, 2]);
        }

        #[test]
        fn multiple_operations_maintain_consistency() {
            let mut state = LayerState::new();

            state.push(1);
            state.push(2);
            assert_eq!(state.top_layer(), 2);

            state.pop();
            assert_eq!(state.top_layer(), 1);

            state.toggle(2);
            assert_eq!(state.top_layer(), 2);

            state.toggle(1);
            assert_eq!(state.top_layer(), 2);
            assert!(!state.is_active(1));

            state.clear();
            assert_eq!(state.top_layer(), 0);
        }

        #[test]
        fn default_creates_base_state() {
            let state = LayerState::default();
            assert_eq!(state.base_layer(), 0);
            assert!(state.is_empty());
        }
    }

    // Legacy LayerStack tests
    mod layer_stack_tests {
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
}
