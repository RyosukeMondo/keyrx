//! Coalescing engine wrapper that batches input events.

use crate::engine::types::{InputEvent, OutputAction};

use super::{CoalescingConfig, EventBuffer};

/// A wrapper around an engine that adds event coalescing.
///
/// The `CoalescingEngine` wraps any engine that implements event processing
/// and adds a buffering layer. Events are batched based on:
/// - Time window (flush_timeout)
/// - Batch size limit (max_batch_size)
/// - Modifier state changes (immediate flush)
///
/// This reduces FFI overhead during rapid typing while preserving timing
/// semantics for critical events like modifier changes.
///
/// # Type Parameters
///
/// - `E`: The inner engine type that processes events.
///
/// # Example
///
/// ```no_run
/// use keyrx_core::engine::{AdvancedEngine, CoalescingEngine, CoalescingConfig};
/// use std::time::Duration;
///
/// let config = CoalescingConfig::new(10, Duration::from_millis(5), true);
/// let inner_engine = AdvancedEngine::new(/* ... */);
/// let mut engine = CoalescingEngine::new(inner_engine, config);
/// ```
pub struct CoalescingEngine<E> {
    /// The wrapped engine that processes events.
    inner: E,
    /// Event buffer for batching.
    buffer: EventBuffer,
}

impl<E> CoalescingEngine<E> {
    /// Create a new coalescing engine wrapping an inner engine.
    ///
    /// # Parameters
    ///
    /// - `inner`: The engine to wrap.
    /// - `config`: Configuration for event coalescing behavior.
    pub fn new(inner: E, config: CoalescingConfig) -> Self {
        Self {
            inner,
            buffer: EventBuffer::new(config),
        }
    }

    /// Get a reference to the inner engine.
    pub fn inner(&self) -> &E {
        &self.inner
    }

    /// Get a mutable reference to the inner engine.
    pub fn inner_mut(&mut self) -> &mut E {
        &mut self.inner
    }

    /// Get a reference to the event buffer.
    pub fn buffer(&self) -> &EventBuffer {
        &self.buffer
    }

    /// Check if the buffer should be flushed due to timeout.
    pub fn should_flush(&self) -> bool {
        self.buffer.should_flush()
    }

    /// Manually flush all buffered events.
    ///
    /// This processes all buffered events through the inner engine
    /// and returns the accumulated output actions.
    ///
    /// # Returns
    ///
    /// A vector of output actions from processing all buffered events.
    pub fn flush(&mut self) -> Vec<OutputAction>
    where
        E: ProcessEvent,
    {
        let events = self.buffer.flush();
        self.process_batch(events)
    }

    /// Process a batch of events through the inner engine.
    ///
    /// This is a helper that processes multiple events and collects
    /// all output actions.
    fn process_batch(&mut self, events: Vec<InputEvent>) -> Vec<OutputAction>
    where
        E: ProcessEvent,
    {
        events
            .into_iter()
            .flat_map(|event| self.inner.process_event(event))
            .collect()
    }
}

impl<E> CoalescingEngine<E>
where
    E: ProcessEvent,
{
    /// Process an input event through the coalescing layer.
    ///
    /// The event is buffered and may trigger an immediate flush if:
    /// - The batch size limit is reached
    /// - A modifier key state changes
    ///
    /// Otherwise, the event is buffered and an empty vector is returned.
    /// The caller should periodically call `should_flush()` and `flush()`
    /// to ensure buffered events are processed in a timely manner.
    ///
    /// # Parameters
    ///
    /// - `event`: The input event to process.
    ///
    /// # Returns
    ///
    /// A vector of output actions. May be empty if the event was buffered
    /// without triggering a flush.
    pub fn process_event(&mut self, event: InputEvent) -> Vec<OutputAction> {
        match self.buffer.push(event) {
            Some(events) => self.process_batch(events),
            None => Vec::new(),
        }
    }
}

/// Trait for engines that can process individual events.
///
/// This trait abstracts over the different engine types (AdvancedEngine, Engine, etc.)
/// to allow the CoalescingEngine to work with any engine implementation.
pub trait ProcessEvent {
    /// Process a single input event and return output actions.
    fn process_event(&mut self, event: InputEvent) -> Vec<OutputAction>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::types::KeyCode;
    use std::time::Duration;

