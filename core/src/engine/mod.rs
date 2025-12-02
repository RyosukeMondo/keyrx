//! Core engine module for event processing.

mod advanced;
mod decision;
mod event_loop;
mod event_recording;
mod state;
mod types;

pub use advanced::{AdvancedEngine, EngineState, PressedKeyState};
pub use decision::{
    ComboDef, ComboRegistry, DecisionQueue, DecisionResolution, PendingDecision,
    PendingDecisionState, TimingConfig,
};
pub use event_loop::Engine;
pub use event_recording::{
    infer_decision_type, DecisionType, EventRecord, EventRecordBuilder, EventRecorder,
    RecordingError, SessionFile, SESSION_FILE_VERSION,
};
pub use state::{
    HoldAction, KeyStateTracker, Layer, LayerAction, LayerId, LayerStack, Modifier, ModifierSet,
    ModifierState, StandardModifier, StandardModifiers, VirtualModifiers,
};
pub use types::{InputEvent, KeyCode, OutputAction, RemapAction};
