//! Main engine event loop.

use crate::engine::coalescing::{CoalescingConfig, EventBuffer, ProcessEvent};
use crate::engine::{
    InputEvent, OutputAction, OutputQueue, RemapAction, ResourceEnforcer, ResourceLimits,
    TimingConfig,
};
use crate::errors::KeyrxError;
use crate::ffi::domains::engine::global_event_registry;
use crate::ffi::events::EventType;
#[allow(deprecated)]
use crate::ffi::publish_state_snapshot_legacy;
use crate::metrics::{MetricsCollector, Operation, OtelMetricsCollector};
use crate::traits::{InputSource, ScriptRuntime, StateStore};
use crate::KeyCode;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tracing::debug;

/// The main KeyRx engine.
///
/// Generic over input source, script runtime, and state store
/// for dependency injection and testability.
pub struct Engine<I, S, St>
where
    I: InputSource,
    S: ScriptRuntime,
    St: StateStore,
{
    input: I,
    script: S,
    state: St,
    metrics: Arc<dyn MetricsCollector>,
    running: bool,
    held_keys: HashSet<KeyCode>,
    buffer: Option<EventBuffer>,
    resource_enforcer: Arc<ResourceEnforcer>,
    output_queue: OutputQueue,
}

impl<I, S, St> Engine<I, S, St>
where
    I: InputSource,
    S: ScriptRuntime,
    St: StateStore,
{
    /// Create a new engine with injected dependencies.
    pub fn new(input: I, script: S, state: St, metrics: Arc<dyn MetricsCollector>) -> Self {
        let resource_enforcer = Arc::new(ResourceEnforcer::new(ResourceLimits::default()));
        Self {
            input,
            script,
            state,
            metrics,
            running: false,
            held_keys: HashSet::new(),
            buffer: None,
            output_queue: OutputQueue::new(Arc::clone(&resource_enforcer)),
            resource_enforcer,
        }
    }

    /// Configure resource enforcement limits.
    pub fn with_resource_limits(mut self, limits: ResourceLimits) -> Self {
        self.resource_enforcer = Arc::new(ResourceEnforcer::new(limits));
        self.output_queue
            .replace_enforcer(Arc::clone(&self.resource_enforcer));
        self
    }

    /// Update resource enforcement limits after construction.
    pub fn set_resource_limits(&mut self, limits: ResourceLimits) {
        self.resource_enforcer = Arc::new(ResourceEnforcer::new(limits));
        self.output_queue
            .replace_enforcer(Arc::clone(&self.resource_enforcer));
    }

    /// Shared resource enforcer for timeout, memory, and queue tracking.
    pub fn resource_enforcer(&self) -> Arc<ResourceEnforcer> {
        Arc::clone(&self.resource_enforcer)
    }

    /// Enable event coalescing with the given configuration.
    ///
    /// This adds a buffering layer that batches events to reduce FFI overhead.
    /// Events are flushed based on time windows, batch size limits, or modifier
    /// state changes.
    pub fn enable_coalescing(&mut self, config: CoalescingConfig) {
        self.buffer = Some(EventBuffer::new(config));
    }

    /// Disable event coalescing, processing events immediately.
    pub fn disable_coalescing(&mut self) {
        self.buffer = None;
    }

    /// Start the engine event loop.
    pub async fn start(&mut self) -> Result<(), KeyrxError> {
        self.input.start().await?;
        self.running = true;
        Ok(())
    }

    /// Stop the engine.
    pub async fn stop(&mut self) -> Result<(), KeyrxError> {
        self.running = false;
        self.input.stop().await?;
        Ok(())
    }

    /// Check if engine is running.
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Get reference to state store.
    pub fn state(&self) -> &St {
        &self.state
    }

    /// Get mutable reference to state store.
    pub fn state_mut(&mut self) -> &mut St {
        &mut self.state
    }

    /// Get reference to script runtime.
    pub fn script(&self) -> &S {
        &self.script
    }

    /// Get mutable reference to script runtime.
    pub fn script_mut(&mut self) -> &mut S {
        &mut self.script
    }

    /// Get reference to metrics collector.
    pub fn metrics(&self) -> &Arc<dyn MetricsCollector> {
        &self.metrics
    }

    fn otel_metrics(&self) -> Option<&OtelMetricsCollector> {
        self.metrics
            .as_ref()
            .as_any()
            .downcast_ref::<OtelMetricsCollector>()
    }

    fn record_key_event_metric(&self, event: &InputEvent) {
        if let Some(otel) = self.otel_metrics() {
            let action = if event.pressed { "press" } else { "release" };
            otel.record_key_event(event.scan_code as u32, action);
        }
    }

    /// Process a single input event and return the appropriate output action.
    ///
    /// Queries the script runtime's registry for remapping decisions and
    /// translates them to output actions. Handles both key-down and key-up events.
    ///
    /// **Synthetic event filtering**: Events with `is_synthetic = true` are
    /// automatically passed through without processing. This prevents infinite
    /// loops when our injected keys are recaptured by the input hook.
    pub fn process_event(&self, event: &InputEvent) -> OutputAction {
        let _guard = self.metrics.start_profile("process_event");

        if event.is_synthetic {
            return Self::handle_synthetic(event);
        }

        self.record_key_event_metric(event);

        let execution_guard = self.resource_enforcer.start_execution();
        let rule_match_start = Instant::now();
        let action = self.script.lookup_remap(event.key);
        let rule_match_micros = rule_match_start.elapsed().as_micros() as u64;
        self.metrics
            .record_latency(Operation::RuleMatch, rule_match_micros);

        if execution_guard.check_timeout().is_err() {
            return OutputAction::PassThrough;
        }

        let action_start = Instant::now();
        let output = match action {
            RemapAction::Remap(target_key) => Self::handle_remap(event, target_key),
            RemapAction::Block => Self::handle_block(event),
            RemapAction::Pass => Self::handle_pass(event),
        };

        let action_elapsed = action_start.elapsed().as_micros() as u64;
        self.metrics
            .record_latency(Operation::ActionExecute, action_elapsed);

        output
    }

    fn handle_synthetic(event: &InputEvent) -> OutputAction {
        debug!(
            service = "keyrx",
            event = "skip_synthetic_event",
            component = "engine_event_loop",
            key = ?event.key,
            pressed = event.pressed,
            "Skipping synthetic input event"
        );
        OutputAction::PassThrough
    }

    fn handle_remap(event: &InputEvent, target_key: KeyCode) -> OutputAction {
        debug!(
            service = "keyrx",
            event = "remap_action",
            component = "engine_event_loop",
            from = ?event.key,
            to = ?target_key,
            pressed = event.pressed,
            "Remapping key"
        );
        if event.pressed {
            OutputAction::KeyDown(target_key)
        } else {
            OutputAction::KeyUp(target_key)
        }
    }

    fn handle_block(event: &InputEvent) -> OutputAction {
        debug!(
            service = "keyrx",
            event = "block_action",
            component = "engine_event_loop",
            key = ?event.key,
            pressed = event.pressed,
            "Blocking key"
        );
        OutputAction::Block
    }

    fn handle_pass(event: &InputEvent) -> OutputAction {
        debug!(
            service = "keyrx",
            event = "pass_action",
            component = "engine_event_loop",
            key = ?event.key,
            pressed = event.pressed,
            "Passing key through"
        );
        OutputAction::PassThrough
    }

    /// Run the main event loop.
    ///
    /// Polls the input source for events, processes each through `process_event`,
    /// and sends the resulting output actions back to the OS. Runs until
    /// `stop()` is called or an error occurs.
    ///
    /// If event coalescing is enabled, events are buffered and processed in
    /// batches to reduce FFI overhead.
    pub async fn run_loop(&mut self) -> Result<(), KeyrxError> {
        debug!(
            service = "keyrx",
            event = "event_loop_start",
            component = "engine_event_loop",
            "Starting event loop"
        );
        self.publish_state(None, Some(0));

        while self.running {
            let events = self.input.poll_events().await?;

            for event in events {
                global_event_registry().invoke(EventType::RawInput, &event);
                let event_start = Instant::now();

                // Track held keys for UI/state streaming.
                if event.pressed {
                    self.held_keys.insert(event.key);
                } else {
                    self.held_keys.remove(&event.key);
                }

                let now_us = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_micros() as u64)
                    .unwrap_or(0);
                let latency = now_us.saturating_sub(event.timestamp_us);
                self.publish_state(
                    Some(format!(
                        "{:?} {}",
                        event.key,
                        if event.pressed { "down" } else { "up" }
                    )),
                    Some(latency),
                );

                // Process event - either through buffer or directly
                if let Some(buffer) = &mut self.buffer {
                    // Coalescing enabled: push to buffer
                    if let Some(batch) = buffer.push(event) {
                        // Immediate flush triggered (batch size or modifier change)
                        self.process_batch(batch).await?;
                    }
                } else {
                    // No coalescing: process immediately
                    let output = Engine::process_event(self, &event);
                    global_event_registry().invoke(EventType::RawOutput, &output);
                    self.output_queue.enqueue(output);
                    self.output_queue.flush(&mut self.input).await?;
                }

                // Record event processing latency
                let elapsed = event_start.elapsed();
                let elapsed_micros = elapsed.as_micros() as u64;
                self.metrics
                    .record_latency(Operation::EventProcess, elapsed_micros);
                if let Some(otel) = self.otel_metrics() {
                    otel.record_processing_latency(elapsed);
                }
            }

            // Check for timeout-based flush when coalescing is enabled
            if let Some(buffer) = &mut self.buffer {
                if buffer.should_flush() {
                    let batch = buffer.flush();
                    if !batch.is_empty() {
                        self.process_batch(batch).await?;
                    }
                }
            }
        }

        debug!(
            service = "keyrx",
            event = "event_loop_stop",
            component = "engine_event_loop",
            "Event loop stopped"
        );
        Ok(())
    }

    /// Process a batch of events.
    ///
    /// This is called when the buffer flushes, either due to timeout, batch size,
    /// or modifier state changes. Each event in the batch is processed and outputs
    /// are sent to the input source.
    async fn process_batch(&mut self, events: Vec<InputEvent>) -> Result<(), KeyrxError> {
        for event in events {
            let output = Engine::process_event(self, &event);
            global_event_registry().invoke(EventType::RawOutput, &output);
            self.output_queue.enqueue(output);
        }
        self.output_queue.flush(&mut self.input).await
    }

    #[allow(deprecated)]
    fn publish_state(&self, event: Option<String>, latency_us: Option<u64>) {
        let layers = self
            .state
            .active_layers()
            .into_iter()
            .map(String::from)
            .collect();
        let modifiers = self
            .state
            .active_modifiers()
            .active_ids()
            .into_iter()
            .map(|id| id.to_string())
            .collect();
        let held: Vec<String> = self.held_keys.iter().map(|k| format!("{k:?}")).collect();

        publish_state_snapshot_legacy(
            layers,
            modifiers,
            held,
            Vec::new(), // Pending decisions are not yet tracked in the basic engine loop.
            event,
            latency_us,
            TimingConfig::default(),
        );
    }
}

