//! End-to-End Integration Tests
//!
//! These tests validate complete system behavior with realistic user scenarios:
//! - Vim-style navigation layers with complex key combinations
//! - Lock state persistence across multiple key presses
//! - Multi-device configurations with independent state management
//!
//! Unlike processor_test.rs which tests individual features, these tests
//! simulate real-world usage patterns and workflows.

use keyrx_core::config::{
    mappings::{BaseKeyMapping, DeviceConfig, DeviceIdentifier, KeyMapping},
    Condition, ConditionItem, KeyCode,
};
use keyrx_core::runtime::KeyEvent;
use keyrx_daemon::platform::{MockInput, MockOutput};
use keyrx_daemon::processor::EventProcessor;

extern crate alloc;
use alloc::string::String;

/// Helper to create a test device config
fn create_test_config(mappings: Vec<KeyMapping>) -> DeviceConfig {
    DeviceConfig {
        identifier: DeviceIdentifier {
            pattern: String::from("*"),
        },
        mappings,
    }
}

/// Test: Vim navigation layer with complete HJKL movement and word navigation
///
/// This test simulates a realistic Vim-style navigation setup where:
/// - CapsLock acts as a layer modifier (MD_00)
/// - H/J/K/L map to arrow keys when layer is active
/// - W maps to Ctrl+Right (word forward)
/// - B maps to Ctrl+Left (word backward)
/// - Keys work normally when layer is inactive
#[test]
fn test_vim_navigation_layer() {
    // Create comprehensive Vim navigation config
    let config = create_test_config(vec![
        // CapsLock → MD_00 (navigation layer modifier)
        KeyMapping::modifier(KeyCode::CapsLock, 0),
        // Navigation mappings (active when MD_00 is held)
        KeyMapping::conditional(
            Condition::AllActive(vec![ConditionItem::ModifierActive(0)]),
            vec![BaseKeyMapping::Simple {
                from: KeyCode::H,
                to: KeyCode::Left,
            }],
        ),
        KeyMapping::conditional(
            Condition::AllActive(vec![ConditionItem::ModifierActive(0)]),
            vec![BaseKeyMapping::Simple {
                from: KeyCode::J,
                to: KeyCode::Down,
            }],
        ),
        KeyMapping::conditional(
            Condition::AllActive(vec![ConditionItem::ModifierActive(0)]),
            vec![BaseKeyMapping::Simple {
                from: KeyCode::K,
                to: KeyCode::Up,
            }],
        ),
        KeyMapping::conditional(
            Condition::AllActive(vec![ConditionItem::ModifierActive(0)]),
            vec![BaseKeyMapping::Simple {
                from: KeyCode::L,
                to: KeyCode::Right,
            }],
        ),
        // Word navigation
        KeyMapping::conditional(
            Condition::AllActive(vec![ConditionItem::ModifierActive(0)]),
            vec![BaseKeyMapping::ModifiedOutput {
                from: KeyCode::W,
                to: KeyCode::Right,
                shift: false,
                ctrl: true,
                alt: false,
                win: false,
            }],
        ),
        KeyMapping::conditional(
            Condition::AllActive(vec![ConditionItem::ModifierActive(0)]),
            vec![BaseKeyMapping::ModifiedOutput {
                from: KeyCode::B,
                to: KeyCode::Left,
                shift: false,
                ctrl: true,
                alt: false,
                win: false,
            }],
        ),
    ]);

    // Simulate realistic editing workflow:
    // 1. Type "hello" normally
    // 2. Activate layer (hold CapsLock)
    // 3. Navigate: left (H), down (J), up (K), right (L)
    // 4. Jump word backward (B), forward (W)
    // 5. Deactivate layer (release CapsLock)
    // 6. Type normally again
    let input = MockInput::new(vec![
        // Type "hello" (should pass through - no layer active)
        KeyEvent::Press(KeyCode::H),
        KeyEvent::Release(KeyCode::H),
        // Activate navigation layer
        KeyEvent::Press(KeyCode::CapsLock),
        // Navigate with HJKL (should map to arrows)
        KeyEvent::Press(KeyCode::H),
        KeyEvent::Release(KeyCode::H),
        KeyEvent::Press(KeyCode::J),
        KeyEvent::Release(KeyCode::J),
        KeyEvent::Press(KeyCode::K),
        KeyEvent::Release(KeyCode::K),
        KeyEvent::Press(KeyCode::L),
        KeyEvent::Release(KeyCode::L),
        // Word navigation (should map to Ctrl+Left/Right)
        KeyEvent::Press(KeyCode::B),
        KeyEvent::Release(KeyCode::B),
        KeyEvent::Press(KeyCode::W),
        KeyEvent::Release(KeyCode::W),
        // Deactivate layer
        KeyEvent::Release(KeyCode::CapsLock),
        // Type "h" normally again (should pass through)
        KeyEvent::Press(KeyCode::H),
        KeyEvent::Release(KeyCode::H),
    ]);
    let output = MockOutput::new();

    let mut processor = EventProcessor::new(&config, input, output);
    processor.run().unwrap();

    // Verify output sequence:
    let events = processor.output().events();

    // H (before layer): passthrough
    assert_eq!(events[0], KeyEvent::Press(KeyCode::H));
    assert_eq!(events[1], KeyEvent::Release(KeyCode::H));

    // CapsLock: no output (modifier)

    // H (with layer): Left
    assert_eq!(events[2], KeyEvent::Press(KeyCode::Left));
    assert_eq!(events[3], KeyEvent::Release(KeyCode::Left));

    // J: Down
    assert_eq!(events[4], KeyEvent::Press(KeyCode::Down));
    assert_eq!(events[5], KeyEvent::Release(KeyCode::Down));

    // K: Up
    assert_eq!(events[6], KeyEvent::Press(KeyCode::Up));
    assert_eq!(events[7], KeyEvent::Release(KeyCode::Up));

    // L: Right
    assert_eq!(events[8], KeyEvent::Press(KeyCode::Right));
    assert_eq!(events[9], KeyEvent::Release(KeyCode::Right));

    // B: Ctrl+Left (word backward)
    assert_eq!(events[10], KeyEvent::Press(KeyCode::LCtrl));
    assert_eq!(events[11], KeyEvent::Press(KeyCode::Left));
    assert_eq!(events[12], KeyEvent::Release(KeyCode::Left));
    assert_eq!(events[13], KeyEvent::Release(KeyCode::LCtrl));

    // W: Ctrl+Right (word forward)
    assert_eq!(events[14], KeyEvent::Press(KeyCode::LCtrl));
    assert_eq!(events[15], KeyEvent::Press(KeyCode::Right));
    assert_eq!(events[16], KeyEvent::Release(KeyCode::Right));
    assert_eq!(events[17], KeyEvent::Release(KeyCode::LCtrl));

    // CapsLock release: no output

    // H (after layer): passthrough
    assert_eq!(events[18], KeyEvent::Press(KeyCode::H));
    assert_eq!(events[19], KeyEvent::Release(KeyCode::H));

    assert_eq!(events.len(), 20);
}

