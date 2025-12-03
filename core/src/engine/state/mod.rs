//! Layer and modifier state management.

mod change;
mod error;
mod key_state;
mod keys;
mod layers;
mod modifiers;
mod mutation;
mod pending;

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::engine::decision::timing::TimingConfig;
use crate::engine::KeyCode;

#[allow(unused_imports)] // Will be used in tasks 9-11 for apply() and apply_batch()
pub use change::{
    AutoReleaseReason, Effect, HoldOutcome, PendingDecisionType, PendingResolution, StateChange,
};
#[allow(unused_imports)] // Will be used in tasks 9-11 for apply() and apply_batch()
pub use error::{StateError, StateResult};
pub use key_state::KeyStateTracker;
pub use keys::KeyState;
pub use layers::LayerState;
pub use layers::{HoldAction, Layer, LayerAction, LayerId, LayerStack};
pub use modifiers::{
    Modifier, ModifierState, StandardModifier, StandardModifiers, VirtualModifiers,
};
pub use mutation::Mutation;
pub use pending::PendingState;

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

/// Unified engine state container.
///
/// EngineState combines all engine state components into a single, cohesive
/// state structure. It provides:
/// - Unified access to all state (keys, layers, modifiers, pending decisions)
/// - Version tracking for state changes
/// - Query methods for efficient state inspection
/// - Clone support for snapshotting and testing
///
/// # Components
///
/// - **KeyState**: Tracks which physical keys are pressed and their timestamps
/// - **LayerState**: Manages the active layer stack
/// - **ModifierState**: Tracks standard and virtual modifier states
/// - **PendingState**: Manages pending tap-hold and combo decisions
///
/// # Version Tracking
///
/// The state version increments with each mutation, enabling:
/// - Detection of stale state references
/// - Ordering of concurrent changes
/// - Cache invalidation based on version
///
/// # Example
///
/// ```no_run
/// use keyrx_core::engine::state::EngineState;
/// use keyrx_core::engine::decision::timing::TimingConfig;
///
/// let state = EngineState::new(TimingConfig::default());
///
/// // Query key state
/// let is_pressed = state.is_key_pressed(keyrx_core::engine::KeyCode::A);
///
/// // Query layer state
/// let active_layers = state.active_layers();
///
/// // Query modifier state
/// let is_shift_active = state.is_modifier_active(
///     keyrx_core::engine::state::Modifier::Standard(
///         keyrx_core::engine::state::StandardModifier::Shift
///     )
/// );
/// ```
#[derive(Debug, Clone)]
#[allow(dead_code)] // Will be used in tasks 9-18 for mutations, integration, and cleanup
pub struct EngineState {
    /// Physical key press state.
    keys: KeyState,
    /// Active layer stack.
    layers: LayerState,
    /// Active modifiers (standard and virtual).
    modifiers: ModifierState,
    /// Pending tap-hold and combo decisions.
    pending: PendingState,
    /// State version counter (increments with each mutation).
    version: u64,
}

#[allow(dead_code)] // Will be used in tasks 9-18 for mutations, integration, and cleanup
impl EngineState {
    /// Create a new EngineState with default components.
    ///
    /// # Arguments
    ///
    /// * `timing_config` - Timing configuration for pending decision resolution
    pub fn new(timing_config: TimingConfig) -> Self {
        Self {
            keys: KeyState::new(),
            layers: LayerState::new(),
            modifiers: ModifierState::new(),
            pending: PendingState::new(timing_config),
            version: 0,
        }
    }

    /// Create a new EngineState with a specific base layer.
    ///
    /// # Arguments
    ///
    /// * `base_layer` - The base layer ID
    /// * `timing_config` - Timing configuration for pending decision resolution
    pub fn with_base_layer(base_layer: LayerId, timing_config: TimingConfig) -> Self {
        Self {
            keys: KeyState::new(),
            layers: LayerState::with_base(base_layer),
            modifiers: ModifierState::new(),
            pending: PendingState::new(timing_config),
            version: 0,
        }
    }

