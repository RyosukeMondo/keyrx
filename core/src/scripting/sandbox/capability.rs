//! Capability tiers for script functions.
//!
//! This module defines security tiers for script functions, enabling fine-grained
//! control over what operations scripts can perform based on trust level.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Capability tier for script functions.
///
/// Functions are assigned a capability tier at registration time. The tier determines
/// in which execution modes the function can be called.
///
/// # Tier Hierarchy
///
/// Tiers are ordered from least to most privileged:
/// - `Safe`: No side effects, bounded execution
/// - `Standard`: May affect engine state
/// - `Advanced`: System interaction, requires trust
/// - `Internal`: Not exposed to user scripts
///
/// # Examples
///
/// ```
/// use keyrx_core::scripting::sandbox::capability::{ScriptCapability, ScriptMode};
///
/// let cap = ScriptCapability::Safe;
/// assert!(cap.is_allowed_in(ScriptMode::Safe));
/// assert!(cap.is_allowed_in(ScriptMode::Standard));
/// assert!(cap.is_allowed_in(ScriptMode::Full));
///
/// let cap = ScriptCapability::Advanced;
/// assert!(!cap.is_allowed_in(ScriptMode::Safe));
/// assert!(!cap.is_allowed_in(ScriptMode::Standard));
/// assert!(cap.is_allowed_in(ScriptMode::Full));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ScriptCapability {
    /// Safe for any script - no side effects, bounded execution.
    ///
    /// Functions in this tier:
    /// - Cannot modify system state
    /// - Cannot access filesystem or network
    /// - Have deterministic, bounded execution time
    /// - Cannot call other unsafe functions
    ///
    /// Examples: arithmetic, string operations, key code constants
    Safe = 0,

    /// Standard operations - may affect engine state.
    ///
    /// Functions in this tier:
    /// - May modify script engine state
    /// - May send key events
    /// - May access keyboard state
    /// - Cannot access system resources
    ///
    /// Examples: send_key, get_layer, conditional logic
    Standard = 1,

    /// Advanced operations - system interaction, requires trust.
    ///
    /// Functions in this tier:
    /// - May interact with system (clipboard, notifications)
    /// - May have side effects outside the keyboard
    /// - Require explicit user trust
    ///
    /// Examples: clipboard operations, system commands
    Advanced = 2,

    /// Internal only - not exposed to user scripts.
    ///
    /// Functions in this tier:
    /// - Used only by engine internals
    /// - Can bypass safety checks
    /// - Never callable from user scripts
    ///
    /// Examples: debug hooks, engine control functions
    Internal = 3,
}

impl ScriptCapability {
    /// Check if this capability is allowed in the given mode.
    ///
    /// # Examples
    ///
    /// ```
    /// use keyrx_core::scripting::sandbox::capability::{ScriptCapability, ScriptMode};
    ///
    /// assert!(ScriptCapability::Safe.is_allowed_in(ScriptMode::Safe));
    /// assert!(!ScriptCapability::Advanced.is_allowed_in(ScriptMode::Safe));
    /// assert!(!ScriptCapability::Internal.is_allowed_in(ScriptMode::Full));
    /// ```
    #[inline]
    pub fn is_allowed_in(&self, mode: ScriptMode) -> bool {
        match mode {
            ScriptMode::Safe => matches!(self, ScriptCapability::Safe),
            ScriptMode::Standard => {
                matches!(self, ScriptCapability::Safe | ScriptCapability::Standard)
            }
            ScriptMode::Full => !matches!(self, ScriptCapability::Internal),
        }
    }

    /// Get a human-readable description of this capability tier.
    ///
    /// # Examples
    ///
    /// ```
    /// use keyrx_core::scripting::sandbox::capability::ScriptCapability;
    ///
    /// assert_eq!(
    ///     ScriptCapability::Safe.description(),
    ///     "Safe functions - no side effects, bounded execution"
    /// );
    /// ```
    pub fn description(&self) -> &'static str {
        match self {
            ScriptCapability::Safe => "Safe functions - no side effects, bounded execution",
            ScriptCapability::Standard => "Standard functions - may affect engine state",
            ScriptCapability::Advanced => "Advanced functions - system interaction, requires trust",
            ScriptCapability::Internal => "Internal functions - not exposed to user scripts",
        }
    }

    /// Get a short label for this capability tier.
    pub fn label(&self) -> &'static str {
        match self {
            ScriptCapability::Safe => "safe",
            ScriptCapability::Standard => "standard",
            ScriptCapability::Advanced => "advanced",
            ScriptCapability::Internal => "internal",
        }
    }
}

