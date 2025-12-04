//! State invariant validation trait and violation types.
//!
//! This module provides a pluggable validation system for enforcing state
//! invariants. Invariants are rules that must always hold true for the engine
//! state (e.g., "layer stack cannot be empty", "no orphaned modifiers").
//!
//! The trait-based design allows custom invariants to be added without modifying
//! core validation logic.

use std::fmt;

/// Violation of a state invariant.
///
/// Represents a detected violation of an invariant rule. Contains information
/// about what rule was violated and why, useful for debugging and error reporting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvariantViolation {
    /// Name of the invariant that was violated.
    pub invariant_name: &'static str,
    /// Human-readable description of the violation.
    pub description: String,
    /// Optional context data for debugging (e.g., "layer_stack_size: 0").
    pub context: Option<String>,
}

impl InvariantViolation {
    /// Create a new invariant violation.
    pub fn new(invariant_name: &'static str, description: String) -> Self {
        Self {
            invariant_name,
            description,
            context: None,
        }
    }

    /// Create a new invariant violation with context.
    pub fn with_context(
        invariant_name: &'static str,
        description: String,
        context: String,
    ) -> Self {
        Self {
            invariant_name,
            description,
            context: Some(context),
        }
    }
}

impl fmt::Display for InvariantViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Invariant '{}' violated: {}",
            self.invariant_name, self.description
        )?;
        if let Some(ref ctx) = self.context {
            write!(f, " (context: {})", ctx)?;
        }
        Ok(())
    }
}

impl std::error::Error for InvariantViolation {}

/// Trait for state invariant checks.
///
/// An invariant is a rule that must always hold true for the engine state.
/// Implementations of this trait define specific invariant checks that can
/// be composed together for comprehensive validation.
///
/// # Design
///
/// - Each invariant has a unique name for identification
/// - The `check()` method receives immutable state references
/// - Returns `Ok(())` if the invariant holds, `Err(InvariantViolation)` if violated
/// - Invariants should be pure checks with no side effects
///
/// # Example
///
/// ```rust
/// use keyrx_core::engine::transitions::invariant::{Invariant, InvariantViolation};
///
/// struct NoOrphanedModifiers;
///
/// impl Invariant for NoOrphanedModifiers {
///     fn name(&self) -> &'static str {
///         "NoOrphanedModifiers"
///     }
///
///     fn check(&self, state: &dyn std::any::Any) -> Result<(), InvariantViolation> {
///         // Implementation would check state for orphaned modifiers
///         Ok(())
///     }
/// }
/// ```
pub trait Invariant: Send + Sync {
    /// Get the unique name of this invariant.
    ///
    /// Used for error reporting and logging.
    fn name(&self) -> &'static str;

    /// Check if the invariant holds for the given state.
    ///
    /// The state parameter is passed as `&dyn Any` to allow flexibility in
    /// state representation. Implementations should downcast to the appropriate
    /// state type(s) they need to validate.
    ///
    /// # Arguments
    ///
    /// * `state` - Reference to the state to validate (typically EngineState)
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the invariant holds
    /// - `Err(InvariantViolation)` if the invariant is violated
    ///
    /// # Errors
    ///
    /// Returns `InvariantViolation` with details about the violation.
    fn check(&self, state: &dyn std::any::Any) -> Result<(), InvariantViolation>;

    /// Optional description of what this invariant checks.
    ///
    /// Used for documentation and debugging. Default implementation returns
    /// the invariant name.
    fn description(&self) -> &str {
        self.name()
    }

    /// Whether this invariant should only be checked in debug builds.
    ///
    /// Some invariants may be expensive to check and are only needed during
    /// development. Default is `false` (always check).
    fn debug_only(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invariant_violation_creation() {
        let violation = InvariantViolation::new("TestInvariant", "Test violation".to_string());

        assert_eq!(violation.invariant_name, "TestInvariant");
        assert_eq!(violation.description, "Test violation");
        assert_eq!(violation.context, None);
    }

    #[test]
    fn test_invariant_violation_with_context() {
        let violation = InvariantViolation::with_context(
            "TestInvariant",
            "Test violation".to_string(),
            "value=42".to_string(),
        );

        assert_eq!(violation.invariant_name, "TestInvariant");
        assert_eq!(violation.description, "Test violation");
        assert_eq!(violation.context, Some("value=42".to_string()));
    }

    #[test]
    fn test_invariant_violation_display() {
        let violation =
            InvariantViolation::new("TestInvariant", "Something went wrong".to_string());
        let display = format!("{}", violation);

        assert!(display.contains("TestInvariant"));
        assert!(display.contains("Something went wrong"));
    }

    #[test]
    fn test_invariant_violation_display_with_context() {
        let violation = InvariantViolation::with_context(
            "TestInvariant",
            "Something went wrong".to_string(),
            "count=0".to_string(),
        );
        let display = format!("{}", violation);

        assert!(display.contains("TestInvariant"));
        assert!(display.contains("Something went wrong"));
        assert!(display.contains("count=0"));
    }

    // Example invariant implementation for testing
    struct AlwaysValid;

    impl Invariant for AlwaysValid {
        fn name(&self) -> &'static str {
            "AlwaysValid"
        }

        fn check(&self, _state: &dyn std::any::Any) -> Result<(), InvariantViolation> {
            Ok(())
        }
    }

    struct AlwaysInvalid;

    impl Invariant for AlwaysInvalid {
        fn name(&self) -> &'static str {
            "AlwaysInvalid"
        }

        fn check(&self, _state: &dyn std::any::Any) -> Result<(), InvariantViolation> {
            Err(InvariantViolation::new(
                self.name(),
                "Always fails".to_string(),
            ))
        }

        fn description(&self) -> &str {
            "An invariant that always fails for testing"
        }
    }

    #[test]
    fn test_invariant_trait_always_valid() {
        let invariant = AlwaysValid;
        assert_eq!(invariant.name(), "AlwaysValid");
        assert_eq!(invariant.description(), "AlwaysValid");
        assert!(!invariant.debug_only());

        let result = invariant.check(&42);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invariant_trait_always_invalid() {
        let invariant = AlwaysInvalid;
        assert_eq!(invariant.name(), "AlwaysInvalid");
        assert_eq!(
            invariant.description(),
            "An invariant that always fails for testing"
        );
        assert!(!invariant.debug_only());

        let result = invariant.check(&42);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.invariant_name, "AlwaysInvalid");
        assert_eq!(err.description, "Always fails");
    }

    struct DebugOnlyInvariant;

    impl Invariant for DebugOnlyInvariant {
        fn name(&self) -> &'static str {
            "DebugOnlyInvariant"
        }

        fn check(&self, _state: &dyn std::any::Any) -> Result<(), InvariantViolation> {
            Ok(())
        }

        fn debug_only(&self) -> bool {
            true
        }
    }

    #[test]
    fn test_invariant_debug_only() {
        let invariant = DebugOnlyInvariant;
        assert!(invariant.debug_only());
    }
}
