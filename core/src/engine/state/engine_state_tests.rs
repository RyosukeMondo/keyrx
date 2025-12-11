//! Tests for EngineState - unified engine state container.
//!
//! These tests verify the EngineState struct's functionality including:
//! - Default construction and initialization
//! - Component queries (keys, layers, modifiers, pending)
//! - Mutation application (single and batch)
//! - State synchronization on layer changes
//! - Invariant validation

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

#[test]
fn delta_tracker_records_key_changes() {
    let mut state = EngineState::default();

    state
        .apply(Mutation::KeyDown {
            key: KeyCode::A,
            timestamp_us: 1000,
            is_repeat: false,
        })
        .expect("valid mutation");

    let delta = state.take_delta();
    assert_eq!(delta.from_version, 0);
    assert_eq!(delta.to_version, 1);
    assert_eq!(delta.changes, vec![DeltaChange::KeyPressed(KeyCode::A)]);
    assert_eq!(state.delta_version(), state.version());
}

#[test]
fn delta_tracker_tracks_layer_toggles() {
    let mut state = EngineState::default();

    state
        .apply(Mutation::ToggleLayer { layer_id: 1 })
        .expect("valid mutation");
    let activated = state.take_delta();
    assert_eq!(activated.changes, vec![DeltaChange::LayerActivated(1)]);
    assert_eq!(activated.from_version, 0);
    assert_eq!(activated.to_version, 1);

    state
        .apply(Mutation::ToggleLayer { layer_id: 1 })
        .expect("valid mutation");
    let deactivated = state.take_delta();
    assert_eq!(deactivated.changes, vec![DeltaChange::LayerDeactivated(1)]);
    assert_eq!(deactivated.from_version, 1);
    assert_eq!(deactivated.to_version, 2);
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
    state
        .pending_mut()
        .add_tap_hold(KeyCode::Space, 1000, KeyCode::Space, HoldAction::Layer(1));

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
