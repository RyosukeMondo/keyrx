//! E2E Test Harness for Virtual Keyboard Testing.
//!
//! This module provides infrastructure for running end-to-end tests using
//! virtual input devices (uinput) instead of requiring physical hardware.
//!
//! # Components
//!
//! - [`error::E2EError`]: Error types for E2E test operations
//! - [`config::E2EConfig`]: Test configuration with helper constructors
//! - [`harness::E2EHarness`]: Complete test orchestration
//! - [`assertions::TestEvents`]: Test event builders and helpers
//! - [`assertions::TeardownResult`]: Test teardown status tracking
//!
//! # Example
//!
//! ```ignore
//! use keyrx_daemon::tests::e2e_harness::{E2EConfig, E2EHarness};
//!
//! // Create a simple remap configuration
//! let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
//!
//! // Setup the test environment (starts daemon as subprocess)
//! let harness = E2EHarness::setup(config)?;
//! ```

#![cfg(any(target_os = "linux", target_os = "windows"))]

pub mod assertions;
pub mod config;
pub mod error;
pub mod harness;

// Re-export main types
pub use assertions::{TestEvents, TeardownResult};
pub use config::E2EConfig;
pub use error::{E2EError, TestTimeoutPhase};
pub use harness::E2EHarness;
