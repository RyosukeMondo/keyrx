# Design Document: code-quality-refactor

## Overview

This refactoring introduces a **declarative macro system** for keycode definitions, splits oversized driver files into focused submodules, adds the `KeyInjector` trait for testability, and extracts shared utilities. All changes maintain **zero runtime overhead** through compile-time code generation.

## Steering Document Alignment

### Technical Standards (tech.md)
- **Dependency Injection**: New `KeyInjector` trait follows existing trait-based DI pattern
- **Performance**: Declarative macros compile to match statements (jump tables)
- **Input latency < 1ms**: No runtime lookups, all conversions are compiled

### Project Structure (structure.md)
- Follows existing `core/src/` organization
- New submodules under `drivers/linux/` and `drivers/windows/`
- Shared utilities in `drivers/common.rs`

## Code Reuse Analysis

### Existing Components to Leverage
- **`InputSource` trait**: Pattern for new `KeyInjector` trait
- **`MockInput`**: Pattern for new `MockKeyInjector`
- **`error.rs`**: Error types for new driver errors
- **Existing tests**: Patterns for new unit tests

### Integration Points
- **`LinuxInput`**: Will use injected `KeyInjector` instead of owned `UinputWriter`
- **`WindowsInput`**: Will use injected `KeyInjector` instead of owned `SendInputInjector`
- **`RhaiRuntime`**: Will use shared `parse_key_or_error()` helper

## Architecture

### Keycode Definition System

```
┌─────────────────────────────────────────────────────────────────┐
│  keycodes.rs (SINGLE SOURCE OF TRUTH)                           │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │ define_keycodes! {                                          ││
│  │   A       => evdev: 30,  vk: 0x41, aliases: ["a"];          ││
│  │   B       => evdev: 48,  vk: 0x42, aliases: ["b"];          ││
│  │   CapsLock=> evdev: 58,  vk: 0x14, aliases: ["caps"];       ││
│  │   // ... all 127 keycodes                                   ││
│  │ }                                                           ││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ Generates at compile time:
                              ▼
┌──────────────────┬──────────────────┬──────────────────┬──────────────────┐
│ KeyCode enum     │ Display impl     │ FromStr impl     │ evdev↔KeyCode    │
│ (types.rs)       │ (auto-derived)   │ (with aliases)   │ vk↔KeyCode       │
└──────────────────┴──────────────────┴──────────────────┴──────────────────┘
```

### Driver Module Structure

```
drivers/
├── mod.rs              # Re-exports: LinuxInput, WindowsInput, DeviceInfo
├── common.rs           # DeviceInfo, extract_panic_message()
├── keycodes.rs         # define_keycodes! macro + invocation
├── injector.rs         # KeyInjector trait + MockKeyInjector
│
├── linux/
│   ├── mod.rs          # LinuxInput struct, InputSource impl
│   ├── reader.rs       # EvdevReader (event capture thread)
│   ├── writer.rs       # UinputWriter impl KeyInjector
│   └── keymap.rs       # evdev_to_keycode(), keycode_to_evdev()
│
└── windows/
    ├── mod.rs          # WindowsInput struct, InputSource impl
    ├── hook.rs         # HookManager (WH_KEYBOARD_LL)
    ├── injector.rs     # SendInputInjector impl KeyInjector
    └── keymap.rs       # vk_to_keycode(), keycode_to_vk()
```

## Components and Interfaces

### Component 1: Keycode Macro System

**Purpose:** Single source of truth for all keycode definitions
**File:** `core/src/drivers/keycodes.rs`

