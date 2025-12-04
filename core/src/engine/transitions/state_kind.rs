//! State kind categorization for the unified state machine.
//!
//! This module defines high-level state categories that describe the current
//! mode of operation of the KeyRX engine. StateKind is used by the StateGraph
//! to determine which transitions are valid in the current state.

use serde::{Deserialize, Serialize};

use crate::engine::state::EngineState;

/// High-level categorization of engine state.
///
/// StateKind represents the operational mode of the engine, which determines
/// what transitions are valid. This enables the StateGraph to enforce
/// state-dependent transition rules.
///
/// # State Categories
///
/// The engine can be in multiple overlapping modes simultaneously (e.g.,
/// `LayerActive` + `ModifierHeld`). The StateKind represents the "primary"
/// state for transition validation purposes.
///
/// # Usage
///
/// StateKind is primarily used by:
/// - `StateGraph` to validate transition legality
/// - Transition logging to annotate state context
/// - State visualization in debugging tools
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum StateKind {
    /// Initial state before engine is fully initialized.
    ///
    /// Valid transitions:
    /// - EngineInitialized → Idle
    #[default]
    Uninitialized,

    /// Engine is initialized but no keys are pressed.
    ///
    /// Characteristics:
    /// - No keys pressed
    /// - Only base layer active
    /// - No modifiers active
    /// - No pending decisions
    ///
    /// Valid transitions:
    /// - KeyPressed → Typing, ModifierHeld, or Pending
    /// - RecordingStarted → Recording
    /// - ReplayStarted → Replaying
    /// - DiscoveryStarted → Discovery
    /// - EngineShutdown → ShuttingDown
    Idle,

    /// Normal key input without modifiers or special states.
    ///
    /// Characteristics:
    /// - One or more non-modifier keys pressed
    /// - Base layer active (or transparent layer)
    /// - No modifiers active
    /// - No pending decisions
    ///
    /// Valid transitions:
    /// - KeyReleased → Typing or Idle
    /// - KeyPressed → Typing
    /// - ModifierActivated → ModifierHeld
    /// - LayerPushed → LayerActive
    Typing,

    /// Awaiting resolution of tap-hold or combo decision.
    ///
    /// Characteristics:
    /// - One or more pending decisions in queue
    /// - Keys may be pressed or released
    /// - Waiting for timeout or disambiguating event
    ///
    /// Valid transitions:
    /// - DecisionResolved → Typing, ModifierHeld, LayerActive, or Idle
    /// - DecisionQueued → Pending (additional decision)
    /// - KeyPressed → Pending (may trigger new decision)
    /// - KeyReleased → Pending (may resolve decision)
    Pending,

    /// Non-base layer is active.
    ///
    /// Characteristics:
    /// - Layer stack has non-base layer
    /// - Layer may have been activated by hold key or toggle
    /// - Keys pressed may resolve differently due to layer
    ///
    /// Valid transitions:
    /// - LayerPopped → LayerActive or Idle
    /// - LayerPushed → LayerActive (nested layers)
    /// - KeyPressed → Typing with layer context
    /// - KeyReleased → LayerActive or Idle
    LayerActive,

    /// One or more modifiers are held.
    ///
    /// Characteristics:
    /// - Standard modifiers (Shift, Ctrl, Alt, Meta) or virtual modifiers active
    /// - Modifiers may be physical keys or activated by tap-hold
    /// - Key presses are modified by active modifiers
    ///
    /// Valid transitions:
    /// - ModifierDeactivated → ModifierHeld or Idle
    /// - ModifierActivated → ModifierHeld (additional modifier)
    /// - KeyPressed → Typing with modifiers
    /// - KeyReleased → ModifierHeld or Idle
    ModifierHeld,

    /// Recording session is active.
    ///
    /// Characteristics:
    /// - Recording session capturing events
    /// - All transitions are logged to recording buffer
    /// - Engine operates normally while recording
    ///
    /// Valid transitions:
    /// - RecordingStopped → Idle or previous state
    /// - RecordingPaused → RecordingPaused
    /// - Normal engine transitions (keys, layers, modifiers)
    Recording,

    /// Recording session is paused.
    ///
    /// Characteristics:
    /// - Recording session exists but not capturing
    /// - Events are not recorded during pause
    ///
    /// Valid transitions:
    /// - RecordingResumed → Recording
    /// - RecordingStopped → Idle or previous state
    RecordingPaused,

    /// Replay session is active.
    ///
    /// Characteristics:
    /// - Replaying recorded event sequence
    /// - Transitions driven by replay timeline
    /// - Normal input may be blocked or mixed
    ///
    /// Valid transitions:
    /// - ReplayStopped → Idle or previous state
    /// - ReplayPaused → ReplayPaused
    /// - ReplayCompleted → Idle
    Replaying,

    /// Replay session is paused.
    ///
    /// Characteristics:
    /// - Replay session exists but not playing
    /// - State frozen at pause point
    ///
    /// Valid transitions:
    /// - ReplayResumed → Replaying
    /// - ReplayStopped → Idle or previous state
    ReplayPaused,

    /// Device discovery is in progress.
    ///
    /// Characteristics:
    /// - Scanning for input devices
    /// - Engine may or may not be processing input
    ///
    /// Valid transitions:
    /// - DiscoveryCompleted → Idle or previous state
    /// - DeviceDiscovered → Discovery (continue scanning)
    Discovery,

    /// Fallback mode is active due to error.
    ///
    /// Characteristics:
    /// - Error occurred, operating in safe mode
    /// - Limited functionality to prevent further errors
    /// - Input may be passed through or blocked
    ///
    /// Valid transitions:
    /// - FallbackDeactivated → Idle
    /// - EngineReset → Idle
    Fallback,

    /// Engine is shutting down.
    ///
    /// Characteristics:
    /// - Cleanup in progress
    /// - No new transitions accepted except completion
    ///
    /// Valid transitions:
    /// - EngineShutdown → (engine destroyed)
    ShuttingDown,
}

