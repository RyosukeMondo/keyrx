//! Dependency injection traits for testability.
//!
//! This module provides abstraction traits over external dependencies like
//! environment variables and filesystem operations, enabling full unit testing
//! without real filesystem access or environment manipulation.

pub mod env;
pub mod filesystem;

pub use env::{EnvProvider, MockEnvProvider, RealEnvProvider};
pub use filesystem::{FileSystem, MockFileSystem, RealFileSystem};
