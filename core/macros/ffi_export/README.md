# FFI Export Macro

Procedural macro for deriving FFI contracts from Rust function signatures. This enables a **Rust-first, AI-native FFI workflow** where Rust code is the single source of truth.

## Features

- **Rust as SSOT**: Derive contracts from Rust code, eliminating manual JSON editing
- **Zero manual JSON**: Contracts are generated automatically at compile time
- **AI-friendly**: Clear, declarative annotations that AI agents can understand and generate
- **Type-safe**: Contract generation respects Rust's type system
- **Documentation integration**: Doc comments become contract descriptions

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ffi_export = { path = "../macros/ffi_export" }
```

## Basic Usage

```rust
use ffi_export::ffi_export;

/// Start the event processing loop.
///
/// This initializes the engine with the given profile.
#[ffi_export(domain = "engine", version = "1.0")]
pub fn start_loop(profile_id: &str) -> Result<EngineState, EngineError> {
    // Implementation
}
```

This generates a contract JSON entry:

```json
{
  "name": "start_loop",
  "description": "Start the event processing loop.\n\nThis initializes the engine with the given profile.",
  "parameters": [
    {
      "name": "profile_id",
      "type": "string",
      "description": "A string value",
      "required": true
    }
  ],
  "returns": {
    "type": "object",
    "description": "An object value"
  },
  "errors": [
    {
      "code": "INTERNAL_ERROR",
      "description": "Internal error occurred"
    }
  ]
}
```

## Attributes

### Required Attributes

- `domain` (required): FFI domain name (e.g., "engine", "config", "audio")

### Optional Attributes

- `version` (optional): Contract version, defaults to "1.0"
- `rename` (optional): Override function name in contract
- `skip` (optional): Skip contract generation for this function

## Supported Types

### Parameter Types

| Rust Type | Contract Type | Example |
|-----------|---------------|---------|
| `&str`, `String` | `"string"` | `profile_id: &str` |
| `u32`, `i32`, etc. | `"number"` | `count: u32` |
| `f32`, `f64` | `"number"` | `volume: f64` |
| `bool` | `"bool"` | `enabled: bool` |
| `Option<T>` | Type of T (marked optional) | `volume: Option<u32>` |
| `Vec<T>` | `"array"` | `items: Vec<String>` |
| Custom types | `"object"` | `config: DeviceConfig` |

### Return Types

| Rust Type | Contract Type | Notes |
|-----------|---------------|-------|
| `Result<T, E>` | Type of T | Required for FFI functions |
| `()` | `"null"` | No return value |
| Custom types | `"object"` | Must implement Serialize |

## Examples

### Optional Parameters

Parameters wrapped in `Option<T>` are marked as optional in the contract:

```rust
#[ffi_export(domain = "audio")]
pub fn set_volume(device_id: &str, volume: Option<u32>) -> Result<(), Error> {
    // volume is optional in the contract
}
```

### Custom Return Types

```rust
#[derive(Serialize)]
pub struct DeviceInfo {
    vendor_id: u16,
    product_id: u16,
    name: String,
}

#[ffi_export(domain = "device")]
pub fn get_device_info(device_id: &str) -> Result<DeviceInfo, Error> {
    // Returns custom type serialized as JSON object
}
```

### Renaming Functions

```rust
#[ffi_export(domain = "engine", rename = "engine_start")]
pub fn start(profile: &str) -> Result<State, Error> {
    // Contract will use "engine_start" instead of "start"
}
```

### Skipping Contract Generation

```rust
#[ffi_export(domain = "internal", skip)]
pub fn internal_helper(data: &str) -> Result<(), Error> {
    // No contract generated for this function
}
```

### Multiple Parameters

```rust
/// Configure device settings.
///
/// Sets multiple configuration parameters for the device.
#[ffi_export(domain = "device")]
pub fn configure(
    device_id: &str,
    brightness: u32,
    color: Option<String>,
    enabled: bool,
) -> Result<(), Error> {
    // Multiple parameters with mixed types
}
```

## AI Agent Integration

The macro is designed to work seamlessly with AI code generation:

### Good Patterns (AI-friendly)

```rust
// ✓ Clear domain and version
#[ffi_export(domain = "engine", version = "1.0")]
pub fn start_loop(profile_id: &str) -> Result<State, Error> { }

// ✓ Descriptive doc comments
/// Start the event processing loop.
///
/// This initializes the engine with the given profile and
/// begins processing events from input devices.
#[ffi_export(domain = "engine")]
pub fn start_loop(profile_id: &str) -> Result<State, Error> { }

// ✓ Explicit Option for optional parameters
#[ffi_export(domain = "config")]
pub fn get_value(key: &str, default: Option<String>) -> Result<String, Error> { }
```

### Bad Patterns (Avoid these)

```rust
// ✗ Missing domain attribute
#[ffi_export]
pub fn my_function() -> Result<(), Error> { }

// ✗ Non-Result return type for FFI
#[ffi_export(domain = "test")]
pub fn my_function() -> String { }

// ✗ No documentation
#[ffi_export(domain = "test")]
pub fn my_function() -> Result<(), Error> { }
```

## Contract Output

The macro generates a compile-time constant for each annotated function:

```rust
#[ffi_export(domain = "engine")]
pub fn start_loop(profile: &str) -> Result<State, Error> { }

// Expands to:
pub fn start_loop(profile: &str) -> Result<State, Error> { }

const __FFI_CONTRACT_START_LOOP: &str = r#"{
  "name": "start_loop",
  ...
}"#;
```

These constants can be collected by the codegen tool during the build process.

## Integration with Codegen Pipeline

The contract constants are collected and processed by the FFI code generation tool to:

1. Generate Dart bindings
2. Validate contract adherence
3. Detect breaking changes
4. Generate TypeScript definitions

## Testing

Run tests with:

```bash
cargo test -p ffi_export
```

## Error Handling

The macro will produce compile-time errors for:

- Missing required `domain` attribute
- Invalid attribute syntax
- Unsupported function signatures
- Invalid return types

Example error:

```rust
#[ffi_export] // Error: Missing required attribute: domain
pub fn my_function() -> Result<(), Error> { }
```

## Best Practices

1. **Always provide doc comments**: They become contract descriptions
2. **Use Result<T, E> return types**: Required for FFI safety
3. **Use Option<T> for optional parameters**: Makes intent explicit
4. **Keep functions focused**: One function, one responsibility
5. **Use descriptive names**: They appear in generated contracts
6. **Document error cases**: Helps FFI consumers handle errors

## License

MIT OR Apache-2.0
