# KeyRx Project Structure

**Version:** 2.0 - Revolutionary Mapping Architecture
**Last Updated:** 2025-12-06

---

## 1. Repository Organization

```
keyrx/
├── core/                           # Rust core library and CLI
│   ├── src/
│   │   ├── lib.rs                 # Library root
│   │   ├── bin/
│   │   │   └── keyrx.rs           # CLI binary
│   │   ├── registry/              # NEW: Device & Profile registries
│   │   │   ├── mod.rs
│   │   │   ├── device.rs          # DeviceRegistry
│   │   │   ├── profile.rs         # ProfileRegistry
│   │   │   └── bindings.rs        # Device-Profile bindings persistence
│   │   ├── identity/              # NEW: Device identity & serial extraction
│   │   │   ├── mod.rs
│   │   │   ├── types.rs           # DeviceIdentity struct
│   │   │   ├── windows.rs         # Windows serial extraction
│   │   │   └── linux.rs           # Linux serial extraction
│   │   ├── definitions/           # NEW: Device definition loader
│   │   │   ├── mod.rs
│   │   │   ├── loader.rs          # TOML parser
│   │   │   ├── library.rs         # DeviceDefinitionLibrary
│   │   │   └── types.rs           # DeviceDefinition structs
│   │   ├── engine/                # Event processing engine
│   │   │   ├── mod.rs
│   │   │   ├── core.rs            # Main engine loop
│   │   │   ├── multi_device.rs    # Multi-device coordinator (MODIFIED)
│   │   │   ├── device_resolver.rs # NEW: Device identity resolution
│   │   │   ├── profile_resolver.rs # NEW: Profile loading
│   │   │   ├── coordinate_translator.rs # NEW: Scancode -> (Row,Col)
│   │   │   ├── action_resolver.rs # NEW: (Row,Col) -> KeyAction
│   │   │   └── executor.rs        # Action execution (MODIFIED)
│   │   ├── drivers/               # Platform-specific I/O
│   │   │   ├── mod.rs
│   │   │   ├── windows/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── raw_input.rs   # Raw Input API (MODIFIED for serial)
│   │   │   │   └── injector.rs    # SendInput output
│   │   │   └── linux/
│   │   │       ├── mod.rs
│   │   │       ├── evdev_input.rs # evdev input (MODIFIED for serial)
│   │   │       └── uinput_output.rs # uinput output
│   │   ├── discovery/             # Device detection (LEGACY - to be refactored)
│   │   │   ├── mod.rs
│   │   │   ├── session.rs         # Discovery session
│   │   │   ├── storage.rs         # Old profile storage (DEPRECATED)
│   │   │   └── types.rs           # Old DeviceProfile (DEPRECATED)
│   │   ├── config/                # Configuration management
│   │   │   ├── mod.rs
│   │   │   ├── paths.rs           # Config directory resolution
│   │   │   └── timing.rs          # Timing constants
│   │   ├── ffi/                   # Foreign Function Interface
│   │   │   ├── mod.rs
│   │   │   ├── domains/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── device.rs      # OLD: Device FFI (DEPRECATED)
│   │   │   │   ├── device_registry.rs # NEW: DeviceRegistry FFI
│   │   │   │   ├── profile_registry.rs # NEW: ProfileRegistry FFI
│   │   │   │   ├── device_definitions.rs # NEW: Definitions FFI
│   │   │   │   ├── engine.rs      # Engine control
│   │   │   │   ├── script.rs      # Script management
│   │   │   │   └── validation.rs  # Validation
│   │   │   └── utils.rs           # FFI utilities
│   │   ├── scripting/             # Rhai scripting engine
│   │   │   ├── mod.rs
│   │   │   ├── engine.rs          # Rhai engine wrapper
│   │   │   └── sandbox.rs         # Sandbox configuration
│   │   ├── validation/            # Schema validation
│   │   │   └── mod.rs
│   │   ├── observability/         # Logging, metrics, tracing
│   │   │   ├── mod.rs
│   │   │   ├── logging.rs
│   │   │   ├── metrics.rs
│   │   │   └── tracing.rs
│   │   ├── safety/                # Panic prevention
│   │   │   └── mod.rs
│   │   ├── traits/                # Core abstractions
│   │   │   └── mod.rs
│   │   └── migration/             # NEW: Data migration utilities
│   │       ├── mod.rs
│   │       └── v1_to_v2.rs        # Migrate old profiles to new system
│   ├── tests/                     # Integration tests
│   │   ├── device_identity_tests.rs # NEW: Serial tracking tests
│   │   ├── profile_registry_tests.rs # NEW: Profile CRUD tests
│   │   ├── multi_device_tests.rs  # NEW: Multi-device scenarios
│   │   └── migration_tests.rs     # NEW: Migration tests
│   ├── benches/                   # Performance benchmarks
│   │   └── mapping_latency.rs     # NEW: Pipeline latency benchmarks
│   ├── Cargo.toml
│   └── build.rs
├── ui/                             # Flutter UI application
│   ├── lib/
│   │   ├── main.dart              # App entry point (MODIFIED navigation order)
│   │   ├── pages/
│   │   │   ├── devices_page.dart  # Device list (MODIFIED - enhanced UI)
│   │   │   ├── device_profiles_page.dart # Profile management (MODIFIED)
│   │   │   ├── visual_editor_page.dart # Visual editor (MAJOR REWRITE)
│   │   │   ├── run_controls_page.dart
│   │   │   ├── debugger_page.dart
│   │   │   └── console_page.dart
│   │   ├── widgets/
│   │   │   ├── device_card.dart   # NEW: Device list item widget
│   │   │   ├── remap_toggle.dart  # NEW: Per-device toggle
│   │   │   ├── profile_selector.dart # NEW: Profile dropdown
│   │   │   ├── layout_grid.dart   # NEW: Dynamic row-col grid renderer
│   │   │   ├── soft_keyboard.dart # NEW: Virtual keyboard palette
│   │   │   └── drag_drop_mapper.dart # MODIFIED: Row-col drag-drop
│   │   ├── models/
│   │   │   ├── device_identity.dart # NEW: DeviceIdentity model
│   │   │   ├── device_state.dart  # NEW: DeviceState model
│   │   │   ├── profile.dart       # NEW: Profile model
│   │   │   ├── layout_type.dart   # NEW: LayoutType model
│   │   │   ├── physical_position.dart # NEW: PhysicalPosition model
│   │   │   ├── key_action.dart    # NEW: KeyAction model
│   │   │   ├── device_definition.dart # NEW: DeviceDefinition model
│   │   │   └── keyboard_layout.dart # OLD: Legacy layout (keep for reference)
│   │   ├── services/
│   │   │   ├── device_registry_service.dart # NEW: Device registry API
│   │   │   ├── profile_registry_service.dart # NEW: Profile registry API
│   │   │   ├── device_definition_service.dart # NEW: Definitions API
│   │   │   ├── device_service.dart # OLD: Legacy (DEPRECATED)
│   │   │   └── device_profile_service.dart # OLD: Legacy (DEPRECATED)
│   │   ├── ffi/
│   │   │   ├── device_registry_ffi.dart # NEW: DeviceRegistry FFI bindings
│   │   │   ├── profile_registry_ffi.dart # NEW: ProfileRegistry FFI bindings
│   │   │   ├── device_definitions_ffi.dart # NEW: Definitions FFI bindings
│   │   │   ├── bridge_device_profile.dart # OLD: Legacy (keep for migration)
│   │   │   └── keyrx_bindings.dart # Generated FFI bindings
│   │   ├── state/
│   │   │   ├── app_state.dart
│   │   │   ├── device_registry_provider.dart # NEW: Device state provider
│   │   │   └── profile_registry_provider.dart # NEW: Profile state provider
│   │   └── repositories/
│   │       ├── device_repository.dart # MODIFIED: Use new registry
│   │       └── profile_repository.dart # MODIFIED: Use new registry
│   ├── pubspec.yaml
│   └── test/
├── device_definitions/             # NEW: Device definition library (TOML)
│   ├── standard/
│   │   ├── ansi-104.toml          # Standard ANSI keyboard
│   │   ├── iso-105.toml           # Standard ISO keyboard
│   │   └── jis-109.toml           # Standard JIS keyboard
│   ├── elgato/
│   │   ├── stream-deck-mk2.toml   # Stream Deck MK.2 (3×5)
│   │   ├── stream-deck-xl.toml    # Stream Deck XL (4×8)
│   │   └── stream-deck-mini.toml  # Stream Deck Mini (2×3)
│   ├── custom/
│   │   ├── macro-pad-5x5.toml     # Generic 5×5 macro pad
│   │   └── split-keyboard.toml    # Generic split keyboard
│   └── README.md                  # Device definition format spec
├── docs/
│   ├── revolutional-mapping.md    # Revolutionary vision document
│   ├── product.md                 # NEW: Product steering document
│   ├── tech.md                    # NEW: Technical steering document
│   ├── structure.md               # NEW: This file
│   ├── ARCHITECTURE.md            # Existing architecture doc
│   ├── features.md
│   ├── ffi-architecture.md
│   └── [other docs...]
├── scripts/
│   ├── test_feature_combinations.sh
│   ├── check_ffi_collisions.sh
│   └── migrate_profiles.sh        # NEW: Profile migration script
├── .vscode/
│   └── tasks.json                 # Build tasks
├── Cargo.toml                     # Workspace root
├── README.md
└── LICENSE
```

