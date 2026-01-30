//! Version information for KeyRx daemon
//!
//! This module provides version constants that are automatically
//! generated from Cargo.toml at build time.

/// KeyRx version (from Cargo.toml)
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Build date (set at compile time)
pub const BUILD_DATE: &str = env!("BUILD_DATE");

/// Git commit hash (if available)
pub const GIT_HASH: &str = env!("GIT_HASH");

/// Full version string with build info
pub fn full_version() -> String {
    format!(
        "KeyRx v{}\nBuild: {}\nCommit: {}",
        VERSION, BUILD_DATE, GIT_HASH
    )
}

/// Short version string
pub fn short_version() -> String {
    format!("v{}", VERSION)
}
