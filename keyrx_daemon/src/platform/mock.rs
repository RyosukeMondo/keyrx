//! Mock platform implementation for testing.
//!
//! This module provides zero-dependency mock implementations of InputDevice and OutputDevice
//! for testing the event processing pipeline without requiring OS-specific functionality.

use std::collections::VecDeque;

use keyrx_core::runtime::event::KeyEvent;

use super::{DeviceError, InputDevice, OutputDevice};

/// Mock input device for testing.
///
/// MockInput simulates an input device by providing a preloaded queue of events.
/// Events are returned in FIFO order via `next_event()`. When the queue is exhausted,
/// `EndOfStream` is returned.
///
/// # Example
///
/// ```
/// use keyrx_daemon::platform::{InputDevice, DeviceError};
/// use keyrx_daemon::platform::mock::MockInput;
/// use keyrx_core::runtime::event::KeyEvent;
/// use keyrx_core::config::KeyCode;
///
/// let events = vec![
///     KeyEvent::Press(KeyCode::A),
///     KeyEvent::Release(KeyCode::A),
/// ];
/// let mut input = MockInput::new(events);
///
/// // Events are returned in FIFO order
/// let event1 = input.next_event().unwrap();
/// assert!(event1.is_press() && event1.keycode() == KeyCode::A);
///
/// let event2 = input.next_event().unwrap();
/// assert!(event2.is_release() && event2.keycode() == KeyCode::A);
///
/// // EndOfStream when exhausted
/// assert!(matches!(input.next_event(), Err(DeviceError::EndOfStream)));
/// ```
#[allow(dead_code)] // Will be used in tasks #17-20
pub struct MockInput {
    /// Event queue (FIFO)
    events: VecDeque<KeyEvent>,
    /// Exclusive access flag (set by grab(), cleared by release())
    grabbed: bool,
}

impl MockInput {
    /// Creates a new MockInput with preloaded events.
    ///
    /// # Arguments
    ///
    /// * `events` - Vector of events to return in FIFO order
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_daemon::platform::mock::MockInput;
    /// use keyrx_core::runtime::event::KeyEvent;
    /// use keyrx_core::config::KeyCode;
    ///
    /// let events = vec![KeyEvent::Press(KeyCode::A)];
    /// let input = MockInput::new(events);
    /// ```
    #[allow(dead_code)] // Will be used in tasks #17-20
    pub fn new(events: Vec<KeyEvent>) -> Self {
        Self {
            events: VecDeque::from(events),
            grabbed: false,
        }
    }

    /// Returns whether the device is grabbed (for testing).
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_daemon::platform::{InputDevice, mock::MockInput};
    ///
    /// let mut input = MockInput::new(vec![]);
    /// assert!(!input.is_grabbed());
    ///
    /// input.grab().unwrap();
    /// assert!(input.is_grabbed());
    ///
    /// input.release().unwrap();
    /// assert!(!input.is_grabbed());
    /// ```
    #[allow(dead_code)] // Will be used in tasks #17-20
    pub fn is_grabbed(&self) -> bool {
        self.grabbed
    }
}

impl InputDevice for MockInput {
    /// Returns the next event from the queue.
    ///
    /// Events are returned in FIFO order. When the queue is exhausted,
    /// `DeviceError::EndOfStream` is returned.
    fn next_event(&mut self) -> Result<KeyEvent, DeviceError> {
        self.events.pop_front().ok_or(DeviceError::EndOfStream)
    }

    /// Sets the grabbed flag to true.
    ///
    /// For MockInput, this only updates an internal flag that can be
    /// queried with `is_grabbed()`. Always succeeds.
    fn grab(&mut self) -> Result<(), DeviceError> {
        self.grabbed = true;
        Ok(())
    }

    /// Sets the grabbed flag to false.
    ///
    /// For MockInput, this only updates an internal flag that can be
    /// queried with `is_grabbed()`. Always succeeds.
    fn release(&mut self) -> Result<(), DeviceError> {
        self.grabbed = false;
        Ok(())
    }
}

/// Mock output device for testing.
///
/// MockOutput captures injected events in a Vec for later verification.
/// This is useful for testing event processing pipelines without requiring
/// OS-specific output functionality.
///
/// # Example
///
/// ```
/// use keyrx_daemon::platform::{OutputDevice, mock::MockOutput};
/// use keyrx_core::runtime::event::KeyEvent;
/// use keyrx_core::config::KeyCode;
///
/// let mut output = MockOutput::new();
///
/// // Inject events
/// output.inject_event(KeyEvent::Press(KeyCode::B)).unwrap();
/// output.inject_event(KeyEvent::Release(KeyCode::B)).unwrap();
///
/// // Verify captured events
/// assert_eq!(output.events().len(), 2);
/// assert_eq!(output.events()[0], KeyEvent::Press(KeyCode::B));
/// assert_eq!(output.events()[1], KeyEvent::Release(KeyCode::B));
/// ```
#[allow(dead_code)] // Will be used in tasks #17-20
pub struct MockOutput {
    /// Captured events (append-only)
    events: Vec<KeyEvent>,
    /// Optional failure mode for testing error handling
    fail_mode: bool,
}

