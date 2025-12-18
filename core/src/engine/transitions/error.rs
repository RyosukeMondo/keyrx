//! Error types for state transitions.

use super::{StateKind, StateTransition};
use thiserror::Error;

/// Error type for state transition validation.
#[derive(Debug, Error, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TransitionError {
    /// The attempted transition is not valid from the current state.
    #[error("Invalid transition {transition_name} from state {from:?}")]
    Invalid {
        from: StateKind,
        transition_name: String,
    },

    /// The transition is valid, but a condition for it to proceed was not met.
    #[error("Transition condition failed for {transition_name} from state {from:?}: {reason}")]
    ConditionFailed {
        from: StateKind,
        transition_name: String,
        reason: String,
    },
}
