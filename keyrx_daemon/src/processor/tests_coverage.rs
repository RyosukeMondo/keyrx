//! Additional tests to improve coverage of processor module.

use super::*;
use crate::platform::{DeviceError, MockInput, MockOutput};
use keyrx_core::config::{
    mappings::{BaseKeyMapping, KeyMapping},
    Condition, ConditionItem, KeyCode,
};
use keyrx_core::runtime::KeyEvent;

use crate::processor::test_utils::create_test_config;

#[test]
fn test_process_one_input_error_not_end_of_stream() {
    let config = create_test_config(vec![]);
    let input = MockInput::new(vec![]);
    let output = MockOutput::new();

    // MockInput returns EndOfStream, which doesn't log
    let mut processor = EventProcessor::new(&config, input, output);
    let result = processor.process_one();
    assert!(matches!(
        result,
        Err(ProcessorError::Input(DeviceError::EndOfStream))
    ));
}

#[test]
fn test_process_one_output_error() {
    let config = create_test_config(vec![]);
    let input = MockInput::new(vec![KeyEvent::Press(KeyCode::A)]);
    let mut output = MockOutput::new();
    output.set_fail_mode(true);

    let mut processor = EventProcessor::new(&config, input, output);
    let result = processor.process_one();
    assert!(matches!(result, Err(ProcessorError::Output(_))));
}

#[test]
fn test_modifier_activation_transition() {
    let config = create_test_config(vec![KeyMapping::modifier(KeyCode::CapsLock, 5)]);
    let input = MockInput::new(vec![
        KeyEvent::Press(KeyCode::CapsLock),
        KeyEvent::Release(KeyCode::CapsLock),
    ]);
    let output = MockOutput::new();

    let mut processor = EventProcessor::new(&config, input, output);

    // Press activates modifier
    processor.process_one().unwrap();
    assert!(processor.state.is_modifier_active(5));

    // Release deactivates modifier
    processor.process_one().unwrap();
    assert!(!processor.state.is_modifier_active(5));
}

#[test]
fn test_lock_toggle_transition() {
    let config = create_test_config(vec![KeyMapping::lock(KeyCode::ScrollLock, 3)]);
    let input = MockInput::new(vec![
        KeyEvent::Press(KeyCode::ScrollLock),
        KeyEvent::Press(KeyCode::ScrollLock),
    ]);
    let output = MockOutput::new();

    let mut processor = EventProcessor::new(&config, input, output);

    // First press toggles on
    processor.process_one().unwrap();
    assert!(processor.state.is_lock_active(3));

    // Second press toggles off
    processor.process_one().unwrap();
    assert!(!processor.state.is_lock_active(3));
}

#[test]
fn test_conditional_not_active() {
    // When MD_00 active: H → Left, but MD_00 is NOT active
    let config = create_test_config(vec![KeyMapping::conditional(
        Condition::AllActive(vec![ConditionItem::ModifierActive(0)]),
        vec![BaseKeyMapping::Simple {
            from: KeyCode::H,
            to: KeyCode::Left,
        }],
    )]);

    let input = MockInput::new(vec![KeyEvent::Press(KeyCode::H)]);
    let output = MockOutput::new();

    let mut processor = EventProcessor::new(&config, input, output);
    processor.process_one().unwrap();

    // Should passthrough since condition not met
    assert_eq!(processor.output.events().len(), 1);
    assert_eq!(processor.output.events()[0], KeyEvent::Press(KeyCode::H));
}

