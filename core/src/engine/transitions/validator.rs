//! State validator combining multiple invariants.
//!
//! This module provides a comprehensive state validation system that combines
//! multiple invariant checks. It supports both release and debug-only invariants,
//! allowing expensive validation to be compiled out in release builds.

use super::invariant::{Invariant, InvariantViolation};
use super::invariants::{
    KeyTimestampsMonotonic, LayerStackNotEmpty, NoOrphanedModifiers, PendingQueueBounds,
};
use std::sync::Arc;

/// Result of state validation.
///
/// Contains all violations found during validation. An empty vector indicates
/// the state is valid. Multiple violations can be reported at once for better
/// debugging.
pub type ValidationResult = Result<(), Vec<InvariantViolation>>;

/// Comprehensive state validator.
///
/// StateValidator combines multiple invariant checks into a single validation
/// pass. It supports both release-mode invariants (always checked) and
/// debug-only invariants (only checked in debug builds).
///
/// # Design
///
/// - Invariants are stored as trait objects for flexibility
/// - Debug-only invariants are compiled out in release builds
/// - All violations are collected before returning (fail-slow)
/// - Thread-safe: uses Arc for shared invariant storage
///
/// # Example
///
/// ```rust
/// use keyrx_core::engine::transitions::validator::StateValidator;
/// use keyrx_core::engine::state::EngineState;
/// use keyrx_core::engine::decision::timing::TimingConfig;
///
/// let validator = StateValidator::new();
/// let state = EngineState::new(TimingConfig::default());
///
/// match validator.validate(&state) {
///     Ok(()) => println!("State is valid"),
///     Err(violations) => {
///         for violation in violations {
///             eprintln!("Validation error: {}", violation);
///         }
///     }
/// }
/// ```
pub struct StateValidator {
    /// Invariants that are always checked (release and debug).
    release_invariants: Vec<Arc<dyn Invariant>>,
    /// Invariants that are only checked in debug builds.
    #[cfg(debug_assertions)]
    debug_invariants: Vec<Arc<dyn Invariant>>,
}

impl StateValidator {
    /// Create a new validator with default invariants.
    ///
    /// This includes all core invariants:
    /// - NoOrphanedModifiers (release)
    /// - LayerStackNotEmpty (release)
    /// - PendingQueueBounds (release)
    /// - KeyTimestampsMonotonic (debug-only)
    pub fn new() -> Self {
        let mut release_invariants: Vec<Arc<dyn Invariant>> = Vec::new();
        #[cfg(debug_assertions)]
        let mut debug_invariants: Vec<Arc<dyn Invariant>> = Vec::new();

        // Add release-mode invariants (always checked)
        release_invariants.push(Arc::new(NoOrphanedModifiers));
        release_invariants.push(Arc::new(LayerStackNotEmpty));
        release_invariants.push(Arc::new(PendingQueueBounds));

        // Add debug-only invariants
        #[cfg(debug_assertions)]
        {
            debug_invariants.push(Arc::new(KeyTimestampsMonotonic::new()));
        }

        Self {
            release_invariants,
            #[cfg(debug_assertions)]
            debug_invariants,
        }
    }

    /// Create an empty validator with no invariants.
    ///
    /// Useful for testing or when you want to add invariants manually.
    pub fn empty() -> Self {
        Self {
            release_invariants: Vec::new(),
            #[cfg(debug_assertions)]
            debug_invariants: Vec::new(),
        }
    }

    /// Add a release-mode invariant (always checked).
    ///
    /// This invariant will be checked in both debug and release builds.
    pub fn add_release_invariant(&mut self, invariant: Arc<dyn Invariant>) {
        self.release_invariants.push(invariant);
    }

    /// Add a debug-only invariant (only checked in debug builds).
    ///
    /// In release builds, this method is a no-op and the invariant is not stored.
    #[cfg(debug_assertions)]
    pub fn add_debug_invariant(&mut self, invariant: Arc<dyn Invariant>) {
        self.debug_invariants.push(invariant);
    }

    /// No-op version for release builds.
    #[cfg(not(debug_assertions))]
    pub fn add_debug_invariant(&mut self, _invariant: Arc<dyn Invariant>) {
        // No-op in release builds
    }

    /// Validate state against all applicable invariants.
    ///
    /// Runs all release-mode invariants, plus debug-only invariants if compiled
    /// with debug assertions. Collects all violations and returns them together.
    ///
    /// # Arguments
    ///
    /// * `state` - The state to validate (typically EngineState)
    ///
    /// # Returns
    ///
    /// - `Ok(())` if all invariants pass
    /// - `Err(violations)` if any invariant fails, with all violations collected
    ///
    /// # Performance
    ///
    /// In release builds, only release-mode invariants are checked. Debug-only
    /// invariants are compiled out entirely (zero runtime cost).
    pub fn validate(&self, state: &dyn std::any::Any) -> ValidationResult {
        let mut violations = Vec::new();

        // Check release-mode invariants
        for invariant in &self.release_invariants {
            if let Err(violation) = invariant.check(state) {
                violations.push(violation);
            }
        }

        // Check debug-only invariants (compiled out in release)
        #[cfg(debug_assertions)]
        {
            for invariant in &self.debug_invariants {
                if let Err(violation) = invariant.check(state) {
                    violations.push(violation);
                }
            }
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(violations)
        }
    }

