//! Core engine module for event processing.

mod advanced;
mod decision;
pub mod decision_engine;
mod event_loop;
mod event_recorder;
mod event_recording;
pub mod layer_actions;
mod processing;
pub mod replay;
mod state;
pub mod tracing;
mod types;

// Backward compatibility: EngineState is the old snapshot type
pub use advanced::{AdvancedEngine, EngineStateSnapshot as EngineState, PressedKeyState};
pub use decision::{
    ComboDef, ComboRegistry, DecisionQueue, DecisionResolution, PendingDecision,
    PendingDecisionState, TimingConfig,
};
pub use event_loop::Engine;
pub use event_recorder::EventRecorder;
pub use event_recording::{
    infer_decision_type, DecisionType, EventRecord, EventRecordBuilder, RecordingError,
    SessionFile, SESSION_FILE_VERSION,
};
pub use replay::{ReplayError, ReplaySession, ReplayState};
pub use state::{
    HoldAction, KeyStateTracker, Layer, LayerAction, LayerId, LayerStack, Modifier, ModifierSet,
    ModifierState, StandardModifier, StandardModifiers, VirtualModifiers,
};

// Export the unified EngineState but with a different name to avoid confusion with the snapshot
// For backward compatibility, EngineState refers to the old snapshot type (now EngineStateSnapshot)
pub use state::EngineState as UnifiedEngineState;
pub use tracing::{EngineTracer, SpanGuard, TracingError, TracingResult};
pub use types::{InputEvent, KeyCode, OutputAction, RemapAction};