#[test]
fn test_modified_output_all_modifiers() {
    let config = create_test_config(vec![KeyMapping::modified_output(
        KeyCode::A,
        KeyCode::A,
        true, // shift
        true, // ctrl
        true, // alt
        true, // win
    )]);
    let input = MockInput::new(vec![KeyEvent::Press(KeyCode::A)]);
    let output = MockOutput::new();

    let mut processor = EventProcessor::new(&config, input, output);
    processor.process_one().unwrap();

    // Should produce 5 events: 4 modifiers + key
    assert_eq!(processor.output.events().len(), 5);
    assert_eq!(
        processor.output.events()[0],
        KeyEvent::Press(KeyCode::LShift)
    );
    assert_eq!(
        processor.output.events()[1],
        KeyEvent::Press(KeyCode::LCtrl)
    );
    assert_eq!(processor.output.events()[2], KeyEvent::Press(KeyCode::LAlt));
    assert_eq!(
        processor.output.events()[3],
        KeyEvent::Press(KeyCode::LMeta)
    );
    assert_eq!(processor.output.events()[4], KeyEvent::Press(KeyCode::A));
}

#[test]
fn test_release_events_processed() {
    let config = create_test_config(vec![KeyMapping::simple(KeyCode::A, KeyCode::B)]);
    let input = MockInput::new(vec![
        KeyEvent::Release(KeyCode::A),
        KeyEvent::Release(KeyCode::B),
    ]);
    let output = MockOutput::new();

    let mut processor = EventProcessor::new(&config, input, output);
    processor.run().unwrap();

    assert_eq!(processor.output.events().len(), 2);
    assert_eq!(processor.output.events()[0], KeyEvent::Release(KeyCode::B));
    assert_eq!(processor.output.events()[1], KeyEvent::Release(KeyCode::B));
}

#[test]
fn test_multiple_mappings_priority() {
    // Multiple mappings: B → C, A → B
    // Press A should map to B (not C, even though B → C exists)
    let config = create_test_config(vec![
        KeyMapping::simple(KeyCode::B, KeyCode::C),
        KeyMapping::simple(KeyCode::A, KeyCode::B),
    ]);
    let input = MockInput::new(vec![KeyEvent::Press(KeyCode::A)]);
    let output = MockOutput::new();

    let mut processor = EventProcessor::new(&config, input, output);
    processor.process_one().unwrap();

    assert_eq!(processor.output.events().len(), 1);
    assert_eq!(processor.output.events()[0], KeyEvent::Press(KeyCode::B));
}

#[test]
fn test_output_accessor() {
    let config = create_test_config(vec![]);
    let input = MockInput::new(vec![]);
    let output = MockOutput::new();

    let processor = EventProcessor::new(&config, input, output);
    let output_ref = processor.output();
    assert_eq!(output_ref.events().len(), 0);
}

#[test]
fn test_run_propagates_output_error() {
    let config = create_test_config(vec![]);
    let input = MockInput::new(vec![KeyEvent::Press(KeyCode::A)]);
    let mut output = MockOutput::new();
    output.set_fail_mode(true);

    let mut processor = EventProcessor::new(&config, input, output);
    let result = processor.run();
    assert!(matches!(result, Err(ProcessorError::Output(_))));
}

#[test]
fn test_complex_conditional_scenario() {
    // Setup: CapsLock = MD_00, ScrollLock = LK_01
    // When MD_00 active AND LK_01 active: J → Down
    let config = create_test_config(vec![
        KeyMapping::modifier(KeyCode::CapsLock, 0),
        KeyMapping::lock(KeyCode::ScrollLock, 1),
        KeyMapping::conditional(
            Condition::AllActive(vec![
                ConditionItem::ModifierActive(0),
                ConditionItem::LockActive(1),
            ]),
            vec![BaseKeyMapping::Simple {
                from: KeyCode::J,
                to: KeyCode::Down,
            }],
        ),
    ]);

    let input = MockInput::new(vec![
        KeyEvent::Press(KeyCode::CapsLock),   // Activate MD_00
        KeyEvent::Press(KeyCode::ScrollLock), // Toggle LK_01 on
        KeyEvent::Press(KeyCode::J),          // Should map to Down
    ]);
    let output = MockOutput::new();

    let mut processor = EventProcessor::new(&config, input, output);
    processor.run().unwrap();

    // Only J → Down should produce output
    assert_eq!(processor.output.events().len(), 1);
    assert_eq!(processor.output.events()[0], KeyEvent::Press(KeyCode::Down));
}
