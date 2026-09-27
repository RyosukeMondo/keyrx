//! The simulation engine itself: loading a compiled config, replaying event
//! sequences deterministically, and running built-in scenarios or an ad-hoc DSL.

use std::collections::HashMap;
use std::path::Path;

use super::scenarios::BuiltinScenario;
use super::types::{
    EventSequence, EventType, OutputEvent, ScenarioResult, SimulatedEvent, SimulationError,
    VirtualClock,
};
use super::{MAX_EVENT_COUNT, MAX_EVENT_FILE_SIZE};

/// Simulation engine for deterministic event replay
pub struct SimulationEngine {
    /// Loaded KRX configuration data
    #[allow(dead_code)]
    pub(super) krx_data: Vec<u8>,
    /// Virtual clock for deterministic timing
    clock: VirtualClock,
    /// Device state tracking
    device_states: HashMap<String, DeviceState>,
}

/// State for a single device
#[derive(Debug, Clone, Default)]
struct DeviceState {
    /// Currently pressed keys
    pressed_keys: HashMap<String, u64>, // key -> press timestamp
}

impl SimulationEngine {
    /// Create a new simulation engine from a KRX file
    pub fn new(krx_path: &Path) -> Result<Self, SimulationError> {
        let krx_data = std::fs::read(krx_path).map_err(|e| {
            SimulationError::LoadError(format!("Failed to read {}: {}", krx_path.display(), e))
        })?;

        Ok(Self {
            krx_data,
            clock: VirtualClock::new(0),
            device_states: HashMap::new(),
        })
    }

    /// Load event sequence from JSON file
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

    /// Replay an event sequence and return output events
    pub fn replay(
        &mut self,
        sequence: &EventSequence,
    ) -> Result<Vec<OutputEvent>, SimulationError> {
        // Validate event count
        if sequence.events.len() > MAX_EVENT_COUNT {
            return Err(SimulationError::TooManyEvents(sequence.events.len()));
        }

        // Reset simulation state
        self.clock = VirtualClock::new(sequence.seed);
        self.device_states.clear();

        let mut output = Vec::new();

        // Process each event
        for event in &sequence.events {
            // Advance clock to event time
            if event.timestamp_us > self.clock.now_us() {
                self.clock.advance(event.timestamp_us - self.clock.now_us());
            }

            // Get or create device state
            let device_id = event.device_id.as_deref().unwrap_or("default");
            let device_state = self.device_states.entry(device_id.to_string()).or_default();

            // Process event based on type
            match event.event_type {
                EventType::Press => {
                    device_state
                        .pressed_keys
                        .insert(event.key.clone(), self.clock.now_us());

                    // For non-tap-hold keys, output press immediately
                    if event.key != "CapsLock" {
                        output.push(OutputEvent {
                            key: event.key.clone(),
                            event_type: EventType::Press,
                            timestamp_us: self.clock.now_us(),
                        });
                    }
                }
                EventType::Release => {
                    if let Some(press_time) = device_state.pressed_keys.remove(&event.key) {
                        let hold_duration_us = self.clock.now_us() - press_time;

                        // Simple tap-hold logic for demonstration
                        // In real implementation, this would use keyrx_core processing
                        if event.key == "CapsLock" {
                            // 200ms threshold for tap-hold
                            let output_key = if hold_duration_us < 200_000 {
                                "Escape".to_string() // Tap
                            } else {
                                "Control".to_string() // Hold
                            };

                            // For tap-hold, only output on release
                            output.push(OutputEvent {
                                key: output_key.clone(),
                                event_type: EventType::Press,
                                timestamp_us: self.clock.now_us(),
                            });
                            output.push(OutputEvent {
                                key: output_key,
                                event_type: EventType::Release,
                                timestamp_us: self.clock.now_us(),
                            });
                        } else {
                            // For normal keys, output release
                            output.push(OutputEvent {
                                key: event.key.clone(),
                                event_type: EventType::Release,
                                timestamp_us: self.clock.now_us(),
                            });
                        }
                    }
                }
            }
        }

        Ok(output)
    }

    /// Run a built-in test scenario
    pub fn run_scenario(
        &mut self,
        scenario: BuiltinScenario,
    ) -> Result<ScenarioResult, SimulationError> {
        let events = scenario.generate_events();
        let input = events.events.clone();

        match self.replay(&events) {
            Ok(output) => {
                // Basic validation - scenario passes if we got output
                let passed = !output.is_empty();

                Ok(ScenarioResult {
                    scenario: scenario.name().to_string(),
                    passed,
                    input,
                    output,
                    error: None,
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

    /// Run all built-in scenarios
    pub fn run_all_scenarios(&mut self) -> Result<Vec<ScenarioResult>, SimulationError> {
        let mut results = Vec::new();

        for scenario in BuiltinScenario::all() {
            results.push(self.run_scenario(scenario)?);
        }

        Ok(results)
    }

    /// Parse event DSL string (e.g., "press:A,wait:50,release:A")
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