impl StateKind {
    /// Infer the current StateKind from EngineState.
    ///
    /// This analyzes the current state and determines the primary state category.
    /// When multiple categories could apply, priority is given in this order:
    /// 1. Pending decisions (most constrained)
    /// 2. Modifiers held
    /// 3. Non-base layers active
    /// 4. Keys pressed (typing)
    /// 5. Idle (nothing active)
    ///
    /// Note: This does NOT account for session state (recording/replay) or
    /// discovery state, which are tracked separately. Use `StateGraph::current_kind()`
    /// for complete state kind including session context.
    pub fn from_engine_state(state: &EngineState) -> Self {
        // Check for pending decisions first (highest priority)
        if !state.pending().is_empty() {
            return Self::Pending;
        }

        // Check for active modifiers
        // Check standard modifiers (Shift, Ctrl, Alt, Meta)
        use crate::engine::{Modifier, StandardModifier};
        let has_standard = [
            StandardModifier::Shift,
            StandardModifier::Control,
            StandardModifier::Alt,
            StandardModifier::Meta,
        ]
        .iter()
        .any(|&m| state.modifiers().is_active(Modifier::Standard(m)));

        // Check virtual modifiers (checking first 32 is sufficient for common usage)
        let has_virtual = (0..32).any(|id| state.modifiers().is_active(Modifier::Virtual(id)));

        if has_standard || has_virtual {
            return Self::ModifierHeld;
        }

        // Check for non-base layers (more than just base layer)
        if state.layers().len() > 1 {
            return Self::LayerActive;
        }

        // Check for pressed keys
        if !state.keys().is_empty() {
            return Self::Typing;
        }

        // Default to idle
        Self::Idle
    }

