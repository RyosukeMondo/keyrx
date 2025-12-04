//! State machine transitions module.
//!
//! This module provides the core state machine infrastructure for KeyRX:
//! - Explicit state transitions (all state changes must be enumerated)
//! - State validation and invariant checking
//! - Transition logging for debugging and replay
//! - State graph for enforcing valid transitions

pub mod graph;
pub mod invariant;
pub mod invariants;
pub mod state_kind;
pub mod transition;

pub use graph::{InvalidTransition, StateGraph};
pub use invariant::{Invariant, InvariantViolation};
pub use invariants::{
    KeyTimestampsMonotonic, LayerStackNotEmpty, NoOrphanedModifiers, PendingQueueBounds,
};
pub use state_kind::StateKind;
pub use transition::{StateTransition, TransitionCategory};

// Re-export decision types with different names to avoid conflicts
pub use transition::{
    DecisionKind as TransitionDecisionKind, DecisionResolution as TransitionDecisionResolution,
};
