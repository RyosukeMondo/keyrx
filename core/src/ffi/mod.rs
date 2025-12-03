//! FFI exports for Flutter integration.
//!
//! The exports are organized into domain-specific modules:
//! - `exports`: Core init/common functions
//! - `exports_device`: Device and key definitions
//! - `exports_discovery`: Device key discovery session management
//! - `exports_engine`: Engine control and state callbacks
//! - `exports_recording`: Session recording control
//! - `exports_script`: Script loading and validation
//! - `exports_session`: Session analysis, benchmarking, diagnostics
//! - `exports_testing`: Test discovery and execution

mod callbacks;
mod exports;
mod exports_device;
mod exports_discovery;
mod exports_engine;
mod exports_recording;
mod exports_script;
mod exports_session;
mod exports_testing;

pub use callbacks::{
    callback_registry, CallbackRegistry, DiscoveryEventCallback, IsolatedRegistry,
    StateEventCallback,
};
pub use exports::*;
pub use exports_device::*;
pub use exports_discovery::*;
pub use exports_engine::*;
pub use exports_recording::*;
pub use exports_script::*;
pub use exports_session::*;
pub use exports_testing::*;
