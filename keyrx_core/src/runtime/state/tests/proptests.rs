use super::*;
use crate::config::MAX_MODIFIER_ID;
use proptest::prelude::*;

// Property test: modifier state is always valid (only bits 0-254 can be set)
//
// This test verifies the invariant that IDs >254 are always rejected
// and never modify the state, while valid IDs (0-254) work correctly.
proptest! {
    #[test]
    fn prop_modifier_state_valid(id in 0u8..=255) {
        let mut state = DeviceState::new();

        if id <= MAX_MODIFIER_ID as u8 {
            // Valid IDs should be settable
            assert!(state.set_modifier(id), "set_modifier({}) should succeed", id);
            assert!(state.is_modifier_active(id), "modifier {} should be active after set", id);

            // Should be clearable
            assert!(state.clear_modifier(id), "clear_modifier({}) should succeed", id);
            assert!(!state.is_modifier_active(id), "modifier {} should be inactive after clear", id);
        } else {
            // Invalid IDs (255) should be rejected
            assert!(!state.set_modifier(id), "set_modifier({}) should fail for invalid ID", id);
            assert!(!state.is_modifier_active(id), "modifier {} should never be active for invalid ID", id);

            assert!(!state.clear_modifier(id), "clear_modifier({}) should fail for invalid ID", id);
        }
    }
}

// Property test: lock toggle cycles correctly (OFF→ON→OFF→...)
//
// This test verifies that toggle_lock correctly cycles state based on
// the number of toggles: even toggles = OFF, odd toggles = ON.
proptest! {
    #[test]
    fn prop_lock_toggle_cycles(id in 0u8..=MAX_MODIFIER_ID as u8, toggle_count in 0usize..=20) {
        let mut state = DeviceState::new();

        // Initial state should be OFF
        assert!(!state.is_lock_active(id));

        // Apply toggles
        for _ in 0..toggle_count {
            assert!(state.toggle_lock(id), "toggle_lock({}) should succeed", id);
        }

        // Final state should match parity: odd toggles = ON, even toggles = OFF
        let expected_active = toggle_count % 2 == 1;
        assert_eq!(
            state.is_lock_active(id),
            expected_active,
            "After {} toggles, lock {} should be {} (got {})",
            toggle_count,
            id,
            if expected_active { "ON" } else { "OFF" },
            if state.is_lock_active(id) { "ON" } else { "OFF" }
        );
    }
}

// Property test: invalid lock IDs are always rejected
//
// This test verifies that ID 255 can never be toggled.
proptest! {
    #[test]
    fn prop_lock_invalid_id_rejected(toggle_count in 0usize..=10) {
        let mut state = DeviceState::new();

        // Apply toggles to invalid ID 255
        for _ in 0..toggle_count {
            assert!(!state.toggle_lock(255), "toggle_lock(255) should fail");
        }

        // Lock 255 should never be active
        assert!(!state.is_lock_active(255), "lock 255 should never be active");
    }
}

// Property test: set/clear operations are independent
//
// This test verifies that setting/clearing one modifier doesn't affect others.
proptest! {
    #[test]
    fn prop_modifiers_independent(
        id1 in 0u8..=MAX_MODIFIER_ID as u8,
        id2 in 0u8..=MAX_MODIFIER_ID as u8,
        id3 in 0u8..=MAX_MODIFIER_ID as u8
    ) {
        // Use distinct IDs
        prop_assume!(id1 != id2 && id2 != id3 && id1 != id3);

        let mut state = DeviceState::new();

        // Set all three
        state.set_modifier(id1);
        state.set_modifier(id2);
        state.set_modifier(id3);

        assert!(state.is_modifier_active(id1));
        assert!(state.is_modifier_active(id2));
        assert!(state.is_modifier_active(id3));

        // Clear middle one
        state.clear_modifier(id2);

        // id2 should be inactive, others still active
        assert!(state.is_modifier_active(id1));
        assert!(!state.is_modifier_active(id2));
        assert!(state.is_modifier_active(id3));
    }
}

// Property test: locks are independent
//
// This test verifies that toggling one lock doesn't affect others.
proptest! {
    #[test]
    fn prop_locks_independent(
        id1 in 0u8..=MAX_MODIFIER_ID as u8,
        id2 in 0u8..=MAX_MODIFIER_ID as u8,
        id3 in 0u8..=MAX_MODIFIER_ID as u8
    ) {
        // Use distinct IDs
        prop_assume!(id1 != id2 && id2 != id3 && id1 != id3);

        let mut state = DeviceState::new();

        // Toggle all three ON
        state.toggle_lock(id1);
        state.toggle_lock(id2);
        state.toggle_lock(id3);

        assert!(state.is_lock_active(id1));
        assert!(state.is_lock_active(id2));
        assert!(state.is_lock_active(id3));

        // Toggle middle one OFF
        state.toggle_lock(id2);

        // id2 should be inactive, others still active
        assert!(state.is_lock_active(id1));
        assert!(!state.is_lock_active(id2));
        assert!(state.is_lock_active(id3));
    }
}
