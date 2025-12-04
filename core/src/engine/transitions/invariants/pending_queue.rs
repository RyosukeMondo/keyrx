//! Invariant ensuring pending queue respects size bounds.
//!
//! This invariant checks that the pending decision queue never exceeds its
//! maximum capacity. Violating this bound could lead to memory issues or
//! dropped events.

use crate::engine::state::EngineState;
use crate::engine::transitions::invariant::{Invariant, InvariantViolation};
use std::any::Any;

/// Maximum allowed pending decisions (from DecisionQueue::MAX_PENDING).
///
/// This constant represents the hard limit on how many pending tap-hold or
/// combo decisions can be queued simultaneously. This limit prevents unbounded
/// memory growth and ensures bounded latency for decision resolution.
const MAX_PENDING_DECISIONS: usize = 32;

/// Ensures the pending decision queue respects size bounds.
///
/// # Rationale
///
/// The pending decision queue has a fixed maximum size to:
/// - Prevent unbounded memory growth
/// - Ensure bounded decision latency
/// - Detect bugs that cause decisions to accumulate
///
/// If the queue exceeds its maximum size, it indicates either:
/// 1. A bug where decisions aren't being resolved
/// 2. Extreme user input that exceeds design limits
/// 3. Configuration error with timeout values
///
/// # Bounds
///
/// - Maximum: [`MAX_PENDING_DECISIONS`] (32)
/// - Normal usage: < 5 pending decisions
/// - Warning threshold: > 16 pending decisions
///
/// # Example Violation
///
/// ```text
/// User rapidly presses 33 tap-hold keys before any resolve
/// → Queue would exceed 32 decision limit
/// → Invariant violation detected
/// → New decisions rejected or oldest evicted
/// ```
pub struct PendingQueueBounds;

impl Invariant for PendingQueueBounds {
    fn name(&self) -> &'static str {
        "PendingQueueBounds"
    }

    fn check(&self, state: &dyn Any) -> Result<(), InvariantViolation> {
        // Downcast to EngineState
        let engine_state = state.downcast_ref::<EngineState>().ok_or_else(|| {
            InvariantViolation::new(
                self.name(),
                "Failed to downcast state to EngineState".to_string(),
            )
        })?;

        // Get pending decision count
        let pending_count = engine_state.pending_count();

        // Check against maximum
        if pending_count > MAX_PENDING_DECISIONS {
            return Err(InvariantViolation::with_context(
                self.name(),
                format!(
                    "Pending queue size ({}) exceeds maximum ({})",
                    pending_count, MAX_PENDING_DECISIONS
                ),
                format!(
                    "pending_count={}, max={}",
                    pending_count, MAX_PENDING_DECISIONS
                ),
            ));
        }

        Ok(())
    }

    fn description(&self) -> &str {
        "Ensures the pending decision queue respects size bounds"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::decision::timing::TimingConfig;

    #[test]
    fn test_pending_queue_bounds_valid_empty() {
        let invariant = PendingQueueBounds;
        let state = EngineState::new(TimingConfig::default());

        // Empty state should be valid
        assert!(invariant.check(&state).is_ok());
    }

    #[test]
    fn test_invariant_trait_implementation() {
        let invariant = PendingQueueBounds;
        assert_eq!(invariant.name(), "PendingQueueBounds");
        assert_eq!(
            invariant.description(),
            "Ensures the pending decision queue respects size bounds"
        );
        assert!(!invariant.debug_only());
    }

    #[test]
    fn test_max_pending_constant() {
        // Verify our constant matches expected value
        assert_eq!(MAX_PENDING_DECISIONS, 32);
    }
}
