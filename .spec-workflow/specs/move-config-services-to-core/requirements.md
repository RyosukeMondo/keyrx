# Requirements: Move Config Services to core_logic

## Overview

Move RuntimeService and KeymapService from `ui/lib/services/` to `packages/core_logic/lib/services/` to establish Single Source of Truth (SSOT) for configuration management. These services have no Flutter dependencies and can be moved immediately without refactoring.

## Context

**Problem**: RuntimeService and KeymapService exist only in UI, preventing CLI from managing runtime slots and keymaps. This violates SSOT principle and limits headless testing capabilities.

**Root Cause**: Services were initially placed in UI during rapid development without considering CLI reusability.

**Impact**:
- CLI cannot manage runtime configuration (adding/removing slots)
- CLI cannot manage keymap operations (list/save/delete)
- Duplicate logic required if CLI needs these features
- Testing requires full Flutter UI instead of lightweight CLI

**Success Criteria**:
- Services moved to `core_logic` with zero functionality changes
- UI continues to work without regressions
- CLI can import and use these services
- All tests pass
- Code metrics maintained (≤500 lines/file, ≤50 lines/function)

## User Stories

### REQ-1: RuntimeService in core_logic
**As a** CLI developer
**I want** RuntimeService available in core_logic
**So that** CLI can manage runtime slots without UI dependencies

**Acceptance Criteria**:
- Service moved to `packages/core_logic/lib/services/runtime_service.dart`
- No Flutter dependencies present
- Exported from `core_logic.dart`
- All existing UI code continues to work
- CLI can import: `import 'package:core_logic/services/runtime_service.dart';`

**EARS Format**:
- **WHEN** CLI needs to manage runtime slots
- **THEN** it can import RuntimeService from core_logic

### REQ-2: KeymapService in core_logic
**As a** CLI developer
**I want** KeymapService available in core_logic
**So that** CLI can manage keymaps without UI dependencies

**Acceptance Criteria**:
- Service moved to `packages/core_logic/lib/services/keymap_service.dart`
- No Flutter dependencies present
- Exported from `core_logic.dart`
- All existing UI code continues to work
- CLI can import: `import 'package:core_logic/services/keymap_service.dart';`

**EARS Format**:
- **WHEN** CLI needs to manage keymaps
- **THEN** it can import KeymapService from core_logic

### REQ-3: UI compatibility maintained
**As a** UI developer
**I want** existing UI code to continue working
**So that** no regressions occur during migration

**Acceptance Criteria**:
- All UI imports updated to use core_logic services
- ServiceRegistry updated to use core_logic services
- All UI tests pass
- No behavioral changes in UI

**EARS Format**:
- **WHEN** UI initializes ServiceRegistry
- **THEN** it uses services from core_logic
- **AND** all functionality works identically

### REQ-4: Result type compatibility
**As a** service consumer
**I want** consistent result types across services
**So that** error handling is uniform

**Acceptance Criteria**:
- RuntimeService uses `ConfigOperationResult<T>` type
- KeymapService uses `ConfigOperationResult<T>` type
- Result types moved to core_logic if needed
- Error messages remain user-friendly

**EARS Format**:
- **WHEN** a service operation fails
- **THEN** it returns ConfigOperationResult with error message

## Non-Functional Requirements

### NFR-1: Zero Dependencies on Flutter
**Performance**: Services must run on bare Dart VM
**Rationale**: Enables CLI usage without Flutter engine
**Measurement**: `flutter pub deps` shows no Flutter deps in service files

### NFR-2: Backward Compatibility
**Compatibility**: UI code continues to work without changes
**Rationale**: Minimize disruption during migration
**Measurement**: All existing UI tests pass

### NFR-3: Code Quality Standards
**Maintainability**: Follow existing code metrics
**Rationale**: Maintain codebase quality
**Measurement**:
- ≤500 lines per file
- ≤50 lines per function
- 80% test coverage minimum

## Dependencies

### Upstream Dependencies
- `core_logic` package structure exists
- `KeyrxBridge` available in core_logic
- Models (RuntimeConfig, Keymap, HardwareProfile) in core_logic

### Downstream Impact
- UI ServiceRegistry must be updated
- UI imports must be updated
- CLI CommandProcessor can add new commands (future work)

## Success Metrics

- ✅ `packages/core_logic/lib/services/runtime_service.dart` exists
- ✅ `packages/core_logic/lib/services/keymap_service.dart` exists
- ✅ Zero Flutter dependencies in moved services
- ✅ All UI tests pass (100%)
- ✅ Services exported from `core_logic.dart`
- ✅ UI ServiceRegistry uses core_logic services
- ✅ Code metrics maintained

## Out of Scope

- Refactoring service internals (move only)
- Adding new CLI commands (separate spec: extend-cli-commands)
- Moving HardwareService or LayoutService (separate spec: refactor-flutter-services)
- Changing FFI bindings or data models
- Performance optimizations
