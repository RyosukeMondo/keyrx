# Migration Guide: Adopting Dependency Injection

This guide helps developers migrate existing code to use the new dependency injection (DI) pattern with `ApiContext`.

## Why Migrate?

The DI pattern provides significant benefits:

| Before (Global) | After (DI) |
|-----------------|-----------|
| Tests require real I/O | Tests use mocks (no I/O) |
| ~1ms/test with I/O overhead | 0.06ms/test (pure memory) |
| Cannot test error paths | Easy error path testing |
| Global state conflicts | Full test isolation |

## Quick Start

### Before (Global Functions)

```rust
// Old pattern - uses global singletons
use keyrx_core::api::{list_devices, get_device};

async fn my_function() -> Result<()> {
    let devices = list_devices().await?;
    let device = get_device("device-key").await?;
    Ok(())
}
```

### After (ApiContext)

```rust
// New pattern - uses injected dependencies
use std::sync::Arc;
use keyrx_core::api::ApiContext;

async fn my_function(api: &ApiContext) -> Result<()> {
    let devices = api.list_devices().await?;
    let device = api.get_device("device-key").await?;
    Ok(())
}
```

## Migration Patterns

### Pattern 1: CLI Commands

CLI commands should accept `ApiContext` as a parameter:

**Before:**
```rust
pub async fn run_list_command() -> Result<()> {
    let devices = keyrx_core::api::list_devices().await?;
    for device in devices {
        println!("{}", device.key);
    }
    Ok(())
}
```

**After:**
```rust
pub async fn run_list_command(api: &ApiContext) -> Result<()> {
    let devices = api.list_devices().await?;
    for device in devices {
        println!("{}", device.key);
    }
    Ok(())
}

// In main.rs or command handler:
let api = ApiContext::with_defaults();
run_list_command(&api).await?;
```

### Pattern 2: FFI Layer Integration

FFI domains can store `ApiContext` in their state:

**Before:**
```rust
pub struct DeviceRegistryDomain {
    // Uses global API functions internally
}

impl DeviceRegistryDomain {
    pub async fn list_devices(&self) -> Result<String> {
        let devices = keyrx_core::api::list_devices().await?;
        serde_json::to_string(&devices)
    }
}
```

**After:**
```rust
pub struct DeviceRegistryDomain {
    api: Arc<ApiContext>,
}

impl DeviceRegistryDomain {
    pub fn new(api: Arc<ApiContext>) -> Self {
        Self { api }
    }

    pub async fn list_devices(&self) -> Result<String> {
        let devices = self.api.list_devices().await?;
        serde_json::to_string(&devices)
    }
}
```

### Pattern 3: Testing with Mocks

Create tests using mock services:

```rust
use std::sync::Arc;
use keyrx_core::api::ApiContext;
use keyrx_core::services::{
    MockDeviceService, MockProfileService, MockRuntimeService
};

#[tokio::test]
async fn test_list_devices_returns_configured_data() {
    // Setup mock with test data
    let mock_device = MockDeviceService::new()
        .with_devices(vec![test_device("key-1")]);

    // Create ApiContext with mock
    let api = ApiContext::new(
        Arc::new(mock_device),
        Arc::new(MockProfileService::new()),
        Arc::new(MockRuntimeService::new()),
    );

    // Test the code
    let devices = api.list_devices().await.unwrap();
    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0].key, "key-1");
}

#[tokio::test]
async fn test_list_devices_handles_error() {
    // Setup mock to return error
    let mock_device = MockDeviceService::new()
        .with_list_error(DeviceServiceError::Io(
            std::io::Error::other("simulated failure")
        ));

    let api = ApiContext::new(
        Arc::new(mock_device),
        Arc::new(MockProfileService::new()),
        Arc::new(MockRuntimeService::new()),
    );

    // Test error handling
    let result = api.list_devices().await;
    assert!(result.is_err());
}
```

### Pattern 4: New Feature Development

For new features, always use DI from the start:

```rust
// Good: Accept ApiContext, easy to test
pub async fn new_feature(api: &ApiContext, param: &str) -> Result<()> {
    let devices = api.list_devices().await?;
    // ... feature logic
    Ok(())
}

// Bad: Uses global function, hard to test
pub async fn new_feature_bad(param: &str) -> Result<()> {
    let devices = keyrx_core::api::list_devices().await?;
    // ... feature logic
    Ok(())
}
```

## Mock Configuration Examples

### MockDeviceService

```rust
// Return specific devices
let mock = MockDeviceService::new()
    .with_devices(vec![device1, device2]);

// Simulate list error
let mock = MockDeviceService::new()
    .with_list_error(DeviceServiceError::Io(io::Error::other("fail")));

// Simulate get error
let mock = MockDeviceService::new()
    .with_get_error(DeviceServiceError::DeviceNotFound("key".into()));
```

### MockProfileService

```rust
// Pre-populate layouts
let mock = MockProfileService::new()
    .with_virtual_layouts(vec![layout1, layout2])
    .with_hardware_profiles(vec![profile1]);

// Simulate save error
let mock = MockProfileService::new()
    .with_save_layout_error("disk full");
```

### MockRuntimeService

```rust
// Pre-populate config
let mock = MockRuntimeService::new()
    .with_config(runtime_config);

// Simulate add_slot error
let mock = MockRuntimeService::new()
    .with_add_slot_error("slot limit reached");
```

## Verifying Method Calls

All mocks track method calls:

```rust
let mock_device = Arc::new(MockDeviceService::new().with_devices(vec![device]));
let api = ApiContext::new(mock_device.clone(), ...);

let _ = api.list_devices().await;
let _ = api.list_devices().await;
let _ = api.get_device("key").await;

assert_eq!(mock_device.get_call_count("list_devices"), 2);
assert_eq!(mock_device.get_call_count("get_device"), 1);
assert_eq!(mock_device.get_call_count("set_label"), 0);
```

## Troubleshooting

### "Cannot find MockDeviceService"

Ensure the `test-utils` feature is enabled:

```toml
# Cargo.toml
[dev-dependencies]
keyrx_core = { path = "../core", features = ["test-utils"] }
```

### "Type mismatch with Arc<dyn Trait>"

Ensure you wrap mocks in `Arc`:

```rust
// Wrong
let api = ApiContext::new(MockDeviceService::new(), ...);

// Correct
let api = ApiContext::new(Arc::new(MockDeviceService::new()), ...);
```

### "Tests still use real I/O"

Make sure you're:
1. Using `ApiContext::new()` not `ApiContext::with_defaults()`
2. Passing mock services, not real services
3. Not calling global functions directly

## Backward Compatibility

The global API functions still work for backward compatibility:

```rust
// These still work (use global ApiContext internally)
keyrx_core::api::list_devices().await?;
keyrx_core::api::get_device("key").await?;
```

However, new code should prefer `ApiContext` for testability.

## Summary

1. **Accept ApiContext** instead of using global functions
2. **Use mocks** in tests for fast, isolated testing
3. **Configure errors** to test error handling paths
4. **Track calls** to verify interactions
5. **Enable test-utils** feature for mock access

For questions, see the service trait documentation in `core/src/services/traits.rs`.
