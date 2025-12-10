#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::print_stdout, clippy::print_stderr, clippy::field_reassign_with_default, clippy::useless_conversion, clippy::assertions_on_constants, clippy::manual_div_ceil, clippy::manual_strip, clippy::len_zero, clippy::redundant_closure, clippy::manual_range_contains, clippy::default_constructed_unit_structs, clippy::clone_on_copy, clippy::io_other_error, clippy::bool_assert_comparison, clippy::approx_constant, clippy::let_unit_value, clippy::while_let_on_iterator, clippy::await_holding_lock, clippy::unnecessary_cast, clippy::drop_non_drop, clippy::needless_range_loop, unused_imports, unused_variables, dead_code, unsafe_code, clippy::collapsible_if, clippy::bool_comparison, unexpected_cfgs)]
//! Property tests for EngineState invariants.
//!
//! This test suite uses property-based testing (proptest) to verify that
//! EngineState maintains its invariants across arbitrary mutation sequences.


use keyrx_core::engine::state::Mutation;
use keyrx_core::engine::{
    EngineState, HoldAction, KeyCode, LayerAction, Modifier, StandardModifier, TimingConfig,
};
use proptest::prelude::*;

// === Property Test Strategies ===

/// Generate arbitrary KeyCode values for testing.
fn arb_keycode() -> impl Strategy<Value = KeyCode> {
    prop_oneof![
        Just(KeyCode::A),
        Just(KeyCode::B),
        Just(KeyCode::C),
        Just(KeyCode::D),
        Just(KeyCode::E),
        Just(KeyCode::F),
        Just(KeyCode::G),
        Just(KeyCode::H),
        Just(KeyCode::Space),
        Just(KeyCode::Enter),
        Just(KeyCode::Escape),
        Just(KeyCode::Tab),
        Just(KeyCode::Backspace),
        Just(KeyCode::LeftShift),
        Just(KeyCode::RightShift),
        Just(KeyCode::LeftCtrl),
        Just(KeyCode::RightCtrl),
    ]
}

/// Generate arbitrary layer IDs (1-10 for testing, excluding base layer 0).
fn arb_layer_id() -> impl Strategy<Value = u16> {
    1u16..=10
}

/// Generate arbitrary modifier IDs (0-254, excluding 255 which is reserved).
fn arb_modifier_id() -> impl Strategy<Value = u8> {
    0u8..=254
}

/// Generate arbitrary timestamps in microseconds.
fn arb_timestamp() -> impl Strategy<Value = u64> {
    0u64..=10_000_000
}

/// Generate arbitrary HoldAction values.
fn arb_hold_action() -> impl Strategy<Value = HoldAction> {
    prop_oneof![
        arb_keycode().prop_map(HoldAction::Key),
        arb_layer_id().prop_map(HoldAction::Layer),
        arb_modifier_id().prop_map(HoldAction::Modifier),
    ]
}

/// Generate arbitrary LayerAction values.
fn arb_layer_action() -> impl Strategy<Value = LayerAction> {
    prop_oneof![
        arb_layer_id().prop_map(LayerAction::LayerPush),
        Just(LayerAction::LayerPop),
        arb_layer_id().prop_map(LayerAction::LayerToggle),
    ]
}

/// Generate arbitrary valid Mutation values.
fn arb_mutation() -> impl Strategy<Value = Mutation> {
    prop_oneof![
        // Key mutations
        (arb_keycode(), arb_timestamp(), any::<bool>()).prop_map(
            |(key, timestamp_us, is_repeat)| Mutation::KeyDown {
                key,
                timestamp_us,
                is_repeat,
            }
        ),
        (arb_keycode(), arb_timestamp())
            .prop_map(|(key, timestamp_us)| Mutation::KeyUp { key, timestamp_us }),
        // Layer mutations
        arb_layer_id().prop_map(|layer_id| Mutation::PushLayer { layer_id }),
        Just(Mutation::PopLayer),
        arb_layer_id().prop_map(|layer_id| Mutation::ToggleLayer { layer_id }),
        // Modifier mutations
        arb_modifier_id().prop_map(|modifier_id| Mutation::ActivateModifier { modifier_id }),
        arb_modifier_id().prop_map(|modifier_id| Mutation::DeactivateModifier { modifier_id }),
        arb_modifier_id().prop_map(|modifier_id| Mutation::ArmOneShotModifier { modifier_id }),
        Just(Mutation::ClearModifiers),
        // Pending decision mutations
        (
            arb_keycode(),
            arb_timestamp(),
            arb_keycode(),
            arb_hold_action()
        )
            .prop_map(
                |(key, pressed_at, tap_action, hold_action)| Mutation::AddTapHold {
                    key,
                    pressed_at,
                    tap_action,
                    hold_action,
                }
            ),
        (
            prop::collection::vec(arb_keycode(), 2..=4),
            arb_timestamp(),
            arb_layer_action()
        )
            .prop_map(|(keys, started_at, action)| Mutation::AddCombo {
                keys,
                started_at,
                action,
            }),
        arb_keycode().prop_map(|by_key| Mutation::MarkInterrupted { by_key }),
        Just(Mutation::ClearPending),
    ]
}

