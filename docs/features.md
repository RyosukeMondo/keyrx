# Feature Flags

This document describes all Cargo feature flags available in the KeyRx core library.

## Overview

KeyRx uses Cargo features to enable optional functionality and platform-specific code. This allows for:

- **Smaller binaries** - Only compile what you need
- **Faster builds** - Reduce compile time by excluding unused features
- **Platform flexibility** - Build for specific platforms without cross-compilation issues
- **Optional integrations** - Enable observability features only when needed

## Available Features

### Default Features

```toml
default = ["windows-driver", "linux-driver"]
```

By default, both platform drivers are enabled. This allows the crate to compile on any platform, though only the platform-specific driver will be functional at runtime.

### Platform Drivers

#### `windows-driver`

Enables Windows input driver support using the windows-rs crate.

- **Dependencies**: `windows` crate with minimal API features
- **Platform**: Windows only (automatically disabled on other platforms)
- **Use case**: Required for KeyRx to function on Windows systems
- **Code location**: `core/src/drivers/windows/`

**Example - Windows-only build:**
```bash
cargo build --no-default-features --features windows-driver
```

#### `linux-driver`

Enables Linux input driver support using evdev.

- **Dependencies**: `evdev`, `nix`, `signal-hook`
- **Platform**: Linux only (automatically disabled on other platforms)
- **Use case**: Required for KeyRx to function on Linux systems
- **Code location**: `core/src/drivers/linux/`

**Example - Linux-only build:**
```bash
cargo build --no-default-features --features linux-driver
```

### Observability

#### `otel-tracing`

Enables OpenTelemetry tracing integration for distributed tracing and observability.

- **Dependencies**: `opentelemetry`, `opentelemetry_sdk`, `opentelemetry-otlp`
- **Platform**: All platforms
- **Use case**: Production monitoring, performance analysis, debugging
- **Code location**: `core/src/engine/tracing/`

When enabled, KeyRx can export trace data to OpenTelemetry-compatible backends (Jaeger, Zipkin, etc.).

**Example - Enable tracing:**
```bash
cargo build --features otel-tracing
```

**Example - Enable tracing with Linux driver only:**
```bash
cargo build --no-default-features --features "linux-driver,otel-tracing"
```

## Common Build Scenarios

### Minimal Build (No Platform Drivers)

Useful for testing core logic without driver dependencies:

```bash
cargo build --no-default-features
```

### Single Platform Build

For production deployments, build only for your target platform:

**Linux:**
```bash
cargo build --release --no-default-features --features linux-driver
```

**Windows:**
```bash
cargo build --release --no-default-features --features windows-driver
```

### Development Build with Tracing

Enable all features for development and debugging:

```bash
cargo build --features otel-tracing
```

### All Features

Build with everything enabled:

```bash
cargo build --all-features
```

## Testing Feature Combinations

A comprehensive test script is available to verify all feature combinations build successfully:

```bash
./scripts/test_feature_combinations.sh
```

This script tests:
- Minimal build (no features)
- Default features
- Each platform driver individually
- OpenTelemetry tracing alone
- All combinations of drivers and tracing
- Full feature set

## Build Profiles

KeyRx defines several build profiles optimized for different use cases:

### `dev` (Development)

```toml
[profile.dev]
opt-level = 0
debug = true
incremental = true
```

- Fast compilation
- Full debug symbols
- Incremental compilation enabled

### `release` (Production)

```toml
[profile.release]
opt-level = "z"
lto = true
strip = true
codegen-units = 1
```

- Maximum size optimization
- Link-time optimization (LTO) enabled
- Debug symbols stripped
- Single codegen unit for best optimization

### `release-debug` (Profiling)

```toml
[profile.release-debug]
inherits = "release"
strip = false
debug = true
```

- Release-level optimizations
- Debug symbols retained
- Useful for profiling production builds

**Usage:**
```bash
cargo build --profile release-debug
```

## Platform-Specific Dependencies

KeyRx uses target-specific dependencies to ensure platform code only compiles when needed:

### Windows Dependencies

```toml
[target.'cfg(windows)'.dependencies]
windows = { version = "0.58", optional = true, features = [...] }
```

Activated by the `windows-driver` feature.

### Linux Dependencies

```toml
[target.'cfg(target_os = "linux")'.dependencies]
evdev = { version = "0.12", optional = true }
nix = { version = "0.29", optional = true, features = ["process", "signal"] }
signal-hook = { version = "0.3", optional = true }
```

Activated by the `linux-driver` feature.

## Configuration Guards

Code is conditionally compiled using feature gates:

```rust
#[cfg(feature = "windows-driver")]
pub mod windows;

#[cfg(feature = "linux-driver")]
pub mod linux;

#[cfg(feature = "otel-tracing")]
pub fn with_file_export(/* ... */) -> TracingResult<Self> {
    // OpenTelemetry tracing implementation
}
```

This ensures:
- Clean compilation on all platforms
- No dead code warnings
- Type-safe feature detection

## Best Practices

### 1. **Use Platform-Specific Builds in Production**

Always build with only the required platform driver:

```bash
# Good - Linux production
cargo build --release --no-default-features --features linux-driver

# Avoid - Includes unnecessary Windows dependencies
cargo build --release
```

### 2. **Enable Tracing for Observability**

In production environments where monitoring is needed:

```bash
cargo build --release --no-default-features --features "linux-driver,otel-tracing"
```

### 3. **Test with Minimal Features**

When writing platform-agnostic tests:

```bash
cargo test --no-default-features
```

### 4. **Use Workspace Dependencies**

All dependencies are managed at the workspace level in the root `Cargo.toml`, ensuring version consistency across the project.

## Troubleshooting

### Build Errors on Cross-Compilation

If you're cross-compiling and encounter driver-related errors:

```bash
# Build without default features, then add only your target platform
cargo build --target x86_64-unknown-linux-gnu --no-default-features --features linux-driver
```

### Missing Symbols with LTO

If you encounter missing symbols with LTO enabled, try building with debug symbols:

```bash
cargo build --profile release-debug
```

### Feature Compatibility

All feature combinations have been tested and verified to compile successfully. If you encounter issues, run:

```bash
./scripts/test_feature_combinations.sh
```

This will test all valid feature combinations and report any compilation errors.

## See Also

- [ARCHITECTURE.md](./ARCHITECTURE.md) - Overall system architecture
- [Cargo.toml](../Cargo.toml) - Workspace configuration
- [core/Cargo.toml](../core/Cargo.toml) - Core library configuration
