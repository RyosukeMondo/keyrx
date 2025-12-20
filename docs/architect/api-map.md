# KeyRx Architecture Data Flow Map

**Last Updated**: 2025-12-20
**Architecture**: Rust Core → FFI Bridge → Dart Core Logic → CLI/UI

---

## Overview

This diagram visualizes the complete data flow architecture of KeyRx, showing how:
1. **Rust Core** provides the engine and device management
2. **FFI Bridge** connects Rust to Dart
3. **Core Logic Services** (pure Dart) provide SSOT for configuration
4. **Dart CLI** uses core services directly for headless operation
5. **Flutter UI** uses core services via ChangeNotifier wrappers for reactive UI

---

## Architecture Sankey Diagram

```mermaid
%%{init: {'theme':'base', 'themeVariables': { 'fontSize':'16px'}}}%%
sankey-beta

%% User Entry Points
User,Rust CLI,100
User,Dart CLI,150
User,Flutter UI,200

%% Rust CLI Flow
Rust CLI,Rust Core Engine,100

%% Dart CLI Flow
Dart CLI,CommandProcessor,150
CommandProcessor,Core Services,150

%% Flutter UI Flow
Flutter UI,UI Widgets,200
UI Widgets,Riverpod Providers,200
Riverpod Providers,UI Service Wrappers,200

%% UI Wrapper Pattern
UI Service Wrappers,HardwareServiceUI,50
UI Service Wrappers,LayoutServiceUI,50
UI Service Wrappers,DeviceRegistryServiceUI,50
UI Service Wrappers,Other Services,50

%% UI Wrappers to Core Services
HardwareServiceUI,Core Services,50
LayoutServiceUI,Core Services,50
DeviceRegistryServiceUI,Core Services,50
Other Services,Core Services,50

%% Core Services Breakdown
Core Services,RuntimeService,60
Core Services,HardwareService,60
Core Services,LayoutService,60
Core Services,KeymapService,60
Core Services,DeviceRegistryService,60
Core Services,Other Core Services,50

%% Core Services to FFI Bridge
RuntimeService,FFI Bridge,60
HardwareService,60,FFI Bridge
LayoutService,FFI Bridge,60
KeymapService,FFI Bridge,60
DeviceRegistryService,FFI Bridge,60
Other Core Services,FFI Bridge,50

%% FFI Bridge to Rust
FFI Bridge,Rust Core Engine,350

%% Rust Core Engine Components
Rust Core Engine,Configuration Manager,100
Rust Core Engine,Device Discovery,80
Rust Core Engine,Event Engine,80
Rust Core Engine,Daemon/IPC,90

%% Reactive Flow Back (Events/Updates)
Configuration Manager,FFI Bridge,80
Device Discovery,FFI Bridge,60
Event Engine,FFI Bridge,60
Daemon/IPC,FFI Bridge,40

%% FFI to Core Services (Reactive)
FFI Bridge,Core Services,240

%% Core Services to UI Wrappers (Streams)
Core Services,UI Service Wrappers,150

%% UI Wrappers notify UI
UI Service Wrappers,Riverpod Providers,150
Riverpod Providers,UI Widgets,150
UI Widgets,Flutter UI,150
```

---

## Data Flow Explanation

### 1. **User Input Paths**

#### Path A: Direct Rust CLI (Legacy)
```
User → Rust CLI (core/src/bin/keyrx/) → Rust Core Engine
```
- **Use Case**: Direct system-level operations
- **Volume**: 100 units
- **Latency**: Lowest (no FFI overhead)

#### Path B: Dart CLI (Headless Harness)
```
User → Dart CLI (apps/cli_tool/)
     → CommandProcessor
     → Core Services (packages/core_logic/)
     → FFI Bridge
     → Rust Core Engine
```
- **Use Case**: Configuration management, testing without UI
- **Volume**: 150 units
- **Latency**: Low (minimal FFI calls)
- **Commands**: 22 total (13 config + 9 device)

#### Path C: Flutter UI (Primary Interface)
```
User → Flutter UI (ui/)
     → UI Widgets
     → Riverpod Providers
     → UI Service Wrappers
     → Core Services
     → FFI Bridge
     → Rust Core Engine
```
- **Use Case**: Full-featured graphical interface
- **Volume**: 200 units (primary user interface)
- **Latency**: Higher (UI rendering + FFI)
- **Reactivity**: Full reactive updates via streams

---

