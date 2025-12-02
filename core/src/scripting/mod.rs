//! Rhai scripting integration.
//!
//! This module provides the scripting runtime for KeyRx:
//! - `runtime`: Core RhaiRuntime struct and ScriptRuntime implementation
//! - `bindings`: Rhai function registrations (remap, layer, modifier, timing)
//! - `builtins`: Helper types and utility functions
//! - `registry`: Remap registry for storing key mappings
//! - `helpers`: Key parsing and validation utilities
//! - `test_harness`: Test harness and context for Rhai script testing
//! - `test_primitives`: Test primitive implementations (simulate_*, assert_*)
//! - `test_discovery`: Test discovery and filtering for Rhai scripts
//! - `test_runner`: Test runner for Rhai scripts

mod bindings;
mod builtins;
pub mod helpers;
mod registry;
mod runtime;
pub mod test_discovery;
pub mod test_harness;
mod test_primitives;
pub mod test_runner;

pub use registry::RemapRegistry;
pub use runtime::{clear_active_runtime, set_active_runtime, with_active_runtime, RhaiRuntime};
pub use test_discovery::{discover_tests, filter_tests, matches_filter, DiscoveredTest};
pub use test_harness::{
    get_pending_inputs, get_test_context, record_output, reset_test_context, AssertionResult,
    TestContext, TestHarness,
};
pub use test_runner::{TestResult, TestRunner, TestSummary};
