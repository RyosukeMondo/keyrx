//! Global engine instance management for FFI access.
//!
//! This module provides a thread-safe way to share the active `AdvancedEngine`
//! instance with FFI functions that need to inspect it (like transition logging).

use crate::engine::AdvancedEngine;
use crate::scripting::RhaiRuntime;
use std::sync::{Arc, Mutex, OnceLock, RwLock};

/// Concrete type of the engine used in FFI.
///
/// We pin the generic `S` to `Arc<Mutex<RhaiRuntime>>` as that's what `exports_engine.rs` uses.
pub type SharedEngine = Arc<Mutex<AdvancedEngine<Arc<Mutex<RhaiRuntime>>>>>;

fn engine_slot() -> &'static RwLock<Option<SharedEngine>> {
    static SLOT: OnceLock<RwLock<Option<SharedEngine>>> = OnceLock::new();
    SLOT.get_or_init(|| RwLock::new(None))
}

/// Set the global engine instance.
pub fn set_global_engine(engine: SharedEngine) {
    if let Ok(mut guard) = engine_slot().write() {
        *guard = Some(engine);
    }
}

/// Clear the global engine instance.
pub fn clear_global_engine() {
    if let Ok(mut guard) = engine_slot().write() {
        *guard = None;
    }
}

/// Execute a function with the global engine instance if available.
pub fn with_global_engine<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut AdvancedEngine<Arc<Mutex<RhaiRuntime>>>) -> R,
{
    if let Ok(guard) = engine_slot().read() {
        if let Some(engine_arc) = guard.as_ref() {
            if let Ok(mut engine) = engine_arc.lock() {
                return Some(f(&mut engine));
            }
        }
    }
    None
}
