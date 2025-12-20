# Requirements: Move Profile & Test Services to Core

**Spec Name**: move-profile-test-services
**Created**: 2025-12-20
**Status**: Draft
**Priority**: High (SSOT completion)

---

## Overview

Move ProfileRegistryService, TestService, and ScriptFileService from UI to core_logic to enable headless testing, CLI profile management, and achieve SSOT for profile/test operations.

---

## Business Goals

1. Enable headless testing in CI/CD pipelines
2. Allow CLI to manage device profiles
3. Support server-side configuration without UI
4. Achieve SSOT for profile and test services

---

## Requirements

### REQ-1: Move ProfileRegistryService to Core

**Description**: Move ProfileRegistryService from UI to core_logic without Flutter dependencies.

**Current State**:
- Location: `ui/lib/services/profile_registry_service.dart`
- Flutter deps: 0 (already pure Dart)
- Used by: UI only

**Target State**:
- Location: `packages/core_logic/lib/services/profile_registry_service.dart`
- Exported from `core_logic.dart`
- Used by: CLI and UI

**Acceptance Criteria**:
- ✅ File moved to core_logic
- ✅ Zero Flutter dependencies verified
- ✅ Exported from core_logic.dart
- ✅ UI imports from core_logic

---

### REQ-2: Move TestService to Core

**Description**: Move TestService from UI to core_logic for headless script testing.

**Current State**:
- Location: `ui/lib/services/test_service.dart`
- Flutter deps: 0 (already pure Dart)
- Used by: UI only

**Target State**:
- Location: `packages/core_logic/lib/services/test_service.dart`
- Exported from `core_logic.dart`
- Used by: CLI and UI

**Acceptance Criteria**:
- ✅ File moved to core_logic
- ✅ Zero Flutter dependencies verified
- ✅ Exported from core_logic.dart
- ✅ UI imports from core_logic

---

### REQ-3: Move ScriptFileService to Core

**Description**: Move ScriptFileService from UI to core_logic for shared script file operations.

**Current State**:
- Location: `ui/lib/services/script_file_service.dart`
- Flutter deps: 0 (already pure Dart)
- Used by: UI only

**Target State**:
- Location: `packages/core_logic/lib/services/script_file_service.dart`
- Exported from `core_logic.dart`
- Used by: CLI and UI

**Acceptance Criteria**:
- ✅ File moved to core_logic
- ✅ Zero Flutter dependencies verified
- ✅ Exported from core_logic.dart
- ✅ UI imports from core_logic

---

### REQ-4: Update UI Imports

**Description**: Update all UI imports to use core_logic package.

**Acceptance Criteria**:
- ✅ ServiceRegistry imports from core_logic
- ✅ All UI screens/widgets import from core_logic
- ✅ No broken imports
- ✅ UI builds successfully

---

### REQ-5: Extend CLI Commands

**Description**: Add CLI commands for profile and test management.

**New Commands**:
```bash
# Profile management
keyrx-cli listProfiles
keyrx-cli getProfile <id>
keyrx-cli saveProfile <json>
keyrx-cli deleteProfile <id>
keyrx-cli findCompatibleProfiles <device-id>

# Testing
keyrx-cli testScript <file>
keyrx-cli validateScript <file>
keyrx-cli runTests <pattern>
```

**Acceptance Criteria**:
- ✅ 8 new CLI commands implemented
- ✅ JSON protocol maintained
- ✅ Error handling graceful
- ✅ All commands tested

---

### REQ-6: Verify and Test

**Description**: Ensure all services work correctly in both CLI and UI.

**Acceptance Criteria**:
- ✅ UI tests pass
- ✅ CLI commands work
- ✅ No Flutter dependencies in core_logic (grep verified)
- ✅ No regressions

---

## Non-Functional Requirements

### NFR-1: Zero Flutter Dependencies

All moved services must have zero Flutter package dependencies.

### NFR-2: Backward Compatibility

UI functionality must remain unchanged.

### NFR-3: Code Quality

All code must pass pre-commit checks (format, clippy, tests).

---

## Success Metrics

- ✅ 3 services moved to core_logic
- ✅ 8 new CLI commands working
- ✅ Zero Flutter deps in core_logic verified
- ✅ UI and CLI both using shared services
- ✅ All tests passing

---

## Out of Scope

- UI wrapper services (not needed - services are already pure Dart)
- Refactoring service logic
- Adding new features to services
