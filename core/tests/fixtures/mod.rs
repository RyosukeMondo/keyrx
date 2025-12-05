//! Test fixtures for KeyRx tests.
//!
//! This module provides reusable test utilities, builders, and common test data
//! to reduce boilerplate in unit and integration tests.
//!
//! ## Available Fixtures
//!
//! - `operations` - Builder patterns for creating PendingOp test instances
//! - `scripts` - Common Rhai script snippets for testing
//! - `engine` - TestEngine wrapper for simplified engine testing
//!
//! ## Usage
//!
//! ```rust
//! use fixtures::operations::OperationBuilder;
//!
//! let op = OperationBuilder::new()
//!     .remap(KeyCode::A, KeyCode::B)
//!     .build();
//! ```

pub mod engine;
pub mod operations;
pub mod scripts;