---

## 2. Module Dependency Graph

### 2.1. Core Library Modules

```
                    ┌─────────────┐
                    │   lib.rs    │
                    └──────┬──────┘
                           │
         ┌─────────────────┼─────────────────┐
         │                 │                 │
         ▼                 ▼                 ▼
    ┌─────────┐      ┌──────────┐     ┌──────────┐
    │registry │      │ identity │     │definitions│
    └────┬────┘      └─────┬────┘     └─────┬────┘
         │                 │                 │
         │                 │                 │
         ▼                 ▼                 ▼
    ┌──────────────────────────────────────────┐
    │              engine                      │
    │  ┌────────────────────────────────────┐  │
    │  │ device_resolver                    │  │
    │  │ profile_resolver                   │  │
    │  │ coordinate_translator              │  │
    │  │ action_resolver                    │  │
    │  │ executor                           │  │
    │  └────────────────────────────────────┘  │
    └──────────────┬───────────────────────────┘
                   │
         ┌─────────┼─────────┐
         │         │         │
         ▼         ▼         ▼
    ┌────────┐ ┌────────┐ ┌──────────┐
    │drivers │ │scripting│ │validation│
    └────────┘ └────────┘ └──────────┘
         │
         ▼
    ┌────────────┐
    │ observability│
    └────────────┘
```

