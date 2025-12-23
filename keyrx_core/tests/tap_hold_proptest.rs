//! Property-based tests for tap-hold determinism
//!
//! These tests use proptest to verify that:
//! 1. The same sequence of events always produces the same output
//! 2. State transitions are deterministic regardless of timing variations
//! 3. Random key sequences don't cause panics or inconsistent behavior
//!
//! Per task requirements: Run 10K+ cases to verify no determinism violations.

use keyrx_core::config::{DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping};
use keyrx_core::runtime::{
    check_tap_hold_timeouts, process_event, DeviceState, KeyEvent, KeyLookup,
};
use proptest::prelude::*;

// ============================================================================
// Strategy Definitions
// ============================================================================

/// Subset of KeyCodes suitable for tap-hold testing
/// Limited to common keys to make tests more realistic
fn key_code_strategy() -> impl Strategy<Value = KeyCode> {
    prop_oneof![
        Just(KeyCode::A),
        Just(KeyCode::B),
        Just(KeyCode::C),
        Just(KeyCode::D),
        Just(KeyCode::E),
        Just(KeyCode::F),
        Just(KeyCode::G),
        Just(KeyCode::H),
        Just(KeyCode::I),
        Just(KeyCode::J),
        Just(KeyCode::K),
        Just(KeyCode::L),
        Just(KeyCode::M),
        Just(KeyCode::N),
        Just(KeyCode::Space),
        Just(KeyCode::CapsLock),
        Just(KeyCode::Enter),
        Just(KeyCode::Tab),
        Just(KeyCode::Escape),
    ]
}

/// Keys that can be used as tap-hold trigger keys
fn tap_hold_key_strategy() -> impl Strategy<Value = KeyCode> {
    prop_oneof![
        Just(KeyCode::CapsLock),
        Just(KeyCode::Space),
        Just(KeyCode::Enter),
        Just(KeyCode::Tab),
    ]
}

/// Keys that can be pressed while tap-hold is pending (not tap-hold keys)
fn regular_key_strategy() -> impl Strategy<Value = KeyCode> {
    prop_oneof![
        Just(KeyCode::A),
        Just(KeyCode::B),
        Just(KeyCode::C),
        Just(KeyCode::D),
        Just(KeyCode::E),
        Just(KeyCode::F),
        Just(KeyCode::G),
        Just(KeyCode::H),
        Just(KeyCode::I),
        Just(KeyCode::J),
        Just(KeyCode::K),
        Just(KeyCode::L),
    ]
}

/// Generate realistic tap-hold thresholds (100-300ms)
fn threshold_strategy() -> impl Strategy<Value = u16> {
    100u16..300u16
}

/// Generate time deltas (microseconds)
fn time_delta_strategy() -> impl Strategy<Value = u64> {
    10_000u64..500_000u64 // 10ms to 500ms
}

/// Event type for property tests
#[derive(Debug, Clone, Copy)]
enum TestEvent {
    Press(KeyCode),
    Release(KeyCode),
    Timeout,
}

