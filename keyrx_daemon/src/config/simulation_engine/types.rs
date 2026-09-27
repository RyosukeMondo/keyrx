//! Data types for the simulation engine: virtual clock, event/result shapes, and errors.

use serde::{Deserialize, Serialize};

use super::{MAX_EVENT_COUNT, MAX_EVENT_FILE_SIZE};

/// Virtual clock for deterministic timing in simulations
#[derive(Debug, Clone)]
pub struct VirtualClock {
    current_time_us: u64,
    #[allow(dead_code)]
    seed: u64,
}

impl VirtualClock {
    /// Create a new virtual clock with the given seed
    pub fn new(seed: u64) -> Self {
        Self {
            current_time_us: 0,
            seed,
        }
    }

    /// Advance the clock by the specified microseconds
    pub fn advance(&mut self, delta_us: u64) {
        self.current_time_us += delta_us;
    }

    /// Get the current time in microseconds
    pub fn now_us(&self) -> u64 {
        self.current_time_us
    }

    /// Reset the clock to zero
    pub fn reset(&mut self) {
        self.current_time_us = 0;
    }
}

/// Type of keyboard event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventType {
    Press,
    Release,
}

/// A simulated keyboard event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatedEvent {
    /// Optional device identifier for multi-device scenarios
    pub device_id: Option<String>,
    /// Timestamp in microseconds from start
    pub timestamp_us: u64,
    /// Key identifier (e.g., "A", "CapsLock", "Shift")
    pub key: String,
    /// Event type (press or release)
    pub event_type: EventType,
}

/// Output event from simulation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputEvent {
    /// Output key identifier
    pub key: String,
    /// Event type (press or release)
    pub event_type: EventType,
    /// Timestamp when event was generated (microseconds)
    pub timestamp_us: u64,
}

/// Sequence of events to replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSequence {
    /// List of events to replay
    pub events: Vec<SimulatedEvent>,
    /// Seed for deterministic behavior
    pub seed: u64,
}

/// Result of running a scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioResult {
    /// Scenario name
    pub scenario: String,
    /// Whether the scenario passed
    pub passed: bool,
    /// Input events
    pub input: Vec<SimulatedEvent>,
    /// Output events generated
    pub output: Vec<OutputEvent>,
    /// Optional error message if failed
    pub error: Option<String>,
}

/// Error types for simulation engine
#[derive(Debug, thiserror::Error)]
pub enum SimulationError {
    #[error("Failed to load KRX file: {0}")]
    LoadError(String),

    #[error("Event sequence too long: {0} events (max {MAX_EVENT_COUNT})")]
    TooManyEvents(usize),

    #[error("Event file too large: {0} bytes (max {MAX_EVENT_FILE_SIZE})")]
    FileTooLarge(usize),

    #[error("Invalid timestamp: {0} (must be >= 0)")]
    InvalidTimestamp(i64),

    #[error("Invalid event file: {0}")]
    InvalidEventFile(String),

    #[error("Scenario not found: {0}")]
    ScenarioNotFound(String),

    #[error("Simulation memory limit exceeded (max 1GB)")]
    MemoryLimitExceeded,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}
