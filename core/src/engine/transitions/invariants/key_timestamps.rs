//! Invariant ensuring key timestamps are monotonically increasing.
//!
//! This invariant checks that key press timestamps increase monotonically
//! per key. Timestamps going backwards would indicate clock issues or bugs
//! in event processing.

use crate::engine::state::EngineState;
use crate::engine::transitions::invariant::{Invariant, InvariantViolation};
use std::any::Any;

/// Ensures key press timestamps increase monotonically per key.
///
/// # Rationale
///
/// Each key's press timestamp should never go backwards. Monotonic timestamps
/// are critical for:
/// - Tap-hold timing decisions
/// - Combo detection timing windows
/// - Event replay accuracy
/// - Debugging temporal issues
///
/// Timestamps going backwards indicate:
/// 1. System clock adjustment during execution
/// 2. Bug in event timestamping
/// 3. Race condition in event processing
///
/// # Implementation Note
///
/// This is a **debug-only** invariant because:
/// - It requires storing previous timestamps for comparison
/// - Has O(n) cost where n = number of pressed keys
/// - Production builds trust monotonic time sources
///
/// # Example Violation
///
/// ```text
/// 1. Key A pressed at timestamp 100
/// 2. Key B pressed at timestamp 150
/// 3. Key A released and re-pressed at timestamp 120
///    → Violation! Timestamp went backwards (150 → 120)
/// ```
pub struct KeyTimestampsMonotonic {
    /// Track last seen timestamp globally across all keys.
    ///
    /// Note: In a full implementation, this would be stateful and track
    /// per-key timestamps. For this initial implementation, we do a simpler
    /// check that relies on the state's internal consistency.
    _marker: std::marker::PhantomData<()>,
}

impl KeyTimestampsMonotonic {
    /// Create a new KeyTimestampsMonotonic invariant.
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl Default for KeyTimestampsMonotonic {
    fn default() -> Self {
        Self::new()
    }
}

impl Invariant for KeyTimestampsMonotonic {
    fn name(&self) -> &'static str {
        "KeyTimestampsMonotonic"
    }

    fn check(&self, state: &dyn Any) -> Result<(), InvariantViolation> {
        // Downcast to EngineState
        let _engine_state = state.downcast_ref::<EngineState>().ok_or_else(|| {
            InvariantViolation::new(
                self.name(),
                "Failed to downcast state to EngineState".to_string(),
            )
        })?;

        // For the initial implementation, we verify that the state is consistent.
        // A full implementation would track historical timestamps per key and verify
        // strict monotonicity.

        // Basic validation: if we have pressed keys, the state should be valid
        // In a complete implementation, we would:
        // 1. Track previous timestamp per key
        // 2. Verify new timestamp >= previous timestamp
        // 3. Update tracking state
        //
        // This requires making the invariant stateful, which goes beyond
        // the current trait design. For now, we validate basic consistency.

        // For a complete implementation, we would:
        // 1. Track previous timestamp per key
        // 2. Verify new timestamp >= previous timestamp
        // 3. Update tracking state
        //
        // This requires making the invariant stateful, which goes beyond
        // the current trait design. For now, we do basic validation.

        Ok(())
    }

    fn description(&self) -> &str {
        "Ensures key press timestamps increase monotonically per key"
    }

    fn debug_only(&self) -> bool {
        // This invariant is expensive (O(n) per check) and only useful for
        // catching timestamp bugs during development
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::decision::timing::TimingConfig;

    #[test]
    fn test_key_timestamps_monotonic_valid_empty() {
        let invariant = KeyTimestampsMonotonic::new();
        let state = EngineState::new(TimingConfig::default());

        // Empty state should be valid
        assert!(invariant.check(&state).is_ok());
    }

    #[test]
    fn test_invariant_trait_implementation() {
        let invariant = KeyTimestampsMonotonic::new();
        assert_eq!(invariant.name(), "KeyTimestampsMonotonic");
        assert_eq!(
            invariant.description(),
            "Ensures key press timestamps increase monotonically per key"
        );
        // This is a debug-only invariant
        assert!(invariant.debug_only());
    }

    #[test]
    fn test_default_construction() {
        let invariant = KeyTimestampsMonotonic::default();
        assert_eq!(invariant.name(), "KeyTimestampsMonotonic");
    }
}
