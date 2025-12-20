# Design: Move Utilities to Core

**Spec Name**: move-utilities-to-core
**Created**: 2025-12-20
**Status**: Draft

---

## Architecture Overview

### Current State

```
ui/lib/services/
├── error_translator.dart           (0 Flutter deps)
├── error_translator_impl.dart      (0 Flutter deps)
└── storage_path_resolver.dart      (0 Flutter deps)

CLI: Cannot use error translation or path resolution
UI: Has exclusive access
```

### Target State

```
packages/core_logic/lib/utils/
├── error_translator.dart           ← Moved
├── error_translator_impl.dart      ← Moved
└── storage_path_resolver.dart      ← Moved

CLI: Shared error messages and paths
UI: Imports from core_logic
```

---

## Design Decisions

### Decision 1: Simple Move (No Refactoring)

**Rationale**: Both utilities already have zero Flutter dependencies.

**Approach**: Copy files to core_logic/utils/, update imports, delete originals.

**Pattern**: Same as ProfileRegistryService/TestService (Spec: move-profile-test-services)

---

### Decision 2: Move to utils/ Directory

**Rationale**: These are utilities, not services or repositories.

**Directory Structure**:
```
packages/core_logic/lib/
├── utils/
│   ├── logger.dart                 ✅ Existing
│   ├── error_translator.dart       🆕 New
│   ├── error_translator_impl.dart  🆕 New
│   └── storage_path_resolver.dart  🆕 New
```

---

### Decision 3: Maintain Interface/Implementation Split

**ErrorTranslator**:
- `error_translator.dart` - Interface (abstract class)
- `error_translator_impl.dart` - Implementation

**Benefit**: Testability (can mock interface)

---

## Utility Details

### ErrorTranslator

**Responsibility**: Translate error codes to user-friendly messages

**Interface**:
```dart
abstract class ErrorTranslator {
  String translate(String errorCode);
  String? tryTranslate(String? errorCode);
}
```

**Implementation**:
```dart
class ErrorTranslatorImpl implements ErrorTranslator {
  const ErrorTranslatorImpl();

  @override
  String translate(String errorCode) {
    // Error code → Message mapping
  }
}
```

**Dependencies**: None (pure Dart)

**Move Strategy**: Direct copy to core_logic/utils/

---

### StoragePathResolver

**Responsibility**: Resolve storage paths for config files

**Key Methods**:
```dart
class StoragePathResolver {
  StoragePathResolver({required KeyrxBridge bridge});

  String getConfigRoot();
  String getProfilePath(String profileId);
  String getDeviceConfigPath();
}
```

**Dependencies**: KeyrxBridge (already in core_logic)

**Move Strategy**: Direct copy to core_logic/utils/

---

## Import Updates

### Before (UI)

```dart
// ui/lib/services/service_registry.dart
import 'error_translator.dart';
import 'error_translator_impl.dart';
import 'storage_path_resolver.dart';
```

### After (UI)

```dart
// ui/lib/services/service_registry.dart
import 'package:core_logic/utils/error_translator.dart';
import 'package:core_logic/utils/error_translator_impl.dart';
import 'package:core_logic/utils/storage_path_resolver.dart';
```

### CLI (New)

```dart
// apps/cli_tool/lib/command_processor.dart
import 'package:core_logic/utils/error_translator.dart';
import 'package:core_logic/utils/storage_path_resolver.dart';
```

---

## File Structure After Migration

```
packages/core_logic/lib/
├── utils/
│   ├── logger.dart                     ✅ Existing
│   ├── error_translator.dart           🆕 New
│   ├── error_translator_impl.dart      🆕 New
│   └── storage_path_resolver.dart      🆕 New
└── core_logic.dart (exports)

ui/lib/services/
├── error_translator.dart               ❌ Deleted
├── error_translator_impl.dart          ❌ Deleted
└── storage_path_resolver.dart          ❌ Deleted
```

---

## Migration Steps

### Phase 1: Move Utilities (Tasks 1-2)
1. Copy ErrorTranslator files to core_logic/utils/
2. Copy StoragePathResolver to core_logic/utils/

### Phase 2: Update Exports & Imports (Task 3)
3. Add exports to core_logic.dart
4. Update UI imports

### Phase 3: Cleanup & Test (Task 4)
5. Delete old UI files
6. Run tests and verify

---

## CLI Use Cases

### Use Case 1: Consistent Error Messages

**Before**: CLI errors are raw strings
**After**: CLI uses same error translator as UI

```dart
// CLI command handler
try {
  await service.operation();
} catch (e) {
  final message = errorTranslator.translate(e.code);
  print(jsonEncode({'error': message}));
}
```

### Use Case 2: Shared Path Resolution

**Before**: CLI hardcodes paths
**After**: CLI uses same path resolver as UI

```dart
// CLI uses shared path logic
final configPath = pathResolver.getConfigRoot();
final profilePath = pathResolver.getProfilePath(profileId);
```

---

## Testing Strategy

### Unit Tests
- ErrorTranslatorImpl: Error code translations
- StoragePathResolver: Path resolution logic

### Integration Tests
- UI using utilities from core_logic
- CLI using utilities from core_logic

### Verification
```bash
# Verify zero Flutter deps
grep -r "package:flutter" packages/core_logic/lib/utils/

# Verify UI builds
cd ui && flutter build --debug

# Verify CLI compiles
cd apps/cli_tool && dart compile exe bin/keyrx_cli.dart
```

---

## Risk Mitigation

### Risk 1: Path Resolution Breaking

**Mitigation**: Thoroughly test path resolution after move

**Test**: Verify config files load correctly in both CLI and UI

### Risk 2: Missing Error Translations

**Mitigation**: Verify all error codes still translate

**Test**: Test error handling in both CLI and UI

---

## Rollback Plan

If issues occur:
1. Revert commits (all changes in git)
2. Restore old UI utility files
3. Remove core_logic versions

---

## Success Criteria

- ✅ All 2 utilities in core_logic/utils/
- ✅ Zero Flutter dependencies (grep verified)
- ✅ UI builds and runs
- ✅ CLI compiles and runs
- ✅ All tests passing
- ✅ Error messages consistent between CLI and UI
