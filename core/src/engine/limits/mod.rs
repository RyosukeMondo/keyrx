//! Resource enforcement utilities for the engine.

pub mod enforcer;

pub use enforcer::{ExecutionGuard, ResourceEnforcer, ResourceLimitError, ResourceUsageSnapshot};
