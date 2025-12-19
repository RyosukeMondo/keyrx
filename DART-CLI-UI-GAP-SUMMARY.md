# Dart CLI/UI Gap Analysis & Remediation Plan

**Date**: 2025-12-20
**Status**: Steering documents updated, 3 specs created

## Summary

Comprehensive analysis identified critical gaps between Dart CLI and UI implementations. The DeviceRegistryService refactor established the correct pattern. This plan extends that pattern to all configuration services.

## Steering Documents Updated

✅ **structure.md**:
- Added "Shared Services Architecture (SSOT Pattern)" section
- Documented DeviceRegistry proven pattern
- Defined rules for service placement (core_logic vs UI)
- Updated module boundaries

✅ **tech.md**:
- Updated "Headless Dart Harness Strategy"
- Added Shared Services foundation
- Documented proven pattern reference

## Specs Created

### 1. move-config-services-to-core
**Status**: Ready for approval
**Priority**: HIGH (Quick wins)
**Duration**: 2-3 hours
**Tasks**: 6

**Scope**:
- Move RuntimeService (177 lines) to core_logic
- Move KeymapService (67 lines) to core_logic
- Move ConfigOperationResult shared type
- Update UI imports
- Zero refactoring needed (no Flutter deps)

**Impact**:
- CLI can manage runtime slots
- CLI can manage keymaps
- Establishes pattern for other services

**Files**:
- ✅ `.spec-workflow/specs/move-config-services-to-core/requirements.md`
- ✅ `.spec-workflow/specs/move-config-services-to-core/design.md`
- ✅ `.spec-workflow/specs/move-config-services-to-core/tasks.md`

---

### 2. refactor-flutter-services
**Status**: Ready for approval
**Priority**: MEDIUM
**Duration**: 6-8 hours
**Tasks**: 8

**Scope**:
- Refactor HardwareService: ChangeNotifier → StreamController
- Refactor LayoutService: ChangeNotifier → StreamController
- Move both to core_logic
- Create UI wrappers (HardwareServiceUI, LayoutServiceUI)
- Update ServiceRegistry

**Impact**:
- CLI can manage hardware profiles
- CLI can manage layouts
- Removes all Flutter dependencies from config services
- UI maintains ChangeNotifier compatibility via wrappers

**Files**:
- ✅ `.spec-workflow/specs/refactor-flutter-services/requirements.md`
- ✅ `.spec-workflow/specs/refactor-flutter-services/design.md`
- ✅ `.spec-workflow/specs/refactor-flutter-services/tasks.md`

---

### 3. extend-cli-commands
**Status**: Ready for approval
**Priority**: MEDIUM
**Duration**: 4-6 hours
**Tasks**: 5
**Dependencies**: Specs 1 and 2

**Scope**:
- Add 12 new CLI commands:
  - Runtime: listRuntime, addSlot, removeSlot, activateSlot
  - Hardware: listHardware, saveHardware, deleteHardware
  - Layout: listLayouts, saveLayout, deleteLayout
  - Keymap: listKeymaps, saveKeymap, deleteKeymap
- Maintain JSON protocol
- Comprehensive testing

**Impact**:
- Full headless configuration management
- AI agents can fully automate system
- Complete UAT automation capability

**Files**:
- ✅ `.spec-workflow/specs/extend-cli-commands/requirements.md`
- ✅ `.spec-workflow/specs/extend-cli-commands/design.md`
- ✅ `.spec-workflow/specs/extend-cli-commands/tasks.md`

## Architecture Vision

### Current State (After DeviceRegistry Refactor)
```
packages/core_logic/lib/services/
  └── device_registry_service.dart  ✅ Shared

ui/lib/services/
  ├── runtime_service.dart          ❌ UI only
  ├── keymap_service.dart            ❌ UI only
  ├── hardware_service.dart          ❌ UI only (ChangeNotifier)
  └── layout_service.dart            ❌ UI only (ChangeNotifier)
```

### Target State (After All Specs)
```
packages/core_logic/lib/services/
  ├── device_registry_service.dart  ✅ Shared
  ├── runtime_service.dart          ✅ Shared (Spec 1)
  ├── keymap_service.dart            ✅ Shared (Spec 1)
  ├── hardware_service.dart          ✅ Shared (Spec 2, refactored)
  └── layout_service.dart            ✅ Shared (Spec 2, refactored)

ui/lib/services/
  ├── device_registry_service.dart  ✅ Thin wrapper
  ├── hardware_service_ui.dart      ✅ ChangeNotifier wrapper (Spec 2)
  ├── layout_service_ui.dart        ✅ ChangeNotifier wrapper (Spec 2)
  ├── daemon_service.dart           ✅ UI-specific
  └── generated/ipc_client.dart     ✅ Generated

apps/cli_tool/
  └── Uses ALL shared services      ✅ Full capability (Spec 3)
```

## Implementation Sequence

### Recommended Order

1. **move-config-services-to-core** (2-3 hours)
   - Quick wins, no refactoring
   - Immediate CLI capability boost
   - Proves pattern scales

2. **refactor-flutter-services** (6-8 hours)
   - More complex refactoring
   - Establishes wrapper pattern
   - Completes SSOT for config services

3. **extend-cli-commands** (4-6 hours)
   - Builds on Specs 1 & 2
   - Unlocks full CLI potential
   - Enables complete headless automation

**Total Time**: 12-17 hours (1.5-2 days)

## Success Metrics

### After Spec 1:
- ✅ CLI can manage runtime slots
- ✅ CLI can manage keymaps
- ✅ 2 services in core_logic

### After Spec 2:
- ✅ CLI can manage hardware profiles
- ✅ CLI can manage layouts
- ✅ 4 services in core_logic
- ✅ Zero Flutter dependencies in config services

### After Spec 3:
- ✅ 12 new CLI commands
- ✅ Full headless configuration capability
- ✅ CLI feature parity with UI (for config operations)

## Next Steps

1. Review steering document updates
2. Review all 3 spec documents
3. Approve specs via dashboard
4. Implement in sequence (Spec 1 → 2 → 3)

## Dashboard Access

**URL**: http://localhost:5000

All specs are ready for review and approval.
