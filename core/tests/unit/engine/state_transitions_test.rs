//! Comprehensive tests for state machine transitions.
//!
//! This test suite verifies all valid and invalid state transitions,
//! ensuring the state graph enforces correct state machine behavior.

use keyrx_core::drivers::common::DeviceInfo;
use keyrx_core::drivers::keycodes::KeyCode;
use keyrx_core::engine::transitions::{
    DecisionKind, DecisionResolution, StateGraph, StateKind, StateTransition, TransitionCategory,
};
use keyrx_core::engine::{LayerId, Modifier};
use std::path::PathBuf;

// ===================================================================
// Helper Functions
// ===================================================================

fn create_test_device() -> DeviceInfo {
    DeviceInfo {
        path: PathBuf::from("/dev/input/test"),
        name: "Test Device".to_string(),
        vendor_id: 0x1234,
        product_id: 0x5678,
        is_keyboard: true,
    }
}

// ===================================================================
// System Transition Tests - Valid Cases
// ===================================================================

#[test]
fn test_engine_initialization() {
    let graph = StateGraph::new();
    let transition = StateTransition::EngineInitialized;

    assert!(graph.is_valid(StateKind::Uninitialized, &transition));
    assert_eq!(
        graph.apply(StateKind::Uninitialized, &transition).unwrap(),
        StateKind::Idle
    );
}

#[test]
fn test_engine_shutdown_from_various_states() {
    let graph = StateGraph::new();
    let transition = StateTransition::EngineShutdown;

    // Can shutdown from any initialized state
    let valid_states = [
        StateKind::Idle,
        StateKind::Typing,
        StateKind::ModifierHeld,
        StateKind::LayerActive,
        StateKind::Pending,
        StateKind::Recording,
        StateKind::RecordingPaused,
        StateKind::Replaying,
        StateKind::ReplayPaused,
        StateKind::Discovery,
        StateKind::Fallback,
    ];

    for state in valid_states {
        assert!(
            graph.is_valid(state, &transition),
            "Shutdown should be valid from {:?}",
            state
        );
        assert_eq!(
            graph.apply(state, &transition).unwrap(),
            StateKind::ShuttingDown,
            "Shutdown from {:?} should transition to ShuttingDown",
            state
        );
    }
}

#[test]
fn test_config_reload() {
    let graph = StateGraph::new();
    let transition = StateTransition::ConfigReloaded;

    // Config reload allowed from initialized states that allow transitions
    let valid_states = [
        StateKind::Idle,
        StateKind::Typing,
        StateKind::ModifierHeld,
        StateKind::LayerActive,
        StateKind::Recording,
        StateKind::Replaying,
        StateKind::Discovery,
    ];

    for state in valid_states {
        assert!(
            graph.is_valid(state, &transition),
            "ConfigReloaded should be valid from {:?}",
            state
        );
        // Config reload stays in same state
        assert_eq!(
            graph.apply(state, &transition).unwrap(),
            state,
            "ConfigReloaded from {:?} should stay in same state",
            state
        );
    }
}

#[test]
fn test_engine_reset() {
    let graph = StateGraph::new();
    let transition = StateTransition::EngineReset;

    // Engine reset allowed from initialized states
    let valid_states = [
        StateKind::Idle,
        StateKind::Typing,
        StateKind::ModifierHeld,
        StateKind::LayerActive,
        StateKind::Pending,
    ];

    for state in valid_states {
        assert!(
            graph.is_valid(state, &transition),
            "EngineReset should be valid from {:?}",
            state
        );
        assert_eq!(
            graph.apply(state, &transition).unwrap(),
            StateKind::Idle,
            "EngineReset from {:?} should transition to Idle",
            state
        );
    }
}

