# Tasks: Move Config Services to core_logic

## Task Overview

Move RuntimeService and KeymapService from UI to core_logic to establish SSOT for configuration management.

**Total Tasks**: 6
**Estimated Time**: 2-3 hours
**Dependencies**: Linear sequence (each task depends on previous)

---

## Task 1: Move config_result.dart to core_logic

**Status**: [-] In-progress

**Files**:
- `ui/lib/services/config_result.dart` → `packages/core_logic/lib/services/config_result.dart`

**Requirements**: REQ-1, REQ-2, REQ-4

**Description**:
Move the shared `ConfigOperationResult<T>` type to core_logic as a foundation for other services.

**Steps**:
1. Create `packages/core_logic/lib/services/` directory if it doesn't exist
2. Copy `ui/lib/services/config_result.dart` to `packages/core_logic/lib/services/config_result.dart`
3. Verify file has no imports (pure Dart class)
4. Add export to `packages/core_logic/lib/core_logic.dart`:
   ```dart
   export 'services/config_result.dart';
   ```

**Success Criteria**:
- File exists at `packages/core_logic/lib/services/config_result.dart`
- Exported from `core_logic.dart`
- No dependencies on other packages

**_Prompt**:
```
Implement the task for spec move-config-services-to-core, first run spec-workflow-guide to get the workflow guide then implement the task:

**Role**: Dart package organizer

**Task**: Move config_result.dart from UI to core_logic package.

**Context**:
- Current location: ui/lib/services/config_result.dart
- Target location: packages/core_logic/lib/services/config_result.dart
- This is a pure Dart class with no dependencies
- Needs to be exported from core_logic.dart

**Restrictions**:
- Do NOT modify the file contents
- Do NOT delete the original file yet (Task 6)
- Do NOT update imports in other files yet

**_Leverage**:
- Existing core_logic package structure
- pattern from device_registry_service.dart

**_Requirements**: REQ-1, REQ-2, REQ-4

**Success**:
- File copied to packages/core_logic/lib/services/config_result.dart
- Export added to packages/core_logic/lib/core_logic.dart
- File contents unchanged

**Instructions**:
1. Mark this task as in-progress in tasks.md (change [ ] to [-])
2. Implement the changes
3. Test your implementation
4. Log implementation using log-implementation tool with detailed artifacts
5. Mark this task as completed in tasks.md (change [-] to [x])
```

---

## Task 2: Move runtime_service.dart to core_logic

**Status**: [ ] Pending

**Files**:
- `ui/lib/services/runtime_service.dart` → `packages/core_logic/lib/services/runtime_service.dart`
- `packages/core_logic/lib/core_logic.dart` (export)

**Requirements**: REQ-1, REQ-3

**Description**:
Move RuntimeService (177 lines) to core_logic and update its imports to use relative paths.

**Steps**:
1. Copy `ui/lib/services/runtime_service.dart` to `packages/core_logic/lib/services/runtime_service.dart`
2. Update imports in the moved file:
   ```dart
   // Change package imports to relative imports
   import '../ffi/bridge.dart';
   import '../ffi/generated_bindings.dart';
   import '../models/hardware_profile.dart';
   import '../models/runtime_config.dart';
   import 'config_result.dart';
   ```
3. Add export to `packages/core_logic/lib/core_logic.dart`:
   ```dart
   export 'services/runtime_service.dart';
   ```
4. Verify no Flutter dependencies: `grep -r "package:flutter" packages/core_logic/lib/services/runtime_service.dart`

**Success Criteria**:
- File exists at `packages/core_logic/lib/services/runtime_service.dart`
- All imports use relative paths
- No Flutter dependencies
- Exported from `core_logic.dart`
- Code unchanged except imports

**_Prompt**:
```
Implement the task for spec move-config-services-to-core, first run spec-workflow-guide to get the workflow guide then implement the task:

**Role**: Dart package organizer

**Task**: Move runtime_service.dart from UI to core_logic and update imports.

**Context**:
- Current location: ui/lib/services/runtime_service.dart (177 lines)
- Target location: packages/core_logic/lib/services/runtime_service.dart
- All dependencies already exist in core_logic
- config_result.dart was moved in Task 1

**Restrictions**:
- Do NOT modify logic or functionality
- Do NOT delete original file yet (Task 6)
- Do NOT update UI imports yet (Task 4)
- Verify NO Flutter dependencies exist

**_Leverage**:
- device_registry_service.dart in core_logic as reference
- config_result.dart (just moved)

**_Requirements**: REQ-1, REQ-3

**Success**:
- File at packages/core_logic/lib/services/runtime_service.dart
- Imports updated to relative paths
- Exported from core_logic.dart
- Zero Flutter dependencies confirmed

**Instructions**:
1. Mark this task as in-progress in tasks.md (change [ ] to [-])
2. Implement the changes
3. Verify no Flutter deps: `grep -r "package:flutter" packages/core_logic/lib/services/runtime_service.dart`
4. Log implementation using log-implementation tool with detailed artifacts
5. Mark this task as completed in tasks.md (change [-] to [x])
```

