//! FFI Testing Infrastructure
//!
//! Integration tests for the AI-native FFI workflow, including:
//! - Contract derivation from macros
//! - Contract validation
//! - Breaking change detection
//! - Module splitting
//! - Incremental generation caching
//! - Property tests for FFI boundary invariants

mod breaking_change_tests;
mod contract_derivation_tests;
mod contract_validation_tests;
mod property_tests;
mod tool_integration_tests;
