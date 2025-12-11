# FFI Contract Schema Specification

## Overview

This document defines the contract schema format for KeyRx FFI functions. The schema serves as a single source of truth for both Rust and Dart FFI code generation and validation.

## Contract File Format

Contract files are JSON documents with `.ffi-contract.json` extension, one per domain.

**Location:** `core/src/ffi/contracts/{domain}.ffi-contract.json`

## Schema Structure

```json
{
  "$schema": "https://keyrx.dev/schemas/ffi-contract-v1.json",
  "version": "1.0.0",
  "domain": "discovery",
  "description": "Device discovery operations",
  "protocol_version": 1,
  "functions": [...],
  "types": {...},
  "events": [...]
}
```

### Top-Level Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `$schema` | string | Yes | Schema version URL |
| `version` | string | Yes | Contract version (semver) |
| `domain` | string | Yes | Domain name (must match Rust domain) |
| `description` | string | Yes | Human-readable domain description |
| `protocol_version` | integer | Yes | FFI protocol version number |
| `functions` | array | Yes | List of FFI functions |
| `types` | object | No | Custom type definitions |
| `events` | array | No | Events emitted by this domain |

## Function Definition

```json
{
  "name": "start_discovery",
  "description": "Start keyboard discovery process",
  "rust_name": "keyrx_discovery_start_discovery",
  "parameters": [
    {
      "name": "device_id",
      "type": "string",
      "description": "USB device ID",
      "required": true,
      "constraints": {
        "min_length": 1,
        "max_length": 256,
        "pattern": "^[0-9a-f]{4}:[0-9a-f]{4}$"
      }
    },
    {
      "name": "rows",
      "type": "uint8",
      "description": "Number of keyboard rows",
      "required": true,
      "constraints": {
        "min": 1,
        "max": 32
      }
    }
  ],
  "returns": {
    "type": "object",
    "description": "Discovery result summary",
    "properties": {
      "device_count": {
        "type": "uint32",
        "description": "Number of devices discovered"
      },
      "duration_ms": {
        "type": "uint64",
        "description": "Discovery duration in milliseconds"
      }
    }
  },
  "errors": [
    {
      "code": "INVALID_INPUT",
      "description": "Invalid device ID format"
    },
    {
      "code": "DEVICE_NOT_FOUND",
      "description": "USB device not found"
    },
    {
      "code": "DISCOVERY_FAILED",
      "description": "Discovery process failed"
    }
  ],
  "events_emitted": ["DiscoveryProgress", "DiscoverySummary"],
  "example": {
    "input": {
      "device_id": "1234:5678",
      "rows": 6
    },
    "output": {
      "device_count": 3,
      "duration_ms": 1234
    }
  }
}
```

### Function Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Function name (without `keyrx_{domain}_` prefix) |
| `description` | string | Yes | Human-readable description |
| `rust_name` | string | No | Full Rust function name (auto-generated if omitted) |
| `parameters` | array | Yes | Function parameters (empty array if none) |
| `returns` | object | Yes | Return type definition |
| `errors` | array | Yes | Possible error codes |
| `events_emitted` | array | No | Events this function may emit |
| `example` | object | No | Example input/output for documentation |
| `deprecated` | boolean | No | Mark function as deprecated |
| `since_version` | string | No | Contract version when added |

## Type System

### Primitive Types

| Type | Rust | Dart FFI | Constraints |
|------|------|----------|-------------|
| `bool` | `bool` | `Bool` | - |
| `int8` | `i8` | `Int8` | `min`, `max` |
| `uint8` | `u8` | `Uint8` | `min`, `max` |
| `int16` | `i16` | `Int16` | `min`, `max` |
| `uint16` | `u16` | `Uint16` | `min`, `max` |
| `int32` | `i32` | `Int32` | `min`, `max` |
| `uint32` | `u32` | `Uint32` | `min`, `max` |
| `int64` | `i64` | `Int64` | `min`, `max` |
| `uint64` | `u64` | `Uint64` | `min`, `max` |
| `float32` | `f32` | `Float` | `min`, `max` |
| `float64` | `f64` | `Double` | `min`, `max` |
| `string` | `*const c_char` | `Pointer<Char>` | `min_length`, `max_length`, `pattern` |
| `void` | `()` | `Void` | - |

### Complex Types

```json
{
  "type": "object",
  "properties": {
    "field_name": {
      "type": "uint32",
      "description": "Field description"
    }
  }
}
```

```json
{
  "type": "array",
  "items": {
    "type": "string"
  }
}
```

```json
{
  "type": "enum",
  "values": ["Active", "Inactive", "Paused"]
}
```

### Custom Types

Define reusable types in the `types` section:

```json
{
  "types": {
    "DeviceId": {
      "type": "string",
      "constraints": {
        "pattern": "^[0-9a-f]{4}:[0-9a-f]{4}$"
      }
    },
    "DiscoveryResult": {
      "type": "object",
      "properties": {
        "device_count": {"type": "uint32"},
        "duration_ms": {"type": "uint64"}
      }
    }
  }
}
```

Reference custom types:

```json
{
  "name": "device_id",
  "type": "$ref:DeviceId"
}
```

