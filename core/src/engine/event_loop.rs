//! Main engine event loop.

use crate::engine::{InputEvent, OutputAction, RemapAction};
use crate::traits::{InputSource, ScriptRuntime, StateStore};
use anyhow::Result;
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
    running: bool,
}

impl<I, S, St> Engine<I, S, St>
where
    I: InputSource,
    S: ScriptRuntime,
    St: StateStore,
{
    /// Create a new engine with injected dependencies.
    pub fn new(input: I, script: S, state: St) -> Self {
        Self {
            input,
            script,
            state,
            running: false,
        }
    }

    /// Start the engine event loop.
    pub async fn start(&mut self) -> Result<()> {
        self.input.start().await?;
        self.running = true;
        Ok(())
    }

    /// Stop the engine.
    pub async fn stop(&mut self) -> Result<()> {
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

    /// Process a single input event and return the appropriate output action.
    ///
    /// Queries the script runtime's registry for remapping decisions and
    /// translates them to output actions. Handles both key-down and key-up events.
    ///
    /// **Synthetic event filtering**: Events with `is_synthetic = true` are
    /// automatically passed through without processing. This prevents infinite
    /// loops when our injected keys are recaptured by the input hook.
    pub fn process_event(&self, event: &InputEvent) -> OutputAction {
        // Skip synthetic events to prevent infinite loops from re-processing
        // keys we injected ourselves. These events should pass through unchanged.
        if event.is_synthetic {
            debug!(
                service = "keyrx",
                event = "skip_synthetic_event",
                component = "engine_event_loop",
                key = ?event.key,
                pressed = event.pressed,
                "Skipping synthetic input event"
            );
            return OutputAction::PassThrough;
        }

        let action = self.script.lookup_remap(event.key);

        match action {
            RemapAction::Remap(target_key) => {
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
            RemapAction::Block => {
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
            RemapAction::Pass => {
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
        }
    }

    /// Run the main event loop.
    ///
    /// Polls the input source for events, processes each through `process_event`,
    /// and sends the resulting output actions back to the OS. Runs until
    /// `stop()` is called or an error occurs.
    pub async fn run_loop(&mut self) -> Result<()> {
        debug!(
            service = "keyrx",
            event = "event_loop_start",
            component = "engine_event_loop",
            "Starting event loop"
        );

        while self.running {
            let events = self.input.poll_events().await?;

            for event in events {
                let output = self.process_event(&event);
                self.input.send_output(output).await?;
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
}
