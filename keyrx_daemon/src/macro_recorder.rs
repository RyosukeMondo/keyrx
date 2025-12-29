//! Macro recorder for capturing and managing keyboard event sequences
//!
//! This module provides functionality to:
//! - Record key events with microsecond-precision timestamps
//! - Store events in an in-memory buffer
//! - Toggle recording mode on/off
//! - Export recorded events for macro generation

use keyrx_core::runtime::KeyEvent;
use std::sync::{Arc, Mutex};

/// Maximum number of events to store in the recording buffer
const MAX_EVENTS: usize = 10_000;

/// Macro event with relative timestamp from recording start
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MacroEvent {
    /// The key event (press or release)
    pub event: KeyEvent,
    /// Relative timestamp in microseconds from recording start
    pub relative_timestamp_us: u64,
}

/// Recording state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecordingState {
    /// Not recording
    Idle,
    /// Currently recording events
    Recording,
}

/// Macro recorder for capturing keyboard events
#[derive(Clone)]
pub struct MacroRecorder {
    /// Recording state
    state: Arc<Mutex<RecordingState>>,
    /// Buffer of recorded events
    events: Arc<Mutex<Vec<MacroEvent>>>,
    /// Timestamp when recording started (for relative timestamps)
    start_timestamp: Arc<Mutex<Option<u64>>>,
}

impl MacroRecorder {
    /// Creates a new macro recorder
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(RecordingState::Idle)),
            events: Arc::new(Mutex::new(Vec::new())),
            start_timestamp: Arc::new(Mutex::new(None)),
        }
    }

    /// Starts recording events
    ///
    /// Clears any previously recorded events and begins recording.
    /// Returns `Ok(())` if recording started, or `Err` if already recording.
    pub fn start_recording(&self) -> Result<(), String> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| format!("Failed to lock state: {}", e))?;

        if *state == RecordingState::Recording {
            return Err("Already recording".to_string());
        }

        // Clear previous recording
        let mut events = self
            .events
            .lock()
            .map_err(|e| format!("Failed to lock events: {}", e))?;
        events.clear();

        // Reset start timestamp
        let mut start_ts = self
            .start_timestamp
            .lock()
            .map_err(|e| format!("Failed to lock timestamp: {}", e))?;
        *start_ts = None;

        *state = RecordingState::Recording;
        Ok(())
    }

    /// Stops recording events
    ///
    /// Returns `Ok(())` if recording stopped, or `Err` if not currently recording.
    pub fn stop_recording(&self) -> Result<(), String> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| format!("Failed to lock state: {}", e))?;

        if *state != RecordingState::Recording {
            return Err("Not recording".to_string());
        }

        *state = RecordingState::Idle;
        Ok(())
    }

    /// Checks if currently recording
    pub fn is_recording(&self) -> bool {
        self.state
            .lock()
            .map(|s| *s == RecordingState::Recording)
            .unwrap_or(false)
    }

    /// Captures a key event during recording
    ///
    /// If recording is active, adds the event to the buffer with a relative timestamp.
    /// The first event sets the start timestamp (t=0).
    ///
    /// # Arguments
    ///
    /// * `event` - The key event to record
    ///
    /// # Returns
    ///
    /// `Ok(())` if event was recorded, `Err` if not recording or buffer is full.
    pub fn capture_event(&self, event: KeyEvent) -> Result<(), String> {
        let state = self
            .state
            .lock()
            .map_err(|e| format!("Failed to lock state: {}", e))?;

        if *state != RecordingState::Recording {
            return Err("Not recording".to_string());
        }
        drop(state);

        let mut events = self
            .events
            .lock()
            .map_err(|e| format!("Failed to lock events: {}", e))?;

        // Check buffer capacity
        if events.len() >= MAX_EVENTS {
            return Err(format!("Recording buffer full (max {} events)", MAX_EVENTS));
        }

        // Set start timestamp on first event
        let mut start_ts = self
            .start_timestamp
            .lock()
            .map_err(|e| format!("Failed to lock timestamp: {}", e))?;

        let relative_timestamp = if let Some(start) = *start_ts {
            // Calculate relative timestamp
            let current = event.timestamp_us();
            current.saturating_sub(start)
        } else {
            // First event - set start timestamp
            *start_ts = Some(event.timestamp_us());
            0
        };

        events.push(MacroEvent {
            event,
            relative_timestamp_us: relative_timestamp,
        });

        Ok(())
    }

    /// Gets the currently recorded events
    ///
    /// Returns a copy of all recorded events.
    pub fn get_recorded_events(&self) -> Result<Vec<MacroEvent>, String> {
        let events = self
            .events
            .lock()
            .map_err(|e| format!("Failed to lock events: {}", e))?;
        Ok(events.clone())
    }

    /// Gets the number of recorded events
    pub fn event_count(&self) -> usize {
        self.events.lock().map(|e| e.len()).unwrap_or(0)
    }

    /// Clears all recorded events without stopping recording
    pub fn clear_events(&self) -> Result<(), String> {
        let mut events = self
            .events
            .lock()
            .map_err(|e| format!("Failed to lock events: {}", e))?;
        events.clear();

        let mut start_ts = self
            .start_timestamp
            .lock()
            .map_err(|e| format!("Failed to lock timestamp: {}", e))?;
        *start_ts = None;

        Ok(())
    }
}

