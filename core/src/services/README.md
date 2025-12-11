# Dependency Injection in KeyRx

This document explains the dependency injection (DI) pattern used in KeyRx services and how to use it effectively for both production code and testing.

## Overview

KeyRx uses constructor-based dependency injection to achieve:

- **Testability**: Services can be easily mocked for fast, isolated unit tests
- **Flexibility**: Different implementations can be swapped at runtime
- **Loose Coupling**: Services depend on abstractions (traits), not concrete implementations
- **Clear Contracts**: Traits define explicit APIs that implementations must fulfill

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        ApiContext                           │
│  ┌─────────────────┐ ┌──────────────────┐ ┌──────────────┐  │
│  │ DeviceService   │ │ ProfileService   │ │RuntimeService│  │
│  │     Trait       │ │     Trait        │ │    Trait     │  │
│  └────────┬────────┘ └────────┬─────────┘ └──────┬───────┘  │
└───────────┼───────────────────┼──────────────────┼──────────┘
            │                   │                  │
    ┌───────┴───────┐   ┌───────┴───────┐   ┌──────┴──────┐
    │               │   │               │   │             │
┌───▼───┐     ┌─────▼───┐ ┌─────▼───┐ ┌─────▼─┐  ┌────▼────┐
│ Real  │     │ Mock    │ │ Real    │ │ Mock  │  │ Real/   │
│Service│     │ Service │ │ Service │ │Service│  │ Mock    │
└───────┘     └─────────┘ └─────────┘ └───────┘  └─────────┘
```

### Key Components

1. **Service Traits** (`traits.rs`): Define contracts for each service
   - `DeviceServiceTrait`: Device management operations (async)
   - `ProfileServiceTrait`: Profile/keymap management (sync)
   - `RuntimeServiceTrait`: Runtime configuration (sync)

2. **Real Implementations**: Production services with actual I/O
   - `DeviceService`: Manages device bindings and registry
   - `ProfileService`: Manages profiles, keymaps, layouts
   - `RuntimeService`: Manages runtime slot configuration

3. **Mock Implementations** (`mocks.rs`): Test doubles for isolated testing
   - `MockDeviceService`: In-memory device mock with call tracking
   - `MockProfileService`: In-memory profile mock with call tracking
   - `MockRuntimeService`: In-memory runtime mock with call tracking

4. **ApiContext** (`api.rs`): Orchestration layer that accepts injected services

## Production Usage

### Using Default Services

For production code, use `ApiContext::with_defaults()`:

```rust
use keyrx_core::api::ApiContext;

// Create API with production services
let api = ApiContext::with_defaults();

// Use the API
let devices = api.list_devices().await?;
let profiles = api.list_hardware_profiles()?;
```

### Using Standalone Functions (Legacy)

For backward compatibility, standalone functions delegate to a global `ApiContext`:

```rust
use keyrx_core::api;

// These use a global ApiContext internally
let devices = api::list_devices().await?;
let profiles = api::list_hardware_profiles()?;
```

### Injecting Custom Services

For custom configurations, inject services directly:

```rust
use std::sync::Arc;
use keyrx_core::api::ApiContext;
use keyrx_core::services::{DeviceService, ProfileService, RuntimeService};

let device_service = Arc::new(DeviceService::new(
    Some(custom_registry),
    custom_bindings,
));
let profile_service = Arc::new(ProfileService::new(custom_config_manager));
let runtime_service = Arc::new(RuntimeService::new(custom_config));

let api = ApiContext::new(device_service, profile_service, runtime_service);
```

## Testing with Mocks

### Basic Mock Usage

```rust
use std::sync::Arc;
use keyrx_core::api::ApiContext;
use keyrx_core::services::{MockDeviceService, MockProfileService, MockRuntimeService};

#[tokio::test]
async fn test_list_devices() {
    // Configure mock with test data
    let mock_device = MockDeviceService::new()
        .with_devices(vec![test_device("device-1")]);

    // Create API with mocked services
    let api = ApiContext::new(
        Arc::new(mock_device),
        Arc::new(MockProfileService::new()),
        Arc::new(MockRuntimeService::new()),
    );

    // Test the API
    let devices = api.list_devices().await.unwrap();
    assert_eq!(devices.len(), 1);
}
```

### Configuring Error Responses

```rust
use keyrx_core::services::device::DeviceServiceError;

