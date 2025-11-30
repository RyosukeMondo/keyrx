//! Decision primitives for timing-sensitive behaviors (tap-hold, combos).

pub mod combos;
pub mod pending;
pub mod timing;

#[allow(unused_imports)]
pub use combos::{ComboDef, ComboRegistry};
pub use pending::{DecisionQueue, DecisionResolution, PendingDecision, PendingDecisionState};
pub use timing::TimingConfig;
