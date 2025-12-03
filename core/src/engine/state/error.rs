//! Error types for state operations.
//!
//! These errors occur when invalid mutations are attempted, such as:
//! - Popping layers when only the base layer remains
//! - Invalid modifier IDs
//! - Conflicting pending decisions
//! - Batch mutations with inconsistent state

use crate::engine::state::LayerId;
use crate::engine::KeyCode;
use thiserror::Error;

/// Errors that can occur during state mutations.
///
/// StateErrors indicate invalid operations that violate state invariants.
/// These are programming errors or invalid input, not expected runtime conditions.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[allow(dead_code)] // Will be used when EngineState is implemented
pub enum StateError {
    // === Key State Errors ===
    /// Attempted to release a key that wasn't pressed.
    #[error("key {key:?} not pressed (cannot release)")]
    KeyNotPressed { key: KeyCode },

    /// Attempted to press a key that's already pressed.
    #[error("key {key:?} already pressed")]
    KeyAlreadyPressed { key: KeyCode },

    /// Invalid timestamp (e.g., going backwards in time).
    #[error("invalid timestamp: {timestamp_us} (expected >= {expected_us})")]
    InvalidTimestamp { timestamp_us: u64, expected_us: u64 },

    // === Layer State Errors ===
    /// Attempted to pop the base layer.
    #[error("cannot pop base layer (layer stack must have at least one layer)")]
    CannotPopBaseLayer,

    /// Layer stack is empty (should never happen).
    #[error("layer stack is empty (internal error)")]
    EmptyLayerStack,

    /// Invalid layer ID.
    #[error("invalid layer ID: {layer_id}")]
    InvalidLayerId { layer_id: LayerId },

    /// Layer stack overflow (too many layers pushed).
    #[error("layer stack overflow (max depth: {max_depth})")]
    LayerStackOverflow { max_depth: usize },

    // === Modifier State Errors ===
    /// Invalid modifier ID (must be 0-254; 255 reserved).
    #[error("invalid modifier ID: {modifier_id} (must be 0-254)")]
    InvalidModifierId { modifier_id: u8 },

    /// Attempted to deactivate a modifier that isn't active.
    #[error("modifier {modifier_id} not active (cannot deactivate)")]
    ModifierNotActive { modifier_id: u8 },

    /// Attempted to activate a modifier that's already active.
    #[error("modifier {modifier_id} already active")]
    ModifierAlreadyActive { modifier_id: u8 },

    // === Pending Decision Errors ===
    /// Conflicting pending decision for the same key.
    #[error("pending decision already exists for key {key:?}")]
    PendingDecisionConflict { key: KeyCode },

    /// Invalid pending decision timeout.
    #[error("invalid timeout: {timeout_ms}ms (must be > 0)")]
    InvalidTimeout { timeout_ms: u64 },

    /// Pending decision not found.
    #[error("no pending decision for key {key:?}")]
    PendingDecisionNotFound { key: KeyCode },

    /// Pending queue is full.
    #[error("pending queue is full (max: {max_size})")]
    PendingQueueFull { max_size: usize },

    // === Batch Mutation Errors ===
    /// Batch mutation is empty.
    #[error("batch mutation is empty (must contain at least one mutation)")]
    EmptyBatch,

    /// Batch mutation failed; state rolled back.
    #[error("batch mutation failed at index {index}: {error}")]
    BatchFailed {
        index: usize,
        error: Box<StateError>,
    },

    /// Nested batch mutations are not allowed.
    #[error("nested batch mutations are not allowed")]
    NestedBatch,

    // === General Errors ===
    /// State invariant violated.
    #[error("state invariant violated: {message}")]
    InvariantViolation { message: String },

    /// Internal state corruption detected.
    #[error("internal state corruption: {message}")]
    Corruption { message: String },
}

impl StateError {
    /// Returns true if this error is a programming bug (internal invariant violation).
    #[allow(dead_code)] // Will be used when EngineState is implemented
    pub fn is_internal_error(&self) -> bool {
        matches!(
            self,
            StateError::EmptyLayerStack
                | StateError::InvariantViolation { .. }
                | StateError::Corruption { .. }
        )
    }