---

## Task 3: Move keymap_service.dart to core_logic

**Status**: [ ] Pending

**Files**:
- `ui/lib/services/keymap_service.dart` → `packages/core_logic/lib/services/keymap_service.dart`
- `packages/core_logic/lib/core_logic.dart` (export)

**Requirements**: REQ-2, REQ-3

**Description**:
Move KeymapService (67 lines) to core_logic and update its imports.

**Steps**:
1. Copy `ui/lib/services/keymap_service.dart` to `packages/core_logic/lib/services/keymap_service.dart`
2. Update imports in the moved file:
   ```dart
   import '../ffi/bridge.dart';
   import '../ffi/generated_bindings.dart';
   import '../models/keymap.dart';
   import 'config_result.dart';
   ```
3. Add export to `packages/core_logic/lib/core_logic.dart`:
   ```dart
   export 'services/keymap_service.dart';
   ```
4. Verify no Flutter dependencies

**Success Criteria**:
- File exists at `packages/core_logic/lib/services/keymap_service.dart`
- All imports use relative paths
- No Flutter dependencies
- Exported from `core_logic.dart`

**_Prompt**:
```
Implement the task for spec move-config-services-to-core, first run spec-workflow-guide to get the workflow guide then implement the task:

**Role**: Dart package organizer

**Task**: Move keymap_service.dart from UI to core_logic and update imports.

**Context**:
- Current location: ui/lib/services/keymap_service.dart (67 lines)
- Target location: packages/core_logic/lib/services/keymap_service.dart
- All dependencies already exist in core_logic
- Similar to runtime_service.dart (Task 2)

**Restrictions**:
- Do NOT modify logic or functionality
- Do NOT delete original file yet (Task 6)
- Do NOT update UI imports yet (Task 4)
- Verify NO Flutter dependencies exist

**_Leverage**:
- runtime_service.dart (just moved in Task 2)
- device_registry_service.dart pattern

**_Requirements**: REQ-2, REQ-3

**Success**:
- File at packages/core_logic/lib/services/keymap_service.dart
- Imports updated to relative paths
- Exported from core_logic.dart
- Zero Flutter dependencies confirmed

**Instructions**:
1. Mark this task as in-progress in tasks.md (change [ ] to [-])
2. Implement the changes
3. Verify no Flutter deps: `grep -r "package:flutter" packages/core_logic/lib/services/keymap_service.dart`
4. Log implementation using log-implementation tool with detailed artifacts
5. Mark this task as completed in tasks.md (change [-] to [x])
```

---

## Task 4: Update UI imports to use core_logic

**Status**: [ ] Pending

**Files**:
- `ui/lib/services/service_registry.dart`
- Any other UI files importing these services

**Requirements**: REQ-3

**Description**:
Update all UI imports to reference services from core_logic instead of local files.

**Steps**:
1. Update `ui/lib/services/service_registry.dart`:
   ```dart
   // Remove old imports
   // import 'runtime_service.dart';
   // import 'keymap_service.dart';

   // Add new imports
   import 'package:core_logic/services/runtime_service.dart';
   import 'package:core_logic/services/keymap_service.dart';
   ```
2. Search for other files importing these services:
   ```bash
   grep -r "import.*runtime_service.dart" ui/lib/
   grep -r "import.*keymap_service.dart" ui/lib/
   ```
3. Update any found imports
4. Run `flutter pub get` in ui/
5. Run `dart analyze` to check for errors

**Success Criteria**:
- All UI imports updated to use `package:core_logic/services/...`
- No import errors
- `dart analyze` passes
- UI code compiles

**_Prompt**:
```
Implement the task for spec move-config-services-to-core, first run spec-workflow-guide to get the workflow guide then implement the task:

**Role**: Import refactoring specialist

**Task**: Update all UI imports to reference services from core_logic.

**Context**:
- Services were moved to core_logic in Tasks 2-3
- UI still imports from local files
- Need to update to package imports

**Restrictions**:
- Do NOT modify service logic
- Do NOT delete old files yet (Task 6)
- Verify all imports resolve correctly

**_Leverage**:
- device_registry_service.dart import pattern in service_registry.dart
- grep to find all import locations

**_Requirements**: REQ-3

**Success**:
- All UI files import from `package:core_logic/services/...`
- `dart analyze ui/` passes
- No broken imports
- UI compiles successfully

**Instructions**:
1. Mark this task as in-progress in tasks.md (change [ ] to [-])
2. Find all imports: `grep -r "import.*runtime_service.dart\|keymap_service.dart" ui/lib/`
3. Update all imports to use package imports
4. Run `cd ui && flutter pub get && dart analyze`
5. Log implementation using log-implementation tool with detailed artifacts
6. Mark this task as completed in tasks.md (change [-] to [x])
```

