//! State transition definitions for the unified state machine.
//!
//! This module defines all valid state transitions in the KeyRX engine as
//! explicit enum variants. Every state change must go through a StateTransition,
//! enabling validation, logging, and replay.

use serde::{Deserialize, Serialize};

use crate::drivers::common::DeviceInfo;
use crate::engine::{KeyCode, LayerId, Modifier};

/// Identifies all valid state transitions in the engine.
///
/// Every state change in the engine must be represented as a StateTransition.
/// This enables:
/// - Explicit validation of state changes
/// - Complete logging of state history
/// - Replay of state sequences for testing and debugging
/// - Clear documentation of all possible state changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateTransition {
    // ===================================================================
    // Engine Transitions - Core key and layer operations
    // ===================================================================
    /// A physical key was pressed.
    KeyPressed { key: KeyCode, timestamp: u64 },

    /// A physical key was released.
    KeyReleased { key: KeyCode, timestamp: u64 },

    /// A layer was pushed onto the layer stack.
    LayerPushed { layer: LayerId },

    /// A layer was popped from the layer stack.
    LayerPopped { layer: LayerId },

    /// A layer was activated (replaced top of stack).
    LayerActivated { layer: LayerId },

    /// A modifier was activated.
    ModifierActivated { modifier: Modifier },

    /// A modifier was deactivated.
    ModifierDeactivated { modifier: Modifier },

    /// A pending decision (tap-hold or combo) was resolved.
    DecisionResolved {
        /// Unique identifier for the pending decision.
        id: u64,
        /// How the decision was resolved.
        resolution: DecisionResolution,
    },

    /// A pending decision was added to the queue.
    DecisionQueued {
        /// Unique identifier for the pending decision.
        id: u64,
        /// Type of decision being queued.
        kind: DecisionKind,
    },

    // ===================================================================
    // Session Transitions - Recording and replay
    // ===================================================================
    /// A recording session was started.
    RecordingStarted { session_id: String },

    /// A recording session was stopped.
    RecordingStopped,

    /// A recording session was paused.
    RecordingPaused,

    /// A recording session was resumed.
    RecordingResumed,

    /// A replay session was started.
    ReplayStarted { session_id: String },

    /// A replay session was stopped.
    ReplayStopped,

    /// A replay session was paused.
    ReplayPaused,

    /// A replay session was resumed.
    ReplayResumed,

    /// A replay session completed normally.
    ReplayCompleted,

    // ===================================================================
    // Discovery Transitions - Device discovery
    // ===================================================================
    /// A new device was discovered.
    DeviceDiscovered { device: DeviceInfo },

    /// A device was lost (disconnected).
    DeviceLost { device_id: String },

    /// Device discovery session started.
    DiscoveryStarted,

    /// Device discovery session completed.
    DiscoveryCompleted,

    // ===================================================================
    // System Transitions - Configuration and engine lifecycle
    // ===================================================================
    /// Configuration was reloaded.
    ConfigReloaded,

    /// Engine state was reset to initial state.
    EngineReset,

    /// Fallback mode was activated due to an error.
    FallbackActivated { reason: String },

    /// Fallback mode was deactivated.
    FallbackDeactivated,

    /// Engine was initialized.
    EngineInitialized,

    /// Engine was shut down.
    EngineShutdown,
}

impl StateTransition {
    /// Get the timestamp of this transition, if available.
    ///
    /// Not all transitions include timestamps. Key press/release events
    /// include timestamps for precise timing reconstruction.
    pub fn timestamp(&self) -> Option<u64> {
        match self {
            Self::KeyPressed { timestamp, .. } | Self::KeyReleased { timestamp, .. } => {
                Some(*timestamp)
            }
            _ => None,
        }
    }

    /// Get the category of this transition.
    ///
    /// Categorizing transitions enables:
    /// - Filtering logs by category
    /// - Validating transitions based on current state kind
    /// - Grouping related transitions for analysis
    pub fn category(&self) -> TransitionCategory {
        match self {
            Self::KeyPressed { .. }
            | Self::KeyReleased { .. }
            | Self::LayerPushed { .. }
            | Self::LayerPopped { .. }
            | Self::LayerActivated { .. }
            | Self::ModifierActivated { .. }
            | Self::ModifierDeactivated { .. }
            | Self::DecisionResolved { .. }
            | Self::DecisionQueued { .. } => TransitionCategory::Engine,

            Self::RecordingStarted { .. }
            | Self::RecordingStopped
            | Self::RecordingPaused
            | Self::RecordingResumed
            | Self::ReplayStarted { .. }
            | Self::ReplayStopped
            | Self::ReplayPaused
            | Self::ReplayResumed
            | Self::ReplayCompleted => TransitionCategory::Session,

            Self::DeviceDiscovered { .. }
            | Self::DeviceLost { .. }
            | Self::DiscoveryStarted
            | Self::DiscoveryCompleted => TransitionCategory::Discovery,

            Self::ConfigReloaded
            | Self::EngineReset
            | Self::FallbackActivated { .. }
            | Self::FallbackDeactivated
            | Self::EngineInitialized
            | Self::EngineShutdown => TransitionCategory::System,
        }
    }

