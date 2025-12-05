//! Resource enforcement utilities for the engine.

pub mod config;
pub mod enforcer;

pub use config::ResourceLimits;
pub use enforcer::{ExecutionGuard, ResourceEnforcer, ResourceLimitError, ResourceUsageSnapshot};
