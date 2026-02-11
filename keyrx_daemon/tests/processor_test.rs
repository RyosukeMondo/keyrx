//! Integration tests for EventProcessor.
//!
//! These tests verify the complete event processing pipeline from config
//! loading through event output, using the mock platform implementations.

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

#[test]
fn test_end_to_end_simple_remap() {
    // Create config with simple A→B mapping
    let config = create_test_config(vec![KeyMapping::simple(KeyCode::A, KeyCode::B)]);

    // Create input with Press(A), Release(A)
    let input = MockInput::new(vec![
        KeyEvent::Press(KeyCode::A),
        KeyEvent::Release(KeyCode::A),
    ]);
    let output = MockOutput::new();

    // Create processor and run
    let mut processor = EventProcessor::new(&config, input, output);
    processor.run().unwrap();

    // Verify output: [Press(B), Release(B)]
    let events = processor.output().events();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0], KeyEvent::Press(KeyCode::B));
    assert_eq!(events[1], KeyEvent::Release(KeyCode::B));
}

#[test]
fn test_end_to_end_passthrough() {
    // Create empty config (no mappings)
    let config = create_test_config(vec![]);

    // Create input with unmapped key
    let input = MockInput::new(vec![
        KeyEvent::Press(KeyCode::X),
        KeyEvent::Release(KeyCode::X),
    ]);
    let output = MockOutput::new();

    // Create processor and run
    let mut processor = EventProcessor::new(&config, input, output);
    processor.run().unwrap();

    // Verify passthrough: original events returned unchanged
    let events = processor.output().events();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0], KeyEvent::Press(KeyCode::X));
    assert_eq!(events[1], KeyEvent::Release(KeyCode::X));
}

#[test]
fn test_conditional_with_modifier() {
    // Create config:
    // 1. CapsLock → MD_00 (modifier)
    // 2. When MD_00 active: H → Left
    let config = create_test_config(vec![
        KeyMapping::modifier(KeyCode::CapsLock, 0),
        KeyMapping::conditional(
            Condition::AllActive(vec![ConditionItem::ModifierActive(0)]),
            vec![BaseKeyMapping::Simple {
                from: KeyCode::H,
                to: KeyCode::Left,
            }],
        ),
    ]);

    // Event sequence:
    // 1. Press CapsLock (activates MD_00)
    // 2. Press H (should map to Left because MD_00 active)
    // 3. Release H
    // 4. Release CapsLock (deactivates MD_00)
    // 5. Press H (should passthrough because MD_00 inactive)
    // 6. Release H
    let input = MockInput::new(vec![
        KeyEvent::Press(KeyCode::CapsLock),
        KeyEvent::Press(KeyCode::H),
        KeyEvent::Release(KeyCode::H),
        KeyEvent::Release(KeyCode::CapsLock),
        KeyEvent::Press(KeyCode::H),
        KeyEvent::Release(KeyCode::H),
    ]);
    let output = MockOutput::new();

    let mut processor = EventProcessor::new(&config, input, output);
    processor.run().unwrap();

    // Verify output:
    // CapsLock press: no output (modifier)
    // H press (with MD_00): Left press
    // H release (with MD_00): Left release
    // CapsLock release: no output (modifier)
    // H press (without MD_00): H press (passthrough)
    // H release (without MD_00): H release (passthrough)
    let events = processor.output().events();
    assert_eq!(events.len(), 4); // Only 4 output events (2 for Left, 2 for H passthrough)
    assert_eq!(events[0], KeyEvent::Press(KeyCode::Left));
    assert_eq!(events[1], KeyEvent::Release(KeyCode::Left));
    assert_eq!(events[2], KeyEvent::Press(KeyCode::H));
    assert_eq!(events[3], KeyEvent::Release(KeyCode::H));
}