/// Test: Lock persistence - verify lock state persists across multiple keypresses
///
/// This test validates that lock toggles (like NumLock/ScrollLock) maintain their
/// state correctly and affect subsequent key presses until toggled again.
#[test]
fn test_lock_persistence() {
    // Create config with ScrollLock as layer lock (LK_01)
    // When locked: number row (1-5) maps to F1-F5
    let config = create_test_config(vec![
        // ScrollLock → LK_01 (layer lock)
        KeyMapping::lock(KeyCode::ScrollLock, 1),
        // Number row mappings when lock active
        KeyMapping::conditional(
            Condition::AllActive(vec![ConditionItem::LockActive(1)]),
            vec![BaseKeyMapping::Simple {
                from: KeyCode::Num1,
                to: KeyCode::F1,
            }],
        ),
        KeyMapping::conditional(
            Condition::AllActive(vec![ConditionItem::LockActive(1)]),
            vec![BaseKeyMapping::Simple {
                from: KeyCode::Num2,
                to: KeyCode::F2,
            }],
        ),
        KeyMapping::conditional(
            Condition::AllActive(vec![ConditionItem::LockActive(1)]),
            vec![BaseKeyMapping::Simple {
                from: KeyCode::Num3,
                to: KeyCode::F3,
            }],
        ),
        KeyMapping::conditional(
            Condition::AllActive(vec![ConditionItem::LockActive(1)]),
            vec![BaseKeyMapping::Simple {
                from: KeyCode::Num4,
                to: KeyCode::F4,
            }],
        ),
        KeyMapping::conditional(
            Condition::AllActive(vec![ConditionItem::LockActive(1)]),
            vec![BaseKeyMapping::Simple {
                from: KeyCode::Num5,
                to: KeyCode::F5,
            }],
        ),
    ]);

    // Simulate realistic workflow:
    // 1. Type numbers normally (1, 2, 3)
    // 2. Toggle lock ON (press/release ScrollLock)
    // 3. Type numbers (should map to F1, F2, F3)
    // 4. Type more numbers to verify persistence (4, 5)
    // 5. Toggle lock OFF (press/release ScrollLock)
    // 6. Type numbers normally again (1, 2)
    let input = MockInput::new(vec![
        // Before lock: numbers should pass through
        KeyEvent::Press(KeyCode::Num1),
        KeyEvent::Release(KeyCode::Num1),
        KeyEvent::Press(KeyCode::Num2),
        KeyEvent::Release(KeyCode::Num2),
        KeyEvent::Press(KeyCode::Num3),
        KeyEvent::Release(KeyCode::Num3),
        // Toggle lock ON
        KeyEvent::Press(KeyCode::ScrollLock),
        KeyEvent::Release(KeyCode::ScrollLock),
        // With lock: numbers should map to F-keys
        KeyEvent::Press(KeyCode::Num1),
        KeyEvent::Release(KeyCode::Num1),
        KeyEvent::Press(KeyCode::Num2),
        KeyEvent::Release(KeyCode::Num2),
        KeyEvent::Press(KeyCode::Num3),
        KeyEvent::Release(KeyCode::Num3),
        // Verify persistence: more keypresses with lock still active
        KeyEvent::Press(KeyCode::Num4),
        KeyEvent::Release(KeyCode::Num4),
        KeyEvent::Press(KeyCode::Num5),
        KeyEvent::Release(KeyCode::Num5),
        // Toggle lock OFF
        KeyEvent::Press(KeyCode::ScrollLock),
        KeyEvent::Release(KeyCode::ScrollLock),
        // After lock: numbers should pass through again
        KeyEvent::Press(KeyCode::Num1),
        KeyEvent::Release(KeyCode::Num1),
        KeyEvent::Press(KeyCode::Num2),
        KeyEvent::Release(KeyCode::Num2),
    ]);
    let output = MockOutput::new();

    let mut processor = EventProcessor::new(&config, input, output);
    processor.run().unwrap();

    let events = processor.output().events();

    // Before lock: passthrough (1, 2, 3)
    assert_eq!(events[0], KeyEvent::Press(KeyCode::Num1));
    assert_eq!(events[1], KeyEvent::Release(KeyCode::Num1));
    assert_eq!(events[2], KeyEvent::Press(KeyCode::Num2));
    assert_eq!(events[3], KeyEvent::Release(KeyCode::Num2));
    assert_eq!(events[4], KeyEvent::Press(KeyCode::Num3));
    assert_eq!(events[5], KeyEvent::Release(KeyCode::Num3));

    // ScrollLock toggle ON: no output

    // With lock: mapped to F-keys (F1, F2, F3)
    assert_eq!(events[6], KeyEvent::Press(KeyCode::F1));
    assert_eq!(events[7], KeyEvent::Release(KeyCode::F1));
    assert_eq!(events[8], KeyEvent::Press(KeyCode::F2));
    assert_eq!(events[9], KeyEvent::Release(KeyCode::F2));
    assert_eq!(events[10], KeyEvent::Press(KeyCode::F3));
    assert_eq!(events[11], KeyEvent::Release(KeyCode::F3));

    // Lock persists: more F-keys (F4, F5)
    assert_eq!(events[12], KeyEvent::Press(KeyCode::F4));
    assert_eq!(events[13], KeyEvent::Release(KeyCode::F4));
    assert_eq!(events[14], KeyEvent::Press(KeyCode::F5));
    assert_eq!(events[15], KeyEvent::Release(KeyCode::F5));

    // ScrollLock toggle OFF: no output

    // After lock: passthrough again (1, 2)
    assert_eq!(events[16], KeyEvent::Press(KeyCode::Num1));
    assert_eq!(events[17], KeyEvent::Release(KeyCode::Num1));
    assert_eq!(events[18], KeyEvent::Press(KeyCode::Num2));
    assert_eq!(events[19], KeyEvent::Release(KeyCode::Num2));

    assert_eq!(events.len(), 20);
}

