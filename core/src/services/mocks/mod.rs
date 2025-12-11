//! Mock service implementations for fast, isolated unit testing.
//!
//! This module provides mock implementations of all service traits, enabling:
//! - Pure in-memory testing without I/O
//! - Configurable success and error responses
//! - Call tracking for verification
//!
//! # Overview
//!
//! Three mock services are available:
//! - [`MockDeviceService`] - Mock for device operations
//! - [`MockProfileService`] - Mock for profile/layout/keymap management
//! - [`MockRuntimeService`] - Mock for runtime configuration
//!
//! # Usage Pattern
//!
//! All mocks follow a builder pattern for configuration:
//!
//! 1. Create with `new()`
//! 2. Configure data with `with_*()` methods
//! 3. Optionally configure errors with `with_*_error()` methods
//! 4. Wrap in `Arc` and inject into [`ApiContext`](crate::api::ApiContext)
//!
//! # Example: Basic Test Setup
//!
//! ```rust,ignore
//! use std::sync::Arc;
//! use keyrx_core::api::ApiContext;
//! use keyrx_core::services::{MockDeviceService, MockProfileService, MockRuntimeService};
//!
//! #[tokio::test]
//! async fn test_list_devices() {
//!     // Create mock with test data
//!     let mock_device = MockDeviceService::new()
//!         .with_devices(vec![test_device("1234:5678:serial")]);
//!
//!     // Inject into API context
//!     let api = ApiContext::new(
//!         Arc::new(mock_device),
//!         Arc::new(MockProfileService::new()),
//!         Arc::new(MockRuntimeService::new()),
//!     );
//!
//!     // Test the API
//!     let devices = api.list_devices().await.unwrap();
//!     assert_eq!(devices.len(), 1);
//! }
//! ```
//!
//! # Example: Testing Error Handling
//!
//! ```rust,ignore
//! #[tokio::test]
//! async fn test_device_error_handling() {
//!     // Configure mock to return an error
//!     let mock_device = MockDeviceService::new()
//!         .with_list_error(DeviceServiceError::Io(std::io::Error::other("connection failed")));
//!
//!     let api = ApiContext::new(
//!         Arc::new(mock_device),
//!         Arc::new(MockProfileService::new()),
//!         Arc::new(MockRuntimeService::new()),
//!     );
//!
//!     // Verify error is propagated
//!     let result = api.list_devices().await;
//!     assert!(result.is_err());
//! }
//! ```
//!
//! # Example: Verifying Method Calls
//!
//! ```rust,ignore
//! #[tokio::test]
//! async fn test_method_called() {
//!     let mock_device = MockDeviceService::new().with_devices(vec![test_device("key")]);
//!     let mock_device = Arc::new(mock_device);
//!
//!     let api = ApiContext::new(
//!         mock_device.clone(),
//!         Arc::new(MockProfileService::new()),
//!         Arc::new(MockRuntimeService::new()),
//!     );
//!
//!     // Call the API
//!     let _ = api.list_devices().await;
//!     let _ = api.list_devices().await;
//!
//!     // Verify call count
//!     assert_eq!(mock_device.get_call_count("list_devices"), 2);
//! }
//! ```

mod device;
mod profile;
mod runtime;

pub use device::MockDeviceService;
pub use profile::MockProfileService;
pub use runtime::MockRuntimeService;

#[cfg(test)]
mod tests;
