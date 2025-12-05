//! Event buffering and batching logic.

use crate::engine::types::InputEvent;
use std::collections::VecDeque;
use std::time::Instant;

use super::config::CoalescingConfig;

/// Timestamped event wrapper for tracking when events entered the buffer.
#[derive(Debug, Clone)]
struct TimestampedEvent {
    event: InputEvent,
    #[allow(dead_code)]
    buffered_at: Instant,
}

/// Event buffer for batching input events.
///
/// Events are buffered and flushed based on:
/// - Time window (flush_timeout)
/// - Batch size limit (max_batch_size)
/// - Modifier state changes
/// - Manual flush requests
pub struct EventBuffer {
    events: VecDeque<TimestampedEvent>,
    config: CoalescingConfig,
    last_flush: Instant,
}

impl EventBuffer {
    /// Create a new event buffer with the given configuration.
    pub fn new(config: CoalescingConfig) -> Self {
        Self {
            events: VecDeque::new(),
            config,
            last_flush: Instant::now(),
        }
    }

    /// Push an event into the buffer.
    ///
    /// Returns Some(batch) if the buffer should flush immediately:
    /// - Batch size limit reached
    /// - Modifier state changed
    ///
    /// Otherwise returns None, indicating the event was buffered.
    pub fn push(&mut self, event: InputEvent) -> Option<Vec<InputEvent>> {
        let should_flush_before = self.should_flush_on_modifier_change(&event);

        let timestamped = TimestampedEvent {
            event,
            buffered_at: Instant::now(),
        };

        self.events.push_back(timestamped);

        if should_flush_before || self.events.len() >= self.config.max_batch_size {
            Some(self.flush())
        } else {
            None
        }
    }

    /// Check if the buffer should flush based on timeout.
    pub fn should_flush(&self) -> bool {
        if self.events.is_empty() {
            return false;
        }

        let elapsed = self.last_flush.elapsed();
        elapsed >= self.config.flush_timeout
    }

    /// Flush all buffered events, applying coalescing rules.
    ///
    /// Returns a Vec of events to process. The buffer is emptied.
    pub fn flush(&mut self) -> Vec<InputEvent> {
        if self.events.is_empty() {
            return Vec::new();
        }

        let events = if self.config.coalesce_repeats {
            self.coalesce_repeats()
        } else {
            self.events.drain(..).map(|te| te.event).collect()
        };

        self.last_flush = Instant::now();
        events
    }

    /// Coalesce consecutive repeat events.
    ///
    /// Keeps only the last repeat event for each key, preserving:
    /// - Non-repeat events
    /// - Down/up pairs
    /// - Timing information from the original events
    fn coalesce_repeats(&mut self) -> Vec<InputEvent> {
        let mut result = Vec::new();
        let mut skip_indices = std::collections::HashSet::new();

        let events: Vec<_> = self.events.drain(..).collect();

        for i in 0..events.len() {
            if skip_indices.contains(&i) {
                continue;
            }

            let current = &events[i];

            // If this is a repeat event, look ahead for more repeats of the same key
            if current.event.is_repeat {
                let mut last_repeat_idx = i;

                for (j, next) in events.iter().enumerate().skip(i + 1) {
                    if next.event.key == current.event.key
                        && next.event.is_repeat
                        && next.event.pressed == current.event.pressed
                    {
                        skip_indices.insert(last_repeat_idx);
                        last_repeat_idx = j;
                    } else if next.event.key == current.event.key {
                        // Different event type for same key, stop looking
                        break;
                    }
                }

                // Mark the last repeat as processed so we don't add it again
                skip_indices.insert(last_repeat_idx);
                result.push(events[last_repeat_idx].event.clone());
            } else {
                result.push(current.event.clone());
            }
        }

        result
    }

    /// Check if the buffer should flush due to modifier state change.
    ///
    /// Flushes when:
    /// - A modifier key is pressed/released
    /// - This ensures modifier state changes are processed immediately
    fn should_flush_on_modifier_change(&self, event: &InputEvent) -> bool {
        use crate::engine::types::KeyCode;

        matches!(
            event.key,
            KeyCode::LeftShift
                | KeyCode::RightShift
                | KeyCode::LeftCtrl
                | KeyCode::RightCtrl
                | KeyCode::LeftAlt
                | KeyCode::RightAlt
                | KeyCode::LeftMeta
                | KeyCode::RightMeta
        )
    }

