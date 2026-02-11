//! Integration tests module
//!
//! This module aggregates all integration tests for the keyrx_compiler.

#[path = "integration_test/compile_test.rs"]
mod compile_test;
#[path = "integration_test/load_test.rs"]
mod load_test;
#[path = "integration_test/workflow_test.rs"]
mod workflow_test;