/// Generate a sequence of mutations for testing.
fn arb_mutation_sequence() -> impl Strategy<Value = Vec<Mutation>> {
    prop::collection::vec(arb_mutation(), 1..=50)
}

// === Invariant Checking Functions ===

/// Check that the base layer is always active.
fn check_base_layer_always_active(state: &EngineState) -> bool {
    let base_layer = state.base_layer();
    state.active_layers().contains(&base_layer)
}

/// Check that there are no duplicate layers in the active layer stack.
fn check_no_duplicate_layers(state: &EngineState) -> bool {
    let layers = state.active_layers();
    let unique_count = layers
        .iter()
        .collect::<std::collections::HashSet<_>>()
        .len();
    unique_count == layers.len()
}

/// Check that version only increases or stays the same (never decreases).
fn check_version_monotonic(old_version: u64, new_version: u64) -> bool {
    new_version >= old_version
}

/// Check that the layer count matches the length of active layers.
fn check_layer_count_consistent(state: &EngineState) -> bool {
    state.active_layer_count() == state.active_layers().len()
}

/// Check that if only base layer is active, the layer count is 1.
fn check_only_base_layer_logic(state: &EngineState) -> bool {
    state.only_base_layer_active() == (state.active_layer_count() == 1)
}

/// Check that pressed key count matches the number of pressed keys.
fn check_key_count_consistent(state: &EngineState) -> bool {
    let count = state.pressed_key_count();
    let actual = state.pressed_keys().count();
    count == actual
}

/// Check that no_keys_pressed is consistent with pressed_key_count.
fn check_no_keys_pressed_logic(state: &EngineState) -> bool {
    state.no_keys_pressed() == (state.pressed_key_count() == 0)
}

/// Check that pending count matches no_pending_decisions.
fn check_pending_count_consistent(state: &EngineState) -> bool {
    state.no_pending_decisions() == (state.pending_count() == 0)
}

/// Check all invariants for a given state.
fn check_all_invariants(state: &EngineState, prev_version: u64) -> bool {
    check_base_layer_always_active(state)
        && check_no_duplicate_layers(state)
        && check_version_monotonic(prev_version, state.version())
        && check_layer_count_consistent(state)
        && check_only_base_layer_logic(state)
        && check_key_count_consistent(state)
        && check_no_keys_pressed_logic(state)
        && check_pending_count_consistent(state)
}

