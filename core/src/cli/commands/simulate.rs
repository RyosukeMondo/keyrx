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
    fn parse_input(&self) -> Result<Vec<InputEvent>> {
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

    pub async fn run(&self) -> Result<()> {
        // Parse input keys
        let events = self.parse_input()?;
        if events.is_empty() {
            bail!("No valid input keys provided");
        }

        // Create runtime and load script if provided
        let mut runtime = RhaiRuntime::new()?;
        if let Some(path) = &self.script_path {
            let path_str = path.to_string_lossy();
            runtime.load_file(&path_str)?;

            // Call on_init if defined
            if runtime.has_hook("on_init") {
                runtime.call_hook("on_init")?;
            }
        }

        // Create engine with mocks
        let mut mock_input = MockInput::new();
        for event in &events {
            mock_input.queue_event(event.clone());
        }

        let engine = Engine::new(mock_input, runtime, MockState::new());

        // Process events and collect results
        let mut results = Vec::new();
        let mut remapped = 0;
        let mut blocked = 0;
        let mut passed = 0;

        for event in &events {
            let output = engine.process_event(event);

            let output_str = match &output {
                OutputAction::KeyDown(k) | OutputAction::KeyUp(k) => {
                    if *k != event.key {
                        remapped += 1;
                    } else {
                        passed += 1;
                    }
                    k.name()
                }
                OutputAction::KeyTap(k) => {
                    remapped += 1;
                    k.name()
                }
                OutputAction::Block => {
                    blocked += 1;
                    "BLOCKED".to_string()
                }
                OutputAction::PassThrough => {
                    passed += 1;
                    event.key.name()
                }
            };

            results.push(SimulationResult {
                input: event.key.name(),
                output: output_str,
                pressed: event.pressed,
            });
        }

        let simulation_output = SimulationOutput {
            total: results.len(),
            results,
            remapped,
            blocked,
            passed,
        };

        // Output results
        self.output.data(&simulation_output)?;

        Ok(())
    }
}
