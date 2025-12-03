//! FFI domain implementations.
//!
//! Each module in this directory implements the FfiExportable trait for a specific
//! domain of functionality (discovery, validation, engine, etc.).

pub mod discovery;
pub mod engine;
pub mod validation;

pub use discovery::DiscoveryFfi;
pub use engine::EngineFfi;
pub use validation::ValidationFfi;
