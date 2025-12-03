//! FFI exports for Flutter integration.
//!
//! The exports are organized into domain-specific modules:
//! - `exports`: Core init/common functions
//! - `exports_device`: Device and key definitions
//! - `exports_engine`: Engine control and state callbacks
//! - `exports_session`: Script loading and discovery session management

mod callbacks;
mod exports;
mod exports_device;
mod exports_engine;
mod exports_session;

pub use callbacks::{
    callback_registry, CallbackRegistry, DiscoveryEventCallback, IsolatedRegistry,
    StateEventCallback,
};
pub use exports::*;
pub use exports_device::*;
pub use exports_engine::*;
pub use exports_session::*;