#[test]
fn test_fallback_activation() {
    let graph = StateGraph::new();
    let transition = StateTransition::FallbackActivated {
        reason: "Test error".to_string(),
    };

    // Fallback can be activated from any non-fallback state
    let valid_states = [
        StateKind::Idle,
        StateKind::Typing,
        StateKind::ModifierHeld,
        StateKind::LayerActive,
        StateKind::Pending,
        StateKind::Recording,
        StateKind::Replaying,
        StateKind::Discovery,
    ];

    for state in valid_states {
        assert!(
            graph.is_valid(state, &transition),
            "FallbackActivated should be valid from {:?}",
            state
        );
        assert_eq!(
            graph.apply(state, &transition).unwrap(),
            StateKind::Fallback,
            "FallbackActivated from {:?} should transition to Fallback",
            state
        );
    }
}

#[test]
fn test_fallback_deactivation() {
    let graph = StateGraph::new();
    let transition = StateTransition::FallbackDeactivated;

    assert!(graph.is_valid(StateKind::Fallback, &transition));
    assert_eq!(
        graph.apply(StateKind::Fallback, &transition).unwrap(),
        StateKind::Idle
    );
}

// ===================================================================
// Engine Transition Tests - Valid Cases
// ===================================================================

#[test]
fn test_key_press_transitions() {
    let graph = StateGraph::new();
    let transition = StateTransition::KeyPressed {
        key: KeyCode::A,
        timestamp: 1000,
    };

    // Valid from states that allow input
    let valid_states = [
        StateKind::Idle,
        StateKind::Typing,
        StateKind::ModifierHeld,
        StateKind::LayerActive,
        StateKind::Pending,
        StateKind::Recording,
        StateKind::RecordingPaused,
        StateKind::Replaying,
        StateKind::ReplayPaused,
        StateKind::Discovery,
    ];

    for state in valid_states {
        assert!(
            graph.is_valid(state, &transition),
            "KeyPressed should be valid from {:?}",
            state
        );
    }

    // From Idle → Typing
    assert_eq!(
        graph.apply(StateKind::Idle, &transition).unwrap(),
        StateKind::Typing
    );

    // From Typing → Typing
    assert_eq!(
        graph.apply(StateKind::Typing, &transition).unwrap(),
        StateKind::Typing
    );
}

#[test]
fn test_key_release_transitions() {
    let graph = StateGraph::new();
    let transition = StateTransition::KeyReleased {
        key: KeyCode::A,
        timestamp: 2000,
    };

    // Valid from states that allow input
    let valid_states = [
        StateKind::Typing,
        StateKind::ModifierHeld,
        StateKind::LayerActive,
        StateKind::Recording,
        StateKind::Replaying,
    ];

    for state in valid_states {
        assert!(
            graph.is_valid(state, &transition),
            "KeyReleased should be valid from {:?}",
            state
        );
        // Key release stays in same state (actual state change depends on engine state)
        assert_eq!(
            graph.apply(state, &transition).unwrap(),
            state,
            "KeyReleased from {:?} should stay in same state",
            state
        );
    }
}

#[test]
fn test_layer_push_transitions() {
    let graph = StateGraph::new();
    let transition = StateTransition::LayerPushed { layer: 1 };

    // From Idle → LayerActive
    assert!(graph.is_valid(StateKind::Idle, &transition));
    assert_eq!(
        graph.apply(StateKind::Idle, &transition).unwrap(),
        StateKind::LayerActive
    );

    // From Typing → stays in Typing
    assert!(graph.is_valid(StateKind::Typing, &transition));
    assert_eq!(
        graph.apply(StateKind::Typing, &transition).unwrap(),
        StateKind::Typing
    );
}

#[test]
fn test_layer_pop_transitions() {
    let graph = StateGraph::new();
    let transition = StateTransition::LayerPopped { layer: 1 };

    // Valid from states that allow input
    assert!(graph.is_valid(StateKind::LayerActive, &transition));
    assert_eq!(
        graph.apply(StateKind::LayerActive, &transition).unwrap(),
        StateKind::LayerActive
    );
}