#[test]
fn test_lock_toggle() {
    // Create config:
    // 1. ScrollLock → LK_01 (lock)
    // 2. When LK_01 active: J → Down
    let config = create_test_config(vec![
        KeyMapping::lock(KeyCode::ScrollLock, 1),
        KeyMapping::conditional(
            Condition::AllActive(vec![ConditionItem::LockActive(1)]),
            vec![BaseKeyMapping::Simple {
                from: KeyCode::J,
                to: KeyCode::Down,
            }],
        ),
    ]);

    // Event sequence:
    // 1. Press ScrollLock (toggles LK_01 ON)
    // 2. Release ScrollLock
    // 3. Press J (should map to Down because LK_01 active)
    // 4. Release J
    // 5. Press ScrollLock (toggles LK_01 OFF)
    // 6. Release ScrollLock
    // 7. Press J (should passthrough because LK_01 inactive)
    // 8. Release J
    let input = MockInput::new(vec![
        KeyEvent::Press(KeyCode::ScrollLock),
        KeyEvent::Release(KeyCode::ScrollLock),
        KeyEvent::Press(KeyCode::J),
        KeyEvent::Release(KeyCode::J),
        KeyEvent::Press(KeyCode::ScrollLock),
        KeyEvent::Release(KeyCode::ScrollLock),
        KeyEvent::Press(KeyCode::J),
        KeyEvent::Release(KeyCode::J),
    ]);
    let output = MockOutput::new();

    let mut processor = EventProcessor::new(&config, input, output);
    processor.run().unwrap();

    // Verify output:
    // ScrollLock press: no output (toggles ON)
    // ScrollLock release: no output
    // J press (with LK_01): Down press
    // J release (with LK_01): Down release
    // ScrollLock press: no output (toggles OFF)
    // ScrollLock release: no output
    // J press (without LK_01): J press (passthrough)
    // J release (without LK_01): J release (passthrough)
    let events = processor.output().events();
    assert_eq!(events.len(), 4); // Only 4 output events (2 for Down, 2 for J passthrough)
    assert_eq!(events[0], KeyEvent::Press(KeyCode::Down));
    assert_eq!(events[1], KeyEvent::Release(KeyCode::Down));
    assert_eq!(events[2], KeyEvent::Press(KeyCode::J));
    assert_eq!(events[3], KeyEvent::Release(KeyCode::J));
}

#[test]
fn test_end_of_stream_handling() {
    // Create simple config
    let config = create_test_config(vec![KeyMapping::simple(KeyCode::A, KeyCode::B)]);

    // Create input with limited events
    let input = MockInput::new(vec![
        KeyEvent::Press(KeyCode::A),
        KeyEvent::Release(KeyCode::A),
    ]);
    let output = MockOutput::new();

    let mut processor = EventProcessor::new(&config, input, output);

    // run() should return Ok(()) when reaching EndOfStream
    let result = processor.run();
    assert!(result.is_ok());

    // Verify events were processed before EndOfStream
    assert_eq!(processor.output().events().len(), 2);
}

#[test]
fn test_multiple_conditional_mappings() {
    // Create config with multiple conditionals:
    // 1. CapsLock → MD_00
    // 2. When MD_00: H → Left
    // 3. When MD_00: J → Down
    // 4. When MD_00: K → Up
    // 5. When MD_00: L → Right
    let config = create_test_config(vec![
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
    ]);

    // Event sequence: CapsLock, H, J, K, L navigation
    let input = MockInput::new(vec![
        KeyEvent::Press(KeyCode::CapsLock),
        KeyEvent::Press(KeyCode::H),
        KeyEvent::Release(KeyCode::H),
        KeyEvent::Press(KeyCode::J),
        KeyEvent::Release(KeyCode::J),
        KeyEvent::Press(KeyCode::K),
        KeyEvent::Release(KeyCode::K),
        KeyEvent::Press(KeyCode::L),
        KeyEvent::Release(KeyCode::L),
        KeyEvent::Release(KeyCode::CapsLock),
    ]);
    let output = MockOutput::new();

    let mut processor = EventProcessor::new(&config, input, output);
    processor.run().unwrap();

    // Verify navigation mappings: Left, Down, Up, Right
    let events = processor.output().events();
    assert_eq!(events.len(), 8);
    assert_eq!(events[0], KeyEvent::Press(KeyCode::Left));
    assert_eq!(events[1], KeyEvent::Release(KeyCode::Left));
    assert_eq!(events[2], KeyEvent::Press(KeyCode::Down));
    assert_eq!(events[3], KeyEvent::Release(KeyCode::Down));
    assert_eq!(events[4], KeyEvent::Press(KeyCode::Up));
    assert_eq!(events[5], KeyEvent::Release(KeyCode::Up));
    assert_eq!(events[6], KeyEvent::Press(KeyCode::Right));
    assert_eq!(events[7], KeyEvent::Release(KeyCode::Right));
}

#[test]
fn test_modified_output_sequence() {
    // Create config: 1 → Shift+1 (exclamation mark)
    let config = create_test_config(vec![KeyMapping::modified_output(
        KeyCode::Num1,
        KeyCode::Num1,
        true,  // shift
        false, // ctrl
        false, // alt
        false, // win
    )]);

    // Press and release 1
    let input = MockInput::new(vec![
        KeyEvent::Press(KeyCode::Num1),
        KeyEvent::Release(KeyCode::Num1),
    ]);
    let output = MockOutput::new();

    let mut processor = EventProcessor::new(&config, input, output);
    processor.run().unwrap();

    // Verify output sequence:
    // Press: Shift press, 1 press
    // Release: 1 release, Shift release (reverse order)
    let events = processor.output().events();
    assert_eq!(events.len(), 4);
    assert_eq!(events[0], KeyEvent::Press(KeyCode::LShift));
    assert_eq!(events[1], KeyEvent::Press(KeyCode::Num1));
    assert_eq!(events[2], KeyEvent::Release(KeyCode::Num1));
    assert_eq!(events[3], KeyEvent::Release(KeyCode::LShift));
}

