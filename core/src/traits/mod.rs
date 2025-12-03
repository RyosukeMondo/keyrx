//! Dependency Injection trait definitions.
//!
//! All external dependencies are abstracted behind traits for testability.

mod input_source;
mod script_runtime;
mod state;
mod state_store;

pub use input_source::InputSource;
pub use script_runtime::ScriptRuntime;
pub use state::{KeyStateProvider, LayerProvider, ModifierProvider};
pub use state_store::StateStore;