#[test]
fn test_layer_activated_transitions() {
    let graph = StateGraph::new();
    let transition = StateTransition::LayerActivated { layer: 2 };

    // Valid from states that allow input
    let valid_states = [
        StateKind::Idle,
        StateKind::Typing,
        StateKind::LayerActive,
        StateKind::Recording,
    ];

    for state in valid_states {
        assert!(
            graph.is_valid(state, &transition),
            "LayerActivated should be valid from {:?}",
            state
        );
        assert_eq!(
            graph.apply(state, &transition).unwrap(),
            StateKind::LayerActive,
            "LayerActivated should transition to LayerActive"
        );
    }
}

#[test]
fn test_modifier_activation() {
    let graph = StateGraph::new();
    let transition = StateTransition::ModifierActivated {
        modifier: Modifier::Virtual(0),
    };

    // From Idle → ModifierHeld
    assert!(graph.is_valid(StateKind::Idle, &transition));
    assert_eq!(
        graph.apply(StateKind::Idle, &transition).unwrap(),
        StateKind::ModifierHeld
    );

    // From Typing → ModifierHeld
    assert!(graph.is_valid(StateKind::Typing, &transition));
    assert_eq!(
        graph.apply(StateKind::Typing, &transition).unwrap(),
        StateKind::ModifierHeld
    );

    // From LayerActive → stays in LayerActive
    assert!(graph.is_valid(StateKind::LayerActive, &transition));
    assert_eq!(
        graph.apply(StateKind::LayerActive, &transition).unwrap(),
        StateKind::LayerActive
    );
}

#[test]
fn test_modifier_deactivation() {
    let graph = StateGraph::new();
    let transition = StateTransition::ModifierDeactivated {
        modifier: Modifier::Virtual(0),
    };

    // Valid from states that allow input
    assert!(graph.is_valid(StateKind::ModifierHeld, &transition));
    assert_eq!(
        graph.apply(StateKind::ModifierHeld, &transition).unwrap(),
        StateKind::ModifierHeld
    );
}

#[test]
fn test_decision_queued() {
    let graph = StateGraph::new();
    let transition = StateTransition::DecisionQueued {
        id: 1,
        kind: DecisionKind::TapHold,
    };

    // Valid from states that allow input
    let valid_states = [StateKind::Idle, StateKind::Typing, StateKind::ModifierHeld];

    for state in valid_states {
        assert!(
            graph.is_valid(state, &transition),
            "DecisionQueued should be valid from {:?}",
            state
        );
        assert_eq!(
            graph.apply(state, &transition).unwrap(),
            StateKind::Pending,
            "DecisionQueued should transition to Pending"
        );
    }
}

#[test]
fn test_decision_resolved() {
    let graph = StateGraph::new();
    let transition = StateTransition::DecisionResolved {
        id: 1,
        resolution: DecisionResolution::Tapped,
    };

    // From Pending → Idle
    assert!(graph.is_valid(StateKind::Pending, &transition));
    assert_eq!(
        graph.apply(StateKind::Pending, &transition).unwrap(),
        StateKind::Idle
    );

    // From other states → stays same
    assert!(graph.is_valid(StateKind::Typing, &transition));
    assert_eq!(
        graph.apply(StateKind::Typing, &transition).unwrap(),
        StateKind::Typing
    );
}

// ===================================================================
// Session Transition Tests - Valid Cases
// ===================================================================

