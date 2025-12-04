//! State graph for enforcing valid state transitions.
//!
//! The StateGraph defines the rules for which transitions are valid from each
//! state kind. It provides validation before applying transitions and ensures
//! the state machine remains in a valid state.

use super::{StateKind, StateTransition, TransitionCategory};

/// Error type for invalid state transitions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidTransition {
    /// Current state kind when transition was attempted.
    pub from_state: StateKind,
    /// Transition that was attempted.
    pub transition_name: &'static str,
    /// Category of the transition.
    pub category: TransitionCategory,
    /// Human-readable reason why the transition is invalid.
    pub reason: String,
}

impl std::fmt::Display for InvalidTransition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Invalid transition '{}' from state '{}': {}",
            self.transition_name,
            self.from_state.name(),
            self.reason
        )
    }
}

impl std::error::Error for InvalidTransition {}

/// State graph that enforces valid transitions.
///
/// The StateGraph defines the rules for which transitions are allowed from
/// each state kind. It provides two main operations:
/// - `is_valid()`: Check if a transition is valid without applying it
/// - `apply()`: Apply a transition and get the new state kind
///
/// # Transition Rules
///
/// The graph enforces state-dependent rules:
/// - System transitions (init, shutdown) can occur from specific states
/// - Engine transitions (keys, layers) are valid during active input states
/// - Session transitions must follow proper start/stop/pause sequences
/// - Discovery transitions have their own lifecycle
///
/// # Example
///
/// ```rust
/// use keyrx_core::engine::transitions::{StateGraph, StateKind, StateTransition};
///
/// let graph = StateGraph::new();
/// let current = StateKind::Idle;
/// let transition = StateTransition::KeyPressed { key: 0, timestamp: 1000 };
///
/// // Check if transition is valid
/// if graph.is_valid(current, &transition) {
///     // Apply transition to get new state
///     let new_state = graph.apply(current, &transition).unwrap();
///     assert_eq!(new_state, StateKind::Typing);
/// }
/// ```
#[derive(Debug, Clone, Default)]
pub struct StateGraph {
    // Future: Could add custom validators, transition hooks, etc.
}

impl StateGraph {
    /// Create a new StateGraph with default transition rules.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a transition is valid from the current state.
    ///
    /// Returns `true` if the transition is allowed, `false` otherwise.
    /// This does not modify state - use `apply()` to perform the transition.
    pub fn is_valid(&self, from: StateKind, transition: &StateTransition) -> bool {
        self.validate_transition(from, transition).is_ok()
    }

    /// Apply a transition and return the new state kind.
    ///
    /// Returns the new state kind if the transition is valid, or an error
    /// describing why the transition is invalid.
    ///
    /// # Errors
    ///
    /// Returns `InvalidTransition` if the transition is not allowed from the
    /// current state.
    pub fn apply(
        &self,
        from: StateKind,
        transition: &StateTransition,
    ) -> Result<StateKind, InvalidTransition> {
        self.validate_transition(from, transition)?;
        Ok(self.next_state(from, transition))
    }

    /// Validate a transition without applying it.
    ///
    /// Returns `Ok(())` if valid, `Err(InvalidTransition)` if invalid.
    fn validate_transition(
        &self,
        from: StateKind,
        transition: &StateTransition,
    ) -> Result<(), InvalidTransition> {
        // ShuttingDown state only allows EngineShutdown transition
        if from == StateKind::ShuttingDown {
            return match transition {
                StateTransition::EngineShutdown => Ok(()),
                _ => Err(InvalidTransition {
                    from_state: from,
                    transition_name: transition.name(),
                    category: transition.category(),
                    reason: "Engine is shutting down, only EngineShutdown allowed".to_string(),
                }),
            };
        }

        // Check category-specific rules
        match transition.category() {
            TransitionCategory::System => self.validate_system_transition(from, transition),
            TransitionCategory::Engine => self.validate_engine_transition(from, transition),
            TransitionCategory::Session => self.validate_session_transition(from, transition),
            TransitionCategory::Discovery => self.validate_discovery_transition(from, transition),
        }
    }

