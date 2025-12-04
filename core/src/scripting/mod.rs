//! Rhai scripting integration.
//!
//! This module provides the scripting runtime for KeyRx:
//! - `runtime`: Core RhaiRuntime struct and ScriptRuntime implementation
//! - `context`: Injectable runtime context replacing global state
//! - `bindings`: Rhai function registrations (remap, layer, modifier, timing)
//! - `builtins`: Helper types and utility functions
//! - `docs`: API documentation types and generation
//! - `registry`: Remap registry for storing key mappings
//! - `helpers`: Key parsing and validation utilities
//! - `test_harness`: Test harness and context for Rhai script testing
//! - `test_primitives`: Test primitive implementations (simulate_*, assert_*)
//! - `test_discovery`: Test discovery and filtering for Rhai scripts
//! - `test_runner`: Test runner for Rhai scripts

mod bindings;
mod builtins;
pub mod cache;
pub mod context;
pub mod docs;
pub mod helpers;
mod pending_ops;
mod registry;
mod registry_sync;
mod row_col_resolver;
mod runtime;
pub mod sandbox;
pub mod test_discovery;
pub mod test_harness;
mod test_primitives;
pub mod test_runner;

// Re-export types needed by validation module
pub use builtins::{LayerMapAction, PendingOp, TimingUpdate};

pub use context::{
    clear_active_runtime, global_context, set_active_runtime, with_active_runtime, RuntimeContext,
};
pub use registry::{RemapRegistry, TapHoldBinding};
pub use row_col_resolver::{ResolverError, RowColResolver};
pub use runtime::RhaiRuntime;
pub use test_discovery::{discover_tests, filter_tests, matches_filter, DiscoveredTest};
pub use test_harness::{
    get_pending_inputs, get_test_context, record_input, record_output, reset_test_context,
    AssertionResult, TestContext, TestHarness,
};
pub use test_runner::{TestResult, TestRunner, TestSummary};