/// Script execution mode determining which functions are available.
///
/// The mode controls which capability tiers are allowed during script execution.
///
/// # Modes
///
/// - `Safe`: Only Safe tier functions (most restrictive)
/// - `Standard`: Safe + Standard tier functions (default)
/// - `Full`: Safe + Standard + Advanced tier functions (most permissive)
///
/// Internal tier functions are never available to user scripts.
///
/// # Examples
///
/// ```
/// use keyrx_core::scripting::sandbox::capability::{ScriptCapability, ScriptMode};
///
/// let mode = ScriptMode::Standard;
/// assert!(ScriptCapability::Safe.is_allowed_in(mode));
/// assert!(ScriptCapability::Standard.is_allowed_in(mode));
/// assert!(!ScriptCapability::Advanced.is_allowed_in(mode));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default, JsonSchema)]
pub enum ScriptMode {
    /// Only safe functions are allowed.
    ///
    /// This is the most restrictive mode, suitable for untrusted scripts.
    Safe,

    /// Safe and standard functions are allowed.
    ///
    /// This is the default mode, balancing safety and functionality.
    #[default]
    Standard,

    /// Safe, standard, and advanced functions are allowed.
    ///
    /// This mode allows full keyboard functionality but requires user trust.
    Full,
}

impl ScriptMode {
    /// Get a human-readable description of this mode.
    pub fn description(&self) -> &'static str {
        match self {
            ScriptMode::Safe => "Safe mode - only functions with no side effects",
            ScriptMode::Standard => "Standard mode - keyboard operations allowed",
            ScriptMode::Full => "Full mode - all functionality including system interaction",
        }
    }

    /// Get a short label for this mode.
    pub fn label(&self) -> &'static str {
        match self {
            ScriptMode::Safe => "safe",
            ScriptMode::Standard => "standard",
            ScriptMode::Full => "full",
        }
    }

    /// Get all capability tiers allowed in this mode.
    pub fn allowed_capabilities(&self) -> &'static [ScriptCapability] {
        match self {
            ScriptMode::Safe => &[ScriptCapability::Safe],
            ScriptMode::Standard => &[ScriptCapability::Safe, ScriptCapability::Standard],
            ScriptMode::Full => &[
                ScriptCapability::Safe,
                ScriptCapability::Standard,
                ScriptCapability::Advanced,
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_ordering() {
        assert!(ScriptCapability::Safe < ScriptCapability::Standard);
        assert!(ScriptCapability::Standard < ScriptCapability::Advanced);
        assert!(ScriptCapability::Advanced < ScriptCapability::Internal);
    }

    #[test]
    fn test_safe_mode_restrictions() {
        let mode = ScriptMode::Safe;
        assert!(ScriptCapability::Safe.is_allowed_in(mode));
        assert!(!ScriptCapability::Standard.is_allowed_in(mode));
        assert!(!ScriptCapability::Advanced.is_allowed_in(mode));
        assert!(!ScriptCapability::Internal.is_allowed_in(mode));
    }

    #[test]
    fn test_standard_mode_restrictions() {
        let mode = ScriptMode::Standard;
        assert!(ScriptCapability::Safe.is_allowed_in(mode));
        assert!(ScriptCapability::Standard.is_allowed_in(mode));
        assert!(!ScriptCapability::Advanced.is_allowed_in(mode));
        assert!(!ScriptCapability::Internal.is_allowed_in(mode));
    }

    #[test]
    fn test_full_mode_restrictions() {
        let mode = ScriptMode::Full;
        assert!(ScriptCapability::Safe.is_allowed_in(mode));
        assert!(ScriptCapability::Standard.is_allowed_in(mode));
        assert!(ScriptCapability::Advanced.is_allowed_in(mode));
        assert!(!ScriptCapability::Internal.is_allowed_in(mode));
    }

    #[test]
    fn test_internal_never_allowed() {
        assert!(!ScriptCapability::Internal.is_allowed_in(ScriptMode::Safe));
        assert!(!ScriptCapability::Internal.is_allowed_in(ScriptMode::Standard));
        assert!(!ScriptCapability::Internal.is_allowed_in(ScriptMode::Full));
    }

    #[test]
    fn test_default_mode() {
        assert_eq!(ScriptMode::default(), ScriptMode::Standard);
    }

    #[test]
    fn test_capability_descriptions() {
        assert!(!ScriptCapability::Safe.description().is_empty());
        assert!(!ScriptCapability::Standard.description().is_empty());
        assert!(!ScriptCapability::Advanced.description().is_empty());
        assert!(!ScriptCapability::Internal.description().is_empty());
    }

    #[test]
    fn test_mode_descriptions() {
        assert!(!ScriptMode::Safe.description().is_empty());
        assert!(!ScriptMode::Standard.description().is_empty());
        assert!(!ScriptMode::Full.description().is_empty());
    }

    #[test]
    fn test_allowed_capabilities() {
        assert_eq!(
            ScriptMode::Safe.allowed_capabilities(),
            &[ScriptCapability::Safe]
        );
        assert_eq!(
            ScriptMode::Standard.allowed_capabilities(),
            &[ScriptCapability::Safe, ScriptCapability::Standard]
        );
        assert_eq!(
            ScriptMode::Full.allowed_capabilities(),
            &[
                ScriptCapability::Safe,
                ScriptCapability::Standard,
                ScriptCapability::Advanced
            ]
        );
    }
}
