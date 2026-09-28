//! Event simulation types and logic for the WASM module.
//!
//! This is not a second remapping implementation: it drives the SAME
//! [`crate::runtime::remapper::Remapper`] + [`crate::simulate::run`]
//! deterministic driver the daemon's `simulate`/`test` CLI and REST/RPC
//! simulator use (see `keyrx_daemon/src/config/simulation_engine/engine.rs`),
//! just fed the events the browser sends. A discrepancy between "what the
//! daemon would do" and "what the WASM simulator shows" is a bug in the
//! shared engine, not two implementations that drifted apart.

extern crate std;

use serde::{Deserialize, Serialize};
use std::{format, string::String, string::ToString, vec::Vec};

use crate::config::DeviceConfig;
use crate::parser::validators::parse_physical_key;
use crate::runtime::Remapper;
use crate::simulate::{self, SimInput, SimOutput};

/// Input event sequence for simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSequence {
    /// List of events to simulate
    pub events: Vec<SimKeyEvent>,
}

/// A single keyboard event, as JSON to/from the browser. Used for both input
/// events and output events - same wire shape either way.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimKeyEvent {
    /// Key name (e.g. "A", "CapsLock", "VK_CapsLock", "LCtrl") - parsed with
    /// the SAME parser the `.rhai` config compiler uses
    /// ([`parse_physical_key`]), so anything a profile accepts is accepted
    /// here too.
    pub keycode: String,
    /// Event type: "press" or "release"
    pub event_type: String,
    /// Timestamp in microseconds
    pub timestamp_us: u64,
}

/// Result of a simulation run. Field names/shape match the UI's declared
/// `SimulationResult` TypeScript interface (`keyrx_ui/src/hooks/useWasm.ts`)
/// exactly, so no UI changes are needed to consume it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    /// One state snapshot per timeline step (one per input processed, plus
    /// one for any tap-hold timeout that fires on its own - see
    /// [`crate::simulate::run`]).
    pub states: Vec<StateTransition>,
    /// Every output event produced, across all steps, in order.
    pub outputs: Vec<SimKeyEvent>,
    /// Per-step processing latency in microseconds - always `0`.
    /// `std::time::Instant::now()` traps at runtime on
    /// `wasm32-unknown-unknown` (verified empirically: it compiles to an
    /// `unreachable` instruction when there is no JS shim under it), so
    /// wall-clock timing is not measured in the browser simulator.
    pub latency: Vec<u64>,
    /// State after the final step.
    pub final_state: SimulationState,
}

/// One state snapshot in the timeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    /// Timestamp in microseconds
    pub timestamp_us: u64,
    /// Active modifiers (list of modifier IDs) - shared across every device
    /// the config routes (see `runtime::remapper` docs).
    pub active_modifiers: Vec<u8>,
    /// Active locks (list of lock IDs)
    pub active_locks: Vec<u8>,
    /// Current active layer, formatted like [`crate::wasm::get_state`]
    /// formats modifiers (`"MD_XX"`) - a layer IS a modifier id, there is no
    /// separate layer-name concept anywhere in `keyrx_core`.
    pub active_layer: Option<String>,
}

/// State snapshot used by `get_state()` (matches `DaemonStateResponse`'s
/// `active_layer`/modifier/lock fields).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationState {
    /// Active modifiers (list of modifier IDs)
    pub active_modifiers: Vec<u8>,
    /// Active locks (list of lock IDs)
    pub active_locks: Vec<u8>,
    /// Current active layer (if any)
    pub active_layer: Option<String>,
}

/// Runs `event_sequence` through a fresh [`Remapper`] built from ALL of
/// `devices`'s blocks (not just the first - routing then behaves exactly
/// like the daemon's), driven by [`crate::simulate::run`]. There is no real
/// platform device list in a browser simulation, so every input is routed
/// with no device id (the wildcard/no-identity block).
pub fn run_simulation(
    devices: &[DeviceConfig],
    event_sequence: &EventSequence,
) -> Result<SimulationResult, String> {
    let inputs = event_sequence
        .events
        .iter()
        .map(to_sim_input)
        .collect::<Result<Vec<_>, _>>()?;
    let end_us = event_sequence
        .events
        .iter()
        .map(|e| e.timestamp_us)
        .max()
        .unwrap_or(0);

    let mut remapper = Remapper::from_blocks(devices);
    let steps = simulate::run(&mut remapper, &inputs, end_us, |id| {
        std::vec![id.to_string()]
    });

    let mut states = Vec::with_capacity(steps.len());
    let mut outputs = Vec::new();
    for step in &steps {
        // An input step's timestamp is its input's; a tap-hold-timeout step
        // (input: None) has none of its own, but its fired events (if any)
        // carry the tick time, and the one timeout step with no fired events
        // is always the trailing end-of-run tick (see `simulate::run`).
        let timestamp_us = step
            .input
            .as_ref()
            .map(|i| i.at_us)
            .or_else(|| step.outputs.first().map(|o| o.at_us))
            .unwrap_or(end_us);
        states.push(StateTransition {
            timestamp_us,
            active_modifiers: step.active_modifiers.clone(),
            active_locks: step.active_locks.clone(),
            active_layer: step.active_layer.map(format_layer),
        });
        outputs.extend(step.outputs.iter().map(to_sim_key_event));
    }

    let final_state = states
        .last()
        .map(|s| SimulationState {
            active_modifiers: s.active_modifiers.clone(),
            active_locks: s.active_locks.clone(),
            active_layer: s.active_layer.clone(),
        })
        .unwrap_or(SimulationState {
            active_modifiers: Vec::new(),
            active_locks: Vec::new(),
            active_layer: None,
        });

    let latency = std::vec![0u64; states.len()];

    Ok(SimulationResult {
        states,
        outputs,
        latency,
        final_state,
    })
}

