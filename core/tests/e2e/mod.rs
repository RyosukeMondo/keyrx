#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::print_stdout, clippy::print_stderr, clippy::field_reassign_with_default, clippy::useless_conversion, clippy::assertions_on_constants, clippy::manual_div_ceil, clippy::manual_strip, clippy::len_zero, clippy::redundant_closure, clippy::manual_range_contains, clippy::default_constructed_unit_structs, clippy::clone_on_copy, clippy::io_other_error, clippy::bool_assert_comparison, clippy::approx_constant, clippy::let_unit_value, clippy::while_let_on_iterator, clippy::await_holding_lock, clippy::unnecessary_cast, clippy::drop_non_drop, clippy::needless_range_loop, unused_imports, unused_variables, dead_code, unsafe_code, clippy::collapsible_if, clippy::bool_comparison, unexpected_cfgs)]
//! End-to-end tests for KeyRx.
//!
//! This module contains end-to-end tests that verify complete workflows
//! and user scenarios, simulating real-world usage patterns.
//!
//! ## Test Organization
//!
//! E2E tests should:
//! - Test complete user workflows from input to output
//! - Use minimal mocking (prefer real implementations)
//! - Verify cross-component integration
//! - Test critical user scenarios
//!
//! ## Running E2E Tests
//!
//! ```bash
//! # Run all e2e tests
//! cargo test --test '*' -- e2e
//!
//! # Run specific e2e test
//! cargo test --test e2e_workflow_name
//! ```