/// Test: Multi-device support with independent state
///
/// This test validates that multiple devices can have independent configurations
/// and state. This is critical for users with multiple keyboards or input devices.
#[test]
fn test_multi_device() {
    // Device 1: Gaming keyboard with WASD→Arrow remapping (for left-hand gaming)
    let device1_config = create_test_config(vec![
        KeyMapping::simple(KeyCode::W, KeyCode::Up),
        KeyMapping::simple(KeyCode::A, KeyCode::Left),
        KeyMapping::simple(KeyCode::S, KeyCode::Down),
        KeyMapping::simple(KeyCode::D, KeyCode::Right),
    ]);

    // Device 2: Productivity keyboard with CapsLock navigation layer
    let device2_config = create_test_config(vec![
        KeyMapping::modifier(KeyCode::CapsLock, 0),
        KeyMapping::conditional(
            Condition::AllActive(vec![ConditionItem::ModifierActive(0)]),
            vec![BaseKeyMapping::Simple {
                from: KeyCode::H,
                to: KeyCode::Left,
            }],
        ),
        KeyMapping::conditional(
            Condition::AllActive(vec![ConditionItem::ModifierActive(0)]),
            vec![BaseKeyMapping::Simple {
                from: KeyCode::L,
                to: KeyCode::Right,
            }],
        ),
    ]);

    // Device 1 event sequence: WASD navigation
    let device1_input = MockInput::new(vec![
        KeyEvent::Press(KeyCode::W),
        KeyEvent::Release(KeyCode::W),
        KeyEvent::Press(KeyCode::A),
        KeyEvent::Release(KeyCode::A),
        KeyEvent::Press(KeyCode::S),
        KeyEvent::Release(KeyCode::S),
        KeyEvent::Press(KeyCode::D),
        KeyEvent::Release(KeyCode::D),
    ]);
    let device1_output = MockOutput::new();

    // Device 2 event sequence: CapsLock + HL navigation
    let device2_input = MockInput::new(vec![
        KeyEvent::Press(KeyCode::CapsLock),
        KeyEvent::Press(KeyCode::H),
        KeyEvent::Release(KeyCode::H),
        KeyEvent::Press(KeyCode::L),
        KeyEvent::Release(KeyCode::L),
        KeyEvent::Release(KeyCode::CapsLock),
    ]);
    let device2_output = MockOutput::new();

    // Create independent processors for each device
    let mut processor1 = EventProcessor::new(&device1_config, device1_input, device1_output);
    let mut processor2 = EventProcessor::new(&device2_config, device2_input, device2_output);

    // Process both devices independently
    processor1.run().unwrap();
    processor2.run().unwrap();

    // Verify Device 1 output: WASD → Arrow keys
    let device1_events = processor1.output().events();
    assert_eq!(device1_events.len(), 8);
    assert_eq!(device1_events[0], KeyEvent::Press(KeyCode::Up));
    assert_eq!(device1_events[1], KeyEvent::Release(KeyCode::Up));
    assert_eq!(device1_events[2], KeyEvent::Press(KeyCode::Left));
    assert_eq!(device1_events[3], KeyEvent::Release(KeyCode::Left));
    assert_eq!(device1_events[4], KeyEvent::Press(KeyCode::Down));
    assert_eq!(device1_events[5], KeyEvent::Release(KeyCode::Down));
    assert_eq!(device1_events[6], KeyEvent::Press(KeyCode::Right));
    assert_eq!(device1_events[7], KeyEvent::Release(KeyCode::Right));

    // Verify Device 2 output: CapsLock + HL → Left/Right (with layer)
    let device2_events = processor2.output().events();
    assert_eq!(device2_events.len(), 4);
    // CapsLock: no output (modifier)
    assert_eq!(device2_events[0], KeyEvent::Press(KeyCode::Left));
    assert_eq!(device2_events[1], KeyEvent::Release(KeyCode::Left));
    assert_eq!(device2_events[2], KeyEvent::Press(KeyCode::Right));
    assert_eq!(device2_events[3], KeyEvent::Release(KeyCode::Right));
    // CapsLock release: no output
}

