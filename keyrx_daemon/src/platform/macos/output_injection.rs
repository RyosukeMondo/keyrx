//! macOS keyboard output injection using enigo.
//!
//! This module provides keyboard event injection using the enigo crate,
//! which uses CGEventPost for synthetic keyboard events.

/// macOS keyboard output injector.
///
/// Injects keyboard events using enigo::Enigo.
pub struct MacosOutputInjector {}

impl MacosOutputInjector {
    /// Creates a new output injector instance.
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for MacosOutputInjector {
    fn default() -> Self {
        Self::new()
    }
}