    /// Get a human-readable name for this state kind.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Uninitialized => "Uninitialized",
            Self::Idle => "Idle",
            Self::Typing => "Typing",
            Self::Pending => "Pending",
            Self::LayerActive => "LayerActive",
            Self::ModifierHeld => "ModifierHeld",
            Self::Recording => "Recording",
            Self::RecordingPaused => "RecordingPaused",
            Self::Replaying => "Replaying",
            Self::ReplayPaused => "ReplayPaused",
            Self::Discovery => "Discovery",
            Self::Fallback => "Fallback",
            Self::ShuttingDown => "ShuttingDown",
        }
    }

    /// Check if this state kind allows normal input processing.
    ///
    /// Some states (Fallback, ShuttingDown, Uninitialized) block normal input.
    pub fn allows_input(&self) -> bool {
        !matches!(
            self,
            Self::Uninitialized | Self::ShuttingDown | Self::Fallback
        )
    }

    /// Check if this state kind allows new transitions.
    ///
    /// ShuttingDown state only allows completion, no new transitions.
    pub fn allows_transitions(&self) -> bool {
        !matches!(self, Self::ShuttingDown)
    }

    /// Check if this state kind is a session-related state.
    pub fn is_session_state(&self) -> bool {
        matches!(
            self,
            Self::Recording | Self::RecordingPaused | Self::Replaying | Self::ReplayPaused
        )
    }

    /// Check if this state kind is an error or degraded state.
    pub fn is_error_state(&self) -> bool {
        matches!(self, Self::Fallback)
    }

    /// Check if this state kind represents active input processing.
    pub fn is_active_input(&self) -> bool {
        matches!(
            self,
            Self::Typing | Self::Pending | Self::LayerActive | Self::ModifierHeld
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_kind_name() {
        assert_eq!(StateKind::Idle.name(), "Idle");
        assert_eq!(StateKind::Typing.name(), "Typing");
        assert_eq!(StateKind::Pending.name(), "Pending");
        assert_eq!(StateKind::LayerActive.name(), "LayerActive");
        assert_eq!(StateKind::ModifierHeld.name(), "ModifierHeld");
    }

    #[test]
    fn test_allows_input() {
        assert!(StateKind::Idle.allows_input());
        assert!(StateKind::Typing.allows_input());
        assert!(StateKind::Pending.allows_input());
        assert!(!StateKind::Uninitialized.allows_input());
        assert!(!StateKind::ShuttingDown.allows_input());
        assert!(!StateKind::Fallback.allows_input());
    }

    #[test]
    fn test_allows_transitions() {
        assert!(StateKind::Idle.allows_transitions());
        assert!(StateKind::Typing.allows_transitions());
        assert!(StateKind::Fallback.allows_transitions());
        assert!(!StateKind::ShuttingDown.allows_transitions());
    }

    #[test]
    fn test_is_session_state() {
        assert!(StateKind::Recording.is_session_state());
        assert!(StateKind::RecordingPaused.is_session_state());
        assert!(StateKind::Replaying.is_session_state());
        assert!(StateKind::ReplayPaused.is_session_state());
        assert!(!StateKind::Idle.is_session_state());
        assert!(!StateKind::Typing.is_session_state());
    }

    #[test]
    fn test_is_error_state() {
        assert!(StateKind::Fallback.is_error_state());
        assert!(!StateKind::Idle.is_error_state());
        assert!(!StateKind::Typing.is_error_state());
    }

    #[test]
    fn test_is_active_input() {
        assert!(StateKind::Typing.is_active_input());
        assert!(StateKind::Pending.is_active_input());
        assert!(StateKind::LayerActive.is_active_input());
        assert!(StateKind::ModifierHeld.is_active_input());
        assert!(!StateKind::Idle.is_active_input());
        assert!(!StateKind::Recording.is_active_input());
    }

    #[test]
    fn test_default() {
        assert_eq!(StateKind::default(), StateKind::Uninitialized);
    }

    #[test]
    fn test_serialization() {
        let kind = StateKind::Typing;
        let json = serde_json::to_string(&kind).unwrap();
        let deserialized: StateKind = serde_json::from_str(&json).unwrap();
        assert_eq!(kind, deserialized);
    }
}