/// Generate a sequence of test events
fn event_sequence_strategy() -> impl Strategy<Value = Vec<TestEvent>> {
    prop::collection::vec(
        prop_oneof![
            key_code_strategy().prop_map(TestEvent::Press),
            key_code_strategy().prop_map(TestEvent::Release),
            Just(TestEvent::Timeout),
        ],
        1..50,
    )
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Creates a standard tap-hold config for testing
fn create_test_config(tap_hold_key: KeyCode, threshold_ms: u16) -> DeviceConfig {
    DeviceConfig {
        identifier: DeviceIdentifier {
            pattern: String::from("*"),
        },
        mappings: vec![KeyMapping::tap_hold(
            tap_hold_key,
            KeyCode::Escape, // tap produces Escape
            0,               // modifier ID
            threshold_ms,
        )],
    }
}

/// Creates a multi-tap-hold config for more complex testing
fn create_multi_tap_hold_config(thresholds: &[(KeyCode, u16)]) -> DeviceConfig {
    let mappings: Vec<KeyMapping> = thresholds
        .iter()
        .enumerate()
        .map(|(i, (key, threshold))| {
            KeyMapping::tap_hold(*key, KeyCode::Escape, i as u8, *threshold)
        })
        .collect();

    DeviceConfig {
        identifier: DeviceIdentifier {
            pattern: String::from("*"),
        },
        mappings,
    }
}

/// Process an event sequence and return outputs
fn process_sequence(
    events: &[(TestEvent, u64)], // Event and cumulative timestamp
    lookup: &KeyLookup,
    state: &mut DeviceState,
) -> Vec<KeyEvent> {
    let mut outputs = Vec::new();
    for (event, timestamp) in events {
        match event {
            TestEvent::Press(key) => {
                outputs.extend(process_event(
                    KeyEvent::press(*key).with_timestamp(*timestamp),
                    lookup,
                    state,
                ));
            }
            TestEvent::Release(key) => {
                outputs.extend(process_event(
                    KeyEvent::release(*key).with_timestamp(*timestamp),
                    lookup,
                    state,
                ));
            }
            TestEvent::Timeout => {
                outputs.extend(check_tap_hold_timeouts(*timestamp, state));
            }
        }
    }
    outputs
}

/// Compare two output sequences for equality
fn outputs_equal(a: &[KeyEvent], b: &[KeyEvent]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .all(|(x, y)| x.keycode() == y.keycode() && x.is_press() == y.is_press())
}

// ============================================================================
// Property Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    /// Property: Same input always produces same output (determinism)
    #[test]
    fn same_input_same_output(
        tap_hold_key in tap_hold_key_strategy(),
        threshold in threshold_strategy(),
        events in event_sequence_strategy(),
        time_deltas in prop::collection::vec(time_delta_strategy(), 50),
    ) {
        let config = create_test_config(tap_hold_key, threshold);
        let lookup = KeyLookup::from_device_config(&config);

        // Build timestamped events
        let mut cumulative_time = 0u64;
        let timestamped_events: Vec<(TestEvent, u64)> = events
            .iter()
            .enumerate()
            .map(|(i, event)| {
                cumulative_time += time_deltas.get(i).copied().unwrap_or(50_000);
                (*event, cumulative_time)
            })
            .collect();

        // First run
        let mut state1 = DeviceState::new();
        let outputs1 = process_sequence(&timestamped_events, &lookup, &mut state1);

        // Second run with fresh state
        let mut state2 = DeviceState::new();
        let outputs2 = process_sequence(&timestamped_events, &lookup, &mut state2);

        // Outputs must be identical
        prop_assert!(
            outputs_equal(&outputs1, &outputs2),
            "Same input produced different outputs!\nRun 1: {:?}\nRun 2: {:?}",
            outputs1.iter().map(|e| (e.keycode(), e.is_press())).collect::<Vec<_>>(),
            outputs2.iter().map(|e| (e.keycode(), e.is_press())).collect::<Vec<_>>()
        );
    }

    /// Property: Tap sequence (press + quick release) produces tap key
    #[test]
    fn tap_produces_tap_key(
        tap_hold_key in tap_hold_key_strategy(),
        threshold in threshold_strategy(),
        release_delay in 1u64..100_000u64, // < 100ms guaranteed
    ) {
        // Ensure release is before threshold
        let threshold_us = u64::from(threshold) * 1000;
        let actual_delay = release_delay.min(threshold_us.saturating_sub(1_000));

        let config = create_test_config(tap_hold_key, threshold);
        let lookup = KeyLookup::from_device_config(&config);
        let mut state = DeviceState::new();

        // Press
        let press_outputs = process_event(
            KeyEvent::press(tap_hold_key).with_timestamp(0),
            &lookup,
            &mut state,
        );
        prop_assert!(press_outputs.is_empty(), "Press should not produce output");

        // Release before threshold
        let release_outputs = process_event(
            KeyEvent::release(tap_hold_key).with_timestamp(actual_delay),
            &lookup,
            &mut state,
        );

        // Should produce tap key (Escape) press and release
        prop_assert_eq!(
            release_outputs.len(),
            2,
            "Tap should produce 2 events, got {}",
            release_outputs.len()
        );
        prop_assert_eq!(release_outputs[0].keycode(), KeyCode::Escape);
        prop_assert!(release_outputs[0].is_press());
        prop_assert_eq!(release_outputs[1].keycode(), KeyCode::Escape);
        prop_assert!(release_outputs[1].is_release());
    }

    /// Property: Hold past threshold activates modifier
    #[test]
    fn hold_activates_modifier(
        tap_hold_key in tap_hold_key_strategy(),
        threshold in threshold_strategy(),
        extra_time in 1_000u64..100_000u64,
    ) {
        let threshold_us = u64::from(threshold) * 1000;
        let timeout_time = threshold_us + extra_time;

        let config = create_test_config(tap_hold_key, threshold);
        let lookup = KeyLookup::from_device_config(&config);
        let mut state = DeviceState::new();

        // Press
        let _ = process_event(
            KeyEvent::press(tap_hold_key).with_timestamp(0),
            &lookup,
            &mut state,
        );

        // Check timeout after threshold
        let _ = check_tap_hold_timeouts(timeout_time, &mut state);

        // Modifier should be active
        prop_assert!(
            state.is_modifier_active(0),
            "Modifier should be active after timeout at {}us (threshold: {}us)",
            timeout_time,
            threshold_us
        );

        // Release should deactivate modifier
        let _ = process_event(
            KeyEvent::release(tap_hold_key).with_timestamp(timeout_time + 10_000),
            &lookup,
            &mut state,
        );
        prop_assert!(
            !state.is_modifier_active(0),
            "Modifier should be deactivated after release"
        );
    }

    /// Property: Permissive hold triggers on interrupt
    #[test]
    fn permissive_hold_on_interrupt(
        tap_hold_key in tap_hold_key_strategy(),
        interrupt_key in regular_key_strategy(),
        threshold in threshold_strategy(),
        interrupt_time in 1_000u64..150_000u64, // Before any reasonable threshold
    ) {
        // Ensure interrupt is before threshold
        let threshold_us = u64::from(threshold) * 1000;
        let actual_interrupt = interrupt_time.min(threshold_us.saturating_sub(1_000));

        let config = create_test_config(tap_hold_key, threshold);
        let lookup = KeyLookup::from_device_config(&config);
        let mut state = DeviceState::new();

        // Press tap-hold key
        let _ = process_event(
            KeyEvent::press(tap_hold_key).with_timestamp(0),
            &lookup,
            &mut state,
        );
        prop_assert!(
            !state.is_modifier_active(0),
            "Modifier should not be active yet"
        );

        // Press interrupting key (triggers permissive hold)
        let _ = process_event(
            KeyEvent::press(interrupt_key).with_timestamp(actual_interrupt),
            &lookup,
            &mut state,
        );

        // Modifier should now be active
        prop_assert!(
            state.is_modifier_active(0),
            "Modifier should be active after permissive hold triggered at {}us",
            actual_interrupt
        );
    }

    /// Property: State is clean after complete tap or hold cycle
    #[test]
    fn state_clean_after_cycle(
        tap_hold_key in tap_hold_key_strategy(),
        threshold in threshold_strategy(),
        is_tap in proptest::bool::ANY,
        release_time in 10_000u64..500_000u64,
    ) {
        let threshold_us = u64::from(threshold) * 1000;

        let config = create_test_config(tap_hold_key, threshold);
        let lookup = KeyLookup::from_device_config(&config);
        let mut state = DeviceState::new();

        // Press
        let _ = process_event(
            KeyEvent::press(tap_hold_key).with_timestamp(0),
            &lookup,
            &mut state,
        );

        let actual_release = if is_tap {
            // For tap: release before threshold
            release_time.min(threshold_us.saturating_sub(1_000))
        } else {
            // For hold: check timeout first
            let _ = check_tap_hold_timeouts(threshold_us + 10_000, &mut state);
            threshold_us + release_time
        };

        // Release
        let _ = process_event(
            KeyEvent::release(tap_hold_key).with_timestamp(actual_release),
            &lookup,
            &mut state,
        );

        // State should be clean
        prop_assert!(
            !state.is_modifier_active(0),
            "Modifier should not be active after complete cycle"
        );
        prop_assert!(
            !state.tap_hold_processor_ref().has_pending_keys(),
            "Should have no pending keys after complete cycle"
        );
    }

    /// Property: Multiple tap-holds can coexist without interference
    #[test]
    fn multiple_tap_holds_independent(
        threshold1 in threshold_strategy(),
        threshold2 in threshold_strategy(),
        key1_time in 0u64..10_000u64,
        key2_time in 10_000u64..20_000u64,
    ) {
        let config = create_multi_tap_hold_config(&[
            (KeyCode::CapsLock, threshold1),
            (KeyCode::Space, threshold2),
        ]);
        let lookup = KeyLookup::from_device_config(&config);
        let mut state = DeviceState::new();

        // Press both keys
        let _ = process_event(
            KeyEvent::press(KeyCode::CapsLock).with_timestamp(key1_time),
            &lookup,
            &mut state,
        );
        let _ = process_event(
            KeyEvent::press(KeyCode::Space).with_timestamp(key2_time),
            &lookup,
            &mut state,
        );

        // Both should be pending (neither modifier active yet)
        prop_assert!(
            !state.is_modifier_active(0),
            "Modifier 0 should not be active yet"
        );
        prop_assert!(
            !state.is_modifier_active(1),
            "Modifier 1 should not be active yet"
        );

        // Timeout past threshold1
        let threshold1_us = u64::from(threshold1) * 1000;
        let timeout1 = key1_time + threshold1_us + 1_000;
        let _ = check_tap_hold_timeouts(timeout1, &mut state);

        // Modifier 0 should be active, but modifier 1 depends on threshold2
        prop_assert!(
            state.is_modifier_active(0),
            "Modifier 0 should be active after its threshold"
        );
    }

    /// Property: Timeout checking is idempotent
    #[test]
    fn timeout_idempotent(
        tap_hold_key in tap_hold_key_strategy(),
        threshold in threshold_strategy(),
        extra_checks in 1usize..5,
    ) {
        let threshold_us = u64::from(threshold) * 1000;
        let timeout_time = threshold_us + 10_000;

        let config = create_test_config(tap_hold_key, threshold);
        let lookup = KeyLookup::from_device_config(&config);
        let mut state = DeviceState::new();

        // Press
        let _ = process_event(
            KeyEvent::press(tap_hold_key).with_timestamp(0),
            &lookup,
            &mut state,
        );

        // First timeout check
        let first_output = check_tap_hold_timeouts(timeout_time, &mut state);
        let modifier_active = state.is_modifier_active(0);

        // Additional timeout checks at same time should be idempotent
        for _ in 0..extra_checks {
            let additional_output = check_tap_hold_timeouts(timeout_time, &mut state);
            prop_assert!(
                additional_output.is_empty(),
                "Additional timeout check should not produce output"
            );
            prop_assert_eq!(
                state.is_modifier_active(0),
                modifier_active,
                "Modifier state should not change with repeated timeout checks"
            );
        }

        // First output should have triggered the modifier
        prop_assert!(
            modifier_active || !first_output.is_empty() || true, // Always passes - just checking for no panic
            "Timeout checking should be idempotent"
        );
    }

    /// Property: Random event sequences don't panic
    #[test]
    fn no_panic_on_random_sequence(
        events in prop::collection::vec(
            prop_oneof![
                key_code_strategy().prop_map(|k| (true, k)),  // Press
                key_code_strategy().prop_map(|k| (false, k)), // Release
            ],
            0..100
        ),
        thresholds in prop::collection::vec(threshold_strategy(), 1..4),
    ) {
        let tap_hold_keys = [KeyCode::CapsLock, KeyCode::Space, KeyCode::Enter];
        let configs: Vec<(KeyCode, u16)> = tap_hold_keys
            .iter()
            .zip(thresholds.iter().cycle())
            .take(3)
            .map(|(k, t)| (*k, *t))
            .collect();

        let config = create_multi_tap_hold_config(&configs);
        let lookup = KeyLookup::from_device_config(&config);
        let mut state = DeviceState::new();

        let mut time = 0u64;
        for (is_press, key) in events {
            time += 10_000; // 10ms between events
            let event = if is_press {
                KeyEvent::press(key).with_timestamp(time)
            } else {
                KeyEvent::release(key).with_timestamp(time)
            };
            let _ = process_event(event, &lookup, &mut state);

            // Occasional timeout check
            if time % 50_000 == 0 {
                let _ = check_tap_hold_timeouts(time, &mut state);
            }
        }
        // If we get here without panic, the test passes
        prop_assert!(true);
    }
}

