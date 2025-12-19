# Project Structure

## High-Level Layout

```
keyrx/
├── core/                  # Rust backend and FFI exports
├── packages/              # Pure Dart packages
│   └── core_logic/        # FFI bindings and shared business logic (Flutter-independent)
├── apps/                  # Dart applications
│   └── cli_tool/          # Headless Dart CLI harness for testing/automation
├── ui/                    # Flutter app (depends on core_logic)
│   ├── lib/               # Dart source (see detailed map below)
│   ├── test/              # Unit/widget/golden/perf tests
│   └── pubspec.yaml
├── scripts/               # Rhai standard library
└── .spec-workflow/        # Specs and steering docs
```

## ui/lib Structure (actual)

```
ui/lib/
├── main.dart                 # App entry
├── config/                   # Constants, thresholds, paths
├── ffi/                      # Bridges to Rust (bindings, sessions, profiles, discovery)
│   └── generated/            # Generated FFI artifacts (do not hand-edit)
├── mixins/                   # Shared mixins (e.g., StreamSubscriber)
├── models/                   # Freezed/data models, codecs, validation helpers
├── repositories/             # Persistence/data access abstractions
├── services/                 # Business logic + bridge adapters (engine, profiles, metrics)
│   └── facade/               # Service facades used by UI
├── state/                    # Riverpod providers/setup, app state containers
├── pages/                    # Screen-level widgets (layouts, mapping, debugger, metrics, trade-offs)
├── widgets/                  # Reusable UI components
│   ├── common/               # Styled inputs/buttons
│   ├── debug/                # Log/metrics viewers
│   ├── flow/                 # Flow editor nodes
│   ├── metrics/              # Metrics dashboard pieces
│   ├── preview/              # Preview widgets
│   ├── scripting/            # API browser, scripting helpers
│   ├── settings/             # Settings widgets
│   └── ... (feature widgets: keyboard, layout_grid, visual keyboards)
├── utils/                    # Logging, timeline utilities
└── ui/                       # Design-system assets/widgets (shared primitives)
    ├── assets/
    ├── styles/
    └── widgets/
```

Representative files:
- `pages/`: `layouts.dart`, `mapping_page.dart`, `debugger_page.dart`, `metrics_dashboard.dart`, `typing_simulator.dart`.
- `services/`: `engine_service.dart`, `device_registry_service.dart`, `mapping_validator.dart`, `observability_service.dart`.
- `state/`: `providers_setup.dart` (Riverpod bootstrap), `providers.dart`, `app_state.dart`.
- `widgets/`: `visual_keyboard.dart`, `virtual_layout_renderer.dart`, `device_card.dart`, `developer_drawer.dart`, `metrics_dashboard.dart`.
- `ui/widgets/`: `app_error_dialog.dart`, `loading_overlay.dart`; `ui/styles/surfaces.dart` centralizes surface theming.
- `ffi/`: `bridge.dart`, `bridge_core.dart`, `bridge_discovery.dart`, `bridge_engine.dart`, `bridge_session.dart`, `bridge_validation.dart`, `mock_bridge.dart` plus generated bindings.

## Naming Conventions

- Files: `snake_case.dart`; generated files keep tool suffixes (e.g., `*.freezed.dart`, `*.g.dart`, `generated_bindings.dart`).
- Classes/types: `PascalCase`; widgets end with `Widget` when not obviously a page (`DeviceCard`, `VisualKeyboard`).
- Functions/methods: `camelCase`; private helpers prefixed with `_`.
- Providers: `xxxProvider` for Riverpod providers; group exports in `state/providers.dart`.
- Constants: prefer `lowerCamelCase` for runtime constants, `UPPER_SNAKE_CASE` for compile-time/environment switches.

## Shared Services Architecture (SSOT Pattern)

**Goal**: Business logic services must be shared between CLI and UI to ensure Single Source of Truth (SSOT).

**Pattern** (following DeviceRegistryService model):

```
packages/core_logic/lib/services/
  ├── device_registry_service.dart  ✅ Shared (CLI + UI)
  ├── runtime_service.dart          🔄 Move here (no Flutter deps)
  ├── keymap_service.dart            🔄 Move here (no Flutter deps)
  ├── hardware_service.dart          🔄 Move here (refactor ChangeNotifier → Stream)
  └── layout_service.dart            🔄 Move here (refactor ChangeNotifier → Stream)

ui/lib/services/
  ├── device_registry_service.dart  ✅ Thin wrapper (UI migration logic)
  ├── hardware_service_ui.dart      🆕 UI wrapper (ChangeNotifier adapter)
  ├── layout_service_ui.dart        🆕 UI wrapper (ChangeNotifier adapter)
  ├── daemon_service.dart           ✅ UI-specific (keep)
  └── generated/ipc_client.dart     ✅ Generated (keep)
```

