//! Invariant ensuring no orphaned modifiers.
//!
//! This invariant checks that all active modifiers have their triggering key
//! currently pressed. A modifier is "orphaned" if it's active but the key that
//! activated it is no longer pressed.

use crate::engine::state::EngineState;
use crate::engine::transitions::invariant::{Invariant, InvariantViolation};
use std::any::Any;

/// Ensures no modifiers are active without their triggering key pressed.
///
/// # Rationale
///
/// Modifiers should only be active while their physical key is held down.
/// Orphaned modifiers indicate a bug in state management where a modifier
/// was activated but not properly cleaned up on key release.
///
/// # Example Violation
///
/// - User presses Shift key (KeyCode::ShiftLeft)
/// - Shift modifier becomes active
/// - User releases Shift key
/// - **BUG:** Shift modifier remains active (orphaned)
///
/// This invariant would detect this condition.
pub struct NoOrphanedModifiers;

impl Invariant for NoOrphanedModifiers {
    fn name(&self) -> &'static str {
        "NoOrphanedModifiers"
    }

    fn check(&self, state: &dyn Any) -> Result<(), InvariantViolation> {
        // Downcast to EngineState
        let engine_state = state.downcast_ref::<EngineState>().ok_or_else(|| {
            InvariantViolation::new(
                self.name(),
                "Failed to downcast state to EngineState".to_string(),
            )
        })?;

        // Note: This is a simplified check. In a full implementation, we would
        // need to track which keys trigger which modifiers. For now, we verify
        // that the state is internally consistent by checking if any standard
        // modifiers are active without corresponding keys pressed.
        //
        // This would require access to the keymap to know which physical keys
        // correspond to which modifiers. For the current implementation, we'll
        // do a basic consistency check.

        use crate::engine::state::{Modifier, StandardModifier};

        // Get modifier state
        let modifiers = engine_state.modifiers();

        // Check if any standard modifiers are active
        let has_shift = modifiers.is_active(Modifier::Standard(StandardModifier::Shift));
        let has_ctrl = modifiers.is_active(Modifier::Standard(StandardModifier::Control));
        let has_alt = modifiers.is_active(Modifier::Standard(StandardModifier::Alt));
        let has_meta = modifiers.is_active(Modifier::Standard(StandardModifier::Meta));

        // If modifiers are active, we should have at least some keys pressed
        // This is a weak check, but without keymap access, we can't do better
        let pressed_count = engine_state.pressed_keys().count();
        if (has_shift || has_ctrl || has_alt || has_meta) && pressed_count == 0 {
            return Err(InvariantViolation::with_context(
                self.name(),
                "Modifiers are active but no keys are pressed".to_string(),
                format!(
                    "shift={}, ctrl={}, alt={}, meta={}, pressed_keys=0",
                    has_shift, has_ctrl, has_alt, has_meta
                ),
            ));
        }

        Ok(())
    }

    fn description(&self) -> &str {
        "Ensures no modifiers are active without their triggering key pressed"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::decision::timing::TimingConfig;

    #[test]
    fn test_no_orphaned_modifiers_valid_empty() {
        let invariant = NoOrphanedModifiers;
        let state = EngineState::new(TimingConfig::default());

        // Empty state should always be valid
        assert!(invariant.check(&state).is_ok());
    }

    #[test]
    fn test_invariant_trait_implementation() {
        let invariant = NoOrphanedModifiers;
        assert_eq!(invariant.name(), "NoOrphanedModifiers");
        assert!(!invariant.debug_only());
    }
}
