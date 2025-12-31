# Project Structure

## Directory Organization

```
keyrx/
├── Cargo.toml              # Workspace root (4 crates)
├── Makefile                # Top-level orchestration
├── .github/workflows/      # CI/CD (clippy, tests, coverage, release)
│
├── crates/
│   ├── keyrx_core/         # Platform-agnostic logic (no_std)
│   ├── keyrx_compiler/     # Rhai → .krx compiler (CLI)
│   ├── keyrx_daemon/       # OS-specific daemon + embedded web server
│   └── keyrx_ui/           # React + WASM frontend
│
├── docs/                   # Architecture & API documentation
├── examples/               # Rhai configuration examples
└── scripts/                # AI-friendly build/test/launch scripts
```

### Crate-Level Organization

**keyrx_core/** (Platform-agnostic, no_std):
```
keyrx_core/
├── Cargo.toml              # no_std, minimal dependencies
├── src/
│   ├── lib.rs              # Public API exports
│   ├── config.rs           # rkyv-serialized config structures
│   ├── lookup.rs           # MPHF-based O(1) key lookup
│   ├── dfa.rs              # Deterministic Finite Automaton (Tap/Hold)
│   ├── state.rs            # 255-bit modifier/lock state (fixedbitset)
│   └── simulator.rs        # Deterministic Simulation Testing (DST)
├── benches/                # Criterion benchmarks
│   ├── lookup_bench.rs
│   └── dfa_bench.rs
├── fuzz/                   # cargo-fuzz targets
│   └── fuzz_targets/
│       └── event_stream.rs
└── tests/                  # Integration tests
    └── dfa_tests.rs
```

**keyrx_compiler/** (Standalone CLI tool):
```
keyrx_compiler/
├── Cargo.toml
├── src/
│   ├── main.rs             # CLI entry point (clap args)
│   ├── parser.rs           # Rhai AST evaluation
│   ├── mphf_gen.rs         # MPHF generation (boomphf)
│   ├── dfa_gen.rs          # DFA compilation from Tap/Hold configs
│   └── serialize.rs        # rkyv binary output (.krx files)
└── tests/
    └── integration/
        ├── basic.rs        # Simple config compilation
        └── complex.rs      # 255 modifiers test
```

**keyrx_daemon/** (OS-specific daemon + web server):
```
keyrx_daemon/
├── Cargo.toml              # Platform-specific dependencies, features
├── src/
│   ├── main.rs             # Daemon entry point, CLI args
│   ├── platform/
│   │   ├── mod.rs          # Platform trait abstraction
│   │   ├── linux.rs        # evdev/uinput implementation
│   │   └── windows.rs      # Low-Level Hooks + Raw Input
│   ├── web/                # Embedded web server (feature-gated)
│   │   ├── mod.rs          # axum server setup
│   │   ├── api.rs          # REST API (config upload, status)
│   │   ├── ws.rs           # WebSocket handler (real-time events)
│   │   └── static_files.rs # Serve embedded UI (include_dir!)
│   ├── loader.rs           # Memory-mapped .krx loading
│   └── logger.rs           # Structured JSON logging
├── ui_dist/                # Embedded UI (from keyrx_ui build)
│   ├── index.html
│   ├── keyrx_core_bg.wasm
│   └── assets/
└── tests/
    └── e2e/                # OS-specific integration tests
        ├── linux_test.rs
        └── windows_test.rs
```

**keyrx_ui/** (React + WASM frontend):
```
keyrx_ui/
├── package.json
├── vite.config.ts          # Vite bundler config
├── src/
│   ├── App.tsx             # Root component
│   ├── components/
│   │   ├── KeyboardVisualizer.tsx  # SVG keyboard with state
│   │   ├── DFADiagram.tsx          # State transition graph
│   │   ├── ConfigEditor.tsx        # Rhai script editor
│   │   └── DeviceSelector.tsx      # Serial number picker
│   ├── wasm/
│   │   └── core.ts         # TypeScript bindings for WASM
│   └── hooks/
│       ├── useSimulator.ts # WASM simulation hook
│       └── useDaemon.ts    # WebSocket connection to daemon
└── public/
    └── fonts/
```

## Naming Conventions

### Files (Rust)
- **Modules**: `snake_case.rs` (e.g., `mphf_gen.rs`, `static_files.rs`)
- **Tests**: `[module_name]_test.rs` or `tests/[feature].rs`
- **Benchmarks**: `[feature]_bench.rs` (e.g., `lookup_bench.rs`)
- **Binary crates**: Match project name (e.g., `keyrx_daemon`, `keyrx_compiler`)

### Files (TypeScript/React)
- **Components**: `PascalCase.tsx` (e.g., `KeyboardVisualizer.tsx`)
- **Hooks**: `use[Feature].ts` (e.g., `useSimulator.ts`)
- **Utils**: `camelCase.ts` (e.g., `wasmBindings.ts`)
- **Tests**: `[Component].test.tsx`

### Code (Rust)
- **Structs/Enums/Traits**: `PascalCase` (e.g., `ExtendedState`, `EventStream`)
- **Functions/Methods**: `snake_case` (e.g., `load_config`, `process_event`)
- **Constants**: `UPPER_SNAKE_CASE` (e.g., `MAX_MODIFIERS`, `DEFAULT_PORT`)
- **Variables**: `snake_case` (e.g., `modifier_state`, `event_queue`)
- **Type parameters**: Single uppercase letter or `PascalCase` (e.g., `T`, `EventType`)

### Code (TypeScript/React)
- **Components**: `PascalCase` (e.g., `KeyboardVisualizer`)
- **Functions/Hooks**: `camelCase` (e.g., `useSimulator`, `connectToDaemon`)
- **Constants**: `UPPER_SNAKE_CASE` (e.g., `WS_PORT`, `MAX_RETRIES`)
- **Interfaces/Types**: `PascalCase` (e.g., `DaemonState`, `KeyEvent`)

## Import Patterns

### Import Order (Rust)
```rust
// 1. Standard library
use std::collections::HashMap;

// 2. External dependencies (alphabetically)
use rkyv::{Archive, Serialize};
use serde::Deserialize;

// 3. Internal workspace crates
use keyrx_core::{EventStream, State};

// 4. Current crate modules (relative)
use crate::config::Config;
use super::utils;
```

### Import Order (TypeScript)
```typescript
// 1. React and framework
import React, { useState, useEffect } from 'react';

// 2. External dependencies
import { WebSocket } from 'ws';

// 3. Internal modules (absolute from src/)
import { WasmCore } from '@/wasm/core';

// 4. Relative imports
import { Button } from './Button';

// 5. Types (if not inline)
import type { KeyEvent } from '@/types';

// 6. Styles (last)
import './App.css';
```

### Module Organization
- **Absolute imports**: Use workspace-relative imports between crates
  - `use keyrx_core::config::Config;` (not relative paths)
- **Re-exports**: `lib.rs` or `mod.rs` re-exports public API
  ```rust
  // keyrx_core/src/lib.rs
  pub use self::config::Config;
  pub use self::dfa::DFA;
  ```
- **Feature gates**: Use `#[cfg(feature = "web")]` for optional dependencies
  ```rust
  #[cfg(feature = "web")]
  pub mod web;
  ```

## Code Structure Patterns

### Module/File Organization (Rust)

**Standard file structure**:
```rust
// 1. Module documentation
//! Module-level documentation explaining purpose.

// 2. Imports (see Import Order above)
use std::collections::HashMap;
use keyrx_core::State;

// 3. Constants
const MAX_RETRIES: usize = 3;
const TIMEOUT_MS: u64 = 1000;

// 4. Type definitions
pub struct EventProcessor {
    state: State,
}

pub enum EventType {
    KeyDown,
    KeyUp,
}

// 5. Main implementation
impl EventProcessor {
    pub fn new() -> Self { /* ... */ }
    pub fn process(&mut self) { /* ... */ }
}