#[test]
fn test_recording_lifecycle() {
    let graph = StateGraph::new();

    // Start recording from Idle
    let start = StateTransition::RecordingStarted {
        session_id: "test_session".to_string(),
    };
    assert!(graph.is_valid(StateKind::Idle, &start));
    assert_eq!(
        graph.apply(StateKind::Idle, &start).unwrap(),
        StateKind::Recording
    );

    // Pause recording
    let pause = StateTransition::RecordingPaused;
    assert!(graph.is_valid(StateKind::Recording, &pause));
    assert_eq!(
        graph.apply(StateKind::Recording, &pause).unwrap(),
        StateKind::RecordingPaused
    );

    // Resume recording
    let resume = StateTransition::RecordingResumed;
    assert!(graph.is_valid(StateKind::RecordingPaused, &resume));
    assert_eq!(
        graph.apply(StateKind::RecordingPaused, &resume).unwrap(),
        StateKind::Recording
    );

    // Stop recording from active state
    let stop = StateTransition::RecordingStopped;
    assert!(graph.is_valid(StateKind::Recording, &stop));
    assert_eq!(
        graph.apply(StateKind::Recording, &stop).unwrap(),
        StateKind::Idle
    );

    // Stop recording from paused state
    assert!(graph.is_valid(StateKind::RecordingPaused, &stop));
    assert_eq!(
        graph.apply(StateKind::RecordingPaused, &stop).unwrap(),
        StateKind::Idle
    );
}

#[test]
fn test_replay_lifecycle() {
    let graph = StateGraph::new();

    // Start replay from Idle
    let start = StateTransition::ReplayStarted {
        session_id: "test_replay".to_string(),
    };
    assert!(graph.is_valid(StateKind::Idle, &start));
    assert_eq!(
        graph.apply(StateKind::Idle, &start).unwrap(),
        StateKind::Replaying
    );

    // Pause replay
    let pause = StateTransition::ReplayPaused;
    assert!(graph.is_valid(StateKind::Replaying, &pause));
    assert_eq!(
        graph.apply(StateKind::Replaying, &pause).unwrap(),
        StateKind::ReplayPaused
    );

    // Resume replay
    let resume = StateTransition::ReplayResumed;
    assert!(graph.is_valid(StateKind::ReplayPaused, &resume));
    assert_eq!(
        graph.apply(StateKind::ReplayPaused, &resume).unwrap(),
        StateKind::Replaying
    );

    // Complete replay
    let complete = StateTransition::ReplayCompleted;
    assert!(graph.is_valid(StateKind::Replaying, &complete));
    assert_eq!(
        graph.apply(StateKind::Replaying, &complete).unwrap(),
        StateKind::Idle
    );

    // Stop replay from active state
    let stop = StateTransition::ReplayStopped;
    assert!(graph.is_valid(StateKind::Replaying, &stop));
    assert_eq!(
        graph.apply(StateKind::Replaying, &stop).unwrap(),
        StateKind::Idle
    );

    // Stop replay from paused state
    assert!(graph.is_valid(StateKind::ReplayPaused, &stop));
    assert_eq!(
        graph.apply(StateKind::ReplayPaused, &stop).unwrap(),
        StateKind::Idle
    );
}

#[test]
fn test_recording_start_from_various_states() {
    let graph = StateGraph::new();
    let transition = StateTransition::RecordingStarted {
        session_id: "test".to_string(),
    };

    // Can start from non-session states that allow transitions
    let valid_states = [
        StateKind::Idle,
        StateKind::Typing,
        StateKind::ModifierHeld,
        StateKind::LayerActive,
        StateKind::Pending,
        StateKind::Discovery,
    ];

    for state in valid_states {
        assert!(
            graph.is_valid(state, &transition),
            "RecordingStarted should be valid from {:?}",
            state
        );
    }
}

#[test]
fn test_replay_start_from_various_states() {
    let graph = StateGraph::new();
    let transition = StateTransition::ReplayStarted {
        session_id: "test".to_string(),
    };

    // Can start from non-session states that allow transitions
    let valid_states = [
        StateKind::Idle,
        StateKind::Typing,
        StateKind::ModifierHeld,
        StateKind::LayerActive,
        StateKind::Pending,
        StateKind::Discovery,
    ];

    for state in valid_states {
        assert!(
            graph.is_valid(state, &transition),
            "ReplayStarted should be valid from {:?}",
            state
        );
    }
}

