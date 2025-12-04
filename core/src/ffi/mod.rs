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
//! - `traits`: FFI exportable trait definitions for domain modules
//! - `domains`: New domain implementations using FfiExportable trait
//! - `compat`: Backward-compatible shims for incremental migration

mod callbacks;
pub mod compat;
pub mod context;
pub mod domains;
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
pub mod traits;

#[cfg(test)]
mod tests;

pub use callbacks::{
    callback_registry, CallbackRegistry, DiscoveryEventCallback, IsolatedRegistry,
    StateEventCallback,
};
pub use exports::*;
pub use exports_analysis::*;
pub use exports_device::*;
pub use exports_diagnostics::*;
// Note: exports_discovery callback functions moved to compat module
pub use exports_engine::*;
pub use exports_errors::*;
pub use exports_recording::*;
pub use exports_script::*;
pub use exports_testing::*;
pub use exports_validation::*;
pub use traits::{FfiDomain, FfiExportable};

// Re-export new domain implementations
pub use domains::{DiscoveryFfi, ValidationFfi};

// Re-export backward-compatible shims (deprecated)
#[allow(deprecated)]
pub use compat::{
    keyrx_on_discovery_duplicate, keyrx_on_discovery_progress, keyrx_on_discovery_summary,
};
