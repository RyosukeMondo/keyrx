//! Script sandbox for secure Rhai execution.
//!
//! This module provides a hardened sandbox for executing Rhai scripts with:
//! - Capability-based function access control
//! - Resource limits (CPU, memory, recursion)
//! - Input validation
//! - O(1) function lookup

pub mod budget;
pub mod capability;

pub use budget::{ResourceBudget, ResourceConfig, ResourceExhausted, ResourceUsage};
pub use capability::{ScriptCapability, ScriptMode};