#[test]
fn test_mixed_simple_and_conditional() {
    // Create config with both simple and conditional mappings:
    // 1. A → B (simple, always active)
    // 2. CapsLock → MD_00 (modifier)
    // 3. When MD_00: A → C (conditional, overrides simple)
    let config = create_test_config(vec![
        KeyMapping::simple(KeyCode::A, KeyCode::B),
        KeyMapping::modifier(KeyCode::CapsLock, 0),
        KeyMapping::conditional(
            Condition::AllActive(vec![ConditionItem::ModifierActive(0)]),
            vec![BaseKeyMapping::Simple {
                from: KeyCode::A,
                to: KeyCode::C,
            }],
        ),
    ]);

    // Event sequence:
    // 1. Press A (should map to B, MD_00 inactive)
    // 2. Release A
    // 3. Press CapsLock (activates MD_00)
    // 4. Press A (should map to C, MD_00 active)
    // 5. Release A
    // 6. Release CapsLock (deactivates MD_00)
    let input = MockInput::new(vec![
        KeyEvent::Press(KeyCode::A),
        KeyEvent::Release(KeyCode::A),
        KeyEvent::Press(KeyCode::CapsLock),
        KeyEvent::Press(KeyCode::A),
        KeyEvent::Release(KeyCode::A),
        KeyEvent::Release(KeyCode::CapsLock),
    ]);
    let output = MockOutput::new();

    let mut processor = EventProcessor::new(&config, input, output);
    processor.run().unwrap();

    // Verify output:
    // A press (no modifier): B press
    // A release: B release
    // CapsLock press: no output
    // A press (with modifier): C press
    // A release: C release
    // CapsLock release: no output
    let events = processor.output().events();
    assert_eq!(events.len(), 4);
    assert_eq!(events[0], KeyEvent::Press(KeyCode::B));
    assert_eq!(events[1], KeyEvent::Release(KeyCode::B));
    assert_eq!(events[2], KeyEvent::Press(KeyCode::C));
    assert_eq!(events[3], KeyEvent::Release(KeyCode::C));
}

#[test]
fn test_empty_input_stream() {
    // Create config with simple mapping
    let config = create_test_config(vec![KeyMapping::simple(KeyCode::A, KeyCode::B)]);

    // Create empty input stream
    let input = MockInput::new(vec![]);
    let output = MockOutput::new();

    let mut processor = EventProcessor::new(&config, input, output);

    // run() should immediately return Ok for empty stream
    let result = processor.run();
    assert!(result.is_ok());

    // No events should be processed
    assert_eq!(processor.output().events().len(), 0);
}

#[test]
fn test_complex_vim_navigation_layer() {
    // Realistic Vim-style navigation layer:
    // CapsLock → MD_00 (navigation layer)
    // When MD_00:
    //   H → Left
    //   J → Down
    //   K → Up
    //   L → Right
    //   W → Ctrl+Right (word forward)
    //   B → Ctrl+Left (word backward)
    let config = create_test_config(vec![
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

    // Simulate: Hold CapsLock, navigate with H/J/K/L, use W for word forward
    let input = MockInput::new(vec![
        KeyEvent::Press(KeyCode::CapsLock), // Activate layer
        KeyEvent::Press(KeyCode::H),        // Left
        KeyEvent::Release(KeyCode::H),
        KeyEvent::Press(KeyCode::J), // Down
        KeyEvent::Release(KeyCode::J),
        KeyEvent::Press(KeyCode::W), // Ctrl+Right (word forward)
        KeyEvent::Release(KeyCode::W),
        KeyEvent::Release(KeyCode::CapsLock), // Deactivate layer
    ]);
    let output = MockOutput::new();

    let mut processor = EventProcessor::new(&config, input, output);
    processor.run().unwrap();

    // Verify output:
    // CapsLock: no output
    // H: Left press/release
    // J: Down press/release
    // W: Ctrl press, Right press, Right release, Ctrl release
    // CapsLock release: no output
    let events = processor.output().events();
    assert_eq!(events.len(), 8);
    // H → Left
    assert_eq!(events[0], KeyEvent::Press(KeyCode::Left));
    assert_eq!(events[1], KeyEvent::Release(KeyCode::Left));
    // J → Down
    assert_eq!(events[2], KeyEvent::Press(KeyCode::Down));
    assert_eq!(events[3], KeyEvent::Release(KeyCode::Down));
    // W → Ctrl+Right
    assert_eq!(events[4], KeyEvent::Press(KeyCode::LCtrl));
    assert_eq!(events[5], KeyEvent::Press(KeyCode::Right));
    assert_eq!(events[6], KeyEvent::Release(KeyCode::Right));
    assert_eq!(events[7], KeyEvent::Release(KeyCode::LCtrl));
}