    /// Validate system transitions (init, shutdown, config, fallback).
    fn validate_system_transition(
        &self,
        from: StateKind,
        transition: &StateTransition,
    ) -> Result<(), InvalidTransition> {
        match transition {
            StateTransition::EngineInitialized => {
                if from == StateKind::Uninitialized {
                    Ok(())
                } else {
                    Err(InvalidTransition {
                        from_state: from,
                        transition_name: transition.name(),
                        category: transition.category(),
                        reason: "EngineInitialized only valid from Uninitialized state".to_string(),
                    })
                }
            }
            StateTransition::EngineShutdown => {
                // Shutdown can occur from any state except Uninitialized
                if from == StateKind::Uninitialized {
                    Err(InvalidTransition {
                        from_state: from,
                        transition_name: transition.name(),
                        category: transition.category(),
                        reason: "Cannot shutdown before initialization".to_string(),
                    })
                } else {
                    Ok(())
                }
            }
            StateTransition::ConfigReloaded | StateTransition::EngineReset => {
                // Config reload and reset allowed from any initialized state
                if from.allows_transitions() && from != StateKind::Uninitialized {
                    Ok(())
                } else {
                    Err(InvalidTransition {
                        from_state: from,
                        transition_name: transition.name(),
                        category: transition.category(),
                        reason: "System operations require initialized state".to_string(),
                    })
                }
            }
            StateTransition::FallbackActivated { .. } => {
                // Fallback can be activated from any state except already in fallback
                if from != StateKind::Fallback {
                    Ok(())
                } else {
                    Err(InvalidTransition {
                        from_state: from,
                        transition_name: transition.name(),
                        category: transition.category(),
                        reason: "Already in fallback mode".to_string(),
                    })
                }
            }
            StateTransition::FallbackDeactivated => {
                if from == StateKind::Fallback {
                    Ok(())
                } else {
                    Err(InvalidTransition {
                        from_state: from,
                        transition_name: transition.name(),
                        category: transition.category(),
                        reason: "Not in fallback mode".to_string(),
                    })
                }
            }
            _ => Ok(()), // Other system transitions handled elsewhere
        }
    }

    /// Validate engine transitions (keys, layers, modifiers, decisions).
    fn validate_engine_transition(
        &self,
        from: StateKind,
        transition: &StateTransition,
    ) -> Result<(), InvalidTransition> {
        // Engine transitions require input to be allowed
        if !from.allows_input() {
            return Err(InvalidTransition {
                from_state: from,
                transition_name: transition.name(),
                category: transition.category(),
                reason: format!("Input not allowed in {} state", from.name()),
            });
        }

        // All engine transitions are valid from states that allow input
        // State-specific validation (e.g., can't pop empty layer stack) is
        // handled by the engine itself, not the graph
        Ok(())
    }