    // === Version Tracking ===

    /// Get the current state version.
    ///
    /// The version increments with each mutation applied to the state.
    #[inline]
    pub fn version(&self) -> u64 {
        self.version
    }

    // === Key State Queries ===

    /// Check if a key is currently pressed.
    #[inline]
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.keys.is_pressed(key)
    }

    /// Get the timestamp when a key was first pressed.
    ///
    /// Returns None if the key is not currently pressed.
    #[inline]
    pub fn key_press_time(&self, key: KeyCode) -> Option<u64> {
        self.keys.press_time(key)
    }

    /// Get an iterator over all currently pressed keys.
    pub fn pressed_keys(&self) -> impl Iterator<Item = KeyCode> + '_ {
        self.keys.pressed_keys()
    }

    /// Get all pressed keys and their timestamps.
    pub fn all_pressed_keys(&self) -> Vec<(KeyCode, u64)> {
        self.keys.all_pressed()
    }

    /// Get the number of currently pressed keys.
    #[inline]
    pub fn pressed_key_count(&self) -> usize {
        self.keys.len()
    }

    /// Check if no keys are currently pressed.
    #[inline]
    pub fn no_keys_pressed(&self) -> bool {
        self.keys.is_empty()
    }

    // === Layer State Queries ===

    /// Get all active layer IDs in priority order (first = lowest priority).
    #[inline]
    pub fn active_layers(&self) -> &[LayerId] {
        self.layers.active_layers()
    }

    /// Get the top-most active layer ID.
    #[inline]
    pub fn top_layer(&self) -> LayerId {
        self.layers.top_layer()
    }

    /// Get the base layer ID.
    #[inline]
    pub fn base_layer(&self) -> LayerId {
        self.layers.base_layer()
    }

    /// Check if a layer is currently active.
    #[inline]
    pub fn is_layer_active(&self, layer_id: LayerId) -> bool {
        self.layers.is_active(layer_id)
    }

    /// Get the number of active layers (including base).
    #[inline]
    pub fn active_layer_count(&self) -> usize {
        self.layers.len()
    }

    /// Check if only the base layer is active.
    #[inline]
    pub fn only_base_layer_active(&self) -> bool {
        self.layers.is_empty()
    }

    // === Modifier State Queries ===

    /// Check if a modifier is currently active.
    #[inline]
    pub fn is_modifier_active(&self, modifier: Modifier) -> bool {
        self.modifiers.is_active(modifier)
    }

    /// Get the standard modifier state.
    #[inline]
    pub fn standard_modifiers(&self) -> StandardModifiers {
        self.modifiers.standard()
    }

    /// Get the virtual modifier state.
    #[inline]
    pub fn virtual_modifiers(&self) -> VirtualModifiers {
        self.modifiers.virtual_mods()
    }

    // === Pending Decision Queries ===

    /// Get the number of pending decisions.
    #[inline]
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Check if there are no pending decisions.
    #[inline]
    pub fn no_pending_decisions(&self) -> bool {
        self.pending.is_empty()
    }

    // === Component Access ===
    // These methods provide direct access to components for advanced use cases.
    // Most callers should prefer the query methods above.

    /// Get a reference to the key state component.
    #[inline]
    pub fn keys(&self) -> &KeyState {
        &self.keys
    }

    /// Get a reference to the layer state component.
    #[inline]
    pub fn layers(&self) -> &LayerState {
        &self.layers
    }

    /// Get a reference to the modifier state component.
    #[inline]
    pub fn modifiers(&self) -> &ModifierState {
        &self.modifiers
    }

    /// Get a reference to the pending state component.
    #[inline]
    pub fn pending(&self) -> &PendingState {
        &self.pending
    }

    /// Get a mutable reference to the key state component.
    ///
    /// # Safety
    ///
    /// Direct mutations bypass version tracking and change events.
    /// Prefer using the mutation API (apply/apply_batch) instead.
    #[inline]
    pub fn keys_mut(&mut self) -> &mut KeyState {
        &mut self.keys
    }

    /// Get a mutable reference to the layer state component.
    ///
    /// # Safety
    ///
    /// Direct mutations bypass version tracking and change events.
    /// Prefer using the mutation API (apply/apply_batch) instead.
    #[inline]
    pub fn layers_mut(&mut self) -> &mut LayerState {
        &mut self.layers
    }

    /// Get a mutable reference to the modifier state component.
    ///
    /// # Safety
    ///
    /// Direct mutations bypass version tracking and change events.
    /// Prefer using the mutation API (apply/apply_batch) instead.
    #[inline]
    pub fn modifiers_mut(&mut self) -> &mut ModifierState {
        &mut self.modifiers
    }

    /// Get a mutable reference to the pending state component.
    ///
    /// # Safety
    ///
    /// Direct mutations bypass version tracking and change events.
    /// Prefer using the mutation API (apply/apply_batch) instead.
    #[inline]
    pub fn pending_mut(&mut self) -> &mut PendingState {
        &mut self.pending
    }
}

