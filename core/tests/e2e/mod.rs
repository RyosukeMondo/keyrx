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
