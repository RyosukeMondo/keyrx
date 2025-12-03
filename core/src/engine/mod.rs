//! Core engine module for event processing.

mod advanced;
mod decision;
pub mod decision_engine;
mod event_loop;
mod event_recorder;
mod event_recording;
mod processing;
pub mod replay;
mod state;
pub mod tracing;
mod types;

pub use advanced::{AdvancedEngine, EngineState, PressedKeyState};
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
pub use tracing::{EngineTracer, SpanGuard, TracingError, TracingResult};
pub use types::{InputEvent, KeyCode, OutputAction, RemapAction};
