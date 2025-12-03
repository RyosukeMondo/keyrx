//! Backward-compatible FFI shims.
//!
//! This module provides deprecated wrapper functions that maintain the existing
//! FFI API while forwarding to the new unified architecture. This allows Flutter
//! code to migrate incrementally without breaking changes.
//!
//! All functions in this module are marked as `#[deprecated]` with migration
//! guidance pointing to the new recommended APIs.

pub mod discovery_compat;
pub mod engine_compat;

// Re-export for convenience
#[allow(deprecated)]
pub use discovery_compat::{
    keyrx_on_discovery_duplicate, keyrx_on_discovery_progress, keyrx_on_discovery_summary,
};

#[allow(deprecated)]
pub use engine_compat::keyrx_on_state;
