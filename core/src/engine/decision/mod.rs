//! Decision primitives for timing-sensitive behaviors (tap-hold, combos).

pub mod pending;
pub mod timing;

pub use pending::{DecisionQueue, DecisionResolution, PendingDecision};
pub use timing::TimingConfig;