// ===================================================================
// Discovery Transition Tests - Valid Cases
// ===================================================================

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

    // Complete discovery
    let complete = StateTransition::DiscoveryCompleted;
    assert!(graph.is_valid(StateKind::Discovery, &complete));
    assert_eq!(
        graph.apply(StateKind::Discovery, &complete).unwrap(),
        StateKind::Idle
    );
}

#[test]
fn test_device_discovered() {
    let graph = StateGraph::new();
    let transition = StateTransition::DeviceDiscovered {
        device: create_test_device(),
    };

    // Device events allowed from any state that allows transitions
    let valid_states = [
        StateKind::Idle,
        StateKind::Discovery,
        StateKind::Typing,
        StateKind::Recording,
        StateKind::Replaying,
    ];

    for state in valid_states {
        assert!(
            graph.is_valid(state, &transition),
            "DeviceDiscovered should be valid from {:?}",
            state
        );
        // State remains unchanged
        assert_eq!(
            graph.apply(state, &transition).unwrap(),
            state,
            "DeviceDiscovered from {:?} should stay in same state",
            state
        );
    }
}

#[test]
fn test_device_lost() {
    let graph = StateGraph::new();
    let transition = StateTransition::DeviceLost {
        device_id: "test_device".to_string(),
    };

    // Device events allowed from any state that allows transitions
    let valid_states = [
        StateKind::Idle,
        StateKind::Discovery,
        StateKind::Typing,
        StateKind::Recording,
    ];

    for state in valid_states {
        assert!(
            graph.is_valid(state, &transition),
            "DeviceLost should be valid from {:?}",
            state
        );
        assert_eq!(
            graph.apply(state, &transition).unwrap(),
            state,
            "DeviceLost should stay in same state"
        );
    }
}

// ===================================================================
// Invalid Transition Tests
// ===================================================================

#[test]
fn test_invalid_initialization() {
    let graph = StateGraph::new();
    let transition = StateTransition::EngineInitialized;

    // Cannot initialize from any state except Uninitialized
    let invalid_states = [
        StateKind::Idle,
        StateKind::Typing,
        StateKind::ShuttingDown,
        StateKind::Recording,
    ];

    for state in invalid_states {
        assert!(
            !graph.is_valid(state, &transition),
            "EngineInitialized should be invalid from {:?}",
            state
        );
        assert!(
            graph.apply(state, &transition).is_err(),
            "EngineInitialized should fail from {:?}",
            state
        );
    }
}

#[test]
fn test_invalid_shutdown() {
    let graph = StateGraph::new();
    let transition = StateTransition::EngineShutdown;

    // Cannot shutdown from uninitialized
    assert!(!graph.is_valid(StateKind::Uninitialized, &transition));
    let result = graph.apply(StateKind::Uninitialized, &transition);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .reason
        .contains("Cannot shutdown before initialization"));
}

#[test]
fn test_invalid_fallback_activation() {
    let graph = StateGraph::new();
    let transition = StateTransition::FallbackActivated {
        reason: "test".to_string(),
    };

    // Cannot activate fallback when already in fallback
    assert!(!graph.is_valid(StateKind::Fallback, &transition));
    let result = graph.apply(StateKind::Fallback, &transition);
    assert!(result.is_err());
    assert!(result.unwrap_err().reason.contains("Already in fallback"));
}

#[test]
fn test_invalid_fallback_deactivation() {
    let graph = StateGraph::new();
    let transition = StateTransition::FallbackDeactivated;

    // Cannot deactivate fallback when not in fallback
    let invalid_states = [StateKind::Idle, StateKind::Typing, StateKind::Recording];

    for state in invalid_states {
        assert!(
            !graph.is_valid(state, &transition),
            "FallbackDeactivated should be invalid from {:?}",
            state
        );
        let result = graph.apply(state, &transition);
        assert!(
            result.is_err(),
            "FallbackDeactivated should fail from {:?}",
            state
        );
        assert!(result.unwrap_err().reason.contains("Not in fallback"));
    }
}