**Dependency Rules:**
- `registry` depends on `identity`, `definitions`
- `engine` depends on `registry`, `drivers`, `scripting`
- `drivers` are platform-specific (conditional compilation)
- `ffi` is the top-level interface (depends on everything)

### 2.2. Flutter UI Modules

```
                    ┌─────────────┐
                    │  main.dart  │
                    └──────┬──────┘
                           │
         ┌─────────────────┼─────────────────┐
         │                 │                 │
         ▼                 ▼                 ▼
    ┌─────────┐      ┌──────────┐     ┌──────────┐
    │  pages  │      │  state   │     │ services │
    └────┬────┘      └─────┬────┘     └─────┬────┘
         │                 │                 │
         │                 └────────┬────────┘
         │                          │
         ▼                          ▼
    ┌─────────┐              ┌────────────┐
    │ widgets │              │repositories│
    └────┬────┘              └──────┬─────┘
         │                          │
         └────────────┬─────────────┘
                      │
                      ▼
                ┌──────────┐
                │   ffi    │
                └────┬─────┘
                     │
                     ▼
                ┌──────────┐
                │  models  │
                └──────────┘
```

**Dependency Rules:**
- `pages` use `widgets`, `state`, `services`
- `services` use `ffi`, `repositories`
- `repositories` use `ffi`, `models`
- `state` uses `models`, `repositories`
- `ffi` is the lowest layer (C interop)

