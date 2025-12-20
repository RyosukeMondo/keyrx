# SSOT Architecture Analysis

**Date**: 2025-12-20
**Current State**: Post-migration (3 specs completed)
**Objective**: Identify remaining SSOT gaps and improvement opportunities

---

## Current SSOT Status

### ✅ Achieved (Complete)

**Config Services** - All in `packages/core_logic/lib/services/`:
- ✅ `RuntimeService` (177 lines, 0 Flutter deps)
- ✅ `HardwareService` (99 lines, 0 Flutter deps)
- ✅ `LayoutService` (92 lines, 0 Flutter deps)
- ✅ `KeymapService` (67 lines, 0 Flutter deps)
- ✅ `DeviceRegistryService` (350+ lines, 0 Flutter deps)

**Repositories** - In `packages/core_logic/lib/repositories/`:
- ✅ `DeviceConfigRepository` (shared by CLI and UI)

**Result**: CLI can perform full configuration management without UI.

---

## Identified SSOT Gaps

### 🔶 Gap 1: Service Layer Fragmentation

**Current State**: 45 services in `ui/lib/services/`, only 5 moved to `core_logic`

**Analysis**:
```bash
Total UI services:        45 files
With Flutter deps:         8 files (18%)
Without Flutter deps:     37 files (82%)
```

**Services that COULD be shared** (zero Flutter deps):

| Service | Purpose | CLI Benefit | Priority |
|---------|---------|-------------|----------|
| `ProfileRegistryService` | Device profile CRUD | CLI profile management | 🔴 High |
| `TestService` | Script testing & validation | CLI test runner | 🔴 High |
| `ScriptFileService` | Script file I/O | CLI script commands | 🟡 Medium |
| `ErrorTranslatorImpl` | Error message translation | Consistent errors | 🟡 Medium |
| `StoragePathResolver` | Config path resolution | Shared paths | 🟢 Low |

**Current Limitation**: CLI cannot manage device profiles or run tests (UI-only features).

---

### 🔶 Gap 2: Repository Layer Not Fully Shared

**Current State**: Only 1 repository in `core_logic/repositories/`

**Repositories still in UI** (`ui/lib/repositories/`):
- `mapping_repository.dart` - Has `package:flutter/foundation.dart` dependency

**Issue**: MappingRepository uses `ChangeNotifier` (Flutter-specific)

**Impact**:
- CLI cannot access mapping/keyflow data
- Duplication risk if CLI needs this functionality

**Root Cause**: Same as HardwareService/LayoutService (needs ChangeNotifier → StreamController refactor)

---

### 🔶 Gap 3: CLI Feature Parity

**What CLI can do**:
- ✅ Device registry management (9 commands)
- ✅ Runtime config (4 commands)
- ✅ Hardware profiles (3 commands)
- ✅ Layouts (3 commands)
- ✅ Keymaps (3 commands)

**What CLI CANNOT do** (UI-only):
- ❌ Device profile management (ProfileRegistryService in UI)
- ❌ Script testing (TestService in UI)
- ❌ Script file operations (ScriptFileService in UI)
- ❌ Mapping/keyflow operations (MappingRepository in UI)
- ❌ Benchmarking (BenchmarkService in UI)
- ❌ Session recording/replay (SessionService in UI)

**Potential Use Cases**:
- **CI/CD**: Run tests without UI
- **Scripting**: Automate profile/script management
- **Server Deployment**: Headless configuration

---

## Recommended Improvements

### 🎯 Recommendation 1: Move Profile & Test Services (High Priority)

**Spec**: `move-profile-test-services-to-core`

**Services to Move**:
1. `ProfileRegistryService` → `packages/core_logic/lib/services/`
2. `TestService` → `packages/core_logic/lib/services/`
3. `ScriptFileService` → `packages/core_logic/lib/services/`

**Benefits**:
- ✅ CLI can manage device profiles
- ✅ CLI can run script tests (headless testing)
- ✅ CI/CD integration possible
- ✅ Server-side profile management

**Effort**: Low (already zero Flutter deps)

**CLI Commands to Add**:
```bash
# Profile management
keyrx-cli listProfiles
keyrx-cli getProfile <id>
keyrx-cli saveProfile <json>
keyrx-cli deleteProfile <id>

# Testing
keyrx-cli testScript <file>
keyrx-cli validateScript <file>
keyrx-cli runTests <pattern>
```

---

### 🎯 Recommendation 2: Refactor MappingRepository (Medium Priority)

**Spec**: `refactor-mapping-repository`

**Current Issue**:
```dart
// ui/lib/repositories/mapping_repository.dart
import 'package:flutter/foundation.dart';  // ← Flutter dep

class MappingRepository extends ChangeNotifier {
  // ...
}
```

**Solution** (same pattern as HardwareService):
1. Remove `ChangeNotifier`, add `StreamController<void>`
2. Move to `packages/core_logic/lib/repositories/`
3. Create `MappingRepositoryUI` wrapper in UI
4. Update ServiceRegistry

**Benefits**:
- ✅ CLI can access mapping data
- ✅ Keyflow operations in CLI
- ✅ Complete repository layer in core_logic

**Effort**: Medium (requires refactoring, same as Spec 2)

---

### 🎯 Recommendation 3: Move Utilities to Core (Low Priority)

**Spec**: `move-utilities-to-core`

**Services to Move**:
1. `ErrorTranslatorImpl` → `packages/core_logic/lib/utils/`
2. `StoragePathResolver` → `packages/core_logic/lib/utils/`

**Benefits**:
- ✅ Consistent error messages in CLI
- ✅ Shared path resolution logic
- ✅ Reduced duplication

**Effort**: Low

---

### 🎯 Recommendation 4: Audit UI-Specific Services

**Goal**: Clearly separate UI-only services from potentially shared ones