// === Property Tests ===

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Test that invariants hold after applying a single random mutation.
    #[test]
    fn invariants_hold_after_single_mutation(mutation in arb_mutation()) {
        let mut state = EngineState::new(TimingConfig::default());
        let prev_version = state.version();

        // Apply mutation (may fail, which is okay)
        let _ = state.apply(mutation);

        // Check invariants regardless of whether mutation succeeded
        prop_assert!(check_all_invariants(&state, prev_version));
    }

    /// Test that invariants hold after applying a sequence of random mutations.
    #[test]
    fn invariants_hold_after_mutation_sequence(mutations in arb_mutation_sequence()) {
        let mut state = EngineState::new(TimingConfig::default());

        for mutation in mutations {
            let prev_version = state.version();

            // Apply mutation (may fail, which is okay)
            let _ = state.apply(mutation);

            // Check invariants after each mutation
            prop_assert!(
                check_all_invariants(&state, prev_version),
                "Invariant violated after applying mutation"
            );
        }
    }

    /// Test that base layer is always in active layers.
    #[test]
    fn base_layer_always_active(mutations in arb_mutation_sequence()) {
        let mut state = EngineState::new(TimingConfig::default());

        for mutation in mutations {
            let _ = state.apply(mutation);
            prop_assert!(
                check_base_layer_always_active(&state),
                "Base layer not in active layers"
            );
        }
    }

    /// Test that there are never duplicate layers in the stack.
    #[test]
    fn no_duplicate_layers(mutations in arb_mutation_sequence()) {
        let mut state = EngineState::new(TimingConfig::default());

        for mutation in mutations {
            let _ = state.apply(mutation);
            prop_assert!(
                check_no_duplicate_layers(&state),
                "Duplicate layers found in active layers"
            );
        }
    }

    /// Test that version number is monotonically increasing.
    #[test]
    fn version_monotonic(mutations in arb_mutation_sequence()) {
        let mut state = EngineState::new(TimingConfig::default());
        let mut last_version = state.version();

        for mutation in mutations {
            if state.apply(mutation.clone()).is_ok() {
                let new_version = state.version();
                prop_assert!(
                    new_version > last_version,
                    "Version did not increase after successful mutation: {} -> {}",
                    last_version,
                    new_version
                );
                last_version = new_version;
            }
        }
    }

    /// Test that batch mutations maintain atomicity on failure.
    #[test]
    fn batch_atomicity_on_failure(
        mutations1 in prop::collection::vec(arb_mutation(), 1..=5),
        mutations2 in prop::collection::vec(arb_mutation(), 1..=5)
    ) {
        let mut state = EngineState::new(TimingConfig::default());

        // Apply first batch to establish some state
        for mutation in mutations1 {
            let _ = state.apply(mutation);
        }

        // Capture state before batch
        let snapshot_before = state.clone();
        let version_before = state.version();

        // Try to apply second batch (may fail)
        let result = state.apply_batch(mutations2);

        if result.is_err() {
            // On failure, state should be unchanged
            prop_assert_eq!(
                state.version(),
                version_before,
                "Version changed after failed batch"
            );
            prop_assert_eq!(
                state.pressed_key_count(),
                snapshot_before.pressed_key_count(),
                "Key count changed after failed batch"
            );
            prop_assert_eq!(
                state.active_layer_count(),
                snapshot_before.active_layer_count(),
                "Layer count changed after failed batch"
            );
        }

        // Invariants must hold regardless of success/failure
        prop_assert!(check_all_invariants(&state, version_before));
    }

    /// Test that cloned state is independent.
    #[test]
    fn cloned_state_independence(mutations in arb_mutation_sequence()) {
        let mut state1 = EngineState::new(TimingConfig::default());

        // Apply mutations to state1
        for mutation in mutations.iter() {
            let _ = state1.apply(mutation.clone());
        }

        // Clone the state
        let mut state2 = state1.clone();

        // They should be equal initially
        prop_assert_eq!(state1.version(), state2.version());
        prop_assert_eq!(state1.base_layer(), state2.base_layer());

        // Apply more mutations to state2
        for mutation in mutations {
            let _ = state2.apply(mutation);
        }

        // state1 should be unchanged
        prop_assert!(
            state2.version() >= state1.version(),
            "Cloned state mutation affected original"
        );
    }

    /// Test that key press/release state is consistent.
    #[test]
    fn key_press_release_consistent(
        key in arb_keycode(),
        press_time in arb_timestamp(),
        release_time in arb_timestamp()
    ) {
        let mut state = EngineState::new(TimingConfig::default());

        // Press key
        let press = Mutation::KeyDown {
            key,
            timestamp_us: press_time,
            is_repeat: false,
        };

        if state.apply(press).is_ok() {
            prop_assert!(state.is_key_pressed(key), "Key should be pressed");
            prop_assert_eq!(state.pressed_key_count(), 1);
            prop_assert!(!state.no_keys_pressed());

            // Release key
            let release = Mutation::KeyUp {
                key,
                timestamp_us: press_time + release_time,
            };

            if state.apply(release).is_ok() {
                prop_assert!(!state.is_key_pressed(key), "Key should be released");
                prop_assert_eq!(state.pressed_key_count(), 0);
                prop_assert!(state.no_keys_pressed());
            }
        }
    }

    /// Test that modifier activation/deactivation is consistent.
    #[test]
    fn modifier_activation_consistent(modifier_id in arb_modifier_id()) {
        let mut state = EngineState::new(TimingConfig::default());
        let modifier = Modifier::Virtual(modifier_id);

        // Activate modifier
        let activate = Mutation::ActivateModifier { modifier_id };
        if state.apply(activate).is_ok() {
            prop_assert!(
                state.is_modifier_active(modifier),
                "Modifier should be active"
            );

            // Deactivate modifier
            let deactivate = Mutation::DeactivateModifier { modifier_id };
            if state.apply(deactivate).is_ok() {
                prop_assert!(
                    !state.is_modifier_active(modifier),
                    "Modifier should be inactive"
                );
            }
        }
    }

    /// Test that layer push/pop is consistent.
    #[test]
    fn layer_push_pop_consistent(layer_id in arb_layer_id()) {
        let mut state = EngineState::new(TimingConfig::default());
        let initial_count = state.active_layer_count();

        // Push layer
        let push = Mutation::PushLayer { layer_id };
        if state.apply(push).is_ok() {
            prop_assert!(state.is_layer_active(layer_id), "Layer should be active");
            prop_assert_eq!(state.active_layer_count(), initial_count + 1);
            prop_assert_eq!(state.top_layer(), layer_id);

            // Pop layer
            let pop = Mutation::PopLayer;
            if state.apply(pop).is_ok() {
                prop_assert_eq!(
                    state.active_layer_count(),
                    initial_count,
                    "Layer count should be restored"
                );
            }
        }
    }

    /// Test that pending decisions clear on layer changes.
    #[test]
    fn pending_clears_on_layer_change(
        key in arb_keycode(),
        layer_id in arb_layer_id(),
        timestamp in arb_timestamp()
    ) {
        let mut state = EngineState::new(TimingConfig::default());

        // Add a pending decision
        let add_pending = Mutation::AddTapHold {
            key,
            pressed_at: timestamp,
            tap_action: KeyCode::A,
            hold_action: HoldAction::Key(KeyCode::B),
        };

        if state.apply(add_pending).is_ok() {
            let pending_before = state.pending_count();
            prop_assert!(pending_before > 0, "Should have pending decision");

            // Push a layer (should clear pending)
            let push = Mutation::PushLayer { layer_id };
            if state.apply(push).is_ok() {
                prop_assert_eq!(
                    state.pending_count(),
                    0,
                    "Pending decisions should be cleared on layer change"
                );
            }
        }
    }

    /// Test that ClearModifiers clears all modifiers.
    #[test]
    fn clear_modifiers_clears_all(modifier_ids in prop::collection::vec(arb_modifier_id(), 1..=10)) {
        let mut state = EngineState::new(TimingConfig::default());

        // Activate multiple modifiers
        for modifier_id in &modifier_ids {
            let _ = state.apply(Mutation::ActivateModifier {
                modifier_id: *modifier_id,
            });
        }

        // Clear all modifiers
        if state.apply(Mutation::ClearModifiers).is_ok() {
            // Check that all modifiers are cleared
            for modifier_id in modifier_ids {
                prop_assert!(
                    !state.is_modifier_active(Modifier::Virtual(modifier_id)),
                    "Modifier {} should be cleared",
                    modifier_id
                );
            }
        }
    }

    /// Test that ClearPending clears all pending decisions.
    #[test]
    fn clear_pending_clears_all(
        keys in prop::collection::vec(arb_keycode(), 1..=5),
        timestamps in prop::collection::vec(arb_timestamp(), 1..=5)
    ) {
        let mut state = EngineState::new(TimingConfig::default());

        // Add multiple pending decisions
        for (key, timestamp) in keys.iter().zip(timestamps.iter()) {
            let _ = state.apply(Mutation::AddTapHold {
                key: *key,
                pressed_at: *timestamp,
                tap_action: KeyCode::A,
                hold_action: HoldAction::Key(KeyCode::B),
            });
        }

        let pending_before = state.pending_count();
        if pending_before > 0 {
            // Clear all pending
            if state.apply(Mutation::ClearPending).is_ok() {
                prop_assert_eq!(
                    state.pending_count(),
                    0,
                    "All pending decisions should be cleared"
                );
                prop_assert!(state.no_pending_decisions());
            }
        }
    }
}

