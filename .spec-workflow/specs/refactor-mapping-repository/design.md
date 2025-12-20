# Design: Refactor Mapping Repository

**Spec Name**: refactor-mapping-repository
**Created**: 2025-12-20
**Status**: Draft

---

## Architecture Overview

### Current State

```
ui/lib/repositories/
└── mapping_repository.dart
    └── extends ChangeNotifier (Flutter dep!)

CLI: Cannot access mapping data
UI: Has exclusive access
```

### Target State

```
packages/core_logic/lib/repositories/
└── mapping_repository.dart
    └── Uses StreamController (pure Dart)

ui/lib/repositories/
└── mapping_repository_ui.dart
    └── Wrapper extends ChangeNotifier

CLI: Can access mapping data
UI: Uses wrapper for Riverpod compatibility
```

---

## Design Decisions

### Decision 1: Same Pattern as HardwareService/LayoutService

**Rationale**: Proven pattern from Spec 2 (refactor-flutter-services)

**Pattern**:
1. Refactor core to use StreamController
2. Move to core_logic
3. Create UI wrapper with ChangeNotifier
4. Update ServiceRegistry

**Precedent**: Successfully applied to HardwareService and LayoutService.

---

### Decision 2: StreamController for Reactivity

**Why StreamController?**
- ✅ Pure Dart (no Flutter deps)
- ✅ Can be listened to (reactive)
- ✅ Works in both CLI and UI
- ✅ Easy to wrap with ChangeNotifier

**Implementation**:
```dart
class MappingRepository {
  final StreamController<void> _changes = StreamController.broadcast();
  Stream<void> get changes => _changes.stream;

  void _notifyChanges() => _changes.add(null);

  void dispose() => _changes.close();
}
```

---

### Decision 3: UI Wrapper Bridges to ChangeNotifier

**Purpose**: Maintain Riverpod compatibility in UI

**Pattern**:
```dart
class MappingRepositoryUI extends ChangeNotifier {
  MappingRepositoryUI(core.MappingRepository core) {
    _subscription = core.changes.listen((_) => notifyListeners());
  }

  // Forward all methods
}
```

**Benefit**: UI code doesn't change, just uses wrapper instead.

---

## Refactoring Strategy

### Before: ChangeNotifier Pattern

```dart
import 'package:flutter/foundation.dart';

class MappingRepository extends ChangeNotifier {
  final Map<String, Mapping> _mappings = {};

  void saveMapping(Mapping mapping) {
    _mappings[mapping.id] = mapping;
    notifyListeners();  // ← Flutter-specific
  }

  List<Mapping> getMappings() => _mappings.values.toList();
}
```

### After: StreamController Pattern

```dart
// NO Flutter imports

class MappingRepository {
  final Map<String, Mapping> _mappings = {};
  final StreamController<void> _changes = StreamController.broadcast();

  Stream<void> get changes => _changes.stream;

  void saveMapping(Mapping mapping) {
    _mappings[mapping.id] = mapping;
    _changes.add(null);  // ← Pure Dart
  }

  List<Mapping> getMappings() => _mappings.values.toList();

  void dispose() => _changes.close();
}
```

---

## UI Wrapper Implementation

### MappingRepositoryUI

```dart
import 'dart:async';
import 'package:core_logic/core_logic.dart' as core;
import 'package:flutter/foundation.dart';

class MappingRepositoryUI extends ChangeNotifier {
  MappingRepositoryUI(this._core) {
    _subscription = _core.changes.listen((_) => notifyListeners());
  }

  final core.MappingRepository _core;
  StreamSubscription<void>? _subscription;

  // Forward all public methods
  void saveMapping(core.Mapping mapping) => _core.saveMapping(mapping);
  List<core.Mapping> getMappings() => _core.getMappings();
  core.Mapping? getMapping(String id) => _core.getMapping(id);
  void deleteMapping(String id) => _core.deleteMapping(id);

  @override
  void dispose() {
    _subscription?.cancel();
    _core.dispose();
    super.dispose();
  }
}
```

---

## ServiceRegistry Updates

### Before

```dart
// ui/lib/services/service_registry.dart
import '../repositories/mapping_repository.dart';

factory ServiceRegistry.real({
  MappingRepository? mappingRepository,
}) {
  final mappingRepo = mappingRepository ?? MappingRepository();

  return ServiceRegistry(
    mappingRepository: mappingRepo,
    // ...
  );
}
```

### After

```dart
// ui/lib/services/service_registry.dart
import 'package:core_logic/repositories/mapping_repository.dart' as core;
import '../repositories/mapping_repository_ui.dart';

factory ServiceRegistry.real({
  MappingRepositoryUI? mappingRepository,
}) {
  // Create core repository
  final coreMappingRepo = core.MappingRepository();

  // Create UI wrapper
  final mappingRepo = mappingRepository ?? MappingRepositoryUI(coreMappingRepo);

  return ServiceRegistry(
    mappingRepository: mappingRepo,
    // ...
  );
}
```

---

## File Structure After Migration

```
packages/core_logic/lib/
├── repositories/
│   ├── device_config_repository.dart   ✅ Existing
│   └── mapping_repository.dart         🆕 New (refactored)
└── core_logic.dart (exports)

ui/lib/repositories/
├── mapping_repository.dart             ❌ Deleted
└── mapping_repository_ui.dart          🆕 New (wrapper)
```

---

## Migration Steps

### Phase 1: Refactor (Task 1)
1. Remove ChangeNotifier from MappingRepository
2. Add StreamController pattern
3. Replace notifyListeners() with _changes.add(null)
4. Add dispose() method

### Phase 2: Move (Task 2)
5. Copy refactored repository to core_logic
6. Update imports to relative paths
7. Add export to core_logic.dart

### Phase 3: Create Wrapper (Task 3)
8. Create MappingRepositoryUI in UI
9. Implement ChangeNotifier bridge
10. Forward all methods

### Phase 4: Wire in ServiceRegistry (Task 4)
11. Update ServiceRegistry to create core repository
12. Create wrapper with core repository
13. Pass wrapper to consumers

### Phase 5: Cleanup (Task 5)
14. Delete old mapping_repository.dart
15. Verify and test

---

## Testing Strategy

### Unit Tests
- Core MappingRepository: CRUD operations
- MappingRepositoryUI: Wrapper forwarding and reactivity

### Integration Tests
- UI using MappingRepositoryUI
- Verify ChangeNotifier notifications work

### Verification
```bash
# Verify zero Flutter deps in core
grep -r "package:flutter" packages/core_logic/lib/repositories/

# Verify UI builds
cd ui && flutter build --debug
```

---

## Risk Mitigation

### Risk 1: Stream Not Triggering UI Updates

**Mitigation**: Wrapper listens to stream and calls notifyListeners()

**Test**: Verify Riverpod providers rebuild on mapping changes

### Risk 2: Missing Method Forwards

**Mitigation**: Review all public methods and forward them in wrapper

**Test**: Use IDE to find all public methods, verify all are forwarded

### Risk 3: Disposal Issues

**Mitigation**: Properly cancel subscription in wrapper dispose()

**Test**: Verify no memory leaks, streams closed properly

---

## Rollback Plan

If issues occur:
1. Revert all commits
2. Restore old mapping_repository.dart in UI
3. Remove core_logic version

---

## Success Criteria

- ✅ MappingRepository in core_logic with zero Flutter deps
- ✅ MappingRepositoryUI wrapper working in UI
- ✅ UI reactivity maintained (Riverpod providers rebuild)
- ✅ All tests passing
- ✅ 100% repository SSOT (2/2 in core_logic)
