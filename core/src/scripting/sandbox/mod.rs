//! Script sandbox for secure Rhai execution.
//!
//! This module provides a hardened sandbox for executing Rhai scripts with:
//! - Capability-based function access control
//! - Resource limits (CPU, memory, recursion)
//! - Input validation
//! - O(1) function lookup

pub mod capability;

pub use capability::{ScriptCapability, ScriptMode};