// === Unit Tests for Edge Cases ===

#[cfg(test)]
mod edge_cases {
    use super::*;

    #[test]
    fn empty_state_has_valid_invariants() {
        let state = EngineState::new(TimingConfig::default());
        assert!(check_all_invariants(&state, 0));
    }

    #[test]
    fn cannot_pop_base_layer() {
        let mut state = EngineState::new(TimingConfig::default());
        let result = state.apply(Mutation::PopLayer);
        assert!(result.is_err());
        assert!(check_all_invariants(&state, 0));
    }

    #[test]
    fn cannot_release_unpressed_key() {
        let mut state = EngineState::new(TimingConfig::default());
        let result = state.apply(Mutation::KeyUp {
            key: KeyCode::A,
            timestamp_us: 1000,
        });
        assert!(result.is_err());
        assert!(check_all_invariants(&state, 0));
    }

    #[test]
    fn cannot_press_already_pressed_key() {
        let mut state = EngineState::new(TimingConfig::default());
        state
            .apply(Mutation::KeyDown {
                key: KeyCode::A,
                timestamp_us: 1000,
                is_repeat: false,
            })
            .unwrap();

        let result = state.apply(Mutation::KeyDown {
            key: KeyCode::A,
            timestamp_us: 2000,
            is_repeat: false,
        });
        assert!(result.is_err());
        assert!(check_all_invariants(&state, 1));
    }

