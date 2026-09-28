//! The simulation engine itself: loading a compiled config, and driving
//! `keyrx_core::simulate` (the SAME remapping engine the daemon's live event
//! loop uses) deterministically with virtual time.

use std::path::Path;

use keyrx_core::config::DeviceConfig;
use keyrx_core::parser::validators::parse_physical_key;
use keyrx_core::runtime::Remapper;
use keyrx_core::simulate::{self, SimInput, SimStep};

use super::scenarios::BuiltinScenario;
use super::types::{
    EventSequence, EventType, OutputEvent, ScenarioResult, SimulatedEvent, SimulationError,
};
use super::{MAX_EVENT_COUNT, MAX_EVENT_FILE_SIZE};

/// Simulation engine: loads a compiled config once, then replays event
/// sequences against a fresh [`Remapper`] each time (so `replay` is
/// deterministic and side-effect-free between calls, matching
/// `test_replay_deterministic`).
pub struct SimulationEngine {
    devices: Vec<DeviceConfig>,
}

impl SimulationEngine {
    /// Create a new simulation engine from a KRX file.
    pub fn new(krx_path: &Path) -> Result<Self, SimulationError> {
        let config = crate::config_loader::load_config(krx_path)
            .map_err(|e| SimulationError::LoadError(e.to_string()))?;
        Ok(Self {
            devices: config.devices,
        })
    }

    /// Load event sequence from JSON file.
    pub fn load_events_from_file(path: &Path) -> Result<EventSequence, SimulationError> {
        // Check file size
        let metadata = std::fs::metadata(path)?;
        if metadata.len() > MAX_EVENT_FILE_SIZE as u64 {
            return Err(SimulationError::FileTooLarge(metadata.len() as usize));
        }

        let contents = std::fs::read_to_string(path)?;
        let sequence: EventSequence = serde_json::from_str(&contents)?;

        // Validate event count
        if sequence.events.len() > MAX_EVENT_COUNT {
            return Err(SimulationError::TooManyEvents(sequence.events.len()));
        }

        // Validate timestamps
        for event in &sequence.events {
            if event.timestamp_us > i64::MAX as u64 {
                return Err(SimulationError::InvalidTimestamp(event.timestamp_us as i64));
            }
        }

        Ok(sequence)
    }

    /// Runs `sequence` through a fresh [`Remapper`] built from the loaded
    /// config and returns the full simulation timeline (input, real
    /// engine output, mapping kind, and active layer/modifiers/locks per
    /// step - see [`SimStep`]).
    fn run_steps(&self, sequence: &EventSequence) -> Result<Vec<SimStep>, SimulationError> {
        if sequence.events.len() > MAX_EVENT_COUNT {
            return Err(SimulationError::TooManyEvents(sequence.events.len()));
        }

        let inputs = sequence
            .events
            .iter()
            .map(to_sim_input)
            .collect::<Result<Vec<_>, _>>()?;
        let end_us = sequence
            .events
            .iter()
            .map(|e| e.timestamp_us)
            .max()
            .unwrap_or(0);

        let mut remapper = Remapper::from_blocks(&self.devices);
        Ok(simulate::run(&mut remapper, &inputs, end_us, |id| {
            vec![id.to_string()]
        }))
    }

    /// Replay an event sequence and return the engine's real output events.
    pub fn replay(
        &mut self,
        sequence: &EventSequence,
    ) -> Result<Vec<OutputEvent>, SimulationError> {
        let steps = self.run_steps(sequence)?;
        Ok(steps
            .iter()
            .flat_map(|s| s.outputs.iter().map(to_output_event))
            .collect())
    }