#[test]
fn test_invalid_key_press_from_blocked_states() {
    let graph = StateGraph::new();
    let transition = StateTransition::KeyPressed {
        key: KeyCode::A,
        timestamp: 1000,
    };

    // Cannot press keys in states that block input
    let invalid_states = [
        StateKind::Uninitialized,
        StateKind::ShuttingDown,
        StateKind::Fallback,
    ];

    for state in invalid_states {
        assert!(
            !graph.is_valid(state, &transition),
            "KeyPressed should be invalid from {:?}",
            state
        );
        let result = graph.apply(state, &transition);
        assert!(result.is_err(), "KeyPressed should fail from {:?}", state);
    }
}

#[test]
fn test_invalid_recording_start() {
    let graph = StateGraph::new();
    let transition = StateTransition::RecordingStarted {
        session_id: "test".to_string(),
    };

    // Cannot start recording during active sessions
    let invalid_states = [
        StateKind::Recording,
        StateKind::RecordingPaused,
        StateKind::Replaying,
        StateKind::ReplayPaused,
    ];

    for state in invalid_states {
        assert!(
            !graph.is_valid(state, &transition),
            "RecordingStarted should be invalid from {:?}",
            state
        );
        let result = graph.apply(state, &transition);
        assert!(
            result.is_err(),
            "RecordingStarted should fail from {:?}",
            state
        );
    }

    // Cannot start from states that don't allow transitions
    assert!(!graph.is_valid(StateKind::Uninitialized, &transition));
    assert!(!graph.is_valid(StateKind::ShuttingDown, &transition));
}

#[test]
fn test_invalid_recording_pause() {
    let graph = StateGraph::new();
    let transition = StateTransition::RecordingPaused;

    // Can only pause active recording
    let invalid_states = [
        StateKind::Idle,
        StateKind::Typing,
        StateKind::RecordingPaused,
        StateKind::Replaying,
    ];

    for state in invalid_states {
        assert!(
            !graph.is_valid(state, &transition),
            "RecordingPaused should be invalid from {:?}",
            state
        );
        let result = graph.apply(state, &transition);
        assert!(
            result.is_err(),
            "RecordingPaused should fail from {:?}",
            state
        );
        assert!(result
            .unwrap_err()
            .reason
            .contains("Can only pause active recording"));
    }
}

#[test]
fn test_invalid_recording_resume() {
    let graph = StateGraph::new();
    let transition = StateTransition::RecordingResumed;

    // Can only resume paused recording
    let invalid_states = [
        StateKind::Idle,
        StateKind::Recording,
        StateKind::Replaying,
        StateKind::ReplayPaused,
    ];

    for state in invalid_states {
        assert!(
            !graph.is_valid(state, &transition),
            "RecordingResumed should be invalid from {:?}",
            state
        );
        let result = graph.apply(state, &transition);
        assert!(
            result.is_err(),
            "RecordingResumed should fail from {:?}",
            state
        );
        assert!(result
            .unwrap_err()
            .reason
            .contains("Can only resume paused recording"));
    }
}

#[test]
fn test_invalid_recording_stop() {
    let graph = StateGraph::new();
    let transition = StateTransition::RecordingStopped;

    // Cannot stop when no recording active
    let invalid_states = [
        StateKind::Idle,
        StateKind::Typing,
        StateKind::Replaying,
        StateKind::Discovery,
    ];

    for state in invalid_states {
        assert!(
            !graph.is_valid(state, &transition),
            "RecordingStopped should be invalid from {:?}",
            state
        );
        let result = graph.apply(state, &transition);
        assert!(
            result.is_err(),
            "RecordingStopped should fail from {:?}",
            state
        );
        assert!(result
            .unwrap_err()
            .reason
            .contains("No recording session active"));
    }
}