#[tokio::test]
async fn test_handles_io_error() {
    let mock = MockDeviceService::new()
        .with_list_error(DeviceServiceError::Io(
            std::io::Error::other("disk failure")
        ));

    let api = ApiContext::new(
        Arc::new(mock),
        Arc::new(MockProfileService::new()),
        Arc::new(MockRuntimeService::new()),
    );

    let result = api.list_devices().await;
    assert!(result.is_err());
}
```

### Call Tracking

Verify that methods were called the expected number of times:

```rust
#[tokio::test]
async fn test_call_tracking() {
    let mock = Arc::new(MockDeviceService::new().with_devices(vec![test_device("key")]));
    let api = ApiContext::new(
        mock.clone(),
        Arc::new(MockProfileService::new()),
        Arc::new(MockRuntimeService::new()),
    );

    let _ = api.list_devices().await;
    let _ = api.list_devices().await;
    let _ = api.get_device("key".to_string()).await;

    // Verify call counts
    assert_eq!(mock.get_call_count("list_devices"), 2);
    assert_eq!(mock.get_call_count("get_device"), 1);
    assert_eq!(mock.get_call_count("set_remap_enabled"), 0);
}
```

### Profile and Runtime Mocking

```rust
use keyrx_core::config::models::{VirtualLayout, RuntimeConfig};

#[test]
fn test_profile_operations() {
    let mock = MockProfileService::new()
        .with_virtual_layouts(vec![test_layout("layout-1")]);

    let api = ApiContext::new(
        Arc::new(MockDeviceService::new()),
        Arc::new(mock),
        Arc::new(MockRuntimeService::new()),
    );

    let layouts = api.list_virtual_layouts().unwrap();
    assert_eq!(layouts.len(), 1);
}

#[test]
fn test_runtime_slot_operations() {
    let mock = MockRuntimeService::new();

    let api = ApiContext::new(
        Arc::new(MockDeviceService::new()),
        Arc::new(MockProfileService::new()),
        Arc::new(mock),
    );

    let config = api.get_runtime_config().unwrap();
    assert!(config.devices.is_empty());
}
```

## Migration Guide

### Updating Existing Code

**Before (using global state):**
```rust
// Service created with hardcoded defaults
let service = DeviceService::new(None);
```

**After (using dependency injection):**
```rust
// For production: use convenience constructor
let service = DeviceService::with_defaults(None);

// For tests: inject dependencies
let service = DeviceService::new(None, test_bindings);
```

### Updating Tests

**Before (slow, uses real I/O):**
```rust
#[tokio::test]
async fn test_device_list() {
    let devices = api::list_devices().await.unwrap();
    // Test with real filesystem - slow, unpredictable
}
```

**After (fast, isolated):**
```rust
#[tokio::test]
async fn test_device_list() {
    let mock = MockDeviceService::new()
        .with_devices(vec![expected_device]);
    let api = ApiContext::new(Arc::new(mock), ...);

    let devices = api.list_devices().await.unwrap();
    // Test with mock - fast, deterministic
}
```

## Best Practices

### Do

- Use `ApiContext::with_defaults()` for production code
- Use `ApiContext::new()` with mocks for unit tests
- Configure mocks with specific test data for each test
- Use call tracking to verify expected interactions
- Keep unit tests pure (no I/O) and fast (<1ms each)
- Use integration tests with real services for E2E scenarios

### Don't

- Don't access global services directly in new code
- Don't create services with `new()` in tests without injecting dependencies
- Don't share mock state between tests
- Don't rely on filesystem state in unit tests
- Don't skip error case testing - mocks make it easy

## Service Constructors

Each service provides two constructors:

| Service | DI Constructor | Convenience Constructor |
|---------|---------------|------------------------|
| `DeviceService` | `new(registry, bindings)` | `with_defaults(registry)` |
| `ProfileService` | `new(config_manager)` | `with_defaults()` |
| `RuntimeService` | `new(config)` | `with_defaults()` |

The `with_defaults()` constructors create production instances with default dependencies.
The `new()` constructors accept injected dependencies for testing or custom configurations.

## File Structure

```
services/
├── mod.rs          # Module exports
├── traits.rs       # Service trait definitions
├── mocks.rs        # Mock implementations (test only)
├── device.rs       # DeviceService implementation
├── profile.rs      # ProfileService implementation
├── runtime.rs      # RuntimeService implementation
└── README.md       # This documentation
```

## Further Reading

- Trait definitions: `core/src/services/traits.rs`
- Mock implementations: `core/src/services/mocks.rs`
- API context: `core/src/api.rs`
- Unit test examples: `core/tests/unit/api/`