### 2. **Core Services Layer (SSOT)**

**Location**: `packages/core_logic/lib/services/`

All configuration and device management flows through these services:

| Service | Responsibility | LOC | Flutter Deps |
|---------|---------------|-----|--------------|
| `RuntimeService` | Runtime slot management | 177 | ❌ Zero |
| `HardwareService` | Hardware profile CRUD | 99 | ❌ Zero |
| `LayoutService` | Virtual layout CRUD | 92 | ❌ Zero |
| `KeymapService` | Keymap CRUD | 67 | ❌ Zero |
| `DeviceRegistryService` | Device registry & config | 350+ | ❌ Zero |

**Key Principle**: Pure Dart, no Flutter dependencies, shared by CLI and UI.

---

### 3. **UI Wrapper Pattern (ChangeNotifier Bridge)**

**Location**: `ui/lib/services/*_ui.dart`

Bridges core services (StreamController) to Flutter (ChangeNotifier):

```dart
// Core Service (pure Dart)
class HardwareService {
  StreamController<void> _changes = StreamController.broadcast();
  Stream<void> get changes => _changes.stream;

  Future<Result> saveProfile() {
    // ... FFI call
    _changes.add(null); // Notify listeners
  }
}

// UI Wrapper (Flutter)
class HardwareServiceUI extends ChangeNotifier {
  HardwareServiceUI(core.HardwareService core) {
    _subscription = core.changes.listen((_) => notifyListeners());
  }

  Future<Result> saveProfile() => _core.saveProfile(); // Forward
}
```

**Wrappers**:
- `HardwareServiceUI` → wraps `core.HardwareService`
- `LayoutServiceUI` → wraps `core.LayoutService`
- `DeviceRegistryServiceUI` → wraps `core.DeviceRegistryService`

---

### 4. **FFI Bridge Layer**

**Location**: `packages/core_logic/lib/ffi/bridge.dart`

Single gateway between Dart and Rust:

```dart
class KeyrxBridge {
  final KeyrxBindings bindings; // Generated FFI bindings

  bool initialize(); // Initialize Rust runtime

  // All core services use this bridge
}
```

**Volume**: 350 units (aggregated from all services)

---

### 5. **Rust Core Engine**

**Location**: `core/src/`

Four main components:

| Component | Responsibility | Data Flow |
|-----------|---------------|-----------|
| **Configuration Manager** | Runtime, Hardware, Layout, Keymap persistence | 100 units |
| **Device Discovery** | Linux evdev device detection & safety | 80 units |
| **Event Engine** | Key event processing & transformations | 80 units |
| **Daemon/IPC** | Background daemon & IPC protocol | 90 units |

---

### 6. **Reactive Flow (Rust → Dart → UI)**

Changes in Rust propagate back to UI:

```
Rust Core (config change)
  → FFI Bridge (callback)
  → Core Services (StreamController.add)
  → UI Wrappers (notifyListeners)
  → Riverpod Providers (rebuild)
  → UI Widgets (re-render)
```

**Volume**: 240 units (events/updates from Rust)

---

## Architecture Achievements

### ✅ Single Source of Truth (SSOT)
- All config services in `packages/core_logic/`
- Shared by CLI and UI
- No duplication

### ✅ Zero Flutter Dependencies in Core
- Core services use pure Dart (StreamController)
- CLI can use services without Flutter SDK
- Enables headless testing

### ✅ Layered Architecture
```
┌─────────────────────────────────────┐
│  Presentation (Flutter UI)          │
├─────────────────────────────────────┤
│  Adapters (UI Wrappers)             │
├─────────────────────────────────────┤
│  Business Logic (Core Services)     │  ← SSOT
├─────────────────────────────────────┤
│  Bridge (FFI Layer)                 │
├─────────────────────────────────────┤
│  Core (Rust Engine)                 │
└─────────────────────────────────────┘
```

### ✅ Dependency Injection
- `ServiceRegistry` wires all dependencies
- No globals, fully testable
- Easy to mock for tests

### ✅ CLI/UI Parity
- Dart CLI: 22 commands using core services
- Flutter UI: Full GUI using same services
- Rust CLI: Direct engine access for performance

---

## Key Design Patterns

### 1. **Wrapper Pattern**
- **Problem**: Core services use Streams, UI needs ChangeNotifier
- **Solution**: Thin UI wrappers that forward calls and bridge reactivity
- **Example**: `HardwareServiceUI` wraps `core.HardwareService`