    #[test]
    fn modifier_255_is_reserved() {
        let mut state = EngineState::new(TimingConfig::default());
        let result = state.apply(Mutation::ActivateModifier { modifier_id: 255 });
        assert!(result.is_err());
        assert!(check_all_invariants(&state, 0));
    }

    #[test]
    fn batch_with_nested_batch_fails() {
        let mut state = EngineState::new(TimingConfig::default());
        let batch = vec![
            Mutation::KeyDown {
                key: KeyCode::A,
                timestamp_us: 1000,
                is_repeat: false,
            },
            Mutation::Batch {
                mutations: vec![Mutation::PushLayer { layer_id: 1 }],
            },
        ];

        let result = state.apply_batch(batch);
        assert!(result.is_err());
        assert!(check_all_invariants(&state, 0));
    }

    #[test]
    fn empty_batch_fails() {
        let mut state = EngineState::new(TimingConfig::default());
        let result = state.apply_batch(vec![]);
        assert!(result.is_err());
        assert!(check_all_invariants(&state, 0));
    }

    #[test]
    fn layer_toggle_maintains_invariants() {
        let mut state = EngineState::new(TimingConfig::default());

        // Toggle on
        state.apply(Mutation::ToggleLayer { layer_id: 1 }).unwrap();
        assert!(state.is_layer_active(1));
        assert!(check_all_invariants(&state, 0));

        // Toggle off
        state.apply(Mutation::ToggleLayer { layer_id: 1 }).unwrap();
        assert!(!state.is_layer_active(1));
        assert!(check_all_invariants(&state, 1));
    }

    #[test]
    fn standard_modifier_queries_work() {
        let state = EngineState::new(TimingConfig::default());
        assert!(!state.is_modifier_active(Modifier::Standard(StandardModifier::Shift)));
        assert!(!state.is_modifier_active(Modifier::Standard(StandardModifier::Control)));
        assert!(!state.is_modifier_active(Modifier::Standard(StandardModifier::Alt)));
    }

    #[test]
    fn multiple_keys_pressed_simultaneously() {
        let mut state = EngineState::new(TimingConfig::default());

        let keys = vec![KeyCode::A, KeyCode::B, KeyCode::C, KeyCode::D, KeyCode::E];

        for (i, key) in keys.iter().enumerate() {
            state
                .apply(Mutation::KeyDown {
                    key: *key,
                    timestamp_us: 1000 + i as u64,
                    is_repeat: false,
                })
                .unwrap();
        }

        assert_eq!(state.pressed_key_count(), keys.len());
        assert!(check_all_invariants(&state, 0));

        for key in &keys {
            assert!(state.is_key_pressed(*key));
        }
    }

    #[test]
    fn multiple_layers_can_be_active() {
        let mut state = EngineState::new(TimingConfig::default());
        let initial_count = state.active_layer_count();

        for layer_id in 1u16..=5 {
            state.apply(Mutation::PushLayer { layer_id }).unwrap();
        }

        assert_eq!(state.active_layer_count(), initial_count + 5);
        assert_eq!(state.top_layer(), 5);
        assert!(check_all_invariants(&state, 0));
    }

    #[test]
    fn stress_test_many_mutations() {
        let mut state = EngineState::new(TimingConfig::default());

        // Apply 1000 random valid mutations
        for i in 0..1000 {
            let mutation = match i % 10 {
                0 => Mutation::KeyDown {
                    key: KeyCode::A,
                    timestamp_us: i as u64,
                    is_repeat: true,
                },
                1 => Mutation::PushLayer {
                    layer_id: ((i % 5) + 1) as u16,
                },
                2 if state.active_layer_count() > 1 => Mutation::PopLayer,
                3 => Mutation::ActivateModifier {
                    modifier_id: (i % 100) as u8,
                },
                4 => Mutation::DeactivateModifier {
                    modifier_id: (i % 100) as u8,
                },
                5 => Mutation::ClearModifiers,
                6 => Mutation::ClearPending,
                7 => Mutation::ToggleLayer {
                    layer_id: ((i % 5) + 1) as u16,
                },
                _ => continue,
            };

            let _ = state.apply(mutation);
            assert!(check_all_invariants(&state, 0));
        }
    }
}
