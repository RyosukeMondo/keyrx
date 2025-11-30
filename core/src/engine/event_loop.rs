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
                "Skipping synthetic event: {:?} (pressed={})",
                event.key, event.pressed
            );
            return OutputAction::PassThrough;
        }

        let action = self.script.lookup_remap(event.key);

        match action {
            RemapAction::Remap(target_key) => {
                debug!(
                    "Remapping {:?} -> {:?} (pressed={})",
                    event.key, target_key, event.pressed
                );
                if event.pressed {
                    OutputAction::KeyDown(target_key)
                } else {
                    OutputAction::KeyUp(target_key)
                }
            }
            RemapAction::Block => {
                debug!("Blocking {:?} (pressed={})", event.key, event.pressed);
                OutputAction::Block
            }
            RemapAction::Pass => {
                debug!("Passing {:?} (pressed={})", event.key, event.pressed);
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
        debug!("Starting event loop");

        while self.running {
            let events = self.input.poll_events().await?;

            for event in events {
                let output = self.process_event(&event);
                self.input.send_output(output).await?;
            }
        }

        debug!("Event loop stopped");
        Ok(())
    }
}
