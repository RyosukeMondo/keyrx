//! Core engine module for event processing.

mod advanced;
pub mod coalescing;
mod decision;
pub mod decision_engine;
mod device_resolver;
mod event_loop;
mod event_recorder;
mod event_recording;
pub mod fallback;
pub mod layer_actions;
pub mod limits;
mod multi_device;
mod output;
mod processing;
mod profile_resolver;
pub mod recording;
pub mod replay;
mod session;
mod session_state;
pub mod state;
pub mod tracing;
pub mod transitions;
mod types;

pub use advanced::AdvancedEngine;
pub use coalescing::{CoalescingConfig, CoalescingEngine, EventBuffer};
pub use decision::{
    ComboDef, ComboRegistry, DecisionQueue, DecisionResolution, PendingDecision,
    PendingDecisionState, TimingConfig,
};
pub use device_resolver::{DeviceResolver, DeviceResolverError};
pub use event_loop::Engine;
pub use event_recorder::EventRecorder;
pub use event_recording::{
    infer_decision_type, DecisionType, EventRecord, EventRecordBuilder, RecordingError,
    SessionFile, SESSION_FILE_VERSION,
};
pub use fallback::{FallbackEngine, FallbackReason};
pub use limits::{
    ExecutionGuard, ResourceEnforcer, ResourceLimitError, ResourceLimits, ResourceUsageSnapshot,
};
pub use multi_device::{CoordinationAction, MultiDeviceCoordinator};
pub use output::OutputQueue;
pub use profile_resolver::{ProfileResolver, ProfileResolverError};
pub use replay::{ReplayError, ReplayManifest, ReplaySession, ReplayState};
pub use session::{HotplugAction, HotplugSession};
pub use session_state::{SessionState, SessionStatus};
pub use state::{
    HoldAction, KeyStateTracker, Layer, LayerAction, LayerId, LayerStack, Modifier, ModifierSet,
    ModifierState, StandardModifier, StandardModifiers, VirtualModifiers,
};

// Export the unified EngineState and its snapshot types
pub use state::snapshot::{PressedKey, StateSnapshot};
pub use state::EngineState;
pub use tracing::{EngineTracer, SpanGuard, TracingError, TracingResult};
pub use transitions::{StateTransition, TransitionCategory};
pub use types::{InputEvent, KeyCode, OutputAction, RemapAction};
