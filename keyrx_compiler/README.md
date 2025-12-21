# keyrx_compiler

Rhai-to-binary configuration compiler for KeyRx.

## Purpose

`keyrx_compiler` is a standalone CLI tool that compiles Rhai DSL configuration scripts into static `.krx` binary files. It performs:

- **Rhai parsing**: Parse configuration scripts written in Rhai DSL with prefix validation
- **Import resolution**: Handle multi-file configurations with circular dependency detection
- **Binary serialization**: Output `.krx` files using rkyv zero-copy format with SHA256 integrity checking
- **Validation**: Verify .krx files for correctness

## Installation

Build from source:
```bash
cargo build --release -p keyrx_compiler
```

The binary will be available at `target/release/keyrx_compiler`.

## CLI Commands

### compile

Compile a Rhai script to a .krx binary file:

```bash
keyrx_compiler compile input.rhai -o output.krx
```

If `-o` is not specified, the output file will be `input.krx` (same name as input with .krx extension).

### verify

Verify a .krx binary file:

```bash
keyrx_compiler verify config.krx
```

This validates:
- Magic bytes correctness
- Version compatibility
- SHA256 hash integrity
- Binary structure validity

### hash

Extract and display the SHA256 hash from a .krx file:

```bash
keyrx_compiler hash config.krx
```

Outputs the hash as a hexadecimal string.

### parse

Parse a Rhai script and output the configuration:

```bash
# Human-readable output
keyrx_compiler parse input.rhai

# JSON output
keyrx_compiler parse input.rhai --json
```

Useful for debugging and inspecting configurations without compiling.

## KeyRx DSL Syntax

KeyRx uses a Rhai-based DSL with a strict prefix system to distinguish between different key types.

### Prefix System

All output keys must have one of these prefixes:

- **VK_** - Virtual Key: Standard key output (e.g., `VK_A`, `VK_Enter`, `VK_Escape`)
- **MD_** - Custom Modifier: Custom modifier state for layer switching (e.g., `MD_00` through `MD_FE`)
- **LK_** - Custom Lock: Toggle state (e.g., `LK_00` through `LK_FE`)

Input keys (the `from` parameter) **never** have a prefix - they use plain key names like `CapsLock`, `Space`, `A`.

### Basic Remapping

```rhai
device_start("*");  // "*" matches all devices

    // Simple key remapping
    map("CapsLock", "VK_Escape");
    map("A", "VK_B");

device_end();
```

### Custom Modifiers (Layers)

```rhai
device_start("*");

    // CapsLock becomes custom modifier MD_00
    map("CapsLock", "MD_00");

    // When MD_00 is held, enable Vim navigation
    when("MD_00", [
        map("H", "VK_Left"),
        map("J", "VK_Down"),
        map("K", "VK_Up"),
        map("L", "VK_Right"),
    ]);

device_end();
```

### Tap-Hold Behavior

```rhai
device_start("*");

    // Space: tap for space, hold for custom modifier MD_01
    // threshold_ms is the tap/hold detection time
    tap_hold("Space", "VK_Space", "MD_01", 200);

    // When Space (MD_01) is held, enable number pad
    when("MD_01", [
        map("U", "VK_7"),
        map("I", "VK_8"),
        map("O", "VK_9"),
    ]);

device_end();
```

### Custom Locks (Toggles)

```rhai
device_start("*");

    // ScrollLock toggles custom lock LK_00 (game mode)
    map("ScrollLock", "LK_00");

    // When game mode is active
    when("LK_00", [
        map("W", "VK_W"),
        map("A", "VK_A"),
        map("S", "VK_S"),
        map("D", "VK_D"),
    ]);

    // When game mode is NOT active
    when_not("LK_00", [
        map("Escape", "VK_Escape"),
    ]);

device_end();
```

### Physical Modifier Outputs

```rhai
device_start("*");

    // Output key with physical modifiers
    with_shift("F1", "VK_F1");      // F1 with Shift
    with_ctrl("F2", "VK_F2");       // F2 with Ctrl
    with_alt("F3", "VK_F3");        // F3 with Alt

    // Combine multiple modifiers
    with_mods("F4", "VK_F4", true, true, false, false);
    // Parameters: from, to, shift, ctrl, alt, win

device_end();
```

### Multiple Conditions (AND Logic)

```rhai
device_start("*");

    map("CapsLock", "MD_00");
    map("Space", "MD_01");

    // When BOTH MD_00 AND MD_01 are active
    when(["MD_00", "MD_01"], [
        map("1", "VK_F1"),
        map("2", "VK_F2"),
    ]);

device_end();
```

### Device-Specific Configuration

```rhai
// Default configuration for all devices
device_start("*");
    map("CapsLock", "VK_Escape");
device_end();

// Configuration specific to USB keyboards
device_start("USB Keyboard");
    map("Enter", "VK_Space");
device_end();
```

## Examples

See the `examples/` directory for complete configuration examples:

- **simple.rhai** - Basic key remapping
- **advanced.rhai** - All DSL features including custom modifiers, locks, tap/hold, conditionals, and physical modifier outputs

To parse an example:
```bash
keyrx_compiler parse examples/simple.rhai
keyrx_compiler parse examples/advanced.rhai --json
```

To compile an example:
```bash
keyrx_compiler compile examples/simple.rhai -o my-config.krx
```

## Error Handling

The compiler provides detailed error messages with file location and suggestions:

```
Parse error: examples/config.rhai:15:5: Invalid prefix: expected valid key name, got 'B'

Suggestion: Output must have VK_, MD_, or LK_ prefix: B → use VK_B for virtual key
```

Common errors:
- **Missing prefix**: Output keys must have `VK_`, `MD_`, or `LK_` prefix
- **Invalid prefix**: Unknown key name after prefix
- **Physical modifier in MD_**: Cannot use physical modifier names (like `MD_LShift`) - use hex IDs (`MD_00` through `MD_FE`)
- **Out of range**: Custom modifier/lock IDs must be 00-FE (0-254)

## Binary Format (.krx)

The `.krx` binary format consists of:

- **Header (48 bytes)**:
  - Magic bytes: `KRX\n` (4 bytes)
  - Version: u32 (4 bytes)
  - SHA256 hash: 32 bytes
  - Data size: u64 (8 bytes)
- **Data**: rkyv-serialized ConfigRoot

The hash ensures integrity - any modification to the binary will be detected during verification.

## Testing

Run unit tests:
```bash
cargo test -p keyrx_compiler
```

Run integration tests:
```bash
cargo test -p keyrx_compiler --test integration_tests
```

Run all tests with coverage:
```bash
cargo tarpaulin -p keyrx_compiler
```

## Dependencies

- **rhai** - Embedded scripting language for DSL
- **rkyv** - Zero-copy serialization
- **serde** - JSON serialization for parse command
- **clap** - CLI argument parsing
- **sha2** - SHA256 hashing for integrity

## Further Reading

For complete DSL documentation and examples, see:
- `examples/simple.rhai` - Basic configuration example
- `examples/advanced.rhai` - Advanced features showcase
- Design docs: `.spec-workflow/specs/core-config-system/design.md`
