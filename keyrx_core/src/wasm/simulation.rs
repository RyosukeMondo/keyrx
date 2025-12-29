//! Event simulation types and logic for WASM module.
//!
//! This module provides the simulation engine that processes keyboard event
//! sequences and tracks state changes and performance metrics.

extern crate std;

use serde::{Deserialize, Serialize};
use std::vec::Vec;
use wasm_bindgen::prelude::*;

use crate::config::KeyCode;
use crate::runtime::{process_event, DeviceState, KeyEvent, KeyEventType, KeyLookup};

/// Input event sequence for simulation.
///
/// This structure defines a sequence of keyboard events to simulate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSequence {
    /// List of events to simulate
    pub events: Vec<SimKeyEvent>,
}

/// A single keyboard event for simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimKeyEvent {
    /// Key code (e.g., "A", "B", "LeftShift")
    pub keycode: String,
    /// Event type: "press" or "release"
    pub event_type: String,
    /// Timestamp in microseconds
    pub timestamp_us: u64,
}

/// Result of a simulation run.
///
/// Contains the full timeline of events, state changes, and performance metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    /// Timeline of all events (input and output)
    pub timeline: Vec<TimelineEntry>,
    /// Latency statistics in microseconds
    pub latency_stats: LatencyStats,
    /// Final state after simulation
    pub final_state: SimulationState,
}

/// Entry in the simulation timeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntry {
    /// Timestamp in microseconds
    pub timestamp_us: u64,
    /// Input event (if this was an input)
    pub input: Option<SimKeyEvent>,
    /// Output events generated from this input
    pub outputs: Vec<SimKeyEvent>,
    /// State snapshot after processing this event
    pub state: SimulationState,
    /// Processing latency for this event in microseconds
    pub latency_us: u64,
}

/// State snapshot during simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationState {
    /// Active modifiers (list of modifier IDs)
    pub active_modifiers: Vec<u8>,
    /// Active locks (list of lock IDs)
    pub active_locks: Vec<u8>,
    /// Current active layer (if any)
    pub active_layer: Option<String>,
}

/// Latency statistics for the simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyStats {
    /// Minimum latency in microseconds
    pub min_us: u64,
    /// Average latency in microseconds
    pub avg_us: u64,
    /// Maximum latency in microseconds
    pub max_us: u64,
    /// 95th percentile latency in microseconds
    pub p95_us: u64,
    /// 99th percentile latency in microseconds
    pub p99_us: u64,
}

/// Run simulation on event sequence.
///
/// This is the core simulation logic that processes events and tracks metrics.
pub fn run_simulation(
    lookup: &KeyLookup,
    event_sequence: &EventSequence,
) -> Result<SimulationResult, String> {
    use std::time::Instant;

    // Initialize state
    let mut state = DeviceState::new();
    let mut timeline = Vec::new();
    let mut latencies = Vec::new();

    for sim_event in &event_sequence.events {
        // Convert SimKeyEvent to KeyEvent
        let keycode = parse_keycode(&sim_event.keycode)?;
        let event_type = match sim_event.event_type.as_str() {
            "press" => KeyEventType::Press,
            "release" => KeyEventType::Release,
            _ => return Err(format!("Invalid event type: {}", sim_event.event_type)),
        };

        let key_event = match event_type {
            KeyEventType::Press => KeyEvent::press(keycode).with_timestamp(sim_event.timestamp_us),
            KeyEventType::Release => {
                KeyEvent::release(keycode).with_timestamp(sim_event.timestamp_us)
            }
        };

        // Measure processing latency
        let start = Instant::now();
        let output_events = process_event(key_event.clone(), lookup, &mut state);
        let latency_us = start.elapsed().as_micros() as u64;

        latencies.push(latency_us);

        // Capture state snapshot
        let state_snapshot = capture_state(&state);

        // Convert output events to SimKeyEvent
        let outputs: Vec<SimKeyEvent> = output_events
            .iter()
            .map(|e| SimKeyEvent {
                keycode: format!("{:?}", e.keycode()),
                event_type: match e.event_type() {
                    KeyEventType::Press => "press".to_string(),
                    KeyEventType::Release => "release".to_string(),
                },
                timestamp_us: e.timestamp_us(),
            })
            .collect();

        timeline.push(TimelineEntry {
            timestamp_us: sim_event.timestamp_us,
            input: Some(sim_event.clone()),
            outputs,
            state: state_snapshot,
            latency_us,
        });
    }

    // Calculate latency statistics
    let latency_stats = calculate_latency_stats(&latencies);

    // Capture final state
    let final_state = capture_state(&state);

    Ok(SimulationResult {
        timeline,
        latency_stats,
        final_state,
    })
}

