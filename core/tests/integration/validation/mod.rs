//! Validation integration tests.
//!
//! Organized by test category:
//! - basic_validation_tests: Core validation engine functionality
//! - safety_tests: Safety warnings and strict mode
//! - coverage_tests: Coverage tracking and reporting
//! - edge_case_tests: Parse errors and real-world scripts
//! - cli_integration_tests: CLI command integration
//! - json_output_tests: JSON serialization
//! - ffi_integration_tests: FFI bindings

mod basic_validation_tests;
mod cli_integration_tests;
mod coverage_tests;
mod edge_case_tests;
mod ffi_integration_tests;
mod json_output_tests;
mod safety_tests;
