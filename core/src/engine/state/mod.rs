//! Layer and modifier state management.

pub mod change;
pub mod delta;
mod error;
pub mod history;
mod key_state;
mod keys;
mod layers;
mod modifiers;
mod mutation;
mod pending;
pub mod persistence;
pub mod snapshot;
pub mod tracker;

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
#[cfg(not(debug_assertions))]
use tracing as log;

use crate::engine::decision::timing::TimingConfig;
use crate::engine::KeyCode;

#[allow(unused_imports)] // Will be used in tasks 9-11 for apply() and apply_batch()
pub use change::{
    AutoReleaseReason, Effect, HoldOutcome, PendingDecisionType, PendingResolution, StateChange,
};
pub use delta::{DeltaChange, StateDelta};
#[allow(unused_imports)] // Will be used in tasks 9-11 for apply() and apply_batch()
pub use error::{StateError, StateResult};
#[allow(unused_imports)] // Will be used in tasks 14+ for state persistence and debugging
pub use history::{HistoryConfig, StateHistory};
pub use key_state::KeyStateTracker;
pub use keys::KeyState;
pub use layers::LayerState;
pub use layers::{HoldAction, Layer, LayerAction, LayerId, LayerStack};
pub use modifiers::{
    Modifier, ModifierState, StandardModifier, StandardModifiers, VirtualModifiers,
};
pub use mutation::Mutation;
pub use pending::PendingState;
pub use tracker::DeltaTracker;

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

    // === Synchronization and Validation ===

    /// Validate state invariants after a mutation.
    ///
    /// This method checks that the state remains consistent after mutations.
    /// In debug builds, invariant violations panic. In release builds, they are logged.
    ///
    /// # Invariants Checked
    ///
    /// 1. Base layer is always active
    /// 2. No duplicate layers in the stack
    /// 3. Version counter never decreases
    fn validate_invariants(&self) {
        // Invariant 1: Base layer must always be active
        debug_assert!(
            self.layers
                .active_layers()
                .contains(&self.layers.base_layer()),
            "State invariant violated: base layer not active"
        );

        // Invariant 2: No duplicate layers (this is checked by LayerState internally)
        debug_assert!(
            self.layers.active_layers().len() == self.layers.len(),
            "State invariant violated: layer count mismatch"
        );

        // In release builds, log errors instead of panicking
        #[cfg(not(debug_assertions))]
        {
            if !self
                .layers
                .active_layers()
                .contains(&self.layers.base_layer())
            {
                log::error!("State invariant violated: base layer not active");
            }
            if self.layers.active_layers().len() != self.layers.len() {
                log::error!("State invariant violated: layer count mismatch");
            }
        }
    }

    /// Synchronize state components after a layer change.
    ///
    /// When layers change, pending decisions that depend on layer-specific
    /// mappings may no longer be valid. This method clears pending decisions
    /// to maintain consistency.
    ///
    /// # Effects
    ///
    /// Returns effects for cleared pending decisions.
    fn sync_on_layer_change(&mut self) -> Vec<Effect> {
        let mut effects = Vec::new();

        // Clear pending decisions that may be invalidated by layer change
        // Note: In a future enhancement, we could be more selective about which
        // pending decisions to clear based on the specific layer that changed
        let cleared_count = self.pending.clear();

        if cleared_count > 0 {
            effects.push(Effect::PendingCleared {
                count: cleared_count,
            });
        }

        effects
    }

    // === Mutation Methods ===

    /// Apply multiple mutations atomically as a batch.
    ///
    /// This method applies a sequence of mutations with full rollback semantics:
    /// - All mutations are applied in order
    /// - If any mutation fails, the entire batch is rolled back
    /// - The state is left unchanged on failure
    /// - Version increments only if the entire batch succeeds
    ///
    /// # Arguments
    ///
    /// * `mutations` - Vector of mutations to apply atomically
    ///
    /// # Returns
    ///
    /// A vector of StateChange events (one per mutation) on success,
    /// or a BatchFailed error on failure with the index of the failing mutation.
    ///
    /// # Errors
    ///
    /// * `StateError::EmptyBatch` - If the mutations vector is empty
    /// * `StateError::NestedBatch` - If any mutation in the batch is itself a Batch
    /// * `StateError::BatchFailed` - If any mutation fails (state is rolled back)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use keyrx_core::engine::state::{EngineState, Mutation};
    /// use keyrx_core::engine::KeyCode;
    /// use keyrx_core::engine::decision::timing::TimingConfig;
    ///
    /// let mut state = EngineState::new(TimingConfig::default());
    /// let mutations = vec![
    ///     Mutation::KeyDown {
    ///         key: KeyCode::A,
    ///         timestamp_us: 1000,
    ///         is_repeat: false,
    ///     },
    ///     Mutation::PushLayer { layer_id: 1 },
    ///     Mutation::ActivateModifier { modifier_id: 5 },
    /// ];
    ///
    /// let changes = state.apply_batch(mutations).expect("valid batch");
    /// assert_eq!(changes.len(), 3);
    /// assert!(state.is_key_pressed(KeyCode::A));
    /// assert!(state.is_layer_active(1));
    /// ```
    pub fn apply_batch(&mut self, mutations: Vec<Mutation>) -> StateResult<Vec<StateChange>> {
        // Validate batch is not empty
        if mutations.is_empty() {
            return Err(StateError::EmptyBatch);
        }

        // Validate no nested batches
        for mutation in &mutations {
            if matches!(mutation, Mutation::Batch { .. }) {
                return Err(StateError::NestedBatch);
            }
        }

        // Clone current state for rollback
        let backup = self.clone();

        // Apply mutations in sequence, collecting changes
        let mut changes = Vec::with_capacity(mutations.len());

        for (index, mutation) in mutations.into_iter().enumerate() {
            match self.apply(mutation) {
                Ok(change) => changes.push(change),
                Err(error) => {
                    // Rollback to backup state
                    *self = backup;
                    return Err(StateError::BatchFailed {
                        index,
                        error: Box::new(error),
                    });
                }
            }
        }

        Ok(changes)
    }

    /// Apply a single mutation atomically.
    ///
    /// This is the primary way to mutate engine state. Each mutation:
    /// - Updates the relevant state component
    /// - Increments the state version
    /// - Produces a StateChange event with effects
    /// - Ensures state invariants are maintained
    ///
    /// # Arguments
    ///
    /// * `mutation` - The mutation to apply
    ///
    /// # Returns
    ///
    /// A StateChange recording the mutation and any secondary effects,
    /// or a StateError if the mutation is invalid.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use keyrx_core::engine::state::{EngineState, Mutation};
    /// use keyrx_core::engine::KeyCode;
    /// use keyrx_core::engine::decision::timing::TimingConfig;
    ///
    /// let mut state = EngineState::new(TimingConfig::default());
    /// let mutation = Mutation::KeyDown {
    ///     key: KeyCode::A,
    ///     timestamp_us: 1000,
    ///     is_repeat: false,
    /// };
    ///
    /// let change = state.apply(mutation).expect("valid mutation");
    /// assert_eq!(change.version, 1);
    /// assert!(state.is_key_pressed(KeyCode::A));
    /// ```
    pub fn apply(&mut self, mutation: Mutation) -> StateResult<StateChange> {
        // Get timestamp from mutation or use 0 for mutations without timestamps
        let timestamp_us = mutation.timestamp().unwrap_or(0);

        // Create change that will be returned
        let mut effects = Vec::new();

        // Apply the mutation to the appropriate component(s)
        match &mutation {
            // === Key State Mutations ===
            Mutation::KeyDown {
                key,
                timestamp_us: ts,
                is_repeat,
            } => {
                let changed = self.keys.press(*key, *ts, *is_repeat);
                if !changed && !is_repeat {
                    return Err(StateError::KeyAlreadyPressed { key: *key });
                }
            }

            Mutation::KeyUp { key, .. } => {
                let press_time = self.keys.release(*key);
                if press_time.is_none() {
                    return Err(StateError::KeyNotPressed { key: *key });
                }
                // Note: Modifier synchronization for key releases is handled by the
                // engine's key processing logic, not at the state mutation level.
                // The engine knows which keys are bound to modifiers and can issue
                // explicit DeactivateModifier mutations when needed.
            }

            // === Layer State Mutations ===
            Mutation::PushLayer { layer_id } => {
                self.layers.push(*layer_id);
                // Synchronize state after layer change
                let sync_effects = self.sync_on_layer_change();
                effects.extend(sync_effects);
            }

            Mutation::PopLayer => {
                let popped = self.layers.pop();
                if let Some(layer_id) = popped {
                    effects.push(Effect::LayerPopped { layer_id });
                    // Synchronize state after layer change
                    let sync_effects = self.sync_on_layer_change();
                    effects.extend(sync_effects);
                } else {
                    return Err(StateError::CannotPopBaseLayer);
                }
            }

            Mutation::ToggleLayer { layer_id } => {
                self.layers.toggle(*layer_id);
                // Synchronize state after layer change
                let sync_effects = self.sync_on_layer_change();
                effects.extend(sync_effects);
            }

            // === Modifier State Mutations ===
            Mutation::ActivateModifier { modifier_id } => {
                // Validate modifier ID (255 is reserved)
                if *modifier_id == 255 {
                    return Err(StateError::InvalidModifierId {
                        modifier_id: *modifier_id,
                    });
                }
                let modifier = Modifier::Virtual(*modifier_id);
                self.modifiers.activate(modifier);
            }

            Mutation::DeactivateModifier { modifier_id } => {
                // Validate modifier ID
                if *modifier_id == 255 {
                    return Err(StateError::InvalidModifierId {
                        modifier_id: *modifier_id,
                    });
                }
                let modifier = Modifier::Virtual(*modifier_id);
                self.modifiers.deactivate(modifier);
                effects.push(Effect::ModifierDeactivated {
                    modifier_id: *modifier_id,
                });
            }

            Mutation::ArmOneShotModifier { modifier_id } => {
                // Validate modifier ID
                if *modifier_id == 255 {
                    return Err(StateError::InvalidModifierId {
                        modifier_id: *modifier_id,
                    });
                }
                let modifier = Modifier::Virtual(*modifier_id);
                self.modifiers.arm_one_shot(modifier);
            }

            Mutation::ClearModifiers => {
                self.modifiers.clear();
                effects.push(Effect::AllModifiersCleared);
            }

            // === Pending Decision Mutations ===
            Mutation::AddTapHold {
                key,
                pressed_at,
                tap_action,
                hold_action,
            } => {
                let (added, eager_resolution) =
                    self.pending
                        .add_tap_hold(*key, *pressed_at, *tap_action, hold_action.clone());

                if !added {
                    return Err(StateError::PendingQueueFull {
                        max_size: PendingState::MAX_PENDING,
                    });
                }

                // Handle eager resolution if configured
                if let Some(resolution) = eager_resolution {
                    effects.push(Effect::PendingResolved {
                        decision_type: PendingDecisionType::TapHold,
                        resolution: match resolution {
                            crate::engine::decision::pending::DecisionResolution::Tap {
                                ..
                            } => PendingResolution::Tap,
                            crate::engine::decision::pending::DecisionResolution::Hold {
                                ..
                            } => PendingResolution::Hold,
                            _ => PendingResolution::Cancelled,
                        },
                    });
                }
            }

            Mutation::AddCombo {
                keys,
                started_at,
                action,
            } => {
                let added = self.pending.add_combo(keys, *started_at, action.clone());
                if !added {
                    return Err(StateError::PendingQueueFull {
                        max_size: PendingState::MAX_PENDING,
                    });
                }
            }

            Mutation::MarkInterrupted { by_key } => {
                self.pending.mark_interrupted(*by_key);
                // Count is tracked via internal queue state, we approximate here
                // A more accurate count would require changes to PendingState API
                effects.push(Effect::PendingInterrupted { count: 1 });
            }

            Mutation::ClearPending => {
                let count = self.pending.clear();
                effects.push(Effect::PendingCleared { count });
            }

            // === Batch Mutations ===
            Mutation::Batch { .. } => {
                // Batch mutations are handled by apply_batch(), not apply()
                return Err(StateError::NestedBatch);
            }
        }

        // Increment version
        self.version += 1;

        // Validate state invariants after mutation
        self.validate_invariants();

        // Return the state change with effects
        Ok(StateChange::with_effects(
            mutation,
            self.version,
            timestamp_us,
            effects,
        ))
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

    // === Mutation Tests ===

    #[test]
    fn apply_key_down_success() {
        let mut state = EngineState::default();
        let mutation = Mutation::KeyDown {
            key: KeyCode::A,
            timestamp_us: 1000,
            is_repeat: false,
        };

        let change = state.apply(mutation.clone()).expect("valid mutation");
        assert_eq!(change.version, 1);
        assert_eq!(change.timestamp_us, 1000);
        assert_eq!(change.mutation, mutation);
        assert!(state.is_key_pressed(KeyCode::A));
        assert_eq!(state.version(), 1);
    }

    #[test]
    fn apply_key_down_already_pressed() {
        let mut state = EngineState::default();
        state.keys_mut().press(KeyCode::A, 1000, false);

        let mutation = Mutation::KeyDown {
            key: KeyCode::A,
            timestamp_us: 2000,
            is_repeat: false,
        };

        let result = state.apply(mutation);
        assert!(matches!(
            result,
            Err(StateError::KeyAlreadyPressed { key: KeyCode::A })
        ));
    }

    #[test]
    fn apply_key_up_success() {
        let mut state = EngineState::default();
        state.keys_mut().press(KeyCode::A, 1000, false);

        let mutation = Mutation::KeyUp {
            key: KeyCode::A,
            timestamp_us: 2000,
        };

        let change = state.apply(mutation).expect("valid mutation");
        assert_eq!(change.version, 1);
        assert!(!state.is_key_pressed(KeyCode::A));
    }

    #[test]
    fn apply_key_up_not_pressed() {
        let mut state = EngineState::default();
        let mutation = Mutation::KeyUp {
            key: KeyCode::A,
            timestamp_us: 1000,
        };

        let result = state.apply(mutation);
        assert!(matches!(
            result,
            Err(StateError::KeyNotPressed { key: KeyCode::A })
        ));
    }

    #[test]
    fn apply_push_layer() {
        let mut state = EngineState::default();
        let mutation = Mutation::PushLayer { layer_id: 1 };

        let change = state.apply(mutation).expect("valid mutation");
        assert_eq!(change.version, 1);
        assert!(state.is_layer_active(1));
        assert_eq!(state.top_layer(), 1);
    }

    #[test]
    fn apply_pop_layer_success() {
        let mut state = EngineState::default();
        state.layers_mut().push(1);

        let mutation = Mutation::PopLayer;
        let change = state.apply(mutation).expect("valid mutation");
        assert_eq!(change.version, 1);
        assert_eq!(change.effects.len(), 1);
        assert!(matches!(
            change.effects[0],
            Effect::LayerPopped { layer_id: 1 }
        ));
        assert!(!state.is_layer_active(1));
    }

    #[test]
    fn apply_pop_layer_base_only() {
        let mut state = EngineState::default();
        let mutation = Mutation::PopLayer;

        let result = state.apply(mutation);
        assert!(matches!(result, Err(StateError::CannotPopBaseLayer)));
    }

    #[test]
    fn apply_toggle_layer() {
        let mut state = EngineState::default();

        // Toggle on
        let mutation = Mutation::ToggleLayer { layer_id: 1 };
        state.apply(mutation).expect("valid mutation");
        assert!(state.is_layer_active(1));

        // Toggle off
        let mutation = Mutation::ToggleLayer { layer_id: 1 };
        state.apply(mutation).expect("valid mutation");
        assert!(!state.is_layer_active(1));
    }

    #[test]
    fn apply_activate_modifier() {
        let mut state = EngineState::default();
        let mutation = Mutation::ActivateModifier { modifier_id: 5 };

        let change = state.apply(mutation).expect("valid mutation");
        assert_eq!(change.version, 1);
        assert!(state.is_modifier_active(Modifier::Virtual(5)));
    }

    #[test]
    fn apply_activate_modifier_invalid_id() {
        let mut state = EngineState::default();
        let mutation = Mutation::ActivateModifier { modifier_id: 255 };

        let result = state.apply(mutation);
        assert!(matches!(
            result,
            Err(StateError::InvalidModifierId { modifier_id: 255 })
        ));
    }

    #[test]
    fn apply_deactivate_modifier() {
        let mut state = EngineState::default();
        state.modifiers_mut().activate(Modifier::Virtual(5));

        let mutation = Mutation::DeactivateModifier { modifier_id: 5 };
        let change = state.apply(mutation).expect("valid mutation");
        assert_eq!(change.version, 1);
        assert_eq!(change.effects.len(), 1);
        assert!(matches!(
            change.effects[0],
            Effect::ModifierDeactivated { modifier_id: 5 }
        ));
        assert!(!state.is_modifier_active(Modifier::Virtual(5)));
    }

    #[test]
    fn apply_arm_one_shot_modifier() {
        let mut state = EngineState::default();
        let mutation = Mutation::ArmOneShotModifier { modifier_id: 3 };

        let change = state.apply(mutation).expect("valid mutation");
        assert_eq!(change.version, 1);
        assert!(state.is_modifier_active(Modifier::Virtual(3)));
    }

    #[test]
    fn apply_clear_modifiers() {
        let mut state = EngineState::default();
        state.modifiers_mut().activate(Modifier::Virtual(1));
        state.modifiers_mut().activate(Modifier::Virtual(2));

        let mutation = Mutation::ClearModifiers;
        let change = state.apply(mutation).expect("valid mutation");
        assert_eq!(change.version, 1);
        assert_eq!(change.effects.len(), 1);
        assert!(matches!(change.effects[0], Effect::AllModifiersCleared));
        assert!(!state.is_modifier_active(Modifier::Virtual(1)));
        assert!(!state.is_modifier_active(Modifier::Virtual(2)));
    }

    #[test]
    fn apply_add_tap_hold() {
        let mut state = EngineState::default();
        let mutation = Mutation::AddTapHold {
            key: KeyCode::A,
            pressed_at: 1000,
            tap_action: KeyCode::B,
            hold_action: HoldAction::Key(KeyCode::C),
        };

        let change = state.apply(mutation).expect("valid mutation");
        assert_eq!(change.version, 1);
        assert_eq!(state.pending_count(), 1);
    }

    #[test]
    fn apply_add_combo() {
        let mut state = EngineState::default();
        let mutation = Mutation::AddCombo {
            keys: vec![KeyCode::A, KeyCode::B],
            started_at: 1000,
            action: LayerAction::LayerPush(1),
        };

        let change = state.apply(mutation).expect("valid mutation");
        assert_eq!(change.version, 1);
        assert_eq!(state.pending_count(), 1);
    }

    #[test]
    fn apply_mark_interrupted() {
        let mut state = EngineState::default();
        // Add a pending decision first
        state
            .pending_mut()
            .add_tap_hold(KeyCode::A, 1000, KeyCode::B, HoldAction::Key(KeyCode::C));

        let mutation = Mutation::MarkInterrupted { by_key: KeyCode::B };
        let change = state.apply(mutation).expect("valid mutation");
        assert_eq!(change.version, 1);
        assert_eq!(change.effects.len(), 1);
        assert!(matches!(
            change.effects[0],
            Effect::PendingInterrupted { .. }
        ));
    }

    #[test]
    fn apply_clear_pending() {
        let mut state = EngineState::default();
        // Add some pending decisions
        state
            .pending_mut()
            .add_tap_hold(KeyCode::A, 1000, KeyCode::B, HoldAction::Key(KeyCode::C));
        state
            .pending_mut()
            .add_tap_hold(KeyCode::D, 1000, KeyCode::E, HoldAction::Key(KeyCode::F));

        let mutation = Mutation::ClearPending;
        let change = state.apply(mutation).expect("valid mutation");
        assert_eq!(change.version, 1);
        assert_eq!(change.effects.len(), 1);
        assert!(matches!(
            change.effects[0],
            Effect::PendingCleared { count: 2 }
        ));
        assert_eq!(state.pending_count(), 0);
    }

    #[test]
    fn apply_batch_returns_error() {
        let mut state = EngineState::default();
        let mutation = Mutation::Batch {
            mutations: vec![Mutation::KeyDown {
                key: KeyCode::A,
                timestamp_us: 1000,
                is_repeat: false,
            }],
        };

        let result = state.apply(mutation);
        assert!(matches!(result, Err(StateError::NestedBatch)));
    }

    #[test]
    fn apply_increments_version() {
        let mut state = EngineState::default();
        assert_eq!(state.version(), 0);

        state
            .apply(Mutation::KeyDown {
                key: KeyCode::A,
                timestamp_us: 1000,
                is_repeat: false,
            })
            .unwrap();
        assert_eq!(state.version(), 1);

        state
            .apply(Mutation::KeyUp {
                key: KeyCode::A,
                timestamp_us: 2000,
            })
            .unwrap();
        assert_eq!(state.version(), 2);

        state.apply(Mutation::PushLayer { layer_id: 1 }).unwrap();
        assert_eq!(state.version(), 3);
    }

    // === Batch Mutation Tests ===

    #[test]
    fn apply_batch_success() {
        let mut state = EngineState::default();
        let mutations = vec![
            Mutation::KeyDown {
                key: KeyCode::A,
                timestamp_us: 1000,
                is_repeat: false,
            },
            Mutation::PushLayer { layer_id: 1 },
            Mutation::ActivateModifier { modifier_id: 5 },
        ];

        let changes = state.apply_batch(mutations).expect("valid batch");
        assert_eq!(changes.len(), 3);

        // Verify all mutations were applied
        assert!(state.is_key_pressed(KeyCode::A));
        assert!(state.is_layer_active(1));
        assert!(state.is_modifier_active(Modifier::Virtual(5)));

        // Verify version incremented for each mutation
        assert_eq!(state.version(), 3);

        // Verify each change has correct version
        assert_eq!(changes[0].version, 1);
        assert_eq!(changes[1].version, 2);
        assert_eq!(changes[2].version, 3);
    }

    #[test]
    fn apply_batch_empty_error() {
        let mut state = EngineState::default();
        let mutations = vec![];

        let result = state.apply_batch(mutations);
        assert!(matches!(result, Err(StateError::EmptyBatch)));
    }

    #[test]
    fn apply_batch_nested_batch_error() {
        let mut state = EngineState::default();
        let mutations = vec![
            Mutation::KeyDown {
                key: KeyCode::A,
                timestamp_us: 1000,
                is_repeat: false,
            },
            Mutation::Batch {
                mutations: vec![Mutation::PushLayer { layer_id: 1 }],
            },
        ];

        let result = state.apply_batch(mutations);
        assert!(matches!(result, Err(StateError::NestedBatch)));
    }

    #[test]
    fn apply_batch_rollback_on_failure() {
        let mut state = EngineState::default();
        let initial_version = state.version();

        let mutations = vec![
            Mutation::KeyDown {
                key: KeyCode::A,
                timestamp_us: 1000,
                is_repeat: false,
            },
            Mutation::PushLayer { layer_id: 1 },
            // This will fail because key A is not pressed yet at batch start
            Mutation::KeyUp {
                key: KeyCode::B,
                timestamp_us: 2000,
            },
        ];

        let result = state.apply_batch(mutations);

        // Verify batch failed at the correct index
        assert!(matches!(
            result,
            Err(StateError::BatchFailed { index: 2, .. })
        ));

        // Verify complete rollback - no state changes should persist
        assert!(!state.is_key_pressed(KeyCode::A));
        assert!(!state.is_layer_active(1));
        assert_eq!(state.version(), initial_version);
    }

    #[test]
    fn apply_batch_rollback_preserves_previous_state() {
        let mut state = EngineState::default();

        // Apply some initial state
        state
            .apply(Mutation::KeyDown {
                key: KeyCode::Z,
                timestamp_us: 500,
                is_repeat: false,
            })
            .unwrap();
        state.apply(Mutation::PushLayer { layer_id: 9 }).unwrap();
        let version_before_batch = state.version();

        // Try a batch that will fail
        let mutations = vec![
            Mutation::KeyDown {
                key: KeyCode::A,
                timestamp_us: 1000,
                is_repeat: false,
            },
            Mutation::PopLayer, // Will pop layer 9
            Mutation::PopLayer, // Will fail - can't pop base layer
        ];

        let result = state.apply_batch(mutations);
        assert!(matches!(
            result,
            Err(StateError::BatchFailed { index: 2, .. })
        ));

        // Verify rollback preserved the state before batch
        assert!(state.is_key_pressed(KeyCode::Z));
        assert!(!state.is_key_pressed(KeyCode::A));
        assert!(state.is_layer_active(9));
        assert_eq!(state.version(), version_before_batch);
    }

    #[test]
    fn apply_batch_complex_sequence() {
        let mut state = EngineState::default();

        let mutations = vec![
            Mutation::KeyDown {
                key: KeyCode::A,
                timestamp_us: 1000,
                is_repeat: false,
            },
            Mutation::KeyDown {
                key: KeyCode::B,
                timestamp_us: 1100,
                is_repeat: false,
            },
            Mutation::PushLayer { layer_id: 1 },
            Mutation::PushLayer { layer_id: 2 },
            Mutation::ActivateModifier { modifier_id: 10 },
            Mutation::ActivateModifier { modifier_id: 20 },
            Mutation::AddTapHold {
                key: KeyCode::C,
                pressed_at: 1200,
                tap_action: KeyCode::D,
                hold_action: HoldAction::Key(KeyCode::E),
            },
            Mutation::KeyUp {
                key: KeyCode::A,
                timestamp_us: 1300,
            },
            Mutation::PopLayer, // Pop layer 2
            Mutation::DeactivateModifier { modifier_id: 10 },
        ];

        let changes = state.apply_batch(mutations).expect("valid complex batch");
        assert_eq!(changes.len(), 10);

        // Verify final state
        assert!(!state.is_key_pressed(KeyCode::A));
        assert!(state.is_key_pressed(KeyCode::B));
        assert_eq!(state.top_layer(), 1);
        assert!(!state.is_layer_active(2));
        assert!(!state.is_modifier_active(Modifier::Virtual(10)));
        assert!(state.is_modifier_active(Modifier::Virtual(20)));
        // Pending decisions were cleared by the PopLayer operation (synchronization)
        assert_eq!(state.pending_count(), 0);
        assert_eq!(state.version(), 10);
    }

    #[test]
    fn apply_batch_single_mutation() {
        let mut state = EngineState::default();
        let mutations = vec![Mutation::KeyDown {
            key: KeyCode::A,
            timestamp_us: 1000,
            is_repeat: false,
        }];

        let changes = state
            .apply_batch(mutations)
            .expect("valid single mutation batch");
        assert_eq!(changes.len(), 1);
        assert!(state.is_key_pressed(KeyCode::A));
        assert_eq!(state.version(), 1);
    }

    #[test]
    fn apply_batch_error_details() {
        let mut state = EngineState::default();
        let mutations = vec![
            Mutation::ActivateModifier { modifier_id: 1 },
            Mutation::ActivateModifier { modifier_id: 2 },
            Mutation::ActivateModifier { modifier_id: 255 }, // Invalid ID
        ];

        let result = state.apply_batch(mutations);

        match result {
            Err(StateError::BatchFailed { index, error }) => {
                assert_eq!(index, 2);
                assert!(matches!(
                    *error,
                    StateError::InvalidModifierId { modifier_id: 255 }
                ));
            }
            _ => panic!("Expected BatchFailed error"),
        }

        // Verify no state changes persisted
        assert!(!state.is_modifier_active(Modifier::Virtual(1)));
        assert!(!state.is_modifier_active(Modifier::Virtual(2)));
    }

    // === Synchronization Tests ===

    #[test]
    fn sync_on_layer_change_clears_pending() {
        let mut state = EngineState::default();

        // Add some pending decisions
        state
            .pending_mut()
            .add_tap_hold(KeyCode::A, 1000, KeyCode::B, HoldAction::Key(KeyCode::C));
        state
            .pending_mut()
            .add_tap_hold(KeyCode::D, 1100, KeyCode::E, HoldAction::Key(KeyCode::F));
        assert_eq!(state.pending_count(), 2);

        // Push a layer, which should clear pending decisions
        let change = state.apply(Mutation::PushLayer { layer_id: 1 }).unwrap();

        // Verify pending was cleared
        assert_eq!(state.pending_count(), 0);

        // Verify effect was recorded
        assert!(
            change
                .effects
                .iter()
                .any(|e| matches!(e, Effect::PendingCleared { count: 2 })),
            "Expected PendingCleared effect, got: {:?}",
            change.effects
        );
    }

    #[test]
    fn sync_on_pop_layer_clears_pending() {
        let mut state = EngineState::default();
        state.layers_mut().push(1);

        // Add a pending decision
        state
            .pending_mut()
            .add_combo(&[KeyCode::A, KeyCode::B], 1000, LayerAction::LayerPush(2));
        assert_eq!(state.pending_count(), 1);

        // Pop the layer
        let change = state.apply(Mutation::PopLayer).unwrap();

        // Verify pending was cleared
        assert_eq!(state.pending_count(), 0);

        // Verify effects include both LayerPopped and PendingCleared
        assert!(change
            .effects
            .iter()
            .any(|e| matches!(e, Effect::LayerPopped { .. })));
        assert!(change
            .effects
            .iter()
            .any(|e| matches!(e, Effect::PendingCleared { count: 1 })));
    }

    #[test]
    fn sync_on_toggle_layer_clears_pending() {
        let mut state = EngineState::default();

        // Add a pending decision
        state.pending_mut().add_tap_hold(
            KeyCode::Space,
            1000,
            KeyCode::Space,
            HoldAction::Layer(1),
        );

        // Toggle a layer
        let change = state.apply(Mutation::ToggleLayer { layer_id: 1 }).unwrap();

        // Verify pending was cleared
        assert_eq!(state.pending_count(), 0);
        assert!(change
            .effects
            .iter()
            .any(|e| matches!(e, Effect::PendingCleared { .. })));
    }

    #[test]
    fn sync_layer_change_no_pending_no_effect() {
        let mut state = EngineState::default();

        // Push a layer without any pending decisions
        let change = state.apply(Mutation::PushLayer { layer_id: 1 }).unwrap();

        // Verify no PendingCleared effect since there were no pending decisions
        assert!(!change
            .effects
            .iter()
            .any(|e| matches!(e, Effect::PendingCleared { .. })));
    }

    #[test]
    fn validate_invariants_base_layer_always_active() {
        let state = EngineState::default();
        // This should not panic
        state.validate_invariants();

        let mut state = EngineState::with_base_layer(5, TimingConfig::default());
        state.layers_mut().push(1);
        state.layers_mut().push(2);
        // Base layer 5 should still be in active layers
        state.validate_invariants();
    }

    #[test]
    fn version_increments_with_each_mutation() {
        let mut state = EngineState::default();
        assert_eq!(state.version(), 0);

        state.apply(Mutation::PushLayer { layer_id: 1 }).unwrap();
        assert_eq!(state.version(), 1);

        state
            .apply(Mutation::ActivateModifier { modifier_id: 5 })
            .unwrap();
        assert_eq!(state.version(), 2);

        state.apply(Mutation::PopLayer).unwrap();
        assert_eq!(state.version(), 3);
    }
}