// 6. Helper functions (private)
fn validate_input(data: &[u8]) -> bool { /* ... */ }

// 7. Tests (same file, feature-gated)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() { /* ... */ }
}
```

### Function/Method Organization

**Standard function structure** (per CLAUDE.md requirements):
```rust
/// Public API documentation
pub fn process_event(event: KeyEvent) -> Result<Action, Error> {
    // 1. Input validation (fail-fast)
    if !event.is_valid() {
        return Err(Error::InvalidInput);
    }

    // 2. Core logic (SLAP - Single Level of Abstraction)
    let state = load_state()?;
    let action = compute_action(&state, &event)?;

    // 3. Side effects (logging, mutations)
    log::debug!("Processed event: {:?}", event);

    // 4. Return result
    Ok(action)
}

// Helper functions extracted for SLAP compliance
fn load_state() -> Result<State, Error> { /* ... */ }
fn compute_action(state: &State, event: &KeyEvent) -> Result<Action, Error> { /* ... */ }
```

**Key principles**:
- **Max 50 lines per function** (per CLAUDE.md)
- **SLAP**: Each function operates at single level of abstraction
- **Early returns**: Fail-fast validation at the top
- **No nested callbacks**: Use `?` operator, extract functions

### Component Organization (React)

**Standard React component structure**:
```typescript
import React, { useState, useEffect } from 'react';
import type { Props } from './types';

