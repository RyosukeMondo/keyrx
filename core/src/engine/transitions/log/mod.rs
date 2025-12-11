//! State transition logging for debugging and replay.
//!
//! This module provides types for logging state transitions with complete
//! before/after state snapshots. This enables:
//! - Debugging state machine behavior
//! - Replay of transition sequences
//! - Analysis of state evolution
//! - Export of transition history
//!
//! # Feature Flag
//!
//! The logging functionality can be disabled at compile time using the
//! `transition-logging` feature flag. When disabled, all logging code is
//! removed at compile time with zero runtime overhead.
//!
//! To disable transition logging:
//! ```toml
//! [dependencies]
//! keyrx_core = { version = "...", default-features = false, features = [...] }
//! ```
//!
//! # Module Structure
//!
//! - [`entry`] - The `TransitionEntry` type for capturing individual transitions
//! - [`ring_buffer`] - The `TransitionLog` ring buffer (when feature enabled)
//! - [`stub`] - Zero-overhead stub implementation (when feature disabled)

mod entry;

#[cfg(feature = "transition-logging")]
mod ring_buffer;

#[cfg(not(feature = "transition-logging"))]
mod stub;

#[cfg(test)]
mod tests;

// Re-export the entry type (always available)
pub use entry::TransitionEntry;

// Re-export TransitionLog from the appropriate module based on feature flag
#[cfg(feature = "transition-logging")]
pub use ring_buffer::TransitionLog;

#[cfg(not(feature = "transition-logging"))]
pub use stub::TransitionLog;
