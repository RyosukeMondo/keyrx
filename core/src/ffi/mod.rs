#![allow(unsafe_code)]
//! FFI exports for Flutter integration.
//!
//! ## Architecture Components
//! - `error`: Standardized FFI error types and result handling
//! - `context`: Handle-based state management replacing global statics
//! - `events`: Unified event registry for all FFI callbacks
//! - `traits`: FFI exportable trait definitions for domain modules
//! - `marshal`: Unified marshaling layer for FFI data transfer
//! - `domains`: Domain implementations using FfiExportable trait
//! - `exports`: Core init/common functions
//! - `contract`: Contract schema for FFI functions (runtime introspection)

pub mod context;
pub mod contract;
pub mod domains;
pub mod error;
pub mod events;
mod exports;
mod exports_compat;
mod exports_engine;
mod exports_metrics;
pub mod introspection;

mod exports_runtime;
mod exports_telemetry;
mod exports_transition_log;
pub mod logging;
pub mod marshal;
pub mod runtime;
pub mod traits;

#[cfg(test)]
mod tests;

pub use exports::*;
pub use exports_compat::*;
pub use exports_engine::*;
pub use exports_metrics::*;

pub use exports_runtime::*;
pub use exports_telemetry::*;
pub use exports_transition_log::*;
pub use traits::{FfiDomain, FfiExportable};

// Re-export domain-specific public functions
#[allow(deprecated)]
pub use domains::engine::publish_state_snapshot_legacy;
pub use domains::engine::{publish_state_change, publish_state_snapshot};