/// Parse keycode string to KeyCode enum.
///
/// Supports common key names like "A", "B", "LeftShift", etc.
fn parse_keycode(keycode_str: &str) -> Result<KeyCode, String> {
    match keycode_str {
        "A" => Ok(KeyCode::A),
        "B" => Ok(KeyCode::B),
        "C" => Ok(KeyCode::C),
        "D" => Ok(KeyCode::D),
        "E" => Ok(KeyCode::E),
        "F" => Ok(KeyCode::F),
        "G" => Ok(KeyCode::G),
        "H" => Ok(KeyCode::H),
        "I" => Ok(KeyCode::I),
        "J" => Ok(KeyCode::J),
        "K" => Ok(KeyCode::K),
        "L" => Ok(KeyCode::L),
        "M" => Ok(KeyCode::M),
        "N" => Ok(KeyCode::N),
        "O" => Ok(KeyCode::O),
        "P" => Ok(KeyCode::P),
        "Q" => Ok(KeyCode::Q),
        "R" => Ok(KeyCode::R),
        "S" => Ok(KeyCode::S),
        "T" => Ok(KeyCode::T),
        "U" => Ok(KeyCode::U),
        "V" => Ok(KeyCode::V),
        "W" => Ok(KeyCode::W),
        "X" => Ok(KeyCode::X),
        "Y" => Ok(KeyCode::Y),
        "Z" => Ok(KeyCode::Z),
        _ => Err(format!("Unsupported keycode: {}", keycode_str)),
    }
}

/// Capture current simulation state.
fn capture_state(state: &DeviceState) -> SimulationState {
    // Extract active modifiers (IDs 0-254)
    let mut active_modifiers = Vec::new();
    for id in 0..255 {
        if state.is_modifier_active(id) {
            active_modifiers.push(id);
        }
    }

    // Extract active locks (IDs 0-254)
    let mut active_locks = Vec::new();
    for id in 0..255 {
        if state.is_lock_active(id) {
            active_locks.push(id);
        }
    }

    // TODO: Extract active layer from state once layer support is added
    let active_layer = None;

    SimulationState {
        active_modifiers,
        active_locks,
        active_layer,
    }
}

/// Calculate latency statistics from recorded latencies.
fn calculate_latency_stats(latencies: &[u64]) -> LatencyStats {
    if latencies.is_empty() {
        return LatencyStats {
            min_us: 0,
            avg_us: 0,
            max_us: 0,
            p95_us: 0,
            p99_us: 0,
        };
    }

    let mut sorted = latencies.to_vec();
    sorted.sort_unstable();

    let min_us = sorted[0];
    let max_us = sorted[sorted.len() - 1];
    let avg_us = sorted.iter().sum::<u64>() / sorted.len() as u64;

    let p95_idx = ((sorted.len() as f64) * 0.95) as usize;
    let p99_idx = ((sorted.len() as f64) * 0.99) as usize;

    let p95_us = sorted[p95_idx.min(sorted.len() - 1)];
    let p99_us = sorted[p99_idx.min(sorted.len() - 1)];

    LatencyStats {
        min_us,
        avg_us,
        max_us,
        p95_us,
        p99_us,
    }
}