#[test]
fn test_invalid_replay_start() {
    let graph = StateGraph::new();
    let transition = StateTransition::ReplayStarted {
        session_id: "test".to_string(),
    };

    // Cannot start replay during active sessions
    let invalid_states = [
        StateKind::Recording,
        StateKind::RecordingPaused,
        StateKind::Replaying,
        StateKind::ReplayPaused,
    ];

    for state in invalid_states {
        assert!(
            !graph.is_valid(state, &transition),
            "ReplayStarted should be invalid from {:?}",
            state
        );
        let result = graph.apply(state, &transition);
        assert!(
            result.is_err(),
            "ReplayStarted should fail from {:?}",
            state
        );
    }
}

#[test]
fn test_invalid_replay_pause() {
    let graph = StateGraph::new();
    let transition = StateTransition::ReplayPaused;

    // Can only pause active replay
    let invalid_states = [
        StateKind::Idle,
        StateKind::Recording,
        StateKind::ReplayPaused,
    ];

    for state in invalid_states {
        assert!(
            !graph.is_valid(state, &transition),
            "ReplayPaused should be invalid from {:?}",
            state
        );
        let result = graph.apply(state, &transition);
        assert!(result.is_err(), "ReplayPaused should fail from {:?}", state);
        assert!(result
            .unwrap_err()
            .reason
            .contains("Can only pause active replay"));
    }
}

#[test]
fn test_invalid_replay_resume() {
    let graph = StateGraph::new();
    let transition = StateTransition::ReplayResumed;

    // Can only resume paused replay
    let invalid_states = [StateKind::Idle, StateKind::Replaying, StateKind::Recording];

    for state in invalid_states {
        assert!(
            !graph.is_valid(state, &transition),
            "ReplayResumed should be invalid from {:?}",
            state
        );
        let result = graph.apply(state, &transition);
        assert!(
            result.is_err(),
            "ReplayResumed should fail from {:?}",
            state
        );
        assert!(result
            .unwrap_err()
            .reason
            .contains("Can only resume paused replay"));
    }
}

#[test]
fn test_invalid_replay_complete() {
    let graph = StateGraph::new();
    let transition = StateTransition::ReplayCompleted;

    // Can only complete active replay
    let invalid_states = [
        StateKind::Idle,
        StateKind::Recording,
        StateKind::ReplayPaused,
    ];

    for state in invalid_states {
        assert!(
            !graph.is_valid(state, &transition),
            "ReplayCompleted should be invalid from {:?}",
            state
        );
        let result = graph.apply(state, &transition);
        assert!(
            result.is_err(),
            "ReplayCompleted should fail from {:?}",
            state
        );
        assert!(result
            .unwrap_err()
            .reason
            .contains("Can only complete active replay"));
    }
}

#[test]
fn test_invalid_replay_stop() {
    let graph = StateGraph::new();
    let transition = StateTransition::ReplayStopped;

    // Cannot stop when no replay active
    let invalid_states = [
        StateKind::Idle,
        StateKind::Typing,
        StateKind::Recording,
        StateKind::Discovery,
    ];

    for state in invalid_states {
        assert!(
            !graph.is_valid(state, &transition),
            "ReplayStopped should be invalid from {:?}",
            state
        );
        let result = graph.apply(state, &transition);
        assert!(
            result.is_err(),
            "ReplayStopped should fail from {:?}",
            state
        );
        assert!(result
            .unwrap_err()
            .reason
            .contains("No replay session active"));
    }
}

#[test]
fn test_invalid_discovery_start() {
    let graph = StateGraph::new();
    let transition = StateTransition::DiscoveryStarted;

    // Cannot start discovery when already in progress
    assert!(!graph.is_valid(StateKind::Discovery, &transition));
    let result = graph.apply(StateKind::Discovery, &transition);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .reason
        .contains("Discovery already in progress"));

    // Cannot start from states that don't allow transitions
    assert!(!graph.is_valid(StateKind::ShuttingDown, &transition));
}