/**
 * Component documentation
 */
export function KeyboardVisualizer({ config }: Props) {
  // 1. Hooks (useState, useEffect, custom hooks)
  const [state, setState] = useState(initialState);
  const daemon = useDaemon();

  // 2. Event handlers
  const handleKeyPress = (key: string) => {
    // ...
  };

  // 3. Render helpers (if needed)
  const renderKey = (key: string) => <div>{key}</div>;

  // 4. Effects
  useEffect(() => {
    // ...
  }, []);

  // 5. Return JSX
  return (
    <div>{/* ... */}</div>
  );
}

// 6. Helper components (local to file)
function KeyButton({ label }: { label: string }) {
  return <button>{label}</button>;
}
```

## Code Organization Principles

### 1. Single Responsibility Principle (SRP)
- Each file has **one clear purpose**
- Example: `lookup.rs` handles MPHF lookup only, not DFA logic
- Example: `KeyboardVisualizer.tsx` renders keyboard only, not WebSocket logic

### 2. Dependency Injection (DI)
- **All external dependencies injected** (per CLAUDE.md):
  ```rust
  // Good: Testable, mockable
  pub fn process_events<S: EventStream>(stream: &mut S) { /* ... */ }

  // Bad: Hard-coded dependency
  pub fn process_events() {
      let stream = evdev::open(); // NOT testable
  }
  ```
- Platform-specific code abstracted via traits:
  ```rust
  pub trait Platform {
      fn capture_input(&mut self) -> Result<KeyEvent>;
      fn inject_output(&mut self, event: KeyEvent) -> Result<()>;
  }
  ```

### 3. SSOT (Single Source of Truth)
- **Configuration**: `.krx` binary file is the ONLY source
  - Daemon, UI, tests all read same `.krx` file
  - No duplication in JSON, TOML, or other formats
- **State**: `ExtendedState` struct is the ONLY state representation
  - No shadow copies, no stale caches

### 4. Modularity & Testability
- **no_std core**: `keyrx_core` compiles without OS dependencies
  - Enables WASM compilation
  - Forces pure logic (no hidden I/O)
- **Trait abstractions**: Platform-specific code behind traits
  - `EventStream` trait for Linux/Windows/Test doubles
  - Mockable in tests

### 5. Consistency
- **Follow Rust API guidelines**: https://rust-lang.github.io/api-guidelines/
- **Follow React best practices**: Hooks, functional components

## Configuration File Organization (Multi-Device Support)

### Recommended Structure

```
~/.config/keyrx/
├── main.rhai              # Entry point (daemon loads this)
├── devices/
│   ├── left_hand.rhai     # Per-device configs
│   ├── right_hand.rhai
│   └── numpad.rhai
├── layers/
│   ├── vim_mode.rhai      # Shared layers (used by multiple devices)
│   ├── gaming.rhai
│   └── base.rhai
└── macros/
    ├── common.rhai        # Utility functions
    └── tap_hold.rhai      # Shared tap/hold definitions