    /// Returns true if this error is due to invalid input.
    #[allow(dead_code)] // Will be used when EngineState is implemented
    pub fn is_invalid_input(&self) -> bool {
        matches!(
            self,
            StateError::InvalidTimestamp { .. }
                | StateError::InvalidLayerId { .. }
                | StateError::InvalidModifierId { .. }
                | StateError::InvalidTimeout { .. }
                | StateError::EmptyBatch
                | StateError::NestedBatch
        )
    }

    /// Returns true if this error indicates a state conflict.
    #[allow(dead_code)] // Will be used when EngineState is implemented
    pub fn is_conflict(&self) -> bool {
        matches!(
            self,
            StateError::KeyAlreadyPressed { .. }
                | StateError::KeyNotPressed { .. }
                | StateError::ModifierAlreadyActive { .. }
                | StateError::ModifierNotActive { .. }
                | StateError::PendingDecisionConflict { .. }
        )
    }

    /// Returns true if this error indicates a capacity limit reached.
    #[allow(dead_code)] // Will be used when EngineState is implemented
    pub fn is_capacity_error(&self) -> bool {
        matches!(
            self,
            StateError::LayerStackOverflow { .. } | StateError::PendingQueueFull { .. }
        )
    }
}

/// Result type for state operations.
#[allow(dead_code)] // Will be used when EngineState is implemented
pub type StateResult<T> = Result<T, StateError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display() {
        let err = StateError::KeyNotPressed { key: KeyCode::A };
        assert_eq!(err.to_string(), "key A not pressed (cannot release)");

        let err = StateError::CannotPopBaseLayer;
        assert_eq!(
            err.to_string(),
            "cannot pop base layer (layer stack must have at least one layer)"
        );

        let err = StateError::InvalidModifierId { modifier_id: 255 };
        assert_eq!(err.to_string(), "invalid modifier ID: 255 (must be 0-254)");
    }

    #[test]
    fn error_classification() {
        // Internal errors
        let err = StateError::EmptyLayerStack;
        assert!(err.is_internal_error());
        assert!(!err.is_invalid_input());

        let err = StateError::InvariantViolation {
            message: "test".to_string(),
        };
        assert!(err.is_internal_error());

        // Invalid input
        let err = StateError::InvalidTimestamp {
            timestamp_us: 100,
            expected_us: 200,
        };
        assert!(err.is_invalid_input());
        assert!(!err.is_internal_error());

        let err = StateError::EmptyBatch;
        assert!(err.is_invalid_input());

        // Conflicts
        let err = StateError::KeyAlreadyPressed { key: KeyCode::A };
        assert!(err.is_conflict());
        assert!(!err.is_invalid_input());

        let err = StateError::ModifierNotActive { modifier_id: 5 };
        assert!(err.is_conflict());

        // Capacity errors
        let err = StateError::LayerStackOverflow { max_depth: 10 };
        assert!(err.is_capacity_error());
        assert!(!err.is_conflict());

        let err = StateError::PendingQueueFull { max_size: 100 };
        assert!(err.is_capacity_error());
    }

    #[test]
    fn error_cloning_and_equality() {
        let err1 = StateError::KeyNotPressed { key: KeyCode::A };
        let err2 = err1.clone();
        assert_eq!(err1, err2);

        let err3 = StateError::KeyNotPressed { key: KeyCode::B };
        assert_ne!(err1, err3);
    }

    #[test]
    fn batch_error_boxing() {
        let inner_error = StateError::InvalidModifierId { modifier_id: 255 };
        let batch_error = StateError::BatchFailed {
            index: 2,
            error: Box::new(inner_error.clone()),
        };

        assert!(batch_error.to_string().contains("index 2"));
        assert!(!batch_error.is_internal_error());
    }

    #[test]
    fn state_result_type() {
        fn example_operation() -> StateResult<()> {
            Err(StateError::CannotPopBaseLayer)
        }

        let result = example_operation();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StateError::CannotPopBaseLayer);
    }

    #[test]
    fn error_context() {
        let corruption_err = StateError::Corruption {
            message: "key state bitmap corrupted".to_string(),
        };
        assert!(corruption_err.is_internal_error());
        assert!(corruption_err.to_string().contains("corrupted"));

        let invariant_err = StateError::InvariantViolation {
            message: "active modifier not in modifier state".to_string(),
        };
        assert!(invariant_err.is_internal_error());
        assert!(invariant_err.to_string().contains("invariant"));
    }
}