---

## 3. File Naming Conventions

### 3.1. Rust Files

**Module Files:**
- `mod.rs` - Module root (public API)
- `types.rs` - Data structures
- `impl.rs` - Implementation details

**Feature-Specific:**
- `{feature}_registry.rs` - Registry implementation
- `{platform}_{feature}.rs` - Platform-specific code (e.g., `windows_serial.rs`)

**Test Files:**
- `{module}_tests.rs` - Unit tests in `tests/` directory
- Inline tests in same file under `#[cfg(test)]` module

### 3.2. Dart Files

**Widgets:**
- `{widget_name}_widget.dart` - Reusable UI components
- Suffix `_widget` optional if context is clear

**Pages:**
- `{page_name}_page.dart` - Full-screen views

**Services:**
- `{domain}_service.dart` - Business logic services

**FFI:**
- `{domain}_ffi.dart` - FFI bindings

**Models:**
- `{entity}.dart` - Data models (singular noun)

**State:**
- `{domain}_provider.dart` - State management providers

---

## 4. Configuration File Locations

### 4.1. User Configuration

**Linux:**
```
$XDG_CONFIG_HOME/keyrx/          (default: ~/.config/keyrx/)
├── profiles/
│   ├── {profile-id}.json
│   └── ...
├── device_bindings.json
└── settings.json
```

**Windows:**
```
%APPDATA%\keyrx\
├── profiles\
│   ├── {profile-id}.json
│   └── ...
├── device_bindings.json
└── settings.json
```

### 4.2. Application Data

**Device Definitions (Shipped with App):**
```
{app_install_dir}/device_definitions/
├── standard/
├── elgato/
└── custom/
```

**User Custom Definitions (Optional):**
```
$XDG_CONFIG_HOME/keyrx/device_definitions/
└── {vendor}/
    └── {device}.toml
```

---

## 5. Build Artifacts

### 5.1. Rust Artifacts

```
target/
├── debug/
│   ├── keyrx                     # CLI binary (debug)
│   └── libkeyrx_core.{so|dll}    # Shared library (debug)
├── release/
│   ├── keyrx                     # CLI binary (release)
│   └── libkeyrx_core.{so|dll}    # Shared library (release)
└── release-debug/
    ├── keyrx                     # CLI binary (profiling)
    └── libkeyrx_core.{so|dll}    # Shared library (profiling)
```

### 5.2. Flutter Artifacts

```
ui/build/
├── linux/
│   └── x64/release/bundle/
│       ├── keyrx_ui              # Flutter executable
│       └── lib/
│           └── libkeyrx_core.so  # Rust library (copied)
└── windows/
    └── x64/runner/Release/
        ├── keyrx_ui.exe
        └── keyrx_core.dll        # Rust library (copied)
```

---

## 6. Critical File Mapping (Current → New)

### 6.1. Rust Core Changes