    /// Run a built-in scenario. Since the same scenario is replayed against
    /// whatever profile is loaded, there is no config-specific expected
    /// output to assert - `passed` instead checks an invariant that holds
    /// for ANY config: no key is left stuck (held) at the end of the run.
    /// `output` is the real engine trace, for inspection either way.
    pub fn run_scenario(
        &mut self,
        scenario: BuiltinScenario,
    ) -> Result<ScenarioResult, SimulationError> {
        let events = scenario.generate_events();
        let input = events.events.clone();

        match self.run_steps(&events) {
            Ok(steps) => {
                let output: Vec<OutputEvent> = steps
                    .iter()
                    .flat_map(|s| s.outputs.iter().map(to_output_event))
                    .collect();
                let stuck = simulate::stuck_keys(&steps);
                let passed = stuck.is_empty();
                let error = (!passed)
                    .then(|| format!("key(s) left stuck (pressed, never released): {stuck:?}"));
                Ok(ScenarioResult {
                    scenario: scenario.name().to_string(),
                    passed,
                    input,
                    output,
                    error,
                })
            }
            Err(e) => Ok(ScenarioResult {
                scenario: scenario.name().to_string(),
                passed: false,
                input,
                output: Vec::new(),
                error: Some(e.to_string()),
            }),
        }
    }

    /// Run all built-in scenarios.
    pub fn run_all_scenarios(&mut self) -> Result<Vec<ScenarioResult>, SimulationError> {
        let mut results = Vec::new();

        for scenario in BuiltinScenario::all() {
            results.push(self.run_scenario(scenario)?);
        }

        Ok(results)
    }

    /// Parse event DSL string (e.g., "press:A,wait:50,release:A").
    pub fn parse_event_dsl(dsl: &str, seed: u64) -> Result<EventSequence, SimulationError> {
        let mut events = Vec::new();
        let mut current_time_us = 0u64;

        for token in dsl.split(',') {
            let token = token.trim();
            let parts: Vec<&str> = token.split(':').collect();

            if parts.len() != 2 {
                return Err(SimulationError::InvalidEventFile(format!(
                    "Invalid DSL token: '{}' (expected format 'action:value')",
                    token
                )));
            }

            let action = parts[0];
            let value = parts[1];

            match action {
                "press" => {
                    events.push(SimulatedEvent {
                        device_id: None,
                        timestamp_us: current_time_us,
                        key: value.to_string(),
                        event_type: EventType::Press,
                    });
                }
                "release" => {
                    events.push(SimulatedEvent {
                        device_id: None,
                        timestamp_us: current_time_us,
                        key: value.to_string(),
                        event_type: EventType::Release,
                    });
                }
                "wait" => {
                    let wait_ms: u64 = value.parse().map_err(|_| {
                        SimulationError::InvalidEventFile(format!(
                            "Invalid wait time: '{}' (expected number)",
                            value
                        ))
                    })?;
                    current_time_us += wait_ms * 1000; // Convert ms to us
                }
                _ => {
                    return Err(SimulationError::InvalidEventFile(format!(
                        "Unknown action: '{}' (expected press, release, or wait)",
                        action
                    )));
                }
            }
        }

        Ok(EventSequence { events, seed })
    }
}

/// Parses a `SimulatedEvent`'s string key name with the SAME parser the DSL
/// config compiler uses (`keyrx_core::parser::validators`), so "CapsLock",
/// "VK_CapsLock", "LCtrl", "A", etc. all resolve exactly as they would in a
/// `.rhai` profile.
fn to_sim_input(event: &SimulatedEvent) -> Result<SimInput, SimulationError> {
    let key = parse_physical_key(&event.key).map_err(|e| {
        SimulationError::InvalidEventFile(format!("invalid key '{}': {e}", event.key))
    })?;
    Ok(SimInput {
        at_us: event.timestamp_us,
        device: event.device_id.clone(),
        press: matches!(event.event_type, EventType::Press),
        key,
    })
}

fn to_output_event(output: &simulate::SimOutput) -> OutputEvent {
    OutputEvent {
        key: format!("{:?}", output.key),
        event_type: if output.press {
            EventType::Press
        } else {
            EventType::Release
        },
        timestamp_us: output.at_us,
    }
}