/// Test: Complex multi-layer scenario with modifier + lock combination
///
/// This test validates that modifiers and locks can work together correctly,
/// enabling complex layering scenarios.
///
/// IMPORTANT: Conditional mappings are evaluated in registration order.
/// More specific conditions (with more requirements) must be registered FIRST
/// to take precedence over less specific ones.
#[test]
fn test_complex_multilayer() {
    // Config with both modifier and lock:
    // - CapsLock → MD_00 (temporary layer while held)
    // - ScrollLock → LK_01 (persistent layer when toggled)
    // - When MD_00 AND LK_01: A→3 (MOST SPECIFIC - registered first)
    // - When MD_00: A→1
    // - When LK_01: A→2
    let config = create_test_config(vec![
        KeyMapping::modifier(KeyCode::CapsLock, 0),
        KeyMapping::lock(KeyCode::ScrollLock, 1),
        // MD_00 AND LK_01: A→3 (MOST SPECIFIC - must be first!)
        // This will match when both conditions are true
        KeyMapping::conditional(
            Condition::AllActive(vec![
                ConditionItem::ModifierActive(0),
                ConditionItem::LockActive(1),
            ]),
            vec![BaseKeyMapping::Simple {
                from: KeyCode::A,
                to: KeyCode::Num3,
            }],
        ),
        // MD_00 only: A→1
        KeyMapping::conditional(
            Condition::AllActive(vec![ConditionItem::ModifierActive(0)]),
            vec![BaseKeyMapping::Simple {
                from: KeyCode::A,
                to: KeyCode::Num1,
            }],
        ),
        // LK_01 only: A→2
        KeyMapping::conditional(
            Condition::AllActive(vec![ConditionItem::LockActive(1)]),
            vec![BaseKeyMapping::Simple {
                from: KeyCode::A,
                to: KeyCode::Num2,
            }],
        ),
    ]);

    // Event sequence testing all combinations:
    // 1. Press A (no layers): passthrough
    // 2. Hold CapsLock, press A (MD_00 only): 1
    // 3. Release CapsLock
    // 4. Toggle ScrollLock ON
    // 5. Press A (LK_01 only): 2
    // 6. Hold CapsLock, press A (MD_00 AND LK_01): 3 (most specific wins)
    // 7. Release CapsLock
    // 8. Toggle ScrollLock OFF
    // 9. Press A (no layers): passthrough
    let input = MockInput::new(vec![
        // No layers
        KeyEvent::Press(KeyCode::A),
        KeyEvent::Release(KeyCode::A),
        // MD_00 only
        KeyEvent::Press(KeyCode::CapsLock),
        KeyEvent::Press(KeyCode::A),
        KeyEvent::Release(KeyCode::A),
        KeyEvent::Release(KeyCode::CapsLock),
        // Toggle LK_01 ON
        KeyEvent::Press(KeyCode::ScrollLock),
        KeyEvent::Release(KeyCode::ScrollLock),
        // LK_01 only
        KeyEvent::Press(KeyCode::A),
        KeyEvent::Release(KeyCode::A),
        // MD_00 AND LK_01
        KeyEvent::Press(KeyCode::CapsLock),
        KeyEvent::Press(KeyCode::A),
        KeyEvent::Release(KeyCode::A),
        KeyEvent::Release(KeyCode::CapsLock),
        // Toggle LK_01 OFF
        KeyEvent::Press(KeyCode::ScrollLock),
        KeyEvent::Release(KeyCode::ScrollLock),
        // No layers again
        KeyEvent::Press(KeyCode::A),
        KeyEvent::Release(KeyCode::A),
    ]);
    let output = MockOutput::new();

    let mut processor = EventProcessor::new(&config, input, output);
    processor.run().unwrap();

    let events = processor.output().events();

    // No layers: A passthrough
    assert_eq!(events[0], KeyEvent::Press(KeyCode::A));
    assert_eq!(events[1], KeyEvent::Release(KeyCode::A));

    // MD_00 only: A→1
    assert_eq!(events[2], KeyEvent::Press(KeyCode::Num1));
    assert_eq!(events[3], KeyEvent::Release(KeyCode::Num1));

    // LK_01 only: A→2
    assert_eq!(events[4], KeyEvent::Press(KeyCode::Num2));
    assert_eq!(events[5], KeyEvent::Release(KeyCode::Num2));

    // MD_00 AND LK_01: A→3 (most specific condition matches first)
    assert_eq!(events[6], KeyEvent::Press(KeyCode::Num3));
    assert_eq!(events[7], KeyEvent::Release(KeyCode::Num3));

    // No layers: A passthrough
    assert_eq!(events[8], KeyEvent::Press(KeyCode::A));
    assert_eq!(events[9], KeyEvent::Release(KeyCode::A));

    assert_eq!(events.len(), 10);
}