| Current File | New/Modified | Description |
|--------------|--------------|-------------|
| `discovery/types.rs` | DEPRECATED | Old DeviceProfile, replaced by registry system |
| `discovery/storage.rs` | DEPRECATED | Old profile storage, replaced by ProfileRegistry |
| - | `registry/device.rs` (NEW) | DeviceRegistry implementation |
| - | `registry/profile.rs` (NEW) | ProfileRegistry implementation |
| - | `registry/bindings.rs` (NEW) | Device-profile binding persistence |
| - | `identity/types.rs` (NEW) | DeviceIdentity struct |
| - | `identity/windows.rs` (NEW) | Windows serial extraction |
| - | `identity/linux.rs` (NEW) | Linux serial extraction |
| - | `definitions/loader.rs` (NEW) | TOML device definition loader |
| - | `definitions/library.rs` (NEW) | DeviceDefinitionLibrary |
| `engine/multi_device.rs` | MODIFIED | Use DeviceRegistry instead of old discovery |
| `drivers/windows/raw_input.rs` | MODIFIED | Extract serial from device path |
| `drivers/linux/evdev_input.rs` | MODIFIED | Extract serial via EVIOCGUNIQ |
| `ffi/domains/device.rs` | DEPRECATED | Old device FFI, replaced by registry FFI |
| - | `ffi/domains/device_registry.rs` (NEW) | DeviceRegistry FFI |
| - | `ffi/domains/profile_registry.rs` (NEW) | ProfileRegistry FFI |
| - | `ffi/domains/device_definitions.rs` (NEW) | DeviceDefinitions FFI |

### 6.2. Flutter UI Changes

| Current File | New/Modified | Description |
|--------------|--------------|-------------|
| `main.dart` | MODIFIED | Reorder navigation: Devices ABOVE Editor |
| `pages/devices_page.dart` | MODIFIED | Add toggle, profile dropdown, user labels |
| `pages/visual_editor_page.dart` | MAJOR REWRITE | Dynamic layouts, row-col grid, soft keyboard |
| - | `widgets/device_card.dart` (NEW) | Device list item with controls |
| - | `widgets/remap_toggle.dart` (NEW) | Per-device remap switch |
| - | `widgets/profile_selector.dart` (NEW) | Profile dropdown |
| - | `widgets/layout_grid.dart` (NEW) | Dynamic row-col renderer |
| - | `widgets/soft_keyboard.dart` (NEW) | Virtual keyboard palette |
| - | `models/device_identity.dart` (NEW) | DeviceIdentity model |
| - | `models/device_state.dart` (NEW) | DeviceState model |
| - | `models/profile.dart` (NEW) | Profile model |
| - | `services/device_registry_service.dart` (NEW) | Device registry service |
| - | `services/profile_registry_service.dart` (NEW) | Profile registry service |
| `services/device_service.dart` | DEPRECATED | Old device service |
| `services/device_profile_service.dart` | DEPRECATED | Old profile service |
| - | `ffi/device_registry_ffi.dart` (NEW) | DeviceRegistry FFI bindings |
| - | `ffi/profile_registry_ffi.dart` (NEW) | ProfileRegistry FFI bindings |

---

## 7. Testing Structure

### 7.1. Rust Tests

```
core/tests/
├── device_identity_tests.rs      # Serial tracking, multi-device distinction
├── profile_registry_tests.rs     # Profile CRUD, compatibility checks
├── device_registry_tests.rs      # Device state management
├── coordinate_translation_tests.rs # Scancode -> (Row, Col) mapping
├── multi_device_tests.rs         # Concurrent device handling
├── migration_tests.rs            # V1 -> V2 profile migration
└── integration_tests.rs          # End-to-end pipeline tests

core/benches/
└── mapping_latency.rs            # Performance benchmarks (Criterion)
```

### 7.2. Flutter Tests

```
ui/test/
├── unit/
│   ├── models/
│   │   ├── device_identity_test.dart
│   │   ├── profile_test.dart
│   │   └── layout_type_test.dart
│   └── services/
│       ├── device_registry_service_test.dart
│       └── profile_registry_service_test.dart
├── widget/
│   ├── device_card_test.dart
│   ├── remap_toggle_test.dart
│   ├── layout_grid_test.dart
│   └── soft_keyboard_test.dart
└── integration/
    ├── devices_page_test.dart
    ├── visual_editor_page_test.dart
    └── profile_assignment_test.dart
```

---

## 8. Build System Integration

### 8.1. Cargo Workspace

