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
    /// Tracker for incremental deltas based on state mutations.
    delta_tracker: DeltaTracker,
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
            delta_tracker: DeltaTracker::new(),
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
            delta_tracker: DeltaTracker::new(),
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

    /// Get the current delta version (mirrors the state version).
    #[inline]
    pub fn delta_version(&self) -> u64 {
        self.delta_tracker.current_version()
    }

    /// Take the accumulated state delta and clear recorded changes.
    ///
    /// This is used by higher layers (e.g., FFI) to stream incremental
    /// updates instead of full snapshots.
    pub fn take_delta(&self) -> StateDelta {
        self.delta_tracker.take_delta()
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
        let mut delta_change: Option<DeltaChange> = None;

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
                delta_change = Some(DeltaChange::KeyPressed(*key));
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
                delta_change = Some(DeltaChange::KeyReleased(*key));
            }

            // === Layer State Mutations ===
            Mutation::PushLayer { layer_id } => {
                self.layers.push(*layer_id);
                // Synchronize state after layer change
                let sync_effects = self.sync_on_layer_change();
                effects.extend(sync_effects);
                delta_change = Some(DeltaChange::LayerActivated(*layer_id));
            }

            Mutation::PopLayer => {
                let popped = self.layers.pop();
                if let Some(layer_id) = popped {
                    effects.push(Effect::LayerPopped { layer_id });
                    // Synchronize state after layer change
                    let sync_effects = self.sync_on_layer_change();
                    effects.extend(sync_effects);
                    delta_change = Some(DeltaChange::LayerDeactivated(layer_id));
                } else {
                    return Err(StateError::CannotPopBaseLayer);
                }
            }

            Mutation::ToggleLayer { layer_id } => {
                self.layers.toggle(*layer_id);
                // Synchronize state after layer change
                let sync_effects = self.sync_on_layer_change();
                effects.extend(sync_effects);
                delta_change = Some(if self.layers.is_active(*layer_id) {
                    DeltaChange::LayerActivated(*layer_id)
                } else {
                    DeltaChange::LayerDeactivated(*layer_id)
                });
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
                delta_change = Some(DeltaChange::ModifierChanged {
                    id: *modifier_id,
                    active: true,
                });
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
                delta_change = Some(DeltaChange::ModifierChanged {
                    id: *modifier_id,
                    active: false,
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
                delta_change = Some(DeltaChange::AllModifiersCleared);
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

                // Pending decisions currently don't expose stable IDs for deltas.
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
                // Pending decisions currently don't expose stable IDs for deltas.
            }

            Mutation::MarkInterrupted { by_key } => {
                self.pending.mark_interrupted(*by_key);
                // Count is tracked via internal queue state, we approximate here
                // A more accurate count would require changes to PendingState API
                effects.push(Effect::PendingInterrupted { count: 1 });
                // Pending decisions currently don't expose stable IDs for deltas.
            }

            Mutation::ClearPending => {
                let count = self.pending.clear();
                effects.push(Effect::PendingCleared { count });
                delta_change = Some(DeltaChange::AllPendingCleared);
            }

            // === Batch Mutations ===
            Mutation::Batch { .. } => {
                // Batch mutations are handled by apply_batch(), not apply()
                return Err(StateError::NestedBatch);
            }
        }

        // Record delta and increment version counters
        let new_version = self.version.wrapping_add(1);
        let recorded_change = delta_change.unwrap_or(DeltaChange::VersionChanged {
            version: new_version,
        });

        self.delta_tracker.record(recorded_change);
        let tracker_version = self.delta_tracker.increment_version();
        debug_assert_eq!(tracker_version, new_version);
        self.version = tracker_version;

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
#[path = "engine_state_tests.rs"]
mod engine_state_tests;
