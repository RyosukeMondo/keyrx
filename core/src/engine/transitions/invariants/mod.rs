//! Core state invariants for validation.
//!
//! This module provides concrete implementations of state invariants that
//! enforce critical rules about engine state. Each invariant checks a specific
//! rule that must always hold true.
//!
//! # Available Invariants
//!
//! - [`NoOrphanedModifiers`]: Ensures modifiers are only active when their
//!   triggering key is pressed
//! - [`LayerStackNotEmpty`]: Ensures the layer stack always has at least the
//!   base layer
//! - [`PendingQueueBounds`]: Ensures the pending decision queue respects size
//!   limits
//! - [`KeyTimestampsMonotonic`]: Ensures key press timestamps increase
//!   monotonically per key

mod key_timestamps;
mod layer_stack;
mod no_orphaned_modifiers;
mod pending_queue;

pub use key_timestamps::KeyTimestampsMonotonic;
pub use layer_stack::LayerStackNotEmpty;
pub use no_orphaned_modifiers::NoOrphanedModifiers;
pub use pending_queue::PendingQueueBounds;
