# Build Integration Guide

This document explains how FFI binding generation is integrated into the KeyRX build pipeline.

## Overview

The FFI binding generation system automatically generates Dart FFI bindings from Rust exports and ensures they stay synchronized throughout the development and build process.

## Components

### 1. Binding Generator (`scripts/generate_dart_bindings.py`)

Python script that:
- Scans Rust FFI source files for `#[no_mangle] pub extern "C"` functions
- Parses function signatures and documentation
- Generates type-safe Dart FFI bindings
- Outputs to `ui/lib/ffi/generated/bindings_generated.dart`

### 2. Build Script (`core/build.rs`)

Rust build script that:
- Tracks FFI source files for changes
- Triggers rebuild when FFI exports are modified
- Integrates with Cargo's build system

### 3. Verification Script (`scripts/verify_bindings.py`)

Python script that:
- Compares existing bindings with freshly generated ones
- Detects binding drift
- Used in CI to ensure bindings are committed and up-to-date

### 4. Build Automation (`justfile`)

Just recipes for common tasks:
- `just gen-bindings` - Generate FFI bindings
- `just verify-bindings` - Verify bindings are in sync
- `just build` - Full build including binding generation
- `just ci-check` - CI-specific checks including binding verification

### 5. Comprehensive Build Script (`scripts/build.sh`)

Bash script orchestrating the complete build:
1. Generate error documentation
2. Generate Dart FFI bindings
3. Verify bindings are in sync
4. Build Rust core
5. Build Flutter UI

## Usage

### Development Workflow

#### After Modifying FFI Exports

```bash
# Regenerate bindings
just gen-bindings

# Or as part of a full build
just build
```

#### Before Committing

```bash
# Run all checks (including binding verification)
just check

# Or just verify bindings
just verify-bindings
```

### CI/CD Integration

The build integration ensures CI catches binding drift:

```bash
# CI command
just ci-check
```

This will:
1. Check Rust formatting
2. Run Clippy
3. Run tests
4. Verify FFI bindings are in sync (FAILS if out of sync)

### Build Modes

#### Debug Build

```bash
# Quick development build
just build

# Or manually
./scripts/build.sh
```

#### Release Build

```bash
# Full release build
just build-full

# Or manually
./scripts/build.sh --release
```

#### Rust-Only Build

```bash
./scripts/build.sh --rust-only --release
```

#### Flutter-Only Build

```bash
./scripts/build.sh --flutter-only
```

## Integration Points

### Cargo Build System

The `core/build.rs` script automatically tracks FFI source files:

```rust
// Triggers rebuild when any FFI domain file changes
println!("cargo:rerun-if-changed=src/ffi/domains/discovery.rs");
```

This ensures that changes to FFI exports trigger:
1. Rust recompilation
2. Awareness that bindings may need regeneration

### Just Build System

The justfile integrates binding generation into standard build tasks:

```just
# Build recipe includes binding generation
build: docs-errors gen-bindings
    cd core && cargo build --release
```

### CI Pipeline

Example GitHub Actions integration (see `.github/workflows/ci-example.yml`):

```yaml
- name: Verify FFI bindings are in sync
  run: python3 scripts/verify_bindings.py

- name: Generate FFI bindings
  run: just gen-bindings
```

## Automatic Verification

### Pre-Commit Hook (Optional)

To automatically verify bindings before committing:

```bash
# Add to .git/hooks/pre-commit
#!/bin/bash
just verify-bindings || {
    echo "ERROR: FFI bindings are out of sync!"
    echo "Run: just gen-bindings"
    exit 1
}
```

### Flutter Pre-Build (Optional)

To verify bindings before Flutter builds:

```bash
# Run before Flutter build
./scripts/flutter_prebuild.sh

# Or using Dart
cd ui
dart tool/verify_bindings.dart
```

## Troubleshooting

### Binding Verification Fails

If `just verify-bindings` fails:

```bash
# Regenerate bindings
just gen-bindings

# Verify they're now in sync
just verify-bindings

# Commit the updated bindings
git add ui/lib/ffi/generated/bindings_generated.dart
git commit -m "chore: update FFI bindings"
```

### Build Fails Due to Bindings

If the build fails with binding errors:

1. Check if FFI exports were modified
2. Regenerate bindings: `just gen-bindings`
3. Verify sync: `just verify-bindings`
4. Commit updated bindings

### CI Fails on Binding Check

If CI fails with "bindings out of sync":

1. Pull latest changes
2. Run `just gen-bindings` locally
3. Commit and push the updated bindings
4. CI will pass on next run

## Best Practices

### 1. Always Run Checks Before Committing

```bash
just check
```

This ensures:
- Code is formatted
- Clippy passes
- Tests pass
- Bindings are in sync

### 2. Generate Bindings After FFI Changes

Whenever you modify FFI exports:

```bash
# Modify Rust FFI code
vim core/src/ffi/domains/my_domain.rs

# Regenerate bindings
just gen-bindings

# Stage both Rust and Dart changes
git add core/src/ffi/domains/my_domain.rs
git add ui/lib/ffi/generated/bindings_generated.dart
```

### 3. Use Build Script for Complex Builds

For release builds or CI:

```bash
./scripts/build.sh --release
```

This ensures all steps run in correct order with proper error handling.

### 4. Keep Bindings in Version Control

Always commit generated bindings:
- Makes code review easier
- Allows CI to verify they're up to date
- Prevents build-time generation issues

## Performance Considerations

### Build Time Impact

- **Binding Generation**: ~1-2 seconds
- **Verification**: ~0.5-1 seconds
- **Minimal overhead** compared to Rust compilation

### Caching

The build system uses Cargo's dependency tracking:
- Only regenerates when FFI files change
- CI can cache Cargo build artifacts
- Verification is fast (no compilation needed)

## Future Enhancements

Potential improvements to the build integration:

1. **Incremental Generation**: Only regenerate changed bindings
2. **C Header Generation**: Generate C headers alongside Dart bindings
3. **TypeScript Bindings**: Support for WebAssembly FFI
4. **Build Server**: Watch mode for continuous regeneration
5. **IDE Integration**: VS Code task for binding generation

## Related Documentation

- [FFI Architecture](./ffi-architecture.md) - Overall FFI design
- [Dart Bindings](../ui/lib/ffi/generated/README.md) - Generated bindings documentation
- [Development Guide](./development.md) - General development workflow
