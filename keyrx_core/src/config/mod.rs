//! Configuration types for keyboard remapping
//!
//! This module defines all configuration structures that are serialized to/from
//! `.krx` binary files using rkyv's zero-copy deserialization.
//!
//! # Security
//!
//! All types in this module implement the `CheckBytes` trait, which enables safe
//! deserialization from untrusted input sources (WASM, network, corrupted files).
//!
//! ## Safe Deserialization Guarantees
//!
//! When deserializing `.krx` files using `rkyv::check_archived_root`:
//!
//! - **No panics**: Malformed or adversarially crafted data will return errors,
//!   never cause panics or crashes
//! - **No undefined behavior**: All memory accesses are validated to be in-bounds
//!   and properly aligned
//! - **No invalid data**: Enum discriminants are validated, preventing invalid
//!   variants from being constructed
//! - **Recursive validation**: All nested structures (Vec, String, nested enums/structs)
//!   are recursively validated before access is allowed
//!
//! ## Performance Impact
//!
//! CheckBytes validation has minimal overhead:
//! - Performed once during deserialization (linear scan of data)
//! - Zero cost during runtime access to validated archives
//! - No impact on serialization performance
//!
//! ## Fuzzing
//!
//! The deserialization path is continuously fuzz-tested (see `fuzz/fuzz_targets/fuzz_deserialize.rs`)
//! to ensure it handles all possible byte sequences safely:
//!
//! ```bash
//! # Run fuzzer for 1 hour minimum before production deployment
//! cargo +nightly fuzz run fuzz_deserialize -- -max_total_time=3600
//! ```
//!
//! ## Example: Safe Deserialization
//!
//! ```rust,ignore
//! use keyrx_core::config::ConfigRoot;
//!
//! // Untrusted input (from WASM, network, user-provided file)
//! let untrusted_bytes: &[u8] = load_from_untrusted_source();
//!
//! // Safe validation - returns error on malformed data, never panics
//! match rkyv::check_archived_root::<ConfigRoot>(untrusted_bytes) {
//!     Ok(config) => {
//!         // Archive is valid and safe to use
//!         println!("Config version: {}", config.version.major);
//!     }
//!     Err(e) => {
//!         // Invalid data detected, handle gracefully
//!         eprintln!("Invalid config file: {}", e);
//!     }
//! }
//! ```

pub mod conditions;
pub mod constants;
pub mod keys;
pub mod mappings;
pub mod types;

// Re-export core types
pub use conditions::{Condition, ConditionItem};
pub use constants::{LOCK_COUNT, MAX_LOCK_ID, MAX_MODIFIER_ID, MODIFIER_COUNT};
pub use keys::KeyCode;
pub use mappings::{BaseKeyMapping, ConfigRoot, DeviceConfig, DeviceIdentifier, KeyMapping};
pub use types::{Metadata, Version};
