use core::fmt;
// CheckBytes is used by #[archive(check_bytes)] derive macro
#[allow(unused_imports)]
use rkyv::{Archive, CheckBytes, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize, Serialize};

/// Version information for binary compatibility checking
///
/// Uses semantic versioning with major.minor.patch format.
/// All fields are u8 to keep the struct compact.
#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Debug,
)]
#[archive(check_bytes)]
#[repr(C)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl Version {
    /// Returns the current version (1.0.0)
    pub const fn current() -> Self {
        Self {
            major: 1,
            minor: 0,
            patch: 0,
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Metadata about the compiled configuration
///
/// Contains information about when and how the configuration was compiled.
/// This is useful for debugging and verification purposes.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, Serialize, Deserialize, Clone, PartialEq, Eq, Debug,
)]
#[archive(check_bytes)]
#[repr(C)]
pub struct Metadata {
    /// Unix timestamp (seconds since epoch) when the config was compiled
    pub compilation_timestamp: u64,
    /// Version of the compiler that created this file
    pub compiler_version: alloc::string::String,
    /// SHA256 hash of the source Rhai script(s)
    pub source_hash: alloc::string::String,
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate alloc;
    use alloc::string::ToString;

    #[test]
    fn test_version_current() {
        let version = Version::current();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 0);
        assert_eq!(version.patch, 0);
    }

    #[test]
    fn test_version_display() {
        let version = Version::current();
        assert_eq!(version.to_string(), "1.0.0");
    }
}
