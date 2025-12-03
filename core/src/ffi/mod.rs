//! FFI exports for Flutter integration.

mod callbacks;
mod exports;

pub use callbacks::{
    callback_registry, CallbackRegistry, DiscoveryEventCallback, IsolatedRegistry,
    StateEventCallback,
};
pub use exports::*;