```rust
/// Declarative macro generating KeyCode enum and all conversions
macro_rules! define_keycodes {
    (
        $(
            $variant:ident => evdev: $evdev:expr, vk: $vk:expr,
                              aliases: [$($alias:literal),*]
        );* $(;)?
    ) => {
        // 1. Generate KeyCode enum (imported by types.rs)
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub enum KeyCode {
            $($variant,)*
            Unknown(u16),
        }

        // 2. Generate Display impl
        impl std::fmt::Display for KeyCode { ... }

        // 3. Generate FromStr impl (with aliases)
        impl std::str::FromStr for KeyCode { ... }

        // 4. Generate evdev conversion (Linux only)
        #[cfg(target_os = "linux")]
        pub fn evdev_to_keycode(code: u16) -> KeyCode { ... }

        #[cfg(target_os = "linux")]
        pub fn keycode_to_evdev(key: KeyCode) -> u16 { ... }

        // 5. Generate VK conversion (Windows only)
        #[cfg(target_os = "windows")]
        pub fn vk_to_keycode(vk: u16) -> KeyCode { ... }

        #[cfg(target_os = "windows")]
        pub fn keycode_to_vk(key: KeyCode) -> u16 { ... }

        // 6. Generate key set for uinput registration
        #[cfg(target_os = "linux")]
        pub fn all_keycodes() -> &'static [u16] { ... }
    };
}

// Single invocation - THE source of truth
define_keycodes! {
    // Letters A-Z
    A       => evdev: 30,  vk: 0x41, aliases: ["a"];
    B       => evdev: 48,  vk: 0x42, aliases: ["b"];
    // ... all 127 keycodes
}
```

**Interfaces:**
- `KeyCode` enum (pub)
- `evdev_to_keycode()`, `keycode_to_evdev()` (pub, cfg linux)
- `vk_to_keycode()`, `keycode_to_vk()` (pub, cfg windows)
- `all_keycodes()` (pub, cfg linux)

**Dependencies:** None (self-contained)

### Component 2: KeyInjector Trait

**Purpose:** Abstract key injection for testability
**File:** `core/src/drivers/injector.rs`

```rust
use crate::engine::KeyCode;
use anyhow::Result;

/// Trait for injecting keyboard events into the OS.
///
/// Enables mock injection for testing without hardware.
pub trait KeyInjector: Send {
    /// Inject a key press or release.
    fn inject(&mut self, key: KeyCode, pressed: bool) -> Result<()>;

    /// Sync/flush any pending events (Linux uinput needs this).
    fn sync(&mut self) -> Result<()>;
}

/// Mock injector for testing.
#[derive(Default)]
pub struct MockKeyInjector {
    pub injected: Vec<(KeyCode, bool)>,
}

impl KeyInjector for MockKeyInjector {
    fn inject(&mut self, key: KeyCode, pressed: bool) -> Result<()> {
        self.injected.push((key, pressed));
        Ok(())
    }

    fn sync(&mut self) -> Result<()> {
        Ok(())
    }
}
```

**Interfaces:** `KeyInjector` trait, `MockKeyInjector` struct
**Dependencies:** `KeyCode` from keycodes module
**Reuses:** Pattern from `InputSource` trait

### Component 3: Shared Utilities

**Purpose:** Extract duplicated patterns
**File:** `core/src/drivers/common.rs` (extend existing)

```rust
/// Extract message from panic payload.
/// Used by both Linux and Windows driver threads.
pub fn extract_panic_message(panic_info: Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = panic_info.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = panic_info.downcast_ref::<String>() {
        s.clone()
    } else {
        "Unknown panic".to_string()
    }
}
```

**File:** `core/src/scripting/helpers.rs` (new)

```rust
use crate::engine::KeyCode;
use rhai::{EvalAltResult, Position};

/// Parse a key name or return a Rhai-compatible error.
/// Used by remap(), block(), pass() functions.
pub fn parse_key_or_error(
    name: &str,
    func_name: &str
) -> std::result::Result<KeyCode, Box<EvalAltResult>> {
    KeyCode::from_name(name).ok_or_else(|| {
        Box::new(EvalAltResult::ErrorRuntime(
            format!(
                "Unknown key '{}' in {}(). See docs/KEYS.md for valid names.",
                name, func_name
            ).into(),
            Position::NONE,
        ))
    })
}
```

### Component 4: Linux Driver Submodules

**Purpose:** Split 1,600-line file into focused modules

