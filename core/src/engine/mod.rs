//! Core engine module for event processing.

mod decision;
mod event_loop;
mod state;
mod types;

pub use decision::{
    ComboDef, ComboRegistry, DecisionQueue, DecisionResolution, PendingDecision, TimingConfig,
};
pub use event_loop::Engine;
pub use state::{
    HoldAction, KeyStateTracker, Layer, LayerAction, LayerId, LayerStack, Modifier, ModifierSet,
    ModifierState, StandardModifier, StandardModifiers, VirtualModifiers,
};
pub use types::{InputEvent, KeyCode, OutputAction, RemapAction};
