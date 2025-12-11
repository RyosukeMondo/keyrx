#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::field_reassign_with_default,
    clippy::useless_conversion,
    clippy::assertions_on_constants,
    clippy::manual_div_ceil,
    clippy::manual_strip,
    clippy::len_zero,
    clippy::redundant_closure,
    clippy::manual_range_contains,
    clippy::default_constructed_unit_structs,
    clippy::clone_on_copy,
    clippy::io_other_error,
    clippy::bool_assert_comparison,
    clippy::approx_constant,
    clippy::let_unit_value,
    clippy::while_let_on_iterator,
    clippy::await_holding_lock,
    clippy::unnecessary_cast,
    clippy::drop_non_drop,
    clippy::needless_range_loop,
    unused_imports,
    unused_variables,
    dead_code,
    unsafe_code,
    clippy::collapsible_if,
    clippy::bool_comparison,
    unexpected_cfgs
)]
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
