//! Service layer for keyrx operations.
//!
//! This module provides the main service abstractions for managing devices,
//! profiles, and runtime configuration. Services act as the single source of
//! truth (SSOT) for their respective domains and handle both live state and
//! persistent storage.
//!
//! # Architecture
//!
//! Services are designed for dependency injection:
//! - Production code uses `with_defaults()` constructors
//! - Tests inject mock dependencies via `new()` constructors
//! - All external dependencies (storage, registry) are injected, not hardcoded
//!
//! # Available Services
//!
//! - [`DeviceService`] - Manages device discovery, binding, and configuration
//! - [`ProfileService`] - Manages keymaps, hardware profiles, and virtual layouts
//! - [`RuntimeService`] - Manages runtime configuration and profile slot assignments
//!
//! # Mock Support
//!
//! Mock implementations are available for testing when the `test-utils` feature is enabled:
//! - `MockDeviceService`
//! - `MockProfileService`
//! - `MockRuntimeService`
//!
//! # Example
//!
//! ```no_run
//! use keyrx_core::services::{RuntimeService, RuntimeServiceTrait};
//!
//! let service = RuntimeService::with_defaults();
//! let config = service.get_config().expect("Failed to load config");
//! ```

pub mod device;
pub mod profile;
pub mod runtime;
pub mod traits;

#[cfg(any(test, feature = "test-utils"))]
pub mod mocks;

#[cfg(any(test, feature = "test-utils"))]
pub use mocks::{MockDeviceService, MockProfileService, MockRuntimeService};

#[cfg(test)]
mod tests;

pub use device::{DeviceService, DeviceServiceError};
pub use profile::{ProfileService, ProfileServiceError};
pub use runtime::{RuntimeService, RuntimeServiceError};
pub use traits::{DeviceServiceTrait, ProfileServiceTrait, RuntimeServiceTrait};
