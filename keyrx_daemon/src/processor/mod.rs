//! Event processor orchestrator.
//!
//! This module provides the `EventProcessor` struct, which orchestrates the
//! complete event processing pipeline: reading from input devices, processing
//! events through the runtime, and injecting output events.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────┐
//! │InputDevice  │ (e.g., MockInput, LinuxPlatform)
//! └──────┬──────┘
//!        │ KeyEvent
//!        ▼
//! ┌─────────────────────┐
//! │ EventProcessor      │
//! │  - KeyLookup        │ (find mapping for key)
//! │  - DeviceState      │ (modifier/lock state)
//! │  - process_event()  │ (apply mapping)
//! └──────┬──────────────┘
//!        │ Vec<KeyEvent>
//!        ▼
//! ┌─────────────┐
//! │OutputDevice │ (e.g., MockOutput, LinuxPlatform)
//! └─────────────┘
//! ```

mod logging;

#[cfg(test)]
mod test_utils;

use std::time::Instant;

use keyrx_core::config::DeviceConfig;
use keyrx_core::runtime::event::{process_event, KeyEvent};
use keyrx_core::runtime::{DeviceState, KeyLookup};
use thiserror::Error;

use crate::platform::{DeviceError, InputDevice, OutputDevice};

/// Errors that can occur during event processing.
#[derive(Debug, Error)]
pub enum ProcessorError {
    /// Error reading from input device.
    #[error("input device error: {0}")]
    Input(#[from] DeviceError),

    /// Error writing to output device.
    #[error("output device error: {0}")]
    Output(DeviceError),
}

/// Event processor orchestrator.
///
/// Coordinates the complete event processing pipeline:
/// 1. Read events from input device
/// 2. Resolve key mappings using `KeyLookup`
/// 3. Update state and apply mappings via `process_event`
/// 4. Inject output events to output device
pub struct EventProcessor<I: InputDevice, O: OutputDevice> {
    /// Input device for reading keyboard events
    input: I,
    /// Output device for injecting keyboard events
    output: O,
    /// Key lookup table for O(1) mapping resolution
    lookup: KeyLookup,
    /// Runtime state (modifier and lock bits)
    state: DeviceState,
}

impl<I: InputDevice, O: OutputDevice> EventProcessor<I, O> {
    /// Creates a new event processor.
    pub fn new(config: &DeviceConfig, input: I, output: O) -> Self {
        let lookup = KeyLookup::from_device_config(config);
        let state = DeviceState::new();

        logging::log_config_loaded(config.mappings.len());

        Self {
            input,
            output,
            lookup,
            state,
        }
    }

    /// Processes a single event from the input device.
    pub fn process_one(&mut self) -> Result<(), ProcessorError> {
        let start = Instant::now();

        let event = self.read_input_event()?;
        let transition_context = self.detect_state_transition(event);
        let output_events = process_event(event, &self.lookup, &mut self.state);

        if let Some(context) = transition_context {
            logging::log_state_transition(&context);
        }

        self.inject_output_events(&output_events)?;

        let latency_us = start.elapsed().as_micros() as u64;
        self.log_processed_event(event, &output_events, latency_us);

        Ok(())
    }

    /// Runs the event processing loop until end of stream.
    pub fn run(&mut self) -> Result<(), ProcessorError> {
        loop {
            match self.process_one() {
                Ok(()) => continue,
                Err(ProcessorError::Input(DeviceError::EndOfStream)) => return Ok(()),
                Err(e) => return Err(e),
            }
        }
    }

    /// Returns a reference to the output device (for testing).
    pub fn output(&self) -> &O {
        &self.output
    }

    // --- Private helper methods ---

    fn read_input_event(&mut self) -> Result<KeyEvent, ProcessorError> {
        self.input.next_event().map_err(|e| {
            if !matches!(e, DeviceError::EndOfStream) {
                logging::log_platform_error(&e.to_string(), "input");
            }
            e.into()
        })
    }

