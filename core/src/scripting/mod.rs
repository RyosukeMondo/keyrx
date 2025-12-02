//! Rhai scripting integration.
//!
//! This module provides the scripting runtime for KeyRx:
//! - `runtime`: Core RhaiRuntime struct and ScriptRuntime implementation
//! - `bindings`: Rhai function registrations (remap, layer, modifier, timing)
//! - `builtins`: Helper types and utility functions
//! - `registry`: Remap registry for storing key mappings
//! - `helpers`: Key parsing and validation utilities

mod bindings;
mod builtins;
pub mod helpers;
mod registry;
mod runtime;

pub use registry::RemapRegistry;
pub use runtime::{clear_active_runtime, set_active_runtime, with_active_runtime, RhaiRuntime};
