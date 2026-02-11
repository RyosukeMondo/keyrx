use keyrx_core::config::{
    BaseKeyMapping, Condition, DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping,
};
use keyrx_core::runtime::{
    check_tap_hold_timeouts, process_event, DeviceState, KeyEvent, KeyLookup,
};

// ============================================================================
// Test Helpers (Copied/Adapted from tap_hold_integration.rs)
// ============================================================================

fn create_config(mappings: Vec<KeyMapping>) -> DeviceConfig {
    DeviceConfig {
        identifier: DeviceIdentifier {
            pattern: String::from("*"),
        },
        mappings,
    }
}

// ============================================================================
// Reproduction Tests
// ============================================================================

/// Scenario: Shift key stuck holding
/// User report: "shift key keep holding"
/// Hypothesis: Permissive hold activation followed by specific release order might leave state dirty.
#[test]
fn test_repro_stuck_shift_permissive_hold_release_order() {
    // Config: A is Tap: Tab, Hold: Shift (MD_00)
    // Layer MD_00: S -> Shift+S (implied by modifier, but let's make it explicit or just check modifier state)
    // Actually, user_layout.rhai uses `tap_hold("VK_A", "VK_Tab", "MD_09", 200)`
    // And `tap_hold("VK_Num1", "VK_Num1", "MD_04", 200)`
    // Let's simulate a generic Tap-Hold modifier.

    let config = create_config(vec![
        KeyMapping::tap_hold(KeyCode::A, KeyCode::Tab, 0, 200), // A -> Tap:Tab, Hold:MD_00
    ]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // 1. Press A (Tap-Hold key)
    let _ = process_event(
        KeyEvent::press(KeyCode::A).with_timestamp(100_000),
        &lookup,
        &mut state,
    );
    assert!(
        !state.is_modifier_active(0),
        "Modifier should not be active yet"
    );

    // 2. Press S immediately (triggers permissive hold)
    let _s_press = process_event(
        KeyEvent::press(KeyCode::S).with_timestamp(150_000),
        &lookup,
        &mut state,
    );
    assert!(
        state.is_modifier_active(0),
        "Modifier should be active (permissive hold)"
    );

    // 3. Release S
    let _ = process_event(
        KeyEvent::release(KeyCode::S).with_timestamp(250_000),
        &lookup,
        &mut state,
    );

    // 4. Release A
    let _ = process_event(
        KeyEvent::release(KeyCode::A).with_timestamp(300_000),
        &lookup,
        &mut state,
    );

    // Modifier should be OFF
    assert!(
        !state.is_modifier_active(0),
        "Modifier should be OFF after releasing hold key"
    );
}

#[test]
fn test_repro_stuck_shift_release_hold_before_interrupter() {
    // Sequence: Press Hold, Press Interrupter, Release Hold, Release Interrupter
    let config = create_config(vec![KeyMapping::tap_hold(KeyCode::A, KeyCode::Tab, 0, 200)]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Press A
    let _ = process_event(
        KeyEvent::press(KeyCode::A).with_timestamp(100_000),
        &lookup,
        &mut state,
    );

    // Press S (Permissive hold)
    let _ = process_event(
        KeyEvent::press(KeyCode::S).with_timestamp(150_000),
        &lookup,
        &mut state,
    );
    assert!(state.is_modifier_active(0));

    // Release A (The hold key) - Modifier should turn off immediately?
    // Or does it stay on until S is released?
    // Standard behavior: Modifiers usually release when the key is released.
    let _ = process_event(
        KeyEvent::release(KeyCode::A).with_timestamp(200_000),
        &lookup,
        &mut state,
    );

    assert!(
        !state.is_modifier_active(0),
        "Modifier should be OFF immediately after releasing A"
    );

    // Release S
    let _ = process_event(
        KeyEvent::release(KeyCode::S).with_timestamp(250_000),
        &lookup,
        &mut state,
    );

    assert!(!state.is_modifier_active(0));
}

/// Scenario: M_04(VK_A) + VK_S not working sometimes
/// "M_04" is on VK_Num1.
/// Layer MD_04: S -> Ctrl+A
#[test]
fn test_repro_md04_s_combo_timing() {
    let config = create_config(vec![
        // VK_Num1: Tap=1, Hold=MD_04 (200ms)
        KeyMapping::tap_hold(KeyCode::Num1, KeyCode::Num1, 4, 200),
        // Layer MD_04: S -> Ctrl+A
        KeyMapping::conditional(
            Condition::ModifierActive(4),
            vec![BaseKeyMapping::ModifiedOutput {
                from: KeyCode::S,
                to: KeyCode::A,
                shift: false,
                ctrl: true,
                alt: false,
                win: false,
            }],
        ),
    ]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Case 1: Fast sequence (Permissive Hold)
    // Press 1, Press S (within 200ms)

    process_event(
        KeyEvent::press(KeyCode::Num1).with_timestamp(100_000),
        &lookup,
        &mut state,
    );

    // Press S at 150ms
    let outputs = process_event(
        KeyEvent::press(KeyCode::S).with_timestamp(150_000),
        &lookup,
        &mut state,
    );

    // Expectation: Permissive hold activates MD_04. S is remapped to Ctrl+A.
    // Check if MD_04 is active
    assert!(
        state.is_modifier_active(4),
        "MD_04 should be active via permissive hold"
    );

    // Check output. Should be Ctrl (press) + A (press).
    // Note: The order depends on implementation. ModifiedOutput usually sends mods then key.
    let has_ctrl = outputs
        .iter()
        .any(|e| e.keycode() == KeyCode::LCtrl && e.is_press());
    let has_a = outputs
        .iter()
        .any(|e| e.keycode() == KeyCode::A && e.is_press());

    assert!(has_ctrl, "Output should contain LCtrl press");
    assert!(has_a, "Output should contain A press");

    // Cleanup
    process_event(
        KeyEvent::release(KeyCode::S).with_timestamp(200_000),
        &lookup,
        &mut state,
    );
    process_event(
        KeyEvent::release(KeyCode::Num1).with_timestamp(250_000),
        &lookup,
        &mut state,
    );
}

#[test]
fn test_repro_md04_s_combo_timeout() {
    // Case 2: Slow sequence (Wait for timeout)
    let config = create_config(vec![
        KeyMapping::tap_hold(KeyCode::Num1, KeyCode::Num1, 4, 200),
        KeyMapping::conditional(
            Condition::ModifierActive(4),
            vec![BaseKeyMapping::ModifiedOutput {
                from: KeyCode::S,
                to: KeyCode::A,
                shift: false,
                ctrl: true,
                alt: false,
                win: false,
            }],
        ),
    ]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Press 1
    process_event(
        KeyEvent::press(KeyCode::Num1).with_timestamp(100_000),
        &lookup,
        &mut state,
    );

    // Wait until 301ms (past 200ms threshold + 100ms start)
    check_tap_hold_timeouts(301_000, &mut state);

    assert!(
        state.is_modifier_active(4),
        "MD_04 should be active after timeout"
    );

    // Press S
    let outputs = process_event(
        KeyEvent::press(KeyCode::S).with_timestamp(350_000),
        &lookup,
        &mut state,
    );

    let has_ctrl = outputs
        .iter()
        .any(|e| e.keycode() == KeyCode::LCtrl && e.is_press());
    let has_a = outputs
        .iter()
        .any(|e| e.keycode() == KeyCode::A && e.is_press());

    assert!(has_ctrl, "Output should contain LCtrl press");
    assert!(has_a, "Output should contain A press");
}

/// Scenario: Rapid repeating of the combo.
/// "not effective sometimes, but after several key input, it works"
/// This suggests a state desync that clears up.
#[test]
fn test_repro_rapid_combo_repetition() {
    let config = create_config(vec![
        KeyMapping::tap_hold(KeyCode::Num1, KeyCode::Num1, 4, 200),
        KeyMapping::conditional(
            Condition::ModifierActive(4),
            vec![BaseKeyMapping::ModifiedOutput {
                from: KeyCode::S,
                to: KeyCode::A,
                shift: false,
                ctrl: true,
                alt: false,
                win: false,
            }],
        ),
    ]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    let mut time = 100_000;

    for i in 0..5 {
        // Press 1
        process_event(
            KeyEvent::press(KeyCode::Num1).with_timestamp(time),
            &lookup,
            &mut state,
        );
        time += 50_000;

        // Press S (Permissive hold)
        let outputs = process_event(
            KeyEvent::press(KeyCode::S).with_timestamp(time),
            &lookup,
            &mut state,
        );

        let has_ctrl = outputs
            .iter()
            .any(|e| e.keycode() == KeyCode::LCtrl && e.is_press());
        let has_a = outputs
            .iter()
            .any(|e| e.keycode() == KeyCode::A && e.is_press());

        assert!(has_ctrl, "Iter {}: Missing Ctrl", i);
        assert!(has_a, "Iter {}: Missing A", i);

        time += 50_000;

        // Release S
        process_event(
            KeyEvent::release(KeyCode::S).with_timestamp(time),
            &lookup,
            &mut state,
        );
        time += 50_000;

        // Release 1
        process_event(
            KeyEvent::release(KeyCode::Num1).with_timestamp(time),
            &lookup,
            &mut state,
        );
        time += 50_000;

        // Ensure state is clean
        assert!(!state.is_modifier_active(4), "Iter {}: Modifier stuck", i);
    }
}

/// Regression test for: tap_hold modifier becomes sticky with rapid input
/// User report: "with rapid input MD=02 + AOEU rapidly input, then it ended up shift toggle"
/// Expected: MD_02 deactivates when M is released
/// Bug: MD_02 stays active (sticky) after M release with rapid typing
#[test]
fn test_repro_tap_hold_sticky_with_rapid_input() {
    // Config: M is Tap: Backspace, Hold: MD_02
    let config = create_config(vec![KeyMapping::tap_hold(
        KeyCode::M,
        KeyCode::Backspace,
        2,
        200,
    )]);
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

/// Test press/release consistency for ModifiedOutput (e.g., Shift+Key)
/// This ensures that when a key is pressed on one layer and released after layer changes,
/// all output keys (including modifiers) are properly released.
#[test]
fn test_press_release_consistency_modified_output() {
    let config = create_config(vec![
        // M: Tap=Backspace, Hold=MD_02
        KeyMapping::tap_hold(KeyCode::M, KeyCode::Backspace, 2, 200),
        // When MD_02 active: E -> Shift+Num9 (produces `)`)
        KeyMapping::conditional(
            Condition::ModifierActive(2),
            vec![BaseKeyMapping::ModifiedOutput {
                from: KeyCode::E,
                to: KeyCode::Num9,
                shift: true,
                ctrl: false,
                alt: false,
                win: false,
            }],
        ),
    ]);
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Press M at t=0
    process_event(
        KeyEvent::press(KeyCode::M).with_timestamp(0),
        &lookup,
        &mut state,
    );

    // Press E at t=50ms (triggers permissive hold)
    let press_outputs = process_event(
        KeyEvent::press(KeyCode::E).with_timestamp(50_000),
        &lookup,
        &mut state,
    );

    // Verify MD_02 is active
    assert!(
        state.is_modifier_active(2),
        "MD_02 should be active via permissive hold"
    );

    // Verify press output is [LShift press, Num9 press]
    assert_eq!(press_outputs.len(), 2, "Should output 2 keys (Shift+Num9)");
    assert!(
        press_outputs[0].keycode() == KeyCode::LShift && press_outputs[0].is_press(),
        "First output should be LShift press"
    );
    assert!(
        press_outputs[1].keycode() == KeyCode::Num9 && press_outputs[1].is_press(),
        "Second output should be Num9 press"
    );

    // Release M at t=100ms (MD_02 deactivates, layer changes)
    process_event(
        KeyEvent::release(KeyCode::M).with_timestamp(100_000),
        &lookup,
        &mut state,
    );

    // Verify MD_02 is now inactive
    assert!(
        !state.is_modifier_active(2),
        "MD_02 should be inactive after releasing M"
    );

    // **CRITICAL**: Release E at t=150ms
    // Even though MD_02 is now inactive, we should release what we pressed: [Num9, LShift]
    let release_outputs = process_event(
        KeyEvent::release(KeyCode::E).with_timestamp(150_000),
        &lookup,
        &mut state,
    );

    // Verify release output is [Num9 release, LShift release] (in reverse order)
    assert_eq!(
        release_outputs.len(),
        2,
        "Should release 2 keys (Num9, LShift)"
    );
    assert!(
        release_outputs[0].keycode() == KeyCode::Num9 && !release_outputs[0].is_press(),
        "First release should be Num9"
    );
    assert!(
        release_outputs[1].keycode() == KeyCode::LShift && !release_outputs[1].is_press(),
        "Second release should be LShift"
    );
}