impl MockOutput {
    /// Creates a new MockOutput with empty event buffer.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_daemon::platform::mock::MockOutput;
    ///
    /// let output = MockOutput::new();
    /// assert_eq!(output.events().len(), 0);
    /// ```
    #[allow(dead_code)] // Will be used in tasks #17-20
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            fail_mode: false,
        }
    }

    /// Returns a slice of all captured events.
    ///
    /// Events are returned in the order they were injected.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_daemon::platform::{OutputDevice, mock::MockOutput};
    /// use keyrx_core::runtime::event::KeyEvent;
    /// use keyrx_core::config::KeyCode;
    ///
    /// let mut output = MockOutput::new();
    /// output.inject_event(KeyEvent::Press(KeyCode::A)).unwrap();
    ///
    /// assert_eq!(output.events().len(), 1);
    /// ```
    #[allow(dead_code)] // Will be used in tasks #17-20
    pub fn events(&self) -> &[KeyEvent] {
        &self.events
    }

    /// Enables failure mode for testing error handling.
    ///
    /// When enabled, `inject_event()` will return `InjectionFailed` errors
    /// instead of succeeding.
    ///
    /// # Example
    ///
    /// ```
    /// use keyrx_daemon::platform::{OutputDevice, DeviceError, mock::MockOutput};
    /// use keyrx_core::runtime::event::KeyEvent;
    /// use keyrx_core::config::KeyCode;
    ///
    /// let mut output = MockOutput::new();
    /// output.set_fail_mode(true);
    ///
    /// let result = output.inject_event(KeyEvent::Press(KeyCode::A));
    /// assert!(matches!(result, Err(DeviceError::InjectionFailed(_))));
    /// ```
    #[allow(dead_code)] // Will be used in tasks #17-20
    pub fn set_fail_mode(&mut self, enabled: bool) {
        self.fail_mode = enabled;
    }
}

impl Default for MockOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputDevice for MockOutput {
    /// Injects an event into the output buffer.
    ///
    /// Events are appended to the internal buffer and can be retrieved
    /// via `events()`. Always succeeds unless `fail_mode` is enabled.
    fn inject_event(&mut self, event: KeyEvent) -> Result<(), DeviceError> {
        if self.fail_mode {
            return Err(DeviceError::InjectionFailed(
                "mock failure mode enabled".to_string(),
            ));
        }
        self.events.push(event);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyrx_core::config::KeyCode;

    #[test]
    fn test_mock_input_event_sequence() {
        let events = vec![
            KeyEvent::Press(KeyCode::A),
            KeyEvent::Release(KeyCode::A),
            KeyEvent::Press(KeyCode::B),
        ];
        let mut input = MockInput::new(events.clone());

        // Events returned in FIFO order
        assert_eq!(input.next_event().unwrap(), events[0]);
        assert_eq!(input.next_event().unwrap(), events[1]);
        assert_eq!(input.next_event().unwrap(), events[2]);

        // EndOfStream when exhausted
        assert!(matches!(input.next_event(), Err(DeviceError::EndOfStream)));
    }

    #[test]
    fn test_mock_input_grab_release() {
        let mut input = MockInput::new(vec![]);

        // Initially not grabbed
        assert!(!input.is_grabbed());

        // Grab sets flag
        input.grab().unwrap();
        assert!(input.is_grabbed());

        // Release clears flag
        input.release().unwrap();
        assert!(!input.is_grabbed());
    }

    #[test]
    fn test_mock_input_end_of_stream() {
        let mut input = MockInput::new(vec![]);

        // Empty queue returns EndOfStream immediately
        assert!(matches!(input.next_event(), Err(DeviceError::EndOfStream)));

        // Subsequent calls still return EndOfStream
        assert!(matches!(input.next_event(), Err(DeviceError::EndOfStream)));
    }

    #[test]
    fn test_mock_output_event_capture() {
        let mut output = MockOutput::new();

        // Initially empty
        assert_eq!(output.events().len(), 0);

        // Inject events
        output.inject_event(KeyEvent::Press(KeyCode::A)).unwrap();
        output.inject_event(KeyEvent::Release(KeyCode::A)).unwrap();
        output.inject_event(KeyEvent::Press(KeyCode::B)).unwrap();

        // Events captured in order
        let events = output.events();
        assert_eq!(events.len(), 3);
        assert_eq!(events[0], KeyEvent::Press(KeyCode::A));
        assert_eq!(events[1], KeyEvent::Release(KeyCode::A));
        assert_eq!(events[2], KeyEvent::Press(KeyCode::B));
    }

    #[test]
    fn test_mock_output_ordering() {
        let mut output = MockOutput::new();

        // Inject sequence
        let sequence = vec![
            KeyEvent::Press(KeyCode::LShift),
            KeyEvent::Press(KeyCode::Num1),
            KeyEvent::Release(KeyCode::Num1),
            KeyEvent::Release(KeyCode::LShift),
        ];

        for event in &sequence {
            output.inject_event(event.clone()).unwrap();
        }

        // Verify order preserved
        assert_eq!(output.events(), &sequence[..]);
    }

    #[test]
    fn test_mock_output_fail_mode() {
        let mut output = MockOutput::new();

        // Normal operation succeeds
        output.inject_event(KeyEvent::Press(KeyCode::A)).unwrap();
        assert_eq!(output.events().len(), 1);

        // Enable fail mode
        output.set_fail_mode(true);

        // Injection fails
        let result = output.inject_event(KeyEvent::Press(KeyCode::B));
        assert!(matches!(result, Err(DeviceError::InjectionFailed(_))));

        // Event not captured when failure occurs
        assert_eq!(output.events().len(), 1);

        // Disable fail mode
        output.set_fail_mode(false);

        // Normal operation resumes
        output.inject_event(KeyEvent::Press(KeyCode::C)).unwrap();
        assert_eq!(output.events().len(), 2);
    }
}