    /// Returns the number of events currently buffered.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Returns true if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::types::KeyCode;
    use std::time::Duration;

    fn make_event(key: KeyCode, pressed: bool, is_repeat: bool) -> InputEvent {
        InputEvent {
            key,
            pressed,
            timestamp_us: 0,
            device_id: None,
            is_repeat,
            is_synthetic: false,
            scan_code: 0,
            serial_number: None,
        }
    }

    #[test]
    fn buffer_respects_batch_size_limit() {
        let config = CoalescingConfig {
            max_batch_size: 3,
            flush_timeout: Duration::from_millis(100),
            coalesce_repeats: false,
        };
        let mut buffer = EventBuffer::new(config);

        assert!(buffer.push(make_event(KeyCode::A, true, false)).is_none());
        assert!(buffer.push(make_event(KeyCode::B, true, false)).is_none());

        let flushed = buffer.push(make_event(KeyCode::C, true, false));
        assert!(flushed.is_some());
        assert_eq!(flushed.unwrap().len(), 3);
        assert!(buffer.is_empty());
    }

    #[test]
    fn buffer_flushes_on_modifier_change() {
        let config = CoalescingConfig::default();
        let mut buffer = EventBuffer::new(config);

        buffer.push(make_event(KeyCode::A, true, false));
        buffer.push(make_event(KeyCode::B, true, false));

        let flushed = buffer.push(make_event(KeyCode::LeftShift, true, false));
        assert!(flushed.is_some());
        assert_eq!(flushed.unwrap().len(), 3);
    }

    #[test]
    fn buffer_coalesces_repeat_events() {
        let config = CoalescingConfig {
            max_batch_size: 10,
            flush_timeout: Duration::from_millis(100),
            coalesce_repeats: true,
        };
        let mut buffer = EventBuffer::new(config);

        buffer.push(make_event(KeyCode::A, true, false));
        buffer.push(make_event(KeyCode::A, true, true));
        buffer.push(make_event(KeyCode::A, true, true));
        buffer.push(make_event(KeyCode::A, true, true));
        buffer.push(make_event(KeyCode::A, false, false));

        let flushed = buffer.flush();
        assert_eq!(flushed.len(), 3);
        assert_eq!(flushed[0].key, KeyCode::A);
        assert!(flushed[0].pressed);
        assert!(!flushed[0].is_repeat);
        assert_eq!(flushed[1].key, KeyCode::A);
        assert!(flushed[1].pressed);
        assert!(flushed[1].is_repeat);
        assert_eq!(flushed[2].key, KeyCode::A);
        assert!(!flushed[2].pressed);
    }

    #[test]
    fn buffer_preserves_non_repeat_events() {
        let config = CoalescingConfig::default();
        let mut buffer = EventBuffer::new(config);

        buffer.push(make_event(KeyCode::A, true, false));
        buffer.push(make_event(KeyCode::B, true, false));
        buffer.push(make_event(KeyCode::A, false, false));

        let flushed = buffer.flush();
        assert_eq!(flushed.len(), 3);
    }

    #[test]
    fn should_flush_returns_true_after_timeout() {
        let config = CoalescingConfig {
            max_batch_size: 10,
            flush_timeout: Duration::from_millis(1),
            coalesce_repeats: false,
        };
        let mut buffer = EventBuffer::new(config);

        buffer.push(make_event(KeyCode::A, true, false));
        std::thread::sleep(Duration::from_millis(2));

        assert!(buffer.should_flush());
    }

    #[test]
    fn empty_buffer_does_not_need_flush() {
        let config = CoalescingConfig::default();
        let buffer = EventBuffer::new(config);

        assert!(!buffer.should_flush());
    }

    #[test]
    fn passthrough_config_flushes_immediately() {
        let config = CoalescingConfig::passthrough();
        let mut buffer = EventBuffer::new(config);

        let flushed = buffer.push(make_event(KeyCode::A, true, false));
        assert!(flushed.is_some());
        assert_eq!(flushed.unwrap().len(), 1);
    }
}
