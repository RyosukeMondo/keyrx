//! FFI exports for Flutter integration.
//!
//! The exports are organized into domain-specific modules:
//! - `exports`: Core init/common functions
//! - `exports_analysis`: Session listing, analysis, and replay
//! - `exports_device`: Device and key definitions
//! - `exports_diagnostics`: System diagnostics and benchmarking
//! - `exports_discovery`: Device key discovery session management
//! - `exports_engine`: Engine control and state callbacks
//! - `exports_errors`: Error code definitions and queries
//! - `exports_recording`: Session recording control
//! - `exports_script`: Script loading and validation
//! - `exports_testing`: Test discovery and execution
//! - `exports_validation`: Script validation and key suggestions
//!
//! ## New Architecture Components
//! - `error`: Standardized FFI error types and result handling
//! - `context`: Handle-based state management replacing global statics
//! - `events`: Unified event registry for all FFI callbacks

mod callbacks;
pub mod context;
pub mod error;
pub mod events;
mod exports;
mod exports_analysis;
mod exports_device;
mod exports_diagnostics;
mod exports_discovery;
mod exports_engine;
mod exports_errors;
mod exports_recording;
mod exports_script;
mod exports_testing;
mod exports_validation;

pub use callbacks::{
    callback_registry, CallbackRegistry, DiscoveryEventCallback, IsolatedRegistry,
    StateEventCallback,
};
pub use exports::*;
pub use exports_analysis::*;
pub use exports_device::*;
pub use exports_diagnostics::*;
pub use exports_discovery::*;
pub use exports_engine::*;
pub use exports_errors::*;
pub use exports_recording::*;
pub use exports_script::*;
pub use exports_testing::*;
pub use exports_validation::*;