```toml
# Root Cargo.toml
[workspace]
members = ["core"]
resolver = "2"

[workspace.dependencies]
tokio = { version = "1.41", features = ["rt-multi-thread", "sync", "macros"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
rhai = { version = "1.19", features = ["sync"] }
# ... other shared dependencies
```

### 8.2. Flutter Build Integration

**Build Script** (`ui/build.sh`):
```bash
#!/bin/bash
set -e

# Build Rust core library
cd ../core
cargo build --release --no-default-features --features linux-driver

# Copy to Flutter lib directory
cp target/release/libkeyrx_core.so ../ui/linux/

# Build Flutter app
cd ../ui
flutter build linux --release
```

### 8.3. CI/CD Pipeline

```yaml
# .github/workflows/build.yml
name: Build and Test

on: [push, pull_request]

jobs:
  rust-tests:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest]
    steps:
      - uses: actions/checkout@v3
      - name: Build Rust
        run: cargo build --all-features
      - name: Run Rust tests
        run: cargo test --all-features
      - name: Run benchmarks
        run: cargo bench --no-run

  flutter-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: subosito/flutter-action@v2
      - name: Build Rust library
        run: cd core && cargo build --release
      - name: Copy library
        run: cp core/target/release/libkeyrx_core.so ui/linux/
      - name: Run Flutter tests
        run: cd ui && flutter test
```

---

## 9. Development Workflows

### 9.1. Adding a New Device Definition

**Steps:**
1. Create TOML file in `device_definitions/{vendor}/{device}.toml`
2. Define layout type (Matrix, Standard, Split)
3. Map scancodes/usage IDs to (row, col) positions
4. Add visual metadata (optional)
5. Test with real device

**Example Workflow:**
```bash
# Create definition
vim device_definitions/acme/macro-pad-pro.toml

# Rebuild to include in bundle
cargo build

# Test with CLI
keyrx detect --device-path /dev/input/event5
```

### 9.2. Adding a New Profile

**Steps:**
1. User creates profile via UI or CLI
2. Profile saved to `$XDG_CONFIG_HOME/keyrx/profiles/{id}.json`
3. Profile appears in dropdown for compatible devices

**Example (CLI):**
```bash
keyrx profile create \
  --name "Gaming FPS" \
  --layout matrix:5x5 \
  --from template:gaming

keyrx profile assign \
  --device "VID:0fd9 PID:0080 Serial:ABC123" \
  --profile gaming-fps
```

### 9.3. Implementing a New FFI Domain

**Steps:**
1. Create Rust module in `core/src/ffi/domains/{domain}.rs`
2. Define `extern "C"` functions with `#[no_mangle]`
3. Create Dart bindings in `ui/lib/ffi/{domain}_ffi.dart`
4. Create service layer in `ui/lib/services/{domain}_service.dart`
5. Add to module exports in `core/src/ffi/mod.rs`

**Template:**
```rust
// core/src/ffi/domains/my_domain.rs
#[no_mangle]
pub extern "C" fn krx_my_domain_action(arg: *const c_char) -> *mut c_char {
    ffi_catch_panic(|| {
        let input = unsafe { CStr::from_ptr(arg).to_str()? };
        // ... implementation
        Ok(CString::new("ok".to_string())?.into_raw())
    })
}
```

```dart
// ui/lib/ffi/my_domain_ffi.dart
class MyDomainFFI {
  late final _action = _lib.lookupFunction<
    Pointer<Utf8> Function(Pointer<Utf8>),
    Pointer<Utf8> Function(Pointer<Utf8>)
  >('krx_my_domain_action');

  Future<void> performAction(String arg) async {
    final argPtr = arg.toNativeUtf8();
    final resultPtr = _action(argPtr);
    calloc.free(argPtr);
    // ... handle result
  }
}
```

---

## 10. Code Organization Best Practices

### 10.1. Rust Module Structure

**Preferred:**
```rust
// mod.rs - Public API
pub use types::*;
pub use registry::DeviceRegistry;

mod types;
mod registry;
mod storage;
```

