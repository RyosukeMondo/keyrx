//! Script sandbox for secure Rhai execution.
//!
//! This module provides a hardened sandbox for executing Rhai scripts with:
//! - Capability-based function access control
//! - Resource limits (CPU, memory, recursion)
//! - Input validation
//! - O(1) function lookup

pub mod budget;
pub mod capability;
pub mod function_capabilities;
pub mod registry;
pub mod validation;
pub mod validators;

pub use budget::{ResourceBudget, ResourceConfig, ResourceExhausted, ResourceUsage};
pub use capability::{ScriptCapability, ScriptMode};
pub use function_capabilities::build_function_registry;
pub use registry::{CapabilityRegistry, FunctionCapability};
pub use validation::{InputValidator, ValidationError, ValidationResult};
