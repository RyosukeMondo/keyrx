# Project Structure

## Directory Organization

```
keyrx/
├── core/                       # Rust backend (primary logic)
│   ├── src/
│   │   ├── lib.rs             # Library root, public API
│   │   ├── engine/            # Event loop and processing
│   │   │   ├── mod.rs
│   │   │   ├── event_loop.rs  # Tokio async event loop
│   │   │   └── state.rs       # Layer state machine
│   │   ├── scripting/         # Rhai integration
│   │   │   ├── mod.rs
│   │   │   ├── runtime.rs     # Rhai engine setup
│   │   │   └── bindings.rs    # Rust-Rhai type bindings
│   │   ├── drivers/           # OS adapter traits
│   │   │   ├── mod.rs
│   │   │   ├── traits.rs      # InputSource trait definition
│   │   │   ├── windows.rs     # WH_KEYBOARD_LL implementation
│   │   │   └── linux.rs       # uinput/evdev implementation
│   │   └── ffi/               # C-ABI exports for Flutter
│   │       ├── mod.rs
│   │       └── exports.rs     # extern "C" functions
│   ├── tests/                 # Integration tests
│   ├── benches/               # Criterion benchmarks
│   └── Cargo.toml
│
├── ui/                         # Flutter frontend
│   ├── lib/
│   │   ├── main.dart          # Application entry point
│   │   ├── ffi/               # Dart FFI bindings
│   │   │   ├── bindings.dart  # Generated FFI bindings
│   │   │   └── bridge.dart    # High-level Rust bridge
│   │   ├── pages/             # UI screens
│   │   │   ├── editor.dart    # Visual keymap editor
│   │   │   ├── debugger.dart  # Real-time state visualizer
│   │   │   └── console.dart   # Rhai REPL terminal
│   │   ├── widgets/           # Reusable UI components
│   │   │   ├── keyboard.dart  # Visual keyboard widget
│   │   │   └── layer_panel.dart
│   │   └── state/             # Application state management
│   ├── test/                  # Widget tests
│   └── pubspec.yaml
│
├── scripts/                    # User Rhai configurations
│   └── std/                   # Standard library
│       ├── layouts/           # Keyboard layout definitions
│       │   ├── 109.rhai       # JIS 109-key layout
│       │   └── ansi.rhai      # ANSI layout
│       ├── layers.rhai        # Layer management utilities
│       └── modifiers.rhai     # Custom modifier helpers
│
├── docs/                       # Documentation
│   ├── ARCHITECTURE.md        # Technical architecture
│   └── STEERING.md            # Project steering document
│
├── .spec-workflow/            # Spec workflow artifacts
│   ├── steering/              # Steering documents
│   └── specs/                 # Feature specifications
│
└── README.md                  # Project overview
```

## Naming Conventions

### Files
- **Rust modules**: `snake_case.rs` (e.g., `event_loop.rs`, `state_machine.rs`)
- **Dart files**: `snake_case.dart` (e.g., `keymap_editor.dart`)
- **Rhai scripts**: `snake_case.rhai` (e.g., `user_config.rhai`)
- **Tests**: `[module]_test.rs` (Rust), `[file]_test.dart` (Dart)

### Code

#### Rust
- **Structs/Enums**: `PascalCase` (e.g., `Engine`, `LayerState`)
- **Functions/Methods**: `snake_case` (e.g., `process_event`, `activate_layer`)
- **Constants**: `UPPER_SNAKE_CASE` (e.g., `MAX_MODIFIERS`, `DEFAULT_LATENCY`)
- **Traits (Interfaces)**: `PascalCase` descriptive names (e.g., `InputSource`, `ScriptRuntime`, `StateStore`)
- **Mock Implementations**: `Mock` + trait name (e.g., `MockInputSource`, `MockScriptRuntime`)

#### Dart
- **Classes**: `PascalCase` (e.g., `KeyboardWidget`, `RustBridge`)
- **Functions/Methods**: `camelCase` (e.g., `processKey`, `updateState`)
- **Constants**: `lowerCamelCase` or `UPPER_SNAKE_CASE` for compile-time
- **Files**: `snake_case.dart`

#### Rhai
- **Functions**: `snake_case` (e.g., `on_key_press`, `activate_layer`)
- **Variables**: `snake_case` (e.g., `current_layer`, `mod_flags`)

## Import Patterns

### Rust Import Order
1. Standard library (`std::`)
2. External crates (`tokio::`, `rhai::`)
3. Crate modules (`crate::engine::`)
4. Super/self imports

### Dart Import Order
1. Dart SDK (`dart:ffi`, `dart:async`)
2. Flutter framework (`package:flutter/`)
3. External packages
4. Project imports (`package:keyrx/`)
5. Relative imports

### Module Organization
- Use re-exports in `mod.rs` / `lib.rs` to create clean public APIs
- Keep implementation details private
- Platform-specific code behind `#[cfg(target_os = "...")]`

