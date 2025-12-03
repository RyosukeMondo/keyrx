//! FFI domain implementations.
//!
//! Each module in this directory implements the FfiExportable trait for a specific
//! domain of functionality (discovery, validation, engine, etc.).

pub mod discovery;

pub use discovery::DiscoveryFfi;