**UI-Only Services** (should stay in UI):
- ✅ `ScreenReaderAnnouncer` - Accessibility (Flutter)
- ✅ `KeyboardShortcuts` - UI shortcuts (Flutter)
- ✅ `ProfileAutosaveService` - UI feature
- ✅ `DaemonDiscoveryService` - UI-specific
- ✅ `FlowValidator` - UI validation
- ✅ `KeyflowAnalytics` - UI analytics

**Potentially Shared** (need investigation):
- 🔶 `BenchmarkService` - Could be CLI tool
- 🔶 `SessionService` - Could be CLI for recording
- 🔶 `MetricsService` - Could be shared observability
- 🔶 `ObservabilityService` - Should be shared

---

## Architecture Goals

### Target State

```
packages/core_logic/
├── services/
│   ├── config_result.dart              ✅ Done
│   ├── device_registry_service.dart    ✅ Done
│   ├── runtime_service.dart            ✅ Done
│   ├── hardware_service.dart           ✅ Done
│   ├── layout_service.dart             ✅ Done
│   ├── keymap_service.dart             ✅ Done
│   ├── profile_registry_service.dart   🔴 TODO (Rec 1)
│   ├── test_service.dart               🔴 TODO (Rec 1)
│   ├── script_file_service.dart        🔴 TODO (Rec 1)
│   └── observability_service.dart      🟡 TODO (Rec 4)
├── repositories/
│   ├── device_config_repository.dart   ✅ Done
│   └── mapping_repository.dart         🟡 TODO (Rec 2)
└── utils/
    ├── logger.dart                     ✅ Done
    ├── error_translator.dart           🟡 TODO (Rec 3)
    └── storage_path_resolver.dart      🟡 TODO (Rec 3)
```

### UI Wrappers (for ChangeNotifier compatibility)

```
ui/lib/services/
├── hardware_service_ui.dart           ✅ Done
├── layout_service_ui.dart             ✅ Done
├── device_registry_service_ui.dart    ✅ Done (as device_registry_service.dart)
├── profile_registry_service_ui.dart   🔴 TODO (Rec 1)
└── mapping_repository_ui.dart         🟡 TODO (Rec 2)
```

---

## Priority Matrix

| Recommendation | Impact | Effort | Priority | Next Spec? |
|----------------|--------|--------|----------|------------|
| 1. Profile & Test Services | High | Low | 🔴 **High** | Yes |
| 2. MappingRepository | Medium | Medium | 🟡 Medium | Maybe |
| 3. Utilities | Low | Low | 🟢 Low | No |
| 4. Service Audit | Medium | High | 🟡 Medium | No |

---

## Implementation Sequence

### Phase 1: Quick Wins (Rec 1)
**Goal**: Enable CLI profile and test management
**Effort**: 1-2 days
**Spec**: `move-profile-test-services-to-core`

### Phase 2: Repository SSOT (Rec 2)
**Goal**: Complete repository layer in core_logic
**Effort**: 2-3 days
**Spec**: `refactor-mapping-repository`

### Phase 3: Utilities (Rec 3)
**Goal**: Shared utilities across CLI/UI
**Effort**: 1 day
**Spec**: `move-utilities-to-core`

### Phase 4: Audit & Cleanup (Rec 4)
**Goal**: Clear separation of concerns
**Effort**: 1-2 days
**Spec**: `audit-ui-services`

---

## Metrics

### Current SSOT Coverage

**Services**:
- In core_logic: 5 services
- Shareable (0 Flutter deps): 37 services
- **Coverage**: 13.5% (5/37)

**Repositories**:
- In core_logic: 1 repository
- Total repositories: 2 repositories
- **Coverage**: 50% (1/2)

**Target After Recommendations**:
- Services: 9/37 = 24% (immediate)
- Repositories: 2/2 = 100% (after Rec 2)

---

## Benefits Summary

### Recommendation 1 (Profile & Test Services)

**Unlocks**:
- ✅ Headless testing in CI/CD
- ✅ CLI profile management for automation
- ✅ Server-side configuration without UI
- ✅ Scripted device setup

**Example Use Case**:
```bash
# CI/CD pipeline
keyrx-cli testScript profile.rhai
keyrx-cli saveProfile production-profile.json
keyrx-cli deploy-config
```

### Recommendation 2 (MappingRepository)

**Unlocks**:
- ✅ CLI access to mapping data
- ✅ Keyflow analysis tools
- ✅ Complete SSOT for repositories

### Combined Impact

**Before**: CLI can only configure (22 commands)
**After**: CLI can test, manage profiles, analyze mappings (30+ commands)

---

## Decision Points

### Should We Proceed?

**Arguments FOR**:
1. **Consistency**: Same pattern as config services (proven)
2. **Testability**: Enable headless testing without UI
3. **Automation**: CI/CD and scripting capabilities
4. **Maintainability**: Single source of truth reduces duplication

**Arguments AGAINST**:
1. **Complexity**: More services to maintain
2. **CLI Scope**: Maybe CLI doesn't need all features
3. **UI-First**: UI is primary interface

**Recommendation**:
- ✅ **Proceed with Rec 1** (Profile & Test) - High value, low effort
- 🤔 **Consider Rec 2** (MappingRepository) - Medium value, medium effort
- ⏸️ **Defer Rec 3 & 4** - Low priority, can wait

---

## Conclusion

Current SSOT implementation is **solid for config management** but has **gaps in profile/test/repository layers**.

**Immediate Action**: Create spec for Recommendation 1 (move Profile & Test services) to unlock headless testing and CLI automation.

**Long-term**: Complete repository layer SSOT by refactoring MappingRepository.

**Status**: Architecture is excellent for current needs, improvements are **optional optimizations** rather than critical gaps.
