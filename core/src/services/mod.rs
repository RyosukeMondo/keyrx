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
