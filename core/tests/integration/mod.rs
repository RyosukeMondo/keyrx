//! Common utilities for integration tests.

// Re-export commonly used types for submodules
pub use keyrx_core::engine::{InputEvent, KeyCode};
pub use keyrx_core::mocks::MockInput;
pub use keyrx_core::traits::InputSource;

// Submodules
pub mod channel_tests;
pub mod drivers;
pub mod state_tests;
pub mod validation;