#[test]
fn test_invalid_discovery_complete() {
    let graph = StateGraph::new();
    let transition = StateTransition::DiscoveryCompleted;

    // Can only complete active discovery
    let invalid_states = [StateKind::Idle, StateKind::Typing, StateKind::Recording];

    for state in invalid_states {
        assert!(
            !graph.is_valid(state, &transition),
            "DiscoveryCompleted should be invalid from {:?}",
            state
        );
        let result = graph.apply(state, &transition);
        assert!(
            result.is_err(),
            "DiscoveryCompleted should fail from {:?}",
            state
        );
        assert!(result
            .unwrap_err()
            .reason
            .contains("No discovery session active"));
    }
}

#[test]
fn test_shutting_down_only_allows_shutdown() {
    let graph = StateGraph::new();

    // Only EngineShutdown allowed
    assert!(graph.is_valid(StateKind::ShuttingDown, &StateTransition::EngineShutdown));

    // All other transitions rejected
    let invalid_transitions = vec![
        StateTransition::KeyPressed {
            key: KeyCode::A,
            timestamp: 1000,
        },
        StateTransition::RecordingStarted {
            session_id: "test".to_string(),
        },
        StateTransition::LayerPushed { layer: 1 },
        StateTransition::ModifierActivated {
            modifier: Modifier::Virtual(0),
        },
        StateTransition::DiscoveryStarted,
    ];

    for transition in invalid_transitions {
        assert!(
            !graph.is_valid(StateKind::ShuttingDown, &transition),
            "Transition {:?} should be invalid from ShuttingDown",
            transition.name()
        );
        let result = graph.apply(StateKind::ShuttingDown, &transition);
        assert!(
            result.is_err(),
            "Transition {:?} should fail from ShuttingDown",
            transition.name()
        );
        assert!(result
            .unwrap_err()
            .reason
            .contains("Engine is shutting down"));
    }
}

// ===================================================================
// Category Tests
// ===================================================================

#[test]
fn test_transition_categories() {
    assert_eq!(
        StateTransition::KeyPressed {
            key: KeyCode::A,
            timestamp: 1000
        }
        .category(),
        TransitionCategory::Engine
    );

    assert_eq!(
        StateTransition::RecordingStarted {
            session_id: "test".to_string()
        }
        .category(),
        TransitionCategory::Session
    );

    assert_eq!(
        StateTransition::DiscoveryStarted.category(),
        TransitionCategory::Discovery
    );

    assert_eq!(
        StateTransition::EngineInitialized.category(),
        TransitionCategory::System
    );
}

// ===================================================================
// Edge Case Tests
// ===================================================================

#[test]
fn test_config_reload_from_uninitialized() {
    let graph = StateGraph::new();
    let transition = StateTransition::ConfigReloaded;

    // Cannot reload config before initialization
    assert!(!graph.is_valid(StateKind::Uninitialized, &transition));
}

#[test]
fn test_device_events_from_blocked_states() {
    let graph = StateGraph::new();
    let discovered = StateTransition::DeviceDiscovered {
        device: create_test_device(),
    };

    // Device events not allowed from ShuttingDown
    assert!(!graph.is_valid(StateKind::ShuttingDown, &discovered));
}

#[test]
fn test_error_message_quality() {
    let graph = StateGraph::new();

    // Test that error messages are descriptive
    let result = graph.apply(StateKind::Idle, &StateTransition::RecordingPaused);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.from_state, StateKind::Idle);
    assert_eq!(err.transition_name, "RecordingPaused");
    assert_eq!(err.category, TransitionCategory::Session);
    assert!(err.reason.contains("active recording"));

    // Test error Display implementation
    let display = format!("{}", err);
    assert!(display.contains("RecordingPaused"));
    assert!(display.contains("Idle"));
}
