# Dart FFI Binding Generator

A Rust CLI tool that generates type-safe Dart FFI bindings from JSON contracts, eliminating manual synchronization between Rust exports and Dart imports.

## Overview

This tool reads FFI contract JSON files from `core/src/ffi/contracts/` and generates:
- **FFI Bindings**: Native function typedefs and Dart function pointers
- **Model Classes**: Dart classes with JSON serialization for custom types

## Quick Start

```bash
# Generate all bindings
just gen-dart-bindings

# Check if bindings are up-to-date (CI mode)
just check-dart-bindings

# Or run directly
cargo run --release --manifest-path core/tools/generate_dart_bindings/Cargo.toml
```

## CLI Options

```
generate-dart-bindings [OPTIONS]

Options:
  -d, --domain <DOMAIN>      Generate bindings for a specific domain only
  -c, --check                Check if bindings are up-to-date without generating
  -v, --verbose              Enable verbose output
      --contracts <PATH>     Path to contracts directory (default: core/src/ffi/contracts)
      --output <PATH>        Path to output directory (default: ui/lib)
  -h, --help                 Print help
  -V, --version              Print version
```

## Usage Examples

### Generate All Bindings

```bash
cargo run --bin generate-dart-bindings
```

### Generate for Specific Domain

```bash
cargo run --bin generate-dart-bindings --domain config
```

### CI Check Mode

Returns exit code 1 if bindings are out of date:

```bash
cargo run --bin generate-dart-bindings --check
```

### Custom Paths

```bash
cargo run --bin generate-dart-bindings \
  --contracts path/to/contracts \
  --output path/to/output
```

## Contract Format

Contracts are JSON files with the `.ffi-contract.json` extension:

```json
{
  "$schema": "https://keyrx.dev/schemas/ffi-contract-v1.json",
  "version": "1.0.0",
  "domain": "config",
  "description": "Configuration management operations",
  "protocol_version": 1,
  "functions": [
    {
      "name": "list_items",
      "rust_name": "keyrx_config_list_items",
      "description": "List all items",
      "parameters": [],
      "returns": {
        "type": "string",
        "description": "JSON array of items"
      },
      "errors": []
    },
    {
      "name": "save_item",
      "rust_name": "keyrx_config_save_item",
      "description": "Save an item",
      "parameters": [
        {
          "name": "json",
          "type": "string",
          "description": "JSON representation",
          "required": true
        }
      ],
      "returns": {
        "type": "string",
        "description": "JSON result"
      },
      "errors": []
    }
  ],
  "events": [],
  "types": []
}
```

### Supported Parameter Types

| Contract Type | Dart FFI Type | Dart Native Type |
|--------------|---------------|------------------|
| `string` | `Pointer<Char>` | `String` |
| `int` / `i32` | `Int32` | `int` |
| `i64` | `Int64` | `int` |
| `u32` | `Uint32` | `int` |
| `u64` | `Uint64` | `int` |
| `bool` | `Bool` | `bool` |
| `void` | `Void` | `void` |
| `pointer` | `Pointer<Void>` | `Pointer<Void>` |

## Generated Output

### Bindings File

Generated at `ui/lib/ffi/generated/bindings_generated.dart`:

```dart
/// Auto-generated FFI bindings for KeyRx Core.
/// DO NOT EDIT MANUALLY - changes will be overwritten.

import 'dart:ffi';

// Function Typedefs
typedef ConfigListItemsNative = Pointer<Char> Function();
typedef ConfigListItems = Pointer<Char> Function();

typedef ConfigSaveItemNative = Pointer<Char> Function(Pointer<Char>);
typedef ConfigSaveItem = Pointer<Char> Function(Pointer<Char>);
```

### Model Classes

For contracts with custom types defined, model classes are generated at `ui/lib/models/generated/`:

```dart
class Item {
  final String id;
  final String name;

  Item({required this.id, required this.name});

  factory Item.fromJson(Map<String, dynamic> json) {
    return Item(
      id: json['id'] as String,
      name: json['name'] as String,
    );
  }

  Map<String, dynamic> toJson() => {
    'id': id,
    'name': name,
  };
}
```

## Using Generated Bindings in Dart

```dart
import 'dart:ffi';
import 'package:ffi/ffi.dart';
import 'ffi/generated/bindings_generated.dart';

class FfiBridge {
  late DynamicLibrary _lib;
  late ConfigListItems listItems;
  late ConfigSaveItem saveItem;

  void init() {
    _lib = DynamicLibrary.open('libkeyrx_core.so');

    // Lookup functions using generated typedefs
    listItems = _lib.lookupFunction<ConfigListItemsNative, ConfigListItems>(
      'keyrx_config_list_items',
    );

    saveItem = _lib.lookupFunction<ConfigSaveItemNative, ConfigSaveItem>(
      'keyrx_config_save_item',
    );
  }

  String getItems() {
    final result = listItems();
    return result.cast<Utf8>().toDartString();
  }

  String saveNewItem(String json) {
    final jsonPtr = json.toNativeUtf8().cast<Char>();
    try {
      final result = saveItem(jsonPtr);
      return result.cast<Utf8>().toDartString();
    } finally {
      malloc.free(jsonPtr);
    }
  }
}
```

## Build Integration

### Justfile Recipes

```bash
# Generate bindings (included in build)
just gen-dart-bindings

# Check bindings are current (CI)
just check-dart-bindings

# Full build includes binding generation
just build
```

### CI Integration

The CI pipeline runs `just check-dart-bindings` to verify bindings match contracts. If contracts are modified, regenerate bindings before committing:

```bash
just gen-dart-bindings
git add ui/lib/ffi/generated/
git commit -m "chore: regenerate dart bindings"
```

## Architecture

```
generate_dart_bindings/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── cli.rs           # Argument parsing (clap)
│   ├── lib.rs           # Library exports
│   ├── loader.rs        # Contract file loading
│   ├── types.rs         # DartType enum
│   ├── type_mapper.rs   # Contract → Dart type mapping
│   ├── templates.rs     # Code generation templates
│   ├── bindings_gen.rs  # FFI signature generator
│   ├── models_gen.rs    # Model class generator
│   ├── header.rs        # File header generator
│   ├── writer.rs        # File output handler
│   ├── formatter.rs     # Dart format integration
│   ├── pipeline.rs      # Orchestration
│   └── tests.rs         # Unit tests
└── tests/
    └── integration_test.rs  # End-to-end tests
```

### Generation Pipeline

1. **Load**: Read contracts from `--contracts` directory
2. **Parse**: Deserialize JSON into contract structures
3. **Map Types**: Convert contract types to Dart FFI types
4. **Generate**: Create Dart code using templates
5. **Write**: Output to `--output` directory (if changed)
6. **Format**: Run `dart format` on generated files

## Development

### Running Tests

```bash
# Unit tests
cd core && cargo test -p generate_dart_bindings

# Integration tests (requires contracts)
cd core && cargo test -p generate_dart_bindings --test integration_test
```

### Adding New Types

1. Add the type mapping in `src/type_mapper.rs`
2. Add corresponding `DartType` variant in `src/types.rs`
3. Update templates if needed in `src/templates.rs`
4. Add tests in `src/type_mapper_tests.rs`

## Troubleshooting

### "Bindings are out of date" in CI

Contracts were modified without regenerating bindings:

```bash
just gen-dart-bindings
git add -A
git commit --amend --no-edit
```

### "dart format not found"

Install the Dart SDK or skip formatting:

```bash
# The tool continues with a warning if dart isn't available
```

### Type mapping errors

Check that all parameter types in your contract are supported. See the type mapping table above.
