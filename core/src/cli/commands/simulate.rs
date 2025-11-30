//! Simulate command for headless testing.

use crate::cli::{OutputFormat, OutputWriter};
use crate::engine::{Engine, InputEvent, KeyCode, OutputAction};
use crate::mocks::{MockInput, MockState};
use crate::scripting::RhaiRuntime;
use crate::traits::ScriptRuntime;
use anyhow::{bail, Result};
use serde::Serialize;
use std::path::PathBuf;

/// Result of simulating a single key event.
#[derive(Debug, Serialize)]
pub struct SimulationResult {
    /// Original input key.
    pub input: String,
    /// Output action (remapped key, blocked, or passed).
    pub output: String,
    /// Whether this was a key press or release.
    pub pressed: bool,
}

/// Complete simulation output.
#[derive(Debug, Serialize)]
pub struct SimulationOutput {
    /// Results for each input event.
    pub results: Vec<SimulationResult>,
    /// Total events processed.
    pub total: usize,
    /// Number of remapped events.
    pub remapped: usize,
    /// Number of blocked events.
    pub blocked: usize,
    /// Number of passed events.
    pub passed: usize,
}

/// Simulate command for headless testing.
pub struct SimulateCommand {
    pub input_keys: String,
    pub script_path: Option<PathBuf>,
    pub output: OutputWriter,
}

impl SimulateCommand {
    pub fn new(input_keys: String, script_path: Option<PathBuf>, format: OutputFormat) -> Self {
        Self {
            input_keys,
            script_path,
            output: OutputWriter::new(format),
        }
    }

    /// Parse comma-separated key names into InputEvents.
    ///
    /// Each key name is converted to a key-down and key-up event pair.
    pub fn parse_input(&self) -> Result<Vec<InputEvent>> {
        let mut events = Vec::new();
        let mut timestamp = 0u64;

        for key_name in self.input_keys.split(',') {
            let key_name = key_name.trim();
            if key_name.is_empty() {
                continue;
            }

            let key = KeyCode::from_name(key_name)
                .ok_or_else(|| anyhow::anyhow!("Unknown key: '{}'", key_name))?;

            // Generate key-down and key-up events
            events.push(InputEvent::key_down(key, timestamp));
            timestamp += 1000; // 1ms between events
            events.push(InputEvent::key_up(key, timestamp));
            timestamp += 1000;
        }

        Ok(events)
    }

    /// Execute simulation and return the output.
    ///
    /// This is the core simulation logic that returns the result directly,
    /// useful for testing without stdout capture.
    pub async fn execute(&self) -> Result<SimulationOutput> {
        // Parse input keys
        let events = self.parse_input()?;
        if events.is_empty() {
            bail!("No valid input keys provided");
        }

        // Create runtime and load script if provided
        let runtime = self.create_runtime()?;

        // Create engine with mocks
        let engine = self.create_engine(&events, runtime);

        // Process events and collect results
        let (results, remapped, blocked, passed) = self.process_events(&engine, &events);

        Ok(SimulationOutput {
            total: results.len(),
            results,
            remapped,
            blocked,
            passed,
        })
    }

    /// Create and initialize the script runtime.
    fn create_runtime(&self) -> Result<RhaiRuntime> {
        let mut runtime = RhaiRuntime::new()?;
        if let Some(path) = &self.script_path {
            let path_str = path.to_string_lossy();
            runtime.load_file(&path_str)?;

            // Run top-level statements (e.g., remap/block/pass calls)
            runtime.run_script()?;

            // Call on_init if defined
            if runtime.has_hook("on_init") {
                runtime.call_hook("on_init")?;
            }
        }
        Ok(runtime)
    }

    /// Create the engine with mocked input events.
    fn create_engine(
        &self,
        events: &[InputEvent],
        runtime: RhaiRuntime,
    ) -> Engine<MockInput, RhaiRuntime, MockState> {
        let mut mock_input = MockInput::new();
        for event in events {
            mock_input.queue_event(event.clone());
        }
        Engine::new(mock_input, runtime, MockState::new())
    }

    /// Process events through the engine and collect results.
    fn process_events(
        &self,
        engine: &Engine<MockInput, RhaiRuntime, MockState>,
        events: &[InputEvent],
    ) -> (Vec<SimulationResult>, usize, usize, usize) {
        let mut results = Vec::new();
        let mut remapped = 0;
        let mut blocked = 0;
        let mut passed = 0;

        for event in events {
            let output = engine.process_event(event);
            let output_str =
                self.format_output_action(&output, event, &mut remapped, &mut blocked, &mut passed);

            results.push(SimulationResult {
                input: event.key.name(),
                output: output_str,
                pressed: event.pressed,
            });
        }

        (results, remapped, blocked, passed)
    }

    /// Format the output action and update counters.
    fn format_output_action(
        &self,
        output: &OutputAction,
        event: &InputEvent,
        remapped: &mut usize,
        blocked: &mut usize,
        passed: &mut usize,
    ) -> String {
        match output {
            OutputAction::KeyDown(k) | OutputAction::KeyUp(k) => {
                if *k != event.key {
                    *remapped += 1;
                } else {
                    *passed += 1;
                }
                k.name()
            }
            OutputAction::KeyTap(k) => {
                *remapped += 1;
                k.name()
            }
            OutputAction::Block => {
                *blocked += 1;
                "BLOCKED".to_string()
            }
            OutputAction::PassThrough => {
                *passed += 1;
                event.key.name()
            }
        }
    }

    /// Run the simulation and output results.
    pub async fn run(&self) -> Result<()> {
        let output = self.execute().await?;
        self.output.data(&output)?;
        Ok(())
    }
}