    /// Validate session transitions (recording, replay).
    fn validate_session_transition(
        &self,
        from: StateKind,
        transition: &StateTransition,
    ) -> Result<(), InvalidTransition> {
        match transition {
            StateTransition::RecordingStarted { .. } => {
                // Can start recording from any non-session state
                if from.is_session_state() {
                    Err(InvalidTransition {
                        from_state: from,
                        transition_name: transition.name(),
                        category: transition.category(),
                        reason: "Cannot start recording during active session".to_string(),
                    })
                } else if !from.allows_transitions() {
                    Err(InvalidTransition {
                        from_state: from,
                        transition_name: transition.name(),
                        category: transition.category(),
                        reason: "Cannot start recording in current state".to_string(),
                    })
                } else {
                    Ok(())
                }
            }
            StateTransition::RecordingStopped => {
                if matches!(from, StateKind::Recording | StateKind::RecordingPaused) {
                    Ok(())
                } else {
                    Err(InvalidTransition {
                        from_state: from,
                        transition_name: transition.name(),
                        category: transition.category(),
                        reason: "No recording session active".to_string(),
                    })
                }
            }
            StateTransition::RecordingPaused => {
                if from == StateKind::Recording {
                    Ok(())
                } else {
                    Err(InvalidTransition {
                        from_state: from,
                        transition_name: transition.name(),
                        category: transition.category(),
                        reason: "Can only pause active recording".to_string(),
                    })
                }
            }
            StateTransition::RecordingResumed => {
                if from == StateKind::RecordingPaused {
                    Ok(())
                } else {
                    Err(InvalidTransition {
                        from_state: from,
                        transition_name: transition.name(),
                        category: transition.category(),
                        reason: "Can only resume paused recording".to_string(),
                    })
                }
            }
            StateTransition::ReplayStarted { .. } => {
                // Can start replay from any non-session state
                if from.is_session_state() {
                    Err(InvalidTransition {
                        from_state: from,
                        transition_name: transition.name(),
                        category: transition.category(),
                        reason: "Cannot start replay during active session".to_string(),
                    })
                } else if !from.allows_transitions() {
                    Err(InvalidTransition {
                        from_state: from,
                        transition_name: transition.name(),
                        category: transition.category(),
                        reason: "Cannot start replay in current state".to_string(),
                    })
                } else {
                    Ok(())
                }
            }
            StateTransition::ReplayStopped => {
                if matches!(from, StateKind::Replaying | StateKind::ReplayPaused) {
                    Ok(())
                } else {
                    Err(InvalidTransition {
                        from_state: from,
                        transition_name: transition.name(),
                        category: transition.category(),
                        reason: "No replay session active".to_string(),
                    })
                }
            }
            StateTransition::ReplayPaused => {
                if from == StateKind::Replaying {
                    Ok(())
                } else {
                    Err(InvalidTransition {
                        from_state: from,
                        transition_name: transition.name(),
                        category: transition.category(),
                        reason: "Can only pause active replay".to_string(),
                    })
                }
            }
            StateTransition::ReplayResumed => {
                if from == StateKind::ReplayPaused {
                    Ok(())
                } else {
                    Err(InvalidTransition {
                        from_state: from,
                        transition_name: transition.name(),
                        category: transition.category(),
                        reason: "Can only resume paused replay".to_string(),
                    })
                }
            }
            StateTransition::ReplayCompleted => {
                if from == StateKind::Replaying {
                    Ok(())
                } else {
                    Err(InvalidTransition {
                        from_state: from,
                        transition_name: transition.name(),
                        category: transition.category(),
                        reason: "Can only complete active replay".to_string(),
                    })
                }
            }
            _ => Ok(()),
        }
    }

    /// Validate discovery transitions.
    fn validate_discovery_transition(
        &self,
        from: StateKind,
        transition: &StateTransition,
    ) -> Result<(), InvalidTransition> {
        match transition {
            StateTransition::DiscoveryStarted => {
                if from == StateKind::Discovery {
                    Err(InvalidTransition {
                        from_state: from,
                        transition_name: transition.name(),
                        category: transition.category(),
                        reason: "Discovery already in progress".to_string(),
                    })
                } else if !from.allows_transitions() {
                    Err(InvalidTransition {
                        from_state: from,
                        transition_name: transition.name(),
                        category: transition.category(),
                        reason: "Cannot start discovery in current state".to_string(),
                    })
                } else {
                    Ok(())
                }
            }
            StateTransition::DiscoveryCompleted => {
                if from == StateKind::Discovery {
                    Ok(())
                } else {
                    Err(InvalidTransition {
                        from_state: from,
                        transition_name: transition.name(),
                        category: transition.category(),
                        reason: "No discovery session active".to_string(),
                    })
                }
            }
            StateTransition::DeviceDiscovered { .. } | StateTransition::DeviceLost { .. } => {
                // Device events can occur anytime (discovery session or not)
                if from.allows_transitions() {
                    Ok(())
                } else {
                    Err(InvalidTransition {
                        from_state: from,
                        transition_name: transition.name(),
                        category: transition.category(),
                        reason: "Device events not allowed in current state".to_string(),
                    })
                }
            }
            _ => Ok(()),
        }
    }