#### `linux/mod.rs` (~150 lines)
- `LinuxInput` struct with `KeyInjector` generic
- `InputSource` trait implementation
- `new()` and `new_with_injector()` constructors
- Re-exports: `list_keyboards`, `EvdevReader`, `UinputWriter`

#### `linux/reader.rs` (~200 lines)
- `EvdevReader` struct
- `spawn()` method (refactored to ~40 lines)
- `run_loop()` helper
- `handle_thread_exit()` helper

#### `linux/writer.rs` (~150 lines)
- `UinputWriter` struct
- `KeyInjector` trait implementation
- `create_virtual_device()` helper

#### `linux/keymap.rs` (~50 lines)
- Re-exports from keycodes.rs
- `evdev_to_keycode()` wrapper
- `keycode_to_evdev()` wrapper

### Component 5: Windows Driver Submodules

**Purpose:** Split 1,743-line file into focused modules

#### `windows/mod.rs` (~200 lines)
- `WindowsInput` struct with `KeyInjector` generic
- `InputSource` trait implementation
- Re-exports: `list_keyboards`, `HookManager`, `SendInputInjector`

#### `windows/hook.rs` (~250 lines)
- `HookManager` struct
- `low_level_keyboard_proc` callback
- Thread-local storage management

#### `windows/injector.rs` (~100 lines)
- `SendInputInjector` struct
- `KeyInjector` trait implementation
- `is_extended_key()` helper

#### `windows/keymap.rs` (~50 lines)
- Re-exports from keycodes.rs
- `vk_to_keycode()` wrapper
- `keycode_to_vk()` wrapper

## Data Models

### KeyCode Enum (generated)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyCode {
    // Letters A-Z (26)
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,

    // Numbers 0-9 (10)
    Key0, Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9,

    // Function keys F1-F12 (12)
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,

    // Modifiers (8)
    LeftShift, RightShift, LeftCtrl, RightCtrl,
    LeftAlt, RightAlt, LeftMeta, RightMeta,

    // ... all 127 variants

    Unknown(u16),
}
```

### Keycode Definition Record

```rust
// Internal to macro - not exposed
struct KeycodeDef {
    variant: &'static str,
    evdev: u16,
    vk: u16,
    aliases: &'static [&'static str],
}
```

## Error Handling

### Error Scenarios

1. **Unknown keycode in evdev/vk conversion**
   - **Handling:** Return `KeyCode::Unknown(code)`
   - **User Impact:** None - graceful fallback

2. **Unknown key name in Rhai script**
   - **Handling:** Return Rhai `ErrorRuntime` with helpful message
   - **User Impact:** Script error with key name and docs reference

3. **Key injection failure**
   - **Handling:** Propagate error through `KeyInjector::inject()`
   - **User Impact:** Error logged, key event dropped

## Testing Strategy

### Unit Testing

**Keycode macro tests:**
- Roundtrip: `evdev_to_keycode(keycode_to_evdev(k)) == k`
- Roundtrip: `vk_to_keycode(keycode_to_vk(k)) == k`
- FromStr with aliases: `KeyCode::from_str("caps") == Ok(KeyCode::CapsLock)`
- Display: `KeyCode::A.to_string() == "A"`

**KeyInjector tests:**
- `MockKeyInjector` captures all injections
- Error propagation from failed injections

**Shared utility tests:**
- `extract_panic_message()` handles &str, String, and unknown
- `parse_key_or_error()` returns correct errors

### Integration Testing

**Driver tests with mocks:**
```rust
#[tokio::test]
async fn linux_input_with_mock_injector() {
    let mock = MockKeyInjector::default();
    let input = LinuxInput::new_with_injector(None, mock)?;
    // ... test without real uinput
}
```

### Benchmark Testing

```rust
#[bench]
fn bench_evdev_to_keycode(b: &mut Bencher) {
    b.iter(|| evdev_to_keycode(30)); // Should be <10ns
}

#[bench]
fn bench_keycode_to_evdev(b: &mut Bencher) {
    b.iter(|| keycode_to_evdev(KeyCode::A)); // Should be <10ns
}
```

**Regression threshold:** Any increase > 100 microseconds fails CI.