```

### Single Entry Point Pattern

**Daemon behavior**:
```bash
# Daemon only knows about ONE file
keyrx_daemon --config ~/.config/keyrx/main.rhai
```

**main.rhai example**:
```rhai
// main.rhai - Single entry point

// Import device-specific configs
import "devices/left_hand.rhai";
import "devices/right_hand.rhai";

// Import shared layers (used by multiple devices)
import "layers/vim_mode.rhai";
import "layers/gaming.rhai";

// Import shared utilities
import "macros/common.rhai";

// Conditional imports (smart loading based on connected devices)
if device_exists("USB\\VID_AAAA&PID_1111\\SERIAL_LEFT") {
    import "devices/left_hand.rhai";
} else {
    log::warn("Left hand keyboard not detected");
}

if device_exists("USB\\VID_CCCC&PID_3333\\SERIAL_NUMPAD") {
    import "devices/numpad.rhai";
}
```

### Import Patterns

**Absolute imports** (from config root `~/.config/keyrx/`):
```rhai
import "devices/left_hand.rhai";
import "layers/vim_mode.rhai";
import "macros/common.rhai";
```

**Relative imports** (from current file's directory):
```rhai
// From devices/left_hand.rhai
import "../layers/vim_mode.rhai";  // Up one level, then into layers/
import "../macros/common.rhai";
```

**Conditional imports**:
```rhai
// Load device config only if device is connected
if device_exists("SERIAL_XXX") {
    import "devices/optional_device.rhai";
}

// Load layer based on environment
if env_var("GAMING_MODE") == "1" {
    import "layers/gaming.rhai";
}
```

### Multi-Device Configuration Examples

#### Single-Device Config
```rhai
// simple.rhai - Works for any keyboard
map(Key::A, Key::B);
map(Key::CapsLock, tap_hold(Key::Escape, Key::LCtrl, 200));
```

#### Multi-Device with Cross-Device State (QMK-Style)
```rhai
// devices/left_hand.rhai
let left = device("USB\\VID_AAAA&PID_1111\\SERIAL_LEFT");

// Global modifiers (shared state across ALL devices)
left.map(Key::LShift, Modifier(1));   // Global Modifier(1)
left.map(Key::RShift, Modifier(2));   // Global Modifier(2)
left.map(Key::LCtrl, Modifier(3));

// devices/right_hand.rhai
let right = device("USB\\VID_BBBB&PID_2222\\SERIAL_RIGHT");

// Respond to left keyboard's modifiers
right.map(Key::A, conditional(
    Modifier(1),  // If left Shift is held
    Key::ShiftA,  // Output uppercase A
    Key::A        // Otherwise lowercase a
));

right.map(Key::B, conditional(
    Modifier(3),  // If left Ctrl is held
    Key::CtrlB,   // Output Ctrl+B
    Key::B
));
```

#### Shared Layer Definition
```rhai
// layers/vim_mode.rhai - Shared layer (used by any device)

// Define vim layer (can be used by left_hand, right_hand, etc.)
define_layer("vim", {
    Key::H: Key::Left,
    Key::J: Key::Down,
    Key::K: Key::Up,
    Key::L: Key::Right,
    Key::D: Key::Delete,
    Key::W: Key::CtrlRight,  // Word forward
    Key::B: Key::CtrlLeft,   // Word backward
});

// devices/left_hand.rhai can then use this layer
// left.map(Key::Space, layer_toggle("vim"));
```

#### Shared Macros
```rhai
// macros/common.rhai - Utility functions

// Define reusable tap/hold configurations
fn my_tap_hold(tap_key, hold_key) {
    tap_hold(tap_key, hold_key, 200, Standard)
}