**Rules**:
1. **Core services** (config, runtime, hardware, layout, keymap) MUST be in `core_logic`
2. **No Flutter dependencies** in core_logic services (use `StreamController` instead of `ChangeNotifier`)
3. **UI wrappers** only for:
   - Migration logic (e.g., SharedPreferences → JSON)
   - Adapting core streams to ChangeNotifier for Riverpod compatibility
4. **CLI tools** import directly from `core_logic` (no UI dependencies)

**Example - Device Registry (Proven Pattern)**:

```dart
// packages/core_logic/lib/services/device_registry_service.dart
class DeviceRegistryServiceImpl implements DeviceRegistryService {
  final KeyrxBridge _bridge;
  final DeviceConfigRepository _repository;
  final StreamController<List<DeviceState>> _devicesController;

  Stream<List<DeviceState>> get devicesStream => _devicesController.stream;
  // ... no Flutter deps, pure Dart
}

// ui/lib/services/device_registry_service.dart (thin wrapper)
class DeviceRegistryServiceImpl implements DeviceRegistryService {
  final core.DeviceRegistryService _core;

  Stream<List<DeviceState>> get devicesStream => _core.devicesStream;
  Future<void> addVirtualDevice(...) => _core.addVirtualDevice(...);

  // UI-specific: migration from SharedPreferences
  static Future<void> migrateFromSharedPreferences(...) { ... }
}
```

## Module Boundaries and Dependencies

- Pages depend on widgets + state providers; they should not talk directly to FFI—use services/facades.
- Widgets are presentation-only; pull data via injected callbacks/providers rather than services directly when possible.
- **Services (core_logic)** coordinate business logic and bridge calls; they consume repositories and FFI bridges. Must be Flutter-independent.
- **Services (UI wrappers)** provide UI-specific adaptations (migration, ChangeNotifier) over core services.
- Repositories encapsulate storage/access paths (files, registries) and stay framework-agnostic.
- Models are pure data and may be shared across layers; Freezed/JSON codecs live here.
- State holds Riverpod providers and app-level state; keep side-effects inside services, not providers.
- Utils (logging/timeline) are leaf dependencies; nothing should import UI from utils.
- Generated code under `ffi/generated` and `*.g.dart`/`*.freezed.dart` must not be edited manually.
- **CLI tools** MUST use services from `core_logic` only; NO UI dependencies allowed.

## Import Organization (Dart)

1) Dart SDK (`dart:`)  
2) Flutter framework (`package:flutter/`)  
3) External packages (`package:flutter_riverpod/`, etc.)  
4) Project packages (`package:keyrx/...`)  
5) Relative imports (for local folders)

Example:
```dart
import 'dart:async';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:keyrx/services/engine_service.dart';
import 'package:keyrx/state/providers.dart';
import '../widgets/visual_keyboard.dart';
```

## Directory Examples and Rules

- `pages/layouts.dart`: screen composition; pulls providers from `state/providers.dart`, renders shared widgets like `visual_keyboard.dart`.
- `widgets/flow/`: node widgets for the flow editor; depend on models and styling, not on services.
- `ui/widgets/app_error_dialog.dart`: design-system overlays; reusable across pages.
- `services/engine_service_impl.dart`: orchestrates FFI bridge calls and translates models; logs via `utils/logger.dart`.
- `state/providers_setup.dart`: wraps the app in `ProviderScope`, registers global providers; keep side-effects centralized.

## Rust Snapshot (for context)

Core keeps the existing layout (`core/src/engine`, `core/src/scripting`, `core/src/ffi`, drivers under `core/src/drivers`, CLI under `core/src/cli`). FFI remains the only boundary to the Flutter UI; UI must not import Rust modules directly.

## Size and Quality Guards

- Max 500 lines per file and 50 lines per function (enforced via metrics checks).
- Keep nesting depth <=4; prefer extracting helpers.
- Public APIs documented with `///`; user-facing widgets include brief usage examples where non-obvious.