    /// Mock engine that echoes events as PassThrough actions.
    struct MockEngine {
        events_processed: Vec<InputEvent>,
    }

    impl MockEngine {
        fn new() -> Self {
            Self {
                events_processed: Vec::new(),
            }
        }
    }

    impl ProcessEvent for MockEngine {
        fn process_event(&mut self, event: InputEvent) -> Vec<OutputAction> {
            self.events_processed.push(event);
            vec![OutputAction::PassThrough]
        }
    }

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
            vendor_id: None,
            product_id: None,
        }
    }

    #[test]
    fn coalescing_engine_batches_events() {
        let config = CoalescingConfig {
            max_batch_size: 3,
            flush_timeout: Duration::from_millis(100),
            coalesce_repeats: false,
        };
        let mock = MockEngine::new();
        let mut engine = CoalescingEngine::new(mock, config);

        // First two events should be buffered
        let output1 = engine.process_event(make_event(KeyCode::A, true, false));
        assert!(output1.is_empty());

        let output2 = engine.process_event(make_event(KeyCode::B, true, false));
        assert!(output2.is_empty());

        // Third event triggers flush
        let output3 = engine.process_event(make_event(KeyCode::C, true, false));
        assert_eq!(output3.len(), 3); // All 3 events processed

        // Verify inner engine received all events
        assert_eq!(engine.inner().events_processed.len(), 3);
    }

    #[test]
    fn coalescing_engine_flushes_on_modifier() {
        let config = CoalescingConfig::default();
        let mock = MockEngine::new();
        let mut engine = CoalescingEngine::new(mock, config);

        // Buffer some events
        engine.process_event(make_event(KeyCode::A, true, false));
        engine.process_event(make_event(KeyCode::B, true, false));

        // Modifier triggers immediate flush
        let output = engine.process_event(make_event(KeyCode::LeftShift, true, false));
        assert_eq!(output.len(), 3);
    }

    #[test]
    fn manual_flush_processes_buffered_events() {
        let config = CoalescingConfig {
            max_batch_size: 10,
            flush_timeout: Duration::from_millis(100),
            coalesce_repeats: false,
        };
        let mock = MockEngine::new();
        let mut engine = CoalescingEngine::new(mock, config);

        // Buffer events
        engine.process_event(make_event(KeyCode::A, true, false));
        engine.process_event(make_event(KeyCode::B, true, false));

        assert_eq!(engine.inner().events_processed.len(), 0);

        // Manual flush
        let output = engine.flush();
        assert_eq!(output.len(), 2);
        assert_eq!(engine.inner().events_processed.len(), 2);
    }

    #[test]
    fn passthrough_config_processes_immediately() {
        let config = CoalescingConfig::passthrough();
        let mock = MockEngine::new();
        let mut engine = CoalescingEngine::new(mock, config);

        let output = engine.process_event(make_event(KeyCode::A, true, false));
        assert_eq!(output.len(), 1);
        assert_eq!(engine.inner().events_processed.len(), 1);
    }

    #[test]
    fn should_flush_indicates_timeout() {
        let config = CoalescingConfig {
            max_batch_size: 10,
            flush_timeout: Duration::from_millis(1),
            coalesce_repeats: false,
        };
        let mock = MockEngine::new();
        let mut engine = CoalescingEngine::new(mock, config);

        engine.process_event(make_event(KeyCode::A, true, false));
        std::thread::sleep(Duration::from_millis(2));

        assert!(engine.should_flush());
    }

    #[test]
    fn coalesce_repeats_reduces_event_count() {
        let config = CoalescingConfig {
            max_batch_size: 10,
            flush_timeout: Duration::from_millis(100),
            coalesce_repeats: true,
        };
        let mock = MockEngine::new();
        let mut engine = CoalescingEngine::new(mock, config);

        // Press A, 3 repeats, then release
        engine.process_event(make_event(KeyCode::A, true, false));
        engine.process_event(make_event(KeyCode::A, true, true));
        engine.process_event(make_event(KeyCode::A, true, true));
        engine.process_event(make_event(KeyCode::A, true, true));
        engine.process_event(make_event(KeyCode::A, false, false));

        // Flush and verify coalescing happened
        let output = engine.flush();
        // Should be 3: press, one repeat, release
        assert_eq!(output.len(), 3);
        assert_eq!(engine.inner().events_processed.len(), 3);
    }
}
