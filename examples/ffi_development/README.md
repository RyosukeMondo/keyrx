# FFI Development Examples

This directory contains example projects demonstrating the AI-native FFI workflow.

## Examples

1. **minimal_ffi_function/** - Minimal example of adding an FFI function
2. **complex_ffi_types/** - FFI function with custom types and error handling
3. **streaming_ffi/** - FFI function with streaming/async patterns
4. **error_handling/** - Comprehensive error handling patterns for FFI

## Usage

Each example is self-contained and demonstrates a specific aspect of the FFI workflow:

- Contract-first development using `#[ffi_export]`
- Proper error handling with `Result<T, E>`
- Type mapping between Rust and Dart
- Testing strategies for FFI functions

## Running Examples

```bash
# Navigate to an example directory
cd examples/ffi_development/minimal_ffi_function

# Build the example
cargo build

# Run tests
cargo test
```

## Learning Path

1. Start with `minimal_ffi_function` to understand the basics
2. Move to `complex_ffi_types` to learn about custom types
3. Explore `error_handling` for robust FFI error patterns
4. Study `streaming_ffi` for advanced async patterns