    /// Get a human-readable name for this transition.
    pub fn name(&self) -> &'static str {
        match self {
            Self::KeyPressed { .. } => "KeyPressed",
            Self::KeyReleased { .. } => "KeyReleased",
            Self::LayerPushed { .. } => "LayerPushed",
            Self::LayerPopped { .. } => "LayerPopped",
            Self::LayerActivated { .. } => "LayerActivated",
            Self::ModifierActivated { .. } => "ModifierActivated",
            Self::ModifierDeactivated { .. } => "ModifierDeactivated",
            Self::DecisionResolved { .. } => "DecisionResolved",
            Self::DecisionQueued { .. } => "DecisionQueued",
            Self::RecordingStarted { .. } => "RecordingStarted",
            Self::RecordingStopped => "RecordingStopped",
            Self::RecordingPaused => "RecordingPaused",
            Self::RecordingResumed => "RecordingResumed",
            Self::ReplayStarted { .. } => "ReplayStarted",
            Self::ReplayStopped => "ReplayStopped",
            Self::ReplayPaused => "ReplayPaused",
            Self::ReplayResumed => "ReplayResumed",
            Self::ReplayCompleted => "ReplayCompleted",
            Self::DeviceDiscovered { .. } => "DeviceDiscovered",
            Self::DeviceLost { .. } => "DeviceLost",
            Self::DiscoveryStarted => "DiscoveryStarted",
            Self::DiscoveryCompleted => "DiscoveryCompleted",
            Self::ConfigReloaded => "ConfigReloaded",
            Self::EngineReset => "EngineReset",
            Self::FallbackActivated { .. } => "FallbackActivated",
            Self::FallbackDeactivated => "FallbackDeactivated",
            Self::EngineInitialized => "EngineInitialized",
            Self::EngineShutdown => "EngineShutdown",
        }
    }
}

/// Categories of state transitions.
///
/// Transitions are grouped into categories to enable:
/// - Filtering logs by type of activity
/// - Different validation rules per category
/// - Analysis of specific subsystems
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransitionCategory {
    /// Core engine operations: keys, layers, modifiers, decisions.
    Engine,
    /// Recording and replay session management.
    Session,
    /// Device discovery and connection management.
    Discovery,
    /// System-level operations: config, initialization, shutdown.
    System,
}

impl TransitionCategory {
    /// Get a human-readable name for this category.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Engine => "Engine",
            Self::Session => "Session",
            Self::Discovery => "Discovery",
            Self::System => "System",
        }
    }
}

/// How a pending decision was resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecisionResolution {
    /// Tap-hold resolved as tap.
    Tap,
    /// Tap-hold resolved as hold.
    Hold,
    /// Decision timed out.
    Timeout,
    /// Decision was cancelled (e.g., key released before timeout).
    Cancelled,
    /// Combo was triggered.
    ComboTriggered,
    /// Combo was not triggered.
    ComboFailed,
}

/// Type of pending decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecisionKind {
    /// Tap-hold decision for a key.
    TapHold,
    /// Combo detection.
    Combo,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transition_category() {
        let key_press = StateTransition::KeyPressed {
            key: KeyCode::A,
            timestamp: 1000,
        };
        assert_eq!(key_press.category(), TransitionCategory::Engine);

        let recording = StateTransition::RecordingStarted {
            session_id: "test".to_string(),
        };
        assert_eq!(recording.category(), TransitionCategory::Session);

        let discovery = StateTransition::DiscoveryStarted;
        assert_eq!(discovery.category(), TransitionCategory::Discovery);

        let system = StateTransition::EngineReset;
        assert_eq!(system.category(), TransitionCategory::System);
    }

    #[test]
    fn test_transition_timestamp() {
        let key_press = StateTransition::KeyPressed {
            key: KeyCode::A,
            timestamp: 1000,
        };
        assert_eq!(key_press.timestamp(), Some(1000));

        let layer_push = StateTransition::LayerPushed { layer: 1 };
        assert_eq!(layer_push.timestamp(), None);
    }

    #[test]
    fn test_transition_name() {
        let key_press = StateTransition::KeyPressed {
            key: KeyCode::A,
            timestamp: 1000,
        };
        assert_eq!(key_press.name(), "KeyPressed");

        let recording = StateTransition::RecordingStarted {
            session_id: "test".to_string(),
        };
        assert_eq!(recording.name(), "RecordingStarted");
    }

    #[test]
    fn test_serialization() {
        let transition = StateTransition::KeyPressed {
            key: KeyCode::A,
            timestamp: 1000,
        };

        let json = serde_json::to_string(&transition).unwrap();
        let deserialized: StateTransition = serde_json::from_str(&json).unwrap();

        assert_eq!(transition.name(), deserialized.name());
        assert_eq!(transition.timestamp(), deserialized.timestamp());
    }

    #[test]
    fn test_decision_resolution() {
        let resolution = DecisionResolution::Tap;
        let json = serde_json::to_string(&resolution).unwrap();
        let deserialized: DecisionResolution = serde_json::from_str(&json).unwrap();
        assert_eq!(resolution, deserialized);
    }

    #[test]
    fn test_decision_kind() {
        let kind = DecisionKind::TapHold;
        let json = serde_json::to_string(&kind).unwrap();
        let deserialized: DecisionKind = serde_json::from_str(&json).unwrap();
        assert_eq!(kind, deserialized);
    }
}