/// Formats an active layer/modifier id the same way `get_state()` formats
/// active modifiers.
fn format_layer(id: u8) -> String {
    format!("MD_{:02X}", id)
}

/// Parses a `SimKeyEvent`'s string key name with the SAME parser the DSL
/// config compiler uses, so "CapsLock", "VK_CapsLock", "LCtrl", "A", etc. all
/// resolve exactly as they would in a `.rhai` profile.
fn to_sim_input(event: &SimKeyEvent) -> Result<SimInput, String> {
    let key = parse_physical_key(&event.keycode)
        .map_err(|e| format!("invalid key '{}': {e}", event.keycode))?;
    let press = match event.event_type.as_str() {
        "press" => true,
        "release" => false,
        other => return Err(format!("Invalid event type: {other}")),
    };
    Ok(SimInput {
        at_us: event.timestamp_us,
        device: None,
        press,
        key,
    })
}

fn to_sim_key_event(output: &SimOutput) -> SimKeyEvent {
    SimKeyEvent {
        keycode: format!("{:?}", output.key),
        event_type: if output.press { "press" } else { "release" }.to_string(),
        timestamp_us: output.at_us,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DeviceIdentifier, KeyCode, KeyMapping};
    use std::vec;

    fn block(pattern: &str, from: KeyCode, to: KeyCode) -> DeviceConfig {
        DeviceConfig {
            identifier: DeviceIdentifier {
                pattern: pattern.to_string(),
            },
            mappings: vec![KeyMapping::simple(from, to)],
        }
    }

    fn event(keycode: &str, event_type: &str, timestamp_us: u64) -> SimKeyEvent {
        SimKeyEvent {
            keycode: keycode.to_string(),
            event_type: event_type.to_string(),
            timestamp_us,
        }
    }

    #[test]
    fn run_simulation_maps_press_and_release() {
        let devices = vec![block("*", KeyCode::A, KeyCode::B)];
        let events = EventSequence {
            events: vec![event("A", "press", 0), event("A", "release", 50_000)],
        };

        let result = run_simulation(&devices, &events).expect("simulation should succeed");

        assert_eq!(result.outputs.len(), 2);
        assert_eq!(result.outputs[0].keycode, "B");
        assert_eq!(result.outputs[0].event_type, "press");
        assert_eq!(result.outputs[1].event_type, "release");
        // Trailing forced tick adds one more state snapshot beyond the two
        // input steps.
        assert_eq!(result.states.len(), 3);
        assert_eq!(result.states.last().unwrap().timestamp_us, 50_000);
        assert_eq!(result.latency, vec![0, 0, 0]);
    }

    #[test]
    fn run_simulation_uses_every_block_not_just_the_first() {
        // A specific-pattern block first, then a wildcard fallback: with no
        // real device identity, only the wildcard block can ever match. If
        // the engine only loaded `devices.first()`, the wildcard block (and
        // its A->B mapping) would never be reachable.
        let devices = vec![
            block("*numpad*", KeyCode::A, KeyCode::X),
            block("*", KeyCode::A, KeyCode::B),
        ];
        let events = EventSequence {
            events: vec![event("A", "press", 0)],
        };

        let result = run_simulation(&devices, &events).expect("simulation should succeed");

        assert_eq!(result.outputs.len(), 1);
        assert_eq!(result.outputs[0].keycode, "B");
    }

    #[test]
    fn run_simulation_accepts_named_keys_not_just_single_letters() {
        let devices = vec![block("*", KeyCode::CapsLock, KeyCode::Escape)];
        let events = EventSequence {
            events: vec![event("CapsLock", "press", 0)],
        };

        let result = run_simulation(&devices, &events).expect("simulation should succeed");

        assert_eq!(result.outputs.len(), 1);
        assert_eq!(result.outputs[0].keycode, "Escape");
    }

    #[test]
    fn run_simulation_rejects_unknown_key_names() {
        let devices = vec![block("*", KeyCode::A, KeyCode::B)];
        let events = EventSequence {
            events: vec![event("NotAKey", "press", 0)],
        };

        let err = run_simulation(&devices, &events).expect_err("unknown key should error");
        assert!(err.contains("NotAKey"));
    }

    #[test]
    fn run_simulation_rejects_unknown_event_type() {
        let devices = vec![block("*", KeyCode::A, KeyCode::B)];
        let events = EventSequence {
            events: vec![event("A", "double-click", 0)],
        };

        assert!(run_simulation(&devices, &events).is_err());
    }

    #[test]
    fn run_simulation_with_no_events_returns_empty_result() {
        let devices = vec![block("*", KeyCode::A, KeyCode::B)];
        let events = EventSequence { events: Vec::new() };

        let result = run_simulation(&devices, &events).expect("simulation should succeed");

        assert!(result.states.is_empty());
        assert!(result.outputs.is_empty());
        assert!(result.latency.is_empty());
        assert_eq!(result.final_state.active_modifiers, Vec::<u8>::new());
        assert_eq!(result.final_state.active_layer, None);
    }

    #[test]
    fn format_layer_matches_get_state_modifier_formatting() {
        assert_eq!(format_layer(0), "MD_00");
        assert_eq!(format_layer(10), "MD_0A");
    }
}