/// Test: Rapid toggling of lock state
///
/// Validates that lock state can be toggled rapidly without errors or state corruption.
#[test]
fn test_rapid_lock_toggling() {
    // Simple config: ScrollLock → LK_01, when LK_01: A→B
    let config = create_test_config(vec![
        KeyMapping::lock(KeyCode::ScrollLock, 1),
        KeyMapping::conditional(
            Condition::AllActive(vec![ConditionItem::LockActive(1)]),
            vec![BaseKeyMapping::Simple {
                from: KeyCode::A,
                to: KeyCode::B,
            }],
        ),
    ]);

    // Rapidly toggle lock multiple times with key presses in between
    let input = MockInput::new(vec![
        // Toggle ON
        KeyEvent::Press(KeyCode::ScrollLock),
        KeyEvent::Release(KeyCode::ScrollLock),
        KeyEvent::Press(KeyCode::A), // Should map to B
        KeyEvent::Release(KeyCode::A),
        // Toggle OFF
        KeyEvent::Press(KeyCode::ScrollLock),
        KeyEvent::Release(KeyCode::ScrollLock),
        KeyEvent::Press(KeyCode::A), // Should pass through
        KeyEvent::Release(KeyCode::A),
        // Toggle ON
        KeyEvent::Press(KeyCode::ScrollLock),
        KeyEvent::Release(KeyCode::ScrollLock),
        KeyEvent::Press(KeyCode::A), // Should map to B
        KeyEvent::Release(KeyCode::A),
        // Toggle OFF
        KeyEvent::Press(KeyCode::ScrollLock),
        KeyEvent::Release(KeyCode::ScrollLock),
        KeyEvent::Press(KeyCode::A), // Should pass through
        KeyEvent::Release(KeyCode::A),
        // Toggle ON
        KeyEvent::Press(KeyCode::ScrollLock),
        KeyEvent::Release(KeyCode::ScrollLock),
        KeyEvent::Press(KeyCode::A), // Should map to B
        KeyEvent::Release(KeyCode::A),
    ]);
    let output = MockOutput::new();

    let mut processor = EventProcessor::new(&config, input, output);
    processor.run().unwrap();

    let events = processor.output().events();

    // Verify alternating pattern: B, A, B, A, B
    assert_eq!(events[0], KeyEvent::Press(KeyCode::B));
    assert_eq!(events[1], KeyEvent::Release(KeyCode::B));
    assert_eq!(events[2], KeyEvent::Press(KeyCode::A));
    assert_eq!(events[3], KeyEvent::Release(KeyCode::A));
    assert_eq!(events[4], KeyEvent::Press(KeyCode::B));
    assert_eq!(events[5], KeyEvent::Release(KeyCode::B));
    assert_eq!(events[6], KeyEvent::Press(KeyCode::A));
    assert_eq!(events[7], KeyEvent::Release(KeyCode::A));
    assert_eq!(events[8], KeyEvent::Press(KeyCode::B));
    assert_eq!(events[9], KeyEvent::Release(KeyCode::B));

    assert_eq!(events.len(), 10);
}
