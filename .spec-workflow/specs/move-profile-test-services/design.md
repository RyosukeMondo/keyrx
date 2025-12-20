# Design: Move Profile & Test Services to Core

**Spec Name**: move-profile-test-services
**Created**: 2025-12-20
**Status**: Draft

---

## Architecture Overview

### Current State

```
ui/lib/services/
├── profile_registry_service.dart   (0 Flutter deps)
├── test_service.dart                (0 Flutter deps)
└── script_file_service.dart         (0 Flutter deps)

CLI: Cannot access profiles or tests
UI: Has exclusive access
```

### Target State

```
packages/core_logic/lib/services/
├── profile_registry_service.dart   ← Moved
├── test_service.dart                ← Moved
└── script_file_service.dart         ← Moved

CLI: Full access to profiles and tests
UI: Imports from core_logic
```

---

## Design Decisions

### Decision 1: Simple Move (No Refactoring)

**Rationale**: All three services already have zero Flutter dependencies.

**Approach**: Copy files to core_logic, update imports, delete originals.

**Pattern**: Same as RuntimeService/KeymapService (Spec 1: move-config-services-to-core)

---

### Decision 2: No UI Wrappers Needed

**Rationale**: Services don't use ChangeNotifier or Flutter-specific features.

**Comparison**:
- HardwareService/LayoutService: Used ChangeNotifier → Needed wrappers
- ProfileRegistryService/TestService: Pure Dart → No wrappers needed

---

### Decision 3: CLI Commands Follow JSON Protocol

**Format**:
```json
// Request
{"command": "listProfiles", "id": 1}

// Response
{"id": 1, "result": [...]}

// Error
{"id": 1, "error": "Profile not found"}
```

**Consistency**: Same protocol as existing 22 CLI commands.

---

## Service Details

### ProfileRegistryService

**Responsibility**: Manage device profiles (CRUD operations)

**Key Methods**:
```dart
Future<List<Profile>> listProfiles()
Future<Profile?> getProfile(String id)
Future<void> saveProfile(Profile profile)
Future<void> deleteProfile(String id)
Future<List<Profile>> findCompatibleProfiles(String deviceId)
```

**Dependencies**: KeyrxBridge (FFI)

**Move Strategy**: Direct copy to core_logic, update imports to use relative paths.

---

### TestService

**Responsibility**: Script testing and validation

**Key Methods**:
```dart
Future<TestResult> testScript(String path)
Future<ValidationResult> validateScript(String content)
Future<List<TestResult>> runTests(String pattern)
```

**Dependencies**: KeyrxBridge (FFI)

**Move Strategy**: Direct copy to core_logic, update imports.

---

### ScriptFileService

**Responsibility**: Script file I/O operations

**Key Methods**:
```dart
Future<String> readScript(String path)
Future<void> writeScript(String path, String content)
Future<List<String>> listScripts()
```

**Dependencies**: Dart I/O only (no FFI)

**Move Strategy**: Direct copy to core_logic.

---

## Import Updates

### Before (UI)

```dart
// ui/lib/services/service_registry.dart
import 'profile_registry_service.dart';
import 'test_service.dart';
import 'script_file_service.dart';
```

### After (UI)

```dart
// ui/lib/services/service_registry.dart
import 'package:core_logic/services/profile_registry_service.dart';
import 'package:core_logic/services/test_service.dart';
import 'package:core_logic/services/script_file_service.dart';
```

### CLI (New)

```dart
// apps/cli_tool/bin/keyrx_cli.dart
import 'package:core_logic/services/profile_registry_service.dart';
import 'package:core_logic/services/test_service.dart';
import 'package:core_logic/services/script_file_service.dart';
```

---

## CLI Command Mapping

| Command | Service | Method |
|---------|---------|--------|
| `listProfiles` | ProfileRegistryService | `listProfiles()` |
| `getProfile` | ProfileRegistryService | `getProfile(id)` |
| `saveProfile` | ProfileRegistryService | `saveProfile(profile)` |
| `deleteProfile` | ProfileRegistryService | `deleteProfile(id)` |
| `findCompatibleProfiles` | ProfileRegistryService | `findCompatibleProfiles(deviceId)` |
| `testScript` | TestService | `testScript(path)` |
| `validateScript` | TestService | `validateScript(content)` |
| `runTests` | TestService | `runTests(pattern)` |

---

## File Structure After Migration

```
packages/core_logic/lib/
├── services/
│   ├── config_result.dart              ✅ Existing
│   ├── device_registry_service.dart    ✅ Existing
│   ├── runtime_service.dart            ✅ Existing
│   ├── hardware_service.dart           ✅ Existing
│   ├── layout_service.dart             ✅ Existing
│   ├── keymap_service.dart             ✅ Existing
│   ├── profile_registry_service.dart   🆕 New
│   ├── test_service.dart               🆕 New
│   └── script_file_service.dart        🆕 New
└── core_logic.dart (exports)

ui/lib/services/
├── profile_registry_service.dart       ❌ Deleted
├── test_service.dart                   ❌ Deleted
└── script_file_service.dart            ❌ Deleted
```

---

## Migration Steps

### Phase 1: Move Services (Tasks 1-3)
1. Copy ProfileRegistryService to core_logic
2. Copy TestService to core_logic
3. Copy ScriptFileService to core_logic

### Phase 2: Update Exports & Imports (Task 4)
4. Add exports to core_logic.dart
5. Update UI imports

### Phase 3: CLI Commands (Task 5)
6. Add CommandProcessor parameters
7. Implement 8 new CLI commands

### Phase 4: Cleanup & Test (Task 6-7)
8. Delete old UI files
9. Run tests and verify

---

## Testing Strategy

### Unit Tests
- ProfileRegistryService: CRUD operations
- TestService: Script validation and testing
- ScriptFileService: File I/O

### Integration Tests
- CLI commands with JSON protocol
- UI using services from core_logic

### Verification
```bash
# Verify zero Flutter deps
grep -r "package:flutter" packages/core_logic/lib/services/

# Verify CLI works
echo '{"command":"listProfiles","id":1}' | dart run apps/cli_tool/bin/keyrx_cli.dart

# Verify UI builds
cd ui && flutter build --debug
```

---

## Risk Mitigation

### Risk 1: Import Path Issues

**Mitigation**: Use relative imports in core_logic, package imports in UI/CLI.

### Risk 2: Missing Dependencies

**Mitigation**: Verify all dependencies exist in core_logic before moving.

### Risk 3: Breaking UI

**Mitigation**: Update imports before deleting old files.

---

## Rollback Plan

If issues occur:
1. Revert commits (all changes in git)
2. Restore old UI service files
3. Remove core_logic exports

---

## Success Criteria

- ✅ All 3 services in core_logic
- ✅ Zero Flutter dependencies (grep verified)
- ✅ 8 CLI commands working
- ✅ UI builds and runs
- ✅ All tests passing
- ✅ No regressions
