//! Invariant ensuring layer stack is never empty.
//!
//! This invariant checks that the layer stack always contains at least the
//! base layer. An empty layer stack would break keymap resolution.

use crate::engine::state::EngineState;
use crate::engine::transitions::invariant::{Invariant, InvariantViolation};
use std::any::Any;

/// Ensures the layer stack always has at least the base layer.
///
/// # Rationale
///
/// The layer system requires at least one active layer at all times (the base
/// layer). Without any active layers, keymap resolution would fail as there
/// would be no layer to look up key bindings in.
///
/// # Critical Invariant
///
/// This is a critical invariant that must NEVER be violated. An empty layer
/// stack represents a corrupted state that would crash the engine.
///
/// # Example Violation
///
/// ```text
/// 1. Engine starts with base layer (layer 0)
/// 2. Bug: Pop operation removes base layer
/// 3. Layer stack is now empty
/// 4. Next key press has no layer to resolve against
/// 5. Engine crashes or produces undefined behavior
/// ```
pub struct LayerStackNotEmpty;

impl Invariant for LayerStackNotEmpty {
    fn name(&self) -> &'static str {
        "LayerStackNotEmpty"
    }

    fn check(&self, state: &dyn Any) -> Result<(), InvariantViolation> {
        // Downcast to EngineState
        let engine_state = state.downcast_ref::<EngineState>().ok_or_else(|| {
            InvariantViolation::new(
                self.name(),
                "Failed to downcast state to EngineState".to_string(),
            )
        })?;

        // Check that we have at least one active layer
        let active_layers = engine_state.active_layers();

        if active_layers.is_empty() {
            return Err(InvariantViolation::with_context(
                self.name(),
                "Layer stack is empty - must have at least base layer".to_string(),
                "active_layers=0".to_string(),
            ));
        }

        Ok(())
    }

    fn description(&self) -> &str {
        "Ensures the layer stack always has at least the base layer"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::decision::timing::TimingConfig;

    #[test]
    fn test_layer_stack_not_empty_valid() {
        let invariant = LayerStackNotEmpty;
        let state = EngineState::new(TimingConfig::default());

        // Default state should have base layer
        assert!(invariant.check(&state).is_ok());
    }

    #[test]
    fn test_invariant_trait_implementation() {
        let invariant = LayerStackNotEmpty;
        assert_eq!(invariant.name(), "LayerStackNotEmpty");
        assert_eq!(
            invariant.description(),
            "Ensures the layer stack always has at least the base layer"
        );
        assert!(!invariant.debug_only());
    }
}