    fn detect_state_transition(&self, event: KeyEvent) -> Option<String> {
        use keyrx_core::config::mappings::BaseKeyMapping;

        let mapping = self.lookup.find_mapping(event.keycode(), &self.state)?;

        match mapping {
            BaseKeyMapping::Modifier { modifier_id, .. } => {
                if event.is_press() {
                    Some(logging::format_modifier_activated(*modifier_id))
                } else {
                    Some(logging::format_modifier_deactivated(*modifier_id))
                }
            }
            BaseKeyMapping::Lock { lock_id, .. } => {
                if event.is_press() {
                    Some(logging::format_lock_toggled(*lock_id))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn inject_output_events(&mut self, events: &[KeyEvent]) -> Result<(), ProcessorError> {
        for event in events {
            self.output.inject_event(*event).map_err(|e| {
                logging::log_platform_error(&e.to_string(), "output");
                ProcessorError::Output(e)
            })?;
        }
        Ok(())
    }

    fn log_processed_event(&self, input: KeyEvent, outputs: &[KeyEvent], latency_us: u64) {
        let output_keys: Vec<_> = outputs.iter().map(|e| e.keycode()).collect();
        logging::log_key_processed(input.keycode(), &output_keys, latency_us);
    }
}

#[cfg(test)]
mod tests_coverage;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::{MockInput, MockOutput};
    use keyrx_core::config::{
        mappings::{BaseKeyMapping, KeyMapping},
        Condition, ConditionItem, KeyCode,
    };
    use test_utils::create_test_config;

    #[test]
    fn test_new_creates_processor() {
        let config = create_test_config(vec![]);
        let input = MockInput::new(vec![]);
        let output = MockOutput::new();

        let _processor = EventProcessor::new(&config, input, output);
    }

    #[test]
    fn test_process_one_passthrough() {
        let config = create_test_config(vec![]);
        let input = MockInput::new(vec![KeyEvent::Press(KeyCode::A)]);
        let output = MockOutput::new();

        let mut processor = EventProcessor::new(&config, input, output);
        processor.process_one().unwrap();

        assert_eq!(processor.output.events().len(), 1);
        assert_eq!(processor.output.events()[0], KeyEvent::Press(KeyCode::A));
    }

    #[test]
    fn test_process_one_simple_remap() {
        let config = create_test_config(vec![KeyMapping::simple(KeyCode::A, KeyCode::B)]);
        let input = MockInput::new(vec![KeyEvent::Press(KeyCode::A)]);
        let output = MockOutput::new();

        let mut processor = EventProcessor::new(&config, input, output);
        processor.process_one().unwrap();

        assert_eq!(processor.output.events().len(), 1);
        assert_eq!(processor.output.events()[0], KeyEvent::Press(KeyCode::B));
    }

    #[test]
    fn test_process_one_modifier_no_output() {
        let config = create_test_config(vec![KeyMapping::modifier(KeyCode::CapsLock, 0)]);
        let input = MockInput::new(vec![KeyEvent::Press(KeyCode::CapsLock)]);
        let output = MockOutput::new();

        let mut processor = EventProcessor::new(&config, input, output);
        processor.process_one().unwrap();

        assert_eq!(processor.output.events().len(), 0);
        assert!(processor.state.is_modifier_active(0));
    }

    #[test]
    fn test_process_one_lock_no_output() {
        let config = create_test_config(vec![KeyMapping::lock(KeyCode::ScrollLock, 1)]);
        let input = MockInput::new(vec![KeyEvent::Press(KeyCode::ScrollLock)]);
        let output = MockOutput::new();

        let mut processor = EventProcessor::new(&config, input, output);
        processor.process_one().unwrap();

        assert_eq!(processor.output.events().len(), 0);
        assert!(processor.state.is_lock_active(1));
    }

    #[test]
    fn test_process_one_modified_output() {
        let config = create_test_config(vec![KeyMapping::modified_output(
            KeyCode::Num1,
            KeyCode::Num1,
            true,
            false,
            false,
            false,
        )]);
        let input = MockInput::new(vec![KeyEvent::Press(KeyCode::Num1)]);
        let output = MockOutput::new();

        let mut processor = EventProcessor::new(&config, input, output);
        processor.process_one().unwrap();

        assert_eq!(processor.output.events().len(), 2);
        assert_eq!(
            processor.output.events()[0],
            KeyEvent::Press(KeyCode::LShift)
        );
        assert_eq!(processor.output.events()[1], KeyEvent::Press(KeyCode::Num1));
    }

    #[test]
    fn test_process_one_conditional_mapping() {
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

        let input = MockInput::new(vec![
            KeyEvent::Press(KeyCode::CapsLock),
            KeyEvent::Press(KeyCode::H),
        ]);
        let output = MockOutput::new();

        let mut processor = EventProcessor::new(&config, input, output);

        processor.process_one().unwrap();
        assert_eq!(processor.output.events().len(), 0);

        processor.process_one().unwrap();
        assert_eq!(processor.output.events().len(), 1);
        assert_eq!(processor.output.events()[0], KeyEvent::Press(KeyCode::Left));
    }

    #[test]
    fn test_run_processes_all_events() {
        let config = create_test_config(vec![KeyMapping::simple(KeyCode::A, KeyCode::B)]);
        let input = MockInput::new(vec![
            KeyEvent::Press(KeyCode::A),
            KeyEvent::Release(KeyCode::A),
            KeyEvent::Press(KeyCode::A),
            KeyEvent::Release(KeyCode::A),
        ]);
        let output = MockOutput::new();

        let mut processor = EventProcessor::new(&config, input, output);
        processor.run().unwrap();

        assert_eq!(processor.output.events().len(), 4);
        assert_eq!(processor.output.events()[0], KeyEvent::Press(KeyCode::B));
        assert_eq!(processor.output.events()[1], KeyEvent::Release(KeyCode::B));
        assert_eq!(processor.output.events()[2], KeyEvent::Press(KeyCode::B));
        assert_eq!(processor.output.events()[3], KeyEvent::Release(KeyCode::B));
    }

    #[test]
    fn test_run_handles_end_of_stream() {
        let config = create_test_config(vec![]);
        let input = MockInput::new(vec![KeyEvent::Press(KeyCode::A)]);
        let output = MockOutput::new();

        let mut processor = EventProcessor::new(&config, input, output);
        let result = processor.run();
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_empty_stream() {
        let config = create_test_config(vec![]);
        let input = MockInput::new(vec![]);
        let output = MockOutput::new();

        let mut processor = EventProcessor::new(&config, input, output);
        let result = processor.run();
        assert!(result.is_ok());
        assert_eq!(processor.output.events().len(), 0);
    }
}
