//! Device state management with bit vectors
//!
//! This module provides `DeviceState` for tracking modifier and lock state
//! using efficient 255-bit vectors, plus tap-hold processor state.

mod condition;
mod core;
mod shared;

// Re-export the main struct publicly
pub use self::core::DeviceState;
pub use shared::{SharedModifierState, SharedState};

#[cfg(test)]
mod tests;
