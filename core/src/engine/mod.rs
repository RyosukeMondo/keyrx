//! Core engine module for event processing.

mod event_loop;
mod state;
mod types;

pub use event_loop::Engine;
pub use state::{
    HoldAction, KeyStateTracker, Layer, LayerAction, LayerId, LayerStack, Modifier, ModifierSet,
    ModifierState, StandardModifier, StandardModifiers, VirtualModifiers,
};
pub use types::{InputEvent, KeyCode, OutputAction, RemapAction};