impl<I, S, St> ProcessEvent for Engine<I, S, St>
where
    I: InputSource,
    S: ScriptRuntime,
    St: StateStore,
{
    fn process_event(&mut self, event: InputEvent) -> Vec<OutputAction> {
        // The Engine's process_event takes &InputEvent and returns OutputAction,
        // but the trait expects InputEvent and returns Vec<OutputAction>
        vec![Engine::process_event(self, &event)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::ResourceLimits;
    use crate::metrics::NoOpCollector;
    use crate::mocks::{MockInput, MockRuntime, MockState};
    use std::time::Duration;

    fn make_event(key: KeyCode, pressed: bool) -> InputEvent {
        InputEvent {
            key,
            pressed,
            timestamp_us: 0,
            device_id: None,
            is_repeat: false,
            is_synthetic: false,
            scan_code: 0,
            serial_number: None,
            vendor_id: None,
            product_id: None,
        }
    }

    #[tokio::test]
    async fn engine_without_coalescing_processes_events_immediately() {
        let input = MockInput::new();
        let script = MockRuntime::default();
        let state = MockState::new();
        let metrics = Arc::new(NoOpCollector::new());

        let mut engine = Engine::new(input, script, state, metrics);
        engine.start().await.unwrap();

        // Without coalescing, the buffer should be None
        assert!(engine.buffer.is_none());
    }

    #[tokio::test]
    async fn engine_with_coalescing_buffers_events() {
        let input = MockInput::new();
        let script = MockRuntime::default();
        let state = MockState::new();
        let metrics = Arc::new(NoOpCollector::new());

        let mut engine = Engine::new(input, script, state, metrics);

        // Enable coalescing with a large batch size to test buffering
        let config = CoalescingConfig {
            max_batch_size: 10,
            flush_timeout: Duration::from_millis(100),
            coalesce_repeats: false,
        };
        engine.enable_coalescing(config);

        // Verify buffer is created
        assert!(engine.buffer.is_some());
        assert_eq!(engine.buffer.as_ref().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn enable_and_disable_coalescing() {
        let input = MockInput::new();
        let script = MockRuntime::default();
        let state = MockState::new();
        let metrics = Arc::new(NoOpCollector::new());

        let mut engine = Engine::new(input, script, state, metrics);

        // Initially disabled
        assert!(engine.buffer.is_none());

        // Enable coalescing
        engine.enable_coalescing(CoalescingConfig::default());
        assert!(engine.buffer.is_some());

        // Disable coalescing
        engine.disable_coalescing();
        assert!(engine.buffer.is_none());
    }

    #[tokio::test]
    async fn process_batch_processes_all_events() {
        let input = MockInput::new();
        let script = MockRuntime::default();
        let state = MockState::new();
        let metrics = Arc::new(NoOpCollector::new());

        let mut engine = Engine::new(input, script, state, metrics);

        let events = vec![
            make_event(KeyCode::A, true),
            make_event(KeyCode::B, true),
            make_event(KeyCode::A, false),
        ];

        // Process batch should handle all events without error
        engine.process_batch(events).await.unwrap();
    }

    #[test]
    fn process_event_trait_implementation() {
        let input = MockInput::new();
        let script = MockRuntime::default();
        let state = MockState::new();
        let metrics = Arc::new(NoOpCollector::new());

        let mut engine = Engine::new(input, script, state, metrics);

        let event = make_event(KeyCode::A, true);
        let outputs = ProcessEvent::process_event(&mut engine, event);

        // Should return a Vec with one output
        assert_eq!(outputs.len(), 1);
    }

    #[test]
    fn process_event_passes_through_on_timeout() {
        let input = MockInput::new();
        let script = MockRuntime::new()
            .with_remap(KeyCode::CapsLock, KeyCode::Escape)
            .with_lookup_delay(Duration::from_millis(5));
        let state = MockState::new();
        let metrics = Arc::new(NoOpCollector::new());

        let mut engine = Engine::new(input, script, state, metrics);
        engine.set_resource_limits(ResourceLimits::new(
            Duration::from_millis(1),
            10 * 1024 * 1024,
            1000,
        ));

        let event = make_event(KeyCode::CapsLock, true);
        let outputs = ProcessEvent::process_event(&mut engine, event);

        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0], OutputAction::PassThrough);
    }
}
