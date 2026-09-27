//! Simulation Engine for deterministic keyboard event testing
//!
//! Provides deterministic replay of keyboard events for testing configurations
//! without physical hardware. Uses VirtualClock for timing to ensure reproducibility.
//!
//! Submodules split the implementation by responsibility:
//! - [`types`] — virtual clock, event/result shapes, and the error type
//! - [`scenarios`] — built-in test scenarios and their canned event sequences
//! - [`engine`] — the engine that loads a config and replays events

mod engine;
mod scenarios;
mod types;

#[cfg(test)]
mod tests;

pub use engine::SimulationEngine;
pub use scenarios::BuiltinScenario;
pub use types::{
    EventSequence, EventType, OutputEvent, ScenarioResult, SimulatedEvent, SimulationError,
    VirtualClock,
};

/// Maximum number of events allowed in a sequence (prevents DoS)
const MAX_EVENT_COUNT: usize = 100_000;

/// Maximum event file size in bytes (10MB)
const MAX_EVENT_FILE_SIZE: usize = 10 * 1024 * 1024;