// Define common modifier combinations
fn hyper_modifier() {
    Modifier(100)  // Custom "Hyper" modifier
}

// Helper for conditional mappings
fn shift_or_normal(key) {
    conditional(Modifier(1), shift_of(key), key)
}
```

### Configuration Naming Conventions

**Files**:
- **Entry point**: `main.rhai` (required)
- **Device configs**: `[device_name].rhai` (e.g., `left_hand.rhai`, `gaming_keyboard.rhai`)
- **Layers**: `[layer_name].rhai` (e.g., `vim_mode.rhai`, `gaming.rhai`)
- **Macros**: `[functionality].rhai` (e.g., `common.rhai`, `tap_hold.rhai`)

**Rhai Code** (within files):
- **Device variables**: `snake_case` (e.g., `let left_hand = device(...)`)
- **Layer names**: `snake_case` (e.g., `define_layer("vim_mode", {...})`)
- **Functions**: `snake_case` (e.g., `fn my_tap_hold(...)`)
- **Constants**: `UPPER_SNAKE_CASE` (e.g., `const TAP_HOLD_TIMEOUT = 200`)

### Compilation Behavior

**Compiler resolves imports**:
```bash
# User runs
keyrx_compiler main.rhai -o main.krx

# Compiler automatically:
# 1. Parses main.rhai
# 2. Resolves all import statements recursively
# 3. Combines into single AST
# 4. Compiles to .krx binary
```

**Import resolution**:
- Paths are relative to config root (`~/.config/keyrx/`)
- Circular imports detected and rejected
- Missing imports fail compilation with clear error

**Conditional imports**:
```rhai
// Evaluated at compile-time
if device_exists("SERIAL_XXX") {
    import "devices/optional.rhai";
}

// NOT evaluated at compile-time (runtime check)
// This would fail - imports must be statically resolvable
let device_id = get_device_id();
import format!("devices/{}.rhai", device_id); // ERROR!
```

## Module Boundaries

### Core vs Platform-Specific
- **keyrx_core** (no_std):
  - Pure logic: DFA, MPHF lookup, state management
  - No OS dependencies (no `std::fs`, `std::net`)
  - Compilable to WASM
- **keyrx_daemon** (platform-specific):
  - OS hooks (evdev, Windows Low-Level Hooks)
  - File I/O, networking, threading
  - Platform code isolated in `platform/` module

**Dependency direction**: daemon → core (never core → daemon)

### Public API vs Internal
- **Public API**: Exposed via `pub use` in `lib.rs`
  ```rust
  // keyrx_core/src/lib.rs
  pub use config::Config;
  pub use dfa::DFA;
  // Internal: state::ExtendedState (not pub re-exported)
  ```
- **Internal**: Not re-exported, can change freely
  - Example: `state::ExtendedState` implementation details

### Stable vs Experimental
- **Stable**: Public API in `keyrx_core` (semver guarantees)
- **Experimental**: Feature-gated behind `#[cfg(feature = "unstable")]`
  - Breaking changes allowed
  - Not included in releases by default

### Optional Features
- **Web server**: `#[cfg(feature = "web")]` in `keyrx_daemon`
  - Headless daemon possible without web dependencies
- **Debug UI**: `#[cfg(feature = "debug_ui")]` for development tools

## Code Size Guidelines

Per CLAUDE.md requirements:

### File Size
- **Maximum 500 lines per file** (excluding comments/blank lines)
- If exceeded:
  - Extract helper modules
  - Split into `[module]/mod.rs` + sub-modules

### Function/Method Size
- **Maximum 50 lines per function**
- If exceeded:
  - Extract helper functions
  - Apply SLAP (Single Level of Abstraction Principle)

### Complexity Limits
- **Test coverage**: 80% minimum, 90% for critical paths (keyrx_core)
- **Cyclomatic complexity**: Aim for <10 per function (use `cargo clippy`)
- **Nesting depth**: Maximum 3 levels
  ```rust
  // Bad: 4 levels of nesting
  if x {
      if y {
          if z {
              if w { /* ... */ }
          }
      }
  }

  // Good: Early returns, flattened
  if !x { return; }
  if !y { return; }
  if !z { return; }
  if w { /* ... */ }
  ```

