# FFI Contract Adherence Validation

This module provides comprehensive AST-based validation of FFI function signatures against their JSON contract definitions.

## Overview

The enhanced validation system uses the `syn` crate to parse Rust source files and extract `extern "C"` function signatures. It then compares these signatures against JSON contract definitions to ensure type-safe FFI boundaries.

### Architecture

```
┌─────────────────────┐     ┌──────────────────────┐
│  Contract Files     │     │   Rust Source Files  │
│  (*.ffi-contract.   │     │   (exports.rs, etc.) │
│   json)             │     │                      │
└─────────┬───────────┘     └──────────┬───────────┘
          │                            │
          │                            │
          ▼                            ▼
┌─────────────────────┐     ┌──────────────────────┐
│  ContractRegistry   │     │     AST Parser       │
│  (loads contracts)  │     │   (syn-based)        │
└─────────┬───────────┘     └──────────┬───────────┘
          │                            │
          │     ┌──────────────┐       │
          └────►│  Validator   │◄──────┘
                │              │
                └──────┬───────┘
                       │
                       ▼
                ┌──────────────┐
                │   Reporter   │
                │ (error fmt)  │
                └──────────────┘
```

## Modules

### `parser.rs`
Parses Rust source files using `syn` to extract FFI function signatures.

- **`ParsedFunction`**: Represents a parsed FFI function with name, params, return type, and location
- **`ParsedParam`**: Represents a parameter with name, type string, and pointer flags
- **`ParsedType`**: Enum for Unit, Pointer, or Primitive types
- **`parse_ffi_exports(path)`**: Main entry point to parse a Rust file

### `type_mapper.rs`
Maps contract type strings to expected Rust FFI types.

- **`RustFfiType`**: Enum representing expected Rust types (Unit, Bool, I32, ConstCharPtr, etc.)
- **`map_contract_to_rust(type_str)`**: Converts contract types to Rust FFI types
- **`validate_type_match(contract, rust)`**: Validates parsed types against contract expectations

### `validator.rs`
Core validation logic comparing contracts against implementations.

- **`ValidationError`**: Rich error enum with variants for all mismatch types
- **`ValidationReport`**: Batch validation results with pass/fail counts
- **`validate_function(contract, parsed)`**: Single function validation
- **`validate_all_functions(contracts, parsed)`**: Batch validation collecting all errors

### `reporter.rs`
Generates human-readable error reports.

- **`generate_full_report(report)`**: Creates formatted report with all errors and suggestions

## Type Mapping Rules

| Contract Type | Rust FFI Type | Notes |
|--------------|---------------|-------|
| `void` | `()` | Unit type |
| `bool` | `bool` | Boolean |
| `int`, `int32`, `i32` | `i32` | Signed 32-bit |
| `uint8`, `u8` | `u8` | Unsigned 8-bit |
| `uint32`, `u32` | `u32` | Unsigned 32-bit |
| `uint64`, `u64` | `u64` | Unsigned 64-bit |
| `float64`, `f64` | `f64` | 64-bit float |
| `string` | `*const c_char` or `*mut c_char` | C string pointer |
| `object` | `*const c_char` | JSON-serialized |
| `array` | `*const c_char` | JSON-serialized |

**Note**: String return types accept `*mut c_char` because owned strings returned from FFI must be freed by the caller.

## Running the Validation

```bash
# Run the contract adherence test
cargo test -p keyrx-core --test contract_adherence_test

# Run with verbose output
cargo test -p keyrx-core --test contract_adherence_test -- --nocapture
```

## Common Errors and Fixes

### MissingFunction

**Error**: Function defined in contract but not found in Rust source.

```
✗ keyrx_missing_fn
  Contract: engine.ffi-contract.json
  Fix: Implement the function 'keyrx_missing_fn' with #[no_mangle] pub extern "C" fn
```

**Fix**: Add the function implementation:
```rust
#[no_mangle]
pub unsafe extern "C" fn keyrx_missing_fn() -> i32 {
    // Implementation
    0
}
```

### UncontractedFunction

**Error**: FFI function exists in source but has no contract definition.

