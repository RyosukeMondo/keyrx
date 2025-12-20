# Requirements: Refactor Mapping Repository

**Spec Name**: refactor-mapping-repository
**Created**: 2025-12-20
**Status**: Draft
**Priority**: Medium (SSOT completion)

---

## Overview

Refactor MappingRepository to remove Flutter dependencies (ChangeNotifier → StreamController pattern), move to core_logic, and create UI wrapper for Riverpod compatibility.

---

## Business Goals

1. Complete SSOT for repository layer (100% repositories in core_logic)
2. Enable CLI access to mapping/keyflow data
3. Remove Flutter dependencies from business logic
4. Maintain UI reactivity via wrapper pattern

---

## Requirements

### REQ-1: Refactor MappingRepository to Remove Flutter Dependencies

**Description**: Replace ChangeNotifier with StreamController pattern.

**Current State**:
```dart
// ui/lib/repositories/mapping_repository.dart
import 'package:flutter/foundation.dart';

class MappingRepository extends ChangeNotifier {
  void saveMapping() {
    // ...
    notifyListeners();
  }
}
```

**Target State**:
```dart
// packages/core_logic/lib/repositories/mapping_repository.dart
// NO Flutter imports

class MappingRepository {
  final StreamController<void> _changes = StreamController.broadcast();
  Stream<void> get changes => _changes.stream;

  void saveMapping() {
    // ...
    _changes.add(null);
  }

  void dispose() => _changes.close();
}
```

**Acceptance Criteria**:
- ✅ Removed `extends ChangeNotifier`
- ✅ Added `StreamController<void>`
- ✅ All `notifyListeners()` → `_changes.add(null)`
- ✅ Added `dispose()` method
- ✅ Zero Flutter dependencies verified

---

### REQ-2: Move MappingRepository to Core Logic

**Description**: Move refactored repository to core_logic package.

**Target State**:
- Location: `packages/core_logic/lib/repositories/mapping_repository.dart`
- Exported from `core_logic.dart`

**Acceptance Criteria**:
- ✅ File in core_logic/repositories/
- ✅ Imports use relative paths
- ✅ Exported from core_logic.dart
- ✅ Zero Flutter dependencies (grep verified)

---

### REQ-3: Create MappingRepositoryUI Wrapper

**Description**: Create ChangeNotifier wrapper for Riverpod compatibility.

**Pattern** (same as HardwareServiceUI):
```dart
// ui/lib/repositories/mapping_repository_ui.dart
import 'package:flutter/foundation.dart';
import 'package:core_logic/core_logic.dart' as core;

class MappingRepositoryUI extends ChangeNotifier {
  MappingRepositoryUI(this._core) {
    _subscription = _core.changes.listen((_) => notifyListeners());
  }

  final core.MappingRepository _core;
  StreamSubscription<void>? _subscription;

  // Forward all methods
  void saveMapping(...) => _core.saveMapping(...);

  @override
  void dispose() {
    _subscription?.cancel();
    _core.dispose();
    super.dispose();
  }
}
```

**Acceptance Criteria**:
- ✅ Extends ChangeNotifier
- ✅ Subscribes to core.changes stream
- ✅ Forwards all methods to core repository
- ✅ Proper disposal (cancels subscription)

---

### REQ-4: Update ServiceRegistry to Use Wrapper

**Description**: Wire core repository and UI wrapper in dependency injection.

**Pattern**:
```dart
// Create core repository
final coreMappingRepo = core.MappingRepository();

// Create UI wrapper
final mappingRepo = mappingRepository ?? MappingRepositoryUI(coreMappingRepo);
```

**Acceptance Criteria**:
- ✅ Core repository created first
- ✅ UI wrapper wraps core repository
- ✅ ServiceRegistry passes wrapper to consumers
- ✅ Disposal chain works correctly

---

### REQ-5: Verify and Test

**Description**: Ensure repository works in both CLI and UI.

**Acceptance Criteria**:
- ✅ UI tests pass
- ✅ Mapping operations work in UI
- ✅ No Flutter dependencies in core_logic
- ✅ No regressions

---

## Non-Functional Requirements

### NFR-1: Zero Flutter Dependencies

Core MappingRepository must have zero Flutter dependencies.

### NFR-2: Backward Compatibility

UI functionality must remain unchanged.

### NFR-3: Code Quality

All code must pass pre-commit checks.

---

## Success Metrics

- ✅ MappingRepository in core_logic
- ✅ Zero Flutter deps verified
- ✅ UI wrapper working
- ✅ 100% repository SSOT achieved (2/2 repositories in core)
- ✅ All tests passing

---

## Out of Scope

- Adding CLI commands for mapping (separate spec if needed)
- Refactoring mapping logic
- Adding new mapping features