impl Default for EngineState {
    fn default() -> Self {
        Self::new(TimingConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_engine_state_has_defaults() {
        let state = EngineState::new(TimingConfig::default());
        assert_eq!(state.version(), 0);
        assert!(state.no_keys_pressed());
        assert!(state.only_base_layer_active());
        assert!(state.no_pending_decisions());
        assert_eq!(state.base_layer(), 0);
    }

    #[test]
    fn with_base_layer_sets_base() {
        let state = EngineState::with_base_layer(5, TimingConfig::default());
        assert_eq!(state.base_layer(), 5);
        assert_eq!(state.top_layer(), 5);
    }

    #[test]
    fn default_creates_valid_state() {
        let state = EngineState::default();
        assert_eq!(state.version(), 0);
        assert!(state.no_keys_pressed());
    }

    #[test]
    fn key_queries() {
        let state = EngineState::default();
        assert!(!state.is_key_pressed(KeyCode::A));
        assert_eq!(state.key_press_time(KeyCode::A), None);
        assert_eq!(state.pressed_key_count(), 0);
        assert!(state.no_keys_pressed());
    }

    #[test]
    fn layer_queries() {
        let state = EngineState::default();
        assert_eq!(state.active_layers(), &[0]);
        assert_eq!(state.top_layer(), 0);
        assert_eq!(state.base_layer(), 0);
        assert!(state.is_layer_active(0));
        assert!(!state.is_layer_active(1));
        assert_eq!(state.active_layer_count(), 1);
        assert!(state.only_base_layer_active());
    }

    #[test]
    fn modifier_queries() {
        let state = EngineState::default();
        assert!(!state.is_modifier_active(Modifier::Standard(StandardModifier::Shift)));
        assert!(!state.is_modifier_active(Modifier::Virtual(0)));
    }

    #[test]
    fn pending_queries() {
        let state = EngineState::default();
        assert_eq!(state.pending_count(), 0);
        assert!(state.no_pending_decisions());
    }

    #[test]
    fn component_access() {
        let mut state = EngineState::default();

        // Immutable access
        let _keys = state.keys();
        let _layers = state.layers();
        let _modifiers = state.modifiers();
        let _pending = state.pending();

        // Mutable access
        let _keys_mut = state.keys_mut();
        let _layers_mut = state.layers_mut();
        let _modifiers_mut = state.modifiers_mut();
        let _pending_mut = state.pending_mut();
    }

    #[test]
    fn state_is_cloneable() {
        let state = EngineState::default();
        let cloned = state.clone();
        assert_eq!(state.version(), cloned.version());
        assert_eq!(state.base_layer(), cloned.base_layer());
    }
}