```
⚠ keyrx_orphan_fn
  Location: src/ffi/exports.rs:42
  Fix: Add a contract definition for 'keyrx_orphan_fn' or remove the function if unused
```

**Fix**: Either add a contract entry in the appropriate `.ffi-contract.json` file or remove the function if no longer needed.

### ParameterCountMismatch

**Error**: Different number of parameters between contract and implementation.

```
✗ keyrx_process (parameter count)
  Location: src/ffi/exports.rs:100
  Expected: 3 parameters
  Found:    1 parameters
  Fix: Add 2 missing parameter(s) to match the contract
```

**Fix**: Update the function signature to match the contract:
```rust
// Contract defines: parameters: [{"name": "a", "param_type": "string"}, ...]
#[no_mangle]
pub unsafe extern "C" fn keyrx_process(
    a: *const c_char,
    b: *const c_char,
    c: i32,
) -> i32 {
    // ...
}
```

### ParameterTypeMismatch

**Error**: Parameter type doesn't match contract specification.

```
✗ keyrx_configure (parameter type)
  Location: src/ffi/exports.rs:50
  Parameter: 'config' (index 0)
  Expected: *const c_char
  Found:    i32
  Fix: Change type of parameter 'config' to '*const c_char'
```

**Fix**: Change the parameter type:
```rust
// Contract specifies: {"name": "config", "param_type": "object"}
// object maps to *const c_char (JSON string)
fn keyrx_configure(config: *const c_char) -> i32 { ... }
```

### ReturnTypeMismatch

**Error**: Return type doesn't match contract specification.

```
✗ keyrx_get_value (return type)
  Location: src/ffi/exports.rs:75
  Expected: *const c_char
  Found:    ()
  Fix: Change return type to '*const c_char'
```

**Fix**: Update the return type:
```rust
// Contract specifies: returns: { "type": "string" }
fn keyrx_get_value() -> *mut c_char { ... }
```

## Contract Schema

Contracts are defined in JSON files with the `.ffi-contract.json` extension:

```json
{
    "$schema": "https://keyrx.dev/schemas/ffi-contract-v1.json",
    "version": "1.0.0",
    "domain": "engine",
    "description": "Core engine operations",
    "protocol_version": 1,
    "functions": [
        {
            "name": "start",
            "rust_name": "keyrx_engine_start",
            "description": "Start the engine",
            "parameters": [
                {
                    "name": "config",
                    "param_type": "object",
                    "description": "JSON configuration",
                    "required": true
                }
            ],
            "returns": {
                "type": "int32",
                "description": "0 on success, error code otherwise"
            },
            "errors": [],
            "events_emitted": []
        }
    ],
    "events": []
}
```

### Function Fields

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Logical function name |
| `rust_name` | No | Actual Rust function name (defaults to `name`) |
| `description` | Yes | Human-readable description |
| `parameters` | Yes | Array of parameter definitions |
| `returns` | Yes | Return type definition |
| `errors` | No | Possible error conditions |
| `events_emitted` | No | Events this function may emit |

## FFI Function Requirements

For a Rust function to be recognized as an FFI export, it must have:

1. **`#[no_mangle]`** attribute - prevents name mangling
2. **`pub`** visibility - exported from the crate
3. **`extern "C"`** ABI - C-compatible calling convention

```rust
#[no_mangle]
pub unsafe extern "C" fn keyrx_example(
    input: *const c_char,
) -> i32 {
    // Implementation
    0
}
```

## Error Handling Pattern

This codebase uses JSON return values for error handling via the `ffi_json` utilities rather than error out-parameters. Functions return structured JSON responses that include success/error status.

## Adding New FFI Functions

1. **Define the contract** in the appropriate `.ffi-contract.json` file
2. **Implement the function** in Rust with correct signature
3. **Run validation** to verify compliance:
   ```bash
   cargo test -p keyrx-core --test contract_adherence_test
   ```

## Integration with CI

The contract adherence test is part of the standard test suite and runs automatically in CI. Any contract violations will cause the build to fail with detailed error messages.