    /// Validate state and panic if any invariants fail.
    ///
    /// This is a convenience method for cases where invariant violations
    /// represent unrecoverable bugs that should crash the program.
    ///
    /// # Panics
    ///
    /// Panics if any invariant fails, with a message containing all violations.
    #[cfg(debug_assertions)]
    #[allow(clippy::panic)]
    pub fn validate_or_panic(&self, state: &dyn std::any::Any) {
        if let Err(violations) = self.validate(state) {
            let mut msg = String::from("State validation failed:\n");
            for violation in violations {
                msg.push_str(&format!("  - {}\n", violation));
            }
            panic!("{}", msg);
        }
    }

    /// No-op version for release builds.
    #[cfg(not(debug_assertions))]
    pub fn validate_or_panic(&self, _state: &dyn std::any::Any) {
        // No-op in release builds to avoid panic in production
    }

    /// Get the number of release-mode invariants.
    pub fn release_invariant_count(&self) -> usize {
        self.release_invariants.len()
    }

    /// Get the number of debug-only invariants.
    pub fn debug_invariant_count(&self) -> usize {
        #[cfg(debug_assertions)]
        {
            self.debug_invariants.len()
        }
        #[cfg(not(debug_assertions))]
        {
            0
        }
    }

    /// Get the total number of active invariants.
    ///
    /// In release builds, this equals release_invariant_count().
    /// In debug builds, this equals release_invariant_count() + debug_invariant_count().
    pub fn total_invariant_count(&self) -> usize {
        self.release_invariant_count() + self.debug_invariant_count()
    }
}

impl Default for StateValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::decision::timing::TimingConfig;
    use crate::engine::state::EngineState;

    #[test]
    fn test_validator_new() {
        let validator = StateValidator::new();

        // Should have core release invariants
        assert!(validator.release_invariant_count() >= 3);

        // Should have debug invariants in debug builds
        #[cfg(debug_assertions)]
        assert!(validator.debug_invariant_count() >= 1);

        #[cfg(not(debug_assertions))]
        assert_eq!(validator.debug_invariant_count(), 0);
    }

    #[test]
    fn test_validator_empty() {
        let validator = StateValidator::empty();
        assert_eq!(validator.release_invariant_count(), 0);
        assert_eq!(validator.debug_invariant_count(), 0);
        assert_eq!(validator.total_invariant_count(), 0);
    }

    #[test]
    fn test_validator_add_release_invariant() {
        let mut validator = StateValidator::empty();
        validator.add_release_invariant(Arc::new(LayerStackNotEmpty));

        assert_eq!(validator.release_invariant_count(), 1);
        assert_eq!(validator.debug_invariant_count(), 0);
    }

    #[cfg(debug_assertions)]
    #[test]
    fn test_validator_add_debug_invariant() {
        let mut validator = StateValidator::empty();
        validator.add_debug_invariant(Arc::new(KeyTimestampsMonotonic::new()));

        assert_eq!(validator.release_invariant_count(), 0);
        assert_eq!(validator.debug_invariant_count(), 1);
    }

    #[test]
    fn test_validate_valid_state() {
        let validator = StateValidator::new();
        let state = EngineState::new(TimingConfig::default());

        let result = validator.validate(&state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_collects_all_violations() {
        // Create a validator with multiple invariants that will fail
        let mut validator = StateValidator::empty();

        // Create a test invariant that always fails
        struct AlwaysFails(&'static str);
        impl Invariant for AlwaysFails {
            fn name(&self) -> &'static str {
                self.0
            }
            fn check(&self, _state: &dyn std::any::Any) -> Result<(), InvariantViolation> {
                Err(InvariantViolation::new(
                    self.name(),
                    "Always fails".to_string(),
                ))
            }
        }

        validator.add_release_invariant(Arc::new(AlwaysFails("Invariant1")));
        validator.add_release_invariant(Arc::new(AlwaysFails("Invariant2")));

        let state = EngineState::new(TimingConfig::default());
        let result = validator.validate(&state);

        assert!(result.is_err());
        let violations = result.unwrap_err();
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].invariant_name, "Invariant1");
        assert_eq!(violations[1].invariant_name, "Invariant2");
    }

    #[test]
    #[should_panic(expected = "State validation failed")]
    fn test_validate_or_panic() {
        struct AlwaysFails;
        impl Invariant for AlwaysFails {
            fn name(&self) -> &'static str {
                "AlwaysFails"
            }
            fn check(&self, _state: &dyn std::any::Any) -> Result<(), InvariantViolation> {
                Err(InvariantViolation::new(
                    self.name(),
                    "Test failure".to_string(),
                ))
            }
        }

        let mut validator = StateValidator::empty();
        validator.add_release_invariant(Arc::new(AlwaysFails));

        let state = EngineState::new(TimingConfig::default());
        validator.validate_or_panic(&state);
    }

    #[test]
    fn test_total_invariant_count() {
        let validator = StateValidator::new();
        let total = validator.total_invariant_count();
        let release = validator.release_invariant_count();
        let debug = validator.debug_invariant_count();

        assert_eq!(total, release + debug);
    }

    #[test]
    fn test_validator_default() {
        let validator = StateValidator::default();
        assert!(validator.release_invariant_count() > 0);
    }
}