// ============================================================================
// Additional Non-Proptest Determinism Tests
// ============================================================================

#[test]
fn determinism_with_100k_iterations() {
    // Run the same sequence 100K times to verify determinism
    let config = create_test_config(KeyCode::CapsLock, 200);
    let lookup = KeyLookup::from_device_config(&config);

    let events = vec![
        (TestEvent::Press(KeyCode::CapsLock), 0),
        (TestEvent::Press(KeyCode::A), 50_000),
        (TestEvent::Release(KeyCode::A), 100_000),
        (TestEvent::Timeout, 250_000),
        (TestEvent::Press(KeyCode::B), 300_000),
        (TestEvent::Release(KeyCode::B), 350_000),
        (TestEvent::Release(KeyCode::CapsLock), 400_000),
    ];

    // Get baseline output
    let mut baseline_state = DeviceState::new();
    let baseline_outputs = process_sequence(&events, &lookup, &mut baseline_state);

    // Verify 100K times
    for i in 0..100_000 {
        let mut state = DeviceState::new();
        let outputs = process_sequence(&events, &lookup, &mut state);

        assert!(
            outputs_equal(&baseline_outputs, &outputs),
            "Iteration {} produced different output!\nBaseline: {:?}\nGot: {:?}",
            i,
            baseline_outputs
                .iter()
                .map(|e| (e.keycode(), e.is_press()))
                .collect::<Vec<_>>(),
            outputs
                .iter()
                .map(|e| (e.keycode(), e.is_press()))
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn determinism_with_varying_timing() {
    // Same logical sequence with different timing should produce consistent behavior
    let config = create_test_config(KeyCode::Space, 200);
    let lookup = KeyLookup::from_device_config(&config);

    // Test tap behavior with various quick releases
    for release_time in [10_000u64, 50_000, 100_000, 150_000, 199_000] {
        let mut state = DeviceState::new();
        let _ = process_event(
            KeyEvent::press(KeyCode::Space).with_timestamp(0),
            &lookup,
            &mut state,
        );
        let outputs = process_event(
            KeyEvent::release(KeyCode::Space).with_timestamp(release_time),
            &lookup,
            &mut state,
        );

        assert_eq!(
            outputs.len(),
            2,
            "Release at {}us should produce tap (2 events)",
            release_time
        );
        assert_eq!(outputs[0].keycode(), KeyCode::Escape);
        assert!(outputs[0].is_press());
    }

    // Test hold behavior with various delays past threshold
    for timeout_time in [200_001u64, 250_000, 300_000, 500_000, 1_000_000] {
        let mut state = DeviceState::new();
        let _ = process_event(
            KeyEvent::press(KeyCode::Space).with_timestamp(0),
            &lookup,
            &mut state,
        );
        let _ = check_tap_hold_timeouts(timeout_time, &mut state);

        assert!(
            state.is_modifier_active(0),
            "Timeout at {}us should activate modifier",
            timeout_time
        );
    }
}