## Event Definition

```json
{
  "name": "DiscoveryProgress",
  "description": "Discovery progress update",
  "payload": {
    "type": "object",
    "properties": {
      "progress": {
        "type": "float64",
        "description": "Progress percentage (0.0-1.0)",
        "constraints": {"min": 0.0, "max": 1.0}
      },
      "current_device": {
        "type": "string",
        "description": "Currently scanning device"
      }
    }
  }
}
```

## Constraints

### Numeric Constraints

```json
{
  "type": "uint8",
  "constraints": {
    "min": 1,
    "max": 255,
    "multiple_of": 2
  }
}
```

### String Constraints

```json
{
  "type": "string",
  "constraints": {
    "min_length": 1,
    "max_length": 256,
    "pattern": "^[a-zA-Z0-9_]+$",
    "enum": ["option1", "option2"]
  }
}
```

### Array Constraints

```json
{
  "type": "array",
  "constraints": {
    "min_items": 1,
    "max_items": 100,
    "unique_items": true
  }
}
```

## Validation Rules

### Build-Time Validation

1. **Contract Completeness**
   - All `#[ffi_export]` functions must have contract definitions
   - All contract functions must have Rust implementations
   - Parameter names and types must match exactly

2. **Type Safety**
   - All parameter types must be valid FFI types
   - Return types must be JSON-serializable
   - No generic types across FFI boundary

3. **Constraint Validation**
   - Numeric constraints must be valid for type
   - String patterns must be valid regex
   - Enum values must be non-empty

### Runtime Validation (Dev Mode)

1. **Parameter Validation**
   - Check parameter types match contract
   - Validate constraints (min/max, length, pattern)
   - Log validation failures with details

2. **Return Validation**
   - Check return structure matches contract
   - Validate return type consistency
   - Log schema mismatches

3. **Event Validation**
   - Check event payloads match contract
   - Validate event types
   - Log unknown events

## Error Handling

### Standard Error Codes

All contracts must document these standard errors:

- `INVALID_INPUT` - Parameter validation failed
- `NULL_POINTER` - Null pointer passed
- `INVALID_UTF8` - String encoding error
- `NOT_FOUND` - Resource not found
- `INTERNAL_ERROR` - Unexpected failure/panic
- `SERIALIZATION_FAILED` - JSON serialization error
- `DESERIALIZATION_FAILED` - JSON deserialization error

### Custom Error Codes

Domains may define custom errors:

```json
{
  "errors": [
    {
      "code": "DISCOVERY_TIMEOUT",
      "description": "Discovery process timed out after 30 seconds",
      "details_schema": {
        "type": "object",
        "properties": {
          "timeout_ms": {"type": "uint64"},
          "devices_found": {"type": "uint32"}
        }
      }
    }
  ]
}
```

## Versioning

### Contract Versioning

Contracts use semantic versioning:
- **MAJOR**: Breaking changes to function signatures
- **MINOR**: New functions or optional parameters
- **PATCH**: Documentation updates, constraint refinements

### Protocol Versioning

Protocol version increments when:
- FFI data structures change
- Callback signatures change
- Error format changes
- Event format changes

## Code Generation

### Rust Code Generation

From contracts, generate:
1. **Trait definitions** for domain interfaces
2. **Validation functions** for parameter checking
3. **Documentation** from descriptions
4. **Test scaffolding** from examples

### Dart Code Generation

From contracts, generate:
1. **FFI function bindings** with typed wrappers
2. **Model classes** for complex return types
3. **Validation functions** for parameters
4. **Mock implementations** for testing
5. **Documentation** with examples

## Example: Complete Contract

See: `core/src/ffi/contracts/discovery.ffi-contract.json`

## Tools

### Contract Validator

```bash
cargo run --bin validate-ffi-contracts
```

Validates:
- JSON schema conformance
- Type correctness
- Constraint validity
- Reference integrity

### Contract Generator

```bash
cargo run --bin generate-ffi-contracts
```

Generates contracts from existing `#[ffi_export]` functions (migration tool).

### Contract Diff

```bash
cargo run --bin diff-ffi-contracts v1.0.0 v2.0.0
```

Shows breaking changes between contract versions.

## Migration Path

1. **Phase 1**: Generate initial contracts from existing FFI code
2. **Phase 2**: Add contracts to build validation (warnings only)
3. **Phase 3**: Enable contract enforcement (build fails on mismatch)
4. **Phase 4**: Add runtime validation in dev mode
5. **Phase 5**: Use contracts for documentation generation

## Best Practices

1. **Keep contracts up to date** - Update contracts before changing FFI code
2. **Document everything** - Add descriptions to all functions, parameters, events
3. **Use examples** - Provide input/output examples for complex functions
4. **Version carefully** - Increment versions appropriately for changes
5. **Validate early** - Run contract validation in pre-commit hooks
6. **Review changes** - Contract changes should be reviewed like code changes

## References

- [FFI Architecture](./ffi-panic-safety.md)
- [Event System](../core/src/ffi/events.rs)
- [FfiExportable Trait](../core/src/ffi/traits.rs)
- [Macro Implementation](../core/ffi-macros/src/lib.rs)