    /// Determine the next state kind after applying a transition.
    ///
    /// This assumes the transition is valid (validated by `validate_transition`).
    fn next_state(&self, from: StateKind, transition: &StateTransition) -> StateKind {
        match transition {
            // System transitions
            StateTransition::EngineInitialized => StateKind::Idle,
            StateTransition::EngineShutdown => StateKind::ShuttingDown,
            StateTransition::EngineReset => StateKind::Idle,
            StateTransition::FallbackActivated { .. } => StateKind::Fallback,
            StateTransition::FallbackDeactivated => StateKind::Idle,
            StateTransition::ConfigReloaded => from, // Stays in same state

            // Session transitions
            StateTransition::RecordingStarted { .. } => StateKind::Recording,
            StateTransition::RecordingStopped => StateKind::Idle,
            StateTransition::RecordingPaused => StateKind::RecordingPaused,
            StateTransition::RecordingResumed => StateKind::Recording,
            StateTransition::ReplayStarted { .. } => StateKind::Replaying,
            StateTransition::ReplayStopped => StateKind::Idle,
            StateTransition::ReplayPaused => StateKind::ReplayPaused,
            StateTransition::ReplayResumed => StateKind::Replaying,
            StateTransition::ReplayCompleted => StateKind::Idle,

            // Discovery transitions
            StateTransition::DiscoveryStarted => StateKind::Discovery,
            StateTransition::DiscoveryCompleted => StateKind::Idle,
            StateTransition::DeviceDiscovered { .. } | StateTransition::DeviceLost { .. } => from,

            // Engine transitions - state inference depends on actual engine state
            // These transitions don't directly map to state kinds because the
            // actual state depends on the combination of active keys, layers, etc.
            // The caller should use StateKind::from_engine_state() after applying.
            StateTransition::KeyPressed { .. } => {
                // Could transition to Typing, ModifierHeld, or stay in current
                // Depends on actual key being pressed
                if from == StateKind::Idle {
                    StateKind::Typing
                } else {
                    from
                }
            }
            StateTransition::KeyReleased { .. } => {
                // Could transition back to Idle or stay in active state
                // Depends on whether other keys are still pressed
                from
            }
            StateTransition::LayerPushed { .. } => {
                if from == StateKind::Idle {
                    StateKind::LayerActive
                } else {
                    from
                }
            }
            StateTransition::LayerPopped { .. } => {
                // Could transition to Idle if layer stack becomes empty
                from
            }
            StateTransition::LayerActivated { .. } => StateKind::LayerActive,
            StateTransition::ModifierActivated { .. } => {
                if from == StateKind::Idle || from == StateKind::Typing {
                    StateKind::ModifierHeld
                } else {
                    from
                }
            }
            StateTransition::ModifierDeactivated { .. } => {
                // Could transition back to Idle or Typing
                from
            }
            StateTransition::DecisionQueued { .. } => StateKind::Pending,
            StateTransition::DecisionResolved { .. } => {
                // Resolution could lead to various states depending on the resolution
                // In practice, caller should use from_engine_state()
                if from == StateKind::Pending {
                    StateKind::Idle
                } else {
                    from
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialization() {
        let graph = StateGraph::new();
        let transition = StateTransition::EngineInitialized;

        // Can initialize from Uninitialized
        assert!(graph.is_valid(StateKind::Uninitialized, &transition));
        assert_eq!(
            graph.apply(StateKind::Uninitialized, &transition).unwrap(),
            StateKind::Idle
        );

        // Cannot initialize from other states
        assert!(!graph.is_valid(StateKind::Idle, &transition));
    }

    #[test]
    fn test_shutdown() {
        let graph = StateGraph::new();
        let transition = StateTransition::EngineShutdown;

        // Can shutdown from any initialized state
        assert!(graph.is_valid(StateKind::Idle, &transition));
        assert!(graph.is_valid(StateKind::Typing, &transition));
        assert!(graph.is_valid(StateKind::Recording, &transition));

        // Cannot shutdown from uninitialized
        assert!(!graph.is_valid(StateKind::Uninitialized, &transition));

        // Shutdown transitions to ShuttingDown
        assert_eq!(
            graph.apply(StateKind::Idle, &transition).unwrap(),
            StateKind::ShuttingDown
        );
    }

    #[test]
    fn test_shutting_down_only_allows_shutdown() {
        use crate::drivers::keycodes::KeyCode;

        let graph = StateGraph::new();

        // Only EngineShutdown allowed
        assert!(graph.is_valid(StateKind::ShuttingDown, &StateTransition::EngineShutdown));

        // Everything else rejected
        assert!(!graph.is_valid(
            StateKind::ShuttingDown,
            &StateTransition::KeyPressed {
                key: KeyCode::A,
                timestamp: 1000
            }
        ));
        assert!(!graph.is_valid(
            StateKind::ShuttingDown,
            &StateTransition::RecordingStarted {
                session_id: "test".to_string()
            }
        ));
    }

    #[test]
    fn test_fallback_transitions() {
        let graph = StateGraph::new();

        // Can activate fallback from any non-fallback state
        let activate = StateTransition::FallbackActivated {
            reason: "test".to_string(),
        };
        assert!(graph.is_valid(StateKind::Idle, &activate));
        assert!(graph.is_valid(StateKind::Typing, &activate));
        assert!(!graph.is_valid(StateKind::Fallback, &activate));

        // Can deactivate only from fallback
        let deactivate = StateTransition::FallbackDeactivated;
        assert!(graph.is_valid(StateKind::Fallback, &deactivate));
        assert!(!graph.is_valid(StateKind::Idle, &deactivate));
    }

    #[test]
    fn test_engine_transitions_require_input_allowed() {
        use crate::drivers::keycodes::KeyCode;

        let graph = StateGraph::new();
        let key_press = StateTransition::KeyPressed {
            key: KeyCode::A,
            timestamp: 1000,
        };

        // Allowed from states that allow input
        assert!(graph.is_valid(StateKind::Idle, &key_press));
        assert!(graph.is_valid(StateKind::Typing, &key_press));

        // Not allowed from states that block input
        assert!(!graph.is_valid(StateKind::Uninitialized, &key_press));
        assert!(!graph.is_valid(StateKind::ShuttingDown, &key_press));
        assert!(!graph.is_valid(StateKind::Fallback, &key_press));
    }

    #[test]
    fn test_recording_session_lifecycle() {
        let graph = StateGraph::new();

        // Start recording
        let start = StateTransition::RecordingStarted {
            session_id: "test".to_string(),
        };
        assert!(graph.is_valid(StateKind::Idle, &start));
        assert_eq!(
            graph.apply(StateKind::Idle, &start).unwrap(),
            StateKind::Recording
        );

        // Cannot start recording during session
        assert!(!graph.is_valid(StateKind::Recording, &start));
        assert!(!graph.is_valid(StateKind::Replaying, &start));

        // Pause recording
        let pause = StateTransition::RecordingPaused;
        assert!(graph.is_valid(StateKind::Recording, &pause));
        assert!(!graph.is_valid(StateKind::Idle, &pause));
        assert_eq!(
            graph.apply(StateKind::Recording, &pause).unwrap(),
            StateKind::RecordingPaused
        );

        // Resume recording
        let resume = StateTransition::RecordingResumed;
        assert!(graph.is_valid(StateKind::RecordingPaused, &resume));
        assert!(!graph.is_valid(StateKind::Recording, &resume));
        assert_eq!(
            graph.apply(StateKind::RecordingPaused, &resume).unwrap(),
            StateKind::Recording
        );

        // Stop recording
        let stop = StateTransition::RecordingStopped;
        assert!(graph.is_valid(StateKind::Recording, &stop));
        assert!(graph.is_valid(StateKind::RecordingPaused, &stop));
        assert!(!graph.is_valid(StateKind::Idle, &stop));
        assert_eq!(
            graph.apply(StateKind::Recording, &stop).unwrap(),
            StateKind::Idle
        );
    }

    #[test]
    fn test_replay_session_lifecycle() {
        let graph = StateGraph::new();

        // Start replay
        let start = StateTransition::ReplayStarted {
            session_id: "test".to_string(),
        };
        assert!(graph.is_valid(StateKind::Idle, &start));
        assert_eq!(
            graph.apply(StateKind::Idle, &start).unwrap(),
            StateKind::Replaying
        );

        // Cannot start replay during session
        assert!(!graph.is_valid(StateKind::Recording, &start));
        assert!(!graph.is_valid(StateKind::Replaying, &start));

        // Pause replay
        let pause = StateTransition::ReplayPaused;
        assert!(graph.is_valid(StateKind::Replaying, &pause));
        assert!(!graph.is_valid(StateKind::Idle, &pause));

        // Resume replay
        let resume = StateTransition::ReplayResumed;
        assert!(graph.is_valid(StateKind::ReplayPaused, &resume));
        assert!(!graph.is_valid(StateKind::Replaying, &resume));

        // Complete replay
        let complete = StateTransition::ReplayCompleted;
        assert!(graph.is_valid(StateKind::Replaying, &complete));
        assert!(!graph.is_valid(StateKind::Idle, &complete));
        assert_eq!(
            graph.apply(StateKind::Replaying, &complete).unwrap(),
            StateKind::Idle
        );

        // Stop replay
        let stop = StateTransition::ReplayStopped;
        assert!(graph.is_valid(StateKind::Replaying, &stop));
        assert!(graph.is_valid(StateKind::ReplayPaused, &stop));
        assert!(!graph.is_valid(StateKind::Idle, &stop));
    }

    #[test]
    fn test_discovery_lifecycle() {
        let graph = StateGraph::new();

        // Start discovery
        let start = StateTransition::DiscoveryStarted;
        assert!(graph.is_valid(StateKind::Idle, &start));
        assert_eq!(
            graph.apply(StateKind::Idle, &start).unwrap(),
            StateKind::Discovery
        );

        // Cannot start discovery when already in progress
        assert!(!graph.is_valid(StateKind::Discovery, &start));

        // Complete discovery
        let complete = StateTransition::DiscoveryCompleted;
        assert!(graph.is_valid(StateKind::Discovery, &complete));
        assert!(!graph.is_valid(StateKind::Idle, &complete));
        assert_eq!(
            graph.apply(StateKind::Discovery, &complete).unwrap(),
            StateKind::Idle
        );
    }

    #[test]
    fn test_device_events() {
        use crate::drivers::common::DeviceInfo;
        use std::path::PathBuf;

        let graph = StateGraph::new();
        let discovered = StateTransition::DeviceDiscovered {
            device: DeviceInfo {
                path: PathBuf::from("/dev/input/test"),
                name: "Test".to_string(),
                vendor_id: 0,
                product_id: 0,
                is_keyboard: true,
            },
        };

        // Device events allowed from any state that allows transitions
        assert!(graph.is_valid(StateKind::Idle, &discovered));
        assert!(graph.is_valid(StateKind::Discovery, &discovered));
        assert!(graph.is_valid(StateKind::Typing, &discovered));

        // Not allowed from states that block transitions
        assert!(!graph.is_valid(StateKind::ShuttingDown, &discovered));
    }

    #[test]
    fn test_key_press_state_transitions() {
        use crate::drivers::keycodes::KeyCode;

        let graph = StateGraph::new();
        let key_press = StateTransition::KeyPressed {
            key: KeyCode::A,
            timestamp: 1000,
        };

        // From Idle → Typing
        assert_eq!(
            graph.apply(StateKind::Idle, &key_press).unwrap(),
            StateKind::Typing
        );

        // From Typing → Typing (stays)
        assert_eq!(
            graph.apply(StateKind::Typing, &key_press).unwrap(),
            StateKind::Typing
        );
    }

    #[test]
    fn test_modifier_transitions() {
        use crate::engine::Modifier;

        let graph = StateGraph::new();
        let activate = StateTransition::ModifierActivated {
            modifier: Modifier::Virtual(0),
        };

        // From Idle or Typing → ModifierHeld
        assert_eq!(
            graph.apply(StateKind::Idle, &activate).unwrap(),
            StateKind::ModifierHeld
        );
        assert_eq!(
            graph.apply(StateKind::Typing, &activate).unwrap(),
            StateKind::ModifierHeld
        );

        // From other states → stays in same state
        assert_eq!(
            graph.apply(StateKind::LayerActive, &activate).unwrap(),
            StateKind::LayerActive
        );
    }

    #[test]
    fn test_layer_transitions() {
        let graph = StateGraph::new();
        let push = StateTransition::LayerPushed { layer: 1 };

        // From Idle → LayerActive
        assert_eq!(
            graph.apply(StateKind::Idle, &push).unwrap(),
            StateKind::LayerActive
        );

        // From other states → stays
        assert_eq!(
            graph.apply(StateKind::Typing, &push).unwrap(),
            StateKind::Typing
        );
    }

    #[test]
    fn test_invalid_transition_error() {
        let graph = StateGraph::new();
        let transition = StateTransition::RecordingPaused;

        let result = graph.apply(StateKind::Idle, &transition);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.from_state, StateKind::Idle);
        assert_eq!(err.transition_name, "RecordingPaused");
        assert!(err.reason.contains("active recording"));
    }

    #[test]
    fn test_error_display() {
        let err = InvalidTransition {
            from_state: StateKind::Idle,
            transition_name: "RecordingPaused",
            category: TransitionCategory::Session,
            reason: "Can only pause active recording".to_string(),
        };

        let display = format!("{}", err);
        assert!(display.contains("RecordingPaused"));
        assert!(display.contains("Idle"));
        assert!(display.contains("Can only pause active recording"));
    }
}