## Code Structure Patterns

### Rust Module Organization
```rust
// 1. Imports
use std::collections::HashMap;
use tokio::sync::mpsc;

// 2. Constants
const MAX_LAYERS: usize = 32;

// 3. Type definitions
pub struct Engine { ... }
pub enum Event { ... }

// 4. Trait implementations
impl Engine { ... }

// 5. Private helpers
fn validate_input(...) { ... }

// 6. Tests (in same file or tests/)
#[cfg(test)]
mod tests { ... }
```

### Dart File Organization
```dart
// 1. Imports
import 'dart:ffi';
import 'package:flutter/material.dart';

// 2. Constants
const kMaxLayers = 32;

// 3. Class definitions
class KeyboardEditor extends StatefulWidget { ... }

// 4. State classes
class _KeyboardEditorState extends State<KeyboardEditor> { ... }

// 5. Helper widgets/functions
Widget _buildKeyButton(...) { ... }
```

## Code Organization Principles

1. **Single Responsibility**: Each module handles one concern (e.g., `engine/` for event processing, `scripting/` for Rhai)
2. **Modularity**: OS drivers are interchangeable via `InputSource` trait
3. **Testability**: All external dependencies injected; MockInputSource for testing
4. **Consistency**: Follow established patterns in each language
5. **CLI First**: Every feature CLI-exercisable before GUI implementation

## Dependency Injection Pattern

All external dependencies are abstracted behind traits and injected:

```
core/src/
├── traits/                    # DI interface definitions
│   ├── mod.rs
│   ├── input_source.rs       # pub trait InputSource
│   ├── script_runtime.rs     # pub trait ScriptRuntime
│   └── state_store.rs        # pub trait StateStore
├── impl/                      # Production implementations
│   ├── mod.rs
│   ├── windows_input.rs      # impl InputSource for WindowsInput
│   ├── linux_input.rs        # impl InputSource for LinuxInput
│   ├── rhai_runtime.rs       # impl ScriptRuntime for RhaiRuntime
│   └── memory_state.rs       # impl StateStore for InMemoryState
└── mocks/                     # Test mocks
    ├── mod.rs
    ├── mock_input.rs         # impl InputSource for MockInput
    ├── mock_runtime.rs       # impl ScriptRuntime for MockRuntime
    └── mock_state.rs         # impl StateStore for MockState
```

**Trait Naming**: Use descriptive `PascalCase` names that describe the capability (e.g., `InputSource`, `ScriptRuntime`), not `ISomething` prefix.

**Mock Naming**: Prefix with `Mock` (e.g., `MockInputSource`).

## CLI Structure

```
core/src/
├── bin/
│   └── keyrx.rs              # CLI entry point
├── cli/
│   ├── mod.rs
│   ├── commands/
│   │   ├── check.rs          # keyrx check - validate scripts
│   │   ├── run.rs            # keyrx run - start engine
│   │   ├── simulate.rs       # keyrx simulate - event simulation
│   │   ├── state.rs          # keyrx state - inspect state
│   │   ├── doctor.rs         # keyrx doctor - self-diagnostics
│   │   ├── bench.rs          # keyrx bench - latency benchmark
│   │   └── repl.rs           # keyrx repl - interactive mode
│   └── output.rs             # JSON/human-readable output formatting
```

Every CLI command supports:
- `--json` flag for machine-readable output (AI agent friendly)
- `--verbose` flag for detailed debugging
- Exit codes for scripting (0=success, 1=error, 2=validation failure)

## Module Boundaries

### Core vs UI
- Core exposes C-ABI functions only via `ffi/` module
- UI never imports Rust types directly; uses FFI bridge
- State synchronization via event passing, not shared memory

### Core vs Drivers
- Core defines `InputSource` trait
- Drivers implement trait without Core knowing OS specifics
- Drivers are compile-time selected via Cargo features

### Public API vs Internal
- `lib.rs` exports only public types
- Internal modules use `pub(crate)` visibility
- FFI functions are the only `extern "C"` exports

### Platform-specific Isolation
```rust
#[cfg(target_os = "windows")]
mod windows_driver;

#[cfg(target_os = "linux")]
mod linux_driver;
```

## Code Size Guidelines

Per CLAUDE.md requirements:
- **File size**: Maximum 500 lines (excluding comments/blank lines)
- **Function size**: Maximum 50 lines
- **Test coverage**: 80% minimum, 90% for critical paths (engine, scripting)
- **Nesting depth**: Maximum 4 levels

## Documentation Standards

### Rust
- All public items have `///` doc comments
- Module-level `//!` documentation in `mod.rs`
- Examples in doc comments where helpful

### Dart
- `///` doc comments for public APIs
- Widget documentation includes usage examples
- README in `lib/` directories for complex modules

### Rhai Standard Library
- Header comment explaining purpose
- Inline comments for non-obvious logic
- Usage examples in script files
