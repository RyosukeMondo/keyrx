#[cfg(test)]
mod tests {
    use keyrx_core::config::{BaseKeyMapping, DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping};
    use keyrx_core::runtime::{process_event, DeviceState, KeyEvent, KeyLookup};

    fn create_test_config(mappings: Vec<KeyMapping>) -> DeviceConfig {
        DeviceConfig {
            identifier: DeviceIdentifier::Any,
            mappings,
        }
    }

    #[test]
    fn test_tap_hold_rapid_input_sequence() {
        // Regression test for: tap_hold modifier becomes sticky with rapid input
        // User scenario: Hold M (MD_02) and rapidly type A, O, E, U
        // Expected: MD_02 deactivates when M is released
        // Bug: MD_02 stays active (sticky) after M release
        let config = create_test_config(vec![
            KeyMapping::tap_hold(KeyCode::M, KeyCode::Backspace, 2, 200),
        ]);
        let lookup = KeyLookup::from_device_config(&config);
        let mut state = DeviceState::new();

        // Press M at t=0 (enters Pending state)
        let _ = process_event(
            KeyEvent::press(KeyCode::M).with_timestamp(0),
            &lookup,
            &mut state,
        );
        assert!(
            !state.is_modifier_active(2),
            "MD_02 should not be active yet (Pending state)"
        );

        // Press A at t=50ms (before 200ms threshold)
        // This triggers permissive hold → MD_02 activates
        let _ = process_event(
            KeyEvent::press(KeyCode::A).with_timestamp(50_000),
            &lookup,
            &mut state,
        );
        assert!(
            state.is_modifier_active(2),
            "MD_02 should activate via permissive hold"
        );

        // Release A at t=100ms (while still holding M)
        let _ = process_event(
            KeyEvent::release(KeyCode::A).with_timestamp(100_000),
            &lookup,
            &mut state,
        );
        assert!(
            state.is_modifier_active(2),
            "MD_02 should still be active (M still held)"
        );

        // Press O at t=150ms
        let _ = process_event(
            KeyEvent::press(KeyCode::O).with_timestamp(150_000),
            &lookup,
            &mut state,
        );
        assert!(state.is_modifier_active(2), "MD_02 should still be active");

        // Release O at t=200ms
        let _ = process_event(
            KeyEvent::release(KeyCode::O).with_timestamp(200_000),
            &lookup,
            &mut state,
        );
        assert!(state.is_modifier_active(2), "MD_02 should still be active");

        // Press E at t=250ms
        let _ = process_event(
            KeyEvent::press(KeyCode::E).with_timestamp(250_000),
            &lookup,
            &mut state,
        );
        assert!(state.is_modifier_active(2), "MD_02 should still be active");

        // Release E at t=300ms
        let _ = process_event(
            KeyEvent::release(KeyCode::E).with_timestamp(300_000),
            &lookup,
            &mut state,
        );
        assert!(state.is_modifier_active(2), "MD_02 should still be active");

        // Press U at t=350ms
        let _ = process_event(
            KeyEvent::press(KeyCode::U).with_timestamp(350_000),
            &lookup,
            &mut state,
        );
        assert!(state.is_modifier_active(2), "MD_02 should still be active");

        // Release U at t=400ms
        let _ = process_event(
            KeyEvent::release(KeyCode::U).with_timestamp(400_000),
            &lookup,
            &mut state,
        );
        assert!(state.is_modifier_active(2), "MD_02 should still be active");

        // **CRITICAL**: Release M at t=450ms
        // This should deactivate MD_02
        let _ = process_event(
            KeyEvent::release(KeyCode::M).with_timestamp(450_000),
            &lookup,
            &mut state,
        );
        assert!(
            !state.is_modifier_active(2),
            "BUG: MD_02 should deactivate when M is released, but it stays sticky"
        );
    }
}
