# Requirements: Refactor Flutter Services for core_logic

## Overview

Refactor HardwareService and LayoutService to remove Flutter dependencies (ChangeNotifier), then move them to `packages/core_logic/lib/services/` to establish SSOT. Create thin UI wrappers to maintain Riverpod compatibility.

## Context

**Problem**: HardwareService and LayoutService use `ChangeNotifier` (Flutter-specific), preventing them from being shared with CLI. This violates SSOT and limits headless testing.

**Root Cause**: Services were written with Flutter UI patterns (ChangeNotifier) instead of pure Dart streams.

**Impact**:
- CLI cannot manage hardware profiles
- CLI cannot manage layouts
- Services tied to Flutter engine
- Testing requires full Flutter instead of bare Dart VM

**Success Criteria**:
- Services refactored to use StreamController instead of ChangeNotifier
- Services moved to `core_logic` with zero Flutter dependencies
- UI wrappers provide ChangeNotifier interface for Riverpod compatibility
- All tests pass
- No behavioral changes in UI

## User Stories

### REQ-1: HardwareService in core_logic
**As a** CLI developer
**I want** HardwareService available in core_logic without Flutter dependencies
**So that** CLI can manage hardware profiles

**Acceptance Criteria**:
- Core service uses `StreamController<void>` instead of `ChangeNotifier`
- Core service moved to `packages/core_logic/lib/services/hardware_service.dart`
- UI wrapper at `ui/lib/services/hardware_service_ui.dart` exposes ChangeNotifier
- All existing UI code continues to work
- CLI can import core service

**EARS Format**:
- **WHEN** service needs to notify listeners of changes
- **THEN** it emits events via StreamController
- **AND** UI wrapper converts stream events to ChangeNotifier notifications

### REQ-2: LayoutService in core_logic
**As a** CLI developer
**I want** LayoutService available in core_logic without Flutter dependencies
**So that** CLI can manage virtual layouts

**Acceptance Criteria**:
- Core service uses `StreamController<void>` instead of `ChangeNotifier`
- Core service moved to `packages/core_logic/lib/services/layout_service.dart`
- UI wrapper at `ui/lib/services/layout_service_ui.dart` exposes ChangeNotifier
- All existing UI code continues to work
- CLI can import core service

**EARS Format**:
- **WHEN** layouts are added/updated/deleted
- **THEN** core service emits stream event
- **AND** UI wrapper triggers notifyListeners()

### REQ-3: Stream-based reactivity
**As a** service developer
**I want** services to use Dart streams for change notification
**So that** they work in both CLI and UI contexts

**Acceptance Criteria**:
- Services expose `Stream<void> get changes`
- Stream emits event when data changes
- No dependency on Flutter packages

**EARS Format**:
- **WHEN** service data changes
- **THEN** stream emits null event
- **AND** listeners can react accordingly

### REQ-4: UI wrapper pattern
**As a** UI developer
**I want** ChangeNotifier wrappers for core services
**So that** Riverpod providers continue to work

**Acceptance Criteria**:
- UI wrappers extend ChangeNotifier
- Wrappers forward method calls to core service
- Wrappers listen to core service stream and call notifyListeners()
- ServiceRegistry uses wrappers for UI compatibility

**EARS Format**:
- **WHEN** UI wrapper is created
- **THEN** it subscribes to core service stream
- **AND** calls notifyListeners() on each event

### REQ-5: Backward compatibility
**As a** UI developer
**I want** existing UI code to work without changes
**So that** migration is seamless

**Acceptance Criteria**:
- ServiceRegistry interface unchanged
- Riverpod providers work identically
- All UI tests pass
- No behavioral changes

**EARS Format**:
- **WHEN** UI initializes services
- **THEN** it uses wrappers that expose same interface
- **AND** all functionality works identically

## Non-Functional Requirements

### NFR-1: Zero Flutter Dependencies in Core
**Performance**: Core services must run on bare Dart VM
**Rationale**: Enables CLI usage without Flutter engine
**Measurement**:
- `flutter pub deps` shows no Flutter in core service files
- Services import only `dart:*` and `package:core_logic/*`

### NFR-2: Stream Performance
**Performance**: Stream overhead must be negligible
**Rationale**: Changes are infrequent (user actions)
**Measurement**: No measurable latency increase in UI

### NFR-3: Memory Management
**Resource**: Streams must be properly disposed
**Rationale**: Prevent memory leaks
**Measurement**:
- dispose() closes StreamController
- UI wrapper unsubscribes on disposal

### NFR-4: Code Quality
**Maintainability**: Follow existing standards
**Rationale**: Maintain codebase quality
**Measurement**:
- ≤500 lines per file
- ≤50 lines per function
- 80% test coverage minimum

## Dependencies

### Upstream Dependencies
- `core_logic` package structure exists
- `KeyrxBridge` available in core_logic
- Models (HardwareProfile, VirtualLayout) in core_logic
- ConfigOperationResult type in core_logic (from move-config-services-to-core)

### Downstream Impact
- UI ServiceRegistry must use wrappers
- Riverpod providers reference UI wrappers
- CLI CommandProcessor can add new commands (future)

## Technical Constraints

### Refactoring Pattern

**Before** (Flutter-dependent):
```dart
class HardwareService with ChangeNotifier {
  Future<ConfigOperationResult<HardwareProfile>> saveProfile(...) async {
    final result = await _guard(...);
    if (!result.hasError) {
      notifyListeners();  // Flutter-specific
    }
    return result;
  }
}
```

**After** (Pure Dart core + UI wrapper):
```dart
// packages/core_logic/lib/services/hardware_service.dart
class HardwareService {
  final StreamController<void> _changesController = StreamController.broadcast();
  Stream<void> get changes => _changesController.stream;

  Future<ConfigOperationResult<HardwareProfile>> saveProfile(...) async {
    final result = await _guard(...);
    if (!result.hasError) {
      _changesController.add(null);  // Pure Dart
    }
    return result;
  }

  void dispose() {
    _changesController.close();
  }
}

// ui/lib/services/hardware_service_ui.dart
class HardwareServiceUI extends ChangeNotifier {
  HardwareServiceUI(this._core) {
    _subscription = _core.changes.listen((_) => notifyListeners());
  }

  final HardwareService _core;
  StreamSubscription<void>? _subscription;

  // Forward all methods
  Future<ConfigOperationResult<List<HardwareProfile>>> listProfiles() =>
      _core.listProfiles();

  @override
  void dispose() {
    _subscription?.cancel();
    super.dispose();
  }
}
```

## Success Metrics

- ✅ Core services in `core_logic` with zero Flutter deps
- ✅ UI wrappers in `ui/lib/services/` provide ChangeNotifier interface
- ✅ All UI tests pass (100%)
- ✅ Stream-based reactivity working
- ✅ Memory leaks prevented (streams disposed)
- ✅ Code metrics maintained

## Out of Scope

- Adding new service methods
- Changing FFI bindings
- Performance optimizations
- Adding CLI commands (separate spec: extend-cli-commands)
- Refactoring other services (only Hardware and Layout)