### Enforcement
- **Pre-commit hooks**: clippy, rustfmt, tests (mandatory per CLAUDE.md)
- **CI checks**: Fail build if limits exceeded
- **cargo clippy**: Enforced with `-D warnings` (treat warnings as errors)

## Dashboard/Monitoring Structure

### Web UI Organization
```
keyrx_daemon/src/web/
├── mod.rs              # axum server setup, routes
├── api.rs              # REST API handlers
│   ├── GET /status     # Daemon health, config hash
│   ├── POST /config    # Upload new .krx config
│   └── GET /devices    # List input devices
├── ws.rs               # WebSocket handler
│   ├── Event stream    # Real-time key events (debug mode)
│   └── State updates   # Modifier/lock state changes
└── static_files.rs     # Serve embedded UI (include_dir!)
```

### Separation of Concerns
- **Web module is optional**: `#[cfg(feature = "web")]`
- **Can be disabled**: Headless daemon for servers
- **Minimal coupling**: Web module only depends on public daemon API
- **Independent port**: Web server on :9876, doesn't interfere with input processing

### Frontend-Backend Contract
- **API spec**: Documented in `docs/api.md`
- **WebSocket events**: JSON schema with versioning
  ```json
  {
    "version": "1.0",
    "type": "key_event",
    "data": { "key": "A", "state": "down" }
  }
  ```
- **Type safety**: TypeScript types generated from Rust (future: use typeshare)

## Documentation Standards

### Rust Documentation
- **All public APIs must have doc comments**:
  ```rust
  /// Processes a key event through the remapping engine.
  ///
  /// # Arguments
  /// * `event` - The input key event
  ///
  /// # Returns
  /// The remapped action or an error
  ///
  /// # Examples
  /// ```
  /// let action = process_event(KeyEvent::new(Key::A))?;
  /// ```
  pub fn process_event(event: KeyEvent) -> Result<Action, Error> { /* ... */ }
  ```
- **Module-level docs**: `//! Module purpose`
- **Complex logic**: Inline `// comments` explaining "why", not "what"

### TypeScript Documentation
- **TSDoc for public APIs**:
  ```typescript
  /**
   * Connects to the keyrx daemon via WebSocket.
   * @param url - WebSocket URL (default: ws://localhost:9876)
   * @returns Connection handle
   */
  export function connectToDaemon(url?: string): Connection { /* ... */ }
  ```

### README Files
- Each major module has `README.md`:
  - `crates/keyrx_core/README.md`
  - `crates/keyrx_daemon/README.md`
  - `scripts/README.md` (or `scripts/CLAUDE.md` for AI agents)

### AI-Friendly Documentation
- **scripts/CLAUDE.md**: Machine-readable script documentation
  - Usage examples
  - Expected output formats
  - Failure scenarios and exit codes
- **Consistent log markers**: `=== accomplished ===` for AI parsing
- **Structured errors**: JSON error output with `--json` flag

## Testing Structure

### Test Organization
```
keyrx_core/
├── src/
│   ├── lib.rs
│   ├── dfa.rs          # Module code
│   └── dfa_tests.rs    # Unit tests (if complex)
└── tests/
    ├── integration/    # Integration tests (cross-module)
    │   └── dfa_integration.rs
    └── fixtures/       # Test data
        └── test_config.krx
```

### Test Naming
- **Unit tests**: `#[cfg(test)] mod tests` in same file
- **Integration tests**: `tests/[feature].rs`
- **Test functions**: `test_[scenario]_[expected_outcome]`
  ```rust
  #[test]
  fn test_tap_hold_timeout_triggers_hold() { /* ... */ }
  ```

### Test Data
- **Fixtures**: `tests/fixtures/` for shared test data
- **Deterministic**: Use seeded RNG, virtual clock (no wall-clock time)
- **AI-verifiable**: Tests output structured logs for AI agents to parse