**Avoid:**
- Deeply nested modules (max 3 levels)
- Circular dependencies
- Public re-exports of internal implementation details

### 10.2. Flutter File Organization

**Preferred:**
- Group by feature/domain (not by type)
- Keep related files together
- Use barrel files (`index.dart`) sparingly

**Example:**
```
lib/features/device_management/
├── widgets/
│   ├── device_card.dart
│   └── device_list.dart
├── services/
│   └── device_service.dart
├── models/
│   └── device.dart
└── device_page.dart
```

### 10.3. Shared Types

**Rust:**
- Define shared types in `{module}/types.rs`
- Use public re-exports in `mod.rs`

**Flutter:**
- Define models in `lib/models/`
- Use `freezed` for immutable data classes
- Use `json_serializable` for JSON conversion

---

## 11. Documentation Standards

### 11.1. Code Comments

**Rust:**
```rust
/// Extracts serial number from device path.
///
/// # Arguments
/// * `device_path` - OS-specific device path
///
/// # Returns
/// Serial number string (hardware or synthetic)
///
/// # Errors
/// Returns `Error::InvalidDevicePath` if path is malformed
pub fn extract_serial_number(device_path: &str) -> Result<String> {
    // Implementation
}
```

**Dart:**
```dart
/// Assigns a profile to a device.
///
/// Throws [DeviceNotFoundException] if device not found.
/// Throws [ProfileNotFoundException] if profile not found.
/// Throws [LayoutIncompatibleException] if layouts don't match.
Future<void> assignProfile(DeviceIdentity device, String profileId) async {
  // Implementation
}
```

### 11.2. Architecture Decision Records (ADRs)

**Location:** `docs/adr/`

**Format:**
```markdown
# ADR-001: Use TOML for Device Definitions

**Status:** Accepted
**Date:** 2025-12-06

## Context
Need static device specifications for hardware layout mapping.

## Decision
Use TOML instead of Rhai for device definitions.

## Rationale
- Static data, not logic
- Safe for community sharing
- Fast parsing

## Consequences
- Separate file format from profiles
- Community can contribute safely
```

---

## 12. Migration Roadmap (File Changes)

### Phase 1: Core Data Structures (Week 1-2)
**New Files:**
- `core/src/registry/device.rs`
- `core/src/registry/profile.rs`
- `core/src/identity/types.rs`
- `core/src/definitions/types.rs`

### Phase 2: Serial Extraction (Week 2-3)
**New Files:**
- `core/src/identity/windows.rs`
- `core/src/identity/linux.rs`

**Modified Files:**
- `core/src/drivers/windows/raw_input.rs`
- `core/src/drivers/linux/evdev_input.rs`

### Phase 3: FFI Layer (Week 3-4)
**New Files:**
- `core/src/ffi/domains/device_registry.rs`
- `core/src/ffi/domains/profile_registry.rs`
- `ui/lib/ffi/device_registry_ffi.dart`
- `ui/lib/ffi/profile_registry_ffi.dart`

### Phase 4: UI Overhaul (Week 4-6)
**Modified Files:**
- `ui/lib/main.dart` (navigation order)
- `ui/lib/pages/devices_page.dart` (UI enhancements)
- `ui/lib/pages/visual_editor_page.dart` (major rewrite)

**New Files:**
- `ui/lib/widgets/device_card.dart`
- `ui/lib/widgets/remap_toggle.dart`
- `ui/lib/widgets/layout_grid.dart`

### Phase 5: Device Definitions (Week 6-7)
**New Files:**
- `device_definitions/standard/ansi-104.toml`
- `device_definitions/elgato/*.toml`
- `core/src/definitions/loader.rs`

### Phase 6: Migration & Testing (Week 7-8)
**New Files:**
- `core/src/migration/v1_to_v2.rs`
- `core/tests/migration_tests.rs`
- `scripts/migrate_profiles.sh`

---

This structure document provides a comprehensive map of the codebase organization for the revolutionary mapping architecture implementation.