---

## Task 5: Run tests and verify functionality

**Status**: [ ] Pending

**Files**: None (validation only)

**Requirements**: REQ-3, NFR-2, NFR-3

**Description**:
Run all UI tests to verify no regressions occurred during migration.

**Steps**:
1. Run UI tests: `cd ui && flutter test`
2. Run analysis: `dart analyze ui/`
3. Check metrics: `dart run ui/scripts/check_metrics.dart`
4. Verify services work in UI by running app briefly

**Success Criteria**:
- All UI tests pass (100%)
- No analyzer errors
- Code metrics pass
- UI app runs without errors

**_Prompt**:
```
Implement the task for spec move-config-services-to-core, first run spec-workflow-guide to get the workflow guide then implement the task:

**Role**: QA engineer

**Task**: Verify all tests pass and functionality is preserved.

**Context**:
- Services moved to core_logic
- UI imports updated
- Need to verify no regressions

**Restrictions**:
- Do NOT modify code to make tests pass
- If tests fail, investigate root cause
- Report any issues found

**_Leverage**:
- Existing test suite
- Code metrics scripts

**_Requirements**: REQ-3, NFR-2, NFR-3

**Success**:
- All UI tests pass
- dart analyze passes
- Code metrics pass
- No functional regressions

**Instructions**:
1. Mark this task as in-progress in tasks.md (change [ ] to [-])
2. Run: `cd ui && flutter test`
3. Run: `cd ui && dart analyze`
4. Run: `dart run ui/scripts/check_metrics.dart`
5. If any failures, investigate and fix
6. Log results using log-implementation tool
7. Mark this task as completed in tasks.md (change [-] to [x])
```

---

## Task 6: Delete old service files from UI

**Status**: [ ] Pending

**Files**:
- `ui/lib/services/runtime_service.dart` (delete)
- `ui/lib/services/keymap_service.dart` (delete)
- `ui/lib/services/config_result.dart` (delete)

**Requirements**: REQ-1, REQ-2

**Description**:
Delete the original service files from UI now that they've been moved to core_logic.

**Steps**:
1. Verify all imports are updated (Task 4 completed)
2. Verify all tests pass (Task 5 completed)
3. Delete old files:
   ```bash
   rm ui/lib/services/runtime_service.dart
   rm ui/lib/services/keymap_service.dart
   rm ui/lib/services/config_result.dart
   ```
4. Run tests again to ensure nothing broke
5. Commit changes

**Success Criteria**:
- Old files deleted
- No references to old files remain
- Tests still pass
- Code compiles

**_Prompt**:
```
Implement the task for spec move-config-services-to-core, first run spec-workflow-guide to get the workflow guide then implement the task:

**Role**: Cleanup specialist

**Task**: Delete old service files from UI after migration.

**Context**:
- Services successfully moved to core_logic
- UI imports updated and tested
- Old files are now duplicates

**Restrictions**:
- Only delete after Tasks 4 and 5 are complete
- Verify no references remain before deleting
- Run tests after deletion

**_Leverage**:
- grep to verify no imports remain

**_Requirements**: REQ-1, REQ-2

**Success**:
- Old files deleted
- No broken references
- Tests still pass
- Clean git status

**Instructions**:
1. Mark this task as in-progress in tasks.md (change [ ] to [-])
2. Verify no imports: `grep -r "services/runtime_service.dart\|services/keymap_service.dart\|services/config_result.dart" ui/lib/ --exclude-dir=generated`
3. Delete files: `rm ui/lib/services/{runtime_service,keymap_service,config_result}.dart`
4. Run tests: `cd ui && flutter test`
5. Log implementation using log-implementation tool
6. Mark this task as completed in tasks.md (change [-] to [x])
```

---

## Summary

**Task Dependencies**:
```
Task 1 (config_result)
  ↓
Task 2 (runtime_service) ──┐
  ↓                        │
Task 3 (keymap_service) ───┤
  ↓                        │
Task 4 (update UI imports) ←┘
  ↓
Task 5 (run tests)
  ↓
Task 6 (delete old files)
```

**Completion Checklist**:
- [ ] All files moved to core_logic
- [ ] All files exported from core_logic.dart
- [ ] All UI imports updated
- [ ] All tests pass
- [ ] Code metrics pass
- [ ] Old files deleted
- [ ] Changes committed