impl Default for MacroRecorder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyrx_core::config::KeyCode;

    #[test]
    fn test_start_stop_recording() {
        let recorder = MacroRecorder::new();
        assert!(!recorder.is_recording());

        // Start recording
        assert!(recorder.start_recording().is_ok());
        assert!(recorder.is_recording());

        // Cannot start twice
        assert!(recorder.start_recording().is_err());

        // Stop recording
        assert!(recorder.stop_recording().is_ok());
        assert!(!recorder.is_recording());

        // Cannot stop twice
        assert!(recorder.stop_recording().is_err());
    }

    #[test]
    fn test_capture_event() {
        let recorder = MacroRecorder::new();

        // Cannot capture when not recording
        let event = KeyEvent::press(KeyCode::A).with_timestamp(1000);
        assert!(recorder.capture_event(event).is_err());

        // Start recording and capture events
        recorder.start_recording().unwrap();

        let event1 = KeyEvent::press(KeyCode::A).with_timestamp(1000);
        assert!(recorder.capture_event(event1).is_ok());

        let event2 = KeyEvent::release(KeyCode::A).with_timestamp(1100);
        assert!(recorder.capture_event(event2).is_ok());

        assert_eq!(recorder.event_count(), 2);

        // Get recorded events
        let events = recorder.get_recorded_events().unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].relative_timestamp_us, 0); // First event at t=0
        assert_eq!(events[1].relative_timestamp_us, 100); // Second event 100us later
    }

    #[test]
    fn test_relative_timestamps() {
        let recorder = MacroRecorder::new();
        recorder.start_recording().unwrap();

        // Record events with absolute timestamps
        recorder
            .capture_event(KeyEvent::press(KeyCode::A).with_timestamp(5000))
            .unwrap();
        recorder
            .capture_event(KeyEvent::release(KeyCode::A).with_timestamp(5500))
            .unwrap();
        recorder
            .capture_event(KeyEvent::press(KeyCode::B).with_timestamp(6000))
            .unwrap();

        let events = recorder.get_recorded_events().unwrap();
        assert_eq!(events[0].relative_timestamp_us, 0); // t=0
        assert_eq!(events[1].relative_timestamp_us, 500); // +500us
        assert_eq!(events[2].relative_timestamp_us, 1000); // +1000us
    }

    #[test]
    fn test_clear_events() {
        let recorder = MacroRecorder::new();
        recorder.start_recording().unwrap();

        recorder
            .capture_event(KeyEvent::press(KeyCode::A).with_timestamp(1000))
            .unwrap();
        assert_eq!(recorder.event_count(), 1);

        recorder.clear_events().unwrap();
        assert_eq!(recorder.event_count(), 0);
        assert!(recorder.is_recording()); // Still recording
    }

    #[test]
    fn test_start_clears_previous_recording() {
        let recorder = MacroRecorder::new();
        recorder.start_recording().unwrap();

        recorder
            .capture_event(KeyEvent::press(KeyCode::A).with_timestamp(1000))
            .unwrap();
        assert_eq!(recorder.event_count(), 1);

        recorder.stop_recording().unwrap();

        // Start new recording - should clear old events
        recorder.start_recording().unwrap();
        assert_eq!(recorder.event_count(), 0);
    }

    #[test]
    fn test_buffer_capacity() {
        let recorder = MacroRecorder::new();
        recorder.start_recording().unwrap();

        // Fill buffer to max capacity
        for i in 0..MAX_EVENTS {
            let event = KeyEvent::press(KeyCode::A).with_timestamp(i as u64);
            assert!(recorder.capture_event(event).is_ok());
        }

        assert_eq!(recorder.event_count(), MAX_EVENTS);

        // Next event should fail (buffer full)
        let event = KeyEvent::press(KeyCode::A).with_timestamp(MAX_EVENTS as u64);
        assert!(recorder.capture_event(event).is_err());
    }

    #[test]
    fn test_default_trait() {
        let recorder = MacroRecorder::default();
        assert!(!recorder.is_recording());
        assert_eq!(recorder.event_count(), 0);
    }

    #[test]
    fn test_clone_recorder() {
        let recorder = MacroRecorder::new();
        recorder.start_recording().unwrap();
        recorder
            .capture_event(KeyEvent::press(KeyCode::A).with_timestamp(1000))
            .unwrap();

        // Clone should share the same state
        let cloned = recorder.clone();
        assert!(cloned.is_recording());
        assert_eq!(cloned.event_count(), 1);

        // Stop recording on original
        recorder.stop_recording().unwrap();

        // Clone should also reflect the change
        assert!(!cloned.is_recording());
    }

    #[test]
    fn test_multiple_recordings() {
        let recorder = MacroRecorder::new();

        // First recording
        recorder.start_recording().unwrap();
        recorder
            .capture_event(KeyEvent::press(KeyCode::A).with_timestamp(1000))
            .unwrap();
        recorder
            .capture_event(KeyEvent::release(KeyCode::A).with_timestamp(1100))
            .unwrap();
        recorder.stop_recording().unwrap();

        let events1 = recorder.get_recorded_events().unwrap();
        assert_eq!(events1.len(), 2);

        // Second recording - should clear first
        recorder.start_recording().unwrap();
        recorder
            .capture_event(KeyEvent::press(KeyCode::B).with_timestamp(2000))
            .unwrap();
        recorder.stop_recording().unwrap();

        let events2 = recorder.get_recorded_events().unwrap();
        assert_eq!(events2.len(), 1);
        assert_eq!(events2[0].event.keycode(), KeyCode::B);
    }

    #[test]
    fn test_timestamp_overflow_handling() {
        let recorder = MacroRecorder::new();
        recorder.start_recording().unwrap();

        // Test with very large timestamps (near u64 max)
        let large_ts = u64::MAX - 1000;
        recorder
            .capture_event(KeyEvent::press(KeyCode::A).with_timestamp(large_ts))
            .unwrap();

        // Next timestamp - saturating_sub prevents overflow
        recorder
            .capture_event(KeyEvent::release(KeyCode::A).with_timestamp(u64::MAX))
            .unwrap();

        let events = recorder.get_recorded_events().unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].relative_timestamp_us, 0);
        // Relative timestamp is calculated with saturating_sub
        assert_eq!(events[1].relative_timestamp_us, 1000);
    }

    #[test]
    fn test_concurrent_access() {
        use std::thread;

        let recorder = MacroRecorder::new();
        recorder.start_recording().unwrap();

        let recorder_clone = recorder.clone();
        let handle = thread::spawn(move || {
            for i in 0..100 {
                let _ = recorder_clone
                    .capture_event(KeyEvent::press(KeyCode::A).with_timestamp(i * 100));
            }
        });

        for i in 0..100 {
            let _ =
                recorder.capture_event(KeyEvent::press(KeyCode::B).with_timestamp(i * 100 + 50));
        }

        handle.join().unwrap();

        // Should have captured events from both threads
        let events = recorder.get_recorded_events().unwrap();
        assert_eq!(events.len(), 200);
    }

    #[test]
    fn test_event_serialization() {
        let event = MacroEvent {
            event: KeyEvent::press(KeyCode::A).with_timestamp(1000),
            relative_timestamp_us: 500,
        };

        // Test that MacroEvent can be serialized/deserialized
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: MacroEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.relative_timestamp_us, 500);
        assert_eq!(deserialized.event.keycode(), KeyCode::A);
    }

    #[test]
    fn test_clear_while_recording() {
        let recorder = MacroRecorder::new();
        recorder.start_recording().unwrap();

        // Add some events
        recorder
            .capture_event(KeyEvent::press(KeyCode::A).with_timestamp(1000))
            .unwrap();
        recorder
            .capture_event(KeyEvent::release(KeyCode::A).with_timestamp(1100))
            .unwrap();
        assert_eq!(recorder.event_count(), 2);

        // Clear events but keep recording
        recorder.clear_events().unwrap();
        assert_eq!(recorder.event_count(), 0);
        assert!(recorder.is_recording());

        // Add new events - should have fresh timestamps
        recorder
            .capture_event(KeyEvent::press(KeyCode::B).with_timestamp(2000))
            .unwrap();

        let events = recorder.get_recorded_events().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].relative_timestamp_us, 0); // Fresh start
    }

    #[test]
    fn test_get_events_while_recording() {
        let recorder = MacroRecorder::new();
        recorder.start_recording().unwrap();

        // Can get events while recording
        recorder
            .capture_event(KeyEvent::press(KeyCode::A).with_timestamp(1000))
            .unwrap();

        let events = recorder.get_recorded_events().unwrap();
        assert_eq!(events.len(), 1);

        // Continue recording
        recorder
            .capture_event(KeyEvent::release(KeyCode::A).with_timestamp(1100))
            .unwrap();

        let events2 = recorder.get_recorded_events().unwrap();
        assert_eq!(events2.len(), 2);
    }
}