### 2. **Repository Pattern**
- **Problem**: Decouple business logic from data persistence
- **Solution**: Repositories (e.g., `DeviceConfigRepository`) handle storage
- **Benefit**: Swappable storage backends

### 3. **Service Layer Pattern**
- **Problem**: Complex business logic in UI or Rust
- **Solution**: Dart services encapsulate config operations
- **Benefit**: Reusable across CLI and UI

### 4. **Bridge Pattern**
- **Problem**: Dart needs to call Rust safely
- **Solution**: Single `KeyrxBridge` with type-safe bindings
- **Benefit**: Centralized FFI management

---

## Data Flow Volumes (Relative)

| Flow Path | Volume | Latency | Use Case |
|-----------|--------|---------|----------|
| Rust CLI → Engine | 100 | Lowest | System ops |
| Dart CLI → Core → FFI → Engine | 150 | Low | Config mgmt |
| UI → Wrappers → Core → FFI → Engine | 200 | Medium | User interaction |
| Engine → FFI → Core → Wrappers → UI | 150 | Medium | Reactive updates |

---

## File Structure Map

```
keyrx/
├── core/                           # Rust Core Engine
│   ├── src/bin/keyrx/             # Rust CLI
│   ├── src/daemon/                # Daemon & IPC
│   ├── src/drivers/linux/         # Device discovery
│   ├── src/engine/                # Event processing
│   └── src/ffi/                   # FFI bindings (Rust side)
│
├── packages/core_logic/            # SSOT - Core Services
│   ├── lib/ffi/bridge.dart        # FFI Bridge (Dart side)
│   ├── lib/services/
│   │   ├── runtime_service.dart   # 177 lines, 0 Flutter deps
│   │   ├── hardware_service.dart  # 99 lines, 0 Flutter deps
│   │   ├── layout_service.dart    # 92 lines, 0 Flutter deps
│   │   ├── keymap_service.dart    # 67 lines, 0 Flutter deps
│   │   └── device_registry_service.dart
│   └── lib/repositories/          # Data persistence
│
├── apps/cli_tool/                  # Dart CLI
│   ├── bin/keyrx_cli.dart         # Entry point
│   └── lib/command_processor.dart # 22 commands
│
└── ui/                             # Flutter UI
    ├── lib/services/
    │   ├── service_registry.dart  # DI container
    │   ├── hardware_service_ui.dart    # Wrapper
    │   ├── layout_service_ui.dart      # Wrapper
    │   └── device_registry_service.dart # Wrapper
    └── lib/screens/               # UI screens
```

---

## Success Metrics

### Code Quality
- ✅ **Zero Flutter deps** in core services (verified with grep)
- ✅ **SSOT achieved**: All services in core_logic
- ✅ **DI pattern**: No globals, fully testable
- ✅ **Layered architecture**: Clear separation of concerns

### Functionality
- ✅ **CLI parity**: 22 commands (13 config + 9 device)
- ✅ **UI reactivity**: Full ChangeNotifier compatibility
- ✅ **Headless testing**: CLI can test without UI

### Maintainability
- ✅ **Single codebase** for business logic
- ✅ **Reusable services** across CLI and UI
- ✅ **Clear data flows** (visualized in this diagram)

---

## Migration Path (Completed 2025-12-20)

### Spec 1: move-config-services-to-core ✅
- Moved RuntimeService, KeymapService to core_logic
- Zero refactoring needed (already pure Dart)

### Spec 2: refactor-flutter-services ✅
- Refactored HardwareService, LayoutService
- ChangeNotifier → StreamController
- Created UI wrappers for ChangeNotifier compatibility

### Spec 3: extend-cli-commands ✅
- Extended CommandProcessor with 13 new config commands
- CLI now has full config management capability

**Result**: Complete architectural transformation in 3 specs (19 tasks)

---

## Conclusion

This architecture achieves:

1. **Separation of Concerns**: UI, Business Logic, Core Engine clearly separated
2. **Code Reuse**: Core services shared by CLI and UI
3. **Testability**: Headless Dart CLI enables testing without Flutter
4. **Performance**: Direct Rust CLI for low-latency operations
5. **Reactivity**: Stream-based updates propagate through all layers
6. **Type Safety**: FFI bindings generated from Rust contracts

The Sankey diagram above visualizes how **450+ units of user interaction flow through 5 architectural layers**, demonstrating a well-structured, maintainable system.
